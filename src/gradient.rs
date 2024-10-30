//! Gradient generation and configuration for ChromaCat.
//!
//! This module handles color gradient creation and manipulation for text colorization.

use crate::error::Result;
use crate::themes;
use colorgrad::{Color, Gradient};
use std::f32::consts::PI;

/// Configuration for gradient generation and application
#[derive(Debug, Clone)]
pub struct GradientConfig {
    /// Whether to use diagonal gradient mode
    pub diagonal: bool,
    /// Angle for diagonal gradient in degrees
    pub angle: i32,
    /// Enable gradient cycling
    pub cycle: bool,
}

/// Manages gradient generation and color calculations
pub struct GradientEngine {
    /// The color gradient to use
    gradient: Box<dyn Gradient + Send + Sync>,
    /// Configuration for gradient behavior
    config: GradientConfig,
    /// Total number of lines in the text
    total_lines: usize,
    /// Current line being processed
    current_line: usize,
    /// Cached trigonometric values for diagonal mode
    cached_sin: f32,
    cached_cos: f32,
}

impl GradientEngine {
    /// Creates a new GradientEngine with the specified gradient and configuration
    #[inline]
    pub fn new(gradient: Box<dyn Gradient + Send + Sync>, config: GradientConfig) -> Self {
        // Pre-calculate trig values if in diagonal mode
        let (cached_sin, cached_cos) = if config.diagonal {
            let angle_rad = (config.angle as f32) * (PI / 180.0);
            (angle_rad.sin(), angle_rad.cos())
        } else {
            (0.0, 0.0)
        };

        Self {
            gradient,
            config,
            total_lines: 0,
            current_line: 0,
            cached_sin,
            cached_cos,
        }
    }

    /// Creates a new GradientEngine from a theme name
    #[inline]
    pub fn from_theme(theme_name: &str, config: GradientConfig) -> Result<Self> {
        let theme = themes::get_theme(theme_name)?;
        let gradient = theme.create_gradient()?;
        Ok(Self::new(gradient, config))
    }

    /// Sets the total number of lines for diagonal gradient calculations
    #[inline]
    pub fn set_total_lines(&mut self, total_lines: usize) {
        self.total_lines = total_lines;
    }

    /// Sets the current line number for diagonal gradient calculations
    #[inline]
    pub fn set_current_line(&mut self, line: usize) {
        self.current_line = line;
    }

    /// Calculates the color at a specific position
    #[inline]
    pub fn get_color_at(&self, char_index: usize, line_length: usize) -> Result<Color> {
        let t = if self.config.diagonal && self.total_lines > 1 {
            self.calculate_diagonal_position(char_index, line_length)
        } else {
            self.calculate_horizontal_position(char_index, line_length)
        };

        Ok(self.gradient.at(t))
    }

    /// Calculates the gradient position for horizontal mode
    #[inline(always)]
    fn calculate_horizontal_position(&self, char_index: usize, line_length: usize) -> f32 {
        if line_length <= 1 {
            return 0.0;
        }

        let mut t = char_index as f32 / (line_length - 1) as f32;
        if self.config.cycle {
            t = (t * PI).sin() * 0.5 + 0.5;
        }
        t.clamp(0.0, 1.0)
    }

    /// Calculates the gradient position for diagonal mode
    #[inline(always)]
    fn calculate_diagonal_position(&self, char_index: usize, line_length: usize) -> f32 {
        if self.total_lines <= 1 || line_length <= 1 {
            return 0.0;
        }

        // Use pre-calculated trig values
        let x = char_index as f32 / (line_length - 1) as f32;
        let y = self.current_line as f32 / (self.total_lines - 1) as f32;

        let mut t = x * self.cached_cos + y * self.cached_sin;
        if self.config.cycle {
            t = (t * PI).sin() * 0.5 + 0.5;
        }

        t.clamp(0.0, 1.0)
    }
}

impl Default for GradientConfig {
    fn default() -> Self {
        Self {
            diagonal: false,
            angle: 45,
            cycle: false,
        }
    }
}
