use std::f64::consts::PI;
use std::any::Any;
use crate::pattern::params::{PatternParam, ParamType};
use crate::define_param;

// Define parameters with proper CLI names and bounds
define_param!(num Spiral, DensityParam, "density", "How tightly wound the spiral is", 0.1, 5.0, 1.0);
define_param!(num Spiral, RotationParam, "rotation", "Base rotation angle in degrees", 0.0, 360.0, 0.0);
define_param!(num Spiral, ExpansionParam, "expansion", "How quickly spiral expands", 0.1, 2.0, 1.0);
define_param!(bool Spiral, ClockwiseParam, "clockwise", "Direction of spiral rotation", true);
define_param!(num Spiral, FrequencyParam, "frequency", "Animation speed", 0.1, 10.0, 1.0);

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

// Use the validate macro to implement validation
define_param!(validate SpiralParams,
    DENSITY_PARAM: SpiralDensityParam,
    ROTATION_PARAM: SpiralRotationParam,
    EXPANSION_PARAM: SpiralExpansionParam,
    CLOCKWISE_PARAM: SpiralClockwiseParam,
    FREQUENCY_PARAM: SpiralFrequencyParam
);

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
        self.validate_params(value)
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
                invalid_param => {
                    return Err(format!("Invalid parameter name: {}", invalid_param));
                }
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
    pub fn spiral(&self, x_norm: f64, y_norm: f64, params: SpiralParams) -> f64 {
        // Calculate angle and distance from center
        let angle = y_norm.atan2(x_norm);
        let distance = (x_norm * x_norm + y_norm * y_norm).sqrt();

        // Make time factor more significant
        let time_factor = self.time * params.frequency * PI * 2.0;

        // Base rotation with time-based animation
        let rot_rad = (params.rotation + time_factor * 20.0) * PI / 180.0;
        
        // Primary spiral component
        let spiral_angle = angle + distance * params.density * params.expansion + rot_rad;
        let primary = ((spiral_angle + time_factor) % (PI * 2.0)) / (PI * 2.0);

        // Add distance-based modulation
        let distance_mod = self.utils.fast_sin(distance * PI * 2.0 + time_factor) * 0.2;

        // Add time-based phase modulation
        let phase_mod = self.utils.fast_sin(time_factor * 0.7 + angle * 2.0) * 0.15;

        // Combine all components with smooth transitions
        let combined = primary + distance_mod + phase_mod;
        let smoothed = (self.utils.fast_sin(combined * PI * 2.0) + 1.0) * 0.5;

        smoothed.clamp(0.0, 1.0)
    }
}
