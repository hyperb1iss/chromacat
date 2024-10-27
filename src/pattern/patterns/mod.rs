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
}

impl Patterns {
    /// Creates a new Patterns instance
    pub fn new(width: usize, height: usize, time: f64, seed: u32) -> Self {
        Self {
            utils: PatternUtils::new(seed),
            width,
            height,
            time,
        }
    }

    /// Generate a pattern value at the given coordinates
    pub fn generate(&self, x: usize, y: usize, params: &PatternParams) -> f64 {
        match params {
            PatternParams::Horizontal(p) => self.horizontal(x, p.clone()),
            PatternParams::Diagonal(p) => self.diagonal(x, y, p.clone()),
            PatternParams::Plasma(p) => self.plasma(x, y, p.clone()),
            PatternParams::Ripple(p) => self.ripple(x, y, p.clone()),
            PatternParams::Wave(p) => self.wave(x, p.clone()),
            PatternParams::Spiral(p) => self.spiral(x, y, p.clone()),
            PatternParams::Checkerboard(p) => self.checkerboard(x, y, p.clone()),
            PatternParams::Diamond(p) => self.diamond(x, y, p.clone()),
            PatternParams::Perlin(p) => self.perlin(x, y, p.clone()),
        }
    }
}
