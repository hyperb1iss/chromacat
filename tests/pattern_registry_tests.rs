use chromacat::pattern::{PatternParams, REGISTRY};

#[test]
fn test_registry_initialization() {
    // Test that all patterns are registered
    let patterns = REGISTRY.list_patterns();
    assert!(patterns.contains(&"horizontal"));
    assert!(patterns.contains(&"diagonal"));
    assert!(patterns.contains(&"plasma"));
    assert!(patterns.contains(&"ripple"));
    assert!(patterns.contains(&"wave"));
    assert!(patterns.contains(&"spiral"));
    assert!(patterns.contains(&"checkerboard"));
    assert!(patterns.contains(&"diamond"));
    assert!(patterns.contains(&"perlin"));
    assert!(patterns.contains(&"rain"));
}

#[test]
fn test_pattern_metadata() {
    for pattern_id in REGISTRY.list_patterns() {
        let metadata = REGISTRY.get_pattern(pattern_id).unwrap();

        // Test metadata completeness
        assert!(!metadata.id.is_empty());
        assert!(!metadata.name.is_empty());
        assert!(!metadata.description.is_empty());

        // Test parameter access
        let params = metadata.params();
        assert!(!params.name().is_empty());
        assert!(!params.description().is_empty());

        // Test that sub-parameters are properly defined
        let sub_params = params.sub_params();
        assert!(
            !sub_params.is_empty(),
            "Pattern {} should have parameters",
            pattern_id
        );

        // Test each sub-parameter
        for param in sub_params {
            assert!(!param.name().is_empty());
            assert!(!param.description().is_empty());
            assert!(!param.default_value().is_empty());
        }
    }
}

#[test]
fn test_pattern_parameter_creation() {
    for pattern_id in REGISTRY.list_patterns() {
        // Test default parameter creation
        let params = REGISTRY.create_pattern_params(pattern_id).unwrap();

        // Verify pattern type matches ID
        match (pattern_id, &params) {
            ("horizontal", PatternParams::Horizontal(_)) => (),
            ("diagonal", PatternParams::Diagonal(_)) => (),
            ("plasma", PatternParams::Plasma(_)) => (),
            ("ripple", PatternParams::Ripple(_)) => (),
            ("wave", PatternParams::Wave(_)) => (),
            ("spiral", PatternParams::Spiral(_)) => (),
            ("checkerboard", PatternParams::Checkerboard(_)) => (),
            ("diamond", PatternParams::Diamond(_)) => (),
            ("perlin", PatternParams::Perlin(_)) => (),
            ("rain", PatternParams::PixelRain(_)) => (),
            ("fire", PatternParams::Fire(_)) => (),
            ("aurora", PatternParams::Aurora(_)) => (),
            _ => panic!("Unexpected pattern type for {}", pattern_id),
        }
    }
}

#[test]
fn test_parameter_validation() {
    let test_cases = vec![
        ("horizontal", "invert=true"),
        ("diagonal", "angle=45,frequency=1.0"),
        (
            "plasma",
            "complexity=3.0,scale=1.5,frequency=1.0,blend_mode=add",
        ),
        (
            "ripple",
            "center_x=0.5,center_y=0.5,wavelength=1.0,damping=0.5,frequency=1.0",
        ),
        (
            "wave",
            "amplitude=1.0,frequency=2.0,phase=0.0,offset=0.5,base_freq=1.0",
        ),
        (
            "spiral",
            "density=1.0,rotation=180,expansion=1.0,clockwise=true,frequency=1.0",
        ),
        ("checkerboard", "size=2,blur=0.1,rotation=45,scale=1.0"),
        (
            "diamond",
            "size=1.0,offset=0.5,sharpness=2.0,rotation=45,speed=1.0,mode=zoom",
        ),
        ("perlin", "octaves=4,persistence=0.5,scale=1.0,seed=0"),
        (
            "rain",
            "speed=1.0,density=1.0,length=3.0,glitch=true,glitch_freq=1.0",
        ),
    ];

    for (pattern_id, valid_params) in test_cases {
        // Test valid parameters
        assert!(
            REGISTRY.validate_params(pattern_id, valid_params).is_ok(),
            "Valid parameters rejected for {}: {}",
            pattern_id,
            valid_params
        );

        // Test invalid parameters
        assert!(
            REGISTRY
                .validate_params(pattern_id, "invalid=value")
                .is_err(),
            "Invalid parameters accepted for {}",
            pattern_id
        );
    }
}

#[test]
fn test_parameter_parsing() {
    let test_cases = vec![
        ("horizontal", "invert=true"),
        ("diagonal", "angle=45,frequency=1.0"),
        ("plasma", "complexity=3.0,scale=1.5"),
        ("ripple", "center_x=0.5,center_y=0.5"),
        ("wave", "amplitude=1.0,frequency=2.0"),
        ("spiral", "density=1.0,rotation=180"),
        ("checkerboard", "size=2,blur=0.1"),
        ("diamond", "size=1.0,offset=0.5"),
        ("perlin", "octaves=4,persistence=0.5"),
        ("rain", "speed=1.0,density=1.0"),
    ];

    for (pattern_id, params) in test_cases {
        let result = REGISTRY.parse_params(pattern_id, params);
        assert!(
            result.is_ok(),
            "Failed to parse valid parameters for {}: {}",
            pattern_id,
            params
        );
    }
}

#[test]
fn test_invalid_pattern_handling() {
    let invalid_pattern = "nonexistent";

    assert!(REGISTRY.get_pattern(invalid_pattern).is_none());
    assert!(REGISTRY.create_pattern_params(invalid_pattern).is_none());
    assert!(REGISTRY
        .validate_params(invalid_pattern, "param=value")
        .is_err());
    assert!(REGISTRY
        .parse_params(invalid_pattern, "param=value")
        .is_err());
}
