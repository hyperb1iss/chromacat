//! Terminal rendering system for ChromaCat
//!
//! This module provides functionality for rendering colored text output to the terminal,
//! handling both static and animated displays. It manages terminal state, text buffers,
//! color calculations, scrolling, and status information display.
//!
//! The rendering system is built around several key components:
//! - Terminal state management and interaction
//! - Double buffered text and color handling
//! - Pattern-based color generation
//! - Scrolling and viewport control
//! - Status bar rendering
//! - Frame timing and synchronization

mod buffer;
mod config;
mod error;
mod scroll;
mod status_bar;
pub mod terminal;

pub use buffer::RenderBuffer;
pub use config::AnimationConfig;
pub use error::RendererError;
pub use scroll::{Action, ScrollState};
pub use status_bar::StatusBar;
pub use terminal::TerminalState;

use crate::pattern::PatternEngine;
use crate::pattern::{PatternConfig, REGISTRY};
use crate::themes;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use std::io::Write;
use std::time::{Duration, Instant};

/// Coordinates all rendering functionality for ChromaCat
pub struct Renderer {
    /// Pattern generation engine
    engine: PatternEngine,
    /// Animation configuration
    config: AnimationConfig,
    /// Double buffered text and colors
    buffer: RenderBuffer,
    /// Terminal state manager
    terminal: TerminalState,
    /// Scrolling state manager
    scroll: ScrollState,
    /// Status bar renderer
    status_bar: StatusBar,
    /// Available theme names
    available_themes: Vec<String>,
    /// Current theme index
    current_theme_index: usize,
    /// Available pattern IDs
    available_patterns: Vec<String>,
    /// Current pattern index
    current_pattern_index: usize,
    /// Last frame timestamp for timing
    last_frame: Option<Instant>,
    /// Frame counter for FPS calculation
    frame_count: u32,
    /// Last FPS update timestamp
    last_fps_update: Instant,
    /// Current FPS measurement
    current_fps: f64,
}

impl Renderer {
    /// Creates a new renderer with the given pattern engine and configuration
    pub fn new(engine: PatternEngine, config: AnimationConfig) -> Result<Self, RendererError> {
        let terminal = TerminalState::new()?;
        let term_size = terminal.size();
        let buffer = RenderBuffer::new(term_size);
        let scroll = ScrollState::new(term_size.1.saturating_sub(2));
        let mut status_bar = StatusBar::new(term_size);

        // Initialize available themes
        let available_themes = themes::all_themes()
            .iter()
            .map(|t| t.name.clone())
            .collect::<Vec<_>>();

        // Initialize available patterns from registry
        let available_patterns = REGISTRY
            .list_patterns()
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        // Set initial theme and pattern in status bar based on engine's configuration
        let initial_theme = match engine.config().common.theme_name.as_ref() {
            Some(name) => name,
            None => "rainbow", // fallback
        };
        status_bar.set_theme(initial_theme);

        // Find the current theme index
        let current_theme_index = available_themes
            .iter()
            .position(|t| t == initial_theme)
            .unwrap_or(0);

        // Get pattern ID from registry based on current params
        let initial_pattern = REGISTRY
            .get_pattern_id(&engine.config().params)
            .unwrap_or("horizontal"); // fallback to horizontal if pattern not found

        let current_pattern_index = available_patterns
            .iter()
            .position(|p| p == initial_pattern)
            .unwrap_or(0);

        status_bar.set_pattern(initial_pattern);

        // Initialize timing state
        let now = Instant::now();

        let fps = config.fps; // Store fps before moving config

        Ok(Self {
            engine,
            config,
            buffer,
            terminal,
            scroll,
            status_bar,
            available_themes,
            current_theme_index,
            available_patterns,
            current_pattern_index,
            last_frame: None,
            frame_count: 0,
            last_fps_update: now,
            current_fps: fps as f64,
        })
    }

