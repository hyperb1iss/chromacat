//! ChromaCat is a versatile command-line tool for applying color gradients to text output.

pub mod app;
pub mod cli;
pub mod colorizer;
pub mod error;
pub mod gradient;
pub mod input;

pub use app::ChromaCat;