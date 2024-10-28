use std::f64::consts::PI;
use std::any::Any;
use crate::pattern::params::{PatternParam, ParamType};
use crate::pattern::utils::PatternUtils;
use crate::define_param;

define_param!(num Checker, SizeParam, "size", "Size of checker squares", 1.0, 10.0, 2.0);
define_param!(num Checker, BlurParam, "blur", "Blur between squares", 0.0, 1.0, 0.1);
define_param!(num Checker, RotationParam, "rotation", "Pattern rotation angle", 0.0, 360.0, 0.0);
define_param!(num Checker, ScaleParam, "scale", "Scale of the pattern", 0.1, 5.0, 1.0);

/// Parameters for configuring checkerboard pattern effects
#[derive(Debug, Clone)]
pub struct CheckerboardParams {
    /// Size of checker squares (1-10)
    pub size: usize,
    /// Blur between squares (0.0-1.0)
    pub blur: f64,
    /// Pattern rotation angle (0-360)
    pub rotation: f64,
    /// Scale of the pattern (0.1-5.0)
    pub scale: f64,
}

impl CheckerboardParams {
    const SIZE_PARAM: CheckerSizeParam = CheckerSizeParam;
    const BLUR_PARAM: CheckerBlurParam = CheckerBlurParam;
    const ROTATION_PARAM: CheckerRotationParam = CheckerRotationParam;
    const SCALE_PARAM: CheckerScaleParam = CheckerScaleParam;
}

impl Default for CheckerboardParams {
    fn default() -> Self {
        Self {
            size: 2,
            blur: 0.1,
            rotation: 0.0,
            scale: 1.0,
        }
    }
}

impl PatternParam for CheckerboardParams {
    fn name(&self) -> &'static str {
        "checkerboard"
    }

    fn description(&self) -> &'static str {
        "Checkerboard pattern with rotation and blur"
    }

    fn param_type(&self) -> ParamType {
        ParamType::Composite
    }

    fn default_value(&self) -> String {
        format!(
            "size={},blur={},rotation={},scale={}",
            self.size, self.blur, self.rotation, self.scale
        )
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        // If the value contains commas, validate each part separately
        if value.contains(',') {
            for part in value.split(',') {
                self.validate(part.trim())?;
            }
            return Ok(());
        }

        // Check each parameter
        let kv: Vec<&str> = value.split('=').collect();
        if kv.len() != 2 {
            return Err("Parameter must be in format key=value".to_string());
        }

        // Validate parameter name first
        let valid_params = ["size", "blur", "rotation", "scale"];
        if !valid_params.contains(&kv[0]) {
            return Err(format!("Invalid parameter name: {}", kv[0]));
        }

        // Then validate the value
        match kv[0] {
            "size" => Self::SIZE_PARAM.validate(kv[1]),
            "blur" => Self::BLUR_PARAM.validate(kv[1]),
            "rotation" => Self::ROTATION_PARAM.validate(kv[1]),
            "scale" => Self::SCALE_PARAM.validate(kv[1]),
            _ => unreachable!(), // We already validated the parameter name
        }
    }

    fn parse(&self, value: &str) -> Result<Box<dyn PatternParam>, String> {
        let mut params = CheckerboardParams::default();
        
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
                "blur" => {
                    Self::BLUR_PARAM.validate(kv[1])?;
                    params.blur = kv[1].parse().unwrap();
                }
                "rotation" => {
                    Self::ROTATION_PARAM.validate(kv[1])?;
                    params.rotation = kv[1].parse().unwrap();
                }
                "scale" => {
                    Self::SCALE_PARAM.validate(kv[1])?;
                    params.scale = kv[1].parse().unwrap();
                }
                _ => {}
            }
        }
        
        Ok(Box::new(params))
    }

    fn sub_params(&self) -> Vec<Box<dyn PatternParam>> {
        vec![
            Box::new(Self::SIZE_PARAM),
            Box::new(Self::BLUR_PARAM),
            Box::new(Self::ROTATION_PARAM),
            Box::new(Self::SCALE_PARAM),
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
    /// Generates a checkerboard pattern with rotation and blur
    pub fn checkerboard(&self, x_norm: f64, y_norm: f64, params: CheckerboardParams) -> f64 {
        // Scale coordinates
        let x_scaled = x_norm * params.scale;
        let y_scaled = y_norm * params.scale;

        // Add time-based rotation
        let time_rotation = self.time * 45.0;
        let total_rotation = (params.rotation + time_rotation) * PI / 180.0;
        
        // Rotate coordinates
        let cos_rot = self.utils.fast_cos(total_rotation);
        let sin_rot = self.utils.fast_sin(total_rotation);
        let x_rot = x_scaled * cos_rot - y_scaled * sin_rot;
        let y_rot = x_scaled * sin_rot + y_scaled * cos_rot;

        // Add time-based scale oscillation
        let scale_oscillation = self.utils.fast_sin(self.time * PI) * 0.2 + 1.0;
        let animated_scale = params.size as f64 * scale_oscillation;

        // Calculate checker pattern with animated scale
        let x_checker = (x_rot * animated_scale).floor() as i32;
        let y_checker = (y_rot * animated_scale).floor() as i32;
        let base_value = (x_checker + y_checker) % 2 == 0;

        if params.blur <= 0.0 {
            return if base_value { 1.0 } else { 0.0 };
        }

        // Apply blur using smoothstep with time-based blur variation
        let x_fract = (x_rot * animated_scale).fract();
        let y_fract = (y_rot * animated_scale).fract();
        let animated_blur = params.blur * (self.utils.fast_sin(self.time * PI * 2.0) * 0.2 + 0.8);
        let blur_scale = animated_blur * 0.5;

        let x_blend = PatternUtils::smoothstep(
            if (0.5 - blur_scale..=0.5 + blur_scale).contains(&x_fract) { 1.0 } else { 0.0 }
        );
        let y_blend = PatternUtils::smoothstep(
            if (0.5 - blur_scale..=0.5 + blur_scale).contains(&y_fract) { 1.0 } else { 0.0 }
        );

        if base_value {
            (1.0 - x_blend) * (1.0 - y_blend) + x_blend * y_blend
        } else {
            x_blend * (1.0 - y_blend) + (1.0 - x_blend) * y_blend
        }
    }
}
