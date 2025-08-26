/// Clean renderer architecture - all ratatui, no legacy code
/// This is the new, simplified renderer module
pub mod automix;
pub mod config;
pub mod core;
pub mod error;
pub mod event_loop;
pub mod input;
pub mod pattern_widget;
pub mod playground;
pub mod scheduler;

// Re-export the main types
pub use config::AnimationConfig;
pub use core::Renderer;
pub use error::RendererError;

// These old modules will be removed once migration is complete:
// - buffer.rs (old terminal rendering)
// - terminal.rs (crossterm-based)
// - status_bar.rs (crossterm-based)
// - scroll.rs (not needed with ratatui)
// - control.rs (old input handling)
// - scheduler.rs (can be simplified)
// - modulation.rs (can be simplified)
