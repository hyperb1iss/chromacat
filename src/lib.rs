//! ChromaCat is a versatile command-line tool for applying color gradients to text output.

pub mod app;
pub mod cli;
pub mod colorizer;
pub mod error;
pub mod gradient;
pub mod input;
pub mod pattern;
pub mod renderer;
pub mod themes;

pub use app::ChromaCat;
pub use error::{ChromaCatError, Result};
pub use themes::Theme;

// Re-export commonly used types for convenience
pub use pattern::{PatternConfig, PatternParams};
pub use renderer::AnimationConfig;
