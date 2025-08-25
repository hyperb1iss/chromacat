use crate::renderer::RendererError;
use std::fmt;
use std::io;

/// Custom error types for ChromaCat
#[derive(Debug)]
pub enum ChromaCatError {
    /// I/O operation failed
    IoError(io::Error),
    /// Invalid parameter value
    InvalidParameter {
        name: String,
        value: f64,
        min: f64,
        max: f64,
    },
    /// Invalid theme name or configuration
    InvalidTheme(String),
    /// Invalid gradient configuration
    GradientError(String),
    /// Pattern parameter validation error
    PatternError {
        pattern: String,
        param: String,
        message: String,
    },
    /// Input file error
    InputError(String),
    /// Parameter parsing error
    ParseError(String),
    /// Rendering error
    RenderError(String),
    /// Invalid pattern name or configuration
    InvalidPattern(String),
    /// Playlist-related error
    PlaylistError(String),
    /// General error with message
    Other(String),
    /// Invalid art type specified
    InvalidArt(String),
}

impl std::error::Error for ChromaCatError {}

impl fmt::Display for ChromaCatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(err) => write!(f, "I/O error: {err}"),
            Self::InvalidParameter {
                name,
                value,
                min,
                max,
            } => {
                write!(
                    f,
                    "Invalid {name} value {value}: must be between {min} and {max}"
                )
            }
            Self::InvalidTheme(msg) => write!(f, "Invalid theme: {msg}"),
            Self::GradientError(msg) => write!(f, "Gradient error: {msg}"),
            Self::PatternError {
                pattern,
                param,
                message,
            } => {
                write!(
                    f,
                    "Pattern '{pattern}' parameter '{param}' error: {message}"
                )
            }
            Self::InputError(msg) => write!(f, "Input error: {msg}"),
            Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
            Self::RenderError(msg) => write!(f, "Render error: {msg}"),
            Self::InvalidPattern(msg) => write!(f, "Invalid pattern: {msg}"),
            Self::PlaylistError(msg) => write!(f, "Playlist error: {msg}"),
            Self::Other(msg) => write!(f, "{msg}"),
            Self::InvalidArt(msg) => write!(f, "Invalid art type: {msg}"),
        }
    }
}

impl From<io::Error> for ChromaCatError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<std::fmt::Error> for ChromaCatError {
    fn from(err: std::fmt::Error) -> Self {
        Self::Other(err.to_string())
    }
}

impl From<String> for ChromaCatError {
    fn from(msg: String) -> Self {
        Self::ParseError(msg)
    }
}

// Add conversion from parameter validation errors
impl From<(String, String, String)> for ChromaCatError {
    fn from((pattern, param, message): (String, String, String)) -> Self {
        Self::PatternError {
            pattern,
            param,
            message,
        }
    }
}

// Add conversion from RendererError
impl From<RendererError> for ChromaCatError {
    fn from(err: RendererError) -> Self {
        match err {
            RendererError::IoError(e) => Self::IoError(e),
            RendererError::TerminalError(msg) => Self::RenderError(msg),
            RendererError::BufferError(msg) => Self::RenderError(msg),
            RendererError::InvalidConfig(msg) => Self::RenderError(msg),
            RendererError::PatternError(msg) => Self::PatternError {
                pattern: "render".to_string(),
                param: "pattern".to_string(),
                message: msg,
            },
            RendererError::InvalidPattern(msg) => Self::InvalidPattern(msg),
            RendererError::InvalidParams(msg) => Self::ParseError(msg),
            RendererError::Other(msg) => Self::RenderError(msg),
        }
    }
}

/// Result type alias using ChromaCatError
pub type Result<T> = std::result::Result<T, ChromaCatError>;
