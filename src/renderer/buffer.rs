//! Buffer management for text and colors
//!
//! This module handles the storage and manipulation of text content and
//! associated color information for rendering. It provides efficient
//! buffer management and updates while supporting Unicode text through
//! double buffering for smooth display updates.

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    queue,
    style::{Color, Print},
};
use std::fmt::Write as FmtWrite;
use std::io::Write;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::error::RendererError;
use crate::pattern::PatternEngine;

/// A cell in the character buffer containing both the character and its color
#[derive(Debug, Clone, PartialEq)]
struct BufferCell {
    /// The character to display
    ch: char,
    /// The color of the character
    color: Color,
    /// Whether this cell has been modified since last swap
    dirty: bool,
}

impl Default for BufferCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            color: Color::Reset,
            dirty: false,
        }
    }
}

/// Manages text content and color information for rendering.
/// Provides efficient storage and updates for text content and associated colors
/// using double buffering for smooth display updates.
#[derive(Debug)]
pub struct RenderBuffer {
    /// Front buffer (currently displayed)
    front: Vec<Vec<BufferCell>>,
    /// Back buffer (being rendered to)
    back: Vec<Vec<BufferCell>>,
    /// Terminal dimensions (width, height)
    term_size: (u16, u16),
    /// Original unwrapped text content
    original_text: String,
    /// Line wrapping information
    line_info: Vec<(usize, usize)>, // (start, length) pairs
}

impl RenderBuffer {
    /// Creates a new render buffer with pre-allocated capacity based on terminal size
    #[inline]
    pub fn new(term_size: (u16, u16)) -> Self {
        let width = term_size.0 as usize;
        let height = term_size.1 as usize;
        let buffer = vec![vec![BufferCell::default(); width]; height];

        Self {
            front: buffer.clone(),
            back: buffer,
            term_size,
            original_text: String::with_capacity(1024), // Pre-allocate reasonable size
            line_info: Vec::with_capacity(height),
        }
    }

    /// Checks if buffer contains any content
    #[inline]
    pub fn has_content(&self) -> bool {
        !self.line_info.is_empty()
    }

    /// Returns the number of lines in the buffer
    #[inline]
    pub fn line_count(&self) -> usize {
        self.line_info.len()
    }

    /// Prepares text content by handling wrapping and line breaks.
    /// Efficiently processes text into lines while respecting terminal width and Unicode.
    pub fn prepare_text(&mut self, text: &str) -> Result<(), RendererError> {
        self.original_text = text.to_string();
        self.line_info.clear();

        let max_width = self.term_size.0.max(1) as usize;
        let mut buffer_pos = 0;

        // Pre-calculate required capacity
        let estimated_lines =
            (text.len() / max_width) + text.chars().filter(|&c| c == '\n').count() + 1;
        self.ensure_buffer_capacity(estimated_lines);

        // Process each line with efficient wrapping
        for input_line in text.split('\n') {
            if input_line.is_empty() {
                self.line_info.push((buffer_pos, 0));

                // Clear the entire line in the back buffer
                while buffer_pos >= self.back.len() {
                    self.back.push(vec![BufferCell::default(); max_width]);
                    self.front.push(vec![BufferCell::default(); max_width]);
                }

                // Mark entire line as dirty to ensure it gets cleared
                for x in 0..max_width {
                    self.back[buffer_pos][x] = BufferCell::default();
                    self.back[buffer_pos][x].dirty = true;
                }

                buffer_pos += 1;
                continue;
            }

            let mut line_width = 0;
            let mut line_start = buffer_pos;
            let mut last_break = None;
            let mut segment_start = 0;

            let graphemes: Vec<_> = input_line.graphemes(true).collect();
            let mut i = 0;

            while i < graphemes.len() {
                let grapheme = &graphemes[i];
                let width = grapheme.width();

                // Handle line wrapping
                if line_width + width > max_width {
                    // Find break point
                    let break_pos = last_break.unwrap_or(i);
                    let length = if last_break.is_some() {
                        break_pos - segment_start
                    } else {
                        i - segment_start
                    };

                    // Record the line segment
                    if length > 0 {
                        self.line_info.push((line_start, length));
                    }

                    // Start new line
                    buffer_pos += 1; // Only advance one line
                    line_start = buffer_pos;

                    if last_break.is_some() {
                        segment_start = break_pos + 1;
                        i = break_pos + 1;
                    } else {
                        segment_start = i;
                    }

                    line_width = 0;
                    last_break = None;
                    continue;
                }

                // Store character in back buffer
                if let Some(ch) = grapheme.chars().next() {
                    let y = buffer_pos;
                    let x = line_width;

                    // Grow buffer if needed
                    while y >= self.back.len() {
                        self.back.push(vec![BufferCell::default(); max_width]);
                        self.front.push(vec![BufferCell::default(); max_width]);
                    }

                    self.back[y][x].ch = ch;
                    self.back[y][x].dirty = true;
                }

                // Update tracking
                if grapheme.chars().all(char::is_whitespace) {
                    last_break = Some(i);
                }
                line_width += width;
                i += 1;
            }

            // Record the final line segment
            if line_width > 0 {
                self.line_info.push((line_start, line_width));
            }

            buffer_pos += 1; // Move to next line
        }

        Ok(())
    }

    /// Updates color information for the entire buffer using pattern-based generation.
    /// Efficiently calculates colors for each character position using normalized coordinates.
    pub fn update_colors(
        &mut self,
        engine: &PatternEngine,
        viewport_start: usize,
    ) -> Result<(), RendererError> {
        let width = self.term_size.0 as usize;
        let height = self.term_size.1 as usize;

        // Pre-calculate constants for coordinate normalization
        let width_f = width as f64;
        let height_f = height as f64;

        // Pre-allocate pattern value buffer to reduce pattern calculation overhead
        let mut pattern_values = vec![0.0f64; width];

        // Process each line in the buffer
        for (buffer_y, line) in self.back.iter_mut().enumerate() {
            // Calculate viewport-relative position
            let viewport_y = if buffer_y >= viewport_start {
                (buffer_y - viewport_start) as f64
            } else {
                continue; // Skip lines above viewport
            };

            // Only process lines within the viewport
            if viewport_y >= height_f {
                continue;
            }

            // Calculate normalized y coordinate once per line
            let norm_y = viewport_y / height_f - 0.5;

            // Calculate pattern values for entire line at once
            for (x, value) in pattern_values.iter_mut().enumerate().take(width) {
                let norm_x = (x as f64 / width_f) - 0.5;
                *value = engine.get_value_at_normalized(norm_x, norm_y)?;
            }

            // Apply colors using pre-calculated pattern values
            for (x, &pattern_value) in pattern_values.iter().enumerate().take(width) {
                let gradient_color = engine.gradient().at(pattern_value as f32);
                let color = Color::Rgb {
                    r: (gradient_color.r * 255.0) as u8,
                    g: (gradient_color.g * 255.0) as u8,
                    b: (gradient_color.b * 255.0) as u8,
                };

                // Only mark as dirty if color actually changed
                if line[x].color != color {
                    line[x].color = color;
                    line[x].dirty = true;
                }
            }
        }

        Ok(())
    }

    /// Updates colors in static mode, creating a flowing effect by advancing the pattern per line.
    pub fn update_colors_static(&mut self, engine: &PatternEngine) -> Result<(), RendererError> {
        let width = self.term_size.0 as usize;
        let width_f = width as f64;
        let height_f = self.line_info.len() as f64;

        // Pre-allocate pattern value buffer
        let mut pattern_values = vec![0.0f64; width];

        for y in 0..self.line_info.len() {
            let (start, len) = self.line_info[y];

            // Skip empty lines
            if len == 0 {
                continue;
            }

            // Ensure buffer has enough rows
            while start >= self.back.len() {
                self.back.push(vec![BufferCell::default(); width]);
                self.front.push(vec![BufferCell::default(); width]);
            }

            // Calculate normalized y coordinate with more dramatic progression
            // Multiply by 2.0 to make the pattern advance twice as fast
            let norm_y = ((y as f64 * 2.0) / height_f) - 0.5;

            // Calculate pattern values for entire line at once
            for (x, value) in pattern_values.iter_mut().enumerate().take(len.min(width)) {
                let norm_x = (x as f64 / width_f) - 0.5;
                *value = engine.get_value_at_normalized(norm_x, norm_y)?;
            }

            // Apply colors using pre-calculated pattern values
            for (x, &pattern_value) in pattern_values.iter().enumerate().take(len.min(width)) {
                let gradient_color = engine.gradient().at(pattern_value as f32);
                let color = Color::Rgb {
                    r: (gradient_color.r * 255.0) as u8,
                    g: (gradient_color.g * 255.0) as u8,
                    b: (gradient_color.b * 255.0) as u8,
                };

                let cell = &mut self.back[start][x];
                if cell.color != color {
                    cell.color = color;
                    cell.dirty = true;
                }
            }
        }

        Ok(())
    }

