/// Core renderer that coordinates all rendering components
/// This is the simplified, clean architecture version
use crossterm::event::{KeyEvent, MouseEvent};
use std::sync::Arc;

use crate::debug_log::debug_log;
use crate::demo::DemoArt;
use crate::input::InputReader;
use crate::pattern::PatternEngine;
use crate::pattern::{PatternConfig, REGISTRY};
use crate::recipes::Recipe;
use crate::renderer::{
    automix::{Automix, AutomixMode},
    blend_engine::{BlendEngine, TransitionEffect},
    error::RendererError,
    input::{InputAction, PlaygroundInputHandler},
    playground::PlaygroundUI,
};
use crate::themes;

/// The main renderer struct - coordinates all rendering
pub struct Renderer {
    /// Pattern engine for generating colors
    engine: PatternEngine,

    /// Blend engine for smooth transitions
    blend_engine: BlendEngine,

    /// Current transition effect
    transition_effect: TransitionEffect,

    /// Current content to render
    content: String,

    /// Playground UI manager
    playground: PlaygroundUI,

    /// Available patterns
    _available_patterns: Vec<String>,

    /// Available themes
    _available_themes: Vec<String>,

    /// Available demo arts
    _available_arts: Vec<String>,

    /// Automix system for seamless transitions
    automix: Automix,
}

impl Renderer {
    /// Create a new renderer
    pub fn new(
        engine: PatternEngine,
        _animation_config: crate::renderer::config::AnimationConfig,
        playlist: Option<crate::playlist::Playlist>,
        initial_theme: &str,
        initial_pattern: &str,
    ) -> Result<Self, RendererError> {
        // Initialize available options and sort them
        let mut available_patterns: Vec<String> = REGISTRY
            .list_patterns()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        available_patterns.sort();

        let mut available_themes: Vec<String> = themes::all_themes()
            .iter()
            .map(|t| t.name.clone())
            .collect();
        available_themes.sort();

        let mut available_arts: Vec<String> = crate::demo::DemoArt::all_types()
            .iter()
            .map(|art| art.as_str().to_string())
            .collect();
        available_arts.sort();

        // Initialize playground UI
        let mut playground = PlaygroundUI::new();
        playground.pattern_names = available_patterns.clone();
        playground.theme_names = available_themes.clone();
        playground.art_names = available_arts.clone();

        // Set initial theme and pattern from CLI args (not hardcoded)
        playground.current_theme = initial_theme.to_string();
        playground.current_pattern = initial_pattern.to_string();

        // Set selection indices to match initial values
        if let Some(idx) = available_patterns.iter().position(|p| p == initial_pattern) {
            playground.pattern_sel = idx;
        }
        if let Some(idx) = available_themes.iter().position(|t| t == initial_theme) {
            playground.theme_sel = idx;
        }

        // Get initial content - always load default art in playground mode
        let content = Self::load_demo_art("rainbow").unwrap_or_else(|_| String::new());

        // Set up automix
        let mut automix = Automix::new();
        let initial_mode = if let Some(playlist) = playlist {
            automix.load_playlist(playlist);
            // When playlist is provided, start in Playlist mode
            automix.set_mode(AutomixMode::Playlist);
            "Playlist"
        } else {
            // Default to Showcase mode when no playlist
            automix.set_mode(AutomixMode::Showcase);
            "Showcase"
        };
        playground.automix_mode = initial_mode.to_string();

        Ok(Self {
            engine,
            blend_engine: BlendEngine::new(),
            transition_effect: TransitionEffect::Crossfade,
            content,
            playground,
            _available_patterns: available_patterns,
            _available_themes: available_themes,
            _available_arts: available_arts,
            automix,
        })
    }