    /// Returns the frame duration based on configured FPS
    #[inline]
    pub fn frame_duration(&self) -> Duration {
        self.config.frame_duration()
    }

    /// Returns whether animation is set to run indefinitely
    #[inline]
    pub fn is_infinite(&self) -> bool {
        self.config.infinite
    }

    /// Returns the configured animation cycle duration
    #[inline]
    pub fn cycle_duration(&self) -> Duration {
        self.config.cycle_duration
    }

    /// Renders static text with pattern-based colors, advancing the pattern
    /// for each line to create a flowing effect similar to lolcat
    pub fn render_static(&mut self, text: &str) -> Result<(), RendererError> {
        // Prepare the full content
        self.buffer.prepare_text(text)?;

        // Use static color update mode for the entire content
        self.buffer.update_colors_static(&self.engine)?;

        // Get a stdout lock for efficient writing
        let mut stdout = self.terminal.stdout();

        // Render the entire buffer content, not just one screen
        self.buffer.render_region(
            &mut stdout,
            0,
            self.buffer.total_lines(),
            self.terminal.colors_enabled(),
            false,
        )?;

        // Ensure everything is flushed
        stdout.flush()?;
        Ok(())
    }

    /// Renders a single animation frame
    pub fn render_frame(&mut self, text: &str, delta_seconds: f64) -> Result<(), RendererError> {
        let now = Instant::now();

        // First-time initialization
        if !self.buffer.has_content() {
            self.terminal.enter_alternate_screen()?;
            self.buffer.prepare_text(text)?;
            self.scroll.set_total_lines(self.buffer.line_count());
            let visible_range = self.scroll.get_visible_range();
            self.buffer.update_colors(&self.engine, visible_range.0)?;
            self.draw_full_screen()?;
            self.last_frame = Some(now);
            self.last_fps_update = now;
            return Ok(());
        }

        // Calculate and enforce frame timing more precisely
        let frame_time = if let Some(last) = self.last_frame {
            now.duration_since(last)
        } else {
            Duration::from_secs_f64(delta_seconds)
        };

        let target_frame_time = Duration::from_secs_f64(1.0 / self.config.fps as f64);
        if frame_time < target_frame_time {
            // Skip frame if we're ahead of schedule
            return Ok(());
        }

        // Update FPS counter with more accurate timing
        self.frame_count += 1;
        let fps_interval = Duration::from_secs(1);
        if now.duration_since(self.last_fps_update) >= fps_interval {
            self.current_fps =
                self.frame_count as f64 / now.duration_since(self.last_fps_update).as_secs_f64();
            self.frame_count = 0;
            self.last_fps_update = now;
            self.status_bar.set_fps(self.current_fps);
        }

        // Update pattern with actual elapsed time
        self.engine.update(frame_time.as_secs_f64());

        // Update engine and colors
        let visible_range = self.scroll.get_visible_range();
        self.buffer.update_colors(&self.engine, visible_range.0)?;

        // Draw frame with current viewport
        let mut stdout = self.terminal.stdout();
        self.buffer.render_region(
            &mut stdout,
            visible_range.0,
            visible_range.1,
            self.terminal.colors_enabled(),
            true,
        )?;

        // Update status bar
        self.status_bar.render(&mut stdout, &self.scroll)?;

        stdout.flush()?;
        self.last_frame = Some(now);

        Ok(())
    }

    /// Handles terminal resize events
    pub fn handle_resize(&mut self, new_width: u16, new_height: u16) -> Result<(), RendererError> {
        // Validate dimensions
        if new_width == 0 || new_height == 0 {
            return Err(RendererError::InvalidConfig(
                "Invalid terminal dimensions".to_string(),
            ));
        }

        self.terminal.resize(new_width, new_height)?;
        self.scroll.update_viewport(new_height.saturating_sub(2));
        self.buffer.resize((new_width, new_height))?;
        self.status_bar.resize((new_width, new_height));

        // Validate scroll state after resize
        self.scroll.validate_viewport();

        self.draw_full_screen()?;
        Ok(())
    }

