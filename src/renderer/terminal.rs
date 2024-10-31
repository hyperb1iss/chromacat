//! Terminal state management and interaction
//!
//! This module handles terminal setup, cleanup, and state management for ChromaCat.
//! It provides safe handling of terminal modes, cursor visibility, alternate screen,
//! and ensures proper cleanup even in error cases.
//!
//! Key responsibilities:
//! - Raw mode and alternate screen management
//! - Cursor visibility control
//! - Terminal size tracking and resizing
//! - Color support management
//! - Safe state cleanup on drop

use crossterm::{
    cursor::{Hide, Show},
    execute, queue,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    tty::IsTty,
};
use std::io::{stdout, StdoutLock, Write};

use super::error::RendererError;

/// Manages terminal state and operations.
/// Ensures proper terminal state management and cleanup.
#[derive(Debug)]
pub struct TerminalState {
    /// Current terminal dimensions (width, height)
    term_size: (u16, u16),
    /// Whether colors are enabled for output
    colors_enabled: bool,
    /// Whether alternate screen mode is active
    alternate_screen: bool,
    /// Whether raw mode is enabled
    raw_mode: bool,
    /// Whether cursor is currently hidden
    cursor_hidden: bool,
    /// Whether stdout is a TTY
    is_tty: bool,
}

impl TerminalState {
    /// Creates a new terminal state manager with default settings.
    ///
    /// # Returns
    /// A new TerminalState instance with detected terminal capabilities.
    ///
    /// # Errors
    /// Returns error if terminal size cannot be detected.
    pub fn new() -> Result<Self, RendererError> {
        // Detect initial terminal size
        let term_size = crossterm::terminal::size().map_err(|e| {
            RendererError::TerminalError(format!("Failed to get terminal size: {}", e))
        })?;

        // Check if stdout is a TTY
        let is_tty = stdout().is_tty();

        // Enable colors by default for TTY
        let colors_enabled = is_tty;

        Ok(Self {
            term_size,
            colors_enabled,
            alternate_screen: false,
            raw_mode: false,
            cursor_hidden: false,
            is_tty,
        })
    }

    /// Sets up the terminal for rendering operations.
    ///
    /// # Effects
    /// - Enables raw mode for better input handling
    /// - Hides cursor for cleaner display
    ///
    /// # Errors
    /// Returns error if terminal modes cannot be set.
    pub fn setup(&mut self) -> Result<(), RendererError> {
        if !self.is_tty {
            return Ok(());
        }

        // Enable raw mode if needed
        if !self.raw_mode {
            enable_raw_mode().map_err(|e| {
                RendererError::TerminalError(format!("Failed to enable raw mode: {}", e))
            })?;
            self.raw_mode = true;
        }

        // Hide cursor if needed
        if !self.cursor_hidden {
            execute!(stdout(), Hide)?;
            self.cursor_hidden = true;
        }

        Ok(())
    }

    /// Restores terminal to its original state.
    ///
    /// # Effects
    /// - Disables raw mode
    /// - Shows cursor
    /// - Leaves alternate screen if active
    ///
    /// # Errors
    /// Returns error if terminal state cannot be restored.
    pub fn cleanup(&mut self) -> Result<(), RendererError> {
        if !self.is_tty {
            return Ok(());
        }

        let mut stdout = stdout();

        // Show cursor if hidden
        if self.cursor_hidden {
            execute!(stdout, Show)?;
            self.cursor_hidden = false;
        }

        // Disable raw mode
        if self.raw_mode {
            disable_raw_mode().map_err(|e| {
                RendererError::TerminalError(format!("Failed to disable raw mode: {}", e))
            })?;
            self.raw_mode = false;
        }

        // Leave alternate screen if active
        if self.alternate_screen {
            execute!(stdout, LeaveAlternateScreen)?;
            self.alternate_screen = false;
        }

        stdout.flush()?;
        Ok(())
    }

    /// Enters alternate screen mode and sets up for rendering.
    ///
    /// # Effects
    /// - Switches to alternate screen
    /// - Sets up terminal modes
    ///
    /// # Errors
    /// Returns error if alternate screen cannot be entered.
    pub fn enter_alternate_screen(&mut self) -> Result<(), RendererError> {
        if !self.is_tty {
            return Ok(());
        }

        if !self.alternate_screen {
            execute!(stdout(), EnterAlternateScreen)?;
            self.alternate_screen = true;
        }

        self.setup()?;
        Ok(())
    }

    /// Clears the entire screen.
    pub fn clear_screen(&mut self) -> Result<(), RendererError> {
        if self.is_tty {
            queue!(stdout(), Clear(ClearType::All))?;
        }
        Ok(())
    }

    /// Ensures all queued output is written to the terminal.
    pub fn flush(&self) -> Result<(), RendererError> {
        stdout().flush()?;
        Ok(())
    }

    /// Gets a locked handle to stdout for efficient writing.
    pub fn stdout(&self) -> StdoutLock {
        stdout().lock()
    }

    /// Gets current terminal dimensions.
    #[inline]
    pub fn size(&self) -> (u16, u16) {
        self.term_size
    }

    /// Updates stored terminal size and handles resize.
    ///
    /// # Effects
    /// - Updates stored dimensions
    /// - Clears screen to handle resize cleanly
    ///
    /// # Errors
    /// Returns error if terminal cannot be resized.
    pub fn resize(&mut self, width: u16, height: u16) -> Result<(), RendererError> {
        self.term_size = (width, height);
        if self.is_tty {
            let mut stdout = stdout();
            queue!(stdout, Clear(ClearType::All))?;
            stdout.flush()?;
        }
        Ok(())
    }

    /// Returns whether color output is enabled.
    #[inline]
    pub fn colors_enabled(&self) -> bool {
        self.colors_enabled
    }

    /// Sets whether color output should be enabled.
    pub fn set_colors_enabled(&mut self, enabled: bool) {
        self.colors_enabled = enabled;
    }

    /// Returns whether stdout is a TTY.
    #[inline]
    pub fn is_tty(&self) -> bool {
        self.is_tty
    }

    /// Shows the cursor if currently hidden.
    pub fn show_cursor(&mut self) -> Result<(), RendererError> {
        if self.is_tty && self.cursor_hidden {
            execute!(stdout(), Show)?;
            self.cursor_hidden = false;
        }
        Ok(())
    }

    /// Hides the cursor if currently visible.
    pub fn hide_cursor(&mut self) -> Result<(), RendererError> {
        if self.is_tty && !self.cursor_hidden {
            execute!(stdout(), Hide)?;
            self.cursor_hidden = true;
        }
        Ok(())
    }

    /// Add recovery method
    pub fn try_recover(&mut self) -> Result<(), RendererError> {
        if self.is_tty {
            // Try to restore known good state
            let _ = execute!(stdout(), Show);
            let _ = disable_raw_mode();
            let _ = execute!(stdout(), LeaveAlternateScreen);

            // Reinitialize
            self.setup()?;
        }
        Ok(())
    }
}

impl Drop for TerminalState {
    fn drop(&mut self) {
        // Always attempt cleanup on drop
        if let Err(e) = self.cleanup() {
            eprintln!("Error cleaning up terminal state: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_creation() {
        let term_state = TerminalState::new();
        assert!(term_state.is_ok());
    }

    #[test]
    fn test_terminal_size() {
        let term_state = TerminalState::new().unwrap();
        let (width, height) = term_state.size();
        assert!(width > 0);
        assert!(height > 0);
    }

    #[test]
    fn test_color_control() {
        let mut term_state = TerminalState::new().unwrap();
        let initial = term_state.colors_enabled();
        term_state.set_colors_enabled(!initial);
        assert_eq!(term_state.colors_enabled(), !initial);
    }
}
