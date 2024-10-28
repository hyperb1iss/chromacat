//! Integration tests for the ChromaCat application core functionality

use chromacat::cli::{Cli, PatternKind, PatternParameters};
use chromacat::ChromaCat;
use std::env;
use std::io::Write;
use tempfile::NamedTempFile;

// Helper function to create a temporary file with content
fn create_test_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{}", content).unwrap();
    file
}

fn setup_test_env() {
    env::set_var("RUST_TEST", "1");
    // Ensure we're using the default theme for tests
    env::set_var("NO_EXTERNAL_THEMES", "1");
}

#[test]
fn test_chromacat_basic() {
    setup_test_env();
    let test_file = create_test_file("Hello, ChromaCat!");

    let cli = Cli {
        files: vec![test_file.path().to_path_buf()],
        pattern: PatternKind::Horizontal,
        theme: String::from("rainbow"),
        animate: false,
        fps: 30,
        duration: 0,
        no_color: true,
        list_available: false,
        smooth: false,
        frequency: 1.0,
        amplitude: 1.0,
        speed: 1.0,
        pattern_params: PatternParameters { params: vec![] },
        theme_file: None,
        pattern_help: false,
        no_aspect_correction: false,
        aspect_ratio: 0.5,
    };

    let mut cat = ChromaCat::new(cli);
    match cat.run() {
        Ok(_) => (),
        Err(e) => panic!("Basic test failed with error: {:?}", e),
    }
}

#[test]
fn test_chromacat_invalid_angle() {
    setup_test_env();
    let test_file = create_test_file("Testing invalid angle");

    let mut pattern_params = PatternParameters::default();
    pattern_params.params.push("angle=400".to_string());

    let cli = Cli {
        files: vec![test_file.path().to_path_buf()],
        pattern: PatternKind::Diagonal,
        theme: String::from("rainbow"),
        animate: false,
        fps: 30,
        duration: 0,
        no_color: true,
        list_available: false,
        smooth: false,
        frequency: 1.0,
        amplitude: 1.0,
        speed: 1.0,
        pattern_params,
        theme_file: None,
        pattern_help: false,
        no_aspect_correction: false,
        aspect_ratio: 0.5,
    };

    let mut cat = ChromaCat::new(cli);
    assert!(cat.run().is_err());
}

#[test]
fn test_chromacat_pattern_params() {
    setup_test_env();
    let test_file = create_test_file("Testing pattern parameters");

    let test_cases = vec![
        (
            PatternKind::Plasma,
            vec![
                "complexity=3.0",
                "scale=1.5",
                "frequency=1.0",
                "blend_mode=add", // Changed from blend=add to blend_mode=add
            ],
        ),
        (
            PatternKind::Ripple,
            vec![
                "center_x=0.5",
                "center_y=0.5",
                "wavelength=1.0",
                "damping=0.5",
                "frequency=1.0",
            ],
        ),
        (
            PatternKind::Wave,
            vec![
                "amplitude=1.0",
                "frequency=2.0",
                "phase=0.0",
                "offset=0.5",
                "base_freq=1.0",
            ],
        ),
    ];

    for (pattern, params) in test_cases {
        let mut pattern_params = PatternParameters::default();
        pattern_params.params = params.iter().map(|s| s.to_string()).collect();

        let cli = Cli {
            files: vec![test_file.path().to_path_buf()],
            pattern,
            theme: String::from("rainbow"),
            animate: false,
            fps: 30,
            duration: 0,
            no_color: true,
            list_available: false,
            smooth: false,
            frequency: 1.0,
            amplitude: 1.0,
            speed: 1.0,
            pattern_params,
            theme_file: None,
            pattern_help: false,
            no_aspect_correction: false,
            aspect_ratio: 0.5,
        };

        let mut cat = ChromaCat::new(cli);
        match cat.run() {
            Ok(_) => (),
            Err(e) => panic!(
                "Failed with pattern {:?}: {:?}\nParameters: {:?}",
                pattern, e, params
            ),
        }
    }
}

#[test]
fn test_chromacat_animation_settings() {
    setup_test_env();
    let test_file = create_test_file("Testing animation");

    let cli = Cli {
        files: vec![test_file.path().to_path_buf()],
        pattern: PatternKind::Horizontal,
        theme: String::from("rainbow"),
        animate: true,
        fps: 60,
        duration: 5,
        no_color: false,
        list_available: false,
        smooth: true,
        frequency: 1.0,
        amplitude: 1.0,
        speed: 1.0,
        pattern_params: PatternParameters::default(),
        theme_file: None,
        pattern_help: false,
        no_aspect_correction: false,
        aspect_ratio: 0.5,
    };

    let mut cat = ChromaCat::new(cli);
    match cat.run() {
        Ok(_) => (),
        Err(e) => panic!("Animation test failed with error: {:?}", e),
    }
}
