use crate::define_param;
use crate::pattern::params::{ParamType, PatternParam};
use std::any::Any;
use std::f64::consts::PI;

// Define parameters with proper CLI names and bounds
define_param!(num Ripple, CenterXParam, "center_x", "X-coordinate of the ripple center", 0.0, 1.0, 0.5);
define_param!(num Ripple, CenterYParam, "center_y", "Y-coordinate of the ripple center", 0.0, 1.0, 0.5);
define_param!(num Ripple, WavelengthParam, "wavelength", "Distance between ripple waves", 0.1, 5.0, 1.0);
define_param!(num Ripple, DampingParam, "damping", "How quickly ripples fade out", 0.0, 1.0, 0.5);
define_param!(num Ripple, FrequencyParam, "frequency", "Speed of ripple animation", 0.1, 10.0, 1.0);
define_param!(num Ripple, CentersParam, "centers", "Number of ripple centers (1-4)", 1.0, 4.0, 1.0);

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
    /// Number of ripple centers (1-4)
    pub centers: u8,
}

impl RippleParams {
    const CENTER_X_PARAM: RippleCenterXParam = RippleCenterXParam;
    const CENTER_Y_PARAM: RippleCenterYParam = RippleCenterYParam;
    const WAVELENGTH_PARAM: RippleWavelengthParam = RippleWavelengthParam;
    const DAMPING_PARAM: RippleDampingParam = RippleDampingParam;
    const FREQUENCY_PARAM: RippleFrequencyParam = RippleFrequencyParam;
    const CENTERS_PARAM: RippleCentersParam = RippleCentersParam;
}

impl Default for RippleParams {
    fn default() -> Self {
        Self {
            center_x: 0.5,
            center_y: 0.5,
            wavelength: 1.0,
            damping: 0.5,
            frequency: 1.0,
            centers: 1,
        }
    }
}

// Use the validate macro to implement validation
define_param!(validate RippleParams,
    CENTER_X_PARAM: RippleCenterXParam,
    CENTER_Y_PARAM: RippleCenterYParam,
    WAVELENGTH_PARAM: RippleWavelengthParam,
    DAMPING_PARAM: RippleDampingParam,
    FREQUENCY_PARAM: RippleFrequencyParam,
    CENTERS_PARAM: RippleCentersParam
);

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
            "center_x={},center_y={},wavelength={},damping={},frequency={},centers={}",
            self.center_x, self.center_y, self.wavelength, self.damping, self.frequency, self.centers
        )
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        self.validate_params(value)
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
                "centers" => {
                    Self::CENTERS_PARAM.validate(kv[1])?;
                    params.centers = kv[1].parse().unwrap();
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
            Box::new(Self::CENTER_X_PARAM),
            Box::new(Self::CENTER_Y_PARAM),
            Box::new(Self::WAVELENGTH_PARAM),
            Box::new(Self::DAMPING_PARAM),
            Box::new(Self::FREQUENCY_PARAM),
            Box::new(Self::CENTERS_PARAM),
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
    /// Generates a ripple pattern emanating from one or more center points
    #[inline(always)]
    pub fn ripple(&self, x_norm: f64, y_norm: f64, params: RippleParams) -> f64 {
        let x_pos = x_norm + 0.5;
        let y_pos = y_norm + 0.5;

        // Pre-calculate time-based values
        let time_factor = self.time * params.frequency * PI * 2.0;
        let time_sin_half = self.utils.fast_sin(time_factor * 0.5);
        let time_sin_07 = self.utils.fast_sin(time_factor * 0.7);

        // Calculate ripple value for a single center
        let calc_ripple = |cx: f64, cy: f64, phase_offset: f64| -> f64 {
            let dx = x_pos - cx;
            let dy = y_pos - cy;
            let dist_sq = dx * dx + dy * dy;
            let distance = dist_sq.sqrt();

            // Ripple wave with phase offset for multi-center variety
            let wave_phase = distance / params.wavelength * PI * 10.0 + time_factor + phase_offset;
            let value = self.utils.fast_sin(wave_phase);

            // Damping
            let amplitude = (-distance * params.damping * 5.0).exp().max(0.2);

            // Modulation
            let base_mod = time_sin_half * 0.3;
            let dist_mod = self.utils.fast_sin(time_factor + distance * PI * 4.0) * 0.2;

            let angle = if dx == 0.0 && dy == 0.0 { 0.0 } else { dy.atan2(dx) };
            let phase_mod = time_sin_07 * self.utils.fast_sin(angle * 2.0 + time_factor * 0.1) * 0.2;

            let modulation = base_mod + dist_mod + phase_mod;
            value * amplitude + modulation
        };

        // Primary center
        let mut total = calc_ripple(params.center_x, params.center_y, 0.0);
        let mut count = 1.0;

        // Additional centers distributed around primary
        if params.centers >= 2 {
            // Second center: opposite corner
            let cx2 = 1.0 - params.center_x;
            let cy2 = 1.0 - params.center_y;
            total += calc_ripple(cx2, cy2, PI * 0.5);
            count += 1.0;
        }
        if params.centers >= 3 {
            // Third center: horizontal flip
            let cx3 = 1.0 - params.center_x;
            let cy3 = params.center_y;
            total += calc_ripple(cx3, cy3, PI);
            count += 1.0;
        }
        if params.centers >= 4 {
            // Fourth center: vertical flip
            let cx4 = params.center_x;
            let cy4 = 1.0 - params.center_y;
            total += calc_ripple(cx4, cy4, PI * 1.5);
            count += 1.0;
        }

        // Average all ripples and normalize
        let combined = total / count;
        let result = (combined + 1.0) * 0.5;
        result.clamp(0.0, 1.0)
    }
}
