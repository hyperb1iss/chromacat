use chromacat::pattern::{PatternParam, CheckerboardParams};

#[test]
fn test_checkerboard_params_validation() {
    let params = CheckerboardParams::default();

    // Test valid values
    assert!(params.validate("size=2,blur=0.1,rotation=45,scale=1.0").is_ok());

    // Test invalid size
    assert!(params.validate("size=11,blur=0.1,rotation=45,scale=1.0").is_err());

    // Test invalid blur
    assert!(params.validate("size=2,blur=1.5,rotation=45,scale=1.0").is_err());

    // Test invalid rotation
    assert!(params.validate("size=2,blur=0.1,rotation=400,scale=1.0").is_err());

    // Test invalid format
    assert!(params.validate("size=2,invalid").is_err());
}

#[test]
fn test_checkerboard_params_parsing() {
    let params = CheckerboardParams::default();
    
    let parsed = params.parse("size=4,blur=0.3,rotation=90,scale=2.0")
        .unwrap();
    
    let checker_params = parsed.as_any()
        .downcast_ref::<CheckerboardParams>()
        .expect("Failed to downcast parsed parameters");

    assert_eq!(checker_params.size, 4);
    assert_eq!(checker_params.blur, 0.3);
    assert_eq!(checker_params.rotation, 90.0);
    assert_eq!(checker_params.scale, 2.0);
}

#[test]
fn test_checkerboard_params_defaults() {
    let params = CheckerboardParams::default();
    assert_eq!(params.size, 2);
    assert_eq!(params.blur, 0.1);
    assert_eq!(params.rotation, 0.0);
    assert_eq!(params.scale, 1.0);
}
