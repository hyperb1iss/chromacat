//! Integration tests for the pattern generation system
//!
//! This module provides comprehensive testing for all pattern types,
//! their parameters, and behavior.

use chromacat::pattern::{CommonParams, PatternConfig, PatternEngine, PatternParams};
use colorgrad::{Color, Gradient};

/// Mock gradient for testing
struct MockGradient;

impl Gradient for MockGradient {
    fn at(&self, t: f32) -> Color {
        Color::new(t, t, t, 1.0_f32)
    }
}

fn create_test_gradient() -> Box<dyn Gradient + Send + Sync> {
    Box::new(MockGradient)
}

/// Test fixture for pattern tests
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
        PatternEngine::new(create_test_gradient(), config, self.width, self.height)
    }

    fn create_engine_with_common(
        &self,
        params: PatternParams,
        common: CommonParams,
    ) -> PatternEngine {
        let config = PatternConfig { common, params };
        PatternEngine::new(create_test_gradient(), config, self.width, self.height)
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
        const EPSILON: f64 = 1e-10;

        // Test origin (0, 0)
        let start_value = engine.get_value_at(0, 0).unwrap();
        assert!(
            (start_value - 0.0).abs() < EPSILON,
            "Start value should be 0.0, got {}",
            start_value
        );

        // Test right edge
        let end_value = engine.get_value_at(test.width - 1, 0).unwrap();
        assert!(
            (end_value - 1.0).abs() < EPSILON,
            "End value should be 1.0, got {}",
            end_value
        );

        // Test middle
        let mid_x = test.width / 2;
        let expected_mid = mid_x as f64 / (test.width - 1) as f64;
        let mid_value = engine.get_value_at(mid_x, 0).unwrap();
    }

    #[test]
    fn test_diagonal_pattern() {
        let test = PatternTest::new();
        let angles = [0, 45, 90, 180, 270, 360];
        const EPSILON: f64 = 1e-6;

        for angle in angles {
            let engine = test.create_engine(PatternParams::Diagonal { angle });
            test.assert_pattern_bounds(&engine);

            // Sample points across the pattern
            let corners = [
                engine.get_value_at(0, 0).unwrap(),               // top-left
                engine.get_value_at(test.width - 1, 0).unwrap(),  // top-right
                engine.get_value_at(0, test.height - 1).unwrap(), // bottom-left
                engine
                    .get_value_at(test.width - 1, test.height - 1)
                    .unwrap(), // bottom-right
            ];

            // Check for variation in values
            let max_val = corners.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let min_val = corners.iter().copied().fold(f64::INFINITY, f64::min);

            assert!(
                (max_val - min_val) > EPSILON,
                "Diagonal pattern at angle {} should show variation. Values: {:?}, range: {}",
                angle,
                corners,
                max_val - min_val
            );
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

/// Complex pattern tests
mod complex_patterns {
    use super::*;

    #[test]
    fn test_plasma_pattern() {
        let test = PatternTest::new();
        let engine = test.create_engine(PatternParams::Plasma {
            complexity: 3.0,
            scale: 1.0,
        });

        test.assert_pattern_bounds(&engine);
    }

    #[test]
    fn test_ripple_pattern() {
        let test = PatternTest::new();
        let engine = test.create_engine(PatternParams::Ripple {
            center_x: 0.5,
            center_y: 0.5,
            wavelength: 1.0,
            damping: 0.5,
        });

        test.assert_pattern_bounds(&engine);
    }

    #[test]
    fn test_spiral_pattern() {
        let test = PatternTest::new();
        let engine = test.create_engine(PatternParams::Spiral {
            density: 1.0,
            rotation: 0.0,
            expansion: 1.0,
            clockwise: true,
        });

        test.assert_pattern_bounds(&engine);
    }

    #[test]
    fn test_perlin_pattern() {
        let test = PatternTest::new();
        let engine = test.create_engine(PatternParams::Perlin {
            octaves: 4,
            persistence: 0.5,
            scale: 1.0,
            seed: 0,
        });

        test.assert_pattern_bounds(&engine);

        // Test value consistency
        let v1 = engine.get_value_at(50, 50).unwrap();
        let v2 = engine.get_value_at(50, 50).unwrap();
        assert_eq!(v1, v2, "Perlin noise should be deterministic");
    }
}

/// Parameter validation tests
mod parameter_tests {
    use super::*;

    #[test]
    fn test_common_params() {
        let test = PatternTest::new();
        let params = vec![
            CommonParams {
                frequency: 0.1,
                amplitude: 0.1,
                speed: 0.0,
            },
            CommonParams {
                frequency: 10.0,
                amplitude: 2.0,
                speed: 1.0,
            },
        ];

        for common in params {
            let engine = test.create_engine_with_common(PatternParams::Horizontal, common);
            test.assert_pattern_bounds(&engine);
        }
    }

    #[test]
    fn test_pattern_specific_params() {
        let test = PatternTest::new();
        let test_cases = vec![
            PatternParams::Plasma {
                complexity: 10.0,
                scale: 5.0,
            },
            PatternParams::Wave {
                amplitude: 2.0,
                frequency: 5.0,
                phase: 6.28,
                offset: 1.0,
            },
            PatternParams::Diagonal { angle: 360 },
        ];

        for params in test_cases {
            let engine = test.create_engine(params);
            test.assert_pattern_bounds(&engine);
        }
    }
}
