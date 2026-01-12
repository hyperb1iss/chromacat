## ChromaCat Playground Architecture — Infinite Evolution, Ratatui-ready ⚡

### Purpose

This document defines the architecture for an endless creative-coding playground, including a `ratatui`-based UI loop, a composable Scene/Modulation system, a scheduler/evolver for continuous variation, and optional AI-assisted evolution. It complements `docs/playground-vision.md` with implementation-ready structure.

---

### High-level system

- Renderer: double-buffered region rendering to the terminal, fixed timestep option.
- SceneGraph: layers of patterns with blend modes and optional masks.
- ModulationGraph: time-based sources (LFO, noise, envelopes, step sequencer, inputs) routed to params.
- Scheduler: decides transitions and large-scale changes; manages durations and playlists.
- Evolver: proposes new params/scenes (heuristic, genetic, or AI-guided) with guardrails.
- ContentRegistry: patterns, themes, transitions, and metadata.
- Exporter: deterministic stepping for WebP/GIF/PNG.
- Provenance: seed + diffs captured as a Recipe for perfect reconstruction.

---

### Data model (core types)

Pattern contract

```rust
pub trait Pattern {
    fn id(&self) -> &'static str;
    fn default_params(&self) -> PatternParams;
    fn generate(&self, x: f64, y: f64, time: f64, params: &PatternParams) -> f64; // [0,1]
    fn metadata(&self) -> PatternMeta; // name, tags, ranges, semantics
}
```

Param metadata (derive)

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PatternParamsMeta)]
pub struct PlasmaParams {
    #[param(range = "0.05..8.0", semantic = "frequency")] pub frequency: f64,
    #[param(range = "0.1..2.0",  semantic = "amplitude")] pub amplitude: f64,
    #[param(range = "0.0..1.0",  semantic = "complexity")] pub complexity: f64,
}
```

Scene graph

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BlendMode { Add, Screen, Multiply, Overlay }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layer {
    pub pattern: String,
    pub params: PatternParams,
    pub theme: String,
    pub opacity: f32,
    pub blend: BlendMode,
    pub mask: Option<Mask>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub layers: Vec<Layer>,
    pub routes: Vec<ModRoute>,
    pub duration: std::time::Duration,
    pub transition_out: TransitionSpec,
}
```

Modulation graph

```rust
pub trait ModSource { fn id(&self) -> &'static str; fn next(&mut self, t: f64) -> f64; /* [-1,1] */ }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModRoute {
    pub source_id: String,
    pub target: ParamAddress, // layer/pattern/param path
    pub depth: f64,
    pub offset: f64,
    pub shape: ModShape, // linear, exp, smoothstep
}
```

Transitions and recipes

```rust
pub trait Transition { fn apply(&self, a: &FrameField, b: &FrameField, t: f32) -> FrameField; }

#[derive(Serialize, Deserialize)]
pub struct Recipe {
    pub seed: u64,
    pub scenes: Vec<Scene>,
    pub schedule: SchedulePolicy,
    pub constraints: SafetyConstraints,
    pub inputs: Inputs,
}
```

---

### Ratatui migration plan (UI loop rewrite)

Goal: adopt `ratatui` for a structured, composable UI while preserving the extant rendering pipeline and performance.

Key points
- Keep terminal backend `crossterm` but let `ratatui` own the UI/layout and input loop.
- Render the animated content into a dedicated panel; status bar and overlays become widgets.
- Maintain fixed timestep ticker separate from input/events for consistent animation.

Event/render loop pseudocode

```rust
struct App { ui_state: UiState, scene: Scene, modulation: ModulationEngine, scheduler: Scheduler, /* ... */ }

fn main() -> Result<()> {
    // Setup ratatui terminal
    let mut terminal = ratatui::Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let tick_rate = Duration::from_millis(16); // ~60 FPS
    let mut last_tick = Instant::now();

    loop {
        // 1) Draw UI
        terminal.draw(|frame| {
            let layout = Layout::default().direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(2)])
                .split(frame.size());

            // Content panel (animation)
            render_animation_panel(frame, layout[0], &mut app)?; // calls renderer to draw region

            // Status bar + overlays
            render_status_and_overlays(frame, layout[1], &app.ui_state);
        })?;

        // 2) Handle input non-blocking
        if crossterm::event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                if !handle_key(&mut app, key)? { break; }
            }
        }

        // 3) Fixed timestep update
        let now = Instant::now();
        while now.duration_since(last_tick) >= tick_rate {
            let dt = tick_rate.as_secs_f64();
            step_animation(&mut app, dt)?; // modulation → engine.update(dt) → buffer.update_colors
            last_tick += tick_rate;
        }
    }
    Ok(())
}
```

