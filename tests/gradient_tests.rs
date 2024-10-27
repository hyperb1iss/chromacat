use chromacat::gradient::{GradientConfig, GradientEngine};
use chromacat::themes;

#[test]
fn test_gradient_creation() {
    let config = GradientConfig::default();
    let theme = themes::get_theme("rainbow").unwrap();
    let gradient = theme.create_gradient().unwrap();
    let engine = GradientEngine::new(gradient, config);
    assert!(engine.get_color_at(0, 10).is_ok());
}

#[test]
fn test_horizontal_gradient() {
    let config = GradientConfig {
        diagonal: false,
        angle: 45,
        cycle: false,
    };
    let theme = themes::get_theme("rainbow").unwrap();
    let gradient = theme.create_gradient().unwrap();
    let engine = GradientEngine::new(gradient, config);

    // Test start position
    let start_color = engine.get_color_at(0, 10).unwrap();
    let end_color = engine.get_color_at(9, 10).unwrap();
    assert_ne!(start_color, end_color);
}

#[test]
fn test_diagonal_gradient() {
    let config = GradientConfig {
        diagonal: true,
        angle: 45,
        cycle: false,
    };
    let theme = themes::get_theme("rainbow").unwrap();
    let gradient = theme.create_gradient().unwrap();
    let mut engine = GradientEngine::new(gradient, config);

    engine.set_total_lines(10);
    engine.set_current_line(5);

    // Test color variation
    let color1 = engine.get_color_at(0, 10).unwrap();
    let color2 = engine.get_color_at(9, 10).unwrap();
    assert_ne!(color1, color2);
}

#[test]
fn test_all_themes_gradient_creation() {
    let config = GradientConfig::default();

    for theme_name in themes::list_categories()
        .iter()
        .flat_map(|cat| themes::list_category(cat).unwrap())
    {
        let theme = themes::get_theme(theme_name).unwrap();
        let gradient = theme.create_gradient().unwrap();
        let engine = GradientEngine::new(gradient, config.clone());
        assert!(engine.get_color_at(0, 10).is_ok());
    }
}

#[test]
fn test_gradient_cycling() {
    let config = GradientConfig {
        diagonal: false,
        angle: 45,
        cycle: true,
    };
    let theme = themes::get_theme("rainbow").unwrap();
    let gradient = theme.create_gradient().unwrap();
    let engine = GradientEngine::new(gradient, config);

    let color1 = engine.get_color_at(0, 100).unwrap();
    let color2 = engine.get_color_at(25, 100).unwrap();
    let color3 = engine.get_color_at(50, 100).unwrap();
    let color4 = engine.get_color_at(75, 100).unwrap();

    // Colors should vary due to cycling
    assert_ne!(color1, color2);
    assert_ne!(color2, color3);
    assert_ne!(color3, color4);
}
