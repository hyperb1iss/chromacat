use chromacat::gradient::{GradientConfig, GradientEngine, Theme};
use std::str::FromStr;

#[test]
fn test_theme_from_str() {
    assert!(matches!(Theme::from_str("rainbow"), Ok(Theme::Rainbow)));
    assert!(matches!(Theme::from_str("heat"), Ok(Theme::Heat)));
    assert!(Theme::from_str("invalid").is_err());
}

#[test]
fn test_gradient_creation() {
    let theme = Theme::Rainbow;
    assert!(theme.create_gradient().is_ok());
}

#[test]
fn test_horizontal_gradient() {
    let theme = Theme::Rainbow;
    let gradient = theme.create_gradient().unwrap();
    let config = GradientConfig {
        diagonal: false,
        angle: 45,
        cycle: false,
    };
    let engine = GradientEngine::new(gradient, config);

    let color_start = engine.get_color_at(0, 10).unwrap();
    let color_end = engine.get_color_at(9, 10).unwrap();
    assert_ne!(color_start, color_end);
}

#[test]
fn test_diagonal_gradient() {
    let theme = Theme::Rainbow;
    let gradient = theme.create_gradient().unwrap();
    let config = GradientConfig {
        diagonal: true,
        angle: 45,
        cycle: false,
    };
    let mut engine = GradientEngine::new(gradient, config);
    
    engine.set_total_lines(10);
    engine.set_current_line(5);

    let color1 = engine.get_color_at(0, 10).unwrap();
    let color2 = engine.get_color_at(9, 10).unwrap();
    assert_ne!(color1, color2);
}