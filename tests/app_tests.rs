//! Integration tests for the ChromaCat application core functionality

use chromacat::cli::Cli;
use chromacat::ChromaCat;
use std::env;
use std::io::Write;
use std::sync::Once;
use tempfile::NamedTempFile;

static INIT: Once = Once::new();

// Helper function to create a temporary file with content
fn create_test_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{content}").unwrap();
    file
}

fn setup_test_env() {
    INIT.call_once(|| {
        env::set_var("RUST_TEST", "1");
        env::set_var("NO_EXTERNAL_THEMES", "1");
        env::set_var("TERM", "dumb");
        env::set_var("CI", "1");
    });
}

#[test]
fn test_chromacat_basic() {
    setup_test_env();
    let test_file = create_test_file("Hello, ChromaCat!");

    let cli = Cli {
        files: vec![test_file.path().to_path_buf()],
        pattern: "horizontal".to_string(),
        theme: String::from("rainbow"),
        no_color: true,
        list_available: false,
        frequency: 1.0,
        amplitude: 1.0,
        params: vec![],
        theme_file: None,
        pattern_help: false,
        no_aspect_correction: false,
        aspect_ratio: 0.5,
        buffer_size: None,
        playlist: None,
        art: None,
        list_art: false,
        no_playground: true,
        debug: false,
    };

    let mut cat = ChromaCat::new(cli);
    match cat.run() {
        Ok(_) => (),
        Err(e) => panic!("Basic test failed with error: {e:?}"),
    }
}

#[test]
fn test_chromacat_invalid_angle() {
    setup_test_env();
    let test_file = create_test_file("Testing invalid angle");

    let cli = Cli {
        files: vec![test_file.path().to_path_buf()],
        pattern: "diagonal".to_string(),
        theme: String::from("rainbow"),
        no_color: true,
        list_available: false,
        frequency: 1.0,
        amplitude: 1.0,
        params: vec!["angle=400".to_string()],
        theme_file: None,
        pattern_help: false,
        no_aspect_correction: false,
        aspect_ratio: 0.5,
        buffer_size: None,
        playlist: None,
        art: None,
        list_art: false,
        no_playground: true,
        debug: false,
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
            "plasma",
            vec![
                "complexity=3.0",
                "scale=1.5",
                "frequency=1.0",
                "blend_mode=add",
            ],
        ),
        (
            "ripple",
            vec![
                "center_x=0.5",
                "center_y=0.5",
                "wavelength=1.0",
                "damping=0.5",
                "frequency=1.0",
            ],
        ),
        (
            "wave",
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
        let cli = Cli {
            files: vec![test_file.path().to_path_buf()],
            pattern: pattern.to_string(),
            theme: String::from("rainbow"),
            no_color: true,
            list_available: false,
            frequency: 1.0,
            amplitude: 1.0,
            params: params.iter().map(|s| s.to_string()).collect(),
            theme_file: None,
            pattern_help: false,
            no_aspect_correction: false,
            aspect_ratio: 0.5,
            buffer_size: None,
            playlist: None,
            art: None,
            list_art: false,
            no_playground: true,
            debug: false,
        };

        let mut cat = ChromaCat::new(cli);
        match cat.run() {
            Ok(_) => (),
            Err(e) => panic!("Failed with pattern {pattern}: {e:?}\nParameters: {params:?}"),
        }
    }
}

#[test]
fn test_chromacat_animation_settings() {
    setup_test_env();
    let test_file = create_test_file("Testing animation");

    let cli = Cli {
        files: vec![test_file.path().to_path_buf()],
        pattern: "horizontal".to_string(),
        theme: String::from("rainbow"),
        no_color: false,
        list_available: false,
        frequency: 1.0,
        amplitude: 1.0,
        params: vec![],
        theme_file: None,
        pattern_help: false,
        no_aspect_correction: false,
        aspect_ratio: 0.5,
        buffer_size: None,
        playlist: None,
        art: None,
        list_art: false,
        no_playground: true,
        debug: false,
    };

    let mut cat = ChromaCat::new(cli);
    match cat.run() {
        Ok(_) => (),
        Err(e) => panic!("Animation test failed with error: {e:?}"),
    }
}

#[test]
fn test_streaming_mode() {
    setup_test_env();
    let test_input = "Test streaming input\n";
    let test_file = create_test_file(test_input);

    let cli = Cli {
        files: vec![test_file.path().to_path_buf()],
        pattern: "horizontal".to_string(),
        theme: String::from("rainbow"),
        no_color: true,
        list_available: false,
        frequency: 1.0,
        amplitude: 1.0,
        params: vec![],
        theme_file: None,
        pattern_help: false,
        no_aspect_correction: false,
        aspect_ratio: 0.5,
        buffer_size: Some(4096),
        playlist: None,
        art: None,
        list_art: false,
        no_playground: true,
        debug: false,
    };

    let mut cat = ChromaCat::new(cli);
    match cat.run() {
        Ok(_) => (),
        Err(e) => panic!("Streaming test failed with error: {e:?}"),
    }
}

#[test]
fn test_playground_mode_disabled_in_tests() {
    setup_test_env();
    println!("Testing that playground mode is disabled in test environment");

    // This should have playground mode disabled due to test environment
    let cli = Cli {
        files: vec![],
        pattern: "horizontal".to_string(),
        theme: String::from("rainbow"),
        no_color: true,
        list_available: false,
        frequency: 0.5,
        amplitude: 0.5,
        params: vec![],
        theme_file: None,
        pattern_help: false,
        no_aspect_correction: true,
        aspect_ratio: 1.0,
        buffer_size: Some(1024),
        playlist: None,
        art: Some("matrix".to_string()),
        list_art: false,
        no_playground: false, // Even though this is false, test env should disable playground
        debug: false,
    };

    let mut cat = ChromaCat::new(cli);

    // This should run quickly since playground mode is disabled in tests
    match cat.run() {
        Ok(_) => println!("Test completed successfully - playground was disabled"),
        Err(e) => {
            println!("Error details: {e:?}");
            panic!("Test failed with error: {e:?}")
        }
    }
}
