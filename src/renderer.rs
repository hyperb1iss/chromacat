//! Renderer module for ChromaCat
//!
//! This module handles the rendering of colored text output using patterns and gradients.
//! It supports both static and animated rendering modes, with configurable parameters
//! for animation speed, frame rate, and display options.

use crate::error::Result;
use crate::pattern::PatternEngine;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{debug, trace};
use std::io::{stdout, Write};
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Configuration for animation rendering
#[derive(Debug, Clone)]
pub struct AnimationConfig {
    /// Frames per second for animation playback
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
        })
    }

    /// Returns the frame duration based on configured FPS
    pub fn frame_duration(&self) -> Duration {
        Duration::from_secs(1) / self.config.fps
    }

    /// Returns whether animation is set to run indefinitely
    pub fn is_infinite(&self) -> bool {
        self.config.infinite
    }

    /// Returns the configured animation cycle duration
    pub fn cycle_duration(&self) -> Duration {
        self.config.cycle_duration
    }

    /// Renders static text with pattern
    ///
    /// For static rendering, we write directly to the main screen
    /// and preserve scrollback buffer
    pub fn render_static(&mut self, text: &str) -> Result<()> {
        let mut stdout = stdout();

        // Process the text
        self.prepare_text_buffer(text)?;
        self.update_color_buffer()?;

        // Render each line with colors and explicit newlines
        for (y, line) in self.line_buffer.iter().enumerate() {
            // Handle plain text mode
            if !self.colors_enabled {
                writeln!(stdout, "{}", line)?;
                continue;
            }

            // Render colored text
            let mut x_pos = 0;
            for grapheme in line.graphemes(true) {
                // Ensure we don't access beyond color buffer bounds
                if x_pos >= self.color_buffer[y].len() {
                    write!(stdout, "{}", grapheme)?;
                    continue;
                }

                queue!(stdout, SetForegroundColor(self.color_buffer[y][x_pos]))?;
                write!(stdout, "{}", grapheme)?;

                x_pos += 1;
            }

            // Always write a newline after each line
            write!(stdout, "\n")?;
        }

        // Reset colors and flush
        queue!(stdout, ResetColor)?;
        stdout.flush()?;
        Ok(())
    }

    /// Renders a single animation frame
    pub fn render_frame(&mut self, text: &str, elapsed: Duration) -> Result<()> {
        let mut stdout = stdout();

        // Enter alternate screen mode if animation
        if !self.alternate_screen {
            execute!(stdout, EnterAlternateScreen, Hide)?;
            self.alternate_screen = true;
        }

        // Calculate progress and update pattern
        let cycle_progress = if self.config.infinite {
            (elapsed.as_secs_f64() % self.cycle_duration().as_secs_f64())
                / self.cycle_duration().as_secs_f64()
        } else {
            elapsed.as_secs_f64() / self.cycle_duration().as_secs_f64()
        };

        self.engine.update(cycle_progress);

        // Render frame
        self.prepare_text_buffer(text)?;
        self.update_color_buffer()?;
        self.render_frame_content()?;

        if self.config.show_progress {
            self.render_progress_bar(cycle_progress)?;
        }

        stdout.flush()?;
        Ok(())
    }

    /// Prepares the text buffer by handling line wrapping and newlines
    fn prepare_text_buffer(&mut self, text: &str) -> Result<()> {
        self.line_buffer.clear();

        // Split text by explicit newlines first
        for input_line in text.split('\n') {
            if input_line.is_empty() {
                // Preserve empty lines
                self.line_buffer.push(String::new());
                continue;
            }

            let mut current_line = String::new();
            let mut current_width = 0;

            // Process each grapheme in the line
            for grapheme in input_line.graphemes(true) {
                let width = grapheme.width();

                // Handle line wrapping at terminal width
                if current_width + width > self.term_size.0 as usize {
                    if !current_line.is_empty() {
                        self.line_buffer.push(current_line);
                        current_line = String::new();
                        current_width = 0;
                    }
                }

                current_line.push_str(grapheme);
                current_width += width;
            }

            // Push final line segment if not empty
            if !current_line.is_empty() || input_line.is_empty() {
                self.line_buffer.push(current_line);
            }
        }

        Ok(())
    }

    /// Updates the color buffer based on pattern values
    fn update_color_buffer(&mut self) -> Result<()> {
        if !self.colors_enabled {
            return Ok(());
        }

        // Ensure color buffer is large enough for current content
        let max_line_length = self
            .line_buffer
            .iter()
            .map(|line| line.graphemes(true).count())
            .max()
            .unwrap_or(0);
        let buffer_height = self.line_buffer.len();

        // Resize buffer if needed
        if self.color_buffer.len() < buffer_height
            || self.color_buffer.first().map_or(0, |row| row.len()) < max_line_length
        {
            self.color_buffer =
                vec![
                    vec![Color::White; max_line_length.max(self.term_size.0 as usize)];
                    buffer_height.max(self.term_size.1 as usize)
                ];
        }

        // Update colors for each character
        for (y, line) in self.line_buffer.iter().enumerate() {
            let mut x_pos = 0;
            for grapheme in line.graphemes(true) {
                if x_pos >= self.color_buffer[y].len() {
                    break;
                }

                let pattern_value = self.engine.get_value_at(x_pos, y)?;
                self.color_buffer[y][x_pos] = self.value_to_color(pattern_value);

                x_pos += 1;
            }
        }

        Ok(())
    }

    /// Renders the current frame content (used for animation)
    fn render_frame_content(&self) -> Result<()> {
        let mut stdout = stdout();

        // Clear screen and move to top
        queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

        // Render each line
        for (y, line) in self.line_buffer.iter().enumerate() {
            if y >= self.term_size.1 as usize {
                break;
            }

            // Position cursor at start of line
            queue!(stdout, MoveTo(0, y as u16))?;

            // Handle plain text mode
            if !self.colors_enabled {
                writeln!(stdout, "{}", line)?;
                continue;
            }

            // Render colored text
            let mut x_pos = 0;
            for grapheme in line.graphemes(true) {
                // Ensure we don't access beyond color buffer bounds
                if x_pos >= self.color_buffer[y].len() {
                    write!(stdout, "{}", grapheme)?;
                    continue;
                }

                queue!(stdout, SetForegroundColor(self.color_buffer[y][x_pos]))?;
                write!(stdout, "{}", grapheme)?;

                x_pos += 1;
            }

            write!(stdout, "\n")?;
        }

        stdout.flush()?;
        Ok(())
    }

    /// Renders the animation progress bar
    fn render_progress_bar(&self, progress: f64) -> Result<()> {
        let mut stdout = stdout();
        let bar_width = self.term_size.0.saturating_sub(20) as usize;
        let filled = (progress * bar_width as f64).min(bar_width as f64) as usize;

        queue!(
            stdout,
            MoveTo(0, self.term_size.1.saturating_sub(1)),
            SetForegroundColor(Color::White)
        )?;

        write!(
            stdout,
            "[{}{}] {:3.0}%",
            "=".repeat(filled),
            " ".repeat(bar_width.saturating_sub(filled)),
            progress * 100.0
        )?;

        stdout.flush()?;
        Ok(())
    }

    /// Converts a pattern value to a terminal color
    fn value_to_color(&self, value: f64) -> Color {
        let clamped_value = value.clamp(0.0, 1.0);
        let gradient_color = self.engine.gradient().at(clamped_value as f32);

        Color::Rgb {
            r: (gradient_color.r * 255.0) as u8,
            g: (gradient_color.g * 255.0) as u8,
            b: (gradient_color.b * 255.0) as u8,
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let mut stdout = stdout();

        // Clean up terminal state
        if self.alternate_screen {
            let _ = execute!(stdout, Show, LeaveAlternateScreen);
        }
        let _ = execute!(stdout, ResetColor);
        let _ = stdout.flush();
    }
}

/// Utility function to convert HSV color values to RGB
fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let h = if h < 0.0 {
        h + 1.0
    } else if h > 1.0 {
        h - 1.0
    } else {
        h
    };
    let h = h * 6.0;

    let i = h.floor();
    let f = h - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    match i as i64 % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}
