//! Integration tests for the rendering system
//!
//! Tests the rendering pipeline, including static and animated rendering,
//! terminal interaction, color handling, and performance.

use chromacat::pattern::{
    CommonParams, HorizontalParams, PatternConfig, PatternEngine, PatternParams,
};
use chromacat::renderer::{AnimationConfig, Renderer};
use colorgrad::{Color, Gradient};
use std::time::Duration;

/// Mock gradient for testing
#[derive(Clone)]
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
            params: PatternParams::Horizontal(HorizontalParams::default()),
        };

        let engine = PatternEngine::new(
            create_test_gradient(),
            pattern_config,
            80, // width
            24, // height
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
        let renderer = Renderer::new(
            self.engine.clone(),
            self.config.clone(),
            None,  // playlist
            false, // demo_mode
        )?;
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

    // Pass delta time in seconds instead of Duration
    assert!(renderer.render_frame("Test", 0.016).is_ok()); // ~60fps
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

    // Calculate expected duration (33.333... ms for 30 FPS)
    let expected = Duration::from_nanos(1_000_000_000u64 / 30);
    let actual = renderer.frame_duration();

    // Compare durations with a small epsilon
    let difference = if actual > expected {
        actual - expected
    } else {
        expected - actual
    };

    assert!(
        difference < Duration::from_micros(1),
        "Frame duration {:?} differs from expected {:?} by {:?}",
        actual,
        expected,
        difference
    );

    assert!(!renderer.is_infinite());
    assert_eq!(renderer.cycle_duration(), Duration::from_secs(1));
}

#[test]
fn test_animation_progress() {
    let test = RendererTest::new();
    let mut renderer = test.create_renderer().unwrap();

    // Test different points in time (in seconds)
    let progress_points = [
        0.0,   // Start
        0.5,   // Middle
        0.999, // Just before end
    ];

    for seconds in progress_points {
        assert!(
            renderer.render_frame("Animation Test", seconds).is_ok(),
            "Failed to render at {} seconds",
            seconds
        );
    }
}

#[test]
fn test_unicode_width() {
    let test = RendererTest::new();
    let mut renderer = test.create_renderer().unwrap();

    // Text with various width characters
    let test_cases = vec![
        "Hello",         // ASCII
        "世界",          // CJK
        "👨‍👩‍👧‍👦",            // Wide emoji
        "α β γ",         // Greek
        "🏳️‍🌈",            // Flag
        "ｆｕｌｌwidth", // Full-width
    ];

    for text in test_cases {
        assert!(
            renderer.render_static(text).is_ok(),
            "Failed to handle width of: {}",
            text
        );
    }
}

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

    let frame_count = 10; // Reduced frame count for testing
    let delta_seconds = 0.016; // ~60 FPS timing

    let start = std::time::Instant::now();

    // Render frames with small, fixed time increments
    for _ in 0..frame_count {
        renderer
            .render_frame("Animation test", delta_seconds)
            .unwrap();
    }

    let duration = start.elapsed();

    // More lenient performance threshold
    let max_allowed_duration = Duration::from_millis((frame_count * 50) as u64); // Allow ~50ms per frame
    assert!(
        duration < max_allowed_duration,
        "Animation too slow: {:?} (allowed: {:?})",
        duration,
        max_allowed_duration
    );
}
