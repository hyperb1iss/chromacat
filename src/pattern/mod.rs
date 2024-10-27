//! Pattern generation and configuration for ChromaCat
//!
//! This module provides the core pattern generation functionality for creating
//! visual effects in text output. It includes:
//!
//! - Pattern type definitions and parameters
//! - Pattern calculation algorithms
//! - Animation timing and updates
//! - Color gradient mapping
//! - Performance optimizations through lookup tables
//!
//! The pattern system supports multiple effect types including waves, spirals,
//! plasma effects, and more, each with configurable parameters for customization.

mod config;
mod engine;
mod patterns;
mod utils;

pub use config::{CommonParams, PatternConfig, PatternParams};
pub use engine::PatternEngine;
