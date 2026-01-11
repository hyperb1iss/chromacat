use crate::pattern::{PatternConfig, PatternEngine, REGISTRY};
use crate::themes;
use colorgrad::{Color, Gradient};
/// Blend engine for smooth transitions between patterns, themes, and parameters
///
/// This module provides real-time blending and interpolation between different
/// visual states, creating smooth transitions instead of jarring switches.
use std::sync::Arc;

/// Convert sRGB component to linear RGB (gamma decode)
/// Uses the precise sRGB transfer function
#[inline]
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert linear RGB component to sRGB (gamma encode)
/// Uses the precise sRGB transfer function
#[inline]
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Create a simple fallback gradient for when theme loading fails
/// Uses a basic magenta-to-cyan gradient that won't panic
fn create_fallback_gradient() -> Box<dyn Gradient + Send + Sync> {
    Box::new(
        colorgrad::GradientBuilder::new()
            .colors(&[
                colorgrad::Color::from_rgba8(255, 0, 128, 255),
                colorgrad::Color::from_rgba8(0, 128, 255, 255),
            ])
            .build::<colorgrad::LinearGradient>()
            .expect("simple two-color gradient should always build"),
    )
}

/// Blending state for smooth transitions
pub struct BlendEngine {
    /// Source pattern engine
    source_engine: Option<PatternEngine>,
    /// Target pattern engine
    target_engine: Option<PatternEngine>,
    /// Source gradient for blending
    source_gradient: Option<Arc<Box<dyn Gradient + Send + Sync>>>,
    /// Target gradient for blending
    target_gradient: Option<Arc<Box<dyn Gradient + Send + Sync>>>,
    /// Current blend factor (0.0 = source, 1.0 = target)
    blend_factor: f32,
    /// Whether a transition is active
    transitioning: bool,
    /// Transition speed (blend units per second)
    transition_speed: f32,
}

impl BlendEngine {
    /// Create a new blend engine
    pub fn new() -> Self {
        Self {
            source_engine: None,
            target_engine: None,
            source_gradient: None,
            target_gradient: None,
            blend_factor: 0.0,
            transitioning: false,
            transition_speed: 0.2, // 5 seconds for testing
        }
    }
    
    /// Initialize with an engine
    pub fn with_engine(engine: PatternEngine) -> Self {
        let mut blend = Self::new();
        blend.source_engine = Some(engine);
        blend
    }

    /// Start a pattern transition
    pub fn start_pattern_transition(
        &mut self,
        current_engine: PatternEngine,
        current_gradient: Arc<Box<dyn Gradient + Send + Sync>>,
        new_pattern: &str,
        width: usize,
        height: usize,
    ) -> Result<(), String> {
        // Allow overlapping transitions for chill vibes
        // Create new pattern config
        let params = REGISTRY
            .create_pattern_params(new_pattern)
            .ok_or_else(|| format!("Unknown pattern: {new_pattern}"))?;

        let config = PatternConfig {
            common: current_engine.config().common.clone(),
            params,
        };

        // Create target engine with a gradient
        // Try to get rainbow theme, fall back to simple gradient if unavailable
        let gradient_for_engine = themes::get_theme("rainbow")
            .ok()
            .and_then(|t| t.create_gradient().ok())
            .unwrap_or_else(create_fallback_gradient);

        let target_engine = PatternEngine::new(gradient_for_engine, config, width, height);

        // Set up transition - preserve the current gradient for blending
        self.source_gradient = Some(current_gradient);
        self.target_gradient = self.source_gradient.clone();

        self.source_engine = Some(current_engine);
        self.target_engine = Some(target_engine);
        self.blend_factor = 0.0;
        self.transitioning = true;

        Ok(())
    }

    /// Start a theme transition
    pub fn start_theme_transition(
        &mut self,
        current_gradient: Arc<Box<dyn Gradient + Send + Sync>>,
        new_theme: &str,
    ) -> Result<(), String> {
        // Allow overlapping transitions for chill vibes
        let new_gradient = themes::get_theme(new_theme)
            .map_err(|e| e.to_string())?
            .create_gradient()
            .map_err(|e| e.to_string())?;

        // Store gradients for blending
        self.source_gradient = Some(current_gradient);
        self.target_gradient = Some(Arc::new(new_gradient));

        self.blend_factor = 0.0;
        self.transitioning = true;

        Ok(())
    }

    /// Update the blend state
    pub fn update(&mut self, delta: f64) {
        if !self.transitioning {
            return;
        }

        // Update blend factor
        self.blend_factor += self.transition_speed * delta as f32;

        if self.blend_factor >= 1.0 {
            self.blend_factor = 1.0;
            self.transitioning = false;

            // Complete transition - move target to source
            if let Some(target) = self.target_engine.take() {
                self.source_engine = Some(target);
            }
            // Also update gradients
            if let Some(target_grad) = self.target_gradient.take() {
                self.source_gradient = Some(target_grad);
            }
        }

        // Update pattern engines
        if let Some(ref mut source) = self.source_engine {
            source.update(delta);
        }
        if let Some(ref mut target) = self.target_engine {
            target.update(delta);
        }
    }

    /// Get blended pattern value at normalized coordinates
    pub fn get_blended_value(&self, x: f64, y: f64) -> f64 {
        match (&self.source_engine, &self.target_engine) {
            (Some(source), Some(target)) if self.transitioning => {
                let source_val = source.get_value_at_normalized(x, y).unwrap_or(0.0);
                let target_val = target.get_value_at_normalized(x, y).unwrap_or(0.0);

                // Smooth interpolation with easing
                let eased_blend = ease_in_out_cubic(self.blend_factor);
                source_val * (1.0 - eased_blend as f64) + target_val * eased_blend as f64
            }
            (Some(source), _) => source.get_value_at_normalized(x, y).unwrap_or(0.0),
            _ => 0.0,
        }
    }

    /// Get the current gradient (possibly blended)
    pub fn get_gradient(&self) -> Arc<Box<dyn Gradient + Send + Sync>> {
        // For now, return target or source gradient without actual blending
        // Real gradient blending would require sampling and recreating
        if self.transitioning && self.blend_factor > 0.5 {
            self.target_gradient.clone().unwrap_or_else(|| {
                Arc::new(
                    themes::get_theme("rainbow")
                        .ok()
                        .and_then(|t| t.create_gradient().ok())
                        .unwrap_or_else(create_fallback_gradient),
                )
            })
        } else if let Some(ref source) = self.source_gradient {
            source.clone()
        } else {
            // Fallback gradient - safe, no panics
            Arc::new(
                themes::get_theme("rainbow")
                    .ok()
                    .and_then(|t| t.create_gradient().ok())
                    .unwrap_or_else(create_fallback_gradient),
            )
        }
    }

    /// Get blended color between two gradients with gamma-correct interpolation
    pub fn get_blended_color(&self, value: f32) -> Color {
        if let (Some(ref source), Some(ref target)) = (&self.source_gradient, &self.target_gradient)
        {
            let source_color = source.at(value);
            let target_color = target.at(value);

            // Gamma-correct interpolation for perceptually accurate blending
            let blend = self.blend_factor;

            // Convert to linear RGB
            let s_r = srgb_to_linear(source_color.r);
            let s_g = srgb_to_linear(source_color.g);
            let s_b = srgb_to_linear(source_color.b);

            let t_r = srgb_to_linear(target_color.r);
            let t_g = srgb_to_linear(target_color.g);
            let t_b = srgb_to_linear(target_color.b);

            // Interpolate in linear space
            let l_r = s_r * (1.0 - blend) + t_r * blend;
            let l_g = s_g * (1.0 - blend) + t_g * blend;
            let l_b = s_b * (1.0 - blend) + t_b * blend;
            let l_a = source_color.a * (1.0 - blend) + target_color.a * blend;

            // Convert back to sRGB
            Color::new(
                linear_to_srgb(l_r),
                linear_to_srgb(l_g),
                linear_to_srgb(l_b),
                l_a,
            )
        } else if let Some(ref source) = &self.source_gradient {
            source.at(value)
        } else {
            // Fallback color
            Color::from_rgba8(255, 0, 0, 255)
        }
    }

    /// Check if currently transitioning
    pub fn is_transitioning(&self) -> bool {
        self.transitioning
    }

    /// Get current blend factor
    pub fn blend_factor(&self) -> f32 {
        self.blend_factor
    }

    /// Set transition speed (0.1 = slow, 1.0 = fast)
    pub fn set_transition_speed(&mut self, speed: f32) {
        self.transition_speed = speed.clamp(0.1, 2.0);
    }
    
    /// Get the current engine (source or completed transition)
    pub fn get_current_engine(&self) -> Option<&PatternEngine> {
        self.source_engine.as_ref()
    }
    
    /// Check if we should update the main engine
    pub fn should_update_main_engine(&self) -> bool {
        !self.transitioning && self.source_engine.is_some()
    }
}

/// Parameter interpolation for smooth morphing
/// Infrastructure for future ShowcaseSequence param application
#[allow(dead_code)]
pub struct ParameterInterpolator {
    /// Source parameter values
    source_params: Vec<(String, f64)>,
    /// Target parameter values
    target_params: Vec<(String, f64)>,
    /// Current interpolated values
    current_params: Vec<(String, f64)>,
}

