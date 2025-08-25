use chromacat::pattern::{PatternParam, SpiralParams};

#[test]
fn test_spiral_params_validation() {
    let params = SpiralParams::default();

    // Test valid values
    assert!(params
        .validate("density=1.0,rotation=180,expansion=1.0,clockwise=true,frequency=1.0")
        .is_ok());

    // Test invalid density
    assert!(params
        .validate("density=0.05,rotation=180,expansion=1.0,clockwise=true,frequency=1.0")
        .is_err());

    // Test invalid clockwise
    assert!(params
        .validate("density=1.0,rotation=180,expansion=1.0,clockwise=invalid,frequency=1.0")
        .is_err());

    // Test invalid format
    assert!(params.validate("density=1.0,invalid").is_err());
}

#[test]
fn test_spiral_params_parsing() {
    let params = SpiralParams::default();

    let parsed = params
        .parse("density=2.0,rotation=90,expansion=1.5,clockwise=false,frequency=3.0")
        .unwrap();

    let spiral_params = parsed
        .as_any()
        .downcast_ref::<SpiralParams>()
        .expect("Failed to downcast parsed parameters");

    assert_eq!(spiral_params.density, 2.0);
    assert_eq!(spiral_params.rotation, 90.0);
    assert_eq!(spiral_params.expansion, 1.5);
    assert!(!spiral_params.clockwise);
    assert_eq!(spiral_params.frequency, 3.0);
}

#[test]
fn test_spiral_params_defaults() {
    let params = SpiralParams::default();
    assert_eq!(params.density, 1.0);
    assert_eq!(params.rotation, 0.0);
    assert_eq!(params.expansion, 1.0);
    assert!(params.clockwise);
    assert_eq!(params.frequency, 1.0);
}
