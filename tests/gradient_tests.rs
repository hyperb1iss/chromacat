use chromacat::gradient::{GradientConfig, GradientEngine};
use chromacat::themes::Theme;

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

#[test]
fn test_all_themes_gradient_creation() {
    let themes = vec![
        Theme::Rainbow,
        Theme::Heat,
        Theme::Ocean,
        Theme::Forest,
        Theme::Pastel,
        Theme::Neon,
        Theme::Autumn,
        Theme::Matrix,
        Theme::Cyberpunk,
        Theme::Nebula,
        Theme::Galaxy,
        Theme::Retrowave,
        Theme::Vaporwave,
        // Add more themes to test...
    ];

    for theme in themes {
        assert!(theme.create_gradient().is_ok());
    }
}

#[test]
fn test_gradient_cycling() {
    let theme = Theme::Rainbow;
    let gradient = theme.create_gradient().unwrap();
    let config = GradientConfig {
        diagonal: false,
        angle: 45,
        cycle: true,
    };
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