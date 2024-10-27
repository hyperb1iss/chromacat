use colorgrad::{Color, Gradient, GradientBuilder, LinearGradient};
use chromacat::pattern::{
    PatternConfig, PatternEngine, PatternParams,
    CheckerboardParams, DiagonalParams, DiamondParams, HorizontalParams,
    PerlinParams, PlasmaParams, RippleParams, SpiralParams, WaveParams,
};

fn create_test_gradient() -> Box<dyn Gradient + Send + Sync> {
    let gradient = GradientBuilder::new()
        .colors(&[
            Color::new(1.0, 0.0, 0.0, 1.0),
            Color::new(0.0, 0.0, 1.0, 1.0),
        ])
        .build::<LinearGradient>()
        .unwrap();
    Box::new(gradient)
}

#[test]
fn test_pattern_value_ranges() {
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

    for pattern in patterns {
        let config = PatternConfig::new(pattern.clone());
        let engine = PatternEngine::new(create_test_gradient(), config, 100, 100);

        // Test multiple points
        for x in [0, 49, 99] {
            for y in [0, 49, 99] {
                let value = engine.get_value_at(x, y).unwrap();
                assert!(
                    (0.0..=1.0).contains(&value),
                    "Pattern {:?} value out of range: {} at ({}, {})",
                    pattern,
                    value,
                    x,
                    y
                );
            }
        }
    }
}

#[test]
fn test_pattern_animation() {
    let animated_patterns = vec![
        PatternParams::Wave(WaveParams::default()),
        PatternParams::Ripple(RippleParams::default()),
        PatternParams::Spiral(SpiralParams::default()),
        PatternParams::Plasma(PlasmaParams::default()),
    ];

    for pattern in animated_patterns {
        let config = PatternConfig::new(pattern.clone());
        let mut engine = PatternEngine::new(create_test_gradient(), config, 100, 100);

        // Test that animation updates produce different values
        let initial = engine.get_value_at(50, 50).unwrap();
        engine.update(0.5);
        let after_update = engine.get_value_at(50, 50).unwrap();

        assert_ne!(
            initial, after_update,
            "Pattern {:?} should animate over time",
            pattern
        );
    }
}

#[test]
fn test_pattern_determinism() {
    let static_patterns = vec![
        PatternParams::Horizontal(HorizontalParams::default()),
        PatternParams::Diagonal(DiagonalParams::default()),
        PatternParams::Checkerboard(CheckerboardParams::default()),
        PatternParams::Diamond(DiamondParams::default()),
        PatternParams::Perlin(PerlinParams::default()),
    ];

    for pattern in static_patterns {
        let config = PatternConfig::new(pattern.clone());
        let engine = PatternEngine::new(create_test_gradient(), config, 100, 100);

        // Test that same coordinates produce same values
        let first = engine.get_value_at(50, 50).unwrap();
        let second = engine.get_value_at(50, 50).unwrap();

        assert_eq!(
            first, second,
            "Pattern {:?} should produce consistent values for same coordinates",
            pattern
        );
    }
}
