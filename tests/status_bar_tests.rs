use chromacat::renderer::StatusBar;


#[test]
fn test_status_bar_creation() {
    let status_bar = StatusBar::new((80, 24));
    assert_eq!(status_bar.width(), 80);
    assert_eq!(status_bar.height(), 24);
}

#[test]
fn test_theme_pattern_update() {
    let mut status_bar = StatusBar::new((80, 24));
    status_bar.set_theme("ocean");
    assert_eq!(status_bar.current_theme(), "ocean");

    status_bar.set_pattern("wave");
    assert_eq!(status_bar.current_pattern(), "wave");
}

#[test]
fn test_fps_update() {
    let mut status_bar = StatusBar::new((80, 24));

    // Test FPS update with significant change
    status_bar.set_fps(60.0);
    assert_eq!(status_bar.fps(), 60.0);

    // Test FPS update with minor change (should not update)
    status_bar.set_fps(60.2);
    assert_eq!(status_bar.fps(), 60.0);

    // Test FPS display toggle
    status_bar.show_fps(false);
    assert!(!status_bar.is_fps_shown());
}

#[test]
fn test_resize() {
    let mut status_bar = StatusBar::new((80, 24));
    status_bar.resize((100, 30));
    assert_eq!(status_bar.width(), 100);
    assert_eq!(status_bar.height(), 30);
}

#[test]
fn test_custom_text() {
    let mut status_bar = StatusBar::new((80, 24));

    // Test setting custom text
    status_bar.set_custom_text(Some("Playing: Cool Song"));
    assert_eq!(status_bar.custom_text(), Some("Playing: Cool Song"));

    // Test clearing custom text
    status_bar.set_custom_text(None);
    assert_eq!(status_bar.custom_text(), None);
}
