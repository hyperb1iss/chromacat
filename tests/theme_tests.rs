use chromacat::error::ChromaCatError;
use chromacat::themes::{self, ColorStop, Distribution, Easing, Repeat, RepeatMode, ThemeDefinition};
use std::f32::consts::PI;

// Helper function to create a basic theme for testing
fn create_test_theme() -> ThemeDefinition {
    ThemeDefinition {
        name: "test".to_string(),
        desc: "Test theme".to_string(),
        colors: vec![
            ColorStop {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                position: Some(0.0),
                name: None,
            },
            ColorStop {
                r: 0.0,
                g: 0.0,
                b: 1.0,
                position: Some(1.0),
                name: None,
            },
        ],
        dist: Distribution::Even,
        repeat: Repeat::Named(RepeatMode::None),
        speed: 1.0,
        ease: Easing::Linear,
    }
}

#[test]
fn test_theme_registry() {
    // Test default theme availability
    let rainbow = themes::get_theme("rainbow").unwrap();
    assert_eq!(rainbow.name, "rainbow");
    assert!(rainbow.colors.len() >= 2);

    // Test theme categories
    let categories = themes::list_categories();
    assert!(categories.contains(&"default"));
    assert!(categories.contains(&"space"));
    assert!(categories.contains(&"tech"));

    // Test theme count
    assert!(themes::theme_count() > 0);
}

#[test]
fn test_theme_validation() {
    let mut theme = create_test_theme();
    assert!(theme.validate().is_ok());

    // Test invalid color values
    theme.colors[0].r = 1.5;
    assert!(matches!(
        theme.validate(),
        Err(ChromaCatError::GradientError(_))
    ));

    // Test invalid position
    theme.colors[0].r = 1.0;
    theme.colors[0].position = Some(1.5);
    assert!(matches!(
        theme.validate(),
        Err(ChromaCatError::GradientError(_))
    ));

    // Test insufficient colors
    theme.colors = vec![ColorStop {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        position: Some(0.0),
        name: None,
    }];
    assert!(matches!(
        theme.validate(),
        Err(ChromaCatError::GradientError(_))
    ));

    // Test invalid speed
    theme = create_test_theme();
    theme.speed = 0.0;
    assert!(matches!(
        theme.validate(),
        Err(ChromaCatError::GradientError(_))
    ));
}

#[test]
fn test_gradient_creation() {
    let theme = create_test_theme();
    let gradient = theme.create_gradient().unwrap();

    // Test gradient interpolation
    let start = gradient.at(0.0);
    let end = gradient.at(1.0);
    let mid = gradient.at(0.5);

    assert_eq!(start.r, 1.0);
    assert_eq!(start.g, 0.0);
    assert_eq!(start.b, 0.0);

    assert_eq!(end.r, 0.0);
    assert_eq!(end.g, 0.0);
    assert_eq!(end.b, 1.0);

    // Middle should be a blend
    assert!(mid.r > 0.0 && mid.r < 1.0);
    assert_eq!(mid.g, 0.0);
    assert!(mid.b > 0.0 && mid.b < 1.0);
}

#[test]
fn test_distribution_functions() {
    let theme = create_test_theme();
    let distributions = [
        (Distribution::Even, 0.5, 0.5),
        (Distribution::Front, 0.5, 0.25),
        (Distribution::Back, 0.5, 0.75),
        (Distribution::Center, 0.25, 0.125),
        (Distribution::Alt, 0.5, (0.5 * PI).sin() * 0.5 + 0.5),
    ];

    for (dist, input, expected) in distributions {
        let mut test_theme = theme.clone();
        // Clone dist before moving it
        let dist_debug = format!("{:?}", dist);
        test_theme.dist = dist;
        let result = test_theme.apply_distribution(input);
        assert!(
            (result - expected).abs() < 0.001,
            "Distribution {} failed: expected {}, got {}",
            dist_debug,
            expected,
            result
        );
    }
}

#[test]
fn test_repeat_modes() {
    let theme = create_test_theme();
    let time = 0.5;
    let test_cases = [
        (Repeat::Named(RepeatMode::None), 1.5, 1.0),
        (Repeat::Named(RepeatMode::Mirror), 1.5, 0.5),
        (Repeat::Named(RepeatMode::Repeat), 1.5, 0.5),
        // For rotate: t + time * rate = 0.5 + 0.5 * 1.0 = 1.0, then fract() = 0.0
        (Repeat::Function("rotate".to_string(), 1.0), 0.5, 0.0),
        // For pulse: (time * rate * PI).sin() = (0.5 * 1.0 * PI).sin() ≈ 1.0
        // Then (t + phase) * 0.5 = (0.5 + 1.0) * 0.5 = 0.75
        (Repeat::Function("pulse".to_string(), 1.0), 0.5, 0.75),
    ];

    for (repeat, input, expected) in test_cases {
        let mut test_theme = theme.clone();
        let repeat_debug = format!("{:?}", repeat);
        test_theme.repeat = repeat;
        let result = test_theme.apply_repeat(input, time);
        assert!(
            (result - expected).abs() < 0.001,
            "Repeat mode {} failed: expected {}, got {}",
            repeat_debug,
            expected,
            result
        );
    }
}

#[test]
fn test_easing_functions() {
    let theme = create_test_theme();
    let test_points = [0.0, 0.25, 0.5, 0.75, 1.0];
    // Removed Elastic easing as it intentionally overshoots for bounce effect
    let easings = [
        Easing::Linear,
        Easing::Smooth,
        Easing::Smoother,
        Easing::Sine,
        Easing::Exp,
    ];

    for easing in easings {
        let mut test_theme = theme.clone();
        let easing_debug = format!("{:?}", easing);
        test_theme.ease = easing;

        for &t in &test_points {
            let result = test_theme.apply_easing(t);
            assert!(
                (0.0..=1.0).contains(&result),
                "Easing {} produced out of bounds value {} for input {}",
                easing_debug,
                result,
                t
            );
        }
    }
}

#[test]
fn test_theme_categories() {
    for category in themes::list_categories() {
        if let Some(theme_names) = themes::list_category(category) {
            for name in theme_names {
                let theme = themes::get_theme(name).unwrap();
                assert!(theme.validate().is_ok());
                assert!(theme.create_gradient().is_ok());
            }
        }
    }
}

#[test]
fn test_invalid_theme_access() {
    assert!(matches!(
        themes::get_theme("nonexistent"),
        Err(ChromaCatError::InvalidTheme(_))
    ));
}

// Add a separate test for Elastic easing
#[test]
fn test_elastic_easing() {
    let mut theme = create_test_theme();
    theme.ease = Easing::Elastic;
    
    // Test only the boundary conditions where we expect exact values
    assert_eq!(theme.apply_easing(0.0), 0.0);
    assert_eq!(theme.apply_easing(1.0), 1.0);
    
    // Test that middle values produce elastic effect (may go outside 0-1 range)
    let mid = theme.apply_easing(0.5);
    assert!(mid != 0.5, "Elastic easing should not be linear");
}
