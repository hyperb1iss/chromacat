//! Buffer management for text and colors
//!
//! This module handles the storage and manipulation of text content and
//! associated color information for rendering. It provides efficient
//! buffer management and updates while supporting Unicode text.

use crossterm::style::Color;
use std::io::Write;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::error::RendererError;
use crate::pattern::PatternEngine;

/// Manages text content and color information for rendering.
/// Provides efficient storage and updates for text content and associated colors.
#[derive(Debug)]
pub struct RenderBuffer {
    /// Buffer for text lines with pre-allocated capacity
    line_buffer: Vec<String>,
    /// Buffer for color information, organized by line
    color_buffer: Vec<Vec<Color>>,
    /// Terminal dimensions (width, height)
    term_size: (u16, u16),
    /// Original unwrapped text content
    original_text: String,
}

impl RenderBuffer {
    /// Creates a new render buffer with pre-allocated capacity based on terminal size
    #[inline]
    pub fn new(term_size: (u16, u16)) -> Self {
        Self {
            line_buffer: Vec::with_capacity(term_size.1 as usize),
            color_buffer: Vec::new(),
            term_size,
            original_text: String::with_capacity(1024), // Pre-allocate reasonable size
        }
    }

    /// Checks if buffer contains any content
    #[inline]
    pub fn has_content(&self) -> bool {
        !self.line_buffer.is_empty()
    }

    /// Returns the number of lines in the buffer
    #[inline]
    pub fn line_count(&self) -> usize {
        self.line_buffer.len()
    }

    /// Prepares text content by handling wrapping and line breaks.
    /// Efficiently processes text into lines while respecting terminal width and Unicode.
    pub fn prepare_text(&mut self, text: &str) -> Result<(), RendererError> {
        // Pre-allocate with estimated capacity
        let estimated_lines = text.chars().filter(|&c| c == '\n').count() + 1;
        self.line_buffer.clear();
        self.line_buffer.reserve(estimated_lines);
        self.original_text = text.to_string();

        let max_width = self.term_size.0.max(1) as usize;

        // Process each line more efficiently
        for input_line in text.split('\n') {
            if input_line.is_empty() {
                self.line_buffer.push(String::new());
                continue;
            }

            let mut current_line = String::with_capacity(max_width);
            let mut current_width = 0;
            let graphemes: Vec<_> = input_line.graphemes(true).collect();

            for grapheme in graphemes {
                let width = grapheme.width();

                if current_width + width > max_width && !current_line.is_empty() {
                    self.line_buffer.push(current_line);
                    current_line = String::with_capacity(max_width);
                    current_width = 0;
                }

                current_line.push_str(grapheme);
                current_width += width;
            }

            if !current_line.is_empty() || input_line.is_empty() {
                self.line_buffer.push(current_line);
            }
        }

        self.resize_color_buffer()?;
        Ok(())
    }

    /// Updates color information for the entire buffer using pattern-based generation.
    /// Efficiently calculates colors for each character position using normalized coordinates.
    pub fn update_colors(&mut self, engine: &PatternEngine) -> Result<(), RendererError> {
        let max_width = self.color_buffer.first().map_or(0, |row| row.len());
        let viewport_height = self.term_size.1 as usize;

        // Pre-calculate constants for coordinate normalization
        let width_f = max_width as f64;
        let height_f = viewport_height as f64;

        // Process each line efficiently
        for (y, line) in self.line_buffer.iter().enumerate() {
            let viewport_y = y % viewport_height;
            let norm_y = (viewport_y as f64 / height_f) - 0.5;

            for (x, _) in line.graphemes(true).enumerate() {
                if x >= max_width {
                    break;
                }

                // Calculate normalized coordinates once per iteration
                let norm_x = (x as f64 / width_f) - 0.5;

                // Get pattern value and color in one pass
                let pattern_value = engine.get_value_at_normalized(norm_x, norm_y)?;
                let gradient_color = engine.gradient().at(pattern_value as f32);

                self.color_buffer[y][x] = Color::Rgb {
                    r: (gradient_color.r * 255.0) as u8,
                    g: (gradient_color.g * 255.0) as u8,
                    b: (gradient_color.b * 255.0) as u8,
                };
            }
        }

        Ok(())
    }

