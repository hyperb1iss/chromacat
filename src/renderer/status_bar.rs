//! Status bar display for ChromaCat
//!
//! This module handles rendering of the status bar, which displays information
//! about current state, themes, patterns, controls, and performance metrics.

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, SetForegroundColor},
};

use super::error::RendererError;
use super::scroll::ScrollState;

/// Renders status and control information at the bottom of the screen.
#[derive(Debug)]
pub struct StatusBar {
    /// Terminal dimensions
    width: u16,
    /// Terminal height
    height: u16,
    /// Current theme name
    current_theme: String,
    /// Current pattern name
    current_pattern: String,
    /// Current FPS measurement
    fps: f64,
    /// Whether to show FPS counter
    show_fps: bool,
    /// Custom status text (for playlists)
    custom_text: Option<String>,
}

impl StatusBar {
    /// Creates a new status bar instance.
    pub fn new(term_size: (u16, u16)) -> Self {
        Self {
            width: term_size.0,
            height: term_size.1,
            current_theme: String::from("rainbow"),
            current_pattern: String::from("diagonal"),
            fps: 0.0,
            show_fps: true,
            custom_text: None,
        }
    }

    /// Updates the current theme name.
    pub fn set_theme(&mut self, theme: &str) {
        self.current_theme = theme.to_string();
    }

    /// Updates the current pattern name.
    pub fn set_pattern(&mut self, pattern: &str) {
        self.current_pattern = pattern.to_string();
    }

    /// Updates the current FPS measurement.
    pub fn set_fps(&mut self, fps: f64) {
        // Only update if change is significant
        if (self.fps - fps).abs() > 0.5 {
            self.fps = fps;
        }
    }

    /// Sets whether to show the FPS counter.
    pub fn show_fps(&mut self, show: bool) {
        self.show_fps = show;
    }

    /// Sets custom text to display in the status bar
    pub fn set_custom_text(&mut self, text: Option<&str>) {
        self.custom_text = text.map(|s| s.to_string());
    }

    /// Gets the custom text if any
    pub fn custom_text(&self) -> Option<&str> {
        self.custom_text.as_deref()
    }

    /// Renders the status bar to the terminal.
    pub fn render(
        &mut self,
        stdout: &mut std::io::StdoutLock,
        scroll: &ScrollState,
    ) -> Result<(), RendererError> {
        // Define colors for different sections
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

        // Draw separator line
        queue!(
            stdout,
            MoveTo(0, self.height - 2),
            Print("\x1b[K"),
            SetForegroundColor(separator_color),
            Print("─".repeat(self.width as usize))
        )?;

        let (start, end) = scroll.get_visible_range();

        // Build status sections
        let mut left_section = if let Some(text) = &self.custom_text {
            format!(" {} ", text)
        } else {
            format!(" {} • {}", self.current_theme, self.current_pattern)
        };
        if self.show_fps {
            left_section.push_str(&format!(" • {:.1} FPS", self.fps));
        }

        // Hide legacy middle hints when custom text is present (playground UI provides its own footer)
        let middle_section = if self.custom_text.is_some() {
            ""
        } else {
            "[T]heme [P]attern"
        };
        let right_section = format!(
            "Lines {}-{}/{}  [Q]uit ",
            start + 1,
            end,
            scroll.total_lines()
        );

        // Calculate section widths
        let total_width = self.width as usize;
        let left_width = left_section.chars().count();
        let middle_width = middle_section.chars().count();
        let right_width = right_section.chars().count();

        // Clear status bar line
        queue!(stdout, MoveTo(0, self.height - 1), Print("\x1b[K"))?;

        // Render sections based on available space
        let available_width = total_width.saturating_sub(2); // Leave 2 chars margin

        if middle_width > 0 && left_width + middle_width + right_width <= available_width {
            // Full render
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
            // Minimal render with truncation
            let max_width = available_width.saturating_sub(3);
            let mut minimal_info = format!(" {}…", self.current_theme);
            if minimal_info.chars().count() > max_width {
                minimal_info = format!(
                    " {}…",
                    self.current_theme
                        .chars()
                        .take(max_width - 2)
                        .collect::<String>()
                );
            }
            queue!(
                stdout,
                SetForegroundColor(accent_color),
                Print(minimal_info)
            )?;
        }

        // Reset color
        queue!(stdout, SetForegroundColor(Color::Reset))?;

        Ok(())
    }

    /// Updates status bar dimensions after terminal resize.
    pub fn resize(&mut self, new_size: (u16, u16)) {
        self.width = new_size.0;
        self.height = new_size.1;
    }

    /// Gets the current width
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Gets the current height
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Gets the current theme
    pub fn current_theme(&self) -> &str {
        &self.current_theme
    }

    /// Gets the current pattern
    pub fn current_pattern(&self) -> &str {
        &self.current_pattern
    }

    /// Gets the current FPS
    pub fn fps(&self) -> f64 {
        self.fps
    }

    /// Returns whether FPS display is enabled
    pub fn is_fps_shown(&self) -> bool {
        self.show_fps
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new((80, 24)) // Default terminal size
    }
}
