## ChromaCat Playground Vision — SilkCircuit Edition ✨

### Electric meets elegant

ChromaCat evolves into a console-native creative coding playground: fast, expressive, accessible. It keeps the current clean core (patterns → engine → buffer → terminal) and layers a live playground on top for composing motion, color, and text art with grace.

This vision is powered by the SilkCircuit design language: high-energy accents with refined calm, clear semantics, and uncompromising readability.

---

### North Star

- **Expressive**: Paint with algorithms. Tweak live. Save the moment.
- **Performant**: Smooth 60–144 FPS where terminals allow; zero flicker.
- **Accessible**: WCAG-aware palettes, keyboard-first, readable at a glance.
- **Composable**: Patterns, params, themes, layers, transitions.
- **Portable**: One binary. No daemons. No fuss.

---

### Core pillars

- **Precision pipeline**: Deterministic frame step, double-buffer rendering, region diffs.
- **Pattern craft**: Clean param models; normalized coords; consistent semantics.
- **Live control**: Key-driven sliders, pickers, and a compact TUI overlay.
- **Playlists → Performances**: Curate sequences with keyframes and transitions.
- **Exports**: Deterministic WebP/GIF/PNG with the same engine you see live.

---

### Product focus: Animation-first

- **Primary**: Real-time animation and creative coding playground with endless evolution (patterns, themes, layers, modulation, transitions). This is the core product.
- **Secondary (utility)**: Simple file colorizer remains as a "Static Mode" for quick one-offs and CI-friendly outputs.
- **CLI/UX**:
  - Default launch goes to animation/playground. A `colorize` subcommand (or `--static`) preserves the old workflow.
  - Export flows are first-class from animation: record deterministic segments to WebP/GIF/PNG.
- **Docs/tests**: Examples, snapshots, and previews emphasize motion-first usage. Static examples remain, clearly marked.
- See also: [playground-architecture.md](./playground-architecture.md) for the ratatui loop and evolution system.

---

### SilkCircuit infusion

- **Variants**: `neon`, `vibrant`, `soft`, `glow` map to terminal-safe palettes. Use variant-specific accents for UI chrome (status bar, pickers) and retain art fidelity for content.
- **Semantic accents**:
  - Keywords/controls (UI labels, hotkeys) → Purple family.
  - Interactive focus (active widget, selection) → Cyan glow.
  - Warnings (overdrive FPS, high CPU) → Yellow.
  - Success (saved/exported) → Green.
  - Errors (invalid params, parse) → Red.
- **Typography cues**: Bold for active value, italic for hints/help. Keep concise.
- **Motion feel**: Ease-out for UI transitions, linear for scrubs; subtle pulses only when useful.
- **Accessibility**: All four variants meet contrast; the status bar always resolves to AA.

---

### Experience flow

- **Default: Playground animation**: Start in animation mode with a compact HUD. Press `;` to toggle the overlay.
  - Left: Pattern browser (search, preview, apply)
  - Center: Parameter sliders (focused param shows coarse/fine controls)
  - Right: Theme picker (categories + preview chips)
  - Bottom: Transport (play/pause, prev/next, export)

- **Static colorizer (secondary)**: One-shot colorization of files/stdin with the same themes. Minimal flags, CI-friendly output.

Controls (proposed, additive):
- `[ / ]` select param • `- / =` tweak • `Shift` = coarse • `Alt` = fine
- `t` theme cycle • `g` theme picker • `p` pattern cycle • `, / .` prev/next param
- `Space` pause playlist • `← / →` playlist step • `r` record • `?` help

---

### System architecture (incremental)

- **Renderer (keep)**: Frame loop, buffer, status bar, scroll.
- **Playground UI (new)**: State machine rendered into bottom rows; event-routed via `renderer.handle_key_event` without touching the outer loop.
- **Modulation engine (new)**: LFOs/noise/envelopes/steps that target any numeric param each frame before color update.
- **Reactive inputs (optional features)**: Audio (CPAL+FFT), MIDI (midir), OSC (rosc) as modulation sources.
- **Composition (later)**: Layer stack with blend modes and per-layer opacity.
- **Exporter (extend)**: Deterministic fixed-step encoder for WebP (exists), GIF, PNG strips.

