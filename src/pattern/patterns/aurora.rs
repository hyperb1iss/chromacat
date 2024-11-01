use crate::define_param;
use crate::pattern::params::{ParamType, PatternParam};
use std::any::Any;
use std::f64::consts::PI;

// Define parameters with proper CLI names and bounds
define_param!(num Aurora, IntensityParam, "intensity", "Intensity of the aurora", 0.1, 2.0, 1.0);
define_param!(num Aurora, SpeedParam, "speed", "Speed of aurora movement", 0.1, 5.0, 1.0);
define_param!(num Aurora, WavinessParam, "waviness", "Amount of wave distortion", 0.1, 2.0, 1.0);
define_param!(num Aurora, LayersParam, "layers", "Number of aurora layers", 1.0, 5.0, 3.0);
define_param!(num Aurora, HeightParam, "height", "Height of the aurora bands", 0.1, 1.0, 0.5);
define_param!(num Aurora, SpreadParam, "spread", "Vertical spread of bands", 0.1, 1.0, 0.3);

/// Parameters for configuring aurora pattern effects
#[derive(Debug, Clone)]
pub struct AuroraParams {
    /// Intensity of the aurora (0.1-2.0)
    pub intensity: f64,
    /// Speed of aurora movement (0.1-5.0)
    pub speed: f64,
    /// Amount of wave distortion (0.1-2.0)
    pub waviness: f64,
    /// Number of aurora layers (1-5)
    pub layers: u32,
    /// Height of the aurora bands (0.1-1.0)
    pub height: f64,
    /// Vertical spread of bands (0.1-1.0)
    pub spread: f64,
}

impl AuroraParams {
    const INTENSITY_PARAM: AuroraIntensityParam = AuroraIntensityParam;
    const SPEED_PARAM: AuroraSpeedParam = AuroraSpeedParam;
    const WAVINESS_PARAM: AuroraWavinessParam = AuroraWavinessParam;
    const LAYERS_PARAM: AuroraLayersParam = AuroraLayersParam;
    const HEIGHT_PARAM: AuroraHeightParam = AuroraHeightParam;
    const SPREAD_PARAM: AuroraSpreadParam = AuroraSpreadParam;
}

impl Default for AuroraParams {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            speed: 1.0,
            waviness: 1.0,
            layers: 3,
            height: 0.5,
            spread: 0.3,
        }
    }
}

// Use the validate macro to implement validation
define_param!(validate AuroraParams,
    INTENSITY_PARAM: AuroraIntensityParam,
    SPEED_PARAM: AuroraSpeedParam,
    WAVINESS_PARAM: AuroraWavinessParam,
    LAYERS_PARAM: AuroraLayersParam,
    HEIGHT_PARAM: AuroraHeightParam,
    SPREAD_PARAM: AuroraSpreadParam
);

impl PatternParam for AuroraParams {
    fn name(&self) -> &'static str {
        "aurora"
    }

    fn description(&self) -> &'static str {
        "Aurora Borealis effect with flowing bands of light"
    }

    fn param_type(&self) -> ParamType {
        ParamType::Composite
    }

    fn default_value(&self) -> String {
        format!(
            "intensity={},speed={},waviness={},layers={},height={},spread={}",
            self.intensity, self.speed, self.waviness, self.layers, self.height, self.spread
        )
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        self.validate_params(value)
    }

    fn parse(&self, value: &str) -> Result<Box<dyn PatternParam>, String> {
        let mut params = AuroraParams::default();

        for part in value.split(',') {
            let kv: Vec<&str> = part.split('=').collect();
            if kv.len() != 2 {
                continue;
            }

            match kv[0] {
                "intensity" => {
                    Self::INTENSITY_PARAM.validate(kv[1])?;
                    params.intensity = kv[1].parse().unwrap();
                }
                "speed" => {
                    Self::SPEED_PARAM.validate(kv[1])?;
                    params.speed = kv[1].parse().unwrap();
                }
                "waviness" => {
                    Self::WAVINESS_PARAM.validate(kv[1])?;
                    params.waviness = kv[1].parse().unwrap();
                }
                "layers" => {
                    Self::LAYERS_PARAM.validate(kv[1])?;
                    params.layers = kv[1].parse().unwrap();
                }
                "height" => {
                    Self::HEIGHT_PARAM.validate(kv[1])?;
                    params.height = kv[1].parse().unwrap();
                }
                "spread" => {
                    Self::SPREAD_PARAM.validate(kv[1])?;
                    params.spread = kv[1].parse().unwrap();
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
            Box::new(Self::INTENSITY_PARAM),
            Box::new(Self::SPEED_PARAM),
            Box::new(Self::WAVINESS_PARAM),
            Box::new(Self::LAYERS_PARAM),
            Box::new(Self::HEIGHT_PARAM),
            Box::new(Self::SPREAD_PARAM),
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
    /// Generates an Aurora Borealis effect
    #[inline(always)]
    pub fn aurora(&self, x_norm: f64, y_norm: f64, params: AuroraParams) -> f64 {
        // Pre-calculate time-based values with SIMD-friendly operations
        let base_time = self.time * params.speed;

        // Handle y coordinate differently for static vs animated mode
        let y_pos = if self.time == 0.0 {
            (y_norm + 0.5).rem_euclid(0.3) * 3.0
        } else {
            y_norm + 0.5
        };

        // Fast early exit for out-of-bounds positions
        #[allow(clippy::collapsible_if)]
        if y_pos > 0.8 + params.height || y_pos < 0.1 {
            return 0.0;
        }

        let x_pos = x_norm + 0.5;
        let time = base_time;
        let time_slow = time * 0.3;

        // Cache trigonometric values in contiguous memory
        let (base_sin_time, base_cos_time) = {
            let sin_val = self.utils.fast_sin(time_slow);
            let cos_val = self.utils.fast_cos(time_slow);
            (sin_val, cos_val)
        };

        // Pre-calculate wave bases with vectorization potential
        let base_wave = {
            let x = x_pos * 2.0 + time_slow;
            let y = y_pos * 2.0 + time_slow * 0.8;
            (x, y)
        };

        // Initialize accumulators with SIMD-friendly alignment
        let mut total_value = 0.0;
        let mut max_value = 0.0;

        // Cache common parameters to reduce memory access
        let waviness_scale = params.waviness * 2.0;
        let intensity_scale = params.intensity * 1.2; // Slightly boosted for better contrast

        // Process layers with optimized memory access pattern
        for i in 0..params.layers {
            let layer_offset = i as f64 / params.layers as f64;
            let layer_phase = layer_offset * PI;

            // Combine wave calculations for better vectorization
            let wave = {
                let x = base_wave.0 + layer_offset * time_slow * 0.8;
                let y = base_wave.1 + layer_offset * time_slow * 0.6;
                (x, y)
            };

            // Optimize noise calculations with fewer memory accesses
            let flow = {
                let primary = self.utils.noise2d(
                    wave.0 * waviness_scale * (1.0 + layer_offset * 0.5),
                    wave.1 * waviness_scale * (1.0 + layer_offset * 0.3),
                );

                let detail = self
                    .utils
                    .noise2d(wave.0 * waviness_scale * 2.0, wave.1 * waviness_scale * 2.0);

                (primary * 2.0 - 1.0) + detail * 0.5 * (1.0 + base_sin_time * 0.3)
            };

            // Optimize band calculations
            let band = {
                let center = 0.3 + layer_offset * params.spread;
                let y_wave = y_pos + flow * 0.3 * params.waviness;
                let pos = (y_wave - center) / params.height;
                (-pos * pos * 3.0).exp()
            };

            // Combine wave movements with optimized math
            let wave_value = {
                let x_wave = x_pos + flow * 0.15 * (1.0 - layer_offset * 0.3);
                let phase = x_wave * 4.0 + time + layer_phase;
                let base = self.utils.fast_sin(phase) * 0.5 + 0.5;
                base * (1.0 + self.utils.fast_sin(time_slow * 1.5 + layer_phase) * 0.3)
            };

            // Optimize curtain effect calculation
            let curtain = self
                .utils
                .fast_sin(x_pos * 3.0 + flow * 0.15 + time_slow + layer_phase)
                * 0.5
                + 0.5;

            // Combine all effects with minimal branching
            let intensity = intensity_scale * (1.0 - layer_offset * 0.2) * (1.0 + curtain * 0.5);

            // Add shimmer and pulse with optimized calculations
            let modulation = {
                let pulse = self
                    .utils
                    .fast_sin(time_slow * (1.5 + layer_offset) + layer_phase)
                    * 0.25
                    + 0.85;
                let shimmer = self
                    .utils
                    .noise2d(x_pos * 10.0 + time, y_pos * 10.0 - time * 0.5)
                    * 0.15
                    + 0.85;
                pulse * shimmer
            };

            // Accumulate values with minimal operations
            let layer_value = band * wave_value * intensity * modulation;
            total_value += layer_value;
            max_value += intensity;
        }

        // Final value calculation with optimized contrast
        if max_value > 0.0 {
            let base_result = (total_value / max_value) * params.intensity;
            let contrast = 1.2 + base_cos_time * 0.1;
            (0.5 + (base_result - 0.5) * contrast).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}