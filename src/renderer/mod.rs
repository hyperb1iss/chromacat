//! Terminal rendering system for ChromaCat
//!
//! This module provides functionality for rendering colored text output to the terminal,
//! handling both static and animated displays. It manages terminal state, text buffers,
//! color calculations, scrolling, and status information display.
//!
//! The rendering system is built around several key components:
//! - Terminal state management and interaction
//! - Text and color buffer handling
//! - Pattern-based color generation
//! - Scrolling and viewport control
//! - Status bar rendering

mod buffer;
mod config;
mod error;
mod scroll;
mod status_bar;
mod terminal;

pub use buffer::RenderBuffer;
pub use config::AnimationConfig;
pub use error::RendererError;
pub use scroll::{Action, ScrollState};
pub use status_bar::StatusBar;
pub use terminal::TerminalState;

use crate::pattern::PatternEngine;
use crossterm::event::KeyEvent;
use std::io::Write;
use std::time::Duration;
use crossterm::event::KeyCode;
use crate::themes;
use crate::pattern::{PatternConfig, PatternParams};
use crate::cli::PatternKind;

/// Coordinates all rendering functionality for ChromaCat
pub struct Renderer {
    /// Pattern generation engine
    engine: PatternEngine,
    /// Animation configuration
    config: AnimationConfig,
    /// Buffer for text and colors
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
    /// Available pattern types
    available_patterns: Vec<PatternKind>,
    /// Current pattern index
    current_pattern_index: usize,
}