---

### Pattern design language

- **Param semantics**: Prefer `frequency`, `amplitude`, `scale`, `complexity`, `density`, `speed` with consistent ranges. Include sane defaults.
- **Time discipline**: All animation originates from engine time; no ad-hoc clocks.
- **Value range**: Return normalized \([0,1]\) for color mapping stability.
- **Performance hygiene**: Avoid per-pixel heap allocs; prefer precomputed constants; consider LUTs for expensive trig.
- **Aesthetic guidelines (SilkCircuit)**: Avoid strobe >20 Hz; respect variant tone; keep motion purposeful.

---

### Roadmap (phased)

1) **Phase 1 — TUI overlay**
   - Pattern browser, parameter sliders, theme picker, minimal transport.
   - Export panel: record N seconds to WebP; add GIF/PNG.

2) **Phase 2 — Modulation**
   - LFO, noise, step sequencer; route to params with depth/offset.
   - Macro knobs mapping to multiple params.

3) **Phase 3 — Reactive**
   - Audio envelope/bands; MIDI CC learn; OSC endpoints.

4) **Phase 4 — Composition**
   - Layers + blend modes; transitions between playlist entries.

5) **Phase 5 — Scripting**
   - Rhai/Lua for custom param functions and light patterns (sandboxed).

6) **Phase 6 — Performance**
   - Parallel color updates of visible rows; gradient LUTs; fixed timestep.

---

### UI micro-spec (silkcircuit-aware)

- **Status bar**: `theme • pattern • FPS` on left, controls hint center, `Lines a-b/N • Quit` right.
  - Colors: text=#abb2bf; separators=#282c34; accent=#61afeF; obey variant contrast.
- **Focus**: Cyan glow on active control. No blinking cursors.
- **Help overlay (`?`)**: Single screen with grouped hotkeys, minimal prose.

---

### Contributing

Adding a pattern:
1. Implement generation in `src/pattern/patterns/your_pattern.rs` returning \([0,1]\).
2. Define params in `params.rs` and register in `registry.rs` with an ID and defaults.
3. Add tests (continuity, bounds, snapshot under seeded time).

Adding a theme:
1. Extend YAML in `themes/*.yaml` with SilkCircuit-aligned stops.
2. Ensure AA contrast in all variants; run preview generator.
3. Tag category for discoverability.

---

### Success criteria

- Launches with `--playground` and feels instantaneous.
- Users compose a 3-entry performance in < 2 minutes, no docs.
- Exports match live frames 1:1.
- No eyestrain in `soft`, full pop in `glow`.

---

### Appendix: Reference snippets

Pattern dispatch (normalized):

```rust
// See src/pattern/patterns/mod.rs
pub fn generate(&self, x: usize, y: usize, params: &PatternParams) -> f64 {
    let (x_norm, y_norm) = self.normalize_coords(x, y);
    match params { /* variants → f64 in [0,1] */ }
}
```

Render loop hook points:

```rust
// See src/renderer/mod.rs
pub fn render_frame(&mut self, text: &str, dt: f64) -> Result<(), RendererError> {
    if let Some(player) = &mut self.playlist_player {
        if player.update(Duration::from_secs_f64(dt)) { self.update_playlist_entry()?; }
    }
    if !self.buffer.has_content() { /* init */ return Ok(()); }
    // 1) modulation.apply(&mut self.engine.config)
    self.engine.update(dt);
    // 2) buffer.update_colors(&self.engine, visible.start)
    // 3) render region + status bar
    Ok(())
}
```

SilkCircuit palette anchors (by variant):

```
- neon:    purple #e135ff, cyan #80ffea, pink #ff99ff, yellow #f1fa8c
- vibrant: magenta #ff00ff, cyan #00ffcc, yellow #ffcc00
- soft:    purple #e892ff, cyan #99ffee, yellow #ffe699
- glow:    magenta #ff00ff, cyan #00ffff, yellow #ffff00
```

— Let’s build something beautiful. 🛠️⚡


