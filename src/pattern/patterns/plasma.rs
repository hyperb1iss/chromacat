use crate::define_param;
use crate::pattern::params::{ParamType, PatternParam};
use std::any::Any;
use std::f64::consts::PI;

/// Blending modes for plasma effect
#[derive(Debug, Clone, Copy)]
pub enum PlasmaBlendMode {
    /// Standard additive blending
    Additive,
    /// Multiplicative blending
    Multiply,
    /// Maximum value blending
    Maximum,
}

impl Default for PlasmaBlendMode {
    fn default() -> Self {
        Self::Additive
    }
}

define_param!(num Plasma, ComplexityParam, "complexity", "Number of sine wave components", 1.0, 10.0, 3.0);
define_param!(num Plasma, ScaleParam, "scale", "Scale of the effect", 0.1, 5.0, 1.0);
define_param!(num Plasma, FrequencyParam, "frequency", "Animation speed", 0.1, 10.0, 1.0);
define_param!(enum Plasma, BlendModeParam, "blend_mode", "Color blending mode", &["add", "multiply", "max"], "add");

/// Parameters for configuring plasma pattern effects
#[derive(Debug, Clone)]
pub struct PlasmaParams {
    /// Number of sine wave components (1.0-10.0)
    pub complexity: f64,
    /// Scale of the effect (0.1-5.0)
    pub scale: f64,
    /// Animation speed multiplier
    pub frequency: f64,
    /// Color blending mode
    pub blend_mode: PlasmaBlendMode,
}

impl PlasmaParams {
    const COMPLEXITY_PARAM: PlasmaComplexityParam = PlasmaComplexityParam;
    const SCALE_PARAM: PlasmaScaleParam = PlasmaScaleParam;
    const FREQUENCY_PARAM: PlasmaFrequencyParam = PlasmaFrequencyParam;
    const BLEND_MODE_PARAM: PlasmaBlendModeParam = PlasmaBlendModeParam;
}

impl Default for PlasmaParams {
    fn default() -> Self {
        Self {
            complexity: 3.0,
            scale: 1.0,
            frequency: 1.0,
            blend_mode: PlasmaBlendMode::default(),
        }
    }
}

// Use the validate macro to implement validation
define_param!(validate PlasmaParams,
    COMPLEXITY_PARAM: PlasmaComplexityParam,
    SCALE_PARAM: PlasmaScaleParam,
    FREQUENCY_PARAM: PlasmaFrequencyParam,
    BLEND_MODE_PARAM: PlasmaBlendModeParam
);

