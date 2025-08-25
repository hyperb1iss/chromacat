use crate::define_param;
use crate::pattern::params::{ParamType, PatternParam};
use std::any::Any;
use std::f64::consts::PI;

// Parameter definitions with clear descriptions
define_param!(num Kaleidoscope, SegmentsParam, "segments", "Number of symmetrical mirror segments in the pattern", 3.0, 12.0, 6.0);
define_param!(num Kaleidoscope, RotationSpeedParam, "rotation_speed", "Speed of pattern rotation and animation", 0.1, 5.0, 1.0);
define_param!(num Kaleidoscope, ZoomParam, "zoom", "Overall scale of the pattern", 0.5, 3.0, 1.0);
define_param!(num Kaleidoscope, ComplexityParam, "complexity", "Amount of geometric detail and layering", 1.0, 5.0, 2.0);
define_param!(num Kaleidoscope, ColorFlowParam, "color_flow", "Speed of color transitions and flow effects", 0.0, 2.0, 1.0);
define_param!(num Kaleidoscope, DistortionParam, "distortion", "Amount of organic distortion applied to the geometric pattern", 0.0, 1.0, 0.3);

/// Parameters for configuring the kaleidoscope pattern effect.
/// Creates a mesmerizing symmetrical pattern with dynamic animations
/// and organic distortions.
#[derive(Debug, Clone)]
pub struct KaleidoscopeParams {
    /// Number of mirror segments (3-12). Higher values create more intricate symmetry.
    pub segments: u32,
    /// Speed of pattern rotation (0.1-5.0). Controls both base rotation and secondary animations.
    pub rotation_speed: f64,
    /// Pattern zoom level (0.5-3.0). Affects the overall scale of the pattern.
    pub zoom: f64,
    /// Pattern detail level (1.0-5.0). Higher values add more geometric complexity and layers.
    pub complexity: f64,
    /// Speed of color transitions (0.0-2.0). Controls flow effects and color movement.
    pub color_flow: f64,
    /// Amount of pattern distortion (0.0-1.0). Adds organic movement to the geometric base.
    pub distortion: f64,
}

impl KaleidoscopeParams {
    const SEGMENTS_PARAM: KaleidoscopeSegmentsParam = KaleidoscopeSegmentsParam;
    const ROTATION_SPEED_PARAM: KaleidoscopeRotationSpeedParam = KaleidoscopeRotationSpeedParam;
    const ZOOM_PARAM: KaleidoscopeZoomParam = KaleidoscopeZoomParam;
    const COMPLEXITY_PARAM: KaleidoscopeComplexityParam = KaleidoscopeComplexityParam;
    const COLOR_FLOW_PARAM: KaleidoscopeColorFlowParam = KaleidoscopeColorFlowParam;
    const DISTORTION_PARAM: KaleidoscopeDistortionParam = KaleidoscopeDistortionParam;
}

impl Default for KaleidoscopeParams {
    fn default() -> Self {
        Self {
            segments: 6,
            rotation_speed: 1.0,
            zoom: 1.0,
            complexity: 2.0,
            color_flow: 1.0,
            distortion: 0.3,
        }
    }
}

// Use the validate macro to implement validation
define_param!(validate KaleidoscopeParams,
    SEGMENTS_PARAM: KaleidoscopeSegmentsParam,
    ROTATION_SPEED_PARAM: KaleidoscopeRotationSpeedParam,
    ZOOM_PARAM: KaleidoscopeZoomParam,
    COMPLEXITY_PARAM: KaleidoscopeComplexityParam,
    COLOR_FLOW_PARAM: KaleidoscopeColorFlowParam,
    DISTORTION_PARAM: KaleidoscopeDistortionParam
);