impl Default for ParameterInterpolator {
    fn default() -> Self {
        Self::new()
    }
}

impl ParameterInterpolator {
    /// Create a new parameter interpolator
    pub fn new() -> Self {
        Self {
            source_params: Vec::new(),
            target_params: Vec::new(),
            current_params: Vec::new(),
        }
    }

    /// Set source parameters
    pub fn set_source(&mut self, params: Vec<(String, f64)>) {
        self.source_params = params;
        self.update_current(0.0);
    }

    /// Set target parameters
    pub fn set_target(&mut self, params: Vec<(String, f64)>) {
        self.target_params = params;
    }

    /// Update interpolated values based on blend factor
    pub fn update_current(&mut self, blend: f32) {
        self.current_params.clear();

        // Find matching parameters and interpolate
        for (source_name, source_val) in &self.source_params {
            if let Some((_, target_val)) = self
                .target_params
                .iter()
                .find(|(name, _)| name == source_name)
            {
                // Interpolate matching parameters
                let interpolated = source_val * (1.0 - blend as f64) + target_val * blend as f64;
                self.current_params
                    .push((source_name.clone(), interpolated));
            } else {
                // Fade out parameters that don't exist in target
                let faded = source_val * (1.0 - blend as f64);
                self.current_params.push((source_name.clone(), faded));
            }
        }

        // Fade in new parameters from target
        for (target_name, target_val) in &self.target_params {
            if !self
                .source_params
                .iter()
                .any(|(name, _)| name == target_name)
            {
                let faded = target_val * blend as f64;
                self.current_params.push((target_name.clone(), faded));
            }
        }
    }

    /// Get current interpolated parameters
    pub fn get_current(&self) -> &[(String, f64)] {
        &self.current_params
    }
}

/// Transition effect types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransitionEffect {
    /// Simple crossfade
    Crossfade,
    /// Ripple outward from center
    Ripple,
    /// Spiral transition
    Spiral,
    /// Wave sweep
    Wave,
    /// Pixelate effect
    Pixelate,
    /// Kaleidoscope rotation
    Kaleidoscope,
}

impl TransitionEffect {
    /// Apply transition effect to blend calculation
    pub fn apply(&self, x: f64, y: f64, time: f64, base_blend: f32) -> f32 {
        match self {
            Self::Crossfade => base_blend,

            Self::Ripple => {
                // Ripple from center
                let cx = 0.5;
                let cy = 0.5;
                let dist = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
                let ripple_progress = base_blend as f64 * 2.0; // Expand ripple faster

                if dist < ripple_progress {
                    1.0
                } else if dist < ripple_progress + 0.1 {
                    // Smooth edge
                    ((ripple_progress + 0.1 - dist) / 0.1).clamp(0.0, 1.0) as f32
                } else {
                    0.0
                }
            }

            Self::Spiral => {
                // Spiral wipe
                let cx = 0.5;
                let cy = 0.5;
                let angle = (y - cy).atan2(x - cx);
                let dist = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
                let spiral_angle = angle + dist * 10.0 - time * 2.0;
                let spiral_progress = (spiral_angle.sin() * 0.5 + 0.5) as f32;

                (base_blend * 2.0 - (1.0 - spiral_progress)).clamp(0.0, 1.0)
            }

            Self::Wave => {
                // Wave sweep from left to right
                let wave_pos = base_blend as f64 * 1.2 - 0.1;
                let wave_width = 0.2;

                if x < wave_pos - wave_width {
                    1.0
                } else if x > wave_pos + wave_width {
                    0.0
                } else {
                    let wave_blend = (x - (wave_pos - wave_width)) / (wave_width * 2.0);
                    (1.0 - wave_blend).clamp(0.0, 1.0) as f32
                }
            }

            Self::Pixelate => {
                // Pixelate transition
                let pixel_size = 0.05 * (1.0 - base_blend as f64) + 0.001;
                let px = (x / pixel_size).floor() * pixel_size;
                let py = (y / pixel_size).floor() * pixel_size;

                // Use pixel position for threshold
                let threshold = ((px * 31.0 + py * 17.0) % 1.0) as f32;
                if base_blend > threshold {
                    1.0
                } else {
                    0.0
                }
            }

            Self::Kaleidoscope => {
                // Rotating kaleidoscope effect
                let cx = 0.5;
                let cy = 0.5;
                let angle = (y - cy).atan2(x - cx) + time;
                let segments = 8.0;
                let segment_angle = (angle * segments / (2.0 * std::f64::consts::PI)) % 1.0;

                if segment_angle < base_blend as f64 {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
}

/// Easing function for smooth transitions
fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

impl Default for BlendEngine {
    fn default() -> Self {
        Self::new()
    }
}
