use std::f64::consts::PI;
use std::any::Any;
use crate::pattern::params::{PatternParam, ParamType};
use crate::define_param;

// First define the individual parameters
define_param!(num Diagonal, AngleParam, "angle", "Angle of the diagonal pattern", 0.0, 360.0, 45.0);
define_param!(num Diagonal, FrequencyParam, "frequency", "Animation speed", 0.1, 10.0, 1.0);

/// Parameters for configuring diagonal pattern effects
#[derive(Debug, Clone)]
pub struct DiagonalParams {
    /// Angle in degrees (0-360)
    pub angle: i32,
    /// Animation frequency (0.1-10.0)
    pub frequency: f64,
}

impl DiagonalParams {
    const ANGLE_PARAM: DiagonalAngleParam = DiagonalAngleParam;
    const FREQUENCY_PARAM: DiagonalFrequencyParam = DiagonalFrequencyParam;
}

impl Default for DiagonalParams {
    fn default() -> Self {
        Self {
            angle: 45,
            frequency: 1.0,
        }
    }
}

// Use the validate macro to implement validation
define_param!(validate DiagonalParams,
    ANGLE_PARAM: DiagonalAngleParam,
    FREQUENCY_PARAM: DiagonalFrequencyParam
);

impl PatternParam for DiagonalParams {
    fn name(&self) -> &'static str {
        "diagonal"
    }

    fn description(&self) -> &'static str {
        "Gradient at an angle with wave animation"
    }

    fn param_type(&self) -> ParamType {
        ParamType::Composite
    }

    fn default_value(&self) -> String {
        format!(
            "angle={},frequency={}",
            self.angle, self.frequency
        )
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        self.validate_params(value)
    }

    fn parse(&self, value: &str) -> Result<Box<dyn PatternParam>, String> {
        let mut params = DiagonalParams::default();
        
        for part in value.split(',') {
            let kv: Vec<&str> = part.split('=').collect();
            if kv.len() != 2 {
                continue;
            }
            
            match kv[0] {
                "angle" => {
                    Self::ANGLE_PARAM.validate(kv[1])?;
                    params.angle = kv[1].parse().unwrap();
                }
                "frequency" => {
                    Self::FREQUENCY_PARAM.validate(kv[1])?;
                    params.frequency = kv[1].parse().unwrap();
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
            Box::new(Self::ANGLE_PARAM),
            Box::new(Self::FREQUENCY_PARAM),
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
    /// Generates an animated diagonal gradient pattern
    pub fn diagonal(&self, x_norm: f64, y_norm: f64, params: DiagonalParams) -> f64 {
        // Convert angle to radians
        let angle = params.angle as f64 * PI / 180.0;
        let cos_angle = self.utils.fast_cos(angle);
        let sin_angle = self.utils.fast_sin(angle);

        // Simple diagonal flow
        let value = ((x_norm + 0.5) * cos_angle + (y_norm + 0.5) * sin_angle + self.time * params.frequency) % 1.0;
        
        // Ensure value stays in [0, 1] range
        if value < 0.0 {
            value + 1.0
        } else {
            value
        }
    }
}
