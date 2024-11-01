use std::f64::consts::PI;
use std::sync::Arc;

/// Utility functions and lookup tables for pattern calculations.
/// Provides optimized trigonometric functions and interpolation utilities.
pub struct PatternUtils {
    /// Lookup table for sine values (0-359 degrees)
    sin_table: Arc<Vec<f64>>,
    /// Lookup table for cosine values (0-359 degrees)
    cos_table: Arc<Vec<f64>>,
    /// Permutation table for noise generation
    perm_table: Arc<Vec<u8>>,
}

impl PatternUtils {
    /// Creates a new PatternUtils instance with pre-calculated lookup tables.
    ///
    /// # Arguments
    /// * `seed` - Random seed for permutation table initialization
    ///
    /// # Returns
    /// A new PatternUtils instance with initialized lookup tables
    pub fn new(seed: u32) -> Self {
        Self {
            sin_table: Arc::new(Self::init_sin_table()),
            cos_table: Arc::new(Self::init_cos_table()),
            perm_table: Arc::new(Self::init_perm_table(seed)),
        }
    }

    /// Fast sine calculation using lookup table.
    ///
    /// # Arguments
    /// * `angle` - Angle in radians
    ///
    /// # Returns
    /// Sine value for the given angle
    #[inline]
    #[rustfmt::skip]
    pub fn fast_sin(&self, angle: f64) -> f64 {
        let normalized_angle = angle.rem_euclid(2.0 * PI);
        let index = ((normalized_angle * 180.0 / PI) as usize) % 360;
        self.sin_table[index]
    }

    /// Fast cosine calculation using lookup table.
    ///
    /// # Arguments
    /// * `angle` - Angle in radians
    ///
    /// # Returns
    /// Cosine value for the given angle
    #[inline]
    #[rustfmt::skip]
    pub fn fast_cos(&self, angle: f64) -> f64 {
        let normalized_angle = angle.rem_euclid(2.0 * PI);
        let index = ((normalized_angle * 180.0 / PI) as usize) % 360;
        self.cos_table[index]
    }

    /// Initializes the sine lookup table with 360 degree values.
    ///
    /// # Returns
    /// Vector containing pre-calculated sine values
    fn init_sin_table() -> Vec<f64> {
        let mut table = Vec::with_capacity(360);
        let factor = PI / 180.0;
        for i in 0..360 {
            table.push((i as f64 * factor).sin());
        }
        table
    }

    /// Initializes the cosine lookup table with 360 degree values.
    ///
    /// # Returns
    /// Vector containing pre-calculated cosine values
    fn init_cos_table() -> Vec<f64> {
        let mut table = Vec::with_capacity(360);
        let factor = PI / 180.0;
        for i in 0..360 {
            table.push((i as f64 * factor).cos());
        }
        table
    }

    /// Initializes a permutation table for noise generation.
    ///
    /// # Arguments
    /// * `seed` - Random seed for generating the permutation
    ///
    /// # Returns
    /// Vector containing permuted values 0-255
    fn init_perm_table(seed: u32) -> Vec<u8> {
        let mut rng = seed;
        let mut perm: Vec<_> = (0..=255).map(|i| i as u8).collect();

        for i in (1..256).rev() {
            rng = rng.wrapping_mul(48271).wrapping_add(1);
            let j = (rng % (i + 1) as u32) as usize;
            perm.swap(i, j);
        }

        perm
    }

    /// Performs smooth interpolation using cubic Hermite curve.
    ///
    /// # Arguments
    /// * `t` - Input value between 0 and 1
    ///
    /// # Returns
    /// Smoothly interpolated value between 0 and 1
    pub fn smoothstep(t: f64) -> f64 {
        let t2 = t * t;
        t2 * (3.0 - 2.0 * t)
    }

    /// Linear interpolation between two values.
    ///
    /// # Arguments
    /// * `a` - Start value
    /// * `b` - End value
    /// * `t` - Interpolation factor (0.0-1.0)
    ///
    /// # Returns
    /// Interpolated value between a and b
    #[inline(always)]
    pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a + t * (b - a)
    }

    /// Interpolates between two values with wrapping around 1.0.
    /// Useful for smooth transitions in cyclic patterns.
    ///
    /// # Arguments
    /// * `prev_value` - Previous value
    /// * `next_value` - Next value
    /// * `alpha` - Interpolation factor (0.0-1.0)
    ///
    /// # Returns
    /// Interpolated value that properly handles wrapping around 1.0
    #[inline(always)]
    pub fn interpolate_value(prev_value: f64, next_value: f64, alpha: f64) -> f64 {
        let diff = next_value - prev_value;

        if diff.abs() <= 0.5 {
            return prev_value + diff * alpha;
        }

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
    }

    /// Hashes coordinates for Perlin noise generation
    #[inline(always)]
    pub fn hash(&self, x: i32, y: i32) -> u8 {
        let x_hash = (x & 255) as usize;
        let y_hash = (y & 255) as usize;
        // Use wrapping arithmetic for better optimization
        self.perm_table[(x_hash.wrapping_add(y_hash.wrapping_mul(256))) & 255]
    }

    /// Performs smooth interpolation using cubic Hermite curve.
    /// Takes a boolean input and returns a smoothed value.
    ///
    /// # Arguments
    /// * `edge_test` - Boolean test for edge detection
    ///
    /// # Returns
    /// Smoothly interpolated value between 0.0 and 1.0
    #[inline(always)]
    pub const fn smoothstep_bool(edge_test: bool) -> f64 {
        if edge_test {
            1.0
        } else {
            0.0
        }
    }

    /// Generates 2D Perlin noise value at given coordinates
    #[inline(always)]
    pub fn noise2d(&self, x: f64, y: f64) -> f64 {
        // Calculate grid cell coordinates
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;
        let x1 = x0 + 1;
        let y1 = y0 + 1;

        // Calculate relative position within cell
        let dx = x - x0 as f64;
        let dy = y - y0 as f64;

        // Pre-calculate smoothstep values
        let sx = Self::smoothstep(dx);
        let sy = Self::smoothstep(dy);

        // Calculate dot products with gradient vectors
        let n00 = self.gradient_dot(self.hash(x0, y0), dx, dy);
        let n10 = self.gradient_dot(self.hash(x1, y0), dx - 1.0, dy);
        let n01 = self.gradient_dot(self.hash(x0, y1), dx, dy - 1.0);
        let n11 = self.gradient_dot(self.hash(x1, y1), dx - 1.0, dy - 1.0);

        // Interpolate results
        let nx0 = Self::lerp(n00, n10, sx);
        let nx1 = Self::lerp(n01, n11, sx);
        Self::lerp(nx0, nx1, sy)
    }

    /// Calculates dot product between gradient vector and distance vector
    #[inline(always)]
    fn gradient_dot(&self, hash: u8, dx: f64, dy: f64) -> f64 {
        // Use bitwise operations for faster gradient selection
        match hash & 3 {
            0 => dx + dy,  // ( 1,  1)
            1 => -dx + dy, // (-1,  1)
            2 => dx - dy,  // ( 1, -1)
            _ => -dx - dy, // (-1, -1)
        }
    }
}

impl Clone for PatternUtils {
    fn clone(&self) -> Self {
        Self {
            sin_table: Arc::clone(&self.sin_table),
            cos_table: Arc::clone(&self.cos_table),
            perm_table: Arc::clone(&self.perm_table),
        }
    }
}
