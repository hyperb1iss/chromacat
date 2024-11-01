use crate::define_param;
use crate::pattern::params::{ParamType, PatternParam};
use crate::pattern::utils::PatternUtils;
use std::any::Any;
use std::f64::consts::PI;

define_param!(num Checker, SizeParam, "size", "Size of checker squares", 1.0, 10.0, 2.0);
define_param!(num Checker, BlurParam, "blur", "Blur between squares", 0.0, 1.0, 0.1);
define_param!(num Checker, RotationParam, "rotation", "Pattern rotation angle", 0.0, 360.0, 0.0);
define_param!(num Checker, ScaleParam, "scale", "Scale of the pattern", 0.1, 5.0, 1.0);
define_param!(num Checker, ColorSpeedParam, "color_speed", "Speed of color variation", 0.0, 5.0, 1.0);
define_param!(num Checker, ColorIntensityParam, "color_intensity", "Intensity of color variation", 0.0, 1.0, 0.5);
define_param!(num Checker, WaveSpeedParam, "wave_speed", "Speed of wave movement", 0.0, 5.0, 1.0);
define_param!(num Checker, WaveIntensityParam, "wave_intensity", "Intensity of wave movement", 0.0, 2.0, 0.5);
define_param!(num Checker, OrbitSpeedParam, "orbit_speed", "Speed of orbital movement", 0.0, 5.0, 1.0);
define_param!(num Checker, AntialiasingParam, "antialiasing", "Edge smoothing amount", 0.0, 1.0, 0.5);
define_param!(num Checker, TimeScaleParam, "time_scale", "Overall animation speed", 0.1, 2.0, 0.5);

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
    /// Speed of color variation (0.0-5.0)
    pub color_speed: f64,
    /// Intensity of color variation (0.0-1.0)
    pub color_intensity: f64,
    /// Speed of wave movement (0.0-5.0)
    pub wave_speed: f64,
    /// Intensity of wave movement (0.0-2.0)
    pub wave_intensity: f64,
    /// Speed of orbital movement (0.0-5.0)
    pub orbit_speed: f64,
    /// Edge smoothing amount (0.0-1.0)
    pub antialiasing: f64,
    /// Overall animation speed (0.1-2.0)
    pub time_scale: f64,
}

impl CheckerboardParams {
    const SIZE_PARAM: CheckerSizeParam = CheckerSizeParam;
    const BLUR_PARAM: CheckerBlurParam = CheckerBlurParam;
    const ROTATION_PARAM: CheckerRotationParam = CheckerRotationParam;
    const SCALE_PARAM: CheckerScaleParam = CheckerScaleParam;
    const COLOR_SPEED_PARAM: CheckerColorSpeedParam = CheckerColorSpeedParam;
    const COLOR_INTENSITY_PARAM: CheckerColorIntensityParam = CheckerColorIntensityParam;
    const WAVE_SPEED_PARAM: CheckerWaveSpeedParam = CheckerWaveSpeedParam;
    const WAVE_INTENSITY_PARAM: CheckerWaveIntensityParam = CheckerWaveIntensityParam;
    const ORBIT_SPEED_PARAM: CheckerOrbitSpeedParam = CheckerOrbitSpeedParam;
    const ANTIALIASING_PARAM: CheckerAntialiasingParam = CheckerAntialiasingParam;
    const TIME_SCALE_PARAM: CheckerTimeScaleParam = CheckerTimeScaleParam;
}

