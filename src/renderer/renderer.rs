/// Core renderer that coordinates all rendering components
/// This is the simplified, clean architecture version
use crossterm::event::{KeyEvent, MouseEvent};

use crate::debug_log::debug_log;
use crate::demo::DemoArt;
use crate::input::InputReader;
use crate::pattern::PatternEngine;
use crate::pattern::{PatternConfig, REGISTRY};
use crate::renderer::{
    error::RendererError,
    input::{InputAction, PlaygroundInputHandler},
    playground::PlaygroundUI,
};
use crate::themes;

/// The main renderer struct - coordinates all rendering
pub struct Renderer {
    /// Pattern engine for generating colors
    engine: PatternEngine,

    /// Current content to render
    content: String,

    /// Playground UI manager
    playground: PlaygroundUI,

    /// Whether we're in demo mode
    demo_mode: bool,

    /// Available patterns
    available_patterns: Vec<String>,

    /// Available themes
    available_themes: Vec<String>,

    /// Available demo arts
    available_arts: Vec<String>,
}

impl Renderer {
    /// Create a new renderer
    pub fn new(
        engine: PatternEngine,
        _animation_config: crate::renderer::config::AnimationConfig,
        _playlist: Option<crate::playlist::Playlist>,
        demo_mode: bool,
    ) -> Result<Self, RendererError> {
        // Initialize available options
        let available_patterns: Vec<String> = REGISTRY
            .list_patterns()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let available_themes: Vec<String> = themes::all_themes()
            .iter()
            .map(|t| t.name.clone())
            .collect();

        let available_arts = if demo_mode {
            crate::demo::DemoArt::all_types()
                .iter()
                .map(|art| art.as_str().to_string())
                .collect()
        } else {
            Vec::new()
        };

        // Initialize playground UI
        let mut playground = PlaygroundUI::new();
        playground.pattern_names = available_patterns.clone();
        playground.theme_names = available_themes.clone();
        playground.art_names = available_arts.clone();

        // Get initial content - load default art for demo mode
        let content = if demo_mode {
            // Load a default art if in demo mode
            Self::load_demo_art("cityscape").unwrap_or_else(|_| String::new())
        } else {
            String::new()
        };

        Ok(Self {
            engine,
            content,
            playground,
            demo_mode,
            available_patterns,
            available_themes,
            available_arts,
        })
    }

    /// Main render method
    pub fn render(&mut self, delta_seconds: f64) -> Result<(), RendererError> {
        // Update animation
        self.engine.update(delta_seconds);

        // Update parameter names in playground
        self.playground.param_names = self.get_current_param_names();

        // Debug log content length
        debug_log(&format!(
            "Rendering with {} chars of content",
            self.content.len()
        ))
        .ok();

        // Render the frame
        self.playground
            .render(&self.content, &self.engine, self.engine.time())
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
                if self.demo_mode {
                    self.apply_art(&art)?;
                }
                Ok(true)
            }
            InputAction::AdjustParam { name, value } => {
                self.adjust_param(&name, value)?;
                Ok(true)
            }
            InputAction::Quit => Ok(false),
        }
    }

    /// Handle mouse input
    pub fn handle_mouse(&mut self, event: MouseEvent) -> Result<bool, RendererError> {
        let action = PlaygroundInputHandler::handle_mouse(&mut self.playground, event)?;

        match action {
            InputAction::None => Ok(false),
            InputAction::Redraw => Ok(true),
            _ => Ok(true), // Handle other actions same as keyboard
        }
    }

    /// Handle terminal resize
    pub fn handle_resize(&mut self, width: u16, height: u16) -> Result<(), RendererError> {
        self.playground.resize()?;

        // Update pattern engine dimensions
        self.engine = self.engine.recreate(width as usize, height as usize);
        let _ = debug_log(&format!("Resized to {}x{}", width, height));

        Ok(())
    }

    /// Load demo art content
    fn load_demo_art(art: &str) -> Result<String, RendererError> {
        let demo = DemoArt::try_from_str(art)
            .ok_or_else(|| RendererError::Other(format!("Unknown art: {}", art)))?;

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
        self.playground.show_toast(&format!("Pattern: {}", pattern));
        Ok(())
    }

    /// Apply a theme
    fn apply_theme(&mut self, theme: &str) -> Result<(), RendererError> {
        let gradient = themes::get_theme(theme)?.create_gradient()?;
        self.engine.update_gradient(gradient);
        self.playground.show_toast(&format!("Theme: {}", theme));
        Ok(())
    }

    /// Apply demo art
    fn apply_art(&mut self, art: &str) -> Result<(), RendererError> {
        self.content = Self::load_demo_art(art)?;
        self.playground.show_toast(&format!("Art: {}", art));
        Ok(())
    }

    /// Adjust a parameter
    fn adjust_param(&mut self, name: &str, value: f64) -> Result<(), RendererError> {
        // TODO: Implement parameter adjustment
        self.playground
            .show_toast(&format!("Param: {} = {:.2}", name, value));
        Ok(())
    }

    /// Set initial demo art (called before run)
    pub fn set_demo_art(&mut self, art: &str) -> Result<(), RendererError> {
        if self.demo_mode {
            self.content = Self::load_demo_art(art)?;
        }
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
        // TODO: Implement scene scheduling if needed
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
