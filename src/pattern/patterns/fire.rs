use crate::define_param;
use crate::pattern::params::{ParamType, PatternParam};
use std::any::Any;

// Define parameters with proper CLI names and bounds
define_param!(num Fire, IntensityParam, "intensity", "Controls the brightness and strength of the flames", 0.1, 2.0, 1.0);
define_param!(num Fire, SpeedParam, "speed", "Controls the animation speed of the flames", 0.1, 5.0, 1.0);
define_param!(num Fire, TurbulenceParam, "turbulence", "Controls the amount of flame distortion and detail", 0.0, 1.0, 0.5);
define_param!(num Fire, HeightParam, "height", "Controls the maximum height of the flames", 0.1, 2.0, 1.0);
define_param!(bool Fire, WindParam, "wind", "Enables horizontal wind effect on flames", true);
define_param!(num Fire, WindStrengthParam, "wind_strength", "Controls the intensity of the wind effect", 0.0, 1.0, 0.3);

/// Parameters for configuring the fire pattern effect.
/// Creates a dynamic flame simulation with configurable properties
/// including intensity, movement speed, turbulence, and wind effects.
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
                    return Err(format!("Invalid parameter name: {invalid_param}"));
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
    /// Generates a dynamic fire pattern effect.
    ///
    /// # Arguments
    /// * `x_norm` - Normalized x coordinate (-0.5 to 0.5)
    /// * `y_norm` - Normalized y coordinate (-0.5 to 0.5)
    /// * `params` - Configuration parameters for the fire effect
    ///
    /// # Returns
    /// A value between 0.0 and 1.0 representing the fire intensity at the given point
    #[inline(always)]
    pub fn fire(&self, x_norm: f64, y_norm: f64, params: FireParams) -> f64 {
        // Calculate time-based animation value
        let time = self.time * params.speed;

        // Transform coordinates to 0.0-1.0 range, with y inverted for upward flames
        let x_pos = x_norm + 0.5;

        // Handle y coordinate:
        // - For static preview (time == 0.0): create repeating pattern
        // - For animation: invert y so flames rise upward
        let y_pos = if self.time == 0.0 {
            (y_norm + 0.5).rem_euclid(0.3) * 3.0
        } else {
            1.0 - (y_norm + 0.5)
        };

        // Skip calculation if point is above maximum flame height
        if y_pos > params.height {
            return 0.0;
        }

        // Calculate base intensity that decreases with height
        let base_intensity = (1.0 - y_pos / params.height).powf(0.35);

        // Generate multi-layered noise for realistic flame movement
        let turbulence = {
            // Large-scale flame movement
            let noise1 = self
                .utils
                .noise2d(x_pos * 6.0 + time * 2.0, y_pos * 6.0 + time * 3.0);
            // Medium-scale detail
            let noise2 = self
                .utils
                .noise2d(x_pos * 12.0 - time * 1.5, y_pos * 12.0 + time * 2.5);
            // Fine detail for added realism
            let noise3 = self
                .utils
                .noise2d(x_pos * 18.0 + time * 3.0, y_pos * 15.0 - time * 4.0);

            // Combine noises with emphasis on vertical movement
            let combined = noise1 * 0.5 + noise2 * 0.3 + noise3 * 0.2;

            // Make turbulence more pronounced in vertical direction
            combined * params.turbulence * (1.0 + y_pos * 0.5)
        };

        // Calculate final intensity with enhanced sharpness
        let mut intensity = base_intensity * (1.0 + turbulence);

        // Add sharper peaks
        intensity = intensity.powf(0.8);

        // Add more defined hot spots near the bottom
        if y_pos < 0.3 {
            let spot_noise = self
                .utils
                .noise2d(x_pos * 15.0 + time * 4.0, y_pos * 10.0 - time * 3.0);

            if spot_noise > 0.5 {
                intensity = intensity.max(0.85);
            }
        }

        // Rest of the color mapping remains the same
        intensity = match intensity {
            i if i < 0.2 => i,
            i if i < 0.4 => 0.2 + (i - 0.2) * 2.0,
            i if i < 0.6 => 0.4 + (i - 0.4) * 2.0,
            i if i < 0.8 => 0.6 + (i - 0.6) * 2.0,
            i => 0.8 + (i - 0.8) * 2.0,
        };

        // Add wind effect with sharper transitions
        if params.wind {
            let wind_time = time * 1.5;
            let wind_offset = self.utils.noise2d(x_pos + wind_time, y_pos * 2.5)
                * params.wind_strength
                * y_pos.powf(0.8);

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
