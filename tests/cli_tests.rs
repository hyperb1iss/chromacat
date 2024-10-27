use chromacat::cli::{Cli, PatternKind};
use clap::Parser;
use std::path::PathBuf;

#[test]
fn test_basic_cli() {
    let args = vec!["chromacat", "input.txt"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert_eq!(cli.files, vec![PathBuf::from("input.txt")]);
    assert_eq!(cli.pattern, PatternKind::Horizontal);
    assert_eq!(cli.theme, "rainbow");
    assert!(!cli.animate);
}

#[test]
fn test_pattern_flags() {
    let pattern_tests = vec![
        (
            vec!["chromacat", "-p", "horizontal"],
            PatternKind::Horizontal,
        ),
        (vec!["chromacat", "-p", "diagonal"], PatternKind::Diagonal),
        (vec!["chromacat", "-p", "plasma"], PatternKind::Plasma),
        (vec!["chromacat", "-p", "ripple"], PatternKind::Ripple),
        (vec!["chromacat", "-p", "wave"], PatternKind::Wave),
        (vec!["chromacat", "-p", "spiral"], PatternKind::Spiral),
        (
            vec!["chromacat", "-p", "checkerboard"],
            PatternKind::Checkerboard,
        ),
        (vec!["chromacat", "-p", "diamond"], PatternKind::Diamond),
        (vec!["chromacat", "-p", "perlin"], PatternKind::Perlin),
    ];

    for (args, expected) in pattern_tests {
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.pattern, expected);
    }
}

#[test]
fn test_pattern_specific_args() {
    // Test diagonal pattern with angle
    let args = vec![
        "chromacat", 
        "-p", "diagonal", 
        "--param", "angle=45"
    ];
    let cli = Cli::try_parse_from(args).unwrap();
    assert_eq!(cli.pattern, PatternKind::Diagonal);
    assert!(cli.pattern_params.params.contains(&"angle=45".to_string()));

    // Test plasma pattern with complexity and scale
    let args = vec![
        "chromacat",
        "-p", "plasma",
        "--param", "complexity=3.0",
        "--param", "scale=1.5",
    ];
    let cli = Cli::try_parse_from(args).unwrap();
    assert_eq!(cli.pattern, PatternKind::Plasma);
    assert!(cli.pattern_params.params.contains(&"complexity=3.0".to_string()));
    assert!(cli.pattern_params.params.contains(&"scale=1.5".to_string()));
}

#[test]
fn test_animation_settings() {
    let args = vec![
        "chromacat",
        "--animate",
        "--fps",
        "60",
        "--duration",
        "5",
        "input.txt",
    ];
    let cli = Cli::try_parse_from(args).unwrap();
    assert!(cli.animate);
    assert_eq!(cli.fps, 60);
    assert_eq!(cli.duration, 5);
}

#[test]
fn test_invalid_fps() {
    let args = vec!["chromacat", "--fps", "200"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert!(cli.validate().is_err());
}

#[test]
fn test_theme_selection() {
    let args = vec!["chromacat", "-t", "ocean"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert_eq!(cli.theme, "ocean");
}

#[test]
fn test_multiple_files() {
    let args = vec!["chromacat", "file1.txt", "file2.txt", "file3.txt"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert_eq!(
        cli.files,
        vec![
            PathBuf::from("file1.txt"),
            PathBuf::from("file2.txt"),
            PathBuf::from("file3.txt")
        ]
    );
}

#[test]
fn test_animation_defaults() {
    let args = vec!["chromacat", "--animate"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert_eq!(cli.fps, 30); // Default FPS
    assert_eq!(cli.duration, 0); // Infinite duration by default
}

#[test]
fn test_pattern_validation() {
    // Test valid parameter ranges
    let args = vec![
        "chromacat",
        "-p", "wave",
        "--param", "amplitude=1.0",
        "--param", "frequency=2.0",
    ];
    let cli = Cli::try_parse_from(args).unwrap();
    assert!(cli.validate().is_ok());

    // Test invalid parameter ranges
    let args = vec![
        "chromacat",
        "-p", "wave",
        "--param", "amplitude=20.0", // Too high
    ];
    let cli = Cli::try_parse_from(args).unwrap();
    assert!(cli.validate().is_err());
}