    /// Updates colors in static mode, creating a flowing effect by advancing the pattern per line.
    #[inline]
    pub fn update_colors_static(
        &mut self,
        engine: &mut PatternEngine,
    ) -> Result<(), RendererError> {
        let max_width = self.color_buffer.first().map_or(0, |row| row.len());
        let width_f = max_width as f64;

        for (y, line) in self.line_buffer.iter().enumerate() {
            for (x, _) in line.graphemes(true).enumerate() {
                if x >= max_width {
                    break;
                }

                let norm_x = (x as f64 / width_f) - 0.5;
                let pattern_value = engine.get_value_at_normalized(norm_x, 0.0)?;
                let gradient_color = engine.gradient().at(pattern_value as f32);

                self.color_buffer[y][x] = Color::Rgb {
                    r: (gradient_color.r * 255.0) as u8,
                    g: (gradient_color.g * 255.0) as u8,
                    b: (gradient_color.b * 255.0) as u8,
                };
            }

            engine.update(0.1);
        }

        Ok(())
    }

    /// Renders a region of the buffer to the terminal with optimized color handling.
    pub fn render_region(
        &self,
        stdout: &mut std::io::StdoutLock,
        start: usize,
        end: usize,
        colors_enabled: bool,
    ) -> Result<(), RendererError> {
        use crossterm::{cursor::MoveTo, queue};

        let end = end.min(self.line_buffer.len());
        let is_static_mode = start == 0 && end == self.line_buffer.len();

        // Pre-allocate ANSI escape sequence buffer
        let mut ansi_buffer = String::with_capacity(32);
        let mut current_color = None;

        for (display_line, line_idx) in (start..end).enumerate() {
            if !is_static_mode {
                queue!(stdout, MoveTo(0, display_line as u16))?;
            }

            if !colors_enabled {
                writeln!(stdout, "{}", self.line_buffer[line_idx])?;
                continue;
            }

            // Process colors more efficiently
            for (x, grapheme) in self.line_buffer[line_idx].graphemes(true).enumerate() {
                if x >= self.color_buffer[line_idx].len() {
                    break;
                }

                let color = self.color_buffer[line_idx][x];
                if current_color != Some(color) {
                    if let Color::Rgb { r, g, b } = color {
                        // Format ANSI sequence into String buffer first
                        ansi_buffer.clear();
                        ansi_buffer.push_str(&format!("\x1b[38;2;{};{};{}m", r, g, b));
                        write!(stdout, "{}", ansi_buffer)?;
                    }
                    current_color = Some(color);
                }
                write!(stdout, "{}", grapheme)?;
            }

            if !is_static_mode {
                write!(stdout, "\x1b[K")?;
            }
            writeln!(stdout)?;
        }

        if colors_enabled {
            write!(stdout, "\x1b[0m")?;
        }

        Ok(())
    }

    /// Resizes the color buffer to match current dimensions while preserving data.
    #[inline]
    pub fn resize_color_buffer(&mut self) -> Result<(), RendererError> {
        let max_line_length = self
            .line_buffer
            .iter()
            .map(|line| line.graphemes(true).count())
            .max()
            .unwrap_or(0)
            .max(self.term_size.0 as usize);

        let buffer_height = self.line_buffer.len();

        // Pre-allocate with capacity
        let mut new_buffer = Vec::with_capacity(buffer_height);
        for _ in 0..buffer_height {
            new_buffer.push(vec![Color::White; max_line_length]);
        }

        self.color_buffer = new_buffer;
        Ok(())
    }

    /// Returns the maximum line length in the buffer
    #[inline]
    pub fn max_line_length(&self) -> usize {
        self.line_buffer
            .iter()
            .map(|line| line.graphemes(true).count())
            .max()
            .unwrap_or(0)
    }

    /// Resizes the buffer for new terminal dimensions while maintaining content.
    pub fn resize(&mut self, new_size: (u16, u16)) -> Result<(), RendererError> {
        self.term_size = new_size;
        // Clone the text first to avoid the borrow conflict
        let text = self.original_text.clone();
        self.prepare_text(&text)?;
        Ok(())
    }
}

impl Default for RenderBuffer {
    fn default() -> Self {
        Self::new((80, 24)) // Default terminal size
    }
}
