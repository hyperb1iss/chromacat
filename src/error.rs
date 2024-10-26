use thiserror::Error;

/// Represents all possible errors that can occur in ChromaCat
#[derive(Error, Debug)]
pub enum ChromaCatError {
    /// Invalid gradient angle (must be between 0 and 360 degrees)
    #[error("Invalid gradient angle: {0}. Angle must be between 0 and 360 degrees")]
    InvalidAngle(i32),

    /// Input/output errors (file access, stdin/stdout)
    #[error("Input/output error: {0}")]
    InputError(#[from] std::io::Error),

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
}

/// A Result type alias using ChromaCatError
pub type Result<T> = std::result::Result<T, ChromaCatError>;