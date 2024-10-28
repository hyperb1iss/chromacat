mod checkerboard;
mod diagonal;
mod diamond;
mod horizontal;
mod perlin;
mod plasma;
mod ripple;
mod spiral;
mod wave;

pub use checkerboard::CheckerboardParams;
pub use diagonal::DiagonalParams;
pub use diamond::DiamondParams;
pub use horizontal::HorizontalParams;
pub use perlin::PerlinParams;
pub use plasma::{PlasmaParams, PlasmaBlendMode};
pub use ripple::RippleParams;
pub use spiral::SpiralParams;
pub use wave::WaveParams;

use crate::pattern::utils::PatternUtils;
use crate::pattern::config::PatternParams;

/// Core pattern generation struct that handles various visual effects
pub struct Patterns {
    /// Utility functions for pattern calculations
    utils: PatternUtils,
    /// Width of the pattern area in pixels
    width: usize,
    /// Height of the pattern area in pixels
    height: usize,
    /// Current animation time in seconds
    time: f64,
    /// Terminal character aspect ratio (width/height)
    char_aspect_ratio: f64,
    /// Whether to apply aspect ratio correction
    correct_aspect: bool,
}

impl Patterns {
    /// Creates a new Patterns instance
    pub fn new(width: usize, height: usize, time: f64, seed: u32) -> Self {
        Self {
            utils: PatternUtils::new(seed),
            width,
            height,
            time,
            char_aspect_ratio: 0.5, // Default terminal character aspect ratio
            correct_aspect: true,  // Enable by default
        }
    }

    /// Helper method to normalize coordinates with optional aspect ratio correction
    pub fn normalize_coords(&self, x: usize, y: usize) -> (f64, f64) {
        let x_norm = x as f64 / self.width as f64;
        let y_norm = y as f64 / self.height as f64;

        if self.correct_aspect {
            // Apply aspect ratio correction
            let x_centered = (x_norm - 0.5) * self.char_aspect_ratio;
            let y_centered = y_norm - 0.5;
            (x_centered, y_centered)
        } else {
            // No correction
            (x_norm - 0.5, y_norm - 0.5)
        }
    }

    /// Set whether to apply aspect ratio correction
    pub fn set_aspect_correction(&mut self, enabled: bool) {
        self.correct_aspect = enabled;
    }

    /// Set the character aspect ratio
    pub fn set_char_aspect_ratio(&mut self, ratio: f64) {
        self.char_aspect_ratio = ratio.clamp(0.1, 2.0);
    }

    /// Generate a pattern value at the given coordinates
    pub fn generate(&self, x: usize, y: usize, params: &PatternParams) -> f64 {
        let (x_norm, y_norm) = self.normalize_coords(x, y);
        
        match params {
            PatternParams::Horizontal(p) => self.horizontal(x_norm + 0.5, p.clone()), // Convert to 0-1 range
            PatternParams::Diagonal(p) => self.diagonal(x_norm, y_norm, p.clone()),
            PatternParams::Plasma(p) => self.plasma(x_norm, y_norm, p.clone()),
            PatternParams::Ripple(p) => self.ripple(x_norm, y_norm, p.clone()),
            PatternParams::Wave(p) => self.wave(x_norm, y_norm, p.clone()),
            PatternParams::Spiral(p) => self.spiral(x_norm, y_norm, p.clone()),
            PatternParams::Checkerboard(p) => self.checkerboard(x_norm, y_norm, p.clone()),
            PatternParams::Diamond(p) => self.diamond(x_norm, y_norm, p.clone()),
            PatternParams::Perlin(p) => self.perlin(x_norm, y_norm, p.clone()),
        }
    }
}
