use chromacat::colorizer::Colorizer;
use chromacat::gradient::{GradientConfig, Theme};
use std::io::{self, Cursor};

#[test]
fn test_colorizer_basic() {
    let theme = Theme::Rainbow;
    let gradient = theme.create_gradient().unwrap();
    let config = GradientConfig {
        diagonal: false,
        angle: 45,
        cycle: false,
    };
    
    let mut colorizer = Colorizer::new(gradient, config, false);
    let input = Cursor::new("Test line\nAnother line");
    assert!(colorizer.colorize(input).is_ok());
}

#[test]
fn test_colorizer_empty_input() {
    let theme = Theme::Rainbow;
    let gradient = theme.create_gradient().unwrap();
    let config = GradientConfig {
        diagonal: false,
        angle: 45,
        cycle: false,
    };
    
    let mut colorizer = Colorizer::new(gradient, config, false);
    let input = Cursor::new("");
    assert!(colorizer.colorize(input).is_ok());
}

#[test]
fn test_colorizer_single_character() {
    let theme = Theme::Rainbow;
    let gradient = theme.create_gradient().unwrap();
    let config = GradientConfig {
        diagonal: false,
        angle: 45,
        cycle: false,
    };
    
    let mut colorizer = Colorizer::new(gradient, config, false);
    let input = Cursor::new("x");
    assert!(colorizer.colorize(input).is_ok());
}

#[test]
fn test_colorizer_unicode() {
    let theme = Theme::Rainbow;
    let gradient = theme.create_gradient().unwrap();
    let config = GradientConfig {
        diagonal: false,
        angle: 45,
        cycle: false,
    };
    
    let mut colorizer = Colorizer::new(gradient, config, false);
    let input = Cursor::new("Hello 世界\nこんにちは\n🌈🌟");
    assert!(colorizer.colorize(input).is_ok());
}

#[test]
fn test_colorizer_long_text() {
    let theme = Theme::Rainbow;
    let gradient = theme.create_gradient().unwrap();
    let config = GradientConfig {
        diagonal: false,
        angle: 45,
        cycle: false,
    };
    
    let mut colorizer = Colorizer::new(gradient, config, false);
    let long_text = "x".repeat(1000) + "\n" + &"y".repeat(1000);
    let input = Cursor::new(long_text);
    assert!(colorizer.colorize(input).is_ok());
}

#[test]
fn test_colorizer_diagonal() {
    let theme = Theme::Rainbow;
    let gradient = theme.create_gradient().unwrap();
    let config = GradientConfig {
        diagonal: true,
        angle: 45,
        cycle: false,
    };
    
    let mut colorizer = Colorizer::new(gradient, config, false);
    let input = Cursor::new("Line 1\nLine 2\nLine 3\nLine 4");
    assert!(colorizer.colorize(input).is_ok());
}

#[test]
fn test_colorizer_cycling() {
    let theme = Theme::Rainbow;
    let gradient = theme.create_gradient().unwrap();
    let config = GradientConfig {
        diagonal: false,
        angle: 45,
        cycle: true,
    };
    
    let mut colorizer = Colorizer::new(gradient, config, false);
    let input = Cursor::new("Test cycling gradient effect");
    assert!(colorizer.colorize(input).is_ok());
}

#[test]
fn test_colorizer_no_color() {
    let theme = Theme::Rainbow;
    let gradient = theme.create_gradient().unwrap();
    let config = GradientConfig {
        diagonal: false,
        angle: 45,
        cycle: false,
    };
    
    let mut colorizer = Colorizer::new(gradient, config, true);
    let input = Cursor::new("Test line\nAnother line");
    assert!(colorizer.colorize(input).is_ok());
}

#[test]
fn test_colorizer_all_themes() {
    let themes = vec![
        Theme::Rainbow,
        Theme::Heat,
        Theme::Ocean,
        Theme::Forest,
        Theme::Pastel,
        Theme::Neon,
        Theme::Autumn,
    ];

    let config = GradientConfig {
        diagonal: false,
        angle: 45,
        cycle: false,
    };

    for theme in themes {
        let gradient = theme.create_gradient().unwrap();
        let mut colorizer = Colorizer::new(gradient, config.clone(), false);
        let input = Cursor::new("Testing different themes");
        assert!(colorizer.colorize(input).is_ok());
    }
}

#[test]
fn test_colorizer_various_angles() {
    let theme = Theme::Rainbow;
    let gradient = theme.create_gradient().unwrap();
    let angles = vec![0, 45, 90, 180, 270, 360];

    for angle in angles {
        let config = GradientConfig {
            diagonal: true,
            angle,
            cycle: false,
        };
        let mut colorizer = Colorizer::new(gradient.clone(), config, false);
        let input = Cursor::new("Line 1\nLine 2\nLine 3");
        assert!(colorizer.colorize(input).is_ok());
    }
}

#[test]
fn test_colorizer_whitespace() {
    let theme = Theme::Rainbow;
    let gradient = theme.create_gradient().unwrap();
    let config = GradientConfig {
        diagonal: false,
        angle: 45,
        cycle: false,
    };
    
    let mut colorizer = Colorizer::new(gradient, config, false);
    let input = Cursor::new("  Leading spaces\nTrailing spaces  \n\tTabs\t\n\n\nMultiple empty lines");
    assert!(colorizer.colorize(input).is_ok());
}

#[test]
fn test_colorizer_special_characters() {
    let theme = Theme::Rainbow;
    let gradient = theme.create_gradient().unwrap();
    let config = GradientConfig {
        diagonal: false,
        angle: 45,
        cycle: false,
    };
    
    let mut colorizer = Colorizer::new(gradient, config, false);
    let input = Cursor::new("Special chars: !@#$%^&*()_+-=[]{}|;:'\",.<>?/\\");
    assert!(colorizer.colorize(input).is_ok());
}