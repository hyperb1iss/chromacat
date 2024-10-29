//! Status bar rendering
//!
//! This module handles the rendering of the status bar, which displays
//! information about scroll position, line counts, themes, patterns and controls.

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, SetForegroundColor},
};

use super::error::RendererError;
use super::scroll::ScrollState;

/// Renders the status bar with animation progress and controls
pub struct StatusBar {
    /// Terminal dimensions
    width: u16,
    height: u16,
    /// Current theme name
    current_theme: String,
    /// Current pattern name
    current_pattern: String,
}

impl StatusBar {
    /// Creates a new status bar
    pub fn new(term_size: (u16, u16)) -> Self {
        Self {
            width: term_size.0,
            height: term_size.1,
            current_theme: String::from("rainbow"),
            current_pattern: String::from("diagonal"),
        }
    }

    /// Updates the current theme name
    pub fn set_theme(&mut self, theme: &str) {
        self.current_theme = theme.to_string();
    }

    /// Updates the current pattern name
    pub fn set_pattern(&mut self, pattern: &str) {
        self.current_pattern = pattern.to_string();
    }

    /// Renders the status bar
    pub fn render(
        &self,
        stdout: &mut std::io::StdoutLock,
        scroll: &ScrollState,
    ) -> Result<(), RendererError> {
        // Draw separator line
        queue!(
            stdout,
            MoveTo(0, self.height - 2),
            Print("\x1b[K"),
            SetForegroundColor(Color::DarkGrey),
            Print("─".repeat(self.width as usize))
        )?;

        let (start, end) = scroll.get_visible_range();

        // Calculate available width
        let total_width = self.width as usize;

        // Build compact status sections
        let left_section = format!("😺 {}│{}", self.current_theme, self.current_pattern);
        let middle_section = "[t]theme [p]pat";
        let right_section = format!(
            "↑↓scroll│q quit│{}-{}/{}",
            start + 1,
            end,
            scroll.total_lines
        );

        // Calculate section widths
        let left_width = left_section.chars().count();
        let middle_width = middle_section.chars().count();
        let right_width = right_section.chars().count();

        // If total width is too small, prioritize important info
        let available_width = total_width.saturating_sub(2); // Leave 2 chars margin

        queue!(
            stdout,
            MoveTo(0, self.height - 1),
            Print("\x1b[K"),
            SetForegroundColor(Color::Rgb {
                r: 255,
                g: 182,
                b: 193
            }),
        )?;

        // Render sections based on available space
        if left_width + middle_width + right_width <= available_width {
            // Full render - everything fits
            queue!(
                stdout,
                Print(&left_section),
                SetForegroundColor(Color::Rgb {
                    r: 180,
                    g: 180,
                    b: 180
                }),
                Print(" "),
                Print(middle_section),
                Print(" "),
                SetForegroundColor(Color::Rgb {
                    r: 150,
                    g: 150,
                    b: 150
                }),
                MoveTo(
                    self.width.saturating_sub(right_width as u16),
                    self.height - 1
                ),
                Print(right_section),
            )?;
        } else if left_width + right_width <= available_width {
            // Medium render - skip middle section
            queue!(
                stdout,
                Print(&left_section),
                Print(" "),
                SetForegroundColor(Color::Rgb {
                    r: 150,
                    g: 150,
                    b: 150
                }),
                MoveTo(
                    self.width.saturating_sub(right_width as u16),
                    self.height - 1
                ),
                Print(right_section),
            )?;
        } else {
            // Minimal render - just theme/pattern
            let max_left = available_width.saturating_sub(3); // Leave room for ellipsis
            let truncated = left_section.chars().take(max_left).collect::<String>();
            queue!(stdout, Print(truncated), Print("..."),)?;
        }

        Ok(())
    }

    /// Updates dimensions after terminal resize
    pub fn resize(&mut self, new_size: (u16, u16)) {
        self.width = new_size.0;
        self.height = new_size.1;
    }
}
