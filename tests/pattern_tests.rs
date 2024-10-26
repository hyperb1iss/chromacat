//! Tests for the pattern generation system
//!
//! This module provides comprehensive testing for all pattern types,
//! their parameters, and behavior.

use chromacat::pattern::{CommonParams, PatternConfig, PatternEngine, PatternParams};

/// Test fixture for pattern tests
#[derive(Default)]
struct PatternTest {
    width: usize,
    height: usize,
}

impl PatternTest {
    fn new() -> Self {
        Self {
            width: 100,
            height: 100,
        }
    }

    fn create_engine(&self, params: PatternParams) -> PatternEngine {
        let config = PatternConfig {
            common: CommonParams::default(),
            params,
        };
        PatternEngine::new(config, self.width, self.height)
    }

    fn create_engine_with_common(
        &self,
        params: PatternParams,
        common: CommonParams,
    ) -> PatternEngine {
        let config = PatternConfig { common, params };
        PatternEngine::new(config, self.width, self.height)
    }

    fn assert_pattern_bounds(&self, engine: &PatternEngine) {
        for y in 0..self.height {
            for x in 0..self.width {
                let value = engine.get_value_at(x, y).unwrap();
                assert!(
                    (0.0..=1.0).contains(&value),
                    "Pattern value out of bounds at ({}, {}): {}",
                    x,
                    y,
                    value
                );
            }
        }
    }
}

/// Basic pattern tests
mod basic_patterns {
    use super::*;

    #[test]
    fn test_horizontal_pattern_values() {
        let test = PatternTest::new();
        let engine = test.create_engine(PatternParams::Horizontal);

        // Test origin (0, 0)
        assert_eq!(engine.get_value_at(0, 0).unwrap(), 0.0);

        // Test right edge
        assert_eq!(engine.get_value_at(99, 0).unwrap(), 1.0);

        // Test center
        assert!((engine.get_value_at(49, 0).unwrap() - 0.49494949494949497).abs() < 1e-10);
    }

    #[test]
    fn test_diagonal_pattern() {
        let test = PatternTest::new();
        let angles = [0, 45, 90, 180, 270, 360];

        for angle in angles {
            let engine = test.create_engine(PatternParams::Diagonal { angle });
            test.assert_pattern_bounds(&engine);

            // Test corner values
            let top_left = engine.get_value_at(0, 0).unwrap();
            let bottom_right = engine.get_value_at(99, 99).unwrap();

            // Values should differ in diagonal pattern
            assert_ne!(top_left, bottom_right);
        }
    }
}

/// Wave pattern tests
mod wave_patterns {
    use super::*;

    #[test]
    fn test_wave_amplitude() {
        let test = PatternTest::new();
        let amplitudes = [0.1f64, 0.5, 1.0, 2.0];

        for &amp in &amplitudes {
            let engine = test.create_engine(PatternParams::Wave {
                amplitude: amp,
                frequency: 1.0,
                phase: 0.0,
                offset: 0.5,
            });

            // Sample middle row
            let mid_y = test.height / 2;
            let mut min_val: f64 = 1.0;
            let mut max_val: f64 = 0.0;

            for x in 0..test.width {
                let value = engine.get_value_at(x, mid_y).unwrap();
                min_val = min_val.min(value);
                max_val = max_val.max(value);
            }

            // Range should be proportional to amplitude, but not exceed 1.0
            let expected_range = amp.min(1.0);
            let range = max_val - min_val;
            assert!(
                (range - expected_range).abs() < 0.1,
                "Expected range {} for amplitude {}, got {}",
                expected_range,
                amp,
                range
            );
        }
    }

    #[test]
    fn test_wave_frequency() {
        let test = PatternTest::new();
        let engine = test.create_engine(PatternParams::Wave {
            amplitude: 1.0,
            frequency: 2.0,
            phase: 0.0,
            offset: 0.5,
        });

        // Count zero crossings to verify frequency
        let mid_y = test.height / 2;
        let mut crossings = 0;
        let mut last_value = engine.get_value_at(0, mid_y).unwrap();

        for x in 1..test.width {
            let value = engine.get_value_at(x, mid_y).unwrap();
            if (value - 0.5) * (last_value - 0.5) < 0.0 {
                crossings += 1;
            }
            last_value = value;
        }

        assert!(crossings > 2, "Wave frequency too low");
    }
}

/// Animation tests
mod animation_tests {
    use super::*;

    #[test]
    fn test_animation_time() {
        let test = PatternTest::new();
        let mut engine = test.create_engine(PatternParams::Wave {
            amplitude: 1.0,
            frequency: 1.0,
            phase: 0.0,
            offset: 0.5,
        });

        let initial_value = engine.get_value_at(50, 50).unwrap();

        // Update time by full cycle
        engine.update(1.0);
        let wrapped_value = engine.get_value_at(50, 50).unwrap();
        assert!(
            (initial_value - wrapped_value).abs() < 0.01,
            "Pattern not repeating after full cycle"
        );
    }

    #[test]
    fn test_animation_speed() {
        let test = PatternTest::new();
        let delta = 0.5;
        let samples = vec![
            (0.5, 0.25), // half speed
            (1.0, 0.5),  // normal speed
            (2.0, 0.0),  // double speed (wrapped)
        ];

        for &(speed, expected_time) in &samples {
            let common = CommonParams {
                speed,
                ..Default::default()
            };

            let mut engine = test.create_engine_with_common(
                PatternParams::Wave {
                    amplitude: 1.0,
                    frequency: 1.0,
                    phase: 0.0,
                    offset: 0.5,
                },
                common,
            );

            engine.update(delta);
            assert!(
                (engine.time() - expected_time).abs() < 0.001,
                "Unexpected time for speed {}: {} != {}",
                speed,
                engine.time(),
                expected_time
            );
        }
    }
}
