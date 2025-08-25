use crate::pattern::PatternEngine;
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
    /// The current time for animation
    pub time: f64,
}

impl<'a> PatternWidget<'a> {
    pub fn new(content: &'a str, engine: &'a PatternEngine, time: f64) -> Self {
        Self {
            content,
            engine,
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

            let mut x_pos = 0;
            for ch in line.chars().take(width) {
                if x_pos >= width {
                    break;
                }

                // Calculate normalized x coordinate based on screen dimensions (-0.5 to 0.5)
                let norm_x = if width > 1 {
                    (x_pos as f64 / (width - 1) as f64) - 0.5
                } else {
                    0.0
                };

                // Get color from pattern engine
                let value = self
                    .engine
                    .get_value_at_normalized(norm_x, norm_y)
                    .unwrap_or(0.0);
                let color_value = self.engine.gradient().at(value as f32);
                let color = Color::Rgb(
                    (color_value.r * 255.0) as u8,
                    (color_value.g * 255.0) as u8,
                    (color_value.b * 255.0) as u8,
                );

                // Set the character with color in the buffer
                if let Some(cell) = buf.cell_mut((area.x + x_pos as u16, area.y + y as u16)) {
                    cell.set_char(ch);
                    cell.set_style(Style::default().fg(color));
                }

                x_pos += 1;
            }
        }
    }
}
