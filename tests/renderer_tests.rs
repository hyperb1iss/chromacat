//! Tests for the rendering system

use chromacat::pattern::{CommonParams, PatternConfig, PatternEngine, PatternParams};
use chromacat::renderer::{AnimationConfig, Renderer};
use std::time::Duration;

/// Test fixture for renderer tests
struct RendererTest {
    engine: PatternEngine,
    config: AnimationConfig,
}

impl RendererTest {
    fn new() -> Self {
        let pattern_config = PatternConfig {
            common: CommonParams::default(),
            params: PatternParams::Horizontal,
        };

        let engine = PatternEngine::new(pattern_config, 80, 24);

        let config = AnimationConfig {
            fps: 30,
            cycle_duration: Duration::from_secs(1),
            infinite: false,
            show_progress: true,
            smooth: false,
        };

        Self { engine, config }
    }

    fn create_renderer(&self) -> Result<Renderer, Box<dyn std::error::Error>> {
        let renderer = Renderer::new(self.engine.clone(), self.config.clone())?;
        Ok(renderer)
    }
}

#[test]
fn test_renderer_creation() {
    let test = RendererTest::new();
    assert!(test.create_renderer().is_ok());
}

#[test]
fn test_static_rendering() {
    let test = RendererTest::new();
    let mut renderer = test.create_renderer().unwrap();

    assert!(renderer.render_static("Hello, World!").is_ok());
}

#[test]
fn test_animated_rendering() {
    let test = RendererTest::new();
    let mut renderer = test.create_renderer().unwrap();

    assert!(renderer
        .render_frame("Test", Duration::from_millis(100))
        .is_ok());
}

#[test]
fn test_text_handling() {
    let test_cases = vec![
        ("Simple text", "ascii only"),
        ("Hello, 世界!", "unicode text"),
        ("👋 🌟 🎨", "emojis"),
        ("Multi\nline\ntext", "multiline"),
    ];

    let test = RendererTest::new();
    let mut renderer = test.create_renderer().unwrap();

    for (text, case) in test_cases {
        assert!(
            renderer.render_static(text).is_ok(),
            "Failed to render {}: {}",
            case,
            text
        );
    }
}

#[test]
fn test_animation_timing() {
    let test = RendererTest::new();
    let renderer = test.create_renderer().unwrap();

    assert_eq!(renderer.frame_duration(), Duration::from_secs(1) / 30);
}

#[test]
fn test_unicode_width() {
    let test = RendererTest::new();
    let mut renderer = test.create_renderer().unwrap();

    // Text with mixed-width characters
    let text = "Hello, 世界! 👋";
    assert!(renderer.render_static(text).is_ok());
}

#[test]
fn test_color_disabled() {
    let mut test = RendererTest::new();
    test.config.show_progress = false;

    let mut renderer = test.create_renderer().unwrap();
    assert!(renderer.render_static("Test").is_ok());
}

#[test]
fn test_progress_bar() {
    let test = RendererTest::new();
    let mut renderer = test.create_renderer().unwrap();

    // Test different progress values
    let progress_values = [0.0, 0.25, 0.5, 0.75, 1.0];
    for &progress in &progress_values {
        assert!(
            renderer
                .render_frame("Test", Duration::from_secs_f64(progress))
                .is_ok(),
            "Failed to render progress bar at {}%",
            progress * 100.0
        );
    }
}

/// Test terminal interaction
mod terminal_tests {
    use super::*;

    #[test]
    fn test_terminal_cleanup() {
        let test = RendererTest::new();
        {
            let _renderer = test.create_renderer().unwrap();
            // Renderer drops here, should clean up terminal state
        }
        // Test passes if no panic occurs during cleanup
    }

    #[test]
    fn test_terminal_resize_handling() {
        let test = RendererTest::new();
        let mut renderer = test.create_renderer().unwrap();

        // Simulate terminal resize by rendering before and after
        assert!(renderer.render_static("Before resize").is_ok());
        // Resize event would happen here in real terminal
        assert!(renderer.render_static("After resize").is_ok());
    }
}

/// Error handling tests
mod error_tests {
    use super::*;

    #[test]
    fn test_oversized_content() {
        let test = RendererTest::new();
        let mut renderer = test.create_renderer().unwrap();

        // Create text larger than terminal dimensions
        let oversized_text = "X".repeat(1000) + "\n" + &"Y".repeat(1000);

        // Should handle gracefully without crashing
        assert!(renderer.render_static(&oversized_text).is_ok());
    }

    #[test]
    fn test_invalid_unicode() {
        let test = RendererTest::new();
        let mut renderer = test.create_renderer().unwrap();

        // Text with invalid UTF-8 sequences (represented as replacement characters)
        let text = "Hello �� World";
        assert!(renderer.render_static(text).is_ok());
    }
}

/// Performance tests
mod performance_tests {
    use super::*;

    #[test]
    fn test_large_text_performance() {
        let test = RendererTest::new();
        let mut renderer = test.create_renderer().unwrap();

        // Generate large text content
        let large_text = (0..1000)
            .map(|i| format!("Line {}\n", i))
            .collect::<String>();

        // Measure render time
        let start = std::time::Instant::now();
        renderer.render_static(&large_text).unwrap();
        let duration = start.elapsed();

        // Should render in reasonable time (adjust threshold as needed)
        assert!(
            duration < Duration::from_secs(1),
            "Rendering took too long: {:?}",
            duration
        );
    }

    #[test]
    fn test_animation_performance() {
        let test = RendererTest::new();
        let mut renderer = test.create_renderer().unwrap();

        // Test multiple animation frames
        let frame_count = 100;
        let frame_duration = Duration::from_secs(1) / 60; // 60 FPS

        let start = std::time::Instant::now();
        for i in 0..frame_count {
            let frame_time = frame_duration * i as u32;
            renderer.render_frame("Animation test", frame_time).unwrap();
        }
        let duration = start.elapsed();

        // Should maintain reasonable performance
        let target_duration = frame_duration * frame_count as u32;
        assert!(
            duration < target_duration * 2,
            "Animation too slow: {:?} vs target {:?}",
            duration,
            target_duration
        );
    }
}
