//! Renderer module for ChromaCat
//!
//! This module handles the rendering of colored text output using patterns and gradients.
//! It supports both static and animated rendering modes, with configurable parameters
//! for animation speed, frame rate, and display options. The renderer handles:
//!
//! - Color gradient application to text
//! - Terminal manipulation and state management
//! - Unicode text handling and proper width calculations
//! - Animation frame timing and progress display
//! - Terminal cleanup and error handling

use crate::error::Result;
use crate::pattern::PatternEngine;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{KeyCode, KeyEvent},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::cmp::min;
use std::fmt::Write as FmtWrite;
use std::io::{stdout, Write};
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Configuration for animation rendering
#[derive(Debug, Clone)]
pub struct AnimationConfig {
    /// Frames per second for animation playback (1-144)
    pub fps: u32,
    /// Duration of one complete pattern cycle
    pub cycle_duration: Duration,
    /// Whether to loop indefinitely
    pub infinite: bool,
    /// Whether to show animation progress bar
    pub show_progress: bool,
    /// Enable smooth transitions between frames
    pub smooth: bool,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            fps: 30,
            cycle_duration: Duration::from_secs(5),
            infinite: false,
            show_progress: true,
            smooth: false,
        }
    }
}

/// Scrolling state for animated viewing
#[derive(Debug)]
struct ScrollState {
    /// Index of the first visible line
    top_line: usize,
    /// Number of lines that fit in the viewport
    viewport_height: u16,
    /// Total number of lines in the content
    total_lines: usize,
}

impl Default for ScrollState {
    fn default() -> Self {
        Self {
            top_line: 0,
            viewport_height: 0,
            total_lines: 0,
        }
    }
}

/// Renders text with gradient patterns
pub struct Renderer {
    /// Pattern generation engine
    engine: PatternEngine,
    /// Animation configuration
    config: AnimationConfig,
    /// Terminal dimensions (width, height)
    term_size: (u16, u16),
    /// Buffer for rendered lines
    line_buffer: Vec<String>,
    /// Color buffer for optimization
    color_buffer: Vec<Vec<Color>>,
    /// Whether colors are enabled
    colors_enabled: bool,
    /// Whether we're in alternate screen mode
    alternate_screen: bool,
    /// Scrolling state for animated viewing
    scroll_state: ScrollState,
}

impl Renderer {
    /// Creates a new renderer instance
    pub fn new(engine: PatternEngine, config: AnimationConfig) -> Result<Self> {
        let term_size = terminal::size()?;
        let colors_enabled = atty::is(atty::Stream::Stdout);
        let color_buffer = vec![vec![Color::White; term_size.0 as usize]; term_size.1 as usize];

        Ok(Self {
            engine,
            config,
            term_size,
            line_buffer: Vec::new(),
            color_buffer,
            colors_enabled,
            alternate_screen: false,
            scroll_state: ScrollState::default(),
        })
    }

