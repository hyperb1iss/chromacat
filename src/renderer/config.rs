//! Configuration types for the rendering system
//!
//! This module defines configuration options for animation and rendering
//! behavior, including frame rates, timing, and display options.

use super::error::RendererError;
use std::time::Duration;

/// Configuration for animation rendering
#[derive(Debug, Clone)]
pub struct AnimationConfig {
    /// Frames per second for animation playback (1-144)
    pub fps: u32,
    /// Duration of one complete pattern cycle
    pub cycle_duration: Duration,
    /// Whether to loop indefinitely
    pub infinite: bool,
    /// Whether to show animation progress bar
    pub show_progress: bool,
    /// Enable smooth transitions between frames
    pub smooth: bool,
}

impl AnimationConfig {
    /// Creates a new animation configuration
    ///
    /// # Arguments
    /// * `fps` - Frames per second (1-144)
    /// * `duration` - Animation duration (0 for infinite)
    pub fn new(fps: u32, duration: Duration) -> Self {
        Self {
            fps: fps.clamp(1, 144),
            cycle_duration: duration,
            infinite: duration.is_zero(),
            show_progress: true,
            smooth: false,
        }
    }

    /// Calculates the duration of a single frame based on FPS
    pub fn frame_duration(&self) -> Duration {
        Duration::from_nanos(1_000_000_000u64 / self.fps as u64)
    }

    /// Validates configuration values
    ///
    /// # Returns
    /// Ok(()) if valid, Error otherwise
    pub fn validate(&self) -> Result<(), RendererError> {
        if !(1..=144).contains(&self.fps) {
            return Err(RendererError::InvalidConfig(format!(
                "FPS must be between 1 and 144, got {}",
                self.fps
            )));
        }

        if !self.infinite && self.cycle_duration.is_zero() {
            return Err(RendererError::InvalidConfig(
                "Non-infinite animation must have non-zero duration".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            fps: 30,
            cycle_duration: Duration::from_secs(5),
            infinite: false,
            show_progress: true,
            smooth: false,
        }
    }
}
