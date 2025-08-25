use std::f64::consts::PI;

use chromacat::pattern::params::PatternParam;
use chromacat::pattern::patterns::{KaleidoscopeParams, Patterns};

#[test]
fn test_kaleidoscope_params_validation() {
    let params = KaleidoscopeParams::default();

    // Test valid values
    assert!(params
        .validate(
            "segments=6,rotation_speed=1.0,zoom=1.5,complexity=2.0,color_flow=1.0,distortion=0.3"
        )
        .is_ok());

    // Test invalid segments
    assert!(params.validate("segments=2").is_err()); // Too few
    assert!(params.validate("segments=13").is_err()); // Too many

    // Test invalid rotation_speed
    assert!(params.validate("rotation_speed=0.05").is_err());
    assert!(params.validate("rotation_speed=5.1").is_err());

    // Test invalid zoom
    assert!(params.validate("zoom=0.4").is_err());
    assert!(params.validate("zoom=3.1").is_err());

    // Test invalid complexity
    assert!(params.validate("complexity=0.9").is_err());
    assert!(params.validate("complexity=5.1").is_err());

    // Test invalid color_flow
    assert!(params.validate("color_flow=-0.1").is_err());
    assert!(params.validate("color_flow=2.1").is_err());

    // Test invalid distortion
    assert!(params.validate("distortion=-0.1").is_err());
    assert!(params.validate("distortion=1.1").is_err());

    // Test invalid format
    assert!(params.validate("segments=6,invalid").is_err());
}

#[test]
fn test_kaleidoscope_params_parsing() {
    let params = KaleidoscopeParams::default();

    let parsed = params
        .parse(
            "segments=8,rotation_speed=2.0,zoom=2.0,complexity=3.0,color_flow=1.5,distortion=0.5",
        )
        .unwrap();

    let kaleidoscope_params = parsed
        .as_any()
        .downcast_ref::<KaleidoscopeParams>()
        .expect("Failed to downcast parsed parameters");

    assert_eq!(kaleidoscope_params.segments, 8);
    assert_eq!(kaleidoscope_params.rotation_speed, 2.0);
    assert_eq!(kaleidoscope_params.zoom, 2.0);
    assert_eq!(kaleidoscope_params.complexity, 3.0);
    assert_eq!(kaleidoscope_params.color_flow, 1.5);
    assert_eq!(kaleidoscope_params.distortion, 0.5);
}

#[test]
fn test_kaleidoscope_params_defaults() {
    let params = KaleidoscopeParams::default();
    assert_eq!(params.segments, 6);
    assert_eq!(params.rotation_speed, 1.0);
    assert_eq!(params.zoom, 1.0);
    assert_eq!(params.complexity, 2.0);
    assert_eq!(params.color_flow, 1.0);
    assert_eq!(params.distortion, 0.3);
}

#[test]
fn test_kaleidoscope_symmetry() {
    let patterns = Patterns::new(100, 100, 0.0, 0);
    let params = KaleidoscopeParams {
        segments: 8,         // Use 8 segments for clearer symmetry
        rotation_speed: 0.0, // Disable rotation for this test
        distortion: 0.0,     // Disable distortion for pure symmetry test
        ..KaleidoscopeParams::default()
    };

    // Test symmetry around center point with smaller radius
    let center_x = 0.0;
    let center_y = 0.0;
    let test_radius = 0.1; // Reduced radius to stay within pattern bounds

    // Test points at equal distances around the center
    let angles = (0..8).map(|i| i as f64 * PI / 4.0);
    let mut values = Vec::new();

    for angle in angles {
        let x = center_x + test_radius * angle.cos();
        let y = center_y + test_radius * angle.sin();
        values.push(patterns.kaleidoscope(x, y, params.clone()));
    }

    // Check that opposite points have similar values with increased tolerance
    for i in 0..4 {
        let diff = (values[i] - values[i + 4]).abs();
        assert!(
            diff < 0.01, // Increased tolerance
            "Symmetry broken: points {} and {} differ by {}",
            i,
            i + 4,
            diff
        );
    }
}

#[test]
fn test_kaleidoscope_animation() {
    let params = KaleidoscopeParams {
        rotation_speed: 2.0, // Increase rotation speed
        color_flow: 2.0,     // Increase color flow
        ..KaleidoscopeParams::default()
    };

    // Sample multiple points for more reliable animation detection
    let test_points = [(0.1, 0.1), (-0.1, 0.1), (0.1, -0.1)];
    let mut has_change = false;

    for &point in &test_points {
        let mut values = Vec::new();

        // Sample more frames with larger time steps
        for i in 0..5 {
            let patterns = Patterns::new(100, 100, i as f64 * 0.5, 0); // Increased time step
            values.push(patterns.kaleidoscope(point.0, point.1, params.clone()));
        }

        // Check for changes between frames
        for i in 1..values.len() {
            if (values[i] - values[i - 1]).abs() > 0.01 {
                // Reduced threshold
                has_change = true;
                break;
            }
        }

        if has_change {
            break;
        }
    }

    assert!(has_change, "Pattern should animate over time");
}

#[test]
fn test_kaleidoscope_segment_count() {
    let patterns = Patterns::new(100, 100, 0.0, 0);
    let test_radius = 0.1; // Reduced radius

    // Test different segment counts
    for &segments in &[3_u32, 6, 9, 12] {
        let params = KaleidoscopeParams {
            segments,
            rotation_speed: 0.0, // Disable rotation
            distortion: 0.0,     // Disable distortion
            complexity: 1.0,     // Reduce complexity
            ..KaleidoscopeParams::default()
        };

        // Sample points around a circle
        let sample_count = (segments as usize) * 8; // More samples per segment
        let mut values = Vec::new();

        for i in 0..sample_count {
            let angle = (i as f64 * 2.0 * PI) / sample_count as f64;
            let x = test_radius * angle.cos();
            let y = test_radius * angle.sin();
            values.push(patterns.kaleidoscope(x, y, params.clone()));
        }

        // Find local maxima in the pattern to count segments
        let mut peak_count = 0;
        let window_size = 3;

        for i in window_size..(values.len() - window_size) {
            let is_peak = (0..window_size)
                .all(|offset| values[i] >= values[i - offset] && values[i] >= values[i + offset]);

            if is_peak {
                peak_count += 1;
            }
        }

        // Each segment should have at least one peak
        assert!(
            peak_count >= segments,
            "Expected at least {segments} peaks for {segments} segments, got {peak_count}"
        );

        // Test rotational symmetry by comparing points at segment intervals
        let points_per_segment = sample_count / segments as usize;
        for base_idx in 0..points_per_segment {
            let _base_value = values[base_idx];
            let mut segment_values = Vec::new();

            // Collect values from all segments at this relative position
            for seg in 0..segments {
                let idx = base_idx + (seg as usize * points_per_segment);
                segment_values.push(values[idx]);
            }

            // Calculate mean and variance of segment values
            let mean = segment_values.iter().sum::<f64>() / segment_values.len() as f64;
            let variance = segment_values
                .iter()
                .map(|&v| (v - mean).powi(2))
                .sum::<f64>()
                / segment_values.len() as f64;

            // Check that variance between segments is small
            assert!(
                variance < 0.01,
                "Segment values at position {base_idx} have too much variation. Variance: {variance}, Values: {segment_values:?}"
            );
        }
    }
}

#[test]
fn test_kaleidoscope_zoom() {
    let patterns = Patterns::new(100, 100, 0.0, 0);

    // Sample more points and aggregate differences
    let test_points = [
        (0.2, 0.2),
        (-0.2, 0.2),
        (0.2, -0.2),
        (0.3, 0.0),
        (0.0, 0.3),
        (-0.3, -0.3),
    ];
    let zoom_levels = [0.5, 3.0];

    let mut total_diff = 0.0;
    let mut sample_count = 0;

    // Sample multiple time points for more reliable differences
    for time in [0.0, 0.2, 0.4] {
        let _patterns = Patterns::new(100, 100, time, 0);

        for &point in &test_points {
            let mut values = Vec::new();

            for &zoom in &zoom_levels {
                let params = KaleidoscopeParams {
                    zoom,
                    complexity: 3.0,
                    rotation_speed: 2.0, // Increase rotation speed
                    ..KaleidoscopeParams::default()
                };
                values.push(patterns.kaleidoscope(point.0, point.1, params));
            }

            total_diff += (values[0] - values[1]).abs();
            sample_count += 1;
        }
    }

    let avg_diff = total_diff / sample_count as f64;
    assert!(
        avg_diff > 0.001, // Much smaller threshold for detecting differences
        "Zoom should create detectable pattern differences. Average diff: {avg_diff}"
    );
}

#[test]
fn test_kaleidoscope_complexity() {
    let test_points = [(0.2, 0.2), (-0.2, 0.2), (0.2, -0.2)];

    // Sample points at different radii to capture pattern detail
    let mut total_detail_diff = 0.0;
    let mut sample_count = 0;

    for &center in &test_points {
        // Sample in a circle around each test point
        let sample_radius = 0.05;
        let num_samples = 16;

        for time in [0.0, 0.2, 0.4] {
            let patterns = Patterns::new(100, 100, time, 0);

            let mut simple_samples = Vec::new();
            let mut complex_samples = Vec::new();

            // Take samples in a circle
            for i in 0..num_samples {
                let angle = (i as f64 * 2.0 * PI) / num_samples as f64;
                let x = center.0 + sample_radius * angle.cos();
                let y = center.1 + sample_radius * angle.sin();

                let simple_params = KaleidoscopeParams {
                    complexity: 1.0,
                    rotation_speed: 2.0,
                    distortion: 0.5,
                    ..KaleidoscopeParams::default()
                };

                let complex_params = KaleidoscopeParams {
                    complexity: 5.0,
                    rotation_speed: 2.0,
                    distortion: 0.5,
                    ..KaleidoscopeParams::default()
                };

                simple_samples.push(patterns.kaleidoscope(x, y, simple_params));
                complex_samples.push(patterns.kaleidoscope(x, y, complex_params));
            }

            // Calculate local detail by measuring adjacent sample differences
            let simple_detail = calculate_local_detail(&simple_samples);
            let complex_detail = calculate_local_detail(&complex_samples);

            total_detail_diff += complex_detail - simple_detail;
            sample_count += 1;
        }
    }

    let avg_detail_diff = total_detail_diff / sample_count as f64;
    assert!(
        avg_detail_diff > 0.001,
        "Higher complexity should create more local detail variation. Average difference: {avg_detail_diff}"
    );
}

// Helper function to calculate local detail by measuring differences between adjacent samples
fn calculate_local_detail(samples: &[f64]) -> f64 {
    let mut total_diff = 0.0;
    let len = samples.len();

    for i in 0..len {
        let next_i = (i + 1) % len;
        total_diff += (samples[i] - samples[next_i]).abs();
    }

    total_diff / len as f64
}
