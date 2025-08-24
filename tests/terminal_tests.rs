//! Tests for terminal state management

use chromacat::renderer::TerminalState;
use std::env;

/// Set up test environment variables
fn setup_test_env() {
    env::set_var("RUST_TEST", "1");
    env::set_var("TERM", "dumb");
}

#[test]
fn test_terminal_creation() {
    setup_test_env();
    let term_state = TerminalState::new();
    assert!(term_state.is_ok());
}

#[test]
fn test_terminal_size() {
    setup_test_env();
    let term_state = TerminalState::new().unwrap();
    let (width, height) = term_state.size();
    // Test against known default dimensions in test environment
    assert_eq!(width, 80);
    assert_eq!(height, 24);
}

#[test]
fn test_color_control() {
    setup_test_env();
    let mut term_state = TerminalState::new().unwrap();
    let initial = term_state.colors_enabled();
    term_state.set_colors_enabled(!initial);
    assert_eq!(term_state.colors_enabled(), !initial);
}

#[test]
fn test_cursor_visibility() {
    setup_test_env();
    let mut term_state = TerminalState::new().unwrap();

    // Test hide cursor
    assert!(term_state.hide_cursor().is_ok());

    // Test show cursor
    assert!(term_state.show_cursor().is_ok());
}

#[test]
fn test_alternate_screen() {
    setup_test_env();
    let mut term_state = TerminalState::new().unwrap();

    // Test entering alternate screen
    assert!(term_state.enter_alternate_screen().is_ok());

    // Cleanup should handle leaving alternate screen
    assert!(term_state.cleanup().is_ok());
}

#[test]
fn test_screen_clear() {
    setup_test_env();
    let mut term_state = TerminalState::new().unwrap();
    assert!(term_state.clear_screen().is_ok());
}

#[test]
fn test_resize_handling() {
    setup_test_env();
    let mut term_state = TerminalState::new().unwrap();

    // Test resize to new dimensions
    assert!(term_state.resize(100, 30).is_ok());

    let (width, height) = term_state.size();
    assert_eq!(width, 100);
    assert_eq!(height, 30);
}

#[test]
fn test_tty_detection() {
    setup_test_env();
    let term_state = TerminalState::new().unwrap();
    // In test environment, should not be a TTY
    assert!(!term_state.is_tty());
}

#[test]
fn test_cleanup_safety() {
    setup_test_env();
    let mut term_state = TerminalState::new().unwrap();

    // Multiple cleanup calls should be safe
    assert!(term_state.cleanup().is_ok());
    assert!(term_state.cleanup().is_ok());
}

#[test]
fn test_state_recovery() {
    setup_test_env();
    let mut term_state = TerminalState::new().unwrap();

    // Test recovery mechanism
    assert!(term_state.try_recover().is_ok());
}

#[test]
fn test_stdout_locking() {
    setup_test_env();
    let term_state = TerminalState::new().unwrap();

    // Should be able to get a stdout lock
    let _lock = term_state.stdout();
}

#[test]
fn test_flush() {
    setup_test_env();
    let term_state = TerminalState::new().unwrap();
    assert!(term_state.flush().is_ok());
}

// Integration-style tests

#[test]
fn test_full_terminal_lifecycle() {
    setup_test_env();
    let mut term_state = TerminalState::new().unwrap();

    // Setup
    assert!(term_state.setup().is_ok());

    // Enter alternate screen
    assert!(term_state.enter_alternate_screen().is_ok());

    // Simulate some operations
    assert!(term_state.hide_cursor().is_ok());
    assert!(term_state.clear_screen().is_ok());
    assert!(term_state.resize(120, 40).is_ok());

    // Cleanup
    assert!(term_state.cleanup().is_ok());
}

#[test]
fn test_color_output() {
    setup_test_env();
    let mut term_state = TerminalState::new().unwrap();

    // Test color output in both enabled and disabled states
    term_state.set_colors_enabled(true);
    assert!(term_state.colors_enabled());

    term_state.set_colors_enabled(false);
    assert!(!term_state.colors_enabled());
}

#[test]
fn test_error_conditions() {
    setup_test_env();
    let mut term_state = TerminalState::new().unwrap();

    // Test resize with invalid dimensions
    assert!(term_state.resize(0, 0).is_ok()); // Should handle gracefully in test env

    // Test recovery after error
    assert!(term_state.try_recover().is_ok());
}
