//! Error types for the rendering system
//!
//! This module defines the error types and conversion implementations
//! for the rendering system.

use crate::error::ChromaCatError;
use std::fmt;
use std::io;

/// Errors that can occur during rendering operations
#[derive(Debug)]
pub enum RendererError {
    /// I/O error during terminal operations
    IoError(io::Error),
    /// Error manipulating terminal state
    TerminalError(String),
    /// Error managing render buffers
    BufferError(String),
    /// Invalid configuration values
    InvalidConfig(String),
    /// Error during pattern generation
    PatternError(String),
    /// General error with message
    Other(String),
}

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(err) => write!(f, "I/O error: {}", err),
            Self::TerminalError(msg) => write!(f, "Terminal error: {}", msg),
            Self::BufferError(msg) => write!(f, "Buffer error: {}", msg),
            Self::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            Self::PatternError(msg) => write!(f, "Pattern error: {}", msg),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for RendererError {}

impl From<io::Error> for RendererError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<std::fmt::Error> for RendererError {
    fn from(err: std::fmt::Error) -> Self {
        Self::Other(format!("Format error: {}", err))
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

impl From<ChromaCatError> for RendererError {
    fn from(err: ChromaCatError) -> Self {
        match err {
            ChromaCatError::IoError(e) => Self::IoError(e),
            ChromaCatError::InvalidParameter { name, value, .. } => {
                Self::InvalidConfig(format!("Invalid parameter {}: {}", name, value))
            }
            ChromaCatError::PatternError { message, .. } => Self::PatternError(message),
            _ => Self::Other(err.to_string()),
        }
    }
}
