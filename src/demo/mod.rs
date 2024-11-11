//! Demo mode and art pattern generation
//!
//! This module provides ChromaCat's demo functionality, including:
//! - A variety of art patterns that showcase terminal capabilities
//! - Integration with playlists for sequenced demonstrations
//! - Configurable generation settings for different terminal sizes
//!
//! # Examples
//!
//! Basic demo mode:
//! ```bash
//! chromacat --demo
//! ```
//!
//! Using a specific art pattern:
//! ```bash
//! chromacat --demo --art matrix
//! ```
//!
//! With a playlist:
//! ```bash
//! chromacat --demo --playlist my-playlist.yaml
//! ```
//!
//! # Art Patterns
//!
//! Available patterns include:
//! - Matrix-style digital rain
//! - Wave interference patterns
//! - Spiral vortex effects
//! - ASCII art showcase
//! - Box drawing patterns
//! - Organic plasma effects
//! - Hypnotic vortex tunnels
//! - Cellular automaton patterns
//! - Fluid simulations
//! - Fractal trees
//! - Mandala designs
//! - And more...
//!
//! Each pattern can be customized through ChromaCat's theme system
//! and animation controls.

pub mod art;
pub mod generator;

pub use art::{ArtSettings, DemoArt};
pub use generator::DemoArtGenerator;

/// Terminal size requirements for demo art
pub const MIN_TERMINAL_WIDTH: u16 = 40;
pub const MIN_TERMINAL_HEIGHT: u16 = 10;

/// Checks if the terminal size is sufficient for demo art
pub fn check_terminal_size(width: u16, height: u16) -> Result<()> {
    if width < MIN_TERMINAL_WIDTH || height < MIN_TERMINAL_HEIGHT {
        return Err(Error::TerminalTooSmall {
            width: MIN_TERMINAL_WIDTH,
            height: MIN_TERMINAL_HEIGHT,
        });
    }
    Ok(())
}

/// Result type for demo operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for demo operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid art pattern specified
    #[error("Invalid art pattern: {0}")]
    InvalidPattern(String),

    /// Terminal size too small for art
    #[error("Terminal too small: minimum size is {width}x{height}")]
    TerminalTooSmall { width: u16, height: u16 },
}

/// Utility function to parse art type from string
pub fn parse_art(s: &str) -> Result<DemoArt> {
    DemoArt::try_from_str(s)
        .ok_or_else(|| Error::InvalidPattern(s.to_string()))
}
