//! Pattern generation and configuration for ChromaCat
//!
//! This module provides the core pattern generation functionality for creating
//! visual effects in text output. It includes:
//!
//! - Pattern type definitions and parameters
//! - Pattern calculation algorithms
//! - Animation timing and updates
//! - Color gradient mapping
//! - Performance optimizations through lookup tables
//!
//! The pattern system supports multiple effect types including waves, spirals,
//! plasma effects, and more, each with configurable parameters for customization.

use crate::error::Result;
use colorgrad::Gradient;
use std::f64::consts::PI;
use std::sync::Arc;

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
        // Apply speed multiplier from common parameters
        let scaled_delta = delta * self.config.common.speed;
        self.time = scaled_delta;
    }

    /// Gets the current animation time
    #[inline]
    pub fn time(&self) -> f64 {
        self.time
    }

    /// Gets the gradient for color mapping
    pub fn gradient(&self) -> &(dyn Gradient + Send + Sync) {
        &**self.gradient
    }

    /// Gets the pattern value at given coordinates
    ///
    /// # Arguments
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    ///
    /// # Returns
    /// * `Result<f64>` - Pattern value between 0.0 and 1.0
    pub fn get_value_at(&self, x: usize, y: usize) -> Result<f64> {
        // Calculate the base pattern value
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
            } => {
                // For static rendering, use a fixed time value instead of 0
                let effective_time = if self.config.common.speed == 0.0 {
                    0.25  // Use a fixed time value that gives good contrast
                } else {
                    self.time
                };

                // Create a temporary copy of self with the effective time
                let mut temp_self = self.clone();
                temp_self.time = effective_time;

                temp_self.checkerboard_pattern(x, y, *size, *blur, *rotation, *scale)
            }
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
            | PatternParams::Checkerboard { .. }
            => base_value,

            // Patterns that should offset by time with improved smoothness
            _ if self.config.common.speed > 0.0 => {
                // Calculate two time positions and interpolate between them
                let time_floor = self.time.floor();
                let time_fract = self.time.fract();

                let value1 = (base_value + time_floor) % 1.0;
                let value2 = (base_value + time_floor + 1.0) % 1.0;

                self.interpolate_value(value1, value2, time_fract)
            }

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

    // Pattern implementations follow...

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

        // Use simpler coordinate normalization without aspect ratio
        let x_norm = x as f64 / (self.width - 1) as f64;
        let y_norm = (y % self.height) as f64 / (self.height - 1) as f64;

        // Scale to -1..1 range
        let x_scaled = x_norm * 2.0 - 1.0;
        let y_scaled = y_norm * 2.0 - 1.0;

        // Add time-based animation
        let time = self.time * PI * 2.0;

        // Create a moving wave effect
        let wave_offset = self.fast_sin(time * 0.5 + y_norm * 8.0) * 0.2;

        // Animate the angle
        let base_angle = angle as f64;
        let animated_angle = (base_angle + self.fast_sin(time * 0.3) * 15.0) * PI / 180.0;

        // Rotate the point
        let cos_angle = self.fast_cos(animated_angle);
        let sin_angle = self.fast_sin(animated_angle);

        // Project onto the angle vector with wave distortion
        let rotated = x_scaled * cos_angle + y_scaled * sin_angle + wave_offset;

        // Add wave distortion perpendicular to the gradient
        let perpendicular = -x_scaled * sin_angle + y_scaled * cos_angle;
        let wave_distortion = self.fast_sin(perpendicular * 4.0 + time) * 0.1;

        // Combine effects and map to 0..1 range
        let result = (rotated + wave_distortion + 1.0) * 0.5;

        // Add pulsing effect
        let pulse = (self.fast_sin(time * 0.7) * 0.1 + 1.0) * result;

        pulse.clamp(0.0, 1.0)
    }

    fn plasma_pattern(&self, x: usize, y: usize, complexity: f64, scale: f64) -> f64 {
        // Increase overall animation speed
        let time = self.time * PI; // Removed the 0.2 multiplier
        let x_norm = x as f64 / self.width as f64;
        let y_norm = y as f64 / self.height as f64;

        // More pronounced center motion
        let cx = 0.5 + 0.4 * self.fast_sin(time * 0.4); // Doubled from 0.2 to 0.4
        let cy = 0.5 + 0.4 * self.fast_cos(time * 0.43); // Doubled from 0.23 to 0.43

        let base_freq = self.config.common.frequency * scale * 2.0; // Increased from 1.5 to 2.0
        let mut sum = 0.0;
        let mut divisor = 0.0;

        // Layer 1: More dynamic circular waves
        let dx1 = x_norm - cx;
        let dy1 = y_norm - cy;
        let dist1 = (dx1 * dx1 + dy1 * dy1).sqrt();
        sum += self.fast_sin(dist1 * 8.0 * base_freq + time * 0.6) * 1.2; // Doubled from 0.3 to 0.6
        divisor += 1.2;

        // Layer 2: Faster perpendicular waves
        sum += self.fast_sin(x_norm * 5.0 * base_freq + time * 0.4) * 0.8; // Doubled from 0.2 to 0.4
        sum += self.fast_sin(y_norm * 5.0 * base_freq + time * 0.47) * 0.8; // Increased from 0.27 to 0.47
        divisor += 1.6;

        // Layer 3: Faster rotating waves
        let angle = time * 0.2; // Doubled from 0.1 to 0.2
        let rx = x_norm * self.fast_cos(angle) - y_norm * self.fast_sin(angle);
        let ry = x_norm * self.fast_sin(angle) + y_norm * self.fast_cos(angle);
        sum += self.fast_sin((rx + ry) * 4.0 * base_freq) * 1.0;
        divisor += 1.0;

        // Layer 4: Faster spiral
        let dx2 = x_norm - 0.5;
        let dy2 = y_norm - 0.5;
        let angle2 = dy2.atan2(dx2) + time * 0.3; // Doubled from 0.15 to 0.3
        let dist2 = (dx2 * dx2 + dy2 * dy2).sqrt() * 6.0;
        sum += self.fast_sin(dist2 + angle2 * 2.0) * 0.8;
        divisor += 0.8;

        // Layer 5: Faster undulating waves
        for i in 0..complexity as u32 {
            let fi = i as f64;
            let speed = 0.2 + fi * 0.04; // Doubled from 0.1/0.02 to 0.2/0.04

            let cx = 0.5 + 0.3 * self.fast_sin(time * speed);
            let cy = 0.5 + 0.3 * self.fast_cos(time * speed + PI * 0.3);

            let dx = x_norm - cx;
            let dy = y_norm - cy;
            let dist = (dx * dx + dy * dy).sqrt();

            let freq = (3.0 + fi) * base_freq;
            sum += self.fast_sin(dist * freq + time * (0.4 + fi * 0.1)) * (1.2 / (fi + 1.0)); // Doubled time multipliers
            divisor += 1.0 / (fi + 1.0);
        }

        let normalized = (sum / divisor) * 1.2;
        (self.fast_sin(normalized * PI * 0.8) + 1.0) * 0.5
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

        let value = self.fast_sin(distance / wavelength * PI * 10.0 * freq + time);
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

        // Multiply by 2π and frequency to get more cycles
        let wave_angle = x_norm * frequency * self.config.common.frequency * PI * 4.0
            + phase
            + self.time * 2.0 * PI;

        let scaled_amplitude = amplitude * self.config.common.amplitude;
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
        // Normalize time and slow it down overall
        let normalized_time = (self.time % (PI * 2.0)) * 0.25;  // Reduced from 0.5 to 0.25

        // Convert coordinates to floating point for rotation
        let x_norm = x as f64 / self.width as f64;
        let y_norm = y as f64 / self.height as f64;

        // Center coordinates around origin for rotation
        let cx = x_norm - 0.5;
        let cy = y_norm - 0.5;

        // Create an even gentler pulsing motion
        let pulse = self.fast_sin(normalized_time * PI * 0.4) * 0.05;  // Reduced from 0.7 to 0.4
        let dynamic_scale = scale * (1.0 + pulse);

        // Apply slower rotation
        let base_rotation = rotation * PI / 180.0;
        let time_rotation = normalized_time * 2.0;  // Reduced from 4.0 to 2.0
        let rot_rad = base_rotation + time_rotation;

        let rx = cx * self.fast_cos(rot_rad) - cy * self.fast_sin(rot_rad);
        let ry = cx * self.fast_sin(rot_rad) + cy * self.fast_cos(rot_rad);

        // Move back from origin and apply dynamic scale
        let scaled_x = (rx + 0.5) * dynamic_scale * self.width as f64;
        let scaled_y = (ry + 0.5) * dynamic_scale * self.height as f64;

        // Calculate cells with floating-point precision
        let cell_size = size as f64 * 2.0;
        let fx = (scaled_x / cell_size).fract() - 0.5;
        let fy = (scaled_y / cell_size).fract() - 0.5;

        // Create soft circular pattern within each cell
        let dist_from_center = (fx * fx + fy * fy).sqrt() * 1.5;

        // Add slower wave motion
        let wave = self.fast_sin(normalized_time * PI * 0.8 + dist_from_center * PI * 2.0) * 0.1;  // Reduced from 1.2 to 0.8

        // Create organic pattern
        let cell_x = (scaled_x / cell_size).floor() as i32;
        let cell_y = (scaled_y / cell_size).floor() as i32;
        let checker = ((cell_x + cell_y) % 2).abs() as f64;

        // Create soft transitions between cells
        let edge_fade = (1.0 - dist_from_center).max(0.0) * (blur + 0.3);
        let base = checker * (1.0 - edge_fade) + (1.0 - checker) * edge_fade;

        // Combine everything with gentler transitions
        let result = base * 0.8 + wave * 0.2 + pulse;

        // Add very subtle position variation
        let position_variation = (fx.abs() + fy.abs()) * 0.05;

        (result + position_variation).clamp(0.0, 1.0)
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
        let mut rng = seed;
        let mut perm: Vec<_> = (0..=255).map(|i| i as u8).collect();

        // Basic shuffle using the seed
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
    #[inline]
    fn smoothstep(t: f64) -> f64 {
        t * t * (3.0 - 2.0 * t)
    }

    /// Linear interpolation
    #[inline]
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

    // Add new helper method for interpolation
    fn interpolate_value(&self, prev_value: f64, next_value: f64, alpha: f64) -> f64 {
        // Handle wrapping for cyclic patterns (like spirals)
        let diff = next_value - prev_value;
        if diff.abs() > 0.5 {
            // Values are across the 0-1 boundary, adjust for smooth wrapping
            let wrapped_next = if diff > 0.0 {
                next_value - 1.0
            } else {
                next_value + 1.0
            };
            let interpolated = prev_value + (wrapped_next - prev_value) * alpha;
            if interpolated < 0.0 {
                interpolated + 1.0
            } else if interpolated > 1.0 {
                interpolated - 1.0
            } else {
                interpolated
            }
        } else {
            // Normal linear interpolation
            prev_value + (next_value - prev_value) * alpha
        }
    }

    // Replace the take_gradient method with this
    pub fn recreate(&self, new_width: usize, new_height: usize) -> Self {
        Self {
            config: self.config.clone(),
            gradient: Arc::clone(&self.gradient),
            time: self.time,
            width: new_width,
            height: new_height,
            sin_table: Arc::clone(&self.sin_table),
            cos_table: Arc::clone(&self.cos_table),
            perm_table: Arc::clone(&self.perm_table),
        }
    }

    /// Returns a reference to the pattern config
    pub fn config(&self) -> &PatternConfig {
        &self.config
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
            sin_table: Arc::clone(&self.sin_table),
            cos_table: Arc::clone(&self.cos_table),
            perm_table: Arc::clone(&self.perm_table),
        }
    }
}
