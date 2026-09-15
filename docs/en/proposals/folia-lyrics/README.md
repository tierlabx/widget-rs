# Proposal PROP-001: Immersive Desktop Folia Lyrics

<p align="center">
  <strong>English</strong> | <a href="../../../proposals/folia-lyrics/README.md">简体中文</a>
</p>

> **Status**: Under Review (RFC / Architecture Review)  
> **Inspiration**: [chthollyphile/folia-major](https://github.com/chthollyphile/folia-major) (high-fidelity animated lyric player) and Typography Music Videos (PV)  
> **Target Architecture**: Rust + GPUI Native GPU Rendering Engine  

---

## 1. Motivation & Product Positioning

### 1.1 Problem Statement
Desktop music lyric displays typically fall into two extremes:
- **Overly Spartan**: A single semi-transparent floating line or rigid LRC scroll without rhythm dynamics or stage immersion.
- **Bloated Runtimes**: Electron/WebGL implementations (such as original Folia-Major) consume 350MB–600MB of resident RAM, hide when `Win+D` (Show Desktop) is pressed, and fail to blend naturally into the desktop wallpaper.
- **Lack of Emotional Impact**: Most solutions limit animation to simple left-to-right color transitions, lacking the dynamic typography, kinetic word bounces, and depth-of-field found in Typography PVs.

### 1.2 Core Product Positioning
Combines typography motion design, Folia-Major's fluid stage presence, and `widget-rs`'s lightweight desktop integration:
- **Typography PV Motion Aesthetics**:
  - **Per-Word Spring Pop**: Each word's onset triggers an elastic scale pop and subtle vertical drift, delivering tactile rhythm impact.
  - **Dynamic Stage Typography**: Active line magnifies at the visual center, preceding lines drift upward with reduced opacity, and upcoming lines glow softly; interlude sections transition smoothly into soundwave rhythms.
  - **Dual-Layer Neon Glow**: Saturated highlight colors paired with 12px dynamic blurred diffusion.
- **Ultra-Lightweight**: Built on GPUI + DirectComposition hardware acceleration; resident memory capped at **30MB–50MB**, CPU idle <0.3%, supporting 120Hz refresh rates.
- **Dual-Mode Architecture (Listener + Standalone)**:
  - **Global SMTC Listener (Primary)**: Zero configuration, no login required. Detects playing tracks and progress across Spotify, Apple Music, Netease, QQ Music, Foobar2000, and web browsers, fetching synchronized lyrics automatically.
  - **Standalone Mode (Advanced)**: Drag-and-drop local audio files (.flac, .mp3, .wav) directly into the widget for local playback.
- **Native Desktop Integration**: Anchored to `Progman` for persistent desktop residency on `Win+D`, paired with DirectComposition hardware transparency.

---

## 2. Feature Matrix

| Domain | Feature Description |
| :--- | :--- |
| **Typography PV Visuals** | Full-screen kinetic stage mode, compact single-line karaoke, desktop vinyl mode, wallpaper glass mode |
| **Per-Word Spring Pop** | Millisecond-accurate word parsing (YRC/QRC/TTML) triggering a $1.0 \to 1.25 \to 1.0$ damped spring bounce |
| **Dynamic Typography** | Active line magnification, depth-of-field attenuation, parallax drift, and interlude waveforms |
| **Dual-Layer Glow** | Subtle breathing glow on unsung words, saturated highlights, and 12px soft luminous glow transitions |
| **Spring Dynamics** | Line transitions driven by an underdamped spring ($k=240.0, d=22.0$) for smooth repositioning without jumps |
| **Fluid Adaptive Stage** | Three-color palette extracted from album art driving a 3-axis Lissajous diffuse lighting stage |
| **System Media Sync** | Windows SMTC WinRT API, monotonic clock timeline extrapolation, and bidirectional playback controls |
| **Smart Lyric Sources** | Local cache + multi-source fallback (Netease YRC per-word priority / LrcLib) |
| **Desktop Compliance** | Strict `WidgetWindow<T>` encapsulation, Progman Win+D persistence, single file <= 400 lines, modal settings shell |

---

## 3. Proposal Navigation

This proposal is modularized into the following design specifications:

- [Architecture & GPUI Motion Rendering Pipeline](./architecture-and-rendering.md)
  - Details the kinetic typography pipeline, per-word bounce formulas, dual-layer neon glow, spring transitions, and fluid backdrop.
- [SMTC Media Listener & Lyric Engine](./smtc-and-lyrics-engine.md)
  - Details Windows SMTC WinRT asynchronous listeners, monotonic clock extrapolation, and YRC/QRC word-level parsers.
- [Window Hosting & System Integration](./window-and-integration.md)
  - Details full-screen stage vs. windowed modes, `Progman` anchoring, DirectComposition transparency, and settings schemas.

---

## 4. Implementation Milestones

### Phase 1: Lyric Parsing & Core Layout
- [ ] Build `YrcParser` and `LrcParser` supporting line-level and millisecond word-level timestamps with translations.
- [ ] Implement basic GPUI lyric list element: active line highlighting, auto-centering scroll, top/bottom gradient masks.
- [ ] Integrate monotonic clock extrapolation to eliminate SMTC timeline jitter.

### Phase 2: Typography PV Pipeline & Full-Screen Stage
- [ ] Implement per-word bounce algorithm ($1.0 \to 1.25 \to 1.0$ scale pop and subtle vertical drift).
- [ ] Implement dual-layer text masks and neon glow in custom GPUI canvas elements.
- [ ] Build kinetic typography engine (focal magnification, depth-of-field decay, spring line-change).
- [ ] Build album palette extraction and 3-axis Lissajous diffuse stage background.

### Phase 3: Windows SMTC Global Capture & Lyric Fetching
- [ ] Implement `GlobalSystemMediaTransportControlsSessionManager` via `windows` crate.
- [ ] Extract real-time album art, title, artist, playback status, and timeline updates.
- [ ] Implement multi-source lyric fetching with local caching (Netease YRC per-word / LrcLib).

### Phase 4: Desktop Polish & Framework Integration
- [ ] Encapsulate into `WidgetWindow<T>` container, supporting full-screen and windowed desktop modes with `Progman` pinning.
- [ ] Build standard settings modal using `render_settings_shell` (animation intensity, stage modes, typography size).
- [ ] Run `cargo clippy`, `cargo fmt`, and CPU/memory benchmarks; maintain files under 400 lines.
