//! Pattern generation and configuration for ChromaCat
//!
//! This module provides the core pattern generation functionality, handling:
//! - Pattern type definitions
//! - Parameter configuration
//! - Pattern calculation algorithms
//! - Animation timing and updates
//! - Color gradient mapping

use crate::error::Result;
use colorgrad::{Color, Gradient};
use std::f64::consts::PI;
use std::sync::Arc;

/// Common parameters that apply to all pattern types
#[derive(Debug, Clone)]
pub struct CommonParams {
    /// Base frequency of the pattern (0.1-10.0)
    pub frequency: f64,
    /// Pattern amplitude/intensity (0.1-2.0)
    pub amplitude: f64,
    /// Animation speed multiplier (0.0-1.0)
    pub speed: f64,
}

impl Default for CommonParams {
    fn default() -> Self {
        Self {
            frequency: 1.0,
            amplitude: 1.0,
            speed: 1.0,
        }
    }
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

/// Complete pattern configuration
#[derive(Debug, Clone)]
pub struct PatternConfig {
    /// Common parameters
    pub common: CommonParams,
    /// Pattern-specific parameters
    pub params: PatternParams,
}

/// Pattern generation engine
pub struct PatternEngine {
    /// Pattern configuration
    config: PatternConfig,
    /// Color gradient for pattern
    gradient: Arc<Box<dyn Gradient + Send + Sync>>,
    /// Current animation time (0.0-1.0)
    time: f64,
    /// Width of pattern area
    width: usize,
    /// Height of pattern area
    height: usize,
    /// Lookup table for sine values
    sin_table: Arc<Vec<f64>>,
    /// Lookup table for cosine values
    cos_table: Arc<Vec<f64>>,
    /// Perlin noise permutation table
    perm_table: Arc<Vec<u8>>,
}

impl PatternEngine {
    /// Creates a new pattern engine instance
    ///
    /// # Arguments
    /// * `gradient` - Color gradient for pattern
    /// * `config` - Pattern configuration
    /// * `width` - Width of pattern area
    /// * `height` - Height of pattern area
    pub fn new(
        gradient: Box<dyn Gradient + Send + Sync>,
        config: PatternConfig,
        width: usize,
        height: usize,
    ) -> Self {
        Self {
            config,
            gradient: Arc::new(gradient),
            time: 0.0,
            width,
            height,
            sin_table: Arc::new(Self::init_sin_table()),
            cos_table: Arc::new(Self::init_cos_table()),
            perm_table: Arc::new(Self::init_perm_table(0)),
        }
    }

    /// Updates the animation time
    ///
    /// # Arguments
    /// * `delta` - Time increment (0.0-1.0)
    pub fn update(&mut self, delta: f64) {
        let adjusted_delta = delta * self.config.common.speed;
        self.time = if adjusted_delta.is_finite() {
            (self.time + adjusted_delta) % 1.0
        } else {
            0.0
        };
    }

    /// Gets the current animation time
    pub fn time(&self) -> f64 {
        self.time
    }

    /// Gets the gradient for color mapping
    pub fn gradient(&self) -> &Box<dyn Gradient + Send + Sync> {
        &self.gradient
    }

