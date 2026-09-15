# Folia Lyrics: SMTC Media Listener & Lyric Engine Design

<p align="center">
  <strong>English</strong> | <a href="../../../proposals/folia-lyrics/smtc-and-lyrics-engine.md">简体中文</a>
</p>

This document designs how the widget observes the Windows Global System Media Transport Controls (SMTC) session, and defines the word-level lyric parsing and high-precision timeline synchronization mechanisms.

---

## 1. Windows SMTC Listener Architecture

Windows 10 and 11 provide the native `GlobalSystemMediaTransportControlsSessionManager` (WinRT API). Virtually all mainstream audio players integrate with SMTC (Spotify, Apple Music, Netease Cloud Music, QQ Music, Foobar2000, and Chrome/Edge media tabs).

```
+--------------------------------------------------------------+
| Active Media Players (Spotify, Apple Music, Netease, etc.)   |
+--------------------------------------------------------------+
                               ↓ (OS Media Pipeline Broadcast)
+--------------------------------------------------------------+
| Windows SMTC Session Manager (WinRT)                         |
+--------------------------------------------------------------+
                               ↓ (Async Channels & Event Hooks)
+--------------------------------------------------------------+
| SmtcListener (widget-rs Background Observer)                 |
| - Session Binding: SessionChanged                            |
| - Metadata: MediaProperties (Title, Artist, Album, Art)      |
| - State: PlaybackInfo (Playing, Paused, Timeline)            |
+--------------------------------------------------------------+
                               ↓ (Normalized Media Event Stream)
+--------------------------------------------------------------+
| Lyric Synchronization State Machine (LyricsTimelineEngine)   |
+--------------------------------------------------------------+
```

---

## 2. Timeline Monitoring & Monotonic Clock Extrapolation

### 2.1 The Need for Timeline Extrapolation
Windows SMTC does not push playback timeline updates every frame; updates occur roughly once per second or only on pause/seek events. Relying directly on static SMTC timestamps causes lyric animations to stutter and jump.

### 2.2 Local High-Precision Extrapolation Formula
Let:
- $T_{base}$: Track playback position reported by SMTC during the last sync.
- $t_{sync}$: System monotonic timestamp when the update was received (`std::time::Instant::now()`).
- $S$: Current playback speed rate (typically 1.0).

At any frame rendering moment $t_{render}$:
$$\text{CurrentPosition} = T_{base} + (t_{render} - t_{sync}) \times S$$

When receiving SMTC `TimelinePropertiesChanged` or `PlaybackInfoChanged`, $T_{base}$ and $t_{sync}$ are recalibrated. If the discrepancy exceeds 500ms (indicating a user seek), the current active lyric line is reset and an elastic camera jump is triggered.

---

## 3. Lyric Parsing Engine Specification

### 3.1 Format Support & Priority
1. **YRC (Yamaha / Netease Word-by-Word Format)**:
   - Syntax: `[1234,4560](1234,200,0)word1(1434,300,0)word2...`
   - Supplies absolute start timestamps, durations, and tone pitch data.
2. **QRC (QQ Music Word-by-Word Format)**:
   - Syntax: `[1234,4560]word1(200)word2(300)...`
3. **LRC (Standard Line-Level Format)**:
   - Syntax: `[00:12.34]Full lyric line text here`
   - For standard LRC, smooth sweeps are generated based on character count and phoneme estimation.

### 3.2 Unified Internal Data Model
```rust
/// Precise timestamp window for individual words (drives spring pops)
#[derive(Clone, Debug, PartialEq)]
pub struct WordTiming {
    pub text: String,
    pub start_ms: u32,
    pub duration_ms: u32,
}

/// Single lyric line (with weights and interlude flags)
#[derive(Clone, Debug, PartialEq)]
pub struct LyricLine {
    pub line_index: usize,
    pub start_ms: u32,
    pub duration_ms: u32,
    pub raw_text: String,
    pub words: Vec<WordTiming>,
    pub translation: Option<String>,
    pub is_highlight: bool,
}

/// Full song lyrics object
#[derive(Clone, Debug, Default)]
pub struct TrackLyrics {
    pub track_title: String,
    pub artist: String,
    pub lines: Vec<LyricLine>,
    pub is_word_by_word: bool,
    pub estimated_bpm: u32,
}
```

---

## 4. Lyric Retrieval & Caching Pipeline

When SMTC detects a track switch, a background async worker initiates lyric matching:

1. **Two-Tier Local Cache Check**:
   - Computes tag hash: `hash(Title + " " + Artist)`.
   - Reads from `%APPDATA%/tierlabx/widget-rs/lyrics_cache/<hash>.json`.
2. **Online Fallback Chain**:
   - **Step 1 (Word-Level Sources)**: Queries Netease/QQ Music APIs for YRC/QRC word-by-word data.
   - **Step 2 (Public Open Repositories)**: If not found, queries LrcLib for standard LRC lines.
   - **Step 3 (Instrumental Mode)**: If no lyrics are found, switches UI to "Instrumental Flow Stage", showing animated album art, soundwave visuals, and track metadata.
3. **Debouncing & Cancellation**:
   - Uses `CancellationToken` to cancel outstanding network requests upon rapid track switching, preventing out-of-order race condition overwrites.
