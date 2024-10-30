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
        // Define colors
        let separator_color = Color::Rgb {
            r: 40,
            g: 44,
            b: 52,
        };
        let accent_color = Color::Rgb {
            r: 97,
            g: 175,
            b: 239,
        };
        let text_color = Color::Rgb {
            r: 171,
            g: 178,
            b: 191,
        };
        let muted_color = Color::Rgb {
            r: 92,
            g: 99,
            b: 112,
        };

        // Draw separator line with a cleaner look
        queue!(
            stdout,
            MoveTo(0, self.height - 2),
            Print("\x1b[K"),
            SetForegroundColor(separator_color),
            Print("─".repeat(self.width as usize))
        )?;

        let (start, end) = scroll.get_visible_range();

        // Calculate available width
        let total_width = self.width as usize;

        // Build status sections with improved formatting
        let left_section = format!(" {} • {}", self.current_theme, self.current_pattern);
        let middle_section = "[T]heme [P]attern";
        let right_section = format!(
            "Lines {}-{}/{}  [Q]uit ",
            start + 1,
            end,
            scroll.total_lines
        );

        // Calculate section widths
        let left_width = left_section.chars().count();
        let middle_width = middle_section.chars().count();
        let right_width = right_section.chars().count();

        // Clear status bar line
        queue!(stdout, MoveTo(0, self.height - 1), Print("\x1b[K"),)?;

        // Render sections based on available space
        let available_width = total_width.saturating_sub(2); // Leave 2 chars margin

        if left_width + middle_width + right_width <= available_width {
            // Full render - everything fits
            queue!(
                stdout,
                SetForegroundColor(accent_color),
                Print(&left_section),
                SetForegroundColor(text_color),
                MoveTo(
                    (self.width as usize / 2 - middle_width / 2) as u16,
                    self.height - 1
                ),
                Print(middle_section),
                SetForegroundColor(muted_color),
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
                SetForegroundColor(accent_color),
                Print(&left_section),
                SetForegroundColor(muted_color),
                MoveTo(
                    self.width.saturating_sub(right_width as u16),
                    self.height - 1
                ),
                Print(right_section),
            )?;
        } else {
            // Minimal render - just essential info
            let max_width = available_width.saturating_sub(3);
            let minimal_info = format!(" {}…", self.current_theme);
            let truncated = minimal_info.chars().take(max_width).collect::<String>();
            queue!(stdout, SetForegroundColor(accent_color), Print(truncated),)?;
        }

        // Reset color at the end
        queue!(stdout, SetForegroundColor(Color::Reset))?;

        Ok(())
    }

    /// Updates dimensions after terminal resize
    pub fn resize(&mut self, new_size: (u16, u16)) {
        self.width = new_size.0;
        self.height = new_size.1;
    }
}
