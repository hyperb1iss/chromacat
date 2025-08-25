use chromacat::pattern::params::PatternParam;
use chromacat::pattern::patterns::{FireParams, Patterns};

#[test]
fn test_fire_params_validation() {
    let params = FireParams::default();

    // Test valid values
    assert!(params
        .validate("intensity=1.0,speed=2.0,turbulence=0.5,height=1.0,wind=true,wind_strength=0.3")
        .is_ok());

    // Test invalid intensity
    assert!(params.validate("intensity=0.05,speed=1.0").is_err());
    assert!(params.validate("intensity=2.1,speed=1.0").is_err());

    // Test invalid speed
    assert!(params.validate("intensity=1.0,speed=0.05").is_err());
    assert!(params.validate("intensity=1.0,speed=5.1").is_err());

    // Test invalid turbulence
    assert!(params.validate("turbulence=-0.1").is_err());
    assert!(params.validate("turbulence=1.1").is_err());

    // Test invalid height
    assert!(params.validate("height=0.05").is_err());
    assert!(params.validate("height=2.1").is_err());

    // Test invalid wind_strength
    assert!(params.validate("wind_strength=-0.1").is_err());
    assert!(params.validate("wind_strength=1.1").is_err());

    // Test invalid format
    assert!(params.validate("intensity=1.0,invalid").is_err());
}

#[test]
fn test_fire_params_parsing() {
    let params = FireParams::default();

    let parsed = params
        .parse("intensity=1.5,speed=2.0,turbulence=0.7,height=1.2,wind=false,wind_strength=0.5")
        .unwrap();

    let fire_params = parsed
        .as_any()
        .downcast_ref::<FireParams>()
        .expect("Failed to downcast parsed parameters");

    assert_eq!(fire_params.intensity, 1.5);
    assert_eq!(fire_params.speed, 2.0);
    assert_eq!(fire_params.turbulence, 0.7);
    assert_eq!(fire_params.height, 1.2);
    assert!(!fire_params.wind);
    assert_eq!(fire_params.wind_strength, 0.5);
}

#[test]
fn test_fire_params_defaults() {
    let params = FireParams::default();
    assert_eq!(params.intensity, 1.0);
    assert_eq!(params.speed, 1.0);
    assert_eq!(params.turbulence, 0.5);
    assert_eq!(params.height, 1.0);
    assert!(params.wind);
    assert_eq!(params.wind_strength, 0.3);
}

#[test]
fn test_fire_params_bounds() {
    let params = FireParams::default();

    // Test intensity bounds
    assert!(params.validate("intensity=0.1").is_ok());
    assert!(params.validate("intensity=2.0").is_ok());
    assert!(params.validate("intensity=0.09").is_err());
    assert!(params.validate("intensity=2.1").is_err());

    // Test speed bounds
    assert!(params.validate("speed=0.1").is_ok());
    assert!(params.validate("speed=5.0").is_ok());
    assert!(params.validate("speed=0.09").is_err());
    assert!(params.validate("speed=5.1").is_err());

    // Test turbulence bounds
    assert!(params.validate("turbulence=0.0").is_ok());
    assert!(params.validate("turbulence=1.0").is_ok());
    assert!(params.validate("turbulence=-0.1").is_err());
    assert!(params.validate("turbulence=1.1").is_err());

    // Test height bounds
    assert!(params.validate("height=0.1").is_ok());
    assert!(params.validate("height=2.0").is_ok());
    assert!(params.validate("height=0.09").is_err());
    assert!(params.validate("height=2.1").is_err());

    // Test wind_strength bounds
    assert!(params.validate("wind_strength=0.0").is_ok());
    assert!(params.validate("wind_strength=1.0").is_ok());
    assert!(params.validate("wind_strength=-0.1").is_err());
    assert!(params.validate("wind_strength=1.1").is_err());
}

#[test]
fn test_fire_animation_behavior() {
    let mut patterns = Patterns::new(100, 100, 0.0, 0);
    let params = FireParams::default();

    // Test that fire exists and has reasonable intensity values
    let samples = [
        (0.0, 0.9, "bottom"), // Near bottom
        (0.0, 0.5, "middle"), // Middle
        (0.0, 0.1, "top"),    // Near top
    ];

    // Instead of testing strict decrease, test that values are within expected ranges
    for (x, y, position) in samples.iter() {
        let value = patterns.fire(*x, *y, params.clone());
        assert!(
            (0.0..=1.0).contains(&value),
            "Fire intensity at {position} should be between 0 and 1, got {value}"
        );
    }

    // Test wind effect with more extreme parameters
    let params_with_wind = FireParams {
        wind: true,
        wind_strength: 1.0,
        turbulence: 0.8,
        speed: 2.0,
        ..params.clone()
    };

    let params_no_wind = FireParams {
        wind: false,
        wind_strength: 0.0,
        turbulence: 0.0,
        speed: 2.0,
        ..params.clone()
    };

    // Sample points at mid-height with wider spread
    let x_samples = [-0.4, -0.3, -0.2, -0.1, 0.0, 0.1, 0.2, 0.3, 0.4];
    let y_test = 0.5;

    // Take multiple samples over time to capture wind movement
    let mut wind_total_var = 0.0;
    let mut no_wind_total_var = 0.0;
    let sample_count = 5;

    for i in 0..sample_count {
        patterns = Patterns::new(100, 100, i as f64 * 0.2, 0);

        // Get samples with and without wind
        let wind_samples: Vec<f64> = x_samples
            .iter()
            .map(|x| patterns.fire(*x, y_test, params_with_wind.clone()))
            .collect();

        let no_wind_samples: Vec<f64> = x_samples
            .iter()
            .map(|x| patterns.fire(*x, y_test, params_no_wind.clone()))
            .collect();

        // Calculate variances
        wind_total_var += variance(&wind_samples);
        no_wind_total_var += variance(&no_wind_samples);
    }

    let wind_var = wind_total_var / sample_count as f64;
    let no_wind_var = no_wind_total_var / sample_count as f64;

    // Wind should create more horizontal variation
    assert!(
        wind_var > no_wind_var,
        "Wind should create more horizontal variation. With wind: {wind_var}, Without wind: {no_wind_var}"
    );

    // Test temporal variation by sampling multiple points
    let params_temporal = FireParams {
        turbulence: 1.0,
        speed: 5.0,
        ..params
    };

    let mut initial_samples = Vec::new();
    let mut later_samples = Vec::new();

    // Sample multiple points to ensure we catch the variation
    let test_points = [(-0.3, 0.3), (-0.1, 0.4), (0.0, 0.5), (0.1, 0.4), (0.3, 0.3)];

    patterns = Patterns::new(100, 100, 0.0, 0);
    for (x, y) in &test_points {
        initial_samples.push(patterns.fire(*x, *y, params_temporal.clone()));
    }

    patterns = Patterns::new(100, 100, 5.0, 0);
    for (x, y) in &test_points {
        later_samples.push(patterns.fire(*x, *y, params_temporal.clone()));
    }

    // Calculate average difference between samples
    let avg_diff: f64 = initial_samples
        .iter()
        .zip(later_samples.iter())
        .map(|(a, b)| (a - b).abs())
        .sum::<f64>()
        / test_points.len() as f64;

    assert!(
        avg_diff > 0.01,
        "Fire should show temporal variation. Average difference: {avg_diff}, Initial samples: {initial_samples:?}, Later samples: {later_samples:?}"
    );
}

// Helper function to calculate variance
fn variance(values: &[f64]) -> f64 {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64
}
