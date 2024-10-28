//! Terminal interaction and state management
//!
//! This module handles terminal setup, cleanup, and state management,
//! including raw mode, alternate screen, and cursor visibility.

use crossterm::{
    cursor::{Hide, Show},
    execute, queue,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::io::{stdout, StdoutLock, Write};

use super::error::RendererError;

/// Manages terminal state and operations
pub struct TerminalState {
    term_size: (u16, u16),
    colors_enabled: bool,
    alternate_screen: bool,
    raw_mode: bool,
}

impl TerminalState {
    /// Creates a new terminal state manager
    pub fn new() -> Result<Self, RendererError> {
        let term_size = crossterm::terminal::size().map_err(|e| {
            RendererError::TerminalError(format!("Failed to get terminal size: {}", e))
        })?;

        Ok(Self {
            term_size,
            colors_enabled: true, // Default to enabled
            alternate_screen: false,
            raw_mode: false,
        })
    }

    /// Sets up the terminal for rendering
    pub fn setup(&mut self) -> Result<(), RendererError> {
        enable_raw_mode().map_err(|e| {
            RendererError::TerminalError(format!("Failed to enable raw mode: {}", e))
        })?;
        self.raw_mode = true;

        execute!(stdout(), Hide)?;
        Ok(())
    }

    /// Cleans up terminal state
    pub fn cleanup(&mut self) -> Result<(), RendererError> {
        let mut stdout = stdout();

        if self.raw_mode {
            disable_raw_mode().map_err(|e| {
                RendererError::TerminalError(format!("Failed to disable raw mode: {}", e))
            })?;
            self.raw_mode = false;
        }

        if self.alternate_screen {
            execute!(stdout, LeaveAlternateScreen)?;
            self.alternate_screen = false;
        }

        execute!(stdout, Show)?;
        stdout.flush()?;
        Ok(())
    }

    /// Enters alternate screen mode
    pub fn enter_alternate_screen(&mut self) -> Result<(), RendererError> {
        execute!(stdout(), EnterAlternateScreen, Hide)?;
        self.alternate_screen = true;
        Ok(())
    }

    /// Clears the screen
    pub fn clear_screen(&mut self) -> Result<(), RendererError> {
        queue!(stdout(), Clear(ClearType::All))?;
        Ok(())
    }

    /// Flushes stdout
    pub fn flush(&self) -> Result<(), RendererError> {
        stdout().flush()?;
        Ok(())
    }

    /// Gets stdout lock
    pub fn stdout(&self) -> StdoutLock {
        stdout().lock()
    }

    /// Gets terminal size
    pub fn size(&self) -> (u16, u16) {
        self.term_size
    }

    /// Updates terminal size
    pub fn resize(&mut self, width: u16, height: u16) -> Result<(), RendererError> {
        self.term_size = (width, height);
        Ok(())
    }

    /// Checks if colors are enabled
    pub fn colors_enabled(&self) -> bool {
        self.colors_enabled
    }
}

impl Drop for TerminalState {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
