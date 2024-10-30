use crate::pattern::patterns::{
    CheckerboardParams, DiagonalParams, DiamondParams, HorizontalParams,
    PerlinParams, PlasmaParams, RippleParams, SpiralParams, WaveParams,
};

/// Common parameters that apply to all pattern types
#[derive(Debug, Clone)]
pub struct CommonParams {
    /// Base frequency of the pattern (0.1-10.0)
    pub frequency: f64,
    /// Pattern amplitude/intensity (0.1-2.0)
    pub amplitude: f64,
    /// Animation speed multiplier (0.0-1.0)
    pub speed: f64,
    /// Correct aspect ratio
    pub correct_aspect: bool,
    /// Aspect ratio (width/height)
    pub aspect_ratio: f64,
    /// Current theme name
    pub theme_name: Option<String>,
}

impl Default for CommonParams {
    fn default() -> Self {
        Self {
            frequency: 1.0,
            amplitude: 1.0,
            speed: 1.0,
            correct_aspect: true,
            aspect_ratio: 0.5,
            theme_name: None,
        }
    }
}

/// Available pattern types with their specific parameters
#[derive(Debug, Clone)]
pub enum PatternParams {
    /// Simple horizontal gradient
    Horizontal(HorizontalParams),
    /// Gradient at an angle
    Diagonal(DiagonalParams),
    /// Psychedelic plasma effect
    Plasma(PlasmaParams),
    /// Ripple effect from center
    Ripple(RippleParams),
    /// Wave distortion pattern
    Wave(WaveParams),
    /// Spiral pattern from center
    Spiral(SpiralParams),
    /// Checkerboard pattern
    Checkerboard(CheckerboardParams),
    /// Diamond-shaped pattern
    Diamond(DiamondParams),
    /// Perlin noise-based pattern
    Perlin(PerlinParams),
}

impl Default for PatternParams {
    fn default() -> Self {
        Self::Diagonal(DiagonalParams::default())
    }
}

/// Complete pattern configuration
#[derive(Debug, Clone, Default)]
pub struct PatternConfig {
    /// Common parameters
    pub common: CommonParams,
    /// Pattern-specific parameters
    pub params: PatternParams,
}

impl PatternConfig {
    /// Creates a new pattern configuration with default parameters
    pub fn new(pattern_type: PatternParams) -> Self {
        Self {
            common: CommonParams::default(),
            params: pattern_type,
        }
    }

    /// Returns a reference to the pattern parameters
    pub fn params(&self) -> &PatternParams {
        &self.params
    }

    /// Returns a mutable reference to the pattern parameters
    pub fn params_mut(&mut self) -> &mut PatternParams {
        &mut self.params
    }

    /// Returns a reference to the common parameters
    pub fn common(&self) -> &CommonParams {
        &self.common
    }

    /// Returns a mutable reference to the common parameters
    pub fn common_mut(&mut self) -> &mut CommonParams {
        &mut self.common
    }
}
