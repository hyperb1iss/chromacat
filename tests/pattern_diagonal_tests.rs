use chromacat::pattern::{DiagonalParams, PatternParam};

#[test]
fn test_diagonal_params_validation() {
    let params = DiagonalParams::default();

    // Test valid values
    assert!(params.validate("angle=45,frequency=1.0").is_ok());

    // Test invalid angle
    assert!(params.validate("angle=400,frequency=1.0").is_err());

    // Test invalid frequency
    assert!(params.validate("angle=45,frequency=0.05").is_err());

    // Test invalid format
    assert!(params.validate("angle=45,invalid").is_err());
}

#[test]
fn test_diagonal_params_parsing() {
    let params = DiagonalParams::default();

    let parsed = params.parse("angle=90,frequency=2.0").unwrap();

    let diagonal_params = parsed
        .as_any()
        .downcast_ref::<DiagonalParams>()
        .expect("Failed to downcast parsed parameters");

    assert_eq!(diagonal_params.angle, 90);
    assert_eq!(diagonal_params.frequency, 2.0);
}

#[test]
fn test_diagonal_params_defaults() {
    let params = DiagonalParams::default();
    assert_eq!(params.angle, 45);
    assert_eq!(params.frequency, 1.0);
}

#[test]
fn test_diagonal_params_bounds() {
    let params = DiagonalParams::default();

    // Test angle bounds
    assert!(params.validate("angle=0,frequency=1.0").is_ok());
    assert!(params.validate("angle=360,frequency=1.0").is_ok());
    assert!(params.validate("angle=-1,frequency=1.0").is_err());
    assert!(params.validate("angle=361,frequency=1.0").is_err());

    // Test frequency bounds
    assert!(params.validate("angle=45,frequency=0.1").is_ok());
    assert!(params.validate("angle=45,frequency=10.0").is_ok());
    assert!(params.validate("angle=45,frequency=0.09").is_err());
    assert!(params.validate("angle=45,frequency=10.1").is_err());
}
