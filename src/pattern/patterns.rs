use crate::pattern::utils::PatternUtils;
use std::f64::consts::PI;

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

/// Parameters for configuring ripple pattern effects
pub struct RippleParams {
    /// X-coordinate of the ripple center (0.0-1.0)
    pub center_x: f64,
    /// Y-coordinate of the ripple center (0.0-1.0)
    pub center_y: f64,
    /// Distance between ripple waves (0.1-5.0)
    pub wavelength: f64,
    /// How quickly ripples fade out with distance (0.0-1.0)
    pub damping: f64,
    /// Speed of ripple animation (0.1-10.0)
    pub frequency: f64,
}

/// Parameters for configuring spiral pattern effects
pub struct SpiralParams {
    /// How tightly wound the spiral is (0.1-5.0)
    pub density: f64,
    /// Base rotation angle in degrees (0-360)
    pub rotation: f64,
    /// How quickly spiral expands from center (0.1-2.0)
    pub expansion: f64,
    /// Direction of spiral rotation
    pub clockwise: bool,
    /// Speed of spiral animation (0.1-10.0)
    pub frequency: f64,
}

impl Patterns {
    /// Creates a new Patterns instance
    ///
    /// # Arguments
    /// * `width` - Width of the pattern area in pixels
    /// * `height` - Height of the pattern area in pixels
    /// * `time` - Initial animation time in seconds
    /// * `seed` - Random seed for noise-based patterns
    pub fn new(width: usize, height: usize, time: f64, seed: u32) -> Self {
        Self {
            utils: PatternUtils::new(seed),
            width,
            height,
            time,
        }
    }

    /// Generates a simple horizontal gradient pattern
    ///
    /// # Arguments
    /// * `x` - X coordinate
    /// * `_y` - Y coordinate (unused)
    ///
    /// # Returns
    /// Value between 0.0 and 1.0 representing position in gradient
    pub fn horizontal(&self, x: usize, _y: usize) -> f64 {
        if self.width <= 1 {
            return 0.0;
        }
        x as f64 / (self.width - 1) as f64
    }

    /// Generates an animated diagonal gradient pattern
    ///
    /// # Arguments
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    /// * `angle` - Base angle in degrees (0-360)
    /// * `frequency` - Animation speed multiplier
    ///
    /// # Returns
    /// Animated gradient value between 0.0 and 1.0
    pub fn diagonal(&self, x: usize, y: usize, angle: i32, frequency: f64) -> f64 {
        if self.width <= 1 || self.height <= 1 {
            return 0.0;
        }

        let x_norm = x as f64 / (self.width - 1) as f64;
        let y_norm = (y % self.height) as f64 / (self.height - 1) as f64;

        let x_scaled = x_norm * 2.0 - 1.0;
        let y_scaled = y_norm * 2.0 - 1.0;

        let time = self.time * PI * 2.0;
        let wave_offset = self.utils.fast_sin(time * 0.5 + y_norm * 8.0) * 0.2;

        let base_angle = angle as f64;
        let animated_angle = (base_angle + self.utils.fast_sin(time * 0.3) * 15.0) * PI / 180.0;

        let cos_angle = self.utils.fast_cos(animated_angle);
        let sin_angle = self.utils.fast_sin(animated_angle);

        let rotated = x_scaled * cos_angle + y_scaled * sin_angle + wave_offset;
        let perpendicular = -x_scaled * sin_angle + y_scaled * cos_angle;
        let wave_distortion = self.utils.fast_sin(perpendicular * 4.0 * frequency + time) * 0.1;

        let result = (rotated + wave_distortion + 1.0) * 0.5;
        let pulse = (self.utils.fast_sin(time * 0.7) * 0.1 + 1.0) * result;

        pulse.clamp(0.0, 1.0)
    }

