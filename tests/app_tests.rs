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
        theme: String::from("rainbow"), // Use default theme
        animate: false,
        fps: 30,
        duration: 0,
        no_color: true,
        list_available: false,
        smooth: false,
        frequency: 1.0,
        amplitude: 1.0,
        speed: 1.0,
        pattern_params: PatternParameters::default(),
        theme_file: None, // Added missing field
    };

    let mut cat = ChromaCat::new(cli);
    assert!(cat.run().is_ok());
}

#[test]
fn test_chromacat_invalid_angle() {
    setup_test_env();
    // Create a temporary file with test content
    let test_file = create_test_file("Testing invalid angle");

    let mut pattern_params = PatternParameters::default();
    pattern_params.angle = Some(400); // Invalid angle

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
        theme_file: None, // Added missing field
    };

    let mut cat = ChromaCat::new(cli);
    assert!(cat.run().is_err());
}

#[test]
fn test_chromacat_invalid_theme() {
    setup_test_env();
    // Create a temporary file with test content
    let test_file = create_test_file("Testing invalid theme");

    let cli = Cli {
        files: vec![test_file.path().to_path_buf()],
        pattern: PatternKind::Horizontal,
        theme: String::from("nonexistent"),
        animate: false,
        fps: 30,
        duration: 0,
        no_color: true,
        list_available: false,
        smooth: false,
        frequency: 1.0,
        amplitude: 1.0,
        speed: 1.0,
        pattern_params: PatternParameters::default(),
        theme_file: None, // Added missing field
    };

    let mut cat = ChromaCat::new(cli);
    // The invalid theme should return an error, not fall back to rainbow
    assert!(cat.run().is_err());
}

#[test]
fn test_chromacat_pattern_params() {
    setup_test_env();
    // Create a temporary file with test content
    let test_file = create_test_file("Testing pattern parameters");

    // Test various pattern parameters
    let pattern_tests = vec![
        (
            PatternKind::Plasma,
            PatternParameters {
                complexity: Some(3.0),
                scale: Some(1.5),
                ..Default::default()
            },
        ),
        (
            PatternKind::Ripple,
            PatternParameters {
                center_x: Some(0.5),
                center_y: Some(0.5),
                wavelength: Some(1.0),
                damping: Some(0.5),
                ..Default::default()
            },
        ),
        (
            PatternKind::Wave,
            PatternParameters {
                height: Some(1.0),
                count: Some(2.0),
                phase: Some(0.0),
                offset: Some(0.5),
                ..Default::default()
            },
        ),
    ];

    for (pattern, params) in pattern_tests {
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
            pattern_params: params,
            theme_file: None, // Added missing field
        };

        let mut cat = ChromaCat::new(cli);
        assert!(cat.run().is_ok());
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
        theme_file: None, // Added missing field
    };

    let mut cat = ChromaCat::new(cli);
    assert!(cat.run().is_ok());
}
