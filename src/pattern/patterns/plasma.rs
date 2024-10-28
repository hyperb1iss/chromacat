use std::f64::consts::PI;
use std::any::Any;
use crate::pattern::params::{PatternParam, ParamType};
use crate::define_param;

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

    fn as_any(&self) -> &dyn Any { self }
}

impl super::Patterns {
    /// Generates a plasma effect pattern
    pub fn plasma(&self, x_norm: f64, y_norm: f64, params: PlasmaParams) -> f64 {
        let time = self.time * PI;

        // Convert back to 0-1 range for plasma calculations
        let x_pos = x_norm + 0.5;
        let y_pos = y_norm + 0.5;

        let cx = 0.5 + 0.4 * self.utils.fast_sin(time * 0.4);
        let cy = 0.5 + 0.4 * self.utils.fast_cos(time * 0.43);

        let base_freq = params.frequency * params.scale * 2.0;
        let mut sum = 0.0;
        let mut divisor = 0.0;

        let dx1 = x_pos - cx;
        let dy1 = y_pos - cy;
        let dist1 = (dx1 * dx1 + dy1 * dy1).sqrt();
        sum += self.utils.fast_sin(dist1 * 8.0 * base_freq + time * 0.6) * 1.2;
        divisor += 1.2;

        sum += self.utils.fast_sin(x_pos * 5.0 * base_freq + time * 0.4) * 0.8;
        sum += self.utils.fast_sin(y_pos * 5.0 * base_freq + time * 0.47) * 0.8;
        divisor += 1.6;

        let angle = time * 0.2;
        let rx = x_pos * self.utils.fast_cos(angle) - y_pos * self.utils.fast_sin(angle);
        let ry = x_pos * self.utils.fast_sin(angle) + y_pos * self.utils.fast_cos(angle);
        sum += self.utils.fast_sin((rx + ry) * 4.0 * base_freq) * 1.0;
        divisor += 1.0;

        let dx2 = x_pos - 0.5;
        let dy2 = y_pos - 0.5;
        let angle2 = dy2.atan2(dx2) + time * 0.3;
        let dist2 = (dx2 * dx2 + dy2 * dy2).sqrt() * 6.0;
        sum += self.utils.fast_sin(dist2 + angle2 * 2.0) * 0.8;
        divisor += 0.8;

        for i in 0..params.complexity as u32 {
            let fi = i as f64;
            let speed = 0.2 + fi * 0.04;

            let cx = 0.5 + 0.3 * self.utils.fast_sin(time * speed);
            let cy = 0.5 + 0.3 * self.utils.fast_cos(time * speed + PI * 0.3);

            let dx = x_pos - cx;
            let dy = y_pos - cy;
            let dist = (dx * dx + dy * dy).sqrt();

            let freq = (3.0 + fi) * base_freq;
            sum += self.utils.fast_sin(dist * freq + time * (0.4 + fi * 0.1)) * (1.2 / (fi + 1.0));
            divisor += 1.0 / (fi + 1.0);
        }

        let normalized = (sum / divisor) * 1.2;
        (self.utils.fast_sin(normalized * PI * 0.8) + 1.0) * 0.5
    }
}
