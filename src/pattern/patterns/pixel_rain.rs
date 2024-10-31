use crate::define_param;
use crate::pattern::params::{ParamType, PatternParam};
use std::any::Any;
use std::f64::consts::PI;

// Define parameters with proper CLI names and bounds
define_param!(num PixelRain, SpeedParam, "speed", "Speed of falling pixels", 0.1, 5.0, 1.0);
define_param!(num PixelRain, DensityParam, "density", "Density of falling pixels", 0.1, 2.0, 1.0);
define_param!(num PixelRain, LengthParam, "length", "Length of pixel trails", 1.0, 10.0, 3.0);
define_param!(bool PixelRain, GlitchParam, "glitch", "Enable glitch effects", true);
define_param!(num PixelRain, GlitchFreqParam, "glitch_freq", "Frequency of glitch effects", 0.1, 5.0, 1.0);
define_param!(num PixelRain, SpeedVarParam, "speed_var", "Speed variation between streams", 0.0, 1.0, 0.5);

/// Parameters for configuring pixel rain pattern effects
#[derive(Debug, Clone)]
pub struct PixelRainParams {
    /// Speed of falling pixels (0.1-5.0)
    pub speed: f64,
    /// Density of falling pixels (0.1-2.0)
    pub density: f64,
    /// Length of pixel trails (1.0-10.0)
    pub length: f64,
    /// Enable glitch effects
    pub glitch: bool,
    /// Frequency of glitch effects (0.1-5.0)
    pub glitch_freq: f64,
    /// Speed variation between streams (0.0-1.0)
    pub speed_var: f64,
}

impl PixelRainParams {
    const SPEED_PARAM: PixelRainSpeedParam = PixelRainSpeedParam;
    const DENSITY_PARAM: PixelRainDensityParam = PixelRainDensityParam;
    const LENGTH_PARAM: PixelRainLengthParam = PixelRainLengthParam;
    const GLITCH_PARAM: PixelRainGlitchParam = PixelRainGlitchParam;
    const GLITCH_FREQ_PARAM: PixelRainGlitchFreqParam = PixelRainGlitchFreqParam;
    const SPEED_VAR_PARAM: PixelRainSpeedVarParam = PixelRainSpeedVarParam;
}

impl Default for PixelRainParams {
    fn default() -> Self {
        Self {
            speed: 1.0,
            density: 1.0,
            length: 3.0,
            glitch: true,
            glitch_freq: 1.0,
            speed_var: 0.5, // Default speed variation
        }
    }
}

// Use the validate macro to implement validation
define_param!(validate PixelRainParams,
    SPEED_PARAM: PixelRainSpeedParam,
    DENSITY_PARAM: PixelRainDensityParam,
    LENGTH_PARAM: PixelRainLengthParam,
    GLITCH_PARAM: PixelRainGlitchParam,
    GLITCH_FREQ_PARAM: PixelRainGlitchFreqParam,
    SPEED_VAR_PARAM: PixelRainSpeedVarParam
);

