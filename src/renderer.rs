//! Renderer module for ChromaCat
//!
//! This module handles the rendering of colored text output using patterns and gradients.
//! It supports both static and animated rendering modes, with configurable parameters
//! for animation speed, frame rate, and display options.

use crate::error::Result;
use crate::pattern::PatternEngine;
use crossterm::{
    cursor::MoveTo,
    execute, queue,
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
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
}

impl Renderer {
    /// Creates a new renderer instance
    ///
    /// # Arguments
    /// * `engine` - Pattern generation engine
    /// * `config` - Animation configuration
    ///
    /// # Returns
    /// * `Result<Self>` - New renderer instance or error
    pub fn new(engine: PatternEngine, config: AnimationConfig) -> Result<Self> {
        let term_size = terminal::size()?;
        let colors_enabled = atty::is(atty::Stream::Stdout);

        Ok(Self {
            engine,
            config,
            term_size,
            line_buffer: Vec::new(),
            color_buffer: vec![vec![Color::White; term_size.0 as usize]; term_size.1 as usize],
            colors_enabled,
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
    /// # Arguments
    /// * `text` - Text to render
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn render_static(&mut self, text: &str) -> Result<()> {
        self.prepare_text_buffer(text)?;
        self.update_color_buffer()?;
        self.render_frame_content()?;

        let mut stdout = stdout();
        queue!(stdout, ResetColor)?;
        stdout.flush()?;

        Ok(())
    }

    /// Renders a single animation frame
    ///
    /// # Arguments
    /// * `text` - Text to render
    /// * `elapsed` - Time elapsed since animation start
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn render_frame(&mut self, text: &str, elapsed: Duration) -> Result<()> {
        self.prepare_text_buffer(text)?;

        // Calculate animation progress
        let cycle_progress = if self.config.infinite {
            (elapsed.as_secs_f64() % self.cycle_duration().as_secs_f64())
                / self.cycle_duration().as_secs_f64()
        } else {
            elapsed.as_secs_f64() / self.cycle_duration().as_secs_f64()
        };

        // Update pattern and render
        self.engine.update(cycle_progress);
        self.update_color_buffer()?;
        self.render_frame_content()?;

        // Show progress bar if enabled
        if self.config.show_progress {
            self.render_progress_bar(cycle_progress)?;
        }

        stdout().flush()?;
        Ok(())
    }

    /// Prepares the text buffer for rendering by splitting into lines and handling wrapping
    ///
    /// # Arguments
    /// * `text` - Input text to prepare
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    fn prepare_text_buffer(&mut self, text: &str) -> Result<()> {
        debug!("Preparing text buffer");
        self.line_buffer.clear();

        let mut current_line = String::new();
        let mut current_width = 0;

        // Process text grapheme by grapheme
        for grapheme in text.graphemes(true) {
            let width = grapheme.width();

            // Handle line wrapping
            if current_width + width > self.term_size.0 as usize {
                self.line_buffer.push(current_line);
                current_line = String::new();
                current_width = 0;
            }

            current_line.push_str(grapheme);
            current_width += width;
        }

        // Add final line if not empty
        if !current_line.is_empty() {
            self.line_buffer.push(current_line);
        }

        trace!("Prepared {} lines", self.line_buffer.len());
        Ok(())
    }

    /// Updates the color buffer based on current pattern values
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    fn update_color_buffer(&mut self) -> Result<()> {
        if !self.colors_enabled {
            return Ok(());
        }

        debug!("Updating color buffer");
        for (y, line) in self.line_buffer.iter().enumerate() {
            if y >= self.term_size.1 as usize {
                break;
            }

            let mut x_pos = 0;
            for grapheme in line.graphemes(true) {
                if x_pos >= self.term_size.0 as usize {
                    break;
                }

                let pattern_value = self.engine.get_value_at(x_pos, y)?;
                self.color_buffer[y][x_pos] = self.value_to_color(pattern_value);

                x_pos += grapheme.width();
            }
        }

        Ok(())
    }

    /// Renders the current frame content to the terminal
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    fn render_frame_content(&self) -> Result<()> {
        let mut stdout = stdout();
        queue!(stdout, Clear(ClearType::All))?;

        for (y, line) in self.line_buffer.iter().enumerate() {
            if y >= self.term_size.1 as usize {
                break;
            }

            queue!(stdout, MoveTo(0, y as u16))?;

            if !self.colors_enabled {
                queue!(stdout, SetForegroundColor(Color::White))?;
                write!(stdout, "{}", line)?;
                continue;
            }

            let mut x_pos = 0;
            for grapheme in line.graphemes(true) {
                if x_pos >= self.term_size.0 as usize {
                    break;
                }

                queue!(stdout, SetForegroundColor(self.color_buffer[y][x_pos]))?;
                write!(stdout, "{}", grapheme)?;

                x_pos += grapheme.width();
            }
        }

        Ok(())
    }

    /// Renders the animation progress bar
    ///
    /// # Arguments
    /// * `progress` - Current animation progress (0.0 to 1.0)
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    fn render_progress_bar(&self, progress: f64) -> Result<()> {
        let bar_width = self.term_size.0.saturating_sub(20) as usize;
        let filled = (progress * bar_width as f64).min(bar_width as f64) as usize;

        let mut stdout = stdout();
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

        Ok(())
    }

    /// Converts a pattern value to a terminal color
    ///
    /// # Arguments
    /// * `value` - Pattern value (0.0 to 1.0)
    ///
    /// # Returns
    /// * `Color` - Terminal color
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
        // Ensure we reset terminal colors on drop
        if let Err(e) = execute!(stdout(), ResetColor) {
            eprintln!("Error resetting colors: {}", e);
        }
    }
}

/// Utility function to convert HSV color values to RGB
///
/// # Arguments
/// * `h` - Hue (0.0 to 1.0)
/// * `s` - Saturation (0.0 to 1.0)
/// * `v` - Value (0.0 to 1.0)
///
/// # Returns
/// * `(f64, f64, f64)` - RGB color components (0.0 to 1.0)
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