    /// Renders a region of the buffer to the terminal with optimized color handling
    /// and double buffering to eliminate flicker.
    pub fn render_region(
        &mut self,
        stdout: &mut std::io::StdoutLock,
        start: usize,
        end: usize,
        colors_enabled: bool,
        is_animated: bool,
    ) -> Result<(), RendererError> {
        let width = self.term_size.0 as usize;

        if is_animated {
            // Animation mode: Use cursor movement and selective updates
            queue!(stdout, Hide)?;

            // Track if any updates were made
            let mut any_updates = false;
            let mut needs_color_reset = false;
            let mut last_color = None;

            // Process each line in the visible region
            for (display_y, line_idx) in (start..end.min(self.line_info.len())).enumerate() {
                let (line_start, line_len) = self.line_info[line_idx];

                // Skip lines that haven't changed
                if !self.back[line_start]
                    .iter()
                    .take(width)
                    .any(|cell| cell.dirty)
                {
                    continue;
                }

                any_updates = true;

                // Move cursor only when we need to update
                queue!(stdout, MoveTo(0, display_y as u16))?;

                // Build line content
                let mut line_buffer = String::with_capacity(width * 4);

                // Always process the full width for consistent display
                for x in 0..width {
                    let back_cell = &mut self.back[line_start][x];

                    // Only update color if it changed
                    if colors_enabled && last_color != Some(back_cell.color) {
                        if let Color::Rgb { r, g, b } = back_cell.color {
                            write!(line_buffer, "\x1b[38;2;{};{};{}m", r, g, b)?;
                            needs_color_reset = true;
                        }
                        last_color = Some(back_cell.color);
                    }

                    line_buffer.push(if x < line_len { back_cell.ch } else { ' ' });

                    // Clear dirty flag after processing
                    back_cell.dirty = false;
                }

                queue!(stdout, Print(&line_buffer))?;
            }

            // Only reset colors if we made updates
            if colors_enabled && needs_color_reset && any_updates {
                queue!(stdout, Print("\x1b[0m"))?;
            }

            queue!(stdout, Show)?;
        } else {
            // Static mode: Simple line-by-line output
            let mut needs_color_reset = false;

            for line_idx in start..end.min(self.line_info.len()) {
                let (line_start, line_len) = self.line_info[line_idx];

                let mut line_buffer = String::with_capacity(width * 4);
                let mut last_color = None;

                for x in 0..line_len.min(width) {
                    let back_cell = &self.back[line_start][x];

                    if colors_enabled && last_color != Some(back_cell.color) {
                        if let Color::Rgb { r, g, b } = back_cell.color {
                            write!(line_buffer, "\x1b[38;2;{};{};{}m", r, g, b)?;
                            needs_color_reset = true;
                        }
                        last_color = Some(back_cell.color);
                    }

                    line_buffer.push(back_cell.ch);
                }

                line_buffer.push('\n');
                write!(stdout, "{}", line_buffer)?;
            }

            if colors_enabled && needs_color_reset {
                write!(stdout, "\x1b[0m")?;
            }
        }

        // Swap buffers after rendering
        for y in start..end {
            if y < self.back.len() {
                for x in 0..width {
                    self.front[y][x] = self.back[y][x].clone();
                }
            }
        }

        Ok(())
    }

    /// Resizes the buffer for new terminal dimensions while maintaining content.
    pub fn resize(&mut self, new_size: (u16, u16)) -> Result<(), RendererError> {
        let new_width = new_size.0 as usize;
        let new_height = new_size.1 as usize;

        // Create new buffers with new dimensions
        let new_buffer = vec![vec![BufferCell::default(); new_width]; new_height];
        self.front = new_buffer.clone();
        self.back = new_buffer;
        self.term_size = new_size;

        // Reprocess text for new dimensions
        let text = self.original_text.clone();
        self.prepare_text(&text)?;

        Ok(())
    }

    /// Returns the maximum line length in the buffer
    #[inline]
    pub fn max_line_length(&self) -> usize {
        self.line_info
            .iter()
            .map(|(_, len)| *len)
            .max()
            .unwrap_or(0)
    }

    /// Returns the total number of lines in the buffer
    #[inline]
    pub fn total_lines(&self) -> usize {
        self.line_info.len()
    }

    // Add this method to manage buffer capacity
    fn ensure_buffer_capacity(&mut self, required_lines: usize) {
        let width = self.term_size.0 as usize;
        let current_capacity = self.back.len();

        if required_lines > current_capacity {
            // Grow by doubling, but not more than needed
            let new_capacity = (current_capacity * 2).min(required_lines + 64);
            self.back
                .resize(new_capacity, vec![BufferCell::default(); width]);
            self.front
                .resize(new_capacity, vec![BufferCell::default(); width]);
        }
    }

    // Add this as a thread_local to avoid repeated allocations
    thread_local! {
        static LINE_BUFFER: std::cell::RefCell<String> = std::cell::RefCell::new(String::with_capacity(512));
    }
}

impl Default for RenderBuffer {
    fn default() -> Self {
        Self::new((80, 24)) // Default terminal size
    }
}
