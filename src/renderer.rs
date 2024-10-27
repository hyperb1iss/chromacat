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
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use std::fmt::Write as FmtWrite;
use std::io::{stdout, Write};
use std::thread;
use std::time::{Duration, Instant};
use std::{cmp::min, f64::consts::PI};
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
        enable_raw_mode()?; // Enable raw mode
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
        // Remove or comment out the debug prints
        // eprintln!(
        //     "Debug - top_line: {}, viewport_height: {}, total_lines: {}, color_buffer_len: {}",
        //     self.scroll_state.top_line,
        //     self.scroll_state.viewport_height,
        //     self.scroll_state.total_lines,
        //     self.color_buffer.len()
        // );

        match key.code {
            KeyCode::PageUp => {
                if self.scroll_state.top_line > 0 {
                    let scroll_amount = self.scroll_state.viewport_height as i32 - 1;
                    self.scroll_viewport(-scroll_amount);
                }
                Ok(true)
            }
            KeyCode::PageDown => {
                let max_scroll = self
                    .scroll_state
                    .total_lines
                    .saturating_sub(self.scroll_state.viewport_height as usize);
                if self.scroll_state.top_line < max_scroll {
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
                let max_scroll = self
                    .scroll_state
                    .total_lines
                    .saturating_sub(self.scroll_state.viewport_height as usize);
                if self.scroll_state.top_line < max_scroll {
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
        let new_top = (self.scroll_state.top_line as i32 + direction).max(0);
        let max_scroll = self
            .scroll_state
            .total_lines
            .saturating_sub(self.scroll_state.viewport_height as usize);
        self.scroll_state.top_line = new_top.min(max_scroll as i32) as usize;
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

            self.update_color_buffer()?;

            // Initial full screen draw
            queue!(stdout, Hide)?;
            self.draw_full_screen(&mut stdout)?;
            stdout.flush()?;
            return Ok(());
        }

        // Increase the animation speed
        let progress = if self.config.infinite {
            elapsed.as_secs_f64() * 0.5
        } else {
            elapsed.as_secs_f64() / self.cycle_duration().as_secs_f64()
        };

        self.engine.update(progress);

        // Update colors for visible region
        let visible_lines = min(
            self.scroll_state.viewport_height as usize,
            self.scroll_state
                .total_lines
                .saturating_sub(self.scroll_state.top_line),
        );

        let end_line = min(
            self.scroll_state.top_line + visible_lines,
            self.line_buffer.len(),
        );

        self.update_color_buffer_range(self.scroll_state.top_line, end_line, progress)?;

        // **Start of changes: Optimize Rendering to Update Only Necessary Lines**

        for display_line in 0..visible_lines {
            let line_idx = self.scroll_state.top_line + display_line;
            if line_idx >= self.line_buffer.len() {
                break;
            }

            // Move cursor to the specific line
            queue!(stdout, MoveTo(0, display_line as u16))?;

            if !self.colors_enabled {
                // No color rendering
                queue!(stdout, Print(&self.line_buffer[line_idx]), Print("\x1b[K"))?;
                continue;
            }

            // Optimize color changes and rendering
            let mut current_color = None;
            let mut line_output = String::new();

            for (x, grapheme) in self.line_buffer[line_idx].graphemes(true).enumerate() {
                if x >= self.color_buffer[line_idx].len() {
                    break;
                }
                let color = self.color_buffer[line_idx][x];
                if current_color != Some(color) {
                    // Only emit color codes when color changes
                    match color {
                        Color::Rgb { r, g, b } => {
                            line_output.push_str(&format!("\x1b[38;2;{};{};{}m", r, g, b));
                        }
                        _ => {}
                    }
                    current_color = Some(color);
                }
                line_output.push_str(grapheme);
            }

            // Write the prepared line and clear to the end of the line
            queue!(stdout, Print(line_output), Print("\x1b[K"))?;
        }

        // Update status line only if necessary
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

        // Restore cursor position and attributes
        queue!(stdout, Show)?;

        // Flush all changes at once
        stdout.flush()?;

        Ok(())
    }

    // Add this new helper method
    fn draw_full_screen(&mut self, stdout: &mut std::io::StdoutLock) -> Result<()> {
        let visible_lines = min(
            self.scroll_state.viewport_height as usize,
            self.scroll_state
                .total_lines
                .saturating_sub(self.scroll_state.top_line),
        );

        let mut current_color = None;

        // Clear screen and move to top
        queue!(stdout, Print("\x1b[2J"), MoveTo(0, 0))?;

        // Render all visible lines
        for display_line in 0..visible_lines {
            let line_idx = self.scroll_state.top_line + display_line;
            if line_idx >= self.line_buffer.len() {
                break;
            }

            if !self.colors_enabled {
                queue!(stdout, Print(&self.line_buffer[line_idx]), Print("\n"))?;
                continue;
            }

            for (x, grapheme) in self.line_buffer[line_idx].graphemes(true).enumerate() {
                if x >= self.color_buffer[line_idx].len() {
                    break;
                }
                let color = self.color_buffer[line_idx][x];
                if current_color != Some(color) {
                    queue!(stdout, SetForegroundColor(color))?;
                    current_color = Some(color);
                }
                queue!(stdout, Print(grapheme))?;
            }
            queue!(stdout, Print("\n"))?;
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
            ))
        )?;

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
            .unwrap_or(0)
            .max(self.term_size.0 as usize);

        // Use the actual content height instead of terminal height
        let buffer_height = self.line_buffer.len();

        // Resize color buffer if needed
        if self.color_buffer.len() < buffer_height || self.color_buffer[0].len() < max_line_length {
            let mut new_buffer = vec![vec![Color::White; max_line_length]; buffer_height];

            // Copy existing colors
            for (y, row) in self
                .color_buffer
                .iter()
                .enumerate()
                .take(min(buffer_height, self.color_buffer.len()))
            {
                for (x, &color) in row.iter().enumerate().take(min(max_line_length, row.len())) {
                    new_buffer[y][x] = color;
                }
            }

            self.color_buffer = new_buffer;
        }

        // Update the entire content at once
        self.update_color_buffer_range(0, buffer_height, 0.0)
    }

    /// Updates colors for a range of lines
    fn update_color_buffer_range(&mut self, start: usize, end: usize, _time: f64) -> Result<()> {
        if !self.colors_enabled {
            return Ok(());
        }

        let end = min(end, self.line_buffer.len());
        let max_width = self.color_buffer[0].len();

        // Calculate total content height for proper pattern scaling
        let total_height = self.line_buffer.len();

        for y in start..end {
            let line = &self.line_buffer[y];
            for (x, _) in line.graphemes(true).enumerate() {
                if x >= max_width {
                    break;
                }

                // Use the actual y position relative to total content height
                // This ensures pattern continuity across the entire content
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

    pub fn run(&mut self) -> Result<()> {
        let frame_duration = self.frame_duration();

        loop {
            let start = Instant::now();

            // Handle input events
            // self.handle_input()?

            // Render frame
            self.render_frame("Your text here", start.elapsed())?;

            let elapsed = start.elapsed();
            if elapsed < frame_duration {
                thread::sleep(frame_duration - elapsed);
            }
        }
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
        let _ = disable_raw_mode(); // Disable raw mode
        let _ = stdout.flush();
    }
}
