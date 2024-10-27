//! Pattern generation and configuration for ChromaCat

pub mod config;
pub mod engine;
pub mod params;
pub mod patterns;
pub mod utils;

pub use config::{CommonParams, PatternConfig, PatternParams};
pub use engine::PatternEngine;
pub use params::{ParamType, PatternParam};
pub use patterns::{
    CheckerboardParams, DiagonalParams, DiamondParams, HorizontalParams,
    PerlinParams, PlasmaParams, RippleParams, SpiralParams, WaveParams,
};