impl PatternParam for KaleidoscopeParams {
    fn name(&self) -> &'static str {
        "kaleidoscope"
    }

    fn description(&self) -> &'static str {
        "Mesmerizing kaleidoscope pattern with mirror segments"
    }

    fn param_type(&self) -> ParamType {
        ParamType::Composite
    }

    fn default_value(&self) -> String {
        format!(
            "segments={},rotation_speed={},zoom={},complexity={},color_flow={},distortion={}",
            self.segments,
            self.rotation_speed,
            self.zoom,
            self.complexity,
            self.color_flow,
            self.distortion
        )
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        self.validate_params(value)
    }

    fn parse(&self, value: &str) -> Result<Box<dyn PatternParam>, String> {
        let mut params = KaleidoscopeParams::default();

        for part in value.split(',') {
            let kv: Vec<&str> = part.split('=').collect();
            if kv.len() != 2 {
                continue;
            }

            match kv[0] {
                "segments" => {
                    Self::SEGMENTS_PARAM.validate(kv[1])?;
                    params.segments = kv[1].parse().unwrap();
                }
                "rotation_speed" => {
                    Self::ROTATION_SPEED_PARAM.validate(kv[1])?;
                    params.rotation_speed = kv[1].parse().unwrap();
                }
                "zoom" => {
                    Self::ZOOM_PARAM.validate(kv[1])?;
                    params.zoom = kv[1].parse().unwrap();
                }
                "complexity" => {
                    Self::COMPLEXITY_PARAM.validate(kv[1])?;
                    params.complexity = kv[1].parse().unwrap();
                }
                "color_flow" => {
                    Self::COLOR_FLOW_PARAM.validate(kv[1])?;
                    params.color_flow = kv[1].parse().unwrap();
                }
                "distortion" => {
                    Self::DISTORTION_PARAM.validate(kv[1])?;
                    params.distortion = kv[1].parse().unwrap();
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
            Box::new(Self::SEGMENTS_PARAM),
            Box::new(Self::ROTATION_SPEED_PARAM),
            Box::new(Self::ZOOM_PARAM),
            Box::new(Self::COMPLEXITY_PARAM),
            Box::new(Self::COLOR_FLOW_PARAM),
            Box::new(Self::DISTORTION_PARAM),
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
    /// Generates a kaleidoscope pattern with mirror segments and dynamic effects.
    ///
    /// The pattern combines several elements:
    /// - Symmetrical mirroring around a central point
    /// - Multiple layers of geometric patterns (spirals, rings, hexagons)
    /// - Organic distortions and flow effects
    /// - Dynamic animations and color transitions
    ///
    /// # Arguments
    /// * `x_norm` - Normalized x coordinate (-0.5 to 0.5)
    /// * `y_norm` - Normalized y coordinate (-0.5 to 0.5)
    /// * `params` - Configuration parameters for the pattern
    ///
    /// # Returns
    /// A value between 0.0 and 1.0 representing the pattern intensity at the given point
    #[inline(always)]
    pub fn kaleidoscope(&self, x_norm: f64, y_norm: f64, params: KaleidoscopeParams) -> f64 {
        // Handle static preview mode
        let y_pos = if self.time == 0.0 {
            // Create a repeating pattern for static preview
            (y_norm + 0.5).rem_euclid(0.3) * 3.0 - 0.5
        } else {
            y_norm
        };

        // Pre-calculate time-based values for consistent animation
        let base_time = self.time * params.rotation_speed * 0.5;
        let flow_time = self.time * params.color_flow * 0.3;

        // Cache frequently used trigonometric values
        let (time_sin, time_cos) = {
            let t = base_time * PI;
            (self.utils.fast_sin(t), self.utils.fast_cos(t))
        };

        let (flow_sin, flow_cos) = {
            let t = flow_time * PI;
            (self.utils.fast_sin(t), self.utils.fast_cos(t))
        };

        // Transform input coordinates with zoom and aspect ratio correction
        let x = x_norm * params.zoom;
        let y = y_pos * params.zoom * self.char_aspect_ratio; // Apply aspect ratio to y

        // Calculate polar coordinates for radial effects
        let (angle, distance) = {
            let angle = y.atan2(x);
            let dist = (x * x + y * y).sqrt();
            (angle, dist)
        };

        // Symmetry calculation
        let segment_angle = 2.0 * PI / params.segments as f64;
        let mut mirrored_angle = angle.rem_euclid(segment_angle);
        if mirrored_angle > segment_angle * 0.5 {
            mirrored_angle = segment_angle - mirrored_angle;
        }

        // Combine rotations for smooth animation
        let total_angle = mirrored_angle
            + base_time * PI * 0.3   // Base rotation
            + time_sin * 0.2; // Secondary wobble

        // Initialize pattern accumulator
        let mut value = 0.0;
        let complexity = params.complexity.min(5.0); // Limit for performance

        // Generate spiral components with varying frequencies
        let spiral_base = total_angle + distance * 2.0 + base_time;
        for i in 0..(complexity as i32) {
            let i_f = i as f64;
            let freq = 1.0 + i_f * 0.7;
            let phase = base_time * (0.8 + i_f * 0.3);
            value += self.utils.fast_sin(spiral_base * freq + phase) * (0.4 / (i_f + 1.0));
        }

        // Add concentric ring pattern
        let ring_phase = distance * 6.0 * complexity - base_time;
        value += self.utils.fast_sin(ring_phase) * 0.4;

        // Add geometric hexagonal grid pattern
        let geo_scale = complexity * 2.0;
        let geo_time = base_time * 0.5;

        let hex_coords = {
            let hx = x * geo_scale * 1.732 + geo_time;
            let hy = y * geo_scale * 2.0 + geo_time;
            let hz = (hx - hy * 0.577) + geo_time;
            (self.utils.fast_sin(hx) * self.utils.fast_sin(hz) * self.utils.fast_sin(hy * 1.155))
                * 0.3
        };
        value += hex_coords;

        // Add mandala-like circular patterns
        let mandala_base = total_angle * 4.0 + base_time;
        for i in 0..2 {
            let i_f = i as f64;
            let radius = distance * (3.0 + i_f) + base_time * (0.5 + i_f * 0.2);
            let angular = mandala_base * (1.0 + i_f * 0.5);
            value += self.utils.fast_sin(radius) * self.utils.fast_cos(angular) * 0.25;
        }

        // Add Perlin noise-based distortion
        if params.distortion > 0.001 {
            let noise_scale = 3.0 * complexity;
            let noise = self.utils.noise2d(
                x * noise_scale + base_time * 0.7,
                y * noise_scale - base_time * 0.5,
            );
            value += noise * params.distortion * 0.6;
        }

        // Add flowing color transitions
        let flow =
            flow_sin * 0.25 * (1.0 + distance * 2.0) + flow_cos * 0.15 * (1.0 + distance * 3.0);
        value += flow;

        // Apply distance-based intensity falloff
        let intensity = (-distance * 0.6).exp() * 1.4;
        value *= intensity;

        // Add subtle pulsing animation
        let pulse = (time_sin * 1.5 * 0.15 + time_cos * 0.8 * 0.1) * (1.0 - distance);
        value += pulse;

        // Add edge highlighting along segment boundaries
        let edge = (1.0 - (distance * params.segments as f64 * 0.5).fract()) * 0.15 * intensity;
        value += edge;

        // Final value adjustments and normalization
        value = value * 0.6 + 0.5;
        value = value.powf(0.9);

        // Clamp to valid range while maintaining smooth transitions
        value = value.clamp(0.05, 0.95);
        (value - 0.05) / 0.9
    }
}
