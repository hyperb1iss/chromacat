use chromacat::pattern::{
    CheckerboardParams, CommonParams, DiagonalParams, DiamondParams, HorizontalParams,
    PatternConfig, PatternEngine, PatternParams, PerlinParams, PlasmaParams, RippleParams,
    SpiralParams, WaveParams,
};
use colorgrad::{Color, Gradient, GradientBuilder, LinearGradient};

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

fn create_test_config() -> PatternConfig {
    PatternConfig {
        common: CommonParams {
            frequency: 1.0,
            amplitude: 1.0,
            speed: 1.0,
            correct_aspect: true,
            aspect_ratio: 0.5,
            theme_name: Some("test".to_string()),
        },
        params: PatternParams::Horizontal(HorizontalParams::default()),
    }
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
        let mut config = create_test_config();
        config.params = pattern.clone();
        let engine = PatternEngine::new(create_test_gradient(), config, 100, 100);

        // Test multiple points
        for x in [0, 49, 99] {
            for y in [0, 49, 99] {
                let value = engine.get_value_at(x, y).unwrap();
                assert!(
                    (0.0..=1.0).contains(&value),
                    "Pattern {pattern:?} value out of range: {value} at ({x}, {y})"
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
        eprintln!("\nDEBUG: Testing animation for pattern: {pattern:?}");
        let mut config = create_test_config();
        config.params = pattern.clone();
        // Ensure animation speed is high enough to see changes
        config.common.speed = 2.0; // Increase speed
        let mut engine = PatternEngine::new(create_test_gradient(), config, 100, 100);

        // Test that animation updates produce different values
        let initial = engine.get_value_at(50, 50).unwrap();
        eprintln!("DEBUG: Initial value at (50,50): {initial}");
        eprintln!("DEBUG: Current time: {}", engine.time());

        // Use a larger time delta to ensure visible change
        engine.update(2.0); // Increase time delta
        eprintln!("DEBUG: Updated time to: {}", engine.time());

        let after_update = engine.get_value_at(50, 50).unwrap();
        eprintln!("DEBUG: Value after update at (50,50): {after_update}");

        assert_ne!(
            initial, after_update,
            "Pattern {pattern:?} should animate over time"
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
        let mut config = PatternConfig::new(pattern.clone());
        config.common.theme_name = Some("test".to_string());
        let engine = PatternEngine::new(create_test_gradient(), config, 100, 100);

        // Test that same coordinates produce same values
        let first = engine.get_value_at(50, 50).unwrap();
        let second = engine.get_value_at(50, 50).unwrap();

        assert_eq!(
            first, second,
            "Pattern {pattern:?} should produce consistent values for same coordinates"
        );
    }
}
