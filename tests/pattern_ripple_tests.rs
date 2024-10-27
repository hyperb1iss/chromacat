use chromacat::pattern::{PatternParam, RippleParams};

#[test]
fn test_ripple_params_validation() {
    let params = RippleParams::default();

    // Test valid values
    assert!(params.validate("center_x=0.5,center_y=0.5,wavelength=1.0,damping=0.5,frequency=1.0").is_ok());

    // Test invalid center_x
    assert!(params.validate("center_x=1.5,center_y=0.5,wavelength=1.0,damping=0.5,frequency=1.0").is_err());

    // Test invalid wavelength
    assert!(params.validate("center_x=0.5,center_y=0.5,wavelength=0.05,damping=0.5,frequency=1.0").is_err());

    // Test invalid format
    assert!(params.validate("center_x=0.5,invalid").is_err());
}

#[test]
fn test_ripple_params_parsing() {
    let params = RippleParams::default();
    
    let parsed = params.parse("center_x=0.7,center_y=0.3,wavelength=2.0,damping=0.8,frequency=5.0")
        .unwrap();
    
    let ripple_params = parsed.as_any()
        .downcast_ref::<RippleParams>()
        .expect("Failed to downcast parsed parameters");

    assert_eq!(ripple_params.center_x, 0.7);
    assert_eq!(ripple_params.center_y, 0.3);
    assert_eq!(ripple_params.wavelength, 2.0);
    assert_eq!(ripple_params.damping, 0.8);
    assert_eq!(ripple_params.frequency, 5.0);
}

#[test]
fn test_ripple_params_defaults() {
    let params = RippleParams::default();
    assert_eq!(params.center_x, 0.5);
    assert_eq!(params.center_y, 0.5);
    assert_eq!(params.wavelength, 1.0);
    assert_eq!(params.damping, 0.5);
    assert_eq!(params.frequency, 1.0);
}