impl PatternParam for PlasmaParams {
    fn name(&self) -> &'static str {
        "plasma"
    }

    fn description(&self) -> &'static str {
        "Psychedelic plasma effect with multiple wave components"
    }

    fn param_type(&self) -> ParamType {
        ParamType::Composite
    }

    fn default_value(&self) -> String {
        format!(
            "complexity={},scale={},frequency={},blend_mode={}",
            self.complexity,
            self.scale,
            self.frequency,
            match self.blend_mode {
                PlasmaBlendMode::Additive => "add",
                PlasmaBlendMode::Multiply => "multiply",
                PlasmaBlendMode::Maximum => "max",
            }
        )
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        self.validate_params(value)
    }

    fn parse(&self, value: &str) -> Result<Box<dyn PatternParam>, String> {
        let mut params = PlasmaParams::default();

        for part in value.split(',') {
            let kv: Vec<&str> = part.split('=').collect();
            if kv.len() != 2 {
                continue;
            }

            match kv[0] {
                "complexity" => {
                    Self::COMPLEXITY_PARAM.validate(kv[1])?;
                    params.complexity = kv[1].parse().unwrap();
                }
                "scale" => {
                    Self::SCALE_PARAM.validate(kv[1])?;
                    params.scale = kv[1].parse().unwrap();
                }
                "frequency" => {
                    Self::FREQUENCY_PARAM.validate(kv[1])?;
                    params.frequency = kv[1].parse().unwrap();
                }
                "blend_mode" => {
                    Self::BLEND_MODE_PARAM.validate(kv[1])?;
                    params.blend_mode = match kv[1] {
                        "add" => PlasmaBlendMode::Additive,
                        "multiply" => PlasmaBlendMode::Multiply,
                        "max" => PlasmaBlendMode::Maximum,
                        _ => return Err("Invalid blend mode".to_string()),
                    };
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
            Box::new(Self::COMPLEXITY_PARAM),
            Box::new(Self::SCALE_PARAM),
            Box::new(Self::FREQUENCY_PARAM),
            Box::new(Self::BLEND_MODE_PARAM),
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
    #[inline]
    pub fn plasma(&self, x_norm: f64, y_norm: f64, params: PlasmaParams) -> f64 {
        let time = self.time * PI;

        // Pre-calculate frequently used values
        let x_pos = x_norm + 0.5;
        let y_pos = y_norm + 0.5;
        let base_freq = params.frequency * params.scale * 2.0;

        // Pre-calculate time-based values used multiple times
        let time_sin04 = self.utils.fast_sin(time * 0.4);
        let time_cos043 = self.utils.fast_cos(time * 0.43);

        // Moving center points for wave origins
        let cx = 0.5 + 0.3 * time_sin04;
        let cy = 0.5 + 0.3 * time_cos043;

        // Calculate distance components with reduced intensity
        let dx1 = x_pos - cx;
        let dy1 = y_pos - cy;
        let dist1 = (dx1 * dx1 + dy1 * dy1).sqrt();

        // Accumulate values with minimal divisions
        let mut sum = 0.0;
        let mut divisor = 0.0;

        // First component - reduced distance influence
        sum += self.utils.fast_sin(dist1 * 6.0 * base_freq + time * 0.6) * 0.8;
        divisor += 0.8;

        // Combine similar operations - increased weight of directional waves
        let x_freq = x_pos * 5.0 * base_freq;
        let y_freq = y_pos * 5.0 * base_freq;
        sum += self.utils.fast_sin(x_freq + time * 0.4) * 1.2
            + self.utils.fast_sin(y_freq + time * 0.47) * 1.2;
        divisor += 2.4;

        // Pre-calculate rotation values - increased weight
        let angle = time * 0.2;
        let (sin_angle, cos_angle) = (self.utils.fast_sin(angle), self.utils.fast_cos(angle));
        let rx = x_pos * cos_angle - y_pos * sin_angle;
        let ry = x_pos * sin_angle + y_pos * cos_angle;
        sum += self.utils.fast_sin((rx + ry) * 4.0 * base_freq) * 1.4;
        divisor += 1.4;

        // Replace center distance calculation with diagonal waves
        sum += self
            .utils
            .fast_sin((x_pos + y_pos) * 4.0 * base_freq + time * 0.3)
            * 1.0
            + self
                .utils
                .fast_sin((x_pos - y_pos) * 4.0 * base_freq + time * 0.35)
                * 1.0;
        divisor += 2.0;

        // Complexity-based components with reduced center dependency
        let complexity = params.complexity as u32;
        if complexity > 0 {
            let mut fi = 0.0;
            for _ in 0..complexity {
                let speed = 0.2 + fi * 0.04;

                // Wider movement range for wave origins
                let cx = 0.5 + 0.4 * self.utils.fast_sin(time * speed);
                let cy = 0.5 + 0.4 * self.utils.fast_cos(time * speed + PI * 0.3);

                let dx = x_pos - cx;
                let dy = y_pos - cy;
                let dist = (dx * dx + dy * dy).sqrt();

                let freq = (2.5 + fi) * base_freq; // Reduced base frequency
                let weight = 1.0 / (fi + 1.0);
                sum += self.utils.fast_sin(dist * freq + time * (0.4 + fi * 0.1)) * weight;
                divisor += weight;

                fi += 1.0;
            }
        }

        // Final normalization with slightly reduced contrast
        let normalized = (sum / divisor) * 1.1;
        (self.utils.fast_sin(normalized * PI * 0.8) + 1.0) * 0.5
    }
}