impl Default for CheckerboardParams {
    fn default() -> Self {
        Self {
            size: 2,
            blur: 0.1,
            rotation: 0.0,
            scale: 1.0,
            color_speed: 1.0,
            color_intensity: 0.5,
            wave_speed: 1.0,
            wave_intensity: 0.5,
            orbit_speed: 1.0,
            antialiasing: 0.5,
            time_scale: 0.5,
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
            "size={},blur={},rotation={},scale={},color_speed={},color_intensity={},wave_speed={},wave_intensity={},orbit_speed={},antialiasing={},time_scale={}",
            self.size, self.blur, self.rotation, self.scale, self.color_speed, self.color_intensity, self.wave_speed, self.wave_intensity, self.orbit_speed, self.antialiasing, self.time_scale
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
        let valid_params = [
            "size",
            "blur",
            "rotation",
            "scale",
            "color_speed",
            "color_intensity",
            "wave_speed",
            "wave_intensity",
            "orbit_speed",
            "antialiasing",
            "time_scale",
        ];
        if !valid_params.contains(&kv[0]) {
            return Err(format!("Invalid parameter name: {}", kv[0]));
        }

        // Then validate the value
        match kv[0] {
            "size" => Self::SIZE_PARAM.validate(kv[1]),
            "blur" => Self::BLUR_PARAM.validate(kv[1]),
            "rotation" => Self::ROTATION_PARAM.validate(kv[1]),
            "scale" => Self::SCALE_PARAM.validate(kv[1]),
            "color_speed" => Self::COLOR_SPEED_PARAM.validate(kv[1]),
            "color_intensity" => Self::COLOR_INTENSITY_PARAM.validate(kv[1]),
            "wave_speed" => Self::WAVE_SPEED_PARAM.validate(kv[1]),
            "wave_intensity" => Self::WAVE_INTENSITY_PARAM.validate(kv[1]),
            "orbit_speed" => Self::ORBIT_SPEED_PARAM.validate(kv[1]),
            "antialiasing" => Self::ANTIALIASING_PARAM.validate(kv[1]),
            "time_scale" => Self::TIME_SCALE_PARAM.validate(kv[1]),
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
                "color_speed" => {
                    Self::COLOR_SPEED_PARAM.validate(kv[1])?;
                    params.color_speed = kv[1].parse().unwrap();
                }
                "color_intensity" => {
                    Self::COLOR_INTENSITY_PARAM.validate(kv[1])?;
                    params.color_intensity = kv[1].parse().unwrap();
                }
                "wave_speed" => {
                    Self::WAVE_SPEED_PARAM.validate(kv[1])?;
                    params.wave_speed = kv[1].parse().unwrap();
                }
                "wave_intensity" => {
                    Self::WAVE_INTENSITY_PARAM.validate(kv[1])?;
                    params.wave_intensity = kv[1].parse().unwrap();
                }
                "orbit_speed" => {
                    Self::ORBIT_SPEED_PARAM.validate(kv[1])?;
                    params.orbit_speed = kv[1].parse().unwrap();
                }
                "antialiasing" => {
                    Self::ANTIALIASING_PARAM.validate(kv[1])?;
                    params.antialiasing = kv[1].parse().unwrap();
                }
                "time_scale" => {
                    Self::TIME_SCALE_PARAM.validate(kv[1])?;
                    params.time_scale = kv[1].parse().unwrap();
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
            Box::new(Self::COLOR_SPEED_PARAM),
            Box::new(Self::COLOR_INTENSITY_PARAM),
            Box::new(Self::WAVE_SPEED_PARAM),
            Box::new(Self::WAVE_INTENSITY_PARAM),
            Box::new(Self::ORBIT_SPEED_PARAM),
            Box::new(Self::ANTIALIASING_PARAM),
            Box::new(Self::TIME_SCALE_PARAM),
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
    #[inline(always)]
    pub fn checkerboard(&self, x_norm: f64, y_norm: f64, params: CheckerboardParams) -> f64 {
        // Scale time for all animations
        let time = self.time * params.time_scale;

        // Calculate wave-based movement with slower default speeds
        let wave_x =
            self.utils.fast_sin(time * params.wave_speed * PI * 0.5) * params.wave_intensity;
        let wave_y =
            self.utils.fast_cos(time * params.wave_speed * PI * 0.35) * params.wave_intensity;

        // Calculate orbital movement with slower rotation
        let orbit_angle = time * params.orbit_speed * PI * 0.5;
        let orbit_radius = 0.3 * self.utils.fast_sin(time * PI * 0.25).abs();
        let orbit_x = self.utils.fast_cos(orbit_angle) * orbit_radius;
        let orbit_y = self.utils.fast_sin(orbit_angle) * orbit_radius;

        // Combine movements
        let x_moved = x_norm + wave_x + orbit_x;
        let y_moved = y_norm + wave_y + orbit_y;

        // Calculate dynamic zoom based on position and time
        let zoom_wave = self
            .utils
            .fast_sin((x_moved * 0.5 + y_moved * 0.5 + time * params.wave_speed) * PI)
            * 0.3
            + 1.0;
        let final_scale = params.scale * zoom_wave;

        // Pre-calculate scaled coordinates with the new movement
        let x_scaled = x_moved * final_scale;
        let y_scaled = y_moved * final_scale;

        // Pre-calculate rotation values with more dynamic rotation
        let base_rotation = params.rotation + time * 45.0;
        let wave_rotation = self.utils.fast_sin(time * params.wave_speed * PI) * 30.0;
        let total_rotation = (base_rotation + wave_rotation) * (PI / 180.0);

        let (sin_rot, cos_rot) = {
            let sin_val = self.utils.fast_sin(total_rotation);
            let cos_val = self.utils.fast_cos(total_rotation);
            (sin_val, cos_val)
        };

        // Combine rotation calculations
        let x_rot = x_scaled * cos_rot - y_scaled * sin_rot;
        let y_rot = x_scaled * sin_rot + y_scaled * cos_rot;

        // Pre-calculate scale animation
        let scale_factor = self.utils.fast_sin(self.time * PI) * 0.2 + 1.0;
        let size_scaled = params.size as f64 * scale_factor;

        // Enhanced antialiasing calculation
        let aa_sample_count = 4; // Number of samples for antialiasing
        let aa_offset = params.antialiasing * 0.5 / size_scaled;
        let mut aa_value = 0.0;

        // Sample multiple points for antialiasing
        for i in 0..aa_sample_count {
            for j in 0..aa_sample_count {
                let aa_x = x_rot + (i as f64 - 1.5) * aa_offset;
                let aa_y = y_rot + (j as f64 - 1.5) * aa_offset;

                let x_checker = (aa_x * size_scaled).floor() as i32;
                let y_checker = (aa_y * size_scaled).floor() as i32;
                let is_white = (x_checker + y_checker) & 1 == 0;

                // Calculate base checker value with position-based variation
                let base_value = if is_white { 1.0 } else { 0.0 };

                // Add to antialiased value
                aa_value += base_value;
            }
        }

        // Average the antialiased samples
        let base_value = aa_value / (aa_sample_count * aa_sample_count) as f64;

        // Enhanced color variation with smoother transitions
        let pos_variation = self.utils.fast_sin(x_rot * 0.25 + y_rot * 0.25);
        let time_variation = self.utils.fast_sin(time * params.color_speed * PI * 0.5);
        let color_variation = (pos_variation + time_variation) * 0.5 * params.color_intensity;

        // Smooth color blending
        let mut final_value = base_value + color_variation;

        // Enhanced blur calculation
        if params.blur > 0.0 {
            let x_fract = (x_rot * size_scaled).fract();
            let y_fract = (y_rot * size_scaled).fract();

            // Smoother blur animation
            let blur_amount = params.blur * (self.utils.fast_sin(time * PI) * 0.15 + 0.85);
            let blur_range = blur_amount * 0.5;
            let half = 0.5;

            // Calculate smooth distance to edges
            let x_dist = (x_fract - half).abs();
            let y_dist = (y_fract - half).abs();

            // Smooth step function for better transitions
            let x_blend =
                1.0 - PatternUtils::smoothstep_custom(blur_range * 0.8, blur_range, x_dist);
            let y_blend =
                1.0 - PatternUtils::smoothstep_custom(blur_range * 0.8, blur_range, y_dist);

            let x_smooth = PatternUtils::smoothstep(x_blend);
            let y_smooth = PatternUtils::smoothstep(y_blend);

            let blur_value =
                if ((x_rot * size_scaled).floor() as i32 + (y_rot * size_scaled).floor() as i32) & 1
                    == 0
                {
                    (1.0 - x_smooth) * (1.0 - y_smooth) + x_smooth * y_smooth
                } else {
                    x_smooth * (1.0 - y_smooth) + (1.0 - x_smooth) * y_smooth
                };

            // Smoother blend between original and blurred value
            final_value = final_value * (1.0 - blur_amount) + blur_value * blur_amount;
        }

        // Clamp with smooth edges
        final_value.clamp(0.0, 1.0)
    }
}

impl PatternUtils {
    #[inline(always)]
    pub fn smoothstep_custom(edge0: f64, edge1: f64, x: f64) -> f64 {
        let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}
