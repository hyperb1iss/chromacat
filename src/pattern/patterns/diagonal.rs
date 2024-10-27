use std::f64::consts::PI;
use std::any::Any;
use crate::pattern::params::{PatternParam, ParamType};
use crate::define_param;

define_param!(num Diagonal, AngleParam, "Angle of the diagonal pattern", 0.0, 360.0, 45.0);
define_param!(num Diagonal, FrequencyParam, "Animation speed", 0.1, 10.0, 1.0);

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
                _ => {}
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
    pub fn diagonal(&self, x: usize, y: usize, params: DiagonalParams) -> f64 {
        if self.width <= 1 || self.height <= 1 {
            return 0.0;
        }

        let x_norm = x as f64 / (self.width - 1) as f64;
        let y_norm = (y % self.height) as f64 / (self.height - 1) as f64;

        let x_scaled = x_norm * 2.0 - 1.0;
        let y_scaled = y_norm * 2.0 - 1.0;

        let time = self.time * PI * 2.0;
        let wave_offset = self.utils.fast_sin(time * 0.5 + y_norm * 8.0) * 0.2;

        let base_angle = params.angle as f64;
        let animated_angle = (base_angle + self.utils.fast_sin(time * 0.3) * 15.0) * PI / 180.0;

        let cos_angle = self.utils.fast_cos(animated_angle);
        let sin_angle = self.utils.fast_sin(animated_angle);

        let rotated = x_scaled * cos_angle + y_scaled * sin_angle + wave_offset;
        let perpendicular = -x_scaled * sin_angle + y_scaled * cos_angle;
        let wave_distortion = self.utils.fast_sin(perpendicular * 4.0 * params.frequency + time) * 0.1;

        let result = (rotated + wave_distortion + 1.0) * 0.5;
        let pulse = (self.utils.fast_sin(time * 0.7) * 0.1 + 1.0) * result;

        pulse.clamp(0.0, 1.0)
    }
}
