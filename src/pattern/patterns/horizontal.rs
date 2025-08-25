use crate::define_param;
use crate::pattern::params::{ParamType, PatternParam};
use std::any::Any;

// Define the parameter with proper CLI name
define_param!(bool Horizontal, InvertParam, "invert", "Invert gradient direction", false);

/// Parameters for configuring horizontal gradient pattern
#[derive(Debug, Clone, Default)]
pub struct HorizontalParams {
    /// Invert the gradient direction (false = left to right, true = right to left)
    pub invert: bool,
}

impl HorizontalParams {
    const INVERT_PARAM: HorizontalInvertParam = HorizontalInvertParam;
}

// Use the validate macro to implement validation
define_param!(validate HorizontalParams,
    INVERT_PARAM: HorizontalInvertParam
);

impl PatternParam for HorizontalParams {
    fn name(&self) -> &'static str {
        "horizontal"
    }

    fn description(&self) -> &'static str {
        "Simple horizontal gradient pattern"
    }

    fn param_type(&self) -> ParamType {
        ParamType::Composite
    }

    fn default_value(&self) -> String {
        format!("invert={}", self.invert)
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        self.validate_params(value)
    }

    fn parse(&self, value: &str) -> Result<Box<dyn PatternParam>, String> {
        let mut params = HorizontalParams::default();

        for part in value.split(',') {
            let kv: Vec<&str> = part.split('=').collect();
            if kv.len() != 2 {
                continue;
            }

            match kv[0] {
                "invert" => {
                    Self::INVERT_PARAM.validate(kv[1])?;
                    params.invert = kv[1].parse().unwrap();
                }
                invalid_param => {
                    return Err(format!("Invalid parameter name: {invalid_param}"));
                }
            }
        }

        Ok(Box::new(params))
    }

    fn sub_params(&self) -> Vec<Box<dyn PatternParam>> {
        vec![Box::new(Self::INVERT_PARAM)]
    }

    fn clone_param(&self) -> Box<dyn PatternParam> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl super::Patterns {
    /// Generates a simple horizontal gradient pattern
    #[inline(always)]
    pub fn horizontal(&self, x_pos: f64, params: HorizontalParams) -> f64 {
        // Pre-calculate animation offset
        let time_offset = self.time * 0.5;

        // Combine position and time in one operation
        let mut value = x_pos + time_offset;

        // Fast modulo using floor
        value -= value.floor();

        // Branchless inversion using arithmetic
        if params.invert {
            1.0 - value
        } else {
            value
        }
    }
}