    /// Gets the pattern value at given coordinates
    pub fn get_value_at(&self, x: usize, y: usize) -> Result<f64> {
        let base_value = match &self.config.params {
            PatternParams::Horizontal => self.horizontal_pattern(x, y),
            PatternParams::Diagonal { angle } => self.diagonal_pattern(x, y, *angle),
            PatternParams::Plasma { complexity, scale } => {
                self.plasma_pattern(x, y, *complexity, *scale)
            }
            PatternParams::Ripple {
                center_x,
                center_y,
                wavelength,
                damping,
            } => self.ripple_pattern(x, y, *center_x, *center_y, *wavelength, *damping),
            PatternParams::Wave {
                amplitude,
                frequency,
                phase,
                offset,
            } => self.wave_pattern(x, y, *amplitude, *frequency, *phase, *offset),
            PatternParams::Spiral {
                density,
                rotation,
                expansion,
                clockwise,
            } => self.spiral_pattern(x, y, *density, *rotation, *expansion, *clockwise),
            PatternParams::Checkerboard {
                size,
                blur,
                rotation,
                scale,
            } => self.checkerboard_pattern(x, y, *size, *blur, *rotation, *scale),
            PatternParams::Diamond {
                size,
                offset,
                sharpness,
                rotation,
            } => self.diamond_pattern(x, y, *size, *offset, *sharpness, *rotation),
            PatternParams::Perlin {
                octaves,
                persistence,
                scale,
                seed: _,
            } => self.perlin_pattern(x, y, *octaves, *persistence, *scale),
        };

        // Apply time-based animation based on pattern type
        let final_value = match &self.config.params {
            // Static patterns - no time animation
            PatternParams::Horizontal | PatternParams::Diagonal { .. } => base_value,

            // Patterns that use time internally
            PatternParams::Plasma { .. }
            | PatternParams::Ripple { .. }
            | PatternParams::Wave { .. }
            | PatternParams::Spiral { .. } => base_value,

            // Patterns that should offset by time
            _ if self.config.common.speed > 0.0 => (base_value + self.time) % 1.0,

            // Default case - no animation
            _ => base_value,
        };

        Ok(final_value.clamp(0.0, 1.0))
    }

    /// Initialize sine lookup table
    fn init_sin_table() -> Vec<f64> {
        (0..360).map(|i| (i as f64 * PI / 180.0).sin()).collect()
    }

    /// Initialize cosine lookup table
    fn init_cos_table() -> Vec<f64> {
        (0..360).map(|i| (i as f64 * PI / 180.0).cos()).collect()
    }

    /// Fast sine approximation using lookup table
    fn fast_sin(&self, angle: f64) -> f64 {
        let normalized_angle = angle.rem_euclid(2.0 * PI);
        let index = ((normalized_angle * 180.0 / PI) as usize) % 360;
        self.sin_table[index]
    }

    /// Fast cosine approximation using lookup table
    fn fast_cos(&self, angle: f64) -> f64 {
        let normalized_angle = angle.rem_euclid(2.0 * PI);
        let index = ((normalized_angle * 180.0 / PI) as usize) % 360;
        self.cos_table[index]
    }

    // Pattern implementations

    fn horizontal_pattern(&self, x: usize, _y: usize) -> f64 {
        if self.width <= 1 {
            return 0.0;
        }
        x as f64 / (self.width - 1) as f64
    }

    fn diagonal_pattern(&self, x: usize, y: usize, angle: i32) -> f64 {
        if self.width <= 1 || self.height <= 1 {
            return 0.0;
        }

        // Convert coordinates to -1..1 range
        let x_norm = (2.0 * x as f64 / (self.width - 1) as f64) - 1.0;
        let y_norm = (2.0 * y as f64 / (self.height - 1) as f64) - 1.0;

        // Convert angle to radians
        let angle_rad = (angle % 360) as f64 * PI / 180.0;

        // Rotate the point
        let cos_angle = self.fast_cos(angle_rad);
        let sin_angle = self.fast_sin(angle_rad);
        
        // Project onto the angle vector (using dot product)
        let rotated = x_norm * cos_angle + y_norm * sin_angle;

        // Map from -1..1 to 0..1
        (rotated + 1.0) * 0.5
    }

    fn plasma_pattern(&self, x: usize, y: usize, complexity: f64, scale: f64) -> f64 {
        let freq = self.config.common.frequency * scale;
        let time = self.time * PI * 2.0;
        let x_norm = x as f64 / self.width as f64;
        let y_norm = y as f64 / self.height as f64;

        let mut sum = 0.0;
        for i in 0..complexity as u32 {
            let factor = 2.0_f64.powi(i as i32);
            sum += self.fast_sin((x_norm * freq * factor + time) * PI * 2.0) / factor;
            sum += self.fast_sin((y_norm * freq * factor + time) * PI * 2.0) / factor;
            sum += self.fast_sin(((x_norm + y_norm) * freq * factor + time) * PI * 2.0) / factor;
        }

        (sum / complexity + 1.0) / 2.0
    }

    fn ripple_pattern(
        &self,
        x: usize,
        y: usize,
        center_x: f64,
        center_y: f64,
        wavelength: f64,
        damping: f64,
    ) -> f64 {
        let dx = x as f64 / self.width as f64 - center_x;
        let dy = y as f64 / self.height as f64 - center_y;
        let distance = (dx * dx + dy * dy).sqrt();

        let freq = self.config.common.frequency;
        let time = self.time * PI * 2.0;

        let value = self.fast_sin(distance / wavelength * PI * 10.0 + time);
        let amplitude = (-distance * damping * 5.0).exp();

        (value * amplitude + 1.0) / 2.0
    }

    fn wave_pattern(
        &self,
        x: usize,
        _y: usize,
        amplitude: f64,
        frequency: f64,
        phase: f64,
        offset: f64,
    ) -> f64 {
        let x_norm = x as f64 / (self.width.max(1) - 1) as f64;
        let wave_angle = x_norm * frequency * self.config.common.frequency * 2.0 * PI
            + phase
            + self.time * 2.0 * PI;

        let scaled_amplitude = amplitude * 0.5;
        let wave = self.fast_sin(wave_angle) * scaled_amplitude;

        (offset + wave).clamp(0.0, 1.0)
    }

    fn spiral_pattern(
        &self,
        x: usize,
        y: usize,
        density: f64,
        rotation: f64,
        expansion: f64,
        clockwise: bool,
    ) -> f64 {
        let center_x = self.width as f64 / 2.0;
        let center_y = self.height as f64 / 2.0;
        let dx = x as f64 - center_x;
        let dy = y as f64 - center_y;

        let mut angle = dy.atan2(dx);
        if !clockwise {
            angle = -angle;
        }

        let distance = (dx * dx + dy * dy).sqrt() / (self.width.min(self.height) as f64 / 2.0);
        let rot_rad = rotation * PI / 180.0;

        let freq = self.config.common.frequency;
        let time = self.time * PI * 2.0;

        ((angle + distance * density * expansion + rot_rad + time * freq) % (PI * 2.0)) / (PI * 2.0)
    }

    fn checkerboard_pattern(
        &self,
        x: usize,
        y: usize,
        size: usize,
        blur: f64,
        rotation: f64,
        scale: f64,
    ) -> f64 {
        let rot_rad = rotation * PI / 180.0;
        let scaled_x = x as f64 * scale;
        let scaled_y = y as f64 * scale;

        // Apply rotation
        let rx = scaled_x * self.fast_cos(rot_rad) - scaled_y * self.fast_sin(rot_rad);
        let ry = scaled_x * self.fast_sin(rot_rad) + scaled_y * self.fast_cos(rot_rad);

        let cell_x = (rx / size as f64).floor() as i32;
        let cell_y = (ry / size as f64).floor() as i32;

        if blur > 0.0 {
            // Add smooth transitions between cells
            let fx = (rx / size as f64).fract();
            let fy = (ry / size as f64).fract();

            let edge_x = (0.5 - (fx - 0.5).abs()) / blur;
            let edge_y = (0.5 - (fy - 0.5).abs()) / blur;

            let base = ((cell_x + cell_y) % 2) as f64;
            base + (edge_x + edge_y).min(1.0) * (1.0 - base)
        } else {
            ((cell_x + cell_y) % 2) as f64
        }
    }

    fn diamond_pattern(
        &self,
        x: usize,
        y: usize,
        size: f64,
        offset: f64,
        sharpness: f64,
        rotation: f64,
    ) -> f64 {
        let x_norm = x as f64 / self.width as f64 - 0.5;
        let y_norm = y as f64 / self.height as f64 - 0.5;

        // Apply rotation
        let rot_rad = rotation * PI / 180.0;
        let rx = x_norm * self.fast_cos(rot_rad) - y_norm * self.fast_sin(rot_rad);
        let ry = x_norm * self.fast_sin(rot_rad) + y_norm * self.fast_cos(rot_rad);

        let freq = self.config.common.frequency * size;
        let time = self.time * PI * 2.0;

        // Calculate diamond pattern
        let dx = (rx * freq).abs();
        let dy = (ry * freq).abs();
        let diamond_val = (dx + dy + offset) % 1.0;

        // Apply sharpness and time factor
        (diamond_val.powf(sharpness) + self.fast_sin(time)) / 2.0
    }

