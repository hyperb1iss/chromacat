use std::str::FromStr;
use std::time::Duration;

use chromacat::playlist::{Playlist, PlaylistPlayer};

#[test]
fn test_playlist_loading() {
    let yaml = r#"
entries:
  - name: Test Entry
    pattern: plasma
    theme: rainbow
    duration: 30
    params:
      complexity: 3.0
      scale: 1.5
      frequency: 1.0
      blend_mode: add
"#;

    let playlist = Playlist::from_str(yaml).unwrap();
    assert_eq!(playlist.entries.len(), 1);

    let entry = &playlist.entries[0];
    assert_eq!(entry.name, "Test Entry");
    assert_eq!(entry.pattern, "plasma");
    assert_eq!(entry.theme, "rainbow");
    assert_eq!(entry.duration, 30);
}

#[test]
fn test_playlist_validation() {
    // Test invalid pattern
    let yaml = r#"
entries:
  - pattern: invalid_pattern
    theme: rainbow
    duration: 30
"#;
    assert!(Playlist::from_str(yaml).is_err());

    // Test invalid theme
    let yaml = r#"
entries:
  - pattern: plasma
    theme: invalid_theme
    duration: 30
"#;
    assert!(Playlist::from_str(yaml).is_err());

    // Test invalid parameters
    let yaml = r#"
entries:
  - pattern: plasma
    theme: rainbow
    duration: 30
    params:
      invalid_param: 1.0
"#;
    assert!(Playlist::from_str(yaml).is_err());

    // Test valid parameters
    let yaml = r#"
entries:
  - pattern: plasma
    theme: rainbow
    duration: 30
    params:
      complexity: 3.0
      scale: 1.5
      frequency: 1.0
      blend_mode: add
"#;
    assert!(Playlist::from_str(yaml).is_ok());
}

#[test]
fn test_playlist_player() {
    let yaml = r#"
entries:
  - pattern: plasma
    theme: rainbow
    duration: 30
  - pattern: wave
    theme: ocean
    duration: 20
"#;

    let playlist = Playlist::from_str(yaml).unwrap();
    let mut player = PlaylistPlayer::new(playlist);

    // Test initial state
    assert_eq!(player.current_index(), 0);
    assert_eq!(player.current_entry().unwrap().pattern, "plasma");

    // Test update within duration
    assert!(!player.update(Duration::from_secs(15)));
    assert_eq!(player.current_index(), 0);
    assert!(player.current_progress() > 0.0 && player.current_progress() < 1.0);

    // Test transition to next entry
    assert!(player.update(Duration::from_secs(20)));
    assert_eq!(player.current_index(), 1);
    assert_eq!(player.current_entry().unwrap().pattern, "wave");

    // Test wrapping around
    assert!(player.update(Duration::from_secs(30)));
    assert_eq!(player.current_index(), 0);
}

#[test]
fn test_playlist_player_controls() {
    let yaml = r#"
entries:
  - pattern: plasma
    theme: rainbow
    duration: 30
  - pattern: wave
    theme: ocean
    duration: 20
  - pattern: fire
    theme: heat
    duration: 25
"#;

    let playlist = Playlist::from_str(yaml).unwrap();
    let mut player = PlaylistPlayer::new(playlist);

    // Test pause/resume
    assert!(!player.is_paused());
    player.toggle_pause();
    assert!(player.is_paused());
    assert!(!player.update(Duration::from_secs(60))); // Should not change while paused
    player.toggle_pause();
    assert!(!player.is_paused());

    // Test manual navigation
    assert_eq!(player.current_index(), 0);
    player.next_entry();
    assert_eq!(player.current_index(), 1);
    player.next_entry();
    assert_eq!(player.current_index(), 2);
    player.previous_entry();
    assert_eq!(player.current_index(), 1);
}
