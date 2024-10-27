use std::f64::consts::PI;
use std::any::Any;
use crate::pattern::params::{PatternParam, ParamType};
use crate::define_param;

define_param!(num Diamond, SizeParam, "Size of diamond shapes", 0.1, 5.0, 1.0);
define_param!(num Diamond, OffsetParam, "Pattern offset", 0.0, 1.0, 0.0);
define_param!(num Diamond, SharpnessParam, "Edge sharpness", 0.1, 5.0, 1.0);
define_param!(num Diamond, RotationParam, "Pattern rotation", 0.0, 360.0, 0.0);

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
}

impl DiamondParams {
    const SIZE_PARAM: DiamondSizeParam = DiamondSizeParam;
    const OFFSET_PARAM: DiamondOffsetParam = DiamondOffsetParam;
    const SHARPNESS_PARAM: DiamondSharpnessParam = DiamondSharpnessParam;
    const ROTATION_PARAM: DiamondRotationParam = DiamondRotationParam;
}

impl Default for DiamondParams {
    fn default() -> Self {
        Self {
            size: 1.0,
            offset: 0.0,
            sharpness: 1.0,
            rotation: 0.0,
        }
    }
}

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
            "size={},offset={},sharpness={},rotation={}",
            self.size, self.offset, self.sharpness, self.rotation
        )
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        for param in self.sub_params() {
            if let Some(param_value) = value.split(',')
                .find(|part| part.starts_with(&format!("{}=", param.name())))
            {
                param.validate(param_value.split('=').nth(1).unwrap_or(""))?;
            }
        }
        Ok(())
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
                _ => {}
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
    pub fn diamond(&self, x: usize, y: usize, params: DiamondParams) -> f64 {
        let x_norm = x as f64 / self.width as f64;
        let y_norm = y as f64 / self.height as f64;

        // Center coordinates
        let x_centered = x_norm - 0.5;
        let y_centered = y_norm - 0.5;

        // Rotate coordinates
        let rot_rad = params.rotation * PI / 180.0;
        let cos_rot = self.utils.fast_cos(rot_rad);
        let sin_rot = self.utils.fast_sin(rot_rad);
        let x_rot = x_centered * cos_rot - y_centered * sin_rot;
        let y_rot = x_centered * sin_rot + y_centered * cos_rot;

        // Calculate diamond pattern
        let x_scaled = x_rot * params.size;
        let y_scaled = y_rot * params.size;

        let x_cell = x_scaled.floor();
        let y_cell = y_scaled.floor();
        let x_fract = x_scaled.fract();
        let y_fract = y_scaled.fract();

        // Calculate distance to diamond center
        let dx = x_fract - 0.5;
        let dy = y_fract - 0.5;
        let diamond_dist = dx.abs() + dy.abs();

        // Apply sharpness and offset
        let pattern = ((diamond_dist * params.sharpness).sin() + params.offset).clamp(0.0, 1.0);

        // Animate pattern
        let time = self.time * PI * 2.0;
        let wave = self.utils.fast_sin(time * 0.5 + (x_cell + y_cell) * 0.2) * 0.1;

        (pattern + wave).clamp(0.0, 1.0)
    }
}
