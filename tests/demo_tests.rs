//! Tests for ChromaCat's demo functionality

use chromacat::demo::{self, ArtSettings};

#[test]
fn test_terminal_size_validation() {
    assert!(demo::check_terminal_size(80, 24).is_ok());
    assert!(demo::check_terminal_size(39, 24).is_err());
    assert!(demo::check_terminal_size(80, 9).is_err());
}

#[test]
fn test_art_pattern_parsing() {
    assert!(demo::parse_art("matrix").is_ok());
    assert!(demo::parse_art("invalid").is_err());
}

#[test]
fn test_art_settings() {
    let settings = ArtSettings::new(80, 24).with_headers(true).with_seed(42);

    assert_eq!(settings.width, 80);
    assert_eq!(settings.height, 24);
    assert!(settings.include_headers);
    assert_eq!(settings.seed, 42);
}
