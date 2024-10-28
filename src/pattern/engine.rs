use colorgrad::Gradient;
use std::sync::Arc;
// use std::f64::consts::PI;

use crate::error::Result;
use crate::pattern::config::PatternConfig;
use crate::pattern::patterns::Patterns;

/// Pattern generation engine that coordinates pattern generation, animation,
/// and color mapping.
pub struct PatternEngine {
    /// Current pattern configuration
    config: PatternConfig,
    /// Thread-safe reference to the color gradient
    gradient: Arc<Box<dyn Gradient + Send + Sync>>,
    /// Current animation time in seconds
    time: f64,
    /// Width of the pattern area in pixels
    width: usize,
    /// Height of the pattern area in pixels
    height: usize,
    /// Pattern generator instance
    patterns: Patterns,
}

impl PatternEngine {
    /// Creates a new pattern engine instance
    pub fn new(
        gradient: Box<dyn Gradient + Send + Sync>,
        config: PatternConfig,
        width: usize,
        height: usize,
    ) -> Self {
        let mut patterns = Patterns::new(width, height, 0.0, 0);
        patterns.set_aspect_correction(config.common.correct_aspect);
        patterns.set_char_aspect_ratio(config.common.aspect_ratio);

        Self {
            config,
            gradient: Arc::new(gradient),
            time: 0.0,
            width,
            height,
            patterns,
        }
    }

    /// Updates the animation time based on delta seconds
    pub fn update(&mut self, delta_seconds: f64) {
        let scaled_delta = delta_seconds * self.config.common.speed;
        self.time += scaled_delta; // Remove the modulo operation
        // Create new pattern generator with updated time
        self.patterns = Patterns::new(self.width, self.height, self.time, 0);
    }

    /// Gets the current animation time
    #[inline]
    pub fn time(&self) -> f64 {
        self.time
    }

    /// Gets a reference to the color gradient
    pub fn gradient(&self) -> &(dyn Gradient + Send + Sync) {
        &**self.gradient
    }

    /// Calculates the pattern value at the specified coordinates
    pub fn get_value_at(&self, x: usize, y: usize) -> Result<f64> {
        let value = self.patterns.generate(x, y, &self.config.params);
        Ok(value)
    }

    /// Returns a reference to the current pattern configuration
    pub fn config(&self) -> &PatternConfig {
        &self.config
    }

    /// Creates a new PatternEngine instance with different dimensions
    pub fn recreate(&self, new_width: usize, new_height: usize) -> Self {
        Self {
            config: self.config.clone(),
            gradient: Arc::clone(&self.gradient),
            time: self.time,
            width: new_width,
            height: new_height,
            patterns: Patterns::new(new_width, new_height, self.time, 0), // Maintain same seed
        }
    }

    /// Sets the animation time directly
    pub fn set_time(&mut self, time: f64) {
        self.time = time; // Remove normalization
        // Update patterns with new time
        self.patterns = Patterns::new(self.width, self.height, self.time, 0);
    }
}

impl Clone for PatternEngine {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            gradient: Arc::clone(&self.gradient),
            time: self.time,
            width: self.width,
            height: self.height,
            patterns: Patterns::new(self.width, self.height, self.time, 0), // Maintain same seed
        }
    }
}