    /// Generates a plasma effect pattern
    ///
    /// # Arguments
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    /// * `complexity` - Number of plasma layers (1.0-10.0)
    /// * `scale` - Size of the plasma effect (0.1-5.0)
    /// * `frequency` - Animation speed multiplier
    ///
    /// # Returns
    /// Animated plasma value between 0.0 and 1.0
    pub fn plasma(&self, x: usize, y: usize, complexity: f64, scale: f64, frequency: f64) -> f64 {
        let time = self.time * PI;
        let x_norm = x as f64 / self.width as f64;
        let y_norm = y as f64 / self.height as f64;

        let cx = 0.5 + 0.4 * self.utils.fast_sin(time * 0.4);
        let cy = 0.5 + 0.4 * self.utils.fast_cos(time * 0.43);

        let base_freq = frequency * scale * 2.0;
        let mut sum = 0.0;
        let mut divisor = 0.0;

        let dx1 = x_norm - cx;
        let dy1 = y_norm - cy;
        let dist1 = (dx1 * dx1 + dy1 * dy1).sqrt();
        sum += self.utils.fast_sin(dist1 * 8.0 * base_freq + time * 0.6) * 1.2;
        divisor += 1.2;

        sum += self.utils.fast_sin(x_norm * 5.0 * base_freq + time * 0.4) * 0.8;
        sum += self.utils.fast_sin(y_norm * 5.0 * base_freq + time * 0.47) * 0.8;
        divisor += 1.6;

        let angle = time * 0.2;
        let rx = x_norm * self.utils.fast_cos(angle) - y_norm * self.utils.fast_sin(angle);
        let ry = x_norm * self.utils.fast_sin(angle) + y_norm * self.utils.fast_cos(angle);
        sum += self.utils.fast_sin((rx + ry) * 4.0 * base_freq) * 1.0;
        divisor += 1.0;

        let dx2 = x_norm - 0.5;
        let dy2 = y_norm - 0.5;
        let angle2 = dy2.atan2(dx2) + time * 0.3;
        let dist2 = (dx2 * dx2 + dy2 * dy2).sqrt() * 6.0;
        sum += self.utils.fast_sin(dist2 + angle2 * 2.0) * 0.8;
        divisor += 0.8;

        for i in 0..complexity as u32 {
            let fi = i as f64;
            let speed = 0.2 + fi * 0.04;

            let cx = 0.5 + 0.3 * self.utils.fast_sin(time * speed);
            let cy = 0.5 + 0.3 * self.utils.fast_cos(time * speed + PI * 0.3);

            let dx = x_norm - cx;
            let dy = y_norm - cy;
            let dist = (dx * dx + dy * dy).sqrt();

            let freq = (3.0 + fi) * base_freq;
            sum += self.utils.fast_sin(dist * freq + time * (0.4 + fi * 0.1)) * (1.2 / (fi + 1.0));
            divisor += 1.0 / (fi + 1.0);
        }

        let normalized = (sum / divisor) * 1.2;
        (self.utils.fast_sin(normalized * PI * 0.8) + 1.0) * 0.5
    }

    /// Generates a ripple pattern emanating from a center point
    ///
    /// # Arguments
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    /// * `params` - RippleParams containing configuration for the ripple effect
    ///
    /// # Returns
    /// Animated ripple value between 0.0 and 1.0
    pub fn ripple(&self, x: usize, y: usize, params: RippleParams) -> f64 {
        let dx = x as f64 / self.width as f64 - params.center_x;
        let dy = y as f64 / self.height as f64 - params.center_y;
        let distance = (dx * dx + dy * dy).sqrt();

        let time = self.time * PI * 2.0;
        let value = self
            .utils
            .fast_sin(distance / params.wavelength * PI * 10.0 * params.frequency + time);
        let amplitude = (-distance * params.damping * 5.0).exp();

        (value * amplitude + 1.0) / 2.0
    }

