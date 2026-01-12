use crate::pattern::PatternEngine;
use crate::renderer::blend_engine::{BlendEngine, TransitionEffect};
/// Ratatui widget for rendering ChromaCat patterns
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

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

/// Perform gamma-correct color interpolation in linear space
/// This produces visually accurate color transitions
#[inline]
fn lerp_colors_gamma_correct(
    source: colorgrad::Color,
    target: colorgrad::Color,
    t: f32,
) -> colorgrad::Color {
    // Convert to linear RGB
    let s_r = srgb_to_linear(source.r);
    let s_g = srgb_to_linear(source.g);
    let s_b = srgb_to_linear(source.b);

    let t_r = srgb_to_linear(target.r);
    let t_g = srgb_to_linear(target.g);
    let t_b = srgb_to_linear(target.b);

    // Interpolate in linear space
    let l_r = s_r * (1.0 - t) + t_r * t;
    let l_g = s_g * (1.0 - t) + t_g * t;
    let l_b = s_b * (1.0 - t) + t_b * t;

    // Convert back to sRGB
    colorgrad::Color::new(
        linear_to_srgb(l_r),
        linear_to_srgb(l_g),
        linear_to_srgb(l_b),
        1.0,
    )
}

/// A widget that renders pattern-colored text
pub struct PatternWidget<'a> {
    /// The text content to render
    pub content: &'a str,
    /// The pattern engine for generating colors
    pub engine: &'a PatternEngine,
    /// Optional blend engine for transitions
    pub blend_engine: Option<&'a BlendEngine>,
    /// Transition effect to apply
    pub transition_effect: TransitionEffect,
    /// The current time for animation
    pub time: f64,
}

impl<'a> PatternWidget<'a> {
    pub fn new(content: &'a str, engine: &'a PatternEngine, time: f64) -> Self {
        Self {
            content,
            engine,
            blend_engine: None,
            transition_effect: TransitionEffect::Crossfade,
            time,
        }
    }

    pub fn with_blending(
        content: &'a str,
        engine: &'a PatternEngine,
        blend_engine: &'a BlendEngine,
        transition_effect: TransitionEffect,
        time: f64,
    ) -> Self {
        Self {
            content,
            engine,
            blend_engine: Some(blend_engine),
            transition_effect,
            time,
        }
    }
}

impl<'a> Widget for PatternWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Split content into lines
        let lines: Vec<&str> = self.content.lines().collect();
        let height = area.height as usize;
        let width = area.width as usize;

        for (y, line) in lines.iter().enumerate().take(height) {
            if y >= height {
                break;
            }

            // Calculate normalized y coordinate based on screen dimensions (-0.5 to 0.5)
            let norm_y = if height > 1 {
                (y as f64 / (height - 1) as f64) - 0.5
            } else {
                0.0
            };

            for (x_pos, ch) in line.chars().take(width).enumerate() {
                if x_pos >= width {
                    break;
                }

                // Calculate normalized x coordinate based on screen dimensions (-0.5 to 0.5)
                let norm_x = if width > 1 {
                    (x_pos as f64 / (width - 1) as f64) - 0.5
                } else {
                    0.0
                };

                // Get color from pattern engine or blend engine
                let color = if let Some(blend_engine) = self.blend_engine {
                    if blend_engine.is_transitioning() {
                        // Apply transition effect to get per-pixel spatial wipe factor
                        let base_blend = blend_engine.blend_factor();
                        let effect_blend = self
                            .transition_effect
                            .apply(norm_x + 0.5, norm_y + 0.5, self.time, base_blend);

                        // Get SEPARATE source and target pattern values
                        let source_value = blend_engine.get_source_value(norm_x, norm_y);
                        let target_value = blend_engine.get_target_value(norm_x, norm_y);

                        // Get colors from their respective gradients
                        let (source_color, target_color) = blend_engine
                            .get_source_target_colors(source_value as f32, target_value as f32);

                        // Gamma-correct interpolation using the spatial wipe factor
                        let color_value =
                            lerp_colors_gamma_correct(source_color, target_color, effect_blend);

                        Color::Rgb(
                            (color_value.r * 255.0) as u8,
                            (color_value.g * 255.0) as u8,
                            (color_value.b * 255.0) as u8,
                        )
                    } else {
                        // Use blend engine's current state
                        let value = blend_engine.get_blended_value(norm_x, norm_y);
                        let color_value = blend_engine.get_blended_color(value as f32);
                        Color::Rgb(
                            (color_value.r * 255.0) as u8,
                            (color_value.g * 255.0) as u8,
                            (color_value.b * 255.0) as u8,
                        )
                    }
                } else {
                    // Normal rendering without blending
                    let value = self
                        .engine
                        .get_value_at_normalized(norm_x, norm_y)
                        .unwrap_or(0.0);
                    let color_value = self.engine.gradient().at(value as f32);
                    Color::Rgb(
                        (color_value.r * 255.0) as u8,
                        (color_value.g * 255.0) as u8,
                        (color_value.b * 255.0) as u8,
                    )
                };

                // Set the character with color in the buffer
                if let Some(cell) = buf.cell_mut((area.x + x_pos as u16, area.y + y as u16)) {
                    cell.set_char(ch);
                    cell.set_style(Style::default().fg(color));
                }
            }
        }
    }
}
