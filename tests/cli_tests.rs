use chromacat::cli::{Cli, PatternKind};
use clap::Parser;
use std::path::PathBuf;

#[test]
fn test_basic_cli() {
    let args = vec!["chromacat", "input.txt"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert_eq!(cli.files, vec![PathBuf::from("input.txt")]);
    assert_eq!(cli.pattern, PatternKind::Diagonal);
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
    let args = vec!["chromacat", "-p", "diagonal", "--param", "angle=45"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert_eq!(cli.pattern, PatternKind::Diagonal);
    assert!(cli.pattern_params.params.contains(&"angle=45".to_string()));

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
    assert_eq!(cli.pattern, PatternKind::Plasma);
    assert!(cli
        .pattern_params
        .params
        .contains(&"complexity=3.0".to_string()));
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
    eprintln!("DEBUG: Testing valid parameters: {:?}", args);
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
    eprintln!("DEBUG: Testing invalid parameter value: {:?}", args);
    let cli = Cli::try_parse_from(args).unwrap();
    eprintln!("DEBUG: Validating CLI...");
    assert!(cli.validate().is_err());

    // Test invalid parameter name
    let args = vec!["chromacat", "-p", "wave", "--param", "invalid_param=1.0"];
    eprintln!("DEBUG: Testing invalid parameter name: {:?}", args);
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
        // Multiple comma-separated params
        (
            vec![
                "chromacat",
                "-p",
                "ripple",
                "--param",
                "center_x=0.5,center_y=0.5",
                "--param",
                "wavelength=1.0,damping=0.5",
            ],
            vec![
                "center_x=0.5",
                "center_y=0.5",
                "wavelength=1.0",
                "damping=0.5",
            ],
        ),
    ];

    for (args, expected_params) in test_cases {
        let cli = Cli::try_parse_from(args).unwrap();

        // Collect all parameters into a flattened vec for comparison
        let actual_params: Vec<String> = cli
            .pattern_params
            .params
            .iter()
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
fn test_invalid_param_format() {
    let invalid_cases = vec![
        vec!["chromacat", "--param", "invalid"],
        vec!["chromacat", "--param", "key=value=extra"],
        vec!["chromacat", "--param", "key=,value=1.0"],
        vec!["chromacat", "--param", ",key=value"],
        vec!["chromacat", "--param", "=value"],
        vec!["chromacat", "--param", "key="],
    ];

    for args in &invalid_cases {
        // Borrow the vector instead of moving it
        assert!(
            Cli::try_parse_from(args).is_err(),
            "Parser should reject invalid parameter format: {:?}",
            args
        );
    }
}

#[test]
fn test_pattern_config_creation() {
    let args = vec![
        "chromacat",
        "-p",
        "wave",
        "--param",
        "amplitude=1.5,frequency=2.0",
        "--param",
        "phase=0.5",
    ];

    let cli = Cli::try_parse_from(args).unwrap();
    let config = cli.create_pattern_config().unwrap();

    // The exact assertions will depend on your PatternConfig structure
    // but you should verify that all parameters were correctly parsed
    if let chromacat::pattern::PatternParams::Wave(params) = config.params {
        assert_eq!(params.amplitude, 1.5);
        assert_eq!(params.frequency, 2.0);
        assert_eq!(params.phase, 0.5);
    } else {
        panic!("Expected Wave pattern parameters");
    }
}

#[test]
fn test_mixed_param_formats() {
    let args = vec![
        "chromacat",
        "-p",
        "plasma",
        "--param",
        "complexity=3.0",
        "--param",
        "scale=1.5,blend_mode=add",
        "--param",
        "frequency=1.0",
    ];

    let cli = Cli::try_parse_from(args).unwrap();
    let expected_params = vec![
        "complexity=3.0",
        "scale=1.5",
        "blend_mode=add",
        "frequency=1.0",
    ];

    let actual_params: Vec<String> = cli
        .pattern_params
        .params
        .iter()
        .flat_map(|p| p.split(','))
        .map(|s| s.trim().to_string())
        .collect();

    assert_eq!(
        actual_params, expected_params,
        "Failed to handle mixed parameter formats correctly"
    );
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
