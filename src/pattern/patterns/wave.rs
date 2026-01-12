use crate::define_param;
use crate::pattern::params::{ParamType, PatternParam};
use std::any::Any;
use std::f64::consts::PI;
use std::f64::consts::TAU;

define_param!(num Wave, AmplitudeParam, "amplitude", "Wave height", 0.1, 2.0, 1.0);
define_param!(num Wave, FrequencyParam, "frequency", "Number of waves", 0.1, 5.0, 1.0);
define_param!(num Wave, PhaseParam, "phase", "Phase shift", 0.0, TAU, 0.0);
define_param!(num Wave, PhaseDriftParam, "phase_drift", "Gradual phase shift over time", 0.0, 2.0, 0.0);
define_param!(num Wave, OffsetParam, "offset", "Vertical offset", 0.0, 1.0, 0.5);
define_param!(num Wave, BaseFreqParam, "base_freq", "Animation speed multiplier", 0.1, 10.0, 1.0);

// ... struct definition and impl blocks ...

#[derive(Debug, Clone)]
pub struct WaveParams {
    pub amplitude: f64,
    pub frequency: f64,
    pub phase: f64,
    pub phase_drift: f64,
    pub offset: f64,
    pub base_freq: f64,
}

impl Default for WaveParams {
    fn default() -> Self {
        Self {
            amplitude: 1.0,
            frequency: 1.0,
            phase: 0.0,
            phase_drift: 0.0,
            offset: 0.5,
            base_freq: 1.0,
        }
    }
}

impl WaveParams {
    const AMPLITUDE_PARAM: WaveAmplitudeParam = WaveAmplitudeParam;
    const FREQUENCY_PARAM: WaveFrequencyParam = WaveFrequencyParam;
    const PHASE_PARAM: WavePhaseParam = WavePhaseParam;
    const PHASE_DRIFT_PARAM: WavePhaseDriftParam = WavePhaseDriftParam;
    const OFFSET_PARAM: WaveOffsetParam = WaveOffsetParam;
    const BASE_FREQ_PARAM: WaveBaseFreqParam = WaveBaseFreqParam;
}

// Use the validate macro to implement validation
define_param!(validate WaveParams,
    AMPLITUDE_PARAM: WaveAmplitudeParam,
    FREQUENCY_PARAM: WaveFrequencyParam,
    PHASE_PARAM: WavePhaseParam,
    PHASE_DRIFT_PARAM: WavePhaseDriftParam,
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
            "amplitude={},frequency={},phase={},phase_drift={},offset={},base_freq={}",
            self.amplitude, self.frequency, self.phase, self.phase_drift, self.offset, self.base_freq
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
                "phase_drift" => {
                    Self::PHASE_DRIFT_PARAM.validate(kv[1])?;
                    params.phase_drift = kv[1].parse().unwrap();
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
                    return Err(format!("Invalid parameter name: {invalid_param}"));
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
            Box::new(Self::PHASE_DRIFT_PARAM),
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
    #[inline(always)]
    pub fn wave(&self, x_norm: f64, y_norm: f64, params: WaveParams) -> f64 {
        // Pre-calculate time-based values
        let time_base = self.time * params.base_freq * PI;
        let time_slow = time_base * 0.7; // Slower time factor for smoother animation

        // Calculate phase drift - gradual shift over time
        let drifted_phase = params.phase + self.time * params.phase_drift * TAU;

        // Pre-calculate trigonometric values
        let time_sin = self.utils.fast_sin(time_slow);
        let time_sin_half = self.utils.fast_sin(time_slow * 0.5);

        // Calculate base coordinates with offset
        let x_pos = x_norm + 0.5;
        let y_pos = y_norm + 0.5;

        // Add flowing motion to frequency
        let freq_mod = 1.0 + time_sin_half * 0.2;
        let wave_freq = params.frequency * freq_mod;

        // Primary wave with smooth phase shift including drift
        let wave_angle = x_pos * wave_freq * PI * 2.0 + drifted_phase + time_base;
        let primary_wave = self.utils.fast_sin(wave_angle) * params.amplitude;

        // Secondary wave with vertical movement and phase variation
        let sec_angle = y_pos * wave_freq * PI + time_slow * 0.7 + x_pos * PI * 0.5;
        let secondary_wave = self.utils.fast_sin(sec_angle) * params.amplitude * 0.3;

        // Add flowing travel wave
        let travel_phase = (x_pos + y_pos + time_slow * 0.3) * PI * 2.0;
        let travel_wave = self.utils.fast_sin(travel_phase) * 0.15;

        // Add distance-based modulation with smooth falloff
        let dist_sq = x_norm * x_norm + y_norm * y_norm;
        let dist_factor = (-dist_sq * 2.0).exp();
        let dist_angle = (dist_sq.sqrt() * 4.0 + time_slow) * PI;
        let dist_mod = self.utils.fast_sin(dist_angle) * 0.12 * dist_factor;

        // Add subtle pulsing effect
        let pulse = time_sin * 0.08 * (1.0 - dist_sq).max(0.0);

        // Combine all components with smooth transitions
        let combined = primary_wave + secondary_wave + travel_wave + dist_mod + pulse;

        // Apply base offset with smooth clamping
        let result = params.offset + combined;
        result.clamp(0.0, 1.0)
    }
}
