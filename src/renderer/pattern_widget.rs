use crate::pattern::PatternEngine;
use crate::renderer::blend_engine::{BlendEngine, TransitionEffect};
/// Ratatui widget for rendering ChromaCat patterns
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

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
                        // Apply transition effect to the blend
                        let base_blend = blend_engine.blend_factor();
                        let effect_blend = self
                            .transition_effect
                            .apply(norm_x, norm_y, self.time, base_blend);

                        // Get blended pattern value
                        let value = blend_engine.get_blended_value(norm_x, norm_y);

                        // Get blended color
                        let color_value = if effect_blend > 0.5 {
                            // Use blended gradient color
                            blend_engine.get_blended_color(value as f32)
                        } else {
                            // Use source gradient
                            self.engine.gradient().at(value as f32)
                        };

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
