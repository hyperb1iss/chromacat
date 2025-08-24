use chromacat::pattern::patterns::PlasmaBlendMode;
use chromacat::pattern::{PatternParam, PlasmaParams};

#[test]
fn test_plasma_params_validation() {
    let params = PlasmaParams::default();

    // Test valid values
    assert!(params
        .validate("complexity=3.0,scale=1.5,frequency=1.0,blend_mode=add")
        .is_ok());

    // Test invalid complexity
    assert!(params
        .validate("complexity=11.0,scale=1.5,frequency=1.0,blend_mode=add")
        .is_err());

    // Test invalid scale
    assert!(params
        .validate("complexity=3.0,scale=0.05,frequency=1.0,blend_mode=add")
        .is_err());

    // Test invalid frequency
    assert!(params
        .validate("complexity=3.0,scale=1.5,frequency=0.05,blend_mode=add")
        .is_err());

    // Test invalid blend mode
    assert!(params
        .validate("complexity=3.0,scale=1.5,frequency=1.0,blend_mode=invalid")
        .is_err());

    // Test invalid format
    assert!(params.validate("complexity=3.0,invalid").is_err());
}

#[test]
fn test_plasma_params_parsing() {
    let params = PlasmaParams::default();

    let parsed = params
        .parse("complexity=5.0,scale=2.0,frequency=3.0,blend_mode=multiply")
        .unwrap();

    let plasma_params = parsed
        .as_any()
        .downcast_ref::<PlasmaParams>()
        .expect("Failed to downcast parsed parameters");

    assert_eq!(plasma_params.complexity, 5.0);
    assert_eq!(plasma_params.scale, 2.0);
    assert_eq!(plasma_params.frequency, 3.0);
    matches!(plasma_params.blend_mode, PlasmaBlendMode::Multiply);
}

#[test]
fn test_plasma_params_defaults() {
    let params = PlasmaParams::default();
    assert_eq!(params.complexity, 3.0);
    assert_eq!(params.scale, 1.0);
    assert_eq!(params.frequency, 1.0);
    matches!(params.blend_mode, PlasmaBlendMode::Additive);
}

#[test]
fn test_plasma_params_bounds() {
    let params = PlasmaParams::default();

    // Test complexity bounds
    assert!(params
        .validate("complexity=1.0,scale=1.0,frequency=1.0,blend_mode=add")
        .is_ok());
    assert!(params
        .validate("complexity=10.0,scale=1.0,frequency=1.0,blend_mode=add")
        .is_ok());
    assert!(params
        .validate("complexity=0.9,scale=1.0,frequency=1.0,blend_mode=add")
        .is_err());
    assert!(params
        .validate("complexity=10.1,scale=1.0,frequency=1.0,blend_mode=add")
        .is_err());

    // Test scale bounds
    assert!(params
        .validate("complexity=3.0,scale=0.1,frequency=1.0,blend_mode=add")
        .is_ok());
    assert!(params
        .validate("complexity=3.0,scale=5.0,frequency=1.0,blend_mode=add")
        .is_ok());
    assert!(params
        .validate("complexity=3.0,scale=0.09,frequency=1.0,blend_mode=add")
        .is_err());
    assert!(params
        .validate("complexity=3.0,scale=5.1,frequency=1.0,blend_mode=add")
        .is_err());

    // Test frequency bounds
    assert!(params
        .validate("complexity=3.0,scale=1.0,frequency=0.1,blend_mode=add")
        .is_ok());
    assert!(params
        .validate("complexity=3.0,scale=1.0,frequency=10.0,blend_mode=add")
        .is_ok());
    assert!(params
        .validate("complexity=3.0,scale=1.0,frequency=0.09,blend_mode=add")
        .is_err());
    assert!(params
        .validate("complexity=3.0,scale=1.0,frequency=10.1,blend_mode=add")
        .is_err());
}
