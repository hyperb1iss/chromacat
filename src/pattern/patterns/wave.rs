use std::f64::consts::PI;
use std::any::Any;
use crate::pattern::params::{PatternParam, ParamType};
use crate::define_param;

define_param!(num Wave, AmplitudeParam, "Wave height", 0.1, 2.0, 1.0);
define_param!(num Wave, FrequencyParam, "Number of waves", 0.1, 5.0, 1.0);
define_param!(num Wave, PhaseParam, "Phase shift", 0.0, 6.28318530718, 0.0);
define_param!(num Wave, OffsetParam, "Vertical offset", 0.0, 1.0, 0.5);
define_param!(num Wave, BaseFreqParam, "Animation speed multiplier", 0.1, 10.0, 1.0);

#[derive(Debug, Clone)]
pub struct WaveParams {
    pub amplitude: f64,
    pub frequency: f64,
    pub phase: f64,
    pub offset: f64,
    pub base_freq: f64,
}

impl WaveParams {
    const AMPLITUDE_PARAM: WaveAmplitudeParam = WaveAmplitudeParam;
    const FREQUENCY_PARAM: WaveFrequencyParam = WaveFrequencyParam;
    const PHASE_PARAM: WavePhaseParam = WavePhaseParam;
    const OFFSET_PARAM: WaveOffsetParam = WaveOffsetParam;
    const BASE_FREQ_PARAM: WaveBaseFreqParam = WaveBaseFreqParam;
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
        for param in self.sub_params() {
            if let Some(param_value) = value.split(',')
                .find(|part| part.starts_with(&format!("{}=", param.name())))
            {
                param.validate(param_value.split('=').nth(1).unwrap_or(""))?;
            }
        }
        Ok(())
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
                _ => {}
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
    pub fn wave(&self, x: usize, params: WaveParams) -> f64 {
        let x_norm = x as f64 / (self.width.max(1) - 1) as f64;
        let wave_angle = x_norm * params.frequency * params.base_freq * PI * 4.0 
            + params.phase + self.time * 2.0 * PI;
        let wave = self.utils.fast_sin(wave_angle) * params.amplitude;

        (params.offset + wave).clamp(0.0, 1.0)
    }
}
