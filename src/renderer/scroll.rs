//! Scrolling and viewport management
//!
//! This module handles scroll state, viewport calculations, and scroll-related
//! user input for the rendering system.

use crossterm::event::{KeyCode, KeyEvent};

/// Action to take after handling a scroll event
#[derive(Debug, PartialEq)]
pub enum Action {
    /// Continue running
    Continue,
    /// Exit the application
    Exit,
    /// No change needed
    NoChange,
}

/// Manages scrolling state and viewport calculations
#[derive(Debug)]
pub struct ScrollState {
    /// Index of the first visible line
    pub top_line: usize,
    /// Number of lines that fit in the viewport
    pub viewport_height: u16,
    /// Total number of lines in the content
    pub total_lines: usize,
}

impl ScrollState {
    /// Creates a new scroll state
    pub fn new(viewport_height: u16) -> Self {
        Self {
            top_line: 0,
            viewport_height,
            total_lines: 0,
        }
    }

    /// Updates the total number of lines
    pub fn set_total_lines(&mut self, total: usize) {
        self.total_lines = total;
        self.clamp_scroll();
    }

    /// Updates the viewport height and clamps scroll position
    pub fn update_viewport(&mut self, height: u16) {
        self.viewport_height = height;
        self.clamp_scroll();
    }

    /// Returns the visible range of lines, clamped to valid bounds
    pub fn get_visible_range(&self) -> (usize, usize) {
        let start = self.top_line;
        let max_visible = self.viewport_height as usize;
        let end = (start + max_visible).min(self.total_lines);
        (start, end)
    }

    /// Handles keyboard input for scrolling
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::PageUp => {
                self.scroll_up(self.viewport_height as i32 - 1);
                Action::Continue
            }
            KeyCode::PageDown => {
                self.scroll_down(self.viewport_height as i32 - 1);
                Action::Continue
            }
            KeyCode::Up => {
                self.scroll_up(1);
                Action::Continue
            }
            KeyCode::Down => {
                self.scroll_down(1);
                Action::Continue
            }
            KeyCode::Char('q') | KeyCode::Esc => Action::Exit,
            _ => Action::NoChange,
        }
    }

    /// Scrolls up by the specified amount with bounds checking
    pub fn scroll_up(&mut self, amount: i32) {
        if amount <= 0 {
            return;
        }
        self.top_line = self.top_line.saturating_sub(amount as usize);
    }

    /// Scrolls down by the specified amount with bounds checking
    pub fn scroll_down(&mut self, amount: i32) {
        if amount <= 0 {
            return;
        }
        let max_scroll = self.max_scroll();
        self.top_line = (self.top_line + amount as usize).min(max_scroll);
    }

    /// Returns the total number of lines
    pub fn total_lines(&self) -> usize {
        self.total_lines
    }

    // Private helper methods

    fn max_scroll(&self) -> usize {
        self.total_lines
            .saturating_sub(self.viewport_height as usize)
    }

    fn clamp_scroll(&mut self) {
        let max_scroll = self.max_scroll();
        self.top_line = self.top_line.min(max_scroll);
    }

    pub fn validate_viewport(&mut self) {
        // Ensure viewport height is reasonable
        self.viewport_height = self.viewport_height.max(1);

        // Ensure top line is in valid range
        let max_scroll = self.max_scroll();
        self.top_line = self.top_line.min(max_scroll);
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new(24) // Default terminal height
    }
}