    /// Main render method
    pub fn render(&mut self, delta_seconds: f64) -> Result<(), RendererError> {
        // Update automix system
        let automix_update = self.automix.update(delta_seconds);

        // Check what's changing to vary transition effect
        let has_pattern_change = automix_update.new_pattern.is_some();
        let has_theme_change = automix_update.new_theme.is_some();

        // Apply all automix updates - they can overlap for chill vibes
        // Different things change at different rates
        if let Some(pattern) = automix_update.new_pattern {
            if let Err(e) = self.start_pattern_transition(&pattern) {
                let _ = debug_log(&format!("Automix pattern transition failed: {e}"));
            }
        }
        if let Some(theme) = automix_update.new_theme {
            if let Err(e) = self.start_theme_transition(&theme) {
                let _ = debug_log(&format!("Automix theme transition failed: {e}"));
            }
        }
        if let Some(art) = automix_update.new_art {
            if let Err(e) = self.apply_art(&art) {
                let _ = debug_log(&format!("Automix art change failed: {e}"));
            }
        }

        // Vary transition effect based on what's changing
        if has_pattern_change && has_theme_change {
            self.transition_effect = TransitionEffect::Kaleidoscope;
        } else if has_pattern_change {
            // Cycle through effects for pattern changes
            self.transition_effect = match self.transition_effect {
                TransitionEffect::Crossfade => TransitionEffect::Ripple,
                TransitionEffect::Ripple => TransitionEffect::Spiral,
                TransitionEffect::Spiral => TransitionEffect::Wave,
                TransitionEffect::Wave => TransitionEffect::Pixelate,
                _ => TransitionEffect::Crossfade,
            };
        }

        // Update playground automix status
        self.playground.scene_progress = automix_update.scene_progress;
        self.playground.is_transitioning = automix_update.is_transitioning;

        // Update blend engine
        self.blend_engine.update(delta_seconds);

        // Update animation - blend engine handles updates during transitions
        if !self.blend_engine.is_transitioning() {
            // Normal update when not transitioning
            self.engine.update(delta_seconds);
        }
        // Note: During transitions, the blend_engine.update() above handles
        // updating both source and target engines

        // Update parameter names in playground
        self.playground.param_names = self.get_current_param_names();

        // Debug log content length
        debug_log(&format!(
            "Rendering with {} chars of content",
            self.content.len()
        ))
        .ok();

        // Render the frame with blending
        self.playground.render_with_blending(
            &self.content,
            &self.engine,
            &self.blend_engine,
            self.transition_effect,
            self.engine.time(),
        )
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<bool, RendererError> {
        let action = PlaygroundInputHandler::handle_key(&mut self.playground, key)?;

        match action {
            InputAction::None => Ok(true), // Don't exit on unhandled keys
            InputAction::Redraw => Ok(true),
            InputAction::ApplyPattern(pattern) => {
                self.apply_pattern(&pattern)?;
                Ok(true)
            }
            InputAction::ApplyTheme(theme) => {
                self.apply_theme(&theme)?;
                Ok(true)
            }
            InputAction::ApplyArt(art) => {
                self.apply_art(&art)?;
                Ok(true)
            }
            InputAction::AdjustParam { name, value } => {
                self.adjust_param(&name, value)?;
                Ok(true)
            }
            InputAction::AutomixToggle => {
                self.toggle_automix();
                Ok(true)
            }
            InputAction::AutomixMode(mode) => {
                self.set_automix_mode(&mode);
                Ok(true)
            }
            InputAction::AutomixNext => {
                self.automix.skip_next();
                Ok(true)
            }
            InputAction::AutomixPrev => {
                self.automix.skip_prev();
                Ok(true)
            }
            InputAction::CycleCrossfadeDuration => {
                self.cycle_crossfade_duration();
                Ok(true)
            }
            InputAction::SaveRecipe => {
                self.save_recipe()?;
                Ok(true)
            }
            InputAction::LoadRecipe => {
                self.load_recipe()?;
                Ok(true)
            }
            InputAction::Quit => Ok(false),
        }
    }

    /// Handle mouse input
    pub fn handle_mouse(&mut self, event: MouseEvent) -> Result<bool, RendererError> {
        let action = PlaygroundInputHandler::handle_mouse(&mut self.playground, event)?;

        match action {
            InputAction::None => Ok(true), // Don't exit on unhandled mouse events
            InputAction::Redraw => Ok(true),
            InputAction::ApplyPattern(pattern) => {
                self.apply_pattern(&pattern)?;
                Ok(true)
            }
            InputAction::ApplyTheme(theme) => {
                self.apply_theme(&theme)?;
                Ok(true)
            }
            InputAction::ApplyArt(art) => {
                self.apply_art(&art)?;
                Ok(true)
            }
            InputAction::AdjustParam { name, value } => {
                self.adjust_param(&name, value)?;
                Ok(true)
            }
            InputAction::AutomixToggle => {
                self.toggle_automix();
                Ok(true)
            }
            InputAction::AutomixMode(mode) => {
                self.set_automix_mode(&mode);
                Ok(true)
            }
            InputAction::AutomixNext => {
                self.automix.skip_next();
                Ok(true)
            }
            InputAction::AutomixPrev => {
                self.automix.skip_prev();
                Ok(true)
            }
            InputAction::CycleCrossfadeDuration => {
                self.cycle_crossfade_duration();
                Ok(true)
            }
            InputAction::SaveRecipe => {
                self.save_recipe()?;
                Ok(true)
            }
            InputAction::LoadRecipe => {
                self.load_recipe()?;
                Ok(true)
            }
            InputAction::Quit => Ok(false),
        }
    }

    /// Handle terminal resize
    pub fn handle_resize(&mut self, width: u16, height: u16) -> Result<(), RendererError> {
        self.playground.resize()?;

        // Update pattern engine dimensions
        self.engine = self.engine.recreate(width as usize, height as usize);
        let _ = debug_log(&format!("Resized to {width}x{height}"));

        Ok(())
    }

    /// Load demo art content
    fn load_demo_art(art: &str) -> Result<String, RendererError> {
        let demo = DemoArt::try_from_str(art)
            .ok_or_else(|| RendererError::Other(format!("Unknown art: {art}")))?;

        let mut reader = InputReader::from_demo(true, None, Some(&demo))?;
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        Ok(content)
    }

    /// Apply a pattern
    fn apply_pattern(&mut self, pattern: &str) -> Result<(), RendererError> {
        let params = REGISTRY
            .create_pattern_params(pattern)
            .ok_or_else(|| RendererError::InvalidPattern(pattern.to_string()))?;

        let config = PatternConfig {
            common: self.engine.config().common.clone(),
            params,
        };

        self.engine.update_pattern_config(config);

        // Update UI state and selection
        self.playground.current_pattern = pattern.to_string();
        if let Some(index) = self
            .playground
            .pattern_names
            .iter()
            .position(|p| p == pattern)
        {
            self.playground.pattern_sel = index;
        }

        Ok(())
    }

    /// Apply a theme
    fn apply_theme(&mut self, theme: &str) -> Result<(), RendererError> {
        let gradient = themes::get_theme(theme)?.create_gradient()?;
        self.engine.update_gradient(gradient);

        // Update UI state and selection
        self.playground.current_theme = theme.to_string();
        if let Some(index) = self.playground.theme_names.iter().position(|t| t == theme) {
            self.playground.theme_sel = index;
        }

        Ok(())
    }

    /// Apply demo art
    fn apply_art(&mut self, art: &str) -> Result<(), RendererError> {
        self.content = Self::load_demo_art(art)?;

        // Update UI state and selection
        self.playground.current_art = Some(art.to_string());
        if let Some(index) = self.playground.art_names.iter().position(|a| a == art) {
            self.playground.art_sel = index;
        }

        Ok(())
    }

    /// Adjust a parameter value by a delta amount
    fn adjust_param(&mut self, name: &str, delta: f64) -> Result<(), RendererError> {
        use crate::pattern::PatternParams;

        let mut config = self.engine.config().clone();

        // Helper to apply delta and clamp
        let adjust =
            |current: f64, min: f64, max: f64| -> f64 { (current + delta).clamp(min, max) };

        // Try common params first
        match name {
            "frequency" => config.common.frequency = adjust(config.common.frequency, 0.1, 10.0),
            "amplitude" => config.common.amplitude = adjust(config.common.amplitude, 0.1, 2.0),
            "speed" => config.common.speed = adjust(config.common.speed, 0.0, 1.0),
            _ => {
                // Pattern-specific params
                match &mut config.params {
                    PatternParams::Horizontal(p) => {
                        if name == "invert" {
                            p.invert = !p.invert; // Toggle on any adjustment
                        }
                    }
                    PatternParams::Diagonal(p) => match name {
                        "angle" => p.angle = ((p.angle as f64 + delta * 10.0) as i32).clamp(0, 360),
                        "frequency" => p.frequency = adjust(p.frequency, 0.1, 10.0),
                        _ => {}
                    },
                    PatternParams::Plasma(p) => match name {
                        "complexity" => p.complexity = adjust(p.complexity, 1.0, 10.0),
                        "scale" => p.scale = adjust(p.scale, 0.1, 5.0),
                        "frequency" => p.frequency = adjust(p.frequency, 0.1, 10.0),
                        _ => {}
                    },
                    PatternParams::Ripple(p) => match name {
                        "center_x" => p.center_x = adjust(p.center_x, 0.0, 1.0),
                        "center_y" => p.center_y = adjust(p.center_y, 0.0, 1.0),
                        "wavelength" => p.wavelength = adjust(p.wavelength, 0.1, 5.0),
                        "damping" => p.damping = adjust(p.damping, 0.0, 1.0),
                        _ => {}
                    },
                    PatternParams::Wave(p) => match name {
                        "amplitude" => p.amplitude = adjust(p.amplitude, 0.1, 2.0),
                        "frequency" => p.frequency = adjust(p.frequency, 0.1, 10.0),
                        "phase" => p.phase = adjust(p.phase, 0.0, std::f64::consts::TAU),
                        "offset" => p.offset = adjust(p.offset, 0.0, 1.0),
                        "base_freq" => p.base_freq = adjust(p.base_freq, 0.1, 5.0),
                        _ => {}
                    },
                    PatternParams::Spiral(p) => match name {
                        "density" => p.density = adjust(p.density, 0.1, 5.0),
                        "rotation" => p.rotation = adjust(p.rotation, 0.0, 360.0),
                        "expansion" => p.expansion = adjust(p.expansion, 0.1, 2.0),
                        "clockwise" => p.clockwise = !p.clockwise, // Toggle
                        _ => {}
                    },
                    PatternParams::Checkerboard(p) => match name {
                        "size" => p.size = ((p.size as f64 + delta * 10.0) as usize).clamp(1, 10),
                        "blur" => p.blur = adjust(p.blur, 0.0, 1.0),
                        "rotation" => p.rotation = adjust(p.rotation, 0.0, 360.0),
                        "scale" => p.scale = adjust(p.scale, 0.1, 5.0),
                        _ => {}
                    },
                    PatternParams::Diamond(p) => match name {
                        "size" => p.size = adjust(p.size, 0.1, 5.0),
                        "offset" => p.offset = adjust(p.offset, 0.0, 1.0),
                        "sharpness" => p.sharpness = adjust(p.sharpness, 0.1, 5.0),
                        "rotation" => p.rotation = adjust(p.rotation, 0.0, 360.0),
                        _ => {}
                    },
                    PatternParams::Perlin(p) => match name {
                        "octaves" => {
                            p.octaves = ((p.octaves as f64 + delta * 10.0) as u32).clamp(1, 8)
                        }
                        "persistence" => p.persistence = adjust(p.persistence, 0.0, 1.0),
                        "scale" => p.scale = adjust(p.scale, 0.1, 5.0),
                        "seed" => p.seed = (p.seed as i32 + (delta * 100.0) as i32).max(0) as u32,
                        _ => {}
                    },
                    PatternParams::PixelRain(p) => match name {
                        "speed" => p.speed = adjust(p.speed, 0.1, 5.0),
                        "density" => p.density = adjust(p.density, 0.1, 2.0),
                        "length" => p.length = adjust(p.length, 1.0, 10.0),
                        "glitch" => p.glitch = !p.glitch, // Toggle
                        _ => {}
                    },
                    PatternParams::Fire(p) => match name {
                        "intensity" => p.intensity = adjust(p.intensity, 0.1, 2.0),
                        "speed" => p.speed = adjust(p.speed, 0.1, 5.0),
                        "turbulence" => p.turbulence = adjust(p.turbulence, 0.0, 1.0),
                        "height" => p.height = adjust(p.height, 0.1, 2.0),
                        _ => {}
                    },
                    PatternParams::Aurora(p) => match name {
                        "intensity" => p.intensity = adjust(p.intensity, 0.1, 2.0),
                        "speed" => p.speed = adjust(p.speed, 0.1, 5.0),
                        "waviness" => p.waviness = adjust(p.waviness, 0.1, 2.0),
                        "layers" => {
                            p.layers = ((p.layers as f64 + delta * 10.0) as u32).clamp(1, 5)
                        }
                        _ => {}
                    },
                    PatternParams::Kaleidoscope(p) => match name {
                        "segments" => {
                            p.segments = ((p.segments as f64 + delta * 10.0) as u32).clamp(3, 12)
                        }
                        "rotation_speed" => p.rotation_speed = adjust(p.rotation_speed, 0.1, 5.0),
                        "zoom" => p.zoom = adjust(p.zoom, 0.5, 3.0),
                        "complexity" => p.complexity = adjust(p.complexity, 1.0, 5.0),
                        _ => {}
                    },
                }
            }
        }

        self.engine.update_pattern_config(config);
        Ok(())
    }

    /// Start a pattern transition with blending
    fn start_pattern_transition(&mut self, pattern: &str) -> Result<(), RendererError> {
        // Start blending to new pattern
        // Use playground terminal size
        let (width, height) = self.playground.terminal_size;
        let width = width as usize;
        let height = height as usize;

        // Get current gradient from the current theme
        let current_gradient = Arc::new(
            themes::get_theme(&self.playground.current_theme)
                .ok()
                .and_then(|t| t.create_gradient().ok())
                .unwrap_or_else(|| {
                    // Fallback gradient
                    Box::new(
                        colorgrad::GradientBuilder::new()
                            .colors(&[
                                colorgrad::Color::from_rgba8(255, 0, 128, 255),
                                colorgrad::Color::from_rgba8(0, 128, 255, 255),
                            ])
                            .build::<colorgrad::LinearGradient>()
                            .expect("simple gradient should build"),
                    )
                }),
        );

        // Clone current engine for transition
        self.blend_engine
            .start_pattern_transition(
                self.engine.clone(),
                current_gradient,
                pattern,
                width,
                height,
            )
            .map_err(RendererError::Other)?;

        // Update UI state
        self.playground.current_pattern = pattern.to_string();
        if let Some(index) = self
            .playground
            .pattern_names
            .iter()
            .position(|p| p == pattern)
        {
            self.playground.pattern_sel = index;
        }

        Ok(())
    }

    /// Start a theme transition with gradient blending
    fn start_theme_transition(&mut self, theme: &str) -> Result<(), RendererError> {
        // Get current gradient (create a new one based on current theme)
        let current_gradient =
            Arc::new(themes::get_theme(&self.playground.current_theme)?.create_gradient()?);

        // Start gradient blending
        self.blend_engine
            .start_theme_transition(current_gradient, theme)
            .map_err(RendererError::Other)?;

        // Also update the engine's gradient (will be blended in render)
        let gradient = themes::get_theme(theme)?.create_gradient()?;
        self.engine.update_gradient(gradient);

        // Update UI state
        self.playground.current_theme = theme.to_string();
        if let Some(index) = self.playground.theme_names.iter().position(|t| t == theme) {
            self.playground.theme_sel = index;
        }

        Ok(())
    }

    /// Set initial demo art (called before run)
    pub fn set_demo_art(&mut self, art: &str) -> Result<(), RendererError> {
        self.content = Self::load_demo_art(art)?;
        Ok(())
    }

    /// Set overlay visibility
    pub fn set_overlay_visible(&mut self, visible: bool) {
        self.playground.overlay_visible = visible;
    }

    /// Set status message
    pub fn set_status_message(&mut self, message: &str) {
        self.playground.show_toast(message);
    }

    /// Enable default scenes (for compatibility)
    pub fn enable_default_scenes(&mut self) {
        // Start automix in showcase mode for demos
        self.automix.set_mode(AutomixMode::Showcase);
        self.playground.automix_mode = "Showcase".to_string();
    }

    /// Toggle automix on/off
    fn toggle_automix(&mut self) {
        let new_mode = if self.automix.mode() == AutomixMode::Off {
            AutomixMode::Showcase
        } else {
            AutomixMode::Off
        };
        self.automix.set_mode(new_mode);
        self.playground.automix_mode = match new_mode {
            AutomixMode::Off => "Off",
            AutomixMode::Random => "Random",
            AutomixMode::Showcase => "Showcase",
            AutomixMode::Playlist => "Playlist",
            AutomixMode::Adaptive => "Adaptive",
        }
        .to_string();
    }

    /// Set specific automix mode
    fn set_automix_mode(&mut self, mode_str: &str) {
        let mode = match mode_str {
            "off" => AutomixMode::Off,
            "random" => AutomixMode::Random,
            "showcase" => AutomixMode::Showcase,
            "playlist" => AutomixMode::Playlist,
            "adaptive" => AutomixMode::Adaptive,
            _ => return,
        };
        self.automix.set_mode(mode);
        self.playground.automix_mode = match mode {
            AutomixMode::Off => "Off",
            AutomixMode::Random => "Random",
            AutomixMode::Showcase => "Showcase",
            AutomixMode::Playlist => "Playlist",
            AutomixMode::Adaptive => "Adaptive",
        }
        .to_string();
    }

    /// Render static content (non-animated)
    pub fn render_static(&mut self, content: &str) -> Result<(), RendererError> {
        self.content = content.to_string();
        // Just render a single frame
        self.render(0.0)
    }

    /// Get frame duration for animation timing
    pub fn frame_duration(&mut self) -> std::time::Duration {
        std::time::Duration::from_millis(33) // ~30 FPS
    }

    /// Render a single animated frame (compatibility)
    pub fn render_frame(&mut self, _text: &str, delta_seconds: f64) -> Result<(), RendererError> {
        self.render(delta_seconds)
    }

    /// Handle key event (compatibility)
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool, RendererError> {
        self.handle_key(key)
    }

    /// Handle mouse event (compatibility)
    pub fn handle_mouse_event(&mut self, event: MouseEvent) -> Result<(), RendererError> {
        self.handle_mouse(event)?;
        Ok(())
    }

    /// Get current parameter names
    fn get_current_param_names(&self) -> Vec<String> {
        // Get the current pattern ID from the config
        let current_id = REGISTRY
            .get_pattern_id(&self.engine.config().params)
            .unwrap_or("horizontal");

        // Get the pattern metadata and extract parameter names
        if let Some(meta) = REGISTRY.get_pattern(current_id) {
            let subs = meta.params().sub_params();
            subs.into_iter().map(|p| p.name().to_string()).collect()
        } else {
            Vec::new()
        }
    }

    /// Save current state as a recipe to chromacat_recipe.yaml
    fn save_recipe(&mut self) -> Result<(), RendererError> {
        let recipe = self.create_recipe_snapshot();
        let yaml = serde_yaml::to_string(&recipe)
            .map_err(|e| RendererError::Other(format!("Failed to serialize recipe: {e}")))?;
        std::fs::write("chromacat_recipe.yaml", yaml)
            .map_err(|e| RendererError::Other(format!("Failed to write recipe: {e}")))?;
        self.playground
            .show_toast("Recipe saved to chromacat_recipe.yaml");
        Ok(())
    }

    /// Load recipe from chromacat_recipe.yaml
    fn load_recipe(&mut self) -> Result<(), RendererError> {
        match std::fs::read_to_string("chromacat_recipe.yaml") {
            Ok(yaml) => match serde_yaml::from_str::<Recipe>(&yaml) {
                Ok(recipe) => {
                    self.apply_recipe(recipe)?;
                    self.playground.show_toast("Recipe loaded");
                }
                Err(e) => {
                    self.playground.show_toast(format!("Parse error: {e}"));
                }
            },
            Err(e) => {
                self.playground.show_toast(format!("Read error: {e}"));
            }
        }
        Ok(())
    }

    /// Cycle through crossfade durations
    fn cycle_crossfade_duration(&mut self) {
        // Cycle through: 1s -> 2s -> 3s -> 5s -> 8s -> 1s
        let durations_ms = [1000, 2000, 3000, 5000, 8000];
        let current = self.automix.transition_duration_ms();
        let next_idx = durations_ms.iter().position(|&d| d > current).unwrap_or(0);
        let next_duration = durations_ms[next_idx];
        self.automix.set_transition_duration_ms(next_duration);
        self.playground
            .show_toast(format!("Crossfade: {}s", next_duration / 1000));
    }

    /// Create a snapshot of current state as a Recipe
    fn create_recipe_snapshot(&self) -> Recipe {
        let current_pattern = REGISTRY
            .get_pattern_id(&self.engine.config().params)
            .map(|s| s.to_string());

        Recipe {
            current_theme: Some(self.playground.current_theme.clone()),
            current_pattern,
            scenes: Vec::new(), // Not using scene scheduler in new arch
            routes: Vec::new(), // Not using modulation routes in new arch
            theme_mode: None,
            crossfade_seconds: None,
        }
    }

    /// Apply a loaded recipe to current state
    fn apply_recipe(&mut self, recipe: Recipe) -> Result<(), RendererError> {
        if let Some(pattern) = recipe.current_pattern {
            self.apply_pattern(&pattern)?;
        }
        if let Some(theme) = recipe.current_theme {
            self.apply_theme(&theme)?;
        }
        Ok(())
    }

    /// Main run method for the renderer - creates and runs the event loop
    pub fn run(mut self, content: String) -> Result<(), RendererError> {
        use crate::renderer::event_loop::EventLoop;

        // Set the initial content if provided
        if !content.is_empty() {
            self.content = content;
        }

        let event_loop = EventLoop::new(self, 30); // 30 FPS
        event_loop.run()
    }
}
