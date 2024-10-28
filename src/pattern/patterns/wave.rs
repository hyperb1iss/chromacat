use std::f64::consts::PI;
use std::any::Any;
use crate::pattern::params::{PatternParam, ParamType};
use crate::define_param;
use std::f64::consts::TAU;

define_param!(num Wave, AmplitudeParam, "amplitude", "Wave height", 0.1, 2.0, 1.0);
define_param!(num Wave, FrequencyParam, "frequency", "Number of waves", 0.1, 5.0, 1.0);
define_param!(num Wave, PhaseParam, "phase", "Phase shift", 0.0, TAU, 0.0);
define_param!(num Wave, OffsetParam, "offset", "Vertical offset", 0.0, 1.0, 0.5);
define_param!(num Wave, BaseFreqParam, "base_freq", "Animation speed multiplier", 0.1, 10.0, 1.0);

// ... struct definition and impl blocks ...

#[derive(Debug, Clone)]
pub struct WaveParams {
    pub amplitude: f64,
    pub frequency: f64,
    pub phase: f64,
    pub offset: f64,
    pub base_freq: f64,
}

impl Default for WaveParams {
    fn default() -> Self {
        Self {
            amplitude: 1.0,
            frequency: 1.0,
            phase: 0.0,
            offset: 0.5,
            base_freq: 1.0,
        }
    }
}

impl WaveParams {
    const AMPLITUDE_PARAM: WaveAmplitudeParam = WaveAmplitudeParam;
    const FREQUENCY_PARAM: WaveFrequencyParam = WaveFrequencyParam;
    const PHASE_PARAM: WavePhaseParam = WavePhaseParam;
    const OFFSET_PARAM: WaveOffsetParam = WaveOffsetParam;
    const BASE_FREQ_PARAM: WaveBaseFreqParam = WaveBaseFreqParam;
}

// Use the validate macro to implement validation
define_param!(validate WaveParams,
    AMPLITUDE_PARAM: WaveAmplitudeParam,
    FREQUENCY_PARAM: WaveFrequencyParam,
    PHASE_PARAM: WavePhaseParam,
    OFFSET_PARAM: WaveOffsetParam,
    BASE_FREQ_PARAM: WaveBaseFreqParam
);

impl PatternParam for WaveParams {
    fn name(&self) -> &'static str {
        "wave"
    }

    fn description(&self) -> &'static str {
        "Wave pattern with configurable properties"
    }

    fn param_type(&self) -> ParamType {
        ParamType::Composite
    }

    fn default_value(&self) -> String {
        format!(
            "amplitude={},frequency={},phase={},offset={},base_freq={}",
            self.amplitude, self.frequency, self.phase, self.offset, self.base_freq
        )
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        self.validate_params(value)
    }

    fn parse(&self, value: &str) -> Result<Box<dyn PatternParam>, String> {
        let mut params = WaveParams::default();
        
        for part in value.split(',') {
            let kv: Vec<&str> = part.split('=').collect();
            if kv.len() != 2 {
                continue;
            }
            
            match kv[0] {
                "amplitude" => {
                    Self::AMPLITUDE_PARAM.validate(kv[1])?;
                    params.amplitude = kv[1].parse().unwrap();
                }
                "frequency" => {
                    Self::FREQUENCY_PARAM.validate(kv[1])?;
                    params.frequency = kv[1].parse().unwrap();
                }
                "phase" => {
                    Self::PHASE_PARAM.validate(kv[1])?;
                    params.phase = kv[1].parse().unwrap();
                }
                "offset" => {
                    Self::OFFSET_PARAM.validate(kv[1])?;
                    params.offset = kv[1].parse().unwrap();
                }
                "base_freq" => {
                    Self::BASE_FREQ_PARAM.validate(kv[1])?;
                    params.base_freq = kv[1].parse().unwrap();
                }
                invalid_param => {
                    return Err(format!("Invalid parameter name: {}", invalid_param));
                }
            }
        }
        
        Ok(Box::new(params))
    }

    fn sub_params(&self) -> Vec<Box<dyn PatternParam>> {
        vec![
            Box::new(Self::AMPLITUDE_PARAM),
            Box::new(Self::FREQUENCY_PARAM),
            Box::new(Self::PHASE_PARAM),
            Box::new(Self::OFFSET_PARAM),
            Box::new(Self::BASE_FREQ_PARAM),
        ]
    }

    fn clone_param(&self) -> Box<dyn PatternParam> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl super::Patterns {
    /// Generates a wave pattern with configurable properties
    pub fn wave(&self, x_norm: f64, y_norm: f64, params: WaveParams) -> f64 {
        // Make animation more pronounced
        let time_factor = self.time * params.frequency;
        
        // Primary wave with time-based phase shift and movement
        let wave_angle = (x_norm + 0.5) * params.frequency * PI * 2.0 + params.phase + time_factor;
        let primary_wave = self.utils.fast_sin(wave_angle) * params.amplitude;

        // Secondary wave for vertical movement
        let secondary_angle = (y_norm + 0.5) * params.frequency * PI + time_factor * 0.7;
        let secondary_wave = self.utils.fast_sin(secondary_angle) * params.amplitude * 0.3;

        // Add a traveling wave component
        let travel_wave = self.utils.fast_sin((x_norm + y_norm + time_factor * 0.5) * PI * 2.0) * 0.2;

        // Add distance-based modulation
        let dist = ((x_norm * x_norm + y_norm * y_norm).sqrt() * 4.0 + time_factor) * PI;
        let dist_mod = self.utils.fast_sin(dist) * 0.15;

        // Combine all components
        let combined = primary_wave + secondary_wave + travel_wave + dist_mod;
        
        // Apply base offset
        (params.offset + combined).clamp(0.0, 1.0)
    }
}
