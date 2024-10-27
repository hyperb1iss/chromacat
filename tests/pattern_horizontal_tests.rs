use chromacat::pattern::{PatternParam, HorizontalParams};

#[test]
fn test_horizontal_params_validation() {
    let params = HorizontalParams::default();

    // Test valid values
    assert!(params.validate("invert=true").is_ok());
    assert!(params.validate("invert=false").is_ok());

    // Test invalid invert value
    assert!(params.validate("invert=yes").is_err());

    // Test invalid format
    assert!(params.validate("invert=true,invalid").is_err());
}

#[test]
fn test_horizontal_params_parsing() {
    let params = HorizontalParams::default();
    
    let parsed = params.parse("invert=true")
        .unwrap();
    
    let horizontal_params = parsed.as_any()
        .downcast_ref::<HorizontalParams>()
        .expect("Failed to downcast parsed parameters");

    assert!(horizontal_params.invert);
}

#[test]
fn test_horizontal_params_defaults() {
    let params = HorizontalParams::default();
    assert!(!params.invert);
}
