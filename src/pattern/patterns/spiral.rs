use std::f64::consts::PI;
use std::any::Any;
use crate::pattern::params::{PatternParam, ParamType};
use crate::define_param;

define_param!(num Spiral, DensityParam, "How tightly wound the spiral is", 0.1, 5.0, 1.0);
define_param!(num Spiral, RotationParam, "Base rotation angle in degrees", 0.0, 360.0, 0.0);
define_param!(num Spiral, ExpansionParam, "How quickly spiral expands", 0.1, 2.0, 1.0);
define_param!(bool Spiral, ClockwiseParam, "Direction of spiral rotation", true);
define_param!(num Spiral, FrequencyParam, "Animation speed", 0.1, 10.0, 1.0);

/// Parameters for configuring spiral pattern effects
#[derive(Debug, Clone)]
pub struct SpiralParams {
    /// How tightly wound the spiral is (0.1-5.0)
    pub density: f64,
    /// Base rotation angle in degrees (0-360)
    pub rotation: f64,
    /// How quickly spiral expands from center (0.1-2.0)
    pub expansion: f64,
    /// Direction of spiral rotation
    pub clockwise: bool,
    /// Speed of spiral animation (0.1-10.0)
    pub frequency: f64,
}

impl SpiralParams {
    const DENSITY_PARAM: SpiralDensityParam = SpiralDensityParam;
    const ROTATION_PARAM: SpiralRotationParam = SpiralRotationParam;
    const EXPANSION_PARAM: SpiralExpansionParam = SpiralExpansionParam;
    const CLOCKWISE_PARAM: SpiralClockwiseParam = SpiralClockwiseParam;
    const FREQUENCY_PARAM: SpiralFrequencyParam = SpiralFrequencyParam;
}

impl Default for SpiralParams {
    fn default() -> Self {
        Self {
            density: 1.0,
            rotation: 0.0,
            expansion: 1.0,
            clockwise: true,
            frequency: 1.0,
        }
    }
}

impl PatternParam for SpiralParams {
    fn name(&self) -> &'static str {
        "spiral"
    }

    fn description(&self) -> &'static str {
        "Spiral pattern rotating from center"
    }

    fn param_type(&self) -> ParamType {
        ParamType::Composite
    }

    fn default_value(&self) -> String {
        format!(
            "density={},rotation={},expansion={},clockwise={},frequency={}",
            self.density, self.rotation, self.expansion, self.clockwise, self.frequency
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
        let mut params = SpiralParams::default();
        
        for part in value.split(',') {
            let kv: Vec<&str> = part.split('=').collect();
            if kv.len() != 2 {
                continue;
            }
            
            match kv[0] {
                "density" => {
                    Self::DENSITY_PARAM.validate(kv[1])?;
                    params.density = kv[1].parse().unwrap();
                }
                "rotation" => {
                    Self::ROTATION_PARAM.validate(kv[1])?;
                    params.rotation = kv[1].parse().unwrap();
                }
                "expansion" => {
                    Self::EXPANSION_PARAM.validate(kv[1])?;
                    params.expansion = kv[1].parse().unwrap();
                }
                "clockwise" => {
                    Self::CLOCKWISE_PARAM.validate(kv[1])?;
                    params.clockwise = kv[1].parse().unwrap();
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
            Box::new(Self::DENSITY_PARAM),
            Box::new(Self::ROTATION_PARAM),
            Box::new(Self::EXPANSION_PARAM),
            Box::new(Self::CLOCKWISE_PARAM),
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
    /// Generates a spiral pattern rotating from the center
    pub fn spiral(&self, x: usize, y: usize, params: SpiralParams) -> f64 {
        let center_x = self.width as f64 / 2.0;
        let center_y = self.height as f64 / 2.0;
        let dx = x as f64 - center_x;
        let dy = y as f64 - center_y;

        let mut angle = dy.atan2(dx);
        if !params.clockwise {
            angle = -angle;
        }

        let distance = (dx * dx + dy * dy).sqrt() / (self.width.min(self.height) as f64 / 2.0);
        let rot_rad = params.rotation * PI / 180.0;
        let time = self.time * PI * 2.0;

        ((angle + distance * params.density * params.expansion + rot_rad + time * params.frequency)
            % (PI * 2.0))
            / (PI * 2.0)
    }
}
