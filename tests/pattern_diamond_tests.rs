use chromacat::pattern::{DiamondParams, PatternParam};

#[test]
fn test_diamond_params_validation() {
    let params = DiamondParams::default();

    // Test valid values
    assert!(params
        .validate("size=1.0,offset=0.5,sharpness=2.0,rotation=45,speed=1.0,mode=zoom")
        .is_ok());
    assert!(params.validate("mode=scroll,speed=2.0").is_ok());
    assert!(params.validate("mode=static").is_ok());

    // Test invalid values
    assert!(params.validate("size=0.05").is_err()); // too small
    assert!(params.validate("offset=1.5").is_err()); // too large
    assert!(params.validate("speed=-1").is_err()); // negative speed
    assert!(params.validate("mode=invalid").is_err()); // invalid mode

    // Test invalid format
    assert!(params.validate("size=1.0,invalid").is_err());
}

#[test]
fn test_diamond_params_parsing() {
    let params = DiamondParams::default();

    let parsed = params
        .parse("size=2.0,offset=0.3,sharpness=3.0,rotation=90,speed=2.5,mode=scroll")
        .unwrap();

    let diamond_params = parsed
        .as_any()
        .downcast_ref::<DiamondParams>()
        .expect("Failed to downcast parsed parameters");

    assert_eq!(diamond_params.size, 2.0);
    assert_eq!(diamond_params.offset, 0.3);
    assert_eq!(diamond_params.sharpness, 3.0);
    assert_eq!(diamond_params.rotation, 90.0);
    assert_eq!(diamond_params.speed, 2.5);
    assert_eq!(diamond_params.mode, "scroll");
}

#[test]
fn test_diamond_params_defaults() {
    let params = DiamondParams::default();
    assert_eq!(params.size, 1.0);
    assert_eq!(params.offset, 0.0);
    assert_eq!(params.sharpness, 1.0);
    assert_eq!(params.rotation, 0.0);
    assert_eq!(params.speed, 1.0);
    assert_eq!(params.mode, "zoom");
}

#[test]
fn test_diamond_animation_modes() {
    let params = DiamondParams::default();

    // Test each valid animation mode
    assert!(params.validate("mode=zoom").is_ok());
    assert!(params.validate("mode=scroll").is_ok());
    assert!(params.validate("mode=static").is_ok());

    // Test parsing of each mode
    let zoom_params = params.parse("mode=zoom").unwrap();
    let scroll_params = params.parse("mode=scroll").unwrap();
    let static_params = params.parse("mode=static").unwrap();

    assert_eq!(
        zoom_params
            .as_any()
            .downcast_ref::<DiamondParams>()
            .unwrap()
            .mode,
        "zoom"
    );
    assert_eq!(
        scroll_params
            .as_any()
            .downcast_ref::<DiamondParams>()
            .unwrap()
            .mode,
        "scroll"
    );
    assert_eq!(
        static_params
            .as_any()
            .downcast_ref::<DiamondParams>()
            .unwrap()
            .mode,
        "static"
    );
}
