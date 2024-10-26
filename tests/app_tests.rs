use chromacat::cli::Cli;
use chromacat::ChromaCat;
use std::path::PathBuf;

#[test]
fn test_chromacat_basic() {
    let cli = Cli {
        theme: String::from("rainbow"),
        cycle: false,
        input: None,
        no_color: true,
        diagonal: false,
        angle: 45,
    };

    let cat = ChromaCat::new(cli);
    assert!(cat.run().is_ok());
}

#[test]
fn test_chromacat_invalid_angle() {
    let cli = Cli {
        theme: String::from("rainbow"),
        cycle: false,
        input: None,
        no_color: true,
        diagonal: true,
        angle: 400, // Invalid angle
    };

    let cat = ChromaCat::new(cli);
    assert!(cat.run().is_err());
}

#[test]
fn test_chromacat_invalid_theme() {
    let cli = Cli {
        theme: String::from("nonexistent"),
        cycle: false,
        input: None,
        no_color: true,
        diagonal: false,
        angle: 45,
    };

    let cat = ChromaCat::new(cli);
    assert!(cat.run().is_ok()); // Should fall back to rainbow theme
}