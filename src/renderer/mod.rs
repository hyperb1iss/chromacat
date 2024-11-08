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
//! - Playlist management and transitions

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
use crate::playlist::{Playlist, PlaylistPlayer};
use crate::{themes, PatternConfig};
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use log::info;
use std::io::Write;
use std::time::{Duration, Instant};
use crate::input::InputReader;

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
    /// Current playlist player if using a playlist
    playlist_player: Option<PlaylistPlayer>,
    /// Current content being displayed
    content: String,
    /// Whether running in demo mode
    demo_mode: bool,
}

impl Renderer {
    /// Creates a new renderer with the given pattern engine and configuration
    pub fn new(
        engine: PatternEngine,
        config: AnimationConfig,
        playlist: Option<Playlist>,
        demo_mode: bool,
    ) -> Result<Self, RendererError> {
        let terminal = TerminalState::new()?;
        let term_size = terminal.size();
        let buffer = RenderBuffer::new(term_size);
        let scroll = ScrollState::new(term_size.1.saturating_sub(2));
        let mut status_bar = StatusBar::new(term_size);

        // Initialize available themes and patterns
        let available_themes = themes::all_themes()
            .iter()
            .map(|t| t.name.clone())
            .collect::<Vec<_>>();

        let available_patterns = crate::pattern::REGISTRY
            .list_patterns()
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        // Initialize playlist player if provided
        let playlist_player = playlist.map(PlaylistPlayer::new);

        // Get the initial engine configuration based on playlist or defaults
        let (initial_engine, initial_theme, initial_pattern) =
            if let Some(player) = &playlist_player {
                if let Some(entry) = player.current_entry() {
                    // Get configuration from first playlist entry
                    let entry_config = entry.to_pattern_config()?;
                    let entry_gradient = themes::get_theme(&entry.theme)?.create_gradient()?;

                    // Create new engine with playlist entry's configuration
                    let new_engine = PatternEngine::new(
                        entry_gradient,
                        entry_config,
                        term_size.0 as usize,
                        term_size.1 as usize,
                    );

                    (new_engine, entry.theme.clone(), entry.pattern.clone())
                } else {
                    let theme = engine
                        .config()
                        .common
                        .theme_name
                        .clone()
                        .unwrap_or_else(|| "rainbow".to_string());
                    let pattern = crate::pattern::REGISTRY
                        .get_pattern_id(&engine.config().params)
                        .unwrap_or("horizontal")
                        .to_string();
                    (engine, theme, pattern)
                }
            } else {
                let theme = engine
                    .config()
                    .common
                    .theme_name
                    .clone()
                    .unwrap_or_else(|| "rainbow".to_string());
                let pattern = crate::pattern::REGISTRY
                    .get_pattern_id(&engine.config().params)
                    .unwrap_or("horizontal")
                    .to_string();
                (engine, theme, pattern)
            };

        // Set initial theme and pattern in status bar
        status_bar.set_theme(&initial_theme);
        status_bar.set_pattern(&initial_pattern);

        // Find current indices
        let current_theme_index = available_themes
            .iter()
            .position(|t| t == &initial_theme)
            .unwrap_or(0);

        let current_pattern_index = available_patterns
            .iter()
            .position(|p| p == &initial_pattern)
            .unwrap_or(0);

        // Initialize timing state
        let now = Instant::now();
        let fps = config.fps as f64;

        Ok(Self {
            engine: initial_engine,
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
            current_fps: fps,
            playlist_player,
            content: String::new(),
            demo_mode,
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
    pub fn render_static(&mut self, text: &str) -> Result<(), RendererError> {
        // Prepare the full content
        self.buffer.prepare_text(text)?;

        // Update colors
        self.buffer.update_colors_static(&self.engine)?;

        // Get a stdout lock for efficient writing
        let mut stdout = self.terminal.stdout();

        // Render the entire buffer content
        self.buffer.render_region(
            &mut stdout,
            0,
            self.buffer.total_lines(),
            self.terminal.colors_enabled(),
            false,
        )?;

        stdout.flush()?;
        Ok(())
    }

    /// Renders a single animation frame
    pub fn render_frame(&mut self, text: &str, delta_seconds: f64) -> Result<(), RendererError> {
        let frame_time = Duration::from_secs_f64(delta_seconds);

        // Handle playlist updates if active
        let needs_update = if let Some(player) = &mut self.playlist_player {
            info!(
                "Updating playlist: current_entry={:?}, time={:?}",
                player.current_entry().map(|e| &e.pattern),
                frame_time
            );
            player.update(frame_time)
        } else {
            false
        };

        if needs_update {
            info!("Playlist entry changed, updating configuration");
            self.update_playlist_entry()?;
        }

        // Update playlist status display
        if let Some(player) = &self.playlist_player {
            if let Some(entry) = player.current_entry() {
                let status = if player.is_paused() {
                    "Paused"
                } else {
                    "Playing"
                };
                self.status_bar.set_custom_text(Some(&format!(
                    "{} - {} [{:.0}%]",
                    status,
                    entry.name,
                    player.current_progress() * 100.0
                )));
            }
        }

        // First-time initialization
        if !self.buffer.has_content() {
            self.terminal.enter_alternate_screen()?;
            self.buffer.prepare_text(text)?;
            self.scroll.set_total_lines(self.buffer.line_count());
            let visible_range = self.scroll.get_visible_range();
            self.buffer.update_colors(&self.engine, visible_range.0)?;
            self.draw_full_screen()?;
            self.last_frame = Some(Instant::now());
            self.last_fps_update = Instant::now();
            return Ok(());
        }

        // Update pattern animation
        self.engine.update(delta_seconds);

        // Update colors and render
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

        // Update FPS counter
        self.frame_count += 1;
        let now = Instant::now();
        if now.duration_since(self.last_fps_update) >= Duration::from_secs(1) {
            self.current_fps = self.frame_count as f64;
            self.frame_count = 0;
            self.last_fps_update = now;
            self.status_bar.set_fps(self.current_fps);
        }

        // Update status bar
        self.status_bar.render(&mut stdout, &self.scroll)?;

        stdout.flush()?;
        self.last_frame = Some(now);

        Ok(())
    }

    /// Handles terminal resize events
    pub fn handle_resize(&mut self, new_width: u16, new_height: u16) -> Result<(), RendererError> {
        self.terminal.resize(new_width, new_height)?;
        self.scroll.update_viewport(new_height.saturating_sub(2));
        self.buffer.resize((new_width, new_height))?;
        self.status_bar.resize((new_width, new_height));
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
            // Playlist controls
            KeyCode::Char(' ') if self.playlist_player.is_some() => {
                if let Some(player) = &mut self.playlist_player {
                    player.toggle_pause();
                    if let Some(entry) = player.current_entry() {
                        let status = if player.is_paused() {
                            "Paused"
                        } else {
                            "Playing"
                        };
                        self.status_bar.set_custom_text(Some(&format!(
                            "{} - {} [{:.0}%]",
                            status,
                            entry.name,
                            player.current_progress() * 100.0
                        )));
                    }
                }
                Ok(true)
            }
            KeyCode::Right if self.playlist_player.is_some() => {
                if let Some(player) = &mut self.playlist_player {
                    player.next_entry();
                    self.update_playlist_entry()?;
                }
                Ok(true)
            }
            KeyCode::Left if self.playlist_player.is_some() => {
                if let Some(player) = &mut self.playlist_player {
                    player.previous_entry();
                    self.update_playlist_entry()?;
                }
                Ok(true)
            }
            _ => match self.scroll.handle_key_event(key) {
                Action::Continue => {
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

    fn update_playlist_entry(&mut self) -> Result<(), RendererError> {
        if let Some(player) = &mut self.playlist_player {
            if let Some(entry) = player.current_entry() {
                let new_config = entry.to_pattern_config()?;
                let new_gradient = themes::get_theme(&entry.theme)?.create_gradient()?;

                self.engine.update_gradient(new_gradient);
                self.engine.update_pattern_config(new_config);

                // Update art type for demo mode
                if self.demo_mode {
                    if let Some(art) = entry.art {
                        // Create new input reader with the entry's art type
                        let mut reader = InputReader::from_demo(true, None, Some(&art))?;
                        let mut new_content = String::new();
                        reader.read_to_string(&mut new_content)?;
                        self.content = new_content;
                        
                        // Prepare the new content for rendering
                        self.buffer.prepare_text(&self.content)?;
                        self.scroll.set_total_lines(self.buffer.line_count());
                    }
                }

                // Update status bar
                self.status_bar.set_pattern(&entry.pattern);
                self.status_bar.set_theme(&entry.theme);
            }
        }
        Ok(())
    }

    /// Switches to the next available theme
    fn next_theme(&mut self) -> Result<(), RendererError> {
        // Increment theme index
        self.current_theme_index = (self.current_theme_index + 1) % self.available_themes.len();
        let new_theme = &self.available_themes[self.current_theme_index];

        // Update theme
        let new_gradient = themes::get_theme(new_theme)?.create_gradient()?;
        self.engine.update_gradient(new_gradient);

        // Update status bar
        self.status_bar.set_theme(new_theme);

        Ok(())
    }

    /// Switches to the next available pattern
    fn next_pattern(&mut self) -> Result<(), RendererError> {
        // Increment pattern index
        self.current_pattern_index =
            (self.current_pattern_index + 1) % self.available_patterns.len();
        let new_pattern = &self.available_patterns[self.current_pattern_index];

        // Create new pattern config
        let new_config = PatternConfig {
            common: self.engine.config().common.clone(),
            params: crate::pattern::REGISTRY
                .create_pattern_params(new_pattern)
                .ok_or_else(|| RendererError::InvalidPattern(new_pattern.clone()))?,
        };

        // Update engine
        self.engine.update_pattern_config(new_config);

        // Update status bar
        self.status_bar.set_pattern(new_pattern);

        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        if let Err(e) = self.terminal.cleanup() {
            eprintln!("Error cleaning up terminal: {}", e);
        }
    }
}
