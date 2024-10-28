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

/// Manages text content and color information for rendering
#[derive(Debug)]
pub struct RenderBuffer {
    /// Buffer for text lines
    line_buffer: Vec<String>,
    /// Buffer for color information
    color_buffer: Vec<Vec<Color>>,
    /// Terminal dimensions (width, height)
    term_size: (u16, u16),
    /// Original unwrapped text
    original_text: String,
}

impl RenderBuffer {
    /// Creates a new render buffer
    ///
    /// # Arguments
    /// * `term_size` - Terminal dimensions (width, height)
    pub fn new(term_size: (u16, u16)) -> Self {
        Self {
            line_buffer: Vec::new(),
            color_buffer: Vec::new(),
            term_size,
            original_text: String::new(),
        }
    }

    /// Checks if buffer contains any content
    pub fn has_content(&self) -> bool {
        !self.line_buffer.is_empty()
    }

    /// Returns the number of lines in the buffer
    pub fn line_count(&self) -> usize {
        self.line_buffer.len()
    }

    /// Prepares text content by handling wrapping and line breaks
    ///
    /// # Arguments
    /// * `text` - Text to prepare
    ///
    /// # Returns
    /// Ok(()) if successful, Error otherwise
    pub fn prepare_text(&mut self, text: &str) -> Result<(), RendererError> {
        self.original_text = text.to_string();
        self.line_buffer.clear();

        let max_width = self.term_size.0.max(1) as usize;

        for input_line in text.split('\n') {
            if input_line.is_empty() {
                self.line_buffer.push(String::new());
                continue;
            }

            let mut current_line = String::new();
            let mut current_width = 0;
            let graphemes: Vec<_> = input_line.graphemes(true).collect();

            for grapheme in graphemes {
                let width = grapheme.width();

                if current_width + width > max_width {
                    if !current_line.is_empty() {
                        self.line_buffer.push(current_line);
                        current_line = String::new();
                        current_width = 0;
                    }
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

    /// Updates color information for the entire buffer
    ///
    /// # Arguments
    /// * `engine` - Pattern engine for color generation
    ///
    /// # Returns
    /// Ok(()) if successful, Error otherwise
    pub fn update_colors(&mut self, engine: &PatternEngine) -> Result<(), RendererError> {
        self.update_colors_range(engine, 0, self.line_buffer.len())
    }

    /// Updates color information for a range of lines
    ///
    /// # Arguments
    /// * `engine` - Pattern engine for color generation
    /// * `start` - Starting line index
    /// * `end` - Ending line index (exclusive)
    ///
    /// # Returns
    /// Ok(()) if successful, Error otherwise
    pub fn update_colors_range(
        &mut self,
        engine: &PatternEngine,
        start: usize,
        end: usize,
    ) -> Result<(), RendererError> {
        let end = end.min(self.line_buffer.len());
        let max_width = self.color_buffer.get(0).map_or(0, |row| row.len());

        for y in start..end {
            let line = &self.line_buffer[y];
            for (x, _) in line.graphemes(true).enumerate() {
                if x >= max_width {
                    break;
                }

                // Calculate pattern value using viewport-relative position
                let pattern_value = engine.get_value_at(x, y)?;
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

    /// Updates color information for the entire buffer in static mode,
    /// advancing the pattern for each line to create a flowing effect
    pub fn update_colors_static(
        &mut self,
        engine: &mut PatternEngine,
    ) -> Result<(), RendererError> {
        let max_width = self.color_buffer.get(0).map_or(0, |row| row.len());

        // Calculate colors for each line, advancing the pattern
        for (y, line) in self.line_buffer.iter().enumerate() {
            // Advance pattern by a small amount for each line
            engine.update(0.1); // Small time increment per line

            for (x, _) in line.graphemes(true).enumerate() {
                if x >= max_width {
                    break;
                }

                let pattern_value = engine.get_value_at(x, y)?;
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
    /// Resizes the buffer for new terminal dimensions
    ///
    /// # Arguments
    /// * `new_size` - New terminal dimensions (width, height)
    ///
    /// # Returns
    /// Ok(()) if successful, Error otherwise
    pub fn resize(&mut self, new_size: (u16, u16)) -> Result<(), RendererError> {
        self.term_size = new_size;

        // Clone the text first to avoid borrowing conflicts
        let text = self.original_text.clone();

        // Rewrap text for new width
        self.prepare_text(&text)?;

        // Resize color buffer
        self.resize_color_buffer()?;

        Ok(())
    }

    /// Renders a region of the buffer to the terminal
    ///
    /// # Arguments
    /// * `stdout` - Terminal output handle
    /// * `start` - Starting line index
    /// * `end` - Ending line index (exclusive)
    /// * `colors_enabled` - Whether color output is enabled
    ///
    /// # Returns
    /// Ok(()) if successful, Error otherwise
    pub fn render_region(
        &self,
        stdout: &mut std::io::StdoutLock,
        start: usize,
        end: usize,
        colors_enabled: bool,
    ) -> Result<(), RendererError> {
        use crossterm::{cursor::MoveTo, queue};

        let end = end.min(self.line_buffer.len());

        for (display_line, line_idx) in (start..end).enumerate() {
            queue!(stdout, MoveTo(0, display_line as u16))?;

            if !colors_enabled {
                write!(stdout, "{}\r\n", self.line_buffer[line_idx])?;
                continue;
            }

            let mut current_color = None;
            for (x, grapheme) in self.line_buffer[line_idx].graphemes(true).enumerate() {
                if x >= self.color_buffer[line_idx].len() {
                    break;
                }

                let color = self.color_buffer[line_idx][x];
                if current_color != Some(color) {
                    if let Color::Rgb { r, g, b } = color {
                        write!(stdout, "\x1b[38;2;{};{};{}m", r, g, b)?;
                    }
                    current_color = Some(color);
                }
                write!(stdout, "{}", grapheme)?;
            }
            write!(stdout, "\x1b[K\r\n")?;
        }

        Ok(())
    }

    /// Returns the maximum line length in the buffer
    pub fn max_line_length(&self) -> usize {
        self.line_buffer
            .iter()
            .map(|line| line.graphemes(true).count())
            .max()
            .unwrap_or(0)
    }

    // Private helper methods

    pub fn resize_color_buffer(&mut self) -> Result<(), RendererError> {
        let max_line_length = self
            .line_buffer
            .iter()
            .map(|line| line.graphemes(true).count())
            .max()
            .unwrap_or(0)
            .max(self.term_size.0 as usize);

        let buffer_height = self.line_buffer.len();

        // Create new buffer first
        let new_buffer = vec![vec![Color::White; max_line_length]; buffer_height];

        // Then assign
        self.color_buffer = new_buffer;

        Ok(())
    }
}

impl Default for RenderBuffer {
    fn default() -> Self {
        Self::new((80, 24)) // Default terminal size
    }
}
