use std::{fmt, io};
use thiserror::Error;

/// Represents all possible errors that can occur in ChromaCat
#[derive(Error, Debug)]
pub enum ChromaCatError {
    /// Invalid gradient angle (must be between 0 and 360 degrees)
    #[error("Invalid gradient angle: {0}. Angle must be between 0 and 360 degrees")]
    InvalidAngle(i32),

    /// Input/output errors (file access, stdin/stdout)
    #[error("Input/output error: {0}")]
    InputError(String),

    /// Invalid theme selection
    #[error("Invalid theme: {0}. Using default theme 'rainbow'")]
    InvalidTheme(String),

    /// Gradient generation error
    #[error("Failed to generate gradient: {0}")]
    GradientError(String),

    /// Color rendering error
    #[error("Failed to render color: {0}")]
    RenderError(String),

    /// Terminal color support error
    #[error("Terminal color support error: {0}")]
    TerminalError(String),

    /// Invalid parameter value
    #[error("Invalid parameter '{name}': {value} (must be between {min} and {max})")]
    InvalidParameter {
        name: String,
        value: f64,
        min: f64,
        max: f64,
    },

    /// IO Error wrapper
    #[error("IO Error: {0}")]
    IoError(#[from] io::Error),

    /// Format Error wrapper
    #[error("Format Error: {0}")]
    FormatError(#[from] fmt::Error),

    #[error("Theme error: {0}")]
    ThemeError(String),
}

/// A Result type alias using ChromaCatError
pub type Result<T> = std::result::Result<T, ChromaCatError>;
