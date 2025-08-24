use chromacat::pattern::{PatternParam, WaveParams};

#[test]
fn test_wave_params_validation() {
    let params = WaveParams::default();

    // Test valid values
    assert!(params
        .validate("amplitude=1.0,frequency=2.0,phase=0.0,offset=0.5,base_freq=1.0")
        .is_ok());

    // Test invalid amplitude
    assert!(params
        .validate("amplitude=2.1,frequency=2.0,phase=0.0,offset=0.5,base_freq=1.0")
        .is_err());

    // Test invalid frequency
    assert!(params
        .validate("amplitude=1.0,frequency=5.1,phase=0.0,offset=0.5,base_freq=1.0")
        .is_err());

    // Test invalid phase
    assert!(params
        .validate("amplitude=1.0,frequency=2.0,phase=7.0,offset=0.5,base_freq=1.0")
        .is_err());

    // Test invalid offset
    assert!(params
        .validate("amplitude=1.0,frequency=2.0,phase=0.0,offset=1.1,base_freq=1.0")
        .is_err());

    // Test invalid base_freq
    assert!(params
        .validate("amplitude=1.0,frequency=2.0,phase=0.0,offset=0.5,base_freq=10.1")
        .is_err());

    // Test invalid format
    assert!(params.validate("amplitude=1.0,invalid").is_err());
}

#[test]
fn test_wave_params_parsing() {
    let params = WaveParams::default();

    let parsed = params
        .parse("amplitude=1.5,frequency=3.0,phase=1.5,offset=0.7,base_freq=2.0")
        .unwrap();

    let wave_params = parsed
        .as_any()
        .downcast_ref::<WaveParams>()
        .expect("Failed to downcast parsed parameters");

    assert_eq!(wave_params.amplitude, 1.5);
    assert_eq!(wave_params.frequency, 3.0);
    assert_eq!(wave_params.phase, 1.5);
    assert_eq!(wave_params.offset, 0.7);
    assert_eq!(wave_params.base_freq, 2.0);
}

#[test]
fn test_wave_params_defaults() {
    let params = WaveParams::default();
    assert_eq!(params.amplitude, 1.0);
    assert_eq!(params.frequency, 1.0);
    assert_eq!(params.phase, 0.0);
    assert_eq!(params.offset, 0.5);
    assert_eq!(params.base_freq, 1.0);
}

#[test]
fn test_wave_params_bounds() {
    let params = WaveParams::default();

    // Test amplitude bounds
    assert!(params
        .validate("amplitude=0.1,frequency=1.0,phase=0.0,offset=0.5,base_freq=1.0")
        .is_ok());
    assert!(params
        .validate("amplitude=2.0,frequency=1.0,phase=0.0,offset=0.5,base_freq=1.0")
        .is_ok());
    assert!(params
        .validate("amplitude=0.09,frequency=1.0,phase=0.0,offset=0.5,base_freq=1.0")
        .is_err());
    assert!(params
        .validate("amplitude=2.1,frequency=1.0,phase=0.0,offset=0.5,base_freq=1.0")
        .is_err());

    // Test frequency bounds
    assert!(params
        .validate("amplitude=1.0,frequency=0.1,phase=0.0,offset=0.5,base_freq=1.0")
        .is_ok());
    assert!(params
        .validate("amplitude=1.0,frequency=5.0,phase=0.0,offset=0.5,base_freq=1.0")
        .is_ok());
    assert!(params
        .validate("amplitude=1.0,frequency=0.09,phase=0.0,offset=0.5,base_freq=1.0")
        .is_err());
    assert!(params
        .validate("amplitude=1.0,frequency=5.1,phase=0.0,offset=0.5,base_freq=1.0")
        .is_err());
}
