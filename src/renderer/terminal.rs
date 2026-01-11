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
//! - Terminal capability detection
//! - Safe state cleanup on drop

use crossterm::{
    cursor::{Hide, Show},
    execute, queue,
    terminal::{
        disable_raw_mode, enable_raw_mode, size as term_size, Clear, ClearType,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
    tty::IsTty,
};
use std::io::{stdout, StdoutLock, Write};

use super::error::RendererError;

/// Terminal color capability levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColorCapability {
    /// No color support (dumb terminals, NO_COLOR set)
    None,
    /// Basic 8/16 colors (ANSI)
    Basic,
    /// 256 color support
    Extended,
    /// True color / 24-bit color support
    TrueColor,
}

impl ColorCapability {
    /// Detect color capability from environment variables
    pub fn detect() -> Self {
        // Check NO_COLOR first - universal disable
        if std::env::var("NO_COLOR").is_ok() {
            return Self::None;
        }

        // Check COLORTERM for true color
        if let Ok(colorterm) = std::env::var("COLORTERM") {
            let colorterm_lower = colorterm.to_lowercase();
            if colorterm_lower.contains("truecolor") || colorterm_lower.contains("24bit") {
                return Self::TrueColor;
            }
        }

        // Check TERM for various capabilities
        if let Ok(term) = std::env::var("TERM") {
            let term_lower = term.to_lowercase();

            // Dumb terminal
            if term_lower == "dumb" {
                return Self::None;
            }

            // True color terminals
            if term_lower.contains("truecolor")
                || term_lower.contains("24bit")
                || term_lower.contains("direct")
            {
                return Self::TrueColor;
            }

            // 256 color terminals
            if term_lower.contains("256color")
                || term_lower.contains("256")
                || term_lower.starts_with("xterm")
                || term_lower.starts_with("screen")
                || term_lower.starts_with("tmux")
                || term_lower.starts_with("vt")
                || term_lower.starts_with("rxvt")
                || term_lower.starts_with("linux")
            {
                // Modern terminals usually support true color even without advertising
                // Check for common true color terminals
                if term_lower.contains("kitty")
                    || term_lower.contains("alacritty")
                    || term_lower.contains("iterm")
                    || term_lower.contains("wezterm")
                    || term_lower.contains("ghostty")
                {
                    return Self::TrueColor;
                }
                return Self::Extended;
            }

            // Basic ANSI support
            if term_lower.contains("ansi")
                || term_lower.contains("color")
                || term_lower.starts_with("vt100")
            {
                return Self::Basic;
            }
        }

        // Check TERM_PROGRAM for additional detection
        if let Ok(program) = std::env::var("TERM_PROGRAM") {
            let program_lower = program.to_lowercase();
            if program_lower.contains("iterm")
                || program_lower.contains("hyper")
                || program_lower.contains("vscode")
                || program_lower.contains("ghostty")
                || program_lower.contains("wezterm")
                || program_lower.contains("alacritty")
                || program_lower.contains("kitty")
            {
                return Self::TrueColor;
            }
        }

        // Check WT_SESSION for Windows Terminal (supports true color)
        if std::env::var("WT_SESSION").is_ok() {
            return Self::TrueColor;
        }

        // Default to extended colors if we're in a TTY
        // Most modern terminals support at least 256 colors
        if atty::is(atty::Stream::Stdout) {
            Self::Extended
        } else {
            Self::None
        }
    }

    /// Returns true if this capability supports at least 256 colors
    pub fn supports_256(&self) -> bool {
        matches!(self, Self::Extended | Self::TrueColor)
    }

    /// Returns true if this capability supports true color (24-bit)
    pub fn supports_truecolor(&self) -> bool {
        matches!(self, Self::TrueColor)
    }

    /// Returns true if any color is supported
    pub fn supports_color(&self) -> bool {
        !matches!(self, Self::None)
    }
}

impl Default for ColorCapability {
    fn default() -> Self {
        Self::detect()
    }
}

/// Manages terminal state and operations.
/// Ensures proper terminal state management and cleanup.
#[derive(Debug)]
pub struct TerminalState {
    /// Current terminal dimensions (width, height)
    term_size: (u16, u16),
    /// Whether colors are enabled for output
    colors_enabled: bool,
    /// Detected color capability level
    color_capability: ColorCapability,
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
        // Get terminal size
        let term_size = if Self::is_test_env() {
            (80, 24) // Default size for tests
        } else {
            term_size().map_err(|e| {
                RendererError::TerminalError(format!("Failed to get terminal size: {}", e))
            })?
        };

        // Detect color capabilities
        let color_capability = ColorCapability::detect();

        // Check if stdout is a TTY
        let is_tty = !Self::is_test_env() && stdout().is_tty();

        // Enable colors based on capability detection
        let colors_enabled = is_tty && color_capability.supports_color();

        Ok(Self {
            term_size,
            colors_enabled,
            color_capability,
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

    /// Returns the detected color capability level.
    #[inline]
    pub fn color_capability(&self) -> ColorCapability {
        self.color_capability
    }

    /// Returns true if true color (24-bit) is supported.
    #[inline]
    pub fn supports_truecolor(&self) -> bool {
        self.color_capability.supports_truecolor()
    }

    /// Returns true if 256 colors are supported.
    #[inline]
    pub fn supports_256_colors(&self) -> bool {
        self.color_capability.supports_256()
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

    /// Returns true if running in a test environment
    #[inline]
    fn is_test_env() -> bool {
        std::env::var("RUST_TEST").is_ok()
            || std::env::var("CARGO_TARGET_DIR").is_ok()
            || std::env::var("CI").is_ok()
            || std::env::var("TERM").map(|v| v == "dumb").unwrap_or(false)
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
