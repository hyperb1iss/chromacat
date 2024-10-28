//! Integration tests for the pattern generation system
//! Tests common behaviors and interactions between patterns and the engine.

use chromacat::pattern::{
    PatternConfig, PatternEngine, PatternParams,
    CheckerboardParams, DiagonalParams, DiamondParams, HorizontalParams,
    PerlinParams, PlasmaParams, RippleParams, SpiralParams, WaveParams,
};
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
        let config = PatternConfig::new(params);
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

#[test]
fn test_all_patterns_bounds() {
    let test = PatternTest::new();
    let patterns = vec![
        PatternParams::Horizontal(HorizontalParams::default()),
        PatternParams::Diagonal(DiagonalParams::default()),
        PatternParams::Plasma(PlasmaParams::default()),
        PatternParams::Ripple(RippleParams::default()),
        PatternParams::Wave(WaveParams::default()),
        PatternParams::Spiral(SpiralParams::default()),
        PatternParams::Checkerboard(CheckerboardParams::default()),
        PatternParams::Diamond(DiamondParams::default()),
        PatternParams::Perlin(PerlinParams::default()),
    ];

    for params in patterns {
        let engine = test.create_engine(params.clone());
        test.assert_pattern_bounds(&engine);
    }
}

#[test]
fn test_pattern_animation() {
    let test = PatternTest::new();
    let animated_patterns = vec![
        PatternParams::Wave(WaveParams::default()),
        PatternParams::Ripple(RippleParams::default()),
        PatternParams::Spiral(SpiralParams::default()),
        PatternParams::Plasma(PlasmaParams::default()),
    ];

    for pattern in animated_patterns {
        eprintln!("\nDEBUG: Testing animation for pattern: {:?}", pattern);
        let mut config = PatternConfig::new(pattern.clone());
        // Set speed to ensure animation occurs
        config.common.speed = 1.0;
        let mut engine = PatternEngine::new(create_test_gradient(), config, test.width, test.height);

        // Get initial value
        let initial = engine.get_value_at(50, 50).unwrap();
        eprintln!("DEBUG: Initial value: {}", initial);
        eprintln!("DEBUG: Current time: {}", engine.time());

        // Update animation time
        engine.update(1.0);
        eprintln!("DEBUG: Updated time to: {}", engine.time());

        // Get value after animation
        let animated = engine.get_value_at(50, 50).unwrap();
        eprintln!("DEBUG: Animated value: {}", animated);

        assert_ne!(
            initial, animated,
            "Pattern {:?} should produce different values after animation",
            pattern
        );
    }
}

#[test]
fn test_pattern_determinism() {
    let test = PatternTest::new();
    let patterns = vec![
        PatternParams::Horizontal(HorizontalParams::default()),
        PatternParams::Diagonal(DiagonalParams::default()),
        PatternParams::Checkerboard(CheckerboardParams::default()),
        PatternParams::Diamond(DiamondParams::default()),
        PatternParams::Perlin(PerlinParams::default()),
    ];

    for params in patterns {
        let engine = test.create_engine(params.clone());
        let first = engine.get_value_at(50, 50).unwrap();
        let second = engine.get_value_at(50, 50).unwrap();
        assert_eq!(
            first, second,
            "Pattern {:?} should produce consistent values for same coordinates",
            params
        );
    }
}