    /// Generates a wave pattern with configurable properties
    ///
    /// # Arguments
    /// * `x` - X coordinate
    /// * `amplitude` - Height of the wave (0.1-2.0)
    /// * `frequency` - Number of waves (0.1-5.0)
    /// * `phase` - Wave phase shift (0.0-2π)
    /// * `offset` - Vertical offset (0.0-1.0)
    /// * `base_freq` - Base frequency multiplier
    ///
    /// # Returns
    /// Animated wave value between 0.0 and 1.0
    pub fn wave(
        &self,
        x: usize,
        amplitude: f64,
        frequency: f64,
        phase: f64,
        offset: f64,
        base_freq: f64,
    ) -> f64 {
        let x_norm = x as f64 / (self.width.max(1) - 1) as f64;
        let wave_angle = x_norm * frequency * base_freq * PI * 4.0 + phase + self.time * 2.0 * PI;
        let wave = self.utils.fast_sin(wave_angle) * amplitude;

        (offset + wave).clamp(0.0, 1.0)
    }

    /// Generates a spiral pattern rotating from the center
    ///
    /// # Arguments
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    /// * `params` - SpiralParams containing configuration for the spiral effect
    ///
    /// # Returns
    /// Animated spiral value between 0.0 and 1.0
    pub fn spiral(&self, x: usize, y: usize, params: SpiralParams) -> f64 {
        let center_x = self.width as f64 / 2.0;
        let center_y = self.height as f64 / 2.0;
        let dx = x as f64 - center_x;
        let dy = y as f64 - center_y;

        let mut angle = dy.atan2(dx);
        if !params.clockwise {
            angle = -angle;
        }

        let distance = (dx * dx + dy * dy).sqrt() / (self.width.min(self.height) as f64 / 2.0);
        let rot_rad = params.rotation * PI / 180.0;
        let time = self.time * PI * 2.0;

        ((angle + distance * params.density * params.expansion + rot_rad + time * params.frequency)
            % (PI * 2.0))
            / (PI * 2.0)
    }

    /// Generates an animated checkerboard pattern
    ///
    /// # Arguments
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    /// * `size` - Size of checker squares (1-10)
    /// * `blur` - Blur between squares (0.0-1.0)
    /// * `rotation` - Pattern rotation in degrees (0-360)
    /// * `scale` - Overall pattern scale (0.1-5.0)
    ///
    /// # Returns
    /// Animated checkerboard value between 0.0 and 1.0
    pub fn checkerboard(
        &self,
        x: usize,
        y: usize,
        size: usize,
        blur: f64,
        rotation: f64,
        scale: f64,
    ) -> f64 {
        let normalized_time = (self.time % (PI * 2.0)) * 0.25;
        let x_norm = x as f64 / self.width as f64;
        let y_norm = y as f64 / self.height as f64;

        let cx = x_norm - 0.5;
        let cy = y_norm - 0.5;

        let pulse = self.utils.fast_sin(normalized_time * PI * 0.4) * 0.05;
        let dynamic_scale = scale * (1.0 + pulse);

        let base_rotation = rotation * PI / 180.0;
        let time_rotation = normalized_time * 2.0;
        let rot_rad = base_rotation + time_rotation;

        let rx = cx * self.utils.fast_cos(rot_rad) - cy * self.utils.fast_sin(rot_rad);
        let ry = cx * self.utils.fast_sin(rot_rad) + cy * self.utils.fast_cos(rot_rad);

        let scaled_x = (rx + 0.5) * dynamic_scale * self.width as f64;
        let scaled_y = (ry + 0.5) * dynamic_scale * self.height as f64;

        let cell_size = size as f64 * 2.0;
        let fx = (scaled_x / cell_size).fract() - 0.5;
        let fy = (scaled_y / cell_size).fract() - 0.5;

        let dist_from_center = (fx * fx + fy * fy).sqrt() * 1.5;
        let wave = self
            .utils
            .fast_sin(normalized_time * PI * 0.8 + dist_from_center * PI * 2.0)
            * 0.1;

        let cell_x = (scaled_x / cell_size).floor() as i32;
        let cell_y = (scaled_y / cell_size).floor() as i32;
        let checker = ((cell_x + cell_y) % 2).abs() as f64;

        let edge_fade = (1.0 - dist_from_center).max(0.0) * (blur + 0.3);
        let base = checker * (1.0 - edge_fade) + (1.0 - checker) * edge_fade;

        let result = base * 0.8 + wave * 0.2 + pulse;
        let position_variation = (fx.abs() + fy.abs()) * 0.05;

        (result + position_variation).clamp(0.0, 1.0)
    }

