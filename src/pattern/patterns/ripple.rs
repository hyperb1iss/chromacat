use std::f64::consts::PI;
use crate::pattern::params::{PatternParam, ParamType};
use crate::define_param;
use std::any::Any;

define_param!(num Ripple, CenterX, "X-coordinate of the ripple center", 0.0, 1.0, 0.5);
define_param!(num Ripple, CenterY, "Y-coordinate of the ripple center", 0.0, 1.0, 0.5);
define_param!(num Ripple, Wavelength, "Distance between ripple waves", 0.1, 5.0, 1.0);
define_param!(num Ripple, Damping, "How quickly ripples fade out", 0.0, 1.0, 0.5);
define_param!(num Ripple, Frequency, "Speed of ripple animation", 0.1, 10.0, 1.0);

/// Parameters for configuring ripple pattern effects
#[derive(Debug, Clone)]
pub struct RippleParams {
    /// X-coordinate of the ripple center (0.0-1.0)
    pub center_x: f64,
    /// Y-coordinate of the ripple center (0.0-1.0)
    pub center_y: f64,
    /// Distance between ripple waves (0.1-5.0)
    pub wavelength: f64,
    /// How quickly ripples fade out with distance (0.0-1.0)
    pub damping: f64,
    /// Speed of ripple animation (0.1-10.0)
    pub frequency: f64,
}

impl RippleParams {
    const CENTER_X_PARAM: RippleCenterX = RippleCenterX;
    const CENTER_Y_PARAM: RippleCenterY = RippleCenterY;
    const WAVELENGTH_PARAM: RippleWavelength = RippleWavelength;
    const DAMPING_PARAM: RippleDamping = RippleDamping;
    const FREQUENCY_PARAM: RippleFrequency = RippleFrequency;
}

impl Default for RippleParams {
    fn default() -> Self {
        Self {
            center_x: 0.5,
            center_y: 0.5,
            wavelength: 1.0,
            damping: 0.5,
            frequency: 1.0,
        }
    }
}

impl PatternParam for RippleParams {
    fn name(&self) -> &'static str {
        "ripple"
    }

    fn description(&self) -> &'static str {
        "Ripple effect emanating from a center point"
    }

    fn param_type(&self) -> ParamType {
        ParamType::Composite
    }

    fn default_value(&self) -> String {
        format!(
            "center_x={},center_y={},wavelength={},damping={},frequency={}",
            self.center_x, self.center_y, self.wavelength, self.damping, self.frequency
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
        let mut params = RippleParams::default();
        
        for part in value.split(',') {
            let kv: Vec<&str> = part.split('=').collect();
            if kv.len() != 2 {
                continue;
            }
            
            match kv[0] {
                "center_x" => {
                    Self::CENTER_X_PARAM.validate(kv[1])?;
                    params.center_x = kv[1].parse().unwrap();
                }
                "center_y" => {
                    Self::CENTER_Y_PARAM.validate(kv[1])?;
                    params.center_y = kv[1].parse().unwrap();
                }
                "wavelength" => {
                    Self::WAVELENGTH_PARAM.validate(kv[1])?;
                    params.wavelength = kv[1].parse().unwrap();
                }
                "damping" => {
                    Self::DAMPING_PARAM.validate(kv[1])?;
                    params.damping = kv[1].parse().unwrap();
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
            Box::new(Self::CENTER_X_PARAM),
            Box::new(Self::CENTER_Y_PARAM),
            Box::new(Self::WAVELENGTH_PARAM),
            Box::new(Self::DAMPING_PARAM),
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
    /// Generates a ripple pattern emanating from a center point
    pub fn ripple(&self, x: usize, y: usize, params: RippleParams) -> f64 {
        let dx = x as f64 / self.width as f64 - params.center_x;
        let dy = y as f64 / self.height as f64 - params.center_y;
        let distance = (dx * dx + dy * dy).sqrt();

        let time = self.time * PI * 2.0;
        let value = self
            .utils
            .fast_sin(distance / params.wavelength * PI * 10.0 * params.frequency + time);
        let amplitude = (-distance * params.damping * 5.0).exp();

        (value * amplitude + 1.0) / 2.0
    }
}
