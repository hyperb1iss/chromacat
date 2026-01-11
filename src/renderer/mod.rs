/// Clean renderer architecture - all ratatui, no legacy code
/// This is the new, simplified renderer module
pub mod art_mixer;
pub mod automix;
pub mod blend_engine;
pub mod buffer;
pub mod config;
pub mod core;
pub mod error;
pub mod event_loop;
pub mod input;
pub mod pattern_widget;
pub mod playground;
pub mod scheduler;
pub mod scroll;
pub mod status_bar;
pub mod terminal;

// Re-export the main types
pub use config::AnimationConfig;
pub use core::Renderer;
pub use error::RendererError;
pub use status_bar::StatusBar;
pub use terminal::{ColorCapability, TerminalState};

// These old modules will be removed once migration is complete:
// - buffer.rs (old terminal rendering)
// - terminal.rs (crossterm-based)
// - status_bar.rs (crossterm-based)
// - scroll.rs (not needed with ratatui)
// - control.rs (old input handling)
// - scheduler.rs (can be simplified)
// - modulation.rs (can be simplified)
