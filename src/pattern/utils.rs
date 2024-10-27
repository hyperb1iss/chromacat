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
        (0..360).map(|i| (i as f64 * PI / 180.0).sin()).collect()
    }

    /// Initializes the cosine lookup table with 360 degree values.
    ///
    /// # Returns
    /// Vector containing pre-calculated cosine values
    fn init_cos_table() -> Vec<f64> {
        (0..360).map(|i| (i as f64 * PI / 180.0).cos()).collect()
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
        t * t * (3.0 - 2.0 * t)
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
    pub fn interpolate_value(prev_value: f64, next_value: f64, alpha: f64) -> f64 {
        let diff = next_value - prev_value;
        if diff.abs() > 0.5 {
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
            prev_value + (next_value - prev_value) * alpha
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
