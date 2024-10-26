use crate::error::{ChromaCatError, Result};
use colorgrad::{Color, Gradient, GradientBuilder, LinearGradient};
use std::f64::consts::PI;
use std::str::FromStr;

/// Available color gradient themes
#[derive(Debug, Clone)]
pub enum Theme {
    Rainbow,
    Heat,
    Ocean,
    Forest,
    Pastel,
    Neon,
    Autumn,
}

impl Theme {
    /// Returns the color stops for the theme
    pub fn get_colors(&self) -> Vec<Color> {
        match self {
            Theme::Rainbow => vec![
                Color::new(1.0, 0.0, 0.0, 1.0),    // Red
                Color::new(1.0, 0.5, 0.0, 1.0),    // Orange
                Color::new(1.0, 1.0, 0.0, 1.0),    // Yellow
                Color::new(0.0, 1.0, 0.0, 1.0),    // Green
                Color::new(0.0, 0.0, 1.0, 1.0),    // Blue
                Color::new(0.29, 0.0, 0.51, 1.0),  // Indigo
                Color::new(0.58, 0.0, 0.83, 1.0),  // Violet
            ],
            Theme::Heat => vec![
                Color::new(1.0, 0.0, 0.0, 1.0),    // Red
                Color::new(1.0, 0.5, 0.0, 1.0),    // Orange
                Color::new(1.0, 1.0, 0.0, 1.0),    // Yellow
            ],
            Theme::Ocean => vec![
                Color::new(0.0, 0.47, 0.75, 1.0),  // Deep blue
                Color::new(0.0, 0.71, 0.85, 1.0),  // Medium blue
                Color::new(0.28, 0.79, 0.89, 1.0), // Light blue
                Color::new(0.56, 0.88, 0.94, 1.0), // Sky blue
            ],
            Theme::Forest => vec![
                Color::new(0.08, 0.32, 0.16, 1.0), // Dark green
                Color::new(0.18, 0.54, 0.34, 1.0), // Forest green
                Color::new(0.13, 0.54, 0.13, 1.0), // Green
                Color::new(0.60, 0.80, 0.20, 1.0), // Yellow green
            ],
            Theme::Pastel => vec![
                Color::new(1.0, 0.71, 0.76, 1.0),  // Light pink
                Color::new(1.0, 0.85, 0.73, 1.0),  // Peach
                Color::new(1.0, 1.0, 0.88, 1.0),   // Light yellow
                Color::new(0.69, 0.88, 0.90, 1.0), // Powder blue
            ],
            Theme::Neon => vec![
                Color::new(1.0, 0.0, 1.0, 1.0),    // Magenta
                Color::new(0.0, 1.0, 1.0, 1.0),    // Cyan
                Color::new(1.0, 1.0, 0.0, 1.0),    // Yellow
                Color::new(0.0, 1.0, 0.0, 1.0),    // Green
            ],
            Theme::Autumn => vec![
                Color::new(0.65, 0.16, 0.16, 1.0), // Brown
                Color::new(0.82, 0.41, 0.12, 1.0), // Chocolate
                Color::new(1.0, 0.27, 0.0, 1.0),   // Red-orange
                Color::new(1.0, 0.55, 0.0, 1.0),   // Dark orange
            ],
        }
    }

    /// Creates a gradient from the theme
    pub fn create_gradient(&self) -> Result<Box<dyn Gradient + Send + Sync>> {
        let colors = self.get_colors();
        let gradient = GradientBuilder::new()
            .colors(&colors)
            .build::<LinearGradient>()
            .map_err(|e| ChromaCatError::GradientError(e.to_string()))?;
        Ok(Box::new(gradient))
    }
}

impl FromStr for Theme {
    type Err = ChromaCatError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "rainbow" => Ok(Theme::Rainbow),
            "heat" => Ok(Theme::Heat),
            "ocean" => Ok(Theme::Ocean),
            "forest" => Ok(Theme::Forest),
            "pastel" => Ok(Theme::Pastel),
            "neon" => Ok(Theme::Neon),
            "autumn" => Ok(Theme::Autumn),
            _ => Err(ChromaCatError::InvalidTheme(s.to_string())),
        }
    }
}

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