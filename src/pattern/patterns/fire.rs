use crate::define_param;
use crate::pattern::params::{ParamType, PatternParam};
use std::any::Any;

// Define parameters with proper CLI names and bounds
define_param!(num Fire, IntensityParam, "intensity", "Intensity of the flames", 0.1, 2.0, 1.0);
define_param!(num Fire, SpeedParam, "speed", "Speed of flame movement", 0.1, 5.0, 1.0);
define_param!(num Fire, TurbulenceParam, "turbulence", "Amount of flame turbulence", 0.0, 1.0, 0.5);
define_param!(num Fire, HeightParam, "height", "Height of the flames", 0.1, 2.0, 1.0);
define_param!(bool Fire, WindParam, "wind", "Enable wind effect", true);
define_param!(num Fire, WindStrengthParam, "wind_strength", "Strength of wind effect", 0.0, 1.0, 0.3);

/// Parameters for configuring fire pattern effects
#[derive(Debug, Clone)]
pub struct FireParams {
    /// Intensity of the flames (0.1-2.0)
    pub intensity: f64,
    /// Speed of flame movement (0.1-5.0)
    pub speed: f64,
    /// Amount of flame turbulence (0.0-1.0)
    pub turbulence: f64,
    /// Height of the flames (0.1-2.0)
    pub height: f64,
    /// Enable wind effect
    pub wind: bool,
    /// Strength of wind effect (0.0-1.0)
    pub wind_strength: f64,
}

impl FireParams {
    const INTENSITY_PARAM: FireIntensityParam = FireIntensityParam;
    const SPEED_PARAM: FireSpeedParam = FireSpeedParam;
    const TURBULENCE_PARAM: FireTurbulenceParam = FireTurbulenceParam;
    const HEIGHT_PARAM: FireHeightParam = FireHeightParam;
    const WIND_PARAM: FireWindParam = FireWindParam;
    const WIND_STRENGTH_PARAM: FireWindStrengthParam = FireWindStrengthParam;
}

impl Default for FireParams {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            speed: 1.0,
            turbulence: 0.5,
            height: 1.0,
            wind: true,
            wind_strength: 0.3,
        }
    }
}

// Use the validate macro to implement validation
define_param!(validate FireParams,
    INTENSITY_PARAM: FireIntensityParam,
    SPEED_PARAM: FireSpeedParam,
    TURBULENCE_PARAM: FireTurbulenceParam,
    HEIGHT_PARAM: FireHeightParam,
    WIND_PARAM: FireWindParam,
    WIND_STRENGTH_PARAM: FireWindStrengthParam
);

impl PatternParam for FireParams {
    fn name(&self) -> &'static str {
        "fire"
    }

    fn description(&self) -> &'static str {
        "Dynamic fire effect with realistic flame movement"
    }

    fn param_type(&self) -> ParamType {
        ParamType::Composite
    }

    fn default_value(&self) -> String {
        format!(
            "intensity={},speed={},turbulence={},height={},wind={},wind_strength={}",
            self.intensity, self.speed, self.turbulence, self.height, self.wind, self.wind_strength
        )
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        self.validate_params(value)
    }

    fn parse(&self, value: &str) -> Result<Box<dyn PatternParam>, String> {
        let mut params = FireParams::default();

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
                "turbulence" => {
                    Self::TURBULENCE_PARAM.validate(kv[1])?;
                    params.turbulence = kv[1].parse().unwrap();
                }
                "height" => {
                    Self::HEIGHT_PARAM.validate(kv[1])?;
                    params.height = kv[1].parse().unwrap();
                }
                "wind" => {
                    Self::WIND_PARAM.validate(kv[1])?;
                    params.wind = kv[1].parse().unwrap();
                }
                "wind_strength" => {
                    Self::WIND_STRENGTH_PARAM.validate(kv[1])?;
                    params.wind_strength = kv[1].parse().unwrap();
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
            Box::new(Self::TURBULENCE_PARAM),
            Box::new(Self::HEIGHT_PARAM),
            Box::new(Self::WIND_PARAM),
            Box::new(Self::WIND_STRENGTH_PARAM),
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
    /// Generates a dynamic fire pattern effect
    #[inline(always)]
    pub fn fire(&self, x_norm: f64, y_norm: f64, params: FireParams) -> f64 {
        // Pre-calculate time-based values
        let time = self.time * params.speed;

        // Base coordinates (keep y inverted for fire rising up)
        let x_pos = x_norm + 0.5;

        // Handle y coordinate differently for static vs animated mode
        let y_pos = if self.time == 0.0 {
            // Static mode: map each line to a portion of the flame
            // This ensures the entire document shows fire effects
            (y_norm + 0.5).rem_euclid(0.3) * 3.0 // Repeat every 30% of height
        } else {
            // Animation mode: keep original mapping
            1.0 - (y_norm + 0.5)
        };

        // Early exit if above max height
        if y_pos > params.height {
            return 0.0;
        }

        // Create base fire shape
        let base_intensity = (1.0 - y_pos / params.height).powf(0.5);

        // Add noise-based turbulence
        let turbulence = {
            let noise1 = self
                .utils
                .noise2d(x_pos * 4.0 + time * 2.0, y_pos * 4.0 + time * 3.0);
            let noise2 = self
                .utils
                .noise2d(x_pos * 8.0 - time * 1.5, y_pos * 8.0 + time * 2.5);

            (noise1 * 0.6 + noise2 * 0.4) * params.turbulence
        };

        // Calculate final intensity with turbulence
        let mut intensity = base_intensity * (1.0 + turbulence);

        // Add hot spots near the bottom
        if y_pos < 0.3 {
            let spot_noise = self
                .utils
                .noise2d(x_pos * 12.0 + time * 4.0, y_pos * 8.0 - time * 3.0);

            if spot_noise > 0.6 {
                intensity = intensity.max(0.8);
            }
        }

        // Map intensity to color ranges
        intensity = match intensity {
            i if i < 0.2 => i,                     // Keep dark reds
            i if i < 0.4 => 0.2 + (i - 0.2) * 2.0, // Expand bright reds
            i if i < 0.6 => 0.4 + (i - 0.4) * 2.0, // Expand oranges
            i if i < 0.8 => 0.6 + (i - 0.6) * 2.0, // Expand yellows
            i => 0.8 + (i - 0.8) * 2.0,            // Expand whites
        };

        // Add wind effect
        if params.wind {
            let wind_time = time * 1.5;
            let wind_offset =
                self.utils.noise2d(x_pos + wind_time, y_pos * 2.0) * params.wind_strength * y_pos;

            // Sample intensity at wind-offset position
            let x_sample = (x_pos + wind_offset).rem_euclid(1.0);
            intensity = self.fire(
                x_sample - 0.5,
                y_norm,
                FireParams {
                    wind: false,
                    ..params
                },
            );
        }

        intensity.clamp(0.0, 1.0)
    }
}
