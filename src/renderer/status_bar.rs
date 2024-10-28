//! Status bar rendering
//!
//! This module handles the rendering of the status bar, which displays
//! information about scroll position, line counts, and available commands.

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, SetForegroundColor},
};

use super::error::RendererError;
use super::scroll::ScrollState;

/// Renders the status bar with animation progress and controls
pub struct StatusBar {
    height: u16,
    width: u16,
}

impl StatusBar {
    /// Creates a new status bar
    pub fn new(term_size: (u16, u16)) -> Self {
        Self {
            width: term_size.0,
            height: term_size.1,
        }
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

        // Create status line with multiple segments
        queue!(
            stdout,
            MoveTo(0, self.height - 1),
            Print("\x1b[K"),
            SetForegroundColor(Color::Rgb {
                r: 255,
                g: 182,
                b: 193
            }),
            Print("😺 "),
            SetForegroundColor(Color::Rgb {
                r: 100,
                g: 100,
                b: 100
            }),
            Print("Lines "),
            SetForegroundColor(Color::Rgb {
                r: 200,
                g: 200,
                b: 200
            }),
            Print(format!("{}-{}", start + 1, end)),
            SetForegroundColor(Color::Rgb {
                r: 100,
                g: 100,
                b: 100
            }),
            Print("/"),
            SetForegroundColor(Color::Rgb {
                r: 200,
                g: 200,
                b: 200
            }),
            Print(scroll.total_lines),
            SetForegroundColor(Color::Rgb {
                r: 100,
                g: 100,
                b: 100
            }),
            Print(" │ "),
            SetForegroundColor(Color::Rgb {
                r: 180,
                g: 180,
                b: 180
            }),
            Print("↑↓"),
            SetForegroundColor(Color::Rgb {
                r: 100,
                g: 100,
                b: 100
            }),
            Print("/"),
            SetForegroundColor(Color::Rgb {
                r: 180,
                g: 180,
                b: 180
            }),
            Print("PgUp PgDn"),
            SetForegroundColor(Color::Rgb {
                r: 100,
                g: 100,
                b: 100
            }),
            Print(" to scroll │ "),
            SetForegroundColor(Color::Rgb {
                r: 180,
                g: 180,
                b: 180
            }),
            Print("q"),
            SetForegroundColor(Color::Rgb {
                r: 100,
                g: 100,
                b: 100
            }),
            Print(" to quit")
        )?;

        Ok(())
    }

    /// Updates dimensions after terminal resize
    pub fn resize(&mut self, new_size: (u16, u16)) {
        self.width = new_size.0;
        self.height = new_size.1;
    }
}
