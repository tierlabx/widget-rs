# Folia Lyrics: Architecture & GPUI Typography PV Rendering Pipeline

<p align="center">
  <strong>English</strong> | <a href="../../../proposals/folia-lyrics/architecture-and-rendering.md">简体中文</a>
</p>

This document details how to construct a **Typography Music Video (PV)**-grade full-screen and desktop lyric animation pipeline natively in `gpui`, including per-word spring impact pops, dynamic camera depth-of-field typography, dual-layer neon glow masks, and fluid stage backdrops.

---

## 1. Overall Rendering Architecture & Layering

The visual stack is composed of five layers from bottom to top:

```
+-------------------------------------------------------------+
| Layer 5: Controls & HUD (SMTC status / track info / buttons)|
+-------------------------------------------------------------+
| Layer 4: Typography PV Motion Core (Per-word pops & layout) |
+-------------------------------------------------------------+
| Layer 3: Neon Glow & Sweep Mask (Dual-layer glow & particles)|
+-------------------------------------------------------------+
| Layer 2: Adaptive Fluid Stage (Dynamic Gaussian diffuse)    |
+-------------------------------------------------------------+
| Layer 1: Backdrop Vignette & Transparency (DirectComposition)|
+-------------------------------------------------------------+
```

---

## 2. Core Algorithm 1: Typography PV Per-Word Spring Pop & Drift

Traditional lyric software simply performs mechanical left-to-right color wipes, lacking tactile rhythm impact. In our typography pipeline, each word behaves as a particle with physical tension.

### 2.1 Per-Word Time Window & Elastic Impact Model
For a word-level synchronized lyric (YRC/QRC):
```json
{
  "text": "Immersed in every note on the desktop",
  "words": [
    { "word": "Immersed", "start_ms": 1200, "duration_ms": 400 },
    { "word": "in",       "start_ms": 1600, "duration_ms": 250 },
    { "word": "every",    "start_ms": 1850, "duration_ms": 500 }
  ]
}
```

Let current playback time be $T$. For the word starting at $T_{start}$, the relative onset time is $\Delta t = (T - T_{start})$ (in seconds).

#### 1. Scale Pop
During the initial 180ms of word vocalization, an underdamped spring bounce triggers:
$$Scale(\Delta t) = 1.0 + 0.26 \cdot e^{-16 \Delta t} \cdot \cos(24 \Delta t)$$
- **Impact Velocity**: The font scales up to $1.26\times$ on onset, producing a crisp percussive punch.
- **Smooth Settlement**: Snaps back smoothly to $1.0\times$ within 180ms.

#### 2. Upward Floating (Y-Drift & Lift)
During vocalization, the word ascends by $4\text{px}$ against gravity, settling back smoothly at word completion to evoke resonant vocal projection.

---

## 3. Core Algorithm 2: Dynamic Dual-Layer Neon Glow Mask

### 3.1 Masking Pipeline
During GPUI custom element rendering (`impl Element for TypographyPvElement`):
1. **Base Layer (Unsung Text - Base Track)**:
   - Renders semi-transparent off-white static text (`rgba(255, 255, 255, 0.38)`) with a subtle low-frequency breathing glow (alpha cycling between $0.35 \sim 0.45$).
2. **Active Layer (Sung Text - Active Neon Glow Track)**:
   - Uses high-saturation dominant colors extracted from album art. Rendered across the active bounding box $[0, X_{progress}]$ via `cx.with_mask(ContentMask { bounds: active_rect })`.
   - **Edge Glow Diffusion**: A 14px radial-linear gradient feathering layer is blended at the sweeping frontier, eliminating abrupt hard cuts.

---

## 4. Core Algorithm 3: Camera & Typography Space Dynamics

### 4.1 Golden Ratio Focus & Parabolic Depth of Field
In full-screen and expanded modes, multiple lyric lines are displayed simultaneously:
- **Active Focus Line**:
  - Positioned at the vertical golden ratio (approx $46\%$ viewport height), scaled up (36px–48px), bold weight, $1.0$ opacity, with kinetic word pops.
- **Historical Lines**:
  - Drift upward with stepwise font scaling ($24\text{px} \to 18\text{px}$) and exponential opacity decay ($0.4 \to 0.15$), creating depth recession.
- **Upcoming Lines**:
  - Rest softly below at medium font size ($22\text{px} \to 16\text{px}$) with reduced opacity ($0.3 \to 0.1$).

### 4.2 Underdamped Spring Line Transitions
Transitions between lines avoid stepped jumps:
$$F = -k(y - y_0) - c \cdot v$$
- **Parameters**: Stiffness $k = 240.0$, Damping $c = 22.0$.
- When advancing to the next line, the camera elevates smoothly as previous lines arc upward and the new line snaps elastically into the focal zone.

### 4.3 Instrumental Interlude Protection
During long instrumental breaks (>4 seconds without lyrics):
- Transitions automatically into **Instrumental Rhythm Mode**: lyrics fade out and a pulsing soundwave orb awakens at the center to keep the stage visually alive.

---

## 5. Core Algorithm 4: Adaptive Fluid Stage Backdrop

### 5.1 Album Art Palette Extraction
- Dynamically extracts 3 representative tones: Dominant, Accent, and Muted Dark.
- Clamps luminance and saturation in HSL space to preserve lyric contrast.

### 5.2 Lissajous Light Orbs
- Custom Canvas renders 3 floating elliptical gradient diffuse light sources drifting along low-frequency sine/cosine curves.
- Subtle breathing scale pulses ($\pm 6\%$) respond to musical beat pulses.
- Radial dark vignette around boundaries guarantees peak text legibility.

---

## 6. Performance & Power Optimization

1. **Zero-CPU Idle**: When paused for >3 seconds, animation loops drop to 0 FPS, entering an event-driven sleep state.
2. **Dirty Rect Repainting**: Only the active line and ambient glow are repainted each frame; inactive lines reuse layout caches.
3. **Memory Footprint**: Palette computation runs asynchronously on background threads; resident memory remains strictly between 30MB–50MB.