impl Renderer {
    /// Creates a new renderer with the given pattern engine and configuration
    pub fn new(engine: PatternEngine, config: AnimationConfig) -> Result<Self, RendererError> {
        let terminal = TerminalState::new()?;
        let term_size = terminal.size();
        let buffer = RenderBuffer::new(term_size);
        let scroll = ScrollState::new(term_size.1.saturating_sub(2));
        let status_bar = StatusBar::new(term_size);

        // Initialize available themes
        let available_themes = themes::all_themes()
            .iter()
            .map(|t| t.name.clone())
            .collect();

        // Initialize available patterns
        let available_patterns = vec![
            PatternKind::Horizontal,
            PatternKind::Diagonal,
            PatternKind::Plasma,
            PatternKind::Ripple,
            PatternKind::Wave,
            PatternKind::Spiral,
            PatternKind::Checkerboard,
            PatternKind::Diamond,
            PatternKind::Perlin,
        ];

        Ok(Self {
            engine,
            config,
            buffer,
            terminal,
            scroll,
            status_bar,
            available_themes,
            current_theme_index: 0,
            available_patterns,
            current_pattern_index: 0,
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

    /// Renders static text with pattern-based colors
    /// Renders static text with pattern-based colors, advancing the pattern
    /// for each line to create a flowing effect similar to lolcat
    pub fn render_static(&mut self, text: &str) -> Result<(), RendererError> {
        // Don't clear screen or enter alternate screen for static mode
        self.buffer.prepare_text(text)?;

        // Use static color update mode
        self.buffer.update_colors_static(&mut self.engine)?;

        // Render the buffer directly to stdout without clearing
        let mut stdout = self.terminal.stdout();
        self.buffer.render_region(
            &mut stdout,
            0,
            self.buffer.line_count(),
            self.terminal.colors_enabled(),
        )?;

        // Flush stdout
        stdout.flush()?;
        Ok(())
    }

    /// Renders a single animation frame
    pub fn render_frame(&mut self, text: &str, delta_seconds: f64) -> Result<(), RendererError> {
        // First-time initialization
        if !self.buffer.has_content() {
            self.terminal.enter_alternate_screen()?;
            self.buffer.prepare_text(text)?;
            self.scroll.set_total_lines(self.buffer.line_count());
            self.buffer.update_colors(&self.engine)?;
            self.draw_full_screen()?;
            return Ok(());
        }

        // Update engine and colors
        self.engine.update(delta_seconds);
        self.update_visible_region()?;
        self.draw_frame()?;
        self.terminal.flush()?;

        Ok(())
    }

    /// Handles terminal resize events
    pub fn handle_resize(&mut self, new_width: u16, new_height: u16) -> Result<(), RendererError> {
        self.terminal.resize(new_width, new_height)?;
        self.scroll.update_viewport(new_height.saturating_sub(2));
        self.buffer.resize((new_width, new_height))?;
        self.status_bar.resize((new_width, new_height));
        self.draw_full_screen()?;
        Ok(())
    }

    /// Handles keyboard input events
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool, RendererError> {
        match key.code {
            KeyCode::Char('t') | KeyCode::Char('T') => {
                // Add debug logging
                eprintln!("Switching theme");
                self.next_theme()?;
                self.update_visible_region()?;
                Ok(true)
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                // Add debug logging
                eprintln!("Switching pattern");
                self.next_pattern()?;
                self.update_visible_region()?;
                Ok(true)
            }
            _ => match self.scroll.handle_key_event(key) {
                Action::Continue => {
                    self.update_visible_region()?;
                    Ok(true)
                }
                Action::Exit => Ok(false),
                Action::NoChange => Ok(true),
            }
        }
    }

    // Private helper methods

    fn draw_full_screen(&mut self) -> Result<(), RendererError> {
        self.terminal.clear_screen()?;
        let mut stdout = self.terminal.stdout();

        // Get the visible range once before the rendering calls
        let visible_range = self.scroll.get_visible_range();

        // Draw the visible region using the pre-computed range
        self.buffer.render_region(
            &mut stdout,
            visible_range.0,
            visible_range.1,
            self.terminal.colors_enabled(),
        )?;

        // Render the status bar using the pre-computed scroll state
        self.status_bar.render(&mut stdout, &self.scroll)?;
        Ok(())
    }

    fn draw_frame(&mut self) -> Result<(), RendererError> {
        let mut stdout = self.terminal.stdout();

        // Get the visible range once
        let visible_range = self.scroll.get_visible_range();

        // Draw the visible region using the pre-computed range
        self.buffer.render_region(
            &mut stdout,
            visible_range.0,
            visible_range.1,
            self.terminal.colors_enabled(),
        )?;

        // Render the status bar
        self.status_bar.render(&mut stdout, &self.scroll)?;
        Ok(())
    }

    fn update_visible_region(&mut self) -> Result<(), RendererError> {
        // Update the entire buffer in animation mode
        self.buffer.update_colors(&self.engine)
    }

    /// Switches to the next available theme
    pub fn next_theme(&mut self) -> Result<(), RendererError> {
        self.current_theme_index = (self.current_theme_index + 1) % self.available_themes.len();
        let new_theme = &self.available_themes[self.current_theme_index];
        
        eprintln!("Switching to theme: {}", new_theme); // Debug logging
        
        // Create new gradient from theme
        let theme = themes::get_theme(new_theme)?;
        let gradient = theme.create_gradient()?;
        
        // Update engine with new gradient
        self.engine.update_gradient(gradient);
        self.status_bar.set_theme(new_theme);
        
        // Force refresh
        self.update_visible_region()?;
        
        Ok(())
    }

    /// Switches to the next available pattern
    pub fn next_pattern(&mut self) -> Result<(), RendererError> {
        self.current_pattern_index = (self.current_pattern_index + 1) % self.available_patterns.len();
        let new_pattern = self.available_patterns[self.current_pattern_index];
        
        eprintln!("Switching to pattern: {:?}", new_pattern); // Debug logging
        
        // Create new pattern configuration
        let pattern_config = PatternConfig {
            common: self.engine.config().common.clone(),
            params: match new_pattern {
                PatternKind::Horizontal => PatternParams::Horizontal(Default::default()),
                PatternKind::Diagonal => PatternParams::Diagonal(Default::default()),
                PatternKind::Plasma => PatternParams::Plasma(Default::default()),
                PatternKind::Ripple => PatternParams::Ripple(Default::default()),
                PatternKind::Wave => PatternParams::Wave(Default::default()),
                PatternKind::Spiral => PatternParams::Spiral(Default::default()),
                PatternKind::Checkerboard => PatternParams::Checkerboard(Default::default()),
                PatternKind::Diamond => PatternParams::Diamond(Default::default()),
                PatternKind::Perlin => PatternParams::Perlin(Default::default()),
            },
        };
        
        // Update engine with new pattern
        self.engine.update_pattern_config(pattern_config);
        self.status_bar.set_pattern(&new_pattern.to_string());
        
        // Force refresh
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
