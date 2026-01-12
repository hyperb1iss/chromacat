## ChromaCat Playground — Build Plan and Checklist ✅

### Goals

- Ship an animation-first creative coding playground with endless evolution.
- Keep current rendering core; add ratatui overlay UI and modulation.
- Remain deterministic and export-identical to live output.

### Success criteria

- `--playground` launches instantly; smooth 60 FPS on typical terminals.
- Users can browse patterns, tweak params, swap themes, and export.
- A single LFO modulates a param live; Scene/Layer types exist.
- Minimal scheduler transitions scenes deterministically; exports match.

---

### Milestone 0 — Mode switch and scaffolding

- [ ] Add `--playground` flag and mode selection (default: playground)
- [ ] Keep static colorizer as `colorize` subcommand or `--static`
- [ ] Ensure raw mode/alternate screen lifecycle stays clean

Acceptance
- Running `chromacat --playground` shows current render with status HUD
- `chromacat colorize README.md` still works (no regressions)

---

### Milestone 1 — Ratatui overlay (compact UI)

- [ ] Integrate `ratatui` event/render loop with fixed-timestep ticker
- [ ] Overlay toggle `;` and help `?`
- [ ] Pattern Browser: list + apply (preserve params when possible)
- [ ] Parameter Sliders: `[ / ]` select, `- / =` tweak, `Shift`/`Alt` coarse/fine
- [ ] Theme Picker: categories + chips, live preview

Acceptance
- Overlay renders over content without flicker
- All listed keybinds work; resize redraws correctly

---

### Milestone 2 — Control surface and engine hooks

- [ ] `RendererControl` API to set pattern/theme/params safely
- [ ] Wire overlay widgets to `RendererControl`
- [ ] Deterministic fixed-timestep option (accumulator)

Acceptance
- Changing UI controls updates frames immediately and stably

---

### Milestone 3 — Modulation v1

- [ ] Introduce `ModSource` and `ModRoute` types
- [ ] Implement `Lfo` source (sine/triangle), depth/offset/shape
- [ ] Apply routes pre-`engine.update` each frame

Acceptance
- A visible param (e.g., `frequency`) modulates smoothly via LFO

---

### Milestone 4 — Scene types and minimal scheduler

- [ ] Add `Scene`, `Layer`, `BlendMode` structs
- [ ] Crossfade transition between scenes
- [ ] Minimal duration-based scheduler (no AI yet)

Acceptance
- Scene A→B crossfades deterministically at set durations

---

### Milestone 5 — Export panel

- [ ] Deterministic encoder integration for WebP (existing), GIF, PNG strip
- [ ] Record N seconds from live with exact match

Acceptance
- Exported frames equal live render (bitwise gradient mapping)

---

### Milestone 6 — Novelty metrics and logs

- [ ] Value histogram distance (Wasserstein) over [0,1]
- [ ] Temporal/spectral simple metrics and strobe guard
- [ ] Log metrics per scene; expose in debug overlay

Acceptance
- Metrics update in real time; no perf regression

---

### Milestone 7 — Recipe and provenance

- [ ] `Recipe { seed, scenes, schedule, constraints }`
- [ ] Save/load snapshot with recent items menu

Acceptance
- Reloading a recipe reproduces animation and exports 1:1

---

### Milestone 8 — AI Muse (feature-flagged)

- [ ] Prompt builder with registry + scene summary
- [ ] Proposal validation/clamping + guards (contrast/flash/CPU)
- [ ] Provenance store (prompt, response, registry hashes)

Acceptance
- AI suggestions occasionally accepted; always reproducible; never jarring

---

### SilkCircuit integration (UI polish)

- [ ] Variant-aware status/overlays (neon/vibrant/soft/glow)
- [ ] Accents: purple controls, cyan focus, yellow caution, green success, red errors
- [ ] AA contrast verified for all UI text

---

### Risks and mitigations

- Flicker with ratatui: keep double-buffered region writes for content area
- Input starvation: separate fixed-timestep from input poll
- Performance dips: gradient LUT and visible-row parallel chunking

---

### Definition of done (initial release)

- M0–M5 complete; M6 logging present; recipe save/load functional
- Docs updated; help overlay complete; tests stable; CI exports deterministic


