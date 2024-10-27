/// Common parameters that apply to all pattern types
#[derive(Debug, Clone, Default)]
pub struct CommonParams {
    /// Base frequency of the pattern (0.1-10.0)
    pub frequency: f64,
    /// Pattern amplitude/intensity (0.1-2.0)
    pub amplitude: f64,
    /// Animation speed multiplier (0.0-1.0)
    pub speed: f64,
}

/// Available pattern types with their specific parameters
#[derive(Debug, Clone)]
pub enum PatternParams {
    /// Simple horizontal gradient
    Horizontal,

    /// Gradient at an angle
    Diagonal {
        /// Angle in degrees (0-360)
        angle: i32,
    },

    /// Psychedelic plasma effect
    Plasma {
        /// Number of sine wave components (1.0-10.0)
        complexity: f64,
        /// Scale of the effect (0.1-5.0)
        scale: f64,
    },

    /// Ripple effect from center
    Ripple {
        /// Center X position (0.0-1.0)
        center_x: f64,
        /// Center Y position (0.0-1.0)
        center_y: f64,
        /// Distance between ripples (0.1-5.0)
        wavelength: f64,
        /// How quickly ripples fade out (0.0-1.0)
        damping: f64,
    },

    /// Wave distortion pattern
    Wave {
        /// Wave height (0.1-2.0)
        amplitude: f64,
        /// Number of waves (0.1-5.0)
        frequency: f64,
        /// Phase shift (0.0-2π)
        phase: f64,
        /// Vertical offset (0.0-1.0)
        offset: f64,
    },

    /// Spiral pattern from center
    Spiral {
        /// How tightly wound the spiral is (0.1-5.0)
        density: f64,
        /// Base rotation angle (0-360)
        rotation: f64,
        /// How quickly spiral expands (0.1-2.0)
        expansion: f64,
        /// Rotation direction
        clockwise: bool,
    },

    /// Checkerboard pattern
    Checkerboard {
        /// Size of checker squares (1-10)
        size: usize,
        /// Blur between squares (0.0-1.0)
        blur: f64,
        /// Pattern rotation angle (0-360)
        rotation: f64,
        /// Scale of the pattern (0.1-5.0)
        scale: f64,
    },

    /// Diamond-shaped pattern
    Diamond {
        /// Size of diamond shapes (0.1-5.0)
        size: f64,
        /// Pattern offset (0.0-1.0)
        offset: f64,
        /// Edge sharpness (0.1-5.0)
        sharpness: f64,
        /// Pattern rotation (0-360)
        rotation: f64,
    },

    /// Perlin noise-based pattern
    Perlin {
        /// Number of noise layers (1-8)
        octaves: u32,
        /// How quickly amplitudes diminish (0.0-1.0)
        persistence: f64,
        /// Scale of the noise (0.1-5.0)
        scale: f64,
        /// Random seed
        seed: u32,
    },
}

impl Default for PatternParams {
    fn default() -> Self {
        Self::Horizontal
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
