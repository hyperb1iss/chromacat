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
        self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::cmp::min;
use std::fmt::Write as FmtWrite;
use std::io::{stdout, Write};
use std::thread;
use std::time::{Duration, Instant};
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
#[derive(Debug, Default)]
struct ScrollState {
    /// Index of the first visible line
    top_line: usize,
    /// Number of lines that fit in the viewport
    viewport_height: u16,
    /// Total number of lines in the content
    total_lines: usize,
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
    /// Original unwrapped text for proper re-wrapping on resize
    original_text: String,
    /// Last frame time
    last_frame: Instant,
}

impl Renderer {
    /// Creates a new renderer instance
    pub fn new(engine: PatternEngine, config: AnimationConfig) -> Result<Self> {
        enable_raw_mode()?; // Enable raw mode
        execute!(stdout(), Hide)?;
        let term_size = terminal::size()?;
        // Always enable colors regardless of stdout type
        let colors_enabled = true;
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
            original_text: String::new(), // **Initialize Field**
            last_frame: Instant::now(), // Add this field to track the last frame's time
        })
    }

    /// Returns the frame duration based on configured FPS
    #[inline]
    pub fn frame_duration(&self) -> Duration {
        // Ensure FPS is at least 1 to prevent division by zero
        let fps = self.config.fps.max(1);
        // Calculate frame duration in nanoseconds for better precision
        let nanos = 1_000_000_000u64 / fps as u64;
        Duration::from_nanos(nanos)
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

        // Clone text first, then use it
        let text_copy = text.to_string();
        self.original_text = text_copy.clone();
        self.prepare_text_buffer(&text_copy)?;
        self.update_color_buffer()?;

        // Build entire output in memory first
        let mut output =
            String::with_capacity(self.line_buffer.iter().map(|l| l.len() + 2).sum::<usize>());

        // Render each line
        for (y, line) in self.line_buffer.iter().enumerate() {
            if !self.colors_enabled {
                output.push_str(line);
                output.push('\r');
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
                        if let Color::Rgb { r, g, b } = color {
                            write!(output, "\x1b[38;2;{};{};{}m", r, g, b)?;
                        }
                        current_color = Some(color);
                    }
                }
                output.push_str(grapheme);
            }
            output.push('\r');
            output.push('\n');
        }

        // Reset colors at end and write everything at once
        write!(stdout, "{}\x1b[0m", output)?;
        stdout.flush()?;

        Ok(())
    }

    /// Renders a single animation frame
    pub fn render_frame(&mut self, text: &str, delta_seconds: f64) -> Result<()> {
        let mut stdout = stdout().lock();

        // First-time initialization
        if self.line_buffer.is_empty() {
            execute!(stdout, EnterAlternateScreen)?;
            execute!(stdout, Hide)?;
            self.alternate_screen = true;

            // Clone text first, then use it
            let text_copy = text.to_string();
            self.original_text = text_copy.clone();
            self.prepare_text_buffer(&text_copy)?;

            self.scroll_state = ScrollState {
                top_line: 0,
                // Reserve 2 full lines for the status bar
                viewport_height: self.term_size.1.saturating_sub(2),
                total_lines: self.line_buffer.len(),
            };

            self.update_color_buffer()?;

            // Initial full screen draw
            self.draw_full_screen(&mut stdout)?;
            stdout.flush()?;
            return Ok(());
        }

        // Update the engine with the delta time
        self.engine.update(delta_seconds);

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

        self.update_color_buffer_range(self.scroll_state.top_line, end_line, 0.0)?; // Removed delta_seconds from here

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
                    if let Color::Rgb { r, g, b } = color {
                        line_output.push_str(&format!("\x1b[38;2;{};{};{}m", r, g, b));
                    }
                    current_color = Some(color);
                }
                line_output.push_str(grapheme);
            }

            // Write the prepared line and clear to the end of the line
            queue!(stdout, Print(line_output), Print("\x1b[K"))?;
        }

        // Draw a fancy status bar
        let visible_lines = min(
            self.scroll_state.viewport_height as usize,
            self.scroll_state
                .total_lines
                .saturating_sub(self.scroll_state.top_line),
        );

        // Draw separator line
        queue!(
            stdout,
            MoveTo(0, self.term_size.1 - 2),
            Print("\x1b[K"), // Clear line
            SetForegroundColor(Color::DarkGrey),
            Print("─".repeat(self.term_size.0 as usize))
        )?;

        // Create status line with multiple segments (without progress bar)
        queue!(
            stdout,
            MoveTo(0, self.term_size.1 - 1),
            Print("\x1b[K"), // Clear line
            SetForegroundColor(Color::Rgb {
                r: 100,
                g: 100,
                b: 100
            }),
            Print("┃ "),
            SetForegroundColor(Color::Rgb {
                r: 200,
                g: 200,
                b: 200
            }),
            Print(&format!(
                "Lines {:>width$}-{}",
                self.scroll_state.top_line + 1,
                self.scroll_state.top_line + visible_lines,
                width = self.scroll_state.total_lines.to_string().len()
            )),
            SetForegroundColor(Color::Rgb {
                r: 100,
                g: 100,
                b: 100
            }),
            Print(" of "),
            SetForegroundColor(Color::Rgb {
                r: 200,
                g: 200,
                b: 200
            }),
            Print(self.scroll_state.total_lines),
            SetForegroundColor(Color::Rgb {
                r: 100,
                g: 100,
                b: 100
            }),
            Print(" ┃ "),
            // Controls section with different colors for keys
            SetForegroundColor(Color::Rgb {
                r: 180,
                g: 180,
                b: 180
            }),
            Print("↑/↓"),
            SetForegroundColor(Color::Rgb {
                r: 100,
                g: 100,
                b: 100
            }),
            Print(" or "),
            SetForegroundColor(Color::Rgb {
                r: 180,
                g: 180,
                b: 180
            }),
            Print("PgUp/PgDn"),
            SetForegroundColor(Color::Rgb {
                r: 100,
                g: 100,
                b: 100
            }),
            Print(" to scroll ┃ Press "),
            SetForegroundColor(Color::Rgb {
                r: 180,
                g: 180,
                b: 180
            }),
            Print("q"),
            SetForegroundColor(Color::Rgb {
                r: 100,
                g: 100,
                b: 100
            }),
            Print(" to quit ┃ "),
            SetForegroundColor(Color::Rgb {
                r: 255,
                g: 182,
                b: 193
            }),
            Print("😺")
        )?;

        // Keep cursor hidden during animation
        queue!(stdout, Hide)?;

        // Flush all changes at once
        stdout.flush()?;

        Ok(())
    }

    fn draw_full_screen(&mut self, stdout: &mut std::io::StdoutLock) -> Result<()> {
        // Hide cursor immediately
        queue!(stdout, Hide)?;

        let visible_lines = min(
            self.scroll_state.viewport_height as usize,
            self.scroll_state
                .total_lines
                .saturating_sub(self.scroll_state.top_line),
        );

        let mut current_color = None;

        // Clear screen and reset cursor
        queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

        // Render all visible lines
        for display_line in 0..visible_lines {
            let line_idx = self.scroll_state.top_line + display_line;
            if line_idx >= self.line_buffer.len() {
                break;
            }

            // Move to start of line
            queue!(stdout, MoveTo(0, display_line as u16))?;

            if !self.colors_enabled {
                queue!(stdout, Print(&self.line_buffer[line_idx]), Print("\x1b[K"))?;
                continue;
            }

            // Render colored line
            let mut line_output = String::new();
            for (x, grapheme) in self.line_buffer[line_idx].graphemes(true).enumerate() {
                if x >= self.color_buffer[line_idx].len() {
                    break;
                }
                let color = self.color_buffer[line_idx][x];
                if current_color != Some(color) {
                    if let Color::Rgb { r, g, b } = color {
                        line_output.push_str(&format!("\x1b[38;2;{};{};{}m", r, g, b));
                    }
                    current_color = Some(color);
                }
                line_output.push_str(grapheme);
            }
            queue!(stdout, Print(line_output), Print("\x1b[K"))?;
        }

        // Clear and render status line
        queue!(
            stdout,
            MoveTo(0, self.term_size.1 - 2),
            Print("\x1b[K"), // Clear second to last line
            MoveTo(0, self.term_size.1 - 1),
            Print("\x1b[K"), // Clear last line
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
        // Store the original unwrapped text
        if self.original_text.is_empty() {
            self.original_text = text.to_string();
        }

        self.line_buffer.clear();

        // Get the maximum width available for text
        let max_width = self.term_size.0.max(1) as usize;

        for input_line in self.original_text.split('\n') {
            if input_line.is_empty() {
                self.line_buffer.push(String::new());
                continue;
            }

            let mut current_line = String::new();
            let mut current_width = 0;
            let graphemes: Vec<_> = input_line.graphemes(true).collect();

            for grapheme in graphemes {
                let width = grapheme.width();

                // If adding this grapheme would exceed the line width
                if current_width + width > max_width {
                    // Push the current line if it's not empty
                    if !current_line.is_empty() {
                        self.line_buffer.push(current_line);
                        current_line = String::new();
                        current_width = 0;
                    }
                }

                // Add the grapheme to the current line
                current_line.push_str(grapheme);
                current_width += width;
            }

            // Push any remaining content
            if !current_line.is_empty() || input_line.is_empty() {
                self.line_buffer.push(current_line);
            }
        }

        // Update total lines count
        self.scroll_state.total_lines = self.line_buffer.len();

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

        for y in start..end {
            let line = &self.line_buffer[y];
            for (x, _) in line.graphemes(true).enumerate() {
                if x >= max_width {
                    break;
                }

                // Calculate pattern value using viewport-relative position
                // This ensures consistent animation speed regardless of scroll position
                let viewport_y = y % self.term_size.1 as usize;
                let pattern_value = self.engine.get_value_at(x, viewport_y)?;
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

    pub fn handle_resize(&mut self, new_width: u16, new_height: u16) -> Result<()> {
        // Store old dimensions for comparison
        let old_width = self.term_size.0;

        // Update terminal dimensions
        self.term_size = (new_width, new_height);

        // Update viewport height while preserving scroll position
        self.scroll_state.viewport_height = new_height.saturating_sub(2);

        // Only rewrap text if width changed
        if old_width != new_width {
            // Clone the original text before using it
            let text_copy = self.original_text.clone();
            self.line_buffer.clear();
            self.prepare_text_buffer(&text_copy)?;

            // Update total lines count after rewrapping
            self.scroll_state.total_lines = self.line_buffer.len();

            // Recreate the pattern engine with new dimensions
            self.engine = self
                .engine
                .recreate(new_width as usize, new_height as usize);

            // Calculate maximum line length after rewrapping
            let max_line_length = self
                .line_buffer
                .iter()
                .map(|line| line.graphemes(true).count())
                .max()
                .unwrap_or(0)
                .max(new_width as usize);

            // Resize color buffer for new dimensions
            self.color_buffer = vec![vec![Color::White; max_line_length]; self.line_buffer.len()];

            // Update the entire color buffer with new pattern values
            self.update_color_buffer()?;
        }

        // Adjust scroll position if viewport got smaller or content was rewrapped
        let max_scroll = self
            .scroll_state
            .total_lines
            .saturating_sub(self.scroll_state.viewport_height as usize);
        self.scroll_state.top_line = self.scroll_state.top_line.min(max_scroll);

        // Clear screen and redraw everything
        let mut stdout = stdout().lock();
        queue!(stdout, Clear(ClearType::All), MoveTo(0, 0), Hide)?;

        // Redraw the screen
        self.draw_full_screen(&mut stdout)?;
        stdout.flush()?;

        Ok(())
    }

    /// Runs the rendering loop
    pub fn run(&mut self) -> Result<()> {
        let frame_duration = self.frame_duration();
        self.last_frame = Instant::now(); // Initialize last_frame

        loop {
            let now = Instant::now();
            let delta_time = now.duration_since(self.last_frame).as_secs_f64();
            self.last_frame = now;

            // Clone the original text before passing it to render_frame
            let text_copy = self.original_text.clone();
            self.render_frame(&text_copy, delta_time)?;

            // Calculate how long the frame took to render
            let frame_elapsed = Instant::now().duration_since(now);
            
            // Calculate remaining time to sleep to maintain frame rate
            if frame_elapsed < frame_duration {
                thread::sleep(frame_duration - frame_elapsed);
            }
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let mut stdout = stdout();

        if self.alternate_screen {
            let _ = execute!(stdout, LeaveAlternateScreen, Show); // Only show cursor here
            self.alternate_screen = false;
        } else {
            let _ = execute!(stdout, Show); // Show cursor if we weren't in alternate screen
        }
        let _ = execute!(stdout, ResetColor);
        let _ = disable_raw_mode();
        let _ = stdout.flush();
    }
}