    /// Handles keyboard input events
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool, RendererError> {
        match key.code {
            KeyCode::Char('t') | KeyCode::Char('T') => {
                self.next_theme()?;
                self.draw_full_screen()?;
                Ok(true)
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.next_pattern()?;
                self.draw_full_screen()?;
                Ok(true)
            }
            _ => match self.scroll.handle_key_event(key) {
                Action::Continue => {
                    // Update colors and render without clearing screen
                    let visible_range = self.scroll.get_visible_range();
                    self.buffer.update_colors(&self.engine, visible_range.0)?;
                    let mut stdout = self.terminal.stdout();
                    self.buffer.render_region(
                        &mut stdout,
                        visible_range.0,
                        visible_range.1,
                        self.terminal.colors_enabled(),
                        true,
                    )?;
                    self.status_bar.render(&mut stdout, &self.scroll)?;
                    stdout.flush()?;
                    Ok(true)
                }
                Action::Exit => Ok(false),
                Action::NoChange => Ok(true),
            },
        }
    }

    // Private helper methods

    fn draw_full_screen(&mut self) -> Result<(), RendererError> {
        let mut stdout = self.terminal.stdout();
        let visible_range = self.scroll.get_visible_range();

        // Draw content and status bar atomically
        self.buffer.render_region(
            &mut stdout,
            visible_range.0,
            visible_range.1,
            self.terminal.colors_enabled(),
            true,
        )?;
        self.status_bar.render(&mut stdout, &self.scroll)?;

        stdout.flush()?;
        Ok(())
    }

    fn update_visible_region(&mut self) -> Result<(), RendererError> {
        let visible_range = self.scroll.get_visible_range();
        self.buffer.update_colors(&self.engine, visible_range.0)?;
        Ok(())
    }

    /// Switches to the next available theme
    pub fn next_theme(&mut self) -> Result<(), RendererError> {
        // Calculate next index
        self.current_theme_index = (self.current_theme_index + 1) % self.available_themes.len();
        let new_theme = &self.available_themes[self.current_theme_index];

        // Create new gradient from theme
        let theme = themes::get_theme(new_theme)?;
        let gradient = theme.create_gradient()?;

        // Update engine with new gradient and theme name
        self.engine.update_gradient(gradient);

        // Update theme name in engine's config
        let mut config = self.engine.config().clone();
        config.common.theme_name = Some(new_theme.clone());
        self.engine.update_pattern_config(config);

        // Update status bar
        self.status_bar.set_theme(new_theme);

        // Force refresh
        self.update_visible_region()?;

        Ok(())
    }

    /// Switches to the next available pattern
    pub fn next_pattern(&mut self) -> Result<(), RendererError> {
        // Calculate next index
        self.current_pattern_index =
            (self.current_pattern_index + 1) % self.available_patterns.len();
        let new_pattern_id = &self.available_patterns[self.current_pattern_index];

        // Get default parameters for the new pattern
        let pattern_params = REGISTRY
            .create_pattern_params(new_pattern_id)
            .ok_or_else(|| {
                RendererError::InvalidConfig(format!(
                    "Failed to create parameters for pattern: {}",
                    new_pattern_id
                ))
            })?;

        // Create new pattern configuration while preserving common parameters
        let pattern_config = PatternConfig {
            common: self.engine.config().common.clone(),
            params: pattern_params,
        };

        // Update engine with new pattern config
        self.engine.update_pattern_config(pattern_config);
        self.status_bar.set_pattern(new_pattern_id);

        // Force complete refresh
        self.terminal.clear_screen()?;
        self.update_visible_region()?;

        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        // Attempt to cleanup terminal state
        if let Err(e) = self.terminal.cleanup() {
            eprintln!("Error cleaning up terminal: {}", e);
        }
    }
}