    /// Returns the frame duration based on configured FPS
    #[inline]
    pub fn frame_duration(&self) -> Duration {
        Duration::from_secs(1) / self.config.fps
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

    /// Handle scrolling key events
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::PageUp => {
                if self.scroll_state.top_line > 0 {
                    let scroll_amount = self.scroll_state.viewport_height as i32 - 1;
                    self.scroll_viewport(-scroll_amount);
                }
                Ok(true)
            }
            KeyCode::PageDown => {
                if self.scroll_state.top_line
                    < self
                        .scroll_state
                        .total_lines
                        .saturating_sub(self.scroll_state.viewport_height as usize)
                {
                    let scroll_amount = self.scroll_state.viewport_height as i32 - 1;
                    self.scroll_viewport(scroll_amount);
                }
                Ok(true)
            }
            KeyCode::Up => {
                if self.scroll_state.top_line > 0 {
                    self.scroll_viewport(-1);
                }
                Ok(true)
            }
            KeyCode::Down => {
                if self.scroll_state.top_line
                    < self
                        .scroll_state
                        .total_lines
                        .saturating_sub(self.scroll_state.viewport_height as usize)
                {
                    self.scroll_viewport(1);
                }
                Ok(true)
            }
            KeyCode::Char('q') | KeyCode::Esc => Ok(false),
            _ => Ok(true),
        }
    }

    /// Adjust viewport position when scrolling
    fn scroll_viewport(&mut self, direction: i32) {
        let new_top = self.scroll_state.top_line as i32 + direction;

        if new_top < 0 {
            self.scroll_state.top_line = 0;
            return;
        }

        let max_scroll = self
            .scroll_state
            .total_lines
            .saturating_sub(self.scroll_state.viewport_height as usize);

        self.scroll_state.top_line = min(new_top as usize, max_scroll);
    }

    /// Renders static text with pattern (non-animated mode)
    pub fn render_static(&mut self, text: &str) -> Result<()> {
        let mut stdout = stdout().lock();

        // Process text into lines and update colors
        self.prepare_text_buffer(text)?;
        self.update_color_buffer()?;

        // Build entire output in memory first
        let mut output =
            String::with_capacity(self.line_buffer.iter().map(|l| l.len() + 1).sum::<usize>());

        // Render each line
        for (y, line) in self.line_buffer.iter().enumerate() {
            if !self.colors_enabled {
                output.push_str(line);
                output.push('\n');
                continue;
            }

            // Render colored text
            let mut current_color = None;
            for (x, grapheme) in line.graphemes(true).enumerate() {
                if x < self.color_buffer[y].len() {
                    let color = self.color_buffer[y][x];
                    if current_color != Some(color) {
                        // Only emit color codes when color changes
                        match color {
                            Color::Rgb { r, g, b } => {
                                write!(output, "\x1b[38;2;{};{};{}m", r, g, b)?;
                            }
                            _ => {} // Handle other color types if needed
                        }
                        current_color = Some(color);
                    }
                }
                output.push_str(grapheme);
            }
            output.push('\n');
        }

        // Reset colors at end and write everything at once
        write!(stdout, "{}\x1b[0m", output)?;
        stdout.flush()?;

        Ok(())
    }

    /// Renders a single animation frame
    pub fn render_frame(&mut self, text: &str, elapsed: Duration) -> Result<()> {
        let mut stdout = stdout().lock();

        // First-time initialization
        if self.line_buffer.is_empty() {
            execute!(stdout, EnterAlternateScreen, Hide)?;
            self.alternate_screen = true;

            self.prepare_text_buffer(text)?;
            self.scroll_state = ScrollState {
                top_line: 0,
                viewport_height: self.term_size.1.saturating_sub(2),
                total_lines: self.line_buffer.len(),
            };
        }

        // Calculate animation progress
        let progress = if self.config.infinite {
            (elapsed.as_secs_f64() * 2.0) % 1.0
        } else {
            elapsed.as_secs_f64() / self.cycle_duration().as_secs_f64()
        };

        self.engine.update(progress);

        // Calculate visible range
        let visible_lines = min(
            self.scroll_state.viewport_height as usize,
            self.scroll_state
                .total_lines
                .saturating_sub(self.scroll_state.top_line),
        );

        // Update colors for visible region
        self.update_color_buffer_range(
            self.scroll_state.top_line,
            self.scroll_state.top_line + visible_lines,
            progress,
        )?;

        // Build frame in memory
        let mut current_color = None;

        // Render visible lines
        for (display_line, line_idx) in
            (self.scroll_state.top_line..self.scroll_state.top_line + visible_lines).enumerate()
        {
            queue!(stdout, MoveTo(0, display_line as u16))?;

            if !self.colors_enabled {
                queue!(stdout, Print(&self.line_buffer[line_idx]))?;
                continue;
            }

            // Render line with colors
            for (x, grapheme) in self.line_buffer[line_idx].graphemes(true).enumerate() {
                if x < self.color_buffer[line_idx].len() {
                    let color = self.color_buffer[line_idx][x];
                    if current_color != Some(color) {
                        queue!(stdout, SetForegroundColor(color))?;
                        current_color = Some(color);
                    }
                }
                queue!(stdout, Print(grapheme))?;
            }

            // Clear to end of line
            queue!(stdout, Print("\x1b[K"))?;
        }

        // Clear remaining lines
        for y in visible_lines..self.scroll_state.viewport_height as usize {
            queue!(stdout, MoveTo(0, y as u16), Print("\x1b[K"))?;
        }

        // Render status line
        queue!(
            stdout,
            MoveTo(0, self.term_size.1 - 1),
            SetForegroundColor(Color::White),
            Print(format!(
                "Lines {}-{}/{} [↑/↓/PgUp/PgDn to scroll, q to quit]",
                self.scroll_state.top_line + 1,
                self.scroll_state.top_line + visible_lines,
                self.scroll_state.total_lines
            )),
            Print("\x1b[K")
        )?;

        queue!(stdout, ResetColor)?;
        stdout.flush()?;
        Ok(())
    }

    /// Prepares the text buffer by handling line wrapping and newlines
    fn prepare_text_buffer(&mut self, text: &str) -> Result<()> {
        self.line_buffer.clear();

        for input_line in text.split('\n') {
            if input_line.is_empty() {
                self.line_buffer.push(String::new());
                continue;
            }

            let mut current_line = String::new();
            let mut current_width = 0;

            for grapheme in input_line.graphemes(true) {
                let width = grapheme.width();

                if current_width + width > self.term_size.0 as usize && !current_line.is_empty() {
                    self.line_buffer.push(current_line);
                    current_line = String::new();
                    current_width = 0;
                }

                current_line.push_str(grapheme);
                current_width += width;
            }

            if !current_line.is_empty() || input_line.is_empty() {
                self.line_buffer.push(current_line);
            }
        }

        Ok(())
    }

    /// Updates the color buffer for all content
    fn update_color_buffer(&mut self) -> Result<()> {
        if !self.colors_enabled {
            return Ok(());
        }

        let max_line_length = self
            .line_buffer
            .iter()
            .map(|line| line.graphemes(true).count())
            .max()
            .unwrap_or(0);

        let buffer_height = self.line_buffer.len();

        if self.color_buffer.len() < buffer_height
            || self.color_buffer.first().map_or(0, |row| row.len()) < max_line_length
        {
            self.color_buffer =
                vec![
                    vec![Color::White; max_line_length.max(self.term_size.0 as usize)];
                    buffer_height.max(self.term_size.1 as usize)
                ];
        }

        self.update_color_buffer_range(0, buffer_height, 0.0)
    }

    /// Updates colors for a range of lines
    fn update_color_buffer_range(&mut self, start: usize, end: usize, _time: f64) -> Result<()> {
        if !self.colors_enabled {
            return Ok(());
        }

        for y in start..end {
            let line = &self.line_buffer[y];
            for (x, _) in line.graphemes(true).enumerate() {
                let pattern_value = self.engine.get_value_at(x, y)?;
                let gradient_color = self.engine.gradient().at(pattern_value as f32);

                self.color_buffer[y][x] = Color::Rgb {
                    r: (gradient_color.r * 255.0) as u8,
                    g: (gradient_color.g * 255.0) as u8,
                    b: (gradient_color.b * 255.0) as u8,
                };
            }
        }

        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let mut stdout = stdout();

        if self.alternate_screen {
            let _ = execute!(stdout, Show, LeaveAlternateScreen);
            self.alternate_screen = false;
        }
        let _ = execute!(stdout, ResetColor);
        let _ = stdout.flush();
    }
}
