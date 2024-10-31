use chromacat::cli::Cli;
use clap::Parser;
use std::path::PathBuf;

#[test]
fn test_basic_cli() {
    let args = vec!["chromacat", "input.txt"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert_eq!(cli.files, vec![PathBuf::from("input.txt")]);
    assert_eq!(cli.pattern, "diagonal");
    assert_eq!(cli.theme, "rainbow");
    assert!(!cli.animate);
}

#[test]
fn test_pattern_flags() {
    let pattern_tests = vec![
        (vec!["chromacat", "-p", "horizontal"], "horizontal"),
        (vec!["chromacat", "-p", "diagonal"], "diagonal"),
        (vec!["chromacat", "-p", "plasma"], "plasma"),
        (vec!["chromacat", "-p", "ripple"], "ripple"),
        (vec!["chromacat", "-p", "wave"], "wave"),
        (vec!["chromacat", "-p", "spiral"], "spiral"),
        (vec!["chromacat", "-p", "checkerboard"], "checkerboard"),
        (vec!["chromacat", "-p", "diamond"], "diamond"),
        (vec!["chromacat", "-p", "perlin"], "perlin"),
    ];

    for (args, expected) in pattern_tests {
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.pattern, expected);
    }
}

#[test]
fn test_pattern_specific_args() {
    // Test diagonal pattern with angle
    let args = vec!["chromacat", "-p", "diagonal", "--param", "angle=45"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert_eq!(cli.pattern, "diagonal");
    assert!(cli.params.contains(&"angle=45".to_string()));

    // Test plasma pattern with complexity and scale
    let args = vec![
        "chromacat",
        "-p",
        "plasma",
        "--param",
        "complexity=3.0",
        "--param",
        "scale=1.5",
    ];
    let cli = Cli::try_parse_from(args).unwrap();
    assert_eq!(cli.pattern, "plasma");
    assert!(cli.params.contains(&"complexity=3.0".to_string()));
    assert!(cli.params.contains(&"scale=1.5".to_string()));
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
    // Test valid parameters
    let args = vec![
        "chromacat",
        "-p",
        "wave",
        "--param",
        "amplitude=1.0",
        "--param",
        "frequency=2.0",
    ];
    let cli = Cli::try_parse_from(args).unwrap();
    assert!(cli.validate().is_ok());

    // Test invalid parameter value
    let args = vec![
        "chromacat",
        "-p",
        "wave",
        "--param",
        "amplitude=20.0", // Invalid value
    ];
    let cli = Cli::try_parse_from(args).unwrap();
    assert!(cli.validate().is_err());

    // Test invalid parameter name
    let args = vec!["chromacat", "-p", "wave", "--param", "invalid_param=1.0"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert!(cli.validate().is_err());
}

#[test]
fn test_comma_separated_params() {
    let test_cases = vec![
        // Single param with comma-separated values
        (
            vec![
                "chromacat",
                "-p",
                "wave",
                "--param",
                "amplitude=1.0,frequency=2.0",
            ],
            vec!["amplitude=1.0", "frequency=2.0"],
        ),
        // Multiple params, one with comma-separated values
        (
            vec![
                "chromacat",
                "-p",
                "plasma",
                "--param",
                "complexity=3.0,scale=1.5",
                "--param",
                "blend_mode=add",
            ],
            vec!["complexity=3.0", "scale=1.5", "blend_mode=add"],
        ),
    ];

    for (args, expected_params) in test_cases {
        let cli = Cli::try_parse_from(args).unwrap();
        let actual_params: Vec<String> = cli.params.iter()
            .flat_map(|p| p.split(','))
            .map(|s| s.trim().to_string())
            .collect();

        assert_eq!(
            actual_params, expected_params,
            "Failed to parse comma-separated parameters correctly"
        );
    }
}

#[test]
fn test_aspect_ratio_settings() {
    let args = vec!["chromacat", "--no-aspect-correction", "--aspect-ratio", "0.7"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert!(cli.no_aspect_correction);
    assert_eq!(cli.aspect_ratio, 0.7);
}

#[test]
fn test_invalid_aspect_ratio() {
    let args = vec!["chromacat", "--aspect-ratio", "2.5"]; // Too large
    let cli = Cli::try_parse_from(args).unwrap();
    assert!(cli.validate().is_err());
}
