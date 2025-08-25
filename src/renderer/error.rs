//! Error types for the rendering system
//!
//! This module defines the error types and conversion implementations
//! for the rendering system.

use crate::error::ChromaCatError;
use std::io;
use thiserror::Error;

/// Errors that can occur during rendering operations
#[derive(Debug, Error)]
pub enum RendererError {
    /// I/O error during terminal operations
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// Error manipulating terminal state
    #[error("Terminal error: {0}")]
    TerminalError(String),

    /// Error managing render buffers
    #[error("Buffer error: {0}")]
    BufferError(String),

    /// Invalid configuration values
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Error during pattern generation
    #[error("Pattern error: {0}")]
    PatternError(String),

    /// Invalid pattern name or configuration
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),

    /// Invalid parameter string for current pattern
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    /// General error with message
    #[error("{0}")]
    Other(String),
}

impl From<ChromaCatError> for RendererError {
    fn from(err: ChromaCatError) -> Self {
        match err {
            ChromaCatError::IoError(e) => Self::IoError(e),
            ChromaCatError::InvalidParameter { name, value, .. } => {
                Self::InvalidConfig(format!("Invalid parameter {name}: {value}"))
            }
            ChromaCatError::PatternError { message, .. } => Self::PatternError(message),
            ChromaCatError::InvalidPattern(msg) => Self::InvalidPattern(msg),
            ChromaCatError::InvalidTheme(msg) => Self::Other(format!("Invalid theme: {msg}")),
            ChromaCatError::GradientError(msg) => Self::Other(format!("Gradient error: {msg}")),
            ChromaCatError::InputError(msg) => Self::Other(format!("Input error: {msg}")),
            ChromaCatError::ParseError(msg) => Self::Other(format!("Parse error: {msg}")),
            ChromaCatError::RenderError(msg) => Self::Other(format!("Render error: {msg}")),
            ChromaCatError::PlaylistError(msg) => Self::Other(format!("Playlist error: {msg}")),
            ChromaCatError::Other(msg) => Self::Other(msg),
            ChromaCatError::InvalidArt(msg) => Self::Other(format!("Invalid art type: {msg}")),
        }
    }
}

impl From<std::fmt::Error> for RendererError {
    fn from(err: std::fmt::Error) -> Self {
        Self::Other(format!("Format error: {err}"))
    }
}

impl From<String> for RendererError {
    fn from(msg: String) -> Self {
        Self::Other(msg)
    }
}

impl From<&str> for RendererError {
    fn from(msg: &str) -> Self {
        Self::Other(msg.to_string())
    }
}
