use chromacat::pattern::{PatternParam, PerlinParams};

#[test]
fn test_perlin_params_validation() {
    let params = PerlinParams::default();

    // Test valid values
    assert!(params.validate("octaves=4,persistence=0.5,scale=1.0,seed=42").is_ok());

    // Test invalid octaves
    assert!(params.validate("octaves=9,persistence=0.5,scale=1.0,seed=42").is_err());

    // Test invalid persistence
    assert!(params.validate("octaves=4,persistence=1.5,scale=1.0,seed=42").is_err());

    // Test invalid scale
    assert!(params.validate("octaves=4,persistence=0.5,scale=0.05,seed=42").is_err());

    // Test invalid format
    assert!(params.validate("octaves=4,invalid").is_err());
}

#[test]
fn test_perlin_params_parsing() {
    let params = PerlinParams::default();
    
    let parsed = params.parse("octaves=6,persistence=0.7,scale=2.0,seed=123")
        .unwrap();
    
    let perlin_params = parsed.as_any()
        .downcast_ref::<PerlinParams>()
        .expect("Failed to downcast parsed parameters");

    assert_eq!(perlin_params.octaves, 6);
    assert_eq!(perlin_params.persistence, 0.7);
    assert_eq!(perlin_params.scale, 2.0);
    assert_eq!(perlin_params.seed, 123);
}

#[test]
fn test_perlin_params_defaults() {
    let params = PerlinParams::default();
    assert_eq!(params.octaves, 4);
    assert_eq!(params.persistence, 0.5);
    assert_eq!(params.scale, 1.0);
    assert_eq!(params.seed, 0);
}

#[test]
fn test_perlin_params_bounds() {
    let params = PerlinParams::default();

    // Test octaves bounds
    assert!(params.validate("octaves=1,persistence=0.5,scale=1.0,seed=0").is_ok());
    assert!(params.validate("octaves=8,persistence=0.5,scale=1.0,seed=0").is_ok());
    assert!(params.validate("octaves=0,persistence=0.5,scale=1.0,seed=0").is_err());
    assert!(params.validate("octaves=9,persistence=0.5,scale=1.0,seed=0").is_err());

    // Test persistence bounds
    assert!(params.validate("octaves=4,persistence=0.0,scale=1.0,seed=0").is_ok());
    assert!(params.validate("octaves=4,persistence=1.0,scale=1.0,seed=0").is_ok());
    assert!(params.validate("octaves=4,persistence=-0.1,scale=1.0,seed=0").is_err());
    assert!(params.validate("octaves=4,persistence=1.1,scale=1.0,seed=0").is_err());

    // Test scale bounds
    assert!(params.validate("octaves=4,persistence=0.5,scale=0.1,seed=0").is_ok());
    assert!(params.validate("octaves=4,persistence=0.5,scale=5.0,seed=0").is_ok());
    assert!(params.validate("octaves=4,persistence=0.5,scale=0.09,seed=0").is_err());
    assert!(params.validate("octaves=4,persistence=0.5,scale=5.1,seed=0").is_err());
}