Rendering strategy
- Use our existing double-buffer and region-diff; pipe ANSI writes into ratatui via a custom widget that writes directly to the backend buffer, or keep direct stdout writes for the content area coordinates.
- Keep color handling as-is (24-bit ANSI); ensure `ratatui` does not reset colors unexpectedly.

UI components
- Pattern Browser: list/grid with search; preview thumbnails rendered via reduced-resolution pattern snapshots.
- Parameter Sliders: focused param with fine/coarse stepping.
- Theme Picker: category tabs + chips; live preview.
- Transport/Status: FPS, scene name, progress, hints (SilkCircuit accents).

Keybindings (additive)
- `;` toggle overlay, `?` help, `t/g` theme cycle/picker, `p` pattern cycle.
- `[ / ]` select param, `- / =` tweak, `Shift` coarse, `Alt` fine.
- `Space` pause, `←/→` scene step, `r` record.

Risks and mitigations
- Flicker: preserve direct buffered writes for content; avoid full-screen clears.
- Input starvation: split fixed-timestep from input poll; keep poll at 1–2 ms.
- Resize handling: recompute layout, resize buffers, redraw full screen.

---

### Scheduler and Evolver

Schedule policy
- Min/max scene duration; pattern/theme category weights; allowed transition set.

Novelty metrics
- Value histogram distance (Wasserstein) over [0,1].
- Spectral energy bands over time (movement character).
- Temporal variance; strobe guard (≤ 20 Hz).

Evolver strategies
- Heuristic drift along smooth param manifolds.
- Genetic sampler with small population; select by novelty + constraints.
- AI Muse (LLM-guided): propose Scene diffs; validate, clamp, and accept if safe.

Guardrails
- Contrast AA, flash limits, CPU budget.
- Determinism: seeded RNG; record all accepted changes in Recipe.

Pseudocode

```rust
fn step_animation(app: &mut App, dt: f64) -> Result<()> {
    app.inputs.update(dt);
    app.modulation.apply(&mut app.scene, app.time);
    if app.scheduler.should_transition(app.time) {
        let proposal = app.evolver.propose(&app.scene, &app.history, &app.registry);
        let next = app.guards.validate(proposal).unwrap_or_else(|| app.scheduler.fallback());
        app.transition.begin(app.scene.clone(), next);
    }
    app.engine.update(dt);
    let field = app.scene_graph.evaluate(&app.engine, app.viewport);
    app.buffer.update_colors_field(&field)?;
    Ok(())
}
```

---

### AI-assisted evolution (optional but glorious)

Context to model
- Current Scene summary: patterns, params (+ ranges), theme/variant, recent features, constraints.
- Registry metadata: available patterns/themes/transitions with tags.

Outputs expected
- Scene diff: param deltas, suggested transition, duration hint, rationale.

Modes
- Suggestion (manual accept), Co-pilot (slow drift), Curator (playlist builder).

Provenance
- Store prompt, response, registry version hashes, final accepted Scene diff.

---

### Endless Mode config (example)

```yaml
mode: hybrid-ai-novelty
min_scene_duration: 8s
max_scene_duration: 45s
ai_cadence: 60s
constraints: { min_contrast: AA, max_flash_hz: 20, cpu_budget_ms: 10 }
weights: { patterns: { fluid: 0.4, geometric: 0.3, organic: 0.3 }, transitions: { crossfade: 0.7, mask_wipe: 0.2, glitch_cut: 0.1 } }
mod_sources:
  - lfo:  { shape: sine, period: 6s, depth: 0.5, to: "layer[0].params.frequency" }
  - noise:{ rate: 2s, depth: 0.2, to: "layer[0].params.complexity" }
record: { webp: true, gif: false, png_strip: true }
```

---

### Implementation steps (incremental)

1) Add `--playground` flag and `ratatui` scaffold with overlay toggle.
2) Introduce `Scene`, `Layer`, `BlendMode`, `ModSource`, `ModRoute` structs (no behavior change).
3) Implement a single `Lfo` source; apply routes before `engine.update`.
4) Minimal scheduler with durations; crossfade transition; novelty metrics logging.
5) Export panel to record deterministic WebP/GIF/PNG.
6) Optional: AI Muse behind a feature flag with strict guards + provenance.

---

### SilkCircuit design hooks

- Variants: neon, vibrant, soft, glow for UI chrome.
- Accents: purple for controls, cyan for focus/glow, yellow for caution, green for success, red for errors.
- AA contrast enforced in status/overlays; motion eased, not flashy.

---

### Open questions

- Fixed timestep default vs. adaptive step on slow terminals?
- Where to draw the line between ratatui-managed buffer vs. direct ANSI writes?
- Script engine (Rhai/Lua) scope for v1: params only or lightweight patterns too?

---

### Done well when

- The playground runs smoothly with overlays, no flicker.
- Users can endlessly evolve scenes hands-free or with minimal keys.
- Exports are identical to live frames; recipes recreate sessions perfectly.


