use crate::error::Result;
use crate::themes::Theme;
use colorgrad::{Color, Gradient};
use std::f64::consts::PI;

/// Configuration for gradient generation and application
#[derive(Debug, Clone)]
pub struct GradientConfig {
    pub diagonal: bool,
    pub angle: i32,
    pub cycle: bool,
}

/// Manages gradient generation and color calculations
pub struct GradientEngine {
    gradient: Box<dyn Gradient + Send + Sync>,
    config: GradientConfig,
    total_lines: usize,
    current_line: usize,
}

impl GradientEngine {
    /// Creates a new GradientEngine with the specified gradient and configuration
    pub fn new(gradient: Box<dyn Gradient + Send + Sync>, config: GradientConfig) -> Self {
        Self {
            gradient,
            config,
            total_lines: 0,
            current_line: 0,
        }
    }

    /// Creates a new GradientEngine from a theme
    pub fn from_theme(theme: &Theme, config: GradientConfig) -> Result<Self> {
        let gradient = theme.create_gradient()?;
        Ok(Self::new(gradient, config))
    }

    /// Sets the total number of lines for diagonal gradient calculations
    pub fn set_total_lines(&mut self, total_lines: usize) {
        self.total_lines = total_lines;
    }

    /// Sets the current line number for diagonal gradient calculations
    pub fn set_current_line(&mut self, line: usize) {
        self.current_line = line;
    }

    /// Calculates the color at a specific position
    pub fn get_color_at(&self, char_index: usize, line_length: usize) -> Result<Color> {
        let t = if self.config.diagonal && self.total_lines > 1 {
            self.calculate_diagonal_position(char_index, line_length)
        } else {
            self.calculate_horizontal_position(char_index, line_length)
        };

        Ok(self.gradient.at(t as f32))
    }

    fn calculate_horizontal_position(&self, char_index: usize, line_length: usize) -> f64 {
        if line_length <= 1 {
            return 0.0;
        }

        let mut t = char_index as f64 / (line_length - 1) as f64;
        if self.config.cycle {
            t = (t * PI * 2.0).sin() * 0.5 + 0.5;
        }
        t.clamp(0.0, 1.0)
    }

    fn calculate_diagonal_position(&self, char_index: usize, line_length: usize) -> f64 {
        if self.total_lines <= 1 || line_length <= 1 {
            return 0.0;
        }

        let angle_rad = (self.config.angle as f64) * PI / 180.0;
        let x = char_index as f64 / (line_length - 1) as f64;
        let y = self.current_line as f64 / (self.total_lines - 1) as f64;

        let mut t = x * angle_rad.cos() + y * angle_rad.sin();
        if self.config.cycle {
            t = (t * PI * 2.0).sin() * 0.5 + 0.5;
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
