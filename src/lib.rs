//! ChromaCat is a versatile command-line tool for applying color gradients to text output.

// First declare the macro module
#[macro_use]
pub mod pattern;

pub mod app;
pub mod cli;
pub mod cli_format;
pub mod error;
pub mod gradient;
pub mod input;
pub mod renderer;
pub mod streaming;
pub mod themes;

pub use app::ChromaCat;
pub use error::{ChromaCatError, Result};

// Re-export commonly used types for convenience
pub use pattern::{PatternConfig, PatternParams};
pub use renderer::{AnimationConfig, Renderer};
pub use streaming::StreamingInput;
