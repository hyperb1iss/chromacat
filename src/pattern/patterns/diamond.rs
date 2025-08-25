use crate::define_param;
use crate::pattern::params::{ParamType, PatternParam};
use std::any::Any;
use std::f64::consts::PI;

// First define the individual parameters with proper CLI names
define_param!(num Diamond, SizeParam, "size", "Size of diamond shapes", 0.1, 5.0, 1.0);
define_param!(num Diamond, OffsetParam, "offset", "Pattern offset", 0.0, 1.0, 0.0);
define_param!(num Diamond, SharpnessParam, "sharpness", "Edge sharpness", 0.1, 5.0, 1.0);
define_param!(num Diamond, RotationParam, "rotation", "Pattern rotation", 0.0, 360.0, 0.0);
define_param!(num Diamond, SpeedParam, "speed", "Animation speed", 0.0, 5.0, 1.0);
define_param!(enum Diamond, AnimationModeParam, "mode", "Animation mode", &["zoom", "scroll", "static"], "zoom");

/// Parameters for configuring diamond pattern effects
#[derive(Debug, Clone)]
pub struct DiamondParams {
    /// Size of diamond shapes (0.1-5.0)
    pub size: f64,
    /// Pattern offset (0.0-1.0)
    pub offset: f64,
    /// Edge sharpness (0.1-5.0)
    pub sharpness: f64,
    /// Pattern rotation (0-360)
    pub rotation: f64,
    /// Animation speed (0.0-5.0)
    pub speed: f64,
    /// Animation mode (zoom/scroll/static)
    pub mode: String,
}

impl DiamondParams {
    const SIZE_PARAM: DiamondSizeParam = DiamondSizeParam;
    const OFFSET_PARAM: DiamondOffsetParam = DiamondOffsetParam;
    const SHARPNESS_PARAM: DiamondSharpnessParam = DiamondSharpnessParam;
    const ROTATION_PARAM: DiamondRotationParam = DiamondRotationParam;
    const SPEED_PARAM: DiamondSpeedParam = DiamondSpeedParam;
    const MODE_PARAM: DiamondAnimationModeParam = DiamondAnimationModeParam;
}

impl Default for DiamondParams {
    fn default() -> Self {
        Self {
            size: 1.0,
            offset: 0.0,
            sharpness: 1.0,
            rotation: 0.0,
            speed: 1.0,
            mode: "zoom".to_string(),
        }
    }
}

// Use the validate macro to implement validation
define_param!(validate DiamondParams,
    SIZE_PARAM: DiamondSizeParam,
    OFFSET_PARAM: DiamondOffsetParam,
    SHARPNESS_PARAM: DiamondSharpnessParam,
    ROTATION_PARAM: DiamondRotationParam,
    SPEED_PARAM: DiamondSpeedParam,
    MODE_PARAM: DiamondAnimationModeParam
);

impl PatternParam for DiamondParams {
    fn name(&self) -> &'static str {
        "diamond"
    }

    fn description(&self) -> &'static str {
        "Diamond-shaped pattern with rotation and sharpness control"
    }

    fn param_type(&self) -> ParamType {
        ParamType::Composite
    }

    fn default_value(&self) -> String {
        format!(
            "size={},offset={},sharpness={},rotation={},speed={},mode={}",
            self.size, self.offset, self.sharpness, self.rotation, self.speed, self.mode
        )
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        self.validate_params(value)
    }

    fn parse(&self, value: &str) -> Result<Box<dyn PatternParam>, String> {
        let mut params = DiamondParams::default();

        for part in value.split(',') {
            let kv: Vec<&str> = part.split('=').collect();
            if kv.len() != 2 {
                continue;
            }

            match kv[0] {
                "size" => {
                    Self::SIZE_PARAM.validate(kv[1])?;
                    params.size = kv[1].parse().unwrap();
                }
                "offset" => {
                    Self::OFFSET_PARAM.validate(kv[1])?;
                    params.offset = kv[1].parse().unwrap();
                }
                "sharpness" => {
                    Self::SHARPNESS_PARAM.validate(kv[1])?;
                    params.sharpness = kv[1].parse().unwrap();
                }
                "rotation" => {
                    Self::ROTATION_PARAM.validate(kv[1])?;
                    params.rotation = kv[1].parse().unwrap();
                }
                "speed" => {
                    Self::SPEED_PARAM.validate(kv[1])?;
                    params.speed = kv[1].parse().unwrap();
                }
                "mode" => {
                    Self::MODE_PARAM.validate(kv[1])?;
                    params.mode = kv[1].to_string();
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
            Box::new(Self::SIZE_PARAM),
            Box::new(Self::OFFSET_PARAM),
            Box::new(Self::SHARPNESS_PARAM),
            Box::new(Self::ROTATION_PARAM),
            Box::new(Self::SPEED_PARAM),
            Box::new(Self::MODE_PARAM),
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
    /// Generates a diamond-shaped pattern
    #[inline(always)]
    pub fn diamond(&self, x_norm: f64, y_norm: f64, params: DiamondParams) -> f64 {
        // Pre-calculate rotation values
        let rot_rad = params.rotation * (PI / 180.0);
        let (sin_rot, cos_rot) = {
            let sin_val = self.utils.fast_sin(rot_rad);
            let cos_val = self.utils.fast_cos(rot_rad);
            (sin_val, cos_val)
        };

        // Optimize coordinate rotation
        let x_rot = x_norm * cos_rot - y_norm * sin_rot;
        let y_rot = x_norm * sin_rot + y_norm * cos_rot;

        // Pre-calculate time-based values
        let time = self.time * PI * params.speed;
        let time_sin = self.utils.fast_sin(time * 0.5);
        let time_sin07 = self.utils.fast_sin(time * 0.7);
        let time_sin2 = self.utils.fast_sin(time * 2.0);

        // Calculate animation factor once
        let animation_factor = match params.mode.as_str() {
            "zoom" => 1.0 + time_sin * 0.5,
            "scroll" => 1.0 + time * 0.1,
            _ => 1.0, // static mode or fallback
        };

        // Combine scale calculations
        let scale = 2.0 * params.size * animation_factor;
        let x_scaled = x_rot * scale;
        let y_scaled = y_rot * scale;

        // Fast absolute value calculation
        let diamond_dist = x_scaled.abs() + y_scaled.abs();

        // Optimize pattern calculation
        let pattern_repeat = diamond_dist - diamond_dist.floor();

        // Combine sharpness calculations
        let sharpness_mod = params.sharpness * (1.0 + time_sin07 * 0.2);

        // Pre-calculate pattern components
        let pattern_base = (pattern_repeat * sharpness_mod * PI).sin();
        let pattern = (pattern_base + params.offset).clamp(0.0, 1.0);

        // Optimize distance and pulse calculations
        let dist_sq = x_rot * x_rot + y_rot * y_rot;
        let pulse = time_sin2 * 0.05 * (-dist_sq * 3.0).exp();

        // Final combination with optimized clamping
        let result = pattern + pulse;
        result.clamp(0.0, 1.0)
    }
}
