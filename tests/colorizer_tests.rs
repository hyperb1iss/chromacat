use chromacat::colorizer::Colorizer;
use chromacat::gradient::GradientConfig;
use chromacat::themes;
use std::io::Cursor;

#[test]
fn test_colorizer_basic() {
    let theme = themes::get_theme("rainbow").unwrap();
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
    let theme = themes::get_theme("rainbow").unwrap();
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
fn test_colorizer_unicode() {
    let theme = themes::get_theme("rainbow").unwrap();
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
    let theme = themes::get_theme("rainbow").unwrap();
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
fn test_all_themes_colorization() {
    let config = GradientConfig::default();

    for theme_name in themes::list_categories()
        .iter()
        .flat_map(|cat| themes::list_category(cat).unwrap())
    {
        let theme = themes::get_theme(theme_name).unwrap();
        let gradient = theme.create_gradient().unwrap();
        let mut colorizer = Colorizer::new(gradient, config.clone(), false);
        let input = Cursor::new("Testing different themes");
        assert!(colorizer.colorize(input).is_ok());
    }
}

#[test]
fn test_colorizer_diagonal() {
    let theme = themes::get_theme("rainbow").unwrap();
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
    let theme = themes::get_theme("rainbow").unwrap();
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
    let theme = themes::get_theme("rainbow").unwrap();
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
fn test_colorizer_whitespace() {
    let theme = themes::get_theme("rainbow").unwrap();
    let gradient = theme.create_gradient().unwrap();
    let config = GradientConfig {
        diagonal: false,
        angle: 45,
        cycle: false,
    };

    let mut colorizer = Colorizer::new(gradient, config, false);
    let input =
        Cursor::new("  Leading spaces\nTrailing spaces  \n\tTabs\t\n\n\nMultiple empty lines");
    assert!(colorizer.colorize(input).is_ok());
}

#[test]
fn test_colorizer_special_characters() {
    let theme = themes::get_theme("rainbow").unwrap();
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
