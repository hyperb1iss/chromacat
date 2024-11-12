use crate::define_param;
use crate::pattern::params::{ParamType, PatternParam};
use std::any::Any;
use std::f64::consts::PI;

// Define parameters with proper CLI names and bounds
define_param!(num Aurora, IntensityParam, "intensity", "Overall brightness and contrast of the aurora", 0.1, 2.0, 1.0);
define_param!(num Aurora, SpeedParam, "speed", "Rate of aurora movement and animation", 0.1, 5.0, 1.0);
define_param!(num Aurora, WavinessParam, "waviness", "Intensity of wave-like distortions", 0.1, 2.0, 1.0);
define_param!(num Aurora, LayersParam, "layers", "Number of overlapping aurora curtains", 1.0, 5.0, 3.0);
define_param!(num Aurora, HeightParam, "height", "Vertical thickness of aurora bands", 0.1, 1.0, 0.5);
define_param!(num Aurora, SpreadParam, "spread", "Vertical spacing between bands", 0.1, 1.0, 0.3);

/// Configuration parameters for the Aurora Borealis effect
#[derive(Debug, Clone)]
pub struct AuroraParams {
    /// Controls overall brightness and contrast (0.1-2.0)
    pub intensity: f64,
    /// Controls animation speed (0.1-5.0)
    pub speed: f64,
    /// Controls wave distortion amount (0.1-2.0)
    pub waviness: f64,
    /// Number of overlapping aurora curtains (1-5)
    pub layers: u32,
    /// Controls vertical thickness of bands (0.1-1.0)
    pub height: f64,
    /// Controls vertical spacing between bands (0.1-1.0)
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
        "Aurora Borealis effect with flowing curtains of light"
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
    /// Generates an Aurora Borealis effect with flowing curtains of light
    ///
    /// The effect simulates the natural phenomenon using multiple overlapping layers
    /// of animated waves with dynamic distortion and shimmer effects.
    #[inline(always)]
    pub fn aurora(&self, x_norm: f64, y_norm: f64, params: AuroraParams) -> f64 {
        // Time variables without wrapping
        let base_time = self.time * params.speed * 0.35;
        let time = base_time;
        let time_slow = base_time * 0.23;
        let time_very_slow = base_time * 0.11;

        // Map y_norm to use full screen height
        let y_pos = y_norm + 0.5; // Map y_norm from [-0.5, 0.5] to [0.0, 1.0]

        let x_pos = x_norm + 0.5;

        // Cache periodic values for repeated use
        let (base_sin_time, base_cos_time) = {
            let sin_val = self.utils.fast_sin(time_slow);
            let cos_val = self.utils.fast_cos(time_slow);
            (sin_val, cos_val)
        };

        // Base wave motion coordinates
        let base_wave = {
            let x = x_pos * 2.0
                + self.utils.fast_sin(time) * 0.6
                + self.utils.fast_sin(time_slow) * 0.1;
            let y = y_pos * 2.0
                + self.utils.fast_cos(time) * 0.3
                + self.utils.fast_cos(time_slow) * 0.15;
            (x, y)
        };

        let mut total_value = 0.0;
        let mut max_value = 0.0;

        let waviness_scale = params.waviness * 2.0;
        let intensity_scale = params.intensity * 1.2;

        // Generate each layer of the aurora effect
        for i in 0..params.layers {
            let layer_offset = i as f64 / params.layers as f64;
            let layer_phase = layer_offset * PI;

            // Calculate wave motion for this layer with more distinct phase offsets
            let wave = {
                let x = base_wave.0
                    + layer_offset
                    + self.utils.fast_sin(time_slow + layer_phase) * (0.4 + layer_offset * 0.2);
                let y = base_wave.1
                    + layer_offset
                    + self.utils.fast_cos(time_slow + layer_phase) * (0.2 + layer_offset * 0.3);
                (x, y)
            };

            // Introduce turbulence for folding effect
            let turbulence = self.utils.fractal_noise(wave.0, wave.1, 4) * params.waviness;

            // Generate flow distortion using layered noise
            let flow = {
                // Noise coordinates without wrapping
                let noise_x = wave.0 * waviness_scale * (0.8 + layer_offset * 0.3);
                let noise_y = wave.1 * waviness_scale * (0.8 + layer_offset * 0.2);
                let primary = self.utils.noise2d(noise_x, noise_y);

                // Detail noise coordinates
                let detail_x =
                    wave.0 * waviness_scale * 1.5 + self.utils.fast_sin(time_very_slow) * 0.1;
                let detail_y =
                    wave.1 * waviness_scale * 1.5 + self.utils.fast_cos(time_very_slow) * 0.1;
                let detail = self.utils.noise2d(detail_x, detail_y);

                (primary * 2.0 - 1.0)
                    + detail * 0.3 * (1.0 + base_sin_time * 0.2)
                    + turbulence * 0.1
            };

            // Calculate vertical band shape with folding effect
            let band = {
                // Adjust center to distribute across screen height
                let center = 0.5 - layer_offset * params.spread - base_sin_time * 0.03;
                let fold = self.utils.noise2d(wave.0 * 0.5, wave.1 * 0.5 + time) * 0.2;
                let y_wave = y_pos + flow * 0.25 * params.waviness + fold;
                let pos = (y_wave - center) / (params.height * 2.0);
                (-pos * pos * 2.0).exp()
            };

            // Generate horizontal wave pattern with turbulence
            let wave_value = {
                let x_wave = x_pos + flow * 0.1 * (1.0 - layer_offset * 0.3) + turbulence * 0.05;
                let phase = x_wave * 3.0 + time + layer_phase + turbulence * 0.2;
                let base = self.utils.fast_sin(phase) * 0.5 + 0.5;
                base * (1.0
                    + self
                        .utils
                        .fast_sin(time_slow * 1.2 + layer_phase + turbulence)
                        * 0.2)
            };

            // Add curtain-like variation
            let curtain = self
                .utils
                .fast_sin(x_pos * 2.5 + y_pos * 0.3 + flow * 0.1 + time_slow + layer_phase)
                * 0.4
                + 0.6;

            let intensity = intensity_scale * (1.0 - layer_offset * 0.1) * (1.0 + curtain * 0.5);

            // Add shimmer and pulsing effects
            let modulation = {
                let pulse = self
                    .utils
                    .fast_sin(time_slow * (1.5 + layer_offset * 0.5) + layer_phase)
                    * (0.25 + layer_offset * 0.1)
                    + 0.85;
                let shimmer = {
                    let shimmer_x =
                        x_pos * (10.0 + layer_offset * 2.0) + self.utils.fast_sin(time * 0.3) * 0.2;
                    let shimmer_y =
                        y_pos * (10.0 + layer_offset * 2.0) + self.utils.fast_cos(time * 0.3) * 0.2;
                    self.utils.noise2d(shimmer_x, shimmer_y) * 0.15 + 0.85
                };
                pulse * shimmer
            };

            let layer_value = band * wave_value * intensity * modulation;
            total_value += layer_value;
            max_value += intensity;
        }

        // Normalize and apply contrast adjustment
        if max_value > 0.0 {
            let base_result = (total_value / max_value) * params.intensity;
            let contrast = 1.2 + base_cos_time * 0.1;
            (0.5 + (base_result - 0.5) * contrast).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}
