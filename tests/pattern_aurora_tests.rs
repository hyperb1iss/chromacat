use chromacat::pattern::patterns::AuroraParams;
use chromacat::pattern::patterns::Patterns;
use chromacat::pattern::PatternParam;

#[test]
fn test_aurora_params_validation() {
    let params = AuroraParams::default();

    // Test valid values
    assert!(params
        .validate("intensity=1.0,speed=2.0,waviness=1.5,layers=3,height=0.5,spread=0.3")
        .is_ok());

    // Test invalid intensity
    assert!(params.validate("intensity=0.05,speed=1.0").is_err());
    assert!(params.validate("intensity=2.1,speed=1.0").is_err());

    // Test invalid speed
    assert!(params.validate("speed=0.05").is_err());
    assert!(params.validate("speed=5.1").is_err());

    // Test invalid waviness
    assert!(params.validate("waviness=0.05").is_err());
    assert!(params.validate("waviness=2.1").is_err());

    // Test invalid layers
    assert!(params.validate("layers=0").is_err());
    assert!(params.validate("layers=6").is_err());

    // Test invalid height
    assert!(params.validate("height=0.05").is_err());
    assert!(params.validate("height=1.1").is_err());

    // Test invalid spread
    assert!(params.validate("spread=0.05").is_err());
    assert!(params.validate("spread=1.1").is_err());

    // Test invalid format
    assert!(params.validate("intensity=1.0,invalid").is_err());
}

#[test]
fn test_aurora_params_parsing() {
    let params = AuroraParams::default();

    let parsed = params
        .parse("intensity=1.5,speed=2.0,waviness=1.2,layers=4,height=0.7,spread=0.4")
        .unwrap();

    let aurora_params = parsed
        .as_any()
        .downcast_ref::<AuroraParams>()
        .expect("Failed to downcast parsed parameters");

    assert_eq!(aurora_params.intensity, 1.5);
    assert_eq!(aurora_params.speed, 2.0);
    assert_eq!(aurora_params.waviness, 1.2);
    assert_eq!(aurora_params.layers, 4);
    assert_eq!(aurora_params.height, 0.7);
    assert_eq!(aurora_params.spread, 0.4);
}

#[test]
fn test_aurora_params_defaults() {
    let params = AuroraParams::default();
    assert_eq!(params.intensity, 1.0);
    assert_eq!(params.speed, 1.0);
    assert_eq!(params.waviness, 1.0);
    assert_eq!(params.layers, 3);
    assert_eq!(params.height, 0.5);
    assert_eq!(params.spread, 0.3);
}

#[test]
fn test_aurora_animation_behavior() {
    let mut patterns = Patterns::new(100, 100, 0.0, 0);
    let params = AuroraParams::default();

    // Test vertical intensity distribution
    let test_points = [
        (0.5, 0.2, "top"),    // Near top
        (0.5, 0.5, "middle"), // Middle
        (0.5, 0.8, "bottom"), // Near bottom
    ];

    for (x, y, position) in test_points.iter() {
        let value = patterns.aurora(*x, *y, params.clone());
        assert!(
            (0.0..=1.0).contains(&value),
            "Aurora intensity at {} should be between 0 and 1, got {}",
            position,
            value
        );
    }

    // Test temporal variation
    let mut initial_samples = Vec::new();
    let mut later_samples = Vec::new();
    let test_points = [(0.3, 0.4), (0.5, 0.5), (0.7, 0.6)];

    // Sample at t=0
    for &(x, y) in &test_points {
        initial_samples.push(patterns.aurora(x, y, params.clone()));
    }

    // Sample at t=1.0
    patterns = Patterns::new(100, 100, 1.0, 0);
    for &(x, y) in &test_points {
        later_samples.push(patterns.aurora(x, y, params.clone()));
    }

    // Verify temporal variation
    let avg_diff: f64 = initial_samples
        .iter()
        .zip(later_samples.iter())
        .map(|(a, b)| (a - b).abs())
        .sum::<f64>()
        / test_points.len() as f64;

    assert!(
        avg_diff > 0.01,
        "Aurora should show temporal variation. Average difference: {}, Initial samples: {:?}, Later samples: {:?}",
        avg_diff,
        initial_samples,
        later_samples
    );

    // Test layer interaction
    let single_layer = AuroraParams {
        layers: 1,
        ..params.clone()
    };

    let multi_layer = AuroraParams {
        layers: 3,
        ..params.clone()
    };

    let mut single_layer_values = Vec::new();
    let mut multi_layer_values = Vec::new();

    for x in (0..10).map(|i| i as f64 * 0.1) {
        single_layer_values.push(patterns.aurora(x, 0.5, single_layer.clone()));
        multi_layer_values.push(patterns.aurora(x, 0.5, multi_layer.clone()));
    }

    // Calculate variation in values
    let single_var = variance(&single_layer_values);
    let multi_var = variance(&multi_layer_values);

    assert!(
        multi_var > single_var,
        "Multiple layers should create more variation. Single layer var: {}, Multi layer var: {}",
        single_var,
        multi_var
    );
}

// Helper function to calculate variance
fn variance(values: &[f64]) -> f64 {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64
}

#[test]
fn test_aurora_parameter_effects() {
    let patterns = Patterns::new(100, 100, 0.0, 0);
    let base_params = AuroraParams::default();

    // Test intensity scaling - sample where aurora is visible
    let high_intensity = AuroraParams {
        intensity: 2.0,
        ..base_params.clone()
    };
    let low_intensity = AuroraParams {
        intensity: 0.5,
        ..base_params.clone()
    };

    // Sample multiple points to ensure we catch the effect
    let test_points = [(0.3, 0.4), (0.5, 0.5), (0.7, 0.6)];
    let mut high_values = Vec::new();
    let mut low_values = Vec::new();

    for &(x, y) in &test_points {
        high_values.push(patterns.aurora(x, y, high_intensity.clone()));
        low_values.push(patterns.aurora(x, y, low_intensity.clone()));
    }

    // Compare maximum values to ensure we catch the intensity difference
    let max_high = high_values.iter().fold(0.0f64, |a, &b| a.max(b));
    let max_low = low_values.iter().fold(0.0f64, |a, &b| a.max(b));

    assert!(
        max_high > max_low,
        "Higher intensity should produce higher values. High: {}, Low: {}",
        max_high,
        max_low
    );

    // Test waviness effect with more appropriate sampling
    let high_wave = AuroraParams {
        waviness: 2.0,
        speed: 1.0, // Ensure movement
        ..base_params.clone()
    };
    let low_wave = AuroraParams {
        waviness: 0.5,
        speed: 1.0,
        ..base_params.clone()
    };

    // Sample over time to catch the wave movement
    let mut high_wave_values = Vec::new();
    let mut low_wave_values = Vec::new();

    for t in 0..10 {
        let patterns_t = Patterns::new(100, 100, t as f64 * 0.1, 0);
        high_wave_values.push(patterns_t.aurora(0.5, 0.5, high_wave.clone()));
        low_wave_values.push(patterns_t.aurora(0.5, 0.5, low_wave.clone()));
    }

    // Calculate temporal variation instead of spatial
    let high_wave_var = variance(&high_wave_values);
    let low_wave_var = variance(&low_wave_values);

    assert!(
        high_wave_var > low_wave_var,
        "Higher waviness should create more temporal variation. High var: {}, Low var: {}",
        high_wave_var,
        low_wave_var
    );

    // Test height and spread interaction
    let tall_narrow = AuroraParams {
        height: 0.8,
        spread: 0.2,
        ..base_params.clone()
    };
    let short_wide = AuroraParams {
        height: 0.3,
        spread: 0.8,
        ..base_params
    };

    // Sample vertical space more densely
    let tall_values: Vec<f64> = (0..20)
        .map(|i| patterns.aurora(0.5, i as f64 * 0.05, tall_narrow.clone()))
        .collect();
    let short_values: Vec<f64> = (0..20)
        .map(|i| patterns.aurora(0.5, i as f64 * 0.05, short_wide.clone()))
        .collect();

    // Count non-zero values to measure effective height
    let tall_coverage = tall_values.iter().filter(|&&v| v > 0.01).count();
    let short_coverage = short_values.iter().filter(|&&v| v > 0.01).count();
    assert!(
        tall_coverage > short_coverage,
        "Taller aurora should cover more vertical space"
    );
}