    fn perlin_pattern(
        &self,
        x: usize,
        y: usize,
        octaves: u32,
        persistence: f64,
        scale: f64,
    ) -> f64 {
        let x_norm = x as f64 / self.width as f64;
        let y_norm = y as f64 / self.height as f64;
        let freq = self.config.common.frequency * scale;
        let time = self.time;

        let mut total = 0.0;
        let mut frequency = freq;
        let mut amplitude = 1.0;
        let mut max_value = 0.0;

        // Sum octaves
        for _ in 0..octaves {
            total +=
                self.perlin_noise(x_norm * frequency + time, y_norm * frequency + time) * amplitude;

            max_value += amplitude;
            amplitude *= persistence;
            frequency *= 2.0;
        }

        // Normalize to 0.0-1.0 range
        (total / max_value + 1.0) / 2.0
    }

    /// Initialize Perlin noise permutation table
    fn init_perm_table(seed: u32) -> Vec<u8> {
        let mut perm = vec![0; 256];
        for i in 0..256 {
            perm[i] = i as u8;
        }

        // Basic shuffle using the seed
        let mut rng = seed;
        for i in (1..256).rev() {
            rng = rng.wrapping_mul(48271).wrapping_add(1);
            let j = (rng % (i + 1) as u32) as usize;
            perm.swap(i, j);
        }

        perm
    }

    /// Calculate 2D Perlin noise value
    fn perlin_noise(&self, x: f64, y: f64) -> f64 {
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;
        let x1 = x0 + 1;
        let y1 = y0 + 1;

        let dx = x - x0 as f64;
        let dy = y - y0 as f64;

        // Smooth interpolation
        let sx = Self::smoothstep(dx);
        let sy = Self::smoothstep(dy);

        // Calculate dot products with gradient vectors
        let n00 = self.perlin_gradient(x0, y0, dx, dy);
        let n10 = self.perlin_gradient(x1, y0, dx - 1.0, dy);
        let n01 = self.perlin_gradient(x0, y1, dx, dy - 1.0);
        let n11 = self.perlin_gradient(x1, y1, dx - 1.0, dy - 1.0);

        // Interpolate between values
        let nx0 = Self::lerp(n00, n10, sx);
        let nx1 = Self::lerp(n01, n11, sx);
        Self::lerp(nx0, nx1, sy)
    }

    /// Smoothstep interpolation
    fn smoothstep(t: f64) -> f64 {
        t * t * (3.0 - 2.0 * t)
    }

    /// Linear interpolation
    fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a + t * (b - a)
    }

    /// Calculate local gradient for Perlin noise
    fn perlin_gradient(&self, x: i32, y: i32, dx: f64, dy: f64) -> f64 {
        // Hash coordinates to get table index
        let hash = self.perm_table
            [(self.perm_table[(x & 255) as usize] as usize + y as usize) & 255]
            as usize;

        // Use hash to select gradient
        match hash & 3 {
            0 => dx + dy,
            1 => -dx + dy,
            2 => dx - dy,
            _ => -dx - dy,
        }
    }
}

impl Clone for PatternEngine {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            gradient: self.gradient.clone(),
            time: self.time,
            width: self.width,
            height: self.height,
            sin_table: self.sin_table.clone(),
            cos_table: self.cos_table.clone(),
            perm_table: self.perm_table.clone(),
        }
    }
}

impl Default for PatternParams {
    fn default() -> Self {
        Self::Horizontal
    }
}

impl Default for PatternConfig {
    fn default() -> Self {
        Self {
            common: CommonParams::default(),
            params: PatternParams::default(),
        }
    }
}