impl PatternParam for PixelRainParams {
    fn name(&self) -> &'static str {
        "pixel_rain"
    }

    fn description(&self) -> &'static str {
        "Matrix-style digital rain effect"
    }

    fn param_type(&self) -> ParamType {
        ParamType::Composite
    }

    fn default_value(&self) -> String {
        format!(
            "speed={},density={},length={},glitch={},glitch_freq={},speed_var={}",
            self.speed, self.density, self.length, self.glitch, self.glitch_freq, self.speed_var
        )
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        self.validate_params(value)
    }

    fn parse(&self, value: &str) -> Result<Box<dyn PatternParam>, String> {
        let mut params = PixelRainParams::default();

        for part in value.split(',') {
            let kv: Vec<&str> = part.split('=').collect();
            if kv.len() != 2 {
                continue;
            }

            match kv[0] {
                "speed" => {
                    Self::SPEED_PARAM.validate(kv[1])?;
                    params.speed = kv[1].parse().unwrap();
                }
                "density" => {
                    Self::DENSITY_PARAM.validate(kv[1])?;
                    params.density = kv[1].parse().unwrap();
                }
                "length" => {
                    Self::LENGTH_PARAM.validate(kv[1])?;
                    params.length = kv[1].parse().unwrap();
                }
                "glitch" => {
                    Self::GLITCH_PARAM.validate(kv[1])?;
                    params.glitch = kv[1].parse().unwrap();
                }
                "glitch_freq" => {
                    Self::GLITCH_FREQ_PARAM.validate(kv[1])?;
                    params.glitch_freq = kv[1].parse().unwrap();
                }
                "speed_var" => {
                    Self::SPEED_VAR_PARAM.validate(kv[1])?;
                    params.speed_var = kv[1].parse().unwrap();
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
            Box::new(Self::SPEED_PARAM),
            Box::new(Self::DENSITY_PARAM),
            Box::new(Self::LENGTH_PARAM),
            Box::new(Self::GLITCH_PARAM),
            Box::new(Self::GLITCH_FREQ_PARAM),
            Box::new(Self::SPEED_VAR_PARAM),
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
    /// Generates a Matrix-style digital rain pattern effect
    #[inline(always)]
    pub fn pixel_rain(&self, x_norm: f64, y_norm: f64, params: PixelRainParams) -> f64 {
        // Pre-calculate time-based values
        let base_time = self.time * params.speed;

        // Calculate base coordinates
        let x_pos = x_norm + 0.5;
        let y_pos = y_norm + 0.5;

        // More efficient column calculations
        let column_width = 0.015 * params.density;
        let column_x = (x_pos / column_width).floor() * column_width;
        let column_index = (column_x / column_width) as i32;
        let dx = (x_pos - column_x - column_width / 2.0).abs() / column_width;

        // Early exit if not near a column
        if dx >= 0.5 {
            return 0.0;
        }

        // More efficient hash calculations
        let column_hash = self.utils.hash(column_index, 0) as f64 / 255.0;
        let secondary_hash = self.utils.hash(column_index * 31, 0) as f64 / 255.0;
        let tertiary_hash = self.utils.hash(column_index * 73, 0) as f64 / 255.0;

        // Pre-calculate speed factors once
        let speed_group = (column_hash * 4.0).floor() / 4.0;
        let speed_factor = 0.05
            + (speed_group * 0.7 + secondary_hash * 0.3 + tertiary_hash * 0.2)
                * 0.8
                * params.speed_var;

        // Calculate stream position
        let stream_time = base_time * speed_factor + column_hash * 2000.0 + secondary_hash * 1000.0;
        let y_stream = stream_time - stream_time.floor();

        // Calculate trail parameters
        let char_spacing = 0.1;
        let trail_length = params.length * (1.2 - speed_factor).max(0.3);
        let num_chars = (trail_length * 2.0) as i32;

        let mut value: f64 = 0.0;

        // Optimize character rendering loop
        for i in 0..num_chars {
            let char_offset = i as f64 * char_spacing;
            let char_y = y_stream + char_offset;
            let wrapped_y = char_y - char_y.floor();
            let dist_y = (y_pos - wrapped_y).abs();

            if dist_y < 0.1 {
                // More efficient brightness calculation
                let char_brightness = if i == 0 {
                    1.0
                } else {
                    let fade = (1.0 - (i as f64 / num_chars as f64)).powf(1.2 + speed_factor);
                    fade * (0.7 + secondary_hash * 0.3)
                };

                // Simplified pulse calculation
                let pulse = self
                    .utils
                    .fast_sin(base_time * 2.0 + column_hash * PI * 2.0 + i as f64 * 0.5)
                    * 0.15
                    + 0.85;

                value = value.max(char_brightness * pulse * (1.0 - dx * 2.0));
            }
        }

        // Optimize glitch effects
        if params.glitch && value > 0.1 {
            let glitch_time = base_time * params.glitch_freq;
            if (glitch_time * 20.0).floor() as i32 % (3 + (column_hash * 4.0) as i32) == 0
                && secondary_hash > 0.7
            {
                value += self.utils.fast_sin(glitch_time + column_hash * PI * 4.0) * 0.3 * value;
            }

            // Simplified bright flash calculation
            if column_hash > 0.95
                && secondary_hash > 0.98
                && ((base_time * 5.0).floor() as i32 & 1) == 0
            {
                value = value.max(0.95);
            }
        }

        value.clamp(0.0, 1.0)
    }
}
