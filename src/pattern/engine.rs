use colorgrad::Gradient;
use std::sync::Arc;

use crate::error::Result;
use crate::pattern::config::{PatternConfig, PatternParams};
use crate::pattern::patterns::{Patterns, RippleParams, SpiralParams};
use crate::pattern::utils::PatternUtils;

/// Pattern generation engine that coordinates pattern generation, animation,
/// and color mapping. This is the main entry point for pattern generation
/// functionality.
///
/// The engine handles:
/// - Pattern configuration and parameters
/// - Animation timing and updates
/// - Color gradient mapping
/// - Pattern value calculation
/// - Thread-safe sharing of resources
pub struct PatternEngine {
    /// Current pattern configuration including type and parameters
    config: PatternConfig,
    /// Thread-safe reference to the color gradient used for mapping pattern values
    gradient: Arc<Box<dyn Gradient + Send + Sync>>,
    /// Current animation time in seconds
    time: f64,
    /// Width of the pattern area in pixels
    width: usize,
    /// Height of the pattern area in pixels
    height: usize,
    /// Pattern generator instance for calculating pattern values
    patterns: Patterns,
}

impl PatternEngine {
    /// Creates a new pattern engine instance with the specified configuration.
    ///
    /// # Arguments
    /// * `gradient` - Color gradient for mapping pattern values to colors
    /// * `config` - Pattern configuration including type and parameters
    /// * `width` - Width of the pattern area in pixels
    /// * `height` - Height of the pattern area in pixels
    ///
    /// # Returns
    /// A new PatternEngine instance ready for generating patterns
    pub fn new(
        gradient: Box<dyn Gradient + Send + Sync>,
        config: PatternConfig,
        width: usize,
        height: usize,
    ) -> Self {
        let seed = match &config.params {
            PatternParams::Perlin { seed, .. } => *seed,
            _ => 0,
        };

        Self {
            config,
            gradient: Arc::new(gradient),
            time: 0.0,
            width,
            height,
            patterns: Patterns::new(width, height, 0.0, seed),
        }
    }

    /// Updates the animation time based on the elapsed time delta.
    /// This drives the pattern animation forward.
    ///
    /// # Arguments
    /// * `delta` - Time elapsed since last update in seconds
    pub fn update(&mut self, delta: f64) {
        let scaled_delta = delta * self.config.common.speed;
        self.time = scaled_delta;
        self.patterns = Patterns::new(self.width, self.height, self.time, 0);
    }

    /// Gets the current animation time in seconds.
    ///
    /// # Returns
    /// Current animation time value
    #[inline]
    pub fn time(&self) -> f64 {
        self.time
    }

    /// Gets a reference to the color gradient used for mapping pattern values.
    ///
    /// # Returns
    /// Reference to the color gradient
    pub fn gradient(&self) -> &(dyn Gradient + Send + Sync) {
        &**self.gradient
    }

    /// Calculates the pattern value at the specified coordinates.
    /// This is the main pattern generation function that delegates to the
    /// appropriate pattern implementation based on the current configuration.
    ///
    /// # Arguments
    /// * `x` - X coordinate in the pattern area
    /// * `y` - Y coordinate in the pattern area
    ///
    /// # Returns
    /// Result containing the pattern value between 0.0 and 1.0
    pub fn get_value_at(&self, x: usize, y: usize) -> Result<f64> {
        let base_value = match &self.config.params {
            PatternParams::Horizontal => self.patterns.horizontal(x, y),

            PatternParams::Diagonal { angle } => {
                self.patterns
                    .diagonal(x, y, *angle, self.config.common.frequency)
            }

            PatternParams::Plasma { complexity, scale } => {
                self.patterns
                    .plasma(x, y, *complexity, *scale, self.config.common.frequency)
            }

            PatternParams::Ripple {
                center_x,
                center_y,
                wavelength,
                damping,
            } => self.patterns.ripple(
                x,
                y,
                RippleParams {
                    center_x: *center_x,
                    center_y: *center_y,
                    wavelength: *wavelength,
                    damping: *damping,
                    frequency: self.config.common.frequency,
                },
            ),

            PatternParams::Wave {
                amplitude,
                frequency,
                phase,
                offset,
            } => self.patterns.wave(
                x,
                *amplitude * self.config.common.amplitude,
                *frequency,
                *phase,
                *offset,
                self.config.common.frequency,
            ),

            PatternParams::Spiral {
                density,
                rotation,
                expansion,
                clockwise,
            } => self.patterns.spiral(
                x,
                y,
                SpiralParams {
                    density: *density,
                    rotation: *rotation,
                    expansion: *expansion,
                    clockwise: *clockwise,
                    frequency: self.config.common.frequency,
                },
            ),

            PatternParams::Checkerboard {
                size,
                blur,
                rotation,
                scale,
            } => self
                .patterns
                .checkerboard(x, y, *size, *blur, *rotation, *scale),

            PatternParams::Diamond {
                size,
                offset,
                sharpness,
                rotation,
            } => self
                .patterns
                .diamond(x, y, *size, *offset, *sharpness, *rotation),

            PatternParams::Perlin {
                octaves,
                persistence,
                scale,
                seed: _,
            } => self.patterns.perlin(x, y, *octaves, *persistence, *scale),
        };

        // Apply time-based animation with improved smoothness
        let final_value = match &self.config.params {
            // Static patterns - no time animation
            PatternParams::Horizontal => base_value,

            // Patterns that use time internally (already smooth)
            PatternParams::Plasma { .. }
            | PatternParams::Ripple { .. }
            | PatternParams::Wave { .. }
            | PatternParams::Spiral { .. }
            | PatternParams::Diagonal { .. }
            | PatternParams::Checkerboard { .. } => base_value,

            // Patterns that should offset by time with improved smoothness
            _ if self.config.common.speed > 0.0 => {
                let time_floor = self.time.floor();
                let time_fract = self.time.fract();

                let value1 = (base_value + time_floor) % 1.0;
                let value2 = (base_value + time_floor + 1.0) % 1.0;

                PatternUtils::interpolate_value(value1, value2, time_fract)
            }

            // Default case - no animation
            _ => base_value,
        };

        Ok(final_value.clamp(0.0, 1.0))
    }

    /// Returns a reference to the current pattern configuration.
    ///
    /// # Returns
    /// Reference to the PatternConfig instance
    pub fn config(&self) -> &PatternConfig {
        &self.config
    }

    /// Creates a new PatternEngine instance with different dimensions
    /// but preserving all other settings.
    ///
    /// # Arguments
    /// * `new_width` - New width for the pattern area
    /// * `new_height` - New height for the pattern area
    ///
    /// # Returns
    /// A new PatternEngine instance with updated dimensions
    pub fn recreate(&self, new_width: usize, new_height: usize) -> Self {
        Self {
            config: self.config.clone(),
            gradient: Arc::clone(&self.gradient),
            time: self.time,
            width: new_width,
            height: new_height,
            patterns: Patterns::new(new_width, new_height, self.time, 0),
        }
    }
}

/// Implements Clone for PatternEngine to allow creating independent copies
/// of the engine while sharing immutable resources like the color gradient.
impl Clone for PatternEngine {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            gradient: Arc::clone(&self.gradient),
            time: self.time,
            width: self.width,
            height: self.height,
            patterns: Patterns::new(self.width, self.height, self.time, 0),
        }
    }
}
