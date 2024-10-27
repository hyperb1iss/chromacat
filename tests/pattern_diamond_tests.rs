use chromacat::pattern::{PatternParam, DiamondParams};

#[test]
fn test_diamond_params_validation() {
    let params = DiamondParams::default();

    // Test valid values
    assert!(params.validate("size=1.0,offset=0.5,sharpness=2.0,rotation=45").is_ok());

    // Test invalid size
    assert!(params.validate("size=0.05,offset=0.5,sharpness=2.0,rotation=45").is_err());

    // Test invalid offset
    assert!(params.validate("size=1.0,offset=1.5,sharpness=2.0,rotation=45").is_err());

    // Test invalid format
    assert!(params.validate("size=1.0,invalid").is_err());
}

#[test]
fn test_diamond_params_parsing() {
    let params = DiamondParams::default();
    
    let parsed = params.parse("size=2.0,offset=0.3,sharpness=3.0,rotation=90")
        .unwrap();
    
    let diamond_params = parsed.as_any()
        .downcast_ref::<DiamondParams>()
        .expect("Failed to downcast parsed parameters");

    assert_eq!(diamond_params.size, 2.0);
    assert_eq!(diamond_params.offset, 0.3);
    assert_eq!(diamond_params.sharpness, 3.0);
    assert_eq!(diamond_params.rotation, 90.0);
}

#[test]
fn test_diamond_params_defaults() {
    let params = DiamondParams::default();
    assert_eq!(params.size, 1.0);
    assert_eq!(params.offset, 0.0);
    assert_eq!(params.sharpness, 1.0);
    assert_eq!(params.rotation, 0.0);
}
