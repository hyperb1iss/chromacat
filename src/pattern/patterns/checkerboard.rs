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
define_param!(num Checker, AnimModeParam, "anim_mode", "Animation mode: 0=static, 1=simple, 2=wave, 3=full", 0.0, 3.0, 3.0);

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
    /// Animation mode: 0=static, 1=simple (rotation/color), 2=wave (adds movement), 3=full
    pub anim_mode: u8,
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
    const ANIM_MODE_PARAM: CheckerAnimModeParam = CheckerAnimModeParam;
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
            anim_mode: 3, // Full animation by default
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
            "size={},blur={},rotation={},scale={},color_speed={},color_intensity={},wave_speed={},wave_intensity={},orbit_speed={},antialiasing={},time_scale={},anim_mode={}",
            self.size, self.blur, self.rotation, self.scale, self.color_speed, self.color_intensity, self.wave_speed, self.wave_intensity, self.orbit_speed, self.antialiasing, self.time_scale, self.anim_mode
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
            "anim_mode",
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
            "anim_mode" => Self::ANIM_MODE_PARAM.validate(kv[1]),
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
                "anim_mode" => {
                    Self::ANIM_MODE_PARAM.validate(kv[1])?;
                    params.anim_mode = kv[1].parse::<f64>().unwrap() as u8;
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
            Box::new(Self::ANIM_MODE_PARAM),
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
    ///
    /// Animation modes:
    /// - 0: Static - no animation, just static rotation
    /// - 1: Simple - slow rotation + color variation
    /// - 2: Wave - mode 1 + wave movement
    /// - 3: Full - all animations (wave, orbit, zoom, dynamic rotation)
    #[inline(always)]
    pub fn checkerboard(&self, x_norm: f64, y_norm: f64, params: CheckerboardParams) -> f64 {
        // Scale time for animations
        let time = self.time * params.time_scale;
        let mode = params.anim_mode;

        // Mode 0: Static - no time-based movement
        // Mode 1: Simple - only rotation animation
        // Mode 2: Wave - adds wave movement
        // Mode 3: Full - all animations (wave, orbit, zoom)

        // Calculate position movement based on mode
        let (x_moved, y_moved) = if mode >= 2 {
            // Wave movement (modes 2+)
            let wave_x =
                self.utils.fast_sin(time * params.wave_speed * PI * 0.5) * params.wave_intensity;
            let wave_y =
                self.utils.fast_cos(time * params.wave_speed * PI * 0.35) * params.wave_intensity;

            if mode >= 3 {
                // Full mode: add orbital movement
                let orbit_angle = time * params.orbit_speed * PI * 0.5;
                let orbit_radius = 0.3 * self.utils.fast_sin(time * PI * 0.25).abs();
                let orbit_x = self.utils.fast_cos(orbit_angle) * orbit_radius;
                let orbit_y = self.utils.fast_sin(orbit_angle) * orbit_radius;
                (x_norm + wave_x + orbit_x, y_norm + wave_y + orbit_y)
            } else {
                (x_norm + wave_x, y_norm + wave_y)
            }
        } else {
            // No position movement
            (x_norm, y_norm)
        };

        // Calculate scale based on mode
        let final_scale = if mode >= 3 {
            // Full mode: dynamic zoom
            let zoom_wave = self
                .utils
                .fast_sin((x_moved * 0.5 + y_moved * 0.5 + time * params.wave_speed) * PI)
                * 0.3
                + 1.0;
            params.scale * zoom_wave
        } else {
            params.scale
        };

        // Pre-calculate scaled coordinates
        let x_scaled = x_moved * final_scale;
        let y_scaled = y_moved * final_scale;

        // Calculate rotation based on mode
        let total_rotation = if mode == 0 {
            // Static: just use configured rotation
            params.rotation * (PI / 180.0)
        } else if mode >= 1 {
            // Simple+: animated rotation
            let base_rotation = params.rotation + (time * 45.0) % 360.0;
            let wave_rotation = if mode >= 3 {
                // Full mode: wave-based rotation wobble
                self.utils.fast_sin(time * params.wave_speed * PI) * 30.0
            } else {
                0.0
            };
            (base_rotation + wave_rotation) * (PI / 180.0)
        } else {
            params.rotation * (PI / 180.0)
        };

        let (sin_rot, cos_rot) = {
            let sin_val = self.utils.fast_sin(total_rotation);
            let cos_val = self.utils.fast_cos(total_rotation);
            (sin_val, cos_val)
        };

        // Combine rotation calculations
        let x_rot = x_scaled * cos_rot - y_scaled * sin_rot;
        let y_rot = x_scaled * sin_rot + y_scaled * cos_rot;

        // Calculate size with optional scale animation
        let size_scaled = if mode >= 3 {
            // Full mode: pulsing scale
            let scale_factor = self.utils.fast_sin(self.time * PI) * 0.2 + 1.0;
            params.size as f64 * scale_factor
        } else {
            params.size as f64
        };

        // Enhanced antialiasing calculation
        let aa_sample_count = 4;
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

                let base_value = if is_white { 1.0 } else { 0.0 };
                aa_value += base_value;
            }
        }

        // Average the antialiased samples
        let base_value = aa_value / (aa_sample_count * aa_sample_count) as f64;

        // Color variation (modes 1+)
        let color_variation = if mode >= 1 {
            let pos_variation = self.utils.fast_sin(x_rot * 0.25 + y_rot * 0.25);
            let time_variation = self.utils.fast_sin(time * params.color_speed * PI * 0.5);
            (pos_variation + time_variation) * 0.5 * params.color_intensity
        } else {
            // Static mode: position-based variation only
            self.utils.fast_sin(x_rot * 0.25 + y_rot * 0.25) * 0.5 * params.color_intensity
        };

        let mut final_value = base_value + color_variation;

        // Enhanced blur calculation
        if params.blur > 0.0 {
            let x_fract = (x_rot * size_scaled).fract();
            let y_fract = (y_rot * size_scaled).fract();

            // Blur animation only in full mode
            let blur_amount = if mode >= 3 {
                params.blur * (self.utils.fast_sin(time * PI) * 0.15 + 0.85)
            } else {
                params.blur
            };
            let blur_range = blur_amount * 0.5;
            let half = 0.5;

            let x_dist = (x_fract - half).abs();
            let y_dist = (y_fract - half).abs();

            let x_blend =
                1.0 - PatternUtils::smoothstep_custom(blur_range * 0.8, blur_range, x_dist);
            let y_blend =
                1.0 - PatternUtils::smoothstep_custom(blur_range * 0.8, blur_range, y_dist);

            let x_smooth = PatternUtils::smoothstep(x_blend);
            let y_smooth = PatternUtils::smoothstep(y_blend);

            let blur_value = if ((x_rot * size_scaled).floor() as i32
                + (y_rot * size_scaled).floor() as i32)
                & 1
                == 0
            {
                (1.0 - x_smooth) * (1.0 - y_smooth) + x_smooth * y_smooth
            } else {
                x_smooth * (1.0 - y_smooth) + (1.0 - x_smooth) * y_smooth
            };

            final_value = final_value * (1.0 - blur_amount) + blur_value * blur_amount;
        }

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