    /// Generates a diamond-shaped pattern
    ///
    /// # Arguments
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    /// * `size` - Size of diamond shapes (0.1-5.0)
    /// * `offset` - Pattern offset (0.0-1.0)
    /// * `sharpness` - Edge sharpness (0.1-5.0)
    /// * `rotation` - Pattern rotation in degrees (0-360)
    ///
    /// # Returns
    /// Animated diamond pattern value between 0.0 and 1.0
    pub fn diamond(
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

        let rot_rad = rotation * PI / 180.0;
        let rx = x_norm * self.utils.fast_cos(rot_rad) - y_norm * self.utils.fast_sin(rot_rad);
        let ry = x_norm * self.utils.fast_sin(rot_rad) + y_norm * self.utils.fast_cos(rot_rad);

        let freq = size;
        let time = self.time * PI * 2.0;

        let dx = (rx * freq).abs();
        let dy = (ry * freq).abs();
        let diamond_val = (dx + dy + offset) % 1.0;

        (diamond_val.powf(sharpness) + self.utils.fast_sin(time)) / 2.0
    }

    /// Generates a Perlin noise-based pattern
    ///
    /// # Arguments
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    /// * `octaves` - Number of noise layers (1-8)
    /// * `persistence` - How quickly amplitudes diminish (0.0-1.0)
    /// * `scale` - Overall noise scale (0.1-5.0)
    ///
    /// # Returns
    /// Animated noise value between 0.0 and 1.0
    pub fn perlin(&self, x: usize, y: usize, octaves: u32, persistence: f64, scale: f64) -> f64 {
        let x_norm = x as f64 / self.width as f64;
        let y_norm = y as f64 / self.height as f64;
        let freq = scale;

        let mut total = 0.0;
        let mut frequency = freq;
        let mut amplitude = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            total += self.perlin_noise(
                x_norm * frequency + self.time,
                y_norm * frequency + self.time,
            ) * amplitude;

            max_value += amplitude;
            amplitude *= persistence;
            frequency *= 2.0;
        }

        (total / max_value + 1.0) / 2.0
    }

    /// Calculates a single octave of Perlin noise
    ///
    /// # Arguments
    /// * `x` - Normalized X coordinate
    /// * `y` - Normalized Y coordinate
    ///
    /// # Returns
    /// Raw noise value between -1.0 and 1.0
    fn perlin_noise(&self, x: f64, y: f64) -> f64 {
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;
        let x1 = x0 + 1;
        let y1 = y0 + 1;

        let dx = x - x0 as f64;
        let dy = y - y0 as f64;

        let sx = PatternUtils::smoothstep(dx);
        let sy = PatternUtils::smoothstep(dy);

        let n00 = self.gradient_dot(x0, y0, dx, dy);
        let n10 = self.gradient_dot(x1, y0, dx - 1.0, dy);
        let n01 = self.gradient_dot(x0, y1, dx, dy - 1.0);
        let n11 = self.gradient_dot(x1, y1, dx - 1.0, dy - 1.0);

        let nx0 = PatternUtils::lerp(n00, n10, sx);
        let nx1 = PatternUtils::lerp(n01, n11, sx);
        PatternUtils::lerp(nx0, nx1, sy)
    }

    /// Calculates dot product between gradient vector and distance vector
    ///
    /// # Arguments
    /// * `hash` - Hash value determining gradient vector
    /// * `_y` - Y coordinate (unused)
    /// * `dx` - X distance from grid point
    /// * `dy` - Y distance from grid point
    ///
    /// # Returns
    /// Dot product result for noise calculation
    fn gradient_dot(&self, hash: i32, _y: i32, dx: f64, dy: f64) -> f64 {
        match hash & 3 {
            0 => dx + dy,
            1 => -dx + dy,
            2 => dx - dy,
            _ => -dx - dy,
        }
    }
}
