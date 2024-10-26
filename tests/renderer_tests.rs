//! Integration tests for the rendering system
//!
//! Tests the rendering pipeline, including static and animated rendering,
//! terminal interaction, color handling, and performance.

use chromacat::pattern::{CommonParams, PatternConfig, PatternEngine, PatternParams};
use chromacat::renderer::{AnimationConfig, Renderer};
use colorgrad::{Color, Gradient};
use std::time::Duration;

/// Mock gradient for testing
struct MockGradient;

impl Gradient for MockGradient {
    fn at(&self, t: f32) -> Color {
        Color::new(t, t, t, 1.0_f32)
    }
}

fn create_test_gradient() -> Box<dyn Gradient + Send + Sync> {
    Box::new(MockGradient)
}

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

        let engine = PatternEngine::new(
            create_test_gradient(),
            pattern_config,
            80,  // width
            24   // height
        );

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
        ("  Leading spaces  ", "whitespace"),
        ("Very long text that should wrap automatically when it reaches the terminal width limit", "wrapping"),
        ("\t\tTabbed\ttext", "tabs"),
        ("Mixed 👨‍👩‍👧‍👦 family emoji", "complex emoji"),
        ("", "empty string"),
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
    assert!(!renderer.is_infinite());
    assert_eq!(renderer.cycle_duration(), Duration::from_secs(1));
}

#[test]
fn test_animation_progress() {
    let test = RendererTest::new();
    let mut renderer = test.create_renderer().unwrap();

    let progress_points = [
        Duration::from_millis(0),    // Start
        Duration::from_millis(500),  // Middle
        Duration::from_millis(999),  // Just before end
    ];

    for duration in progress_points {
        assert!(
            renderer.render_frame("Animation Test", duration).is_ok(),
            "Failed to render at {:?}",
            duration
        );
    }
}

#[test]
fn test_unicode_width() {
    let test = RendererTest::new();
    let mut renderer = test.create_renderer().unwrap();

    // Text with various width characters
    let test_cases = vec![
        "Hello",             // ASCII
        "世界",              // CJK
        "👨‍👩‍👧‍👦",    // Wide emoji
        "α β γ",            // Greek
        "🏳️‍🌈",           // Flag
        "ｆｕｌｌwidth",    // Full-width
    ];

    for text in test_cases {
        assert!(
            renderer.render_static(text).is_ok(),
            "Failed to handle width of: {}",
            text
        );
    }
}

/// Terminal interaction tests
mod terminal_tests {
    use super::*;

    #[test]
    fn test_terminal_cleanup() {
        let test = RendererTest::new();
        {
            let renderer = test.create_renderer().unwrap();
            drop(renderer); // Should clean up terminal state
        }
    }

    #[test]
    fn test_terminal_resize() {
        let test = RendererTest::new();
        let mut renderer = test.create_renderer().unwrap();
        
        // Test rendering before and after theoretical resize
        assert!(renderer.render_static("Before resize").is_ok());
        assert!(renderer.render_static("After resize").is_ok());
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

        let start = std::time::Instant::now();
        renderer.render_static(&large_text).unwrap();
        let duration = start.elapsed();

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

        let frame_count = 100;
        let frame_duration = Duration::from_secs(1) / 60; // 60 FPS

        let start = std::time::Instant::now();
        for i in 0..frame_count {
            let frame_time = frame_duration * i as u32;
            renderer.render_frame("Animation test", frame_time).unwrap();
        }
        let duration = start.elapsed();

        let target_duration = frame_duration * frame_count as u32;
        assert!(
            duration < target_duration * 2,
            "Animation too slow: {:?} vs target {:?}",
            duration,
            target_duration
        );
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

        assert!(renderer.render_static(&oversized_text).is_ok());
    }

    #[test]
    fn test_invalid_unicode() {
        let test = RendererTest::new();
        let mut renderer = test.create_renderer().unwrap();

        let text = "Hello �� World";
        assert!(renderer.render_static(text).is_ok());
    }

    #[test]
    fn test_null_character() {
        let test = RendererTest::new();
        let mut renderer = test.create_renderer().unwrap();

        let text = "Hello\0World";
        assert!(renderer.render_static(text).is_ok());
    }
}

/// Configuration tests
mod config_tests {
    use super::*;

    #[test]
    fn test_various_fps() {
        let fps_values = [1, 30, 60, 144];
        
        for fps in fps_values {
            let mut config = AnimationConfig::default();
            config.fps = fps;
            
            let test = RendererTest {
                config,
                ..RendererTest::new()
            };
            
            let renderer = test.create_renderer().unwrap();
            assert_eq!(
                renderer.frame_duration(),
                Duration::from_secs(1) / fps as u32
            );
        }
    }

    #[test]
    fn test_infinite_animation() {
        let mut config = AnimationConfig::default();
        config.infinite = true;
        
        let test = RendererTest {
            config,
            ..RendererTest::new()
        };
        
        let renderer = test.create_renderer().unwrap();
        assert!(renderer.is_infinite());
    }
}