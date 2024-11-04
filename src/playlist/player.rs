//! Playlist playback and state management.
//!
//! This module provides functionality for controlling playlist playback, including:
//! - Automatic transitions between entries based on duration
//! - Manual navigation (next/previous)
//! - Pause/resume control
//! - Progress tracking

use super::entry::{Playlist, PlaylistEntry};
use crate::error::{ChromaCatError, Result};
use crate::pattern::PatternConfig;
use std::time::Duration;

/// Controls playback of a playlist, managing transitions between entries.
///
/// The player keeps track of:
/// - Current entry and its elapsed time
/// - Pause state
/// - Automatic transitions based on entry durations
#[derive(Debug)]
pub struct PlaylistPlayer {
    /// The playlist being played
    playlist: Playlist,
    /// Current entry index in the playlist
    current_index: usize,
    /// Time spent playing the current entry
    time_in_current: Duration,
    /// Whether playback is currently paused
    paused: bool,
}

impl PlaylistPlayer {
    /// Creates a new playlist player starting at the first entry.
    ///
    /// # Arguments
    /// * `playlist` - The playlist to play
    pub fn new(playlist: Playlist) -> Self {
        Self {
            playlist,
            current_index: 0,
            time_in_current: Duration::ZERO,
            paused: false,
        }
    }

    /// Gets the current pattern configuration for rendering.
    ///
    /// # Returns
    /// * `Ok(PatternConfig)` - Configuration for the current entry
    /// * `Err(ChromaCatError)` - If there is no current entry
    pub fn current_config(&self) -> Result<PatternConfig> {
        self.current_entry()
            .ok_or_else(|| ChromaCatError::Other("No current entry".to_string()))?
            .to_pattern_config()
    }

    /// Gets a reference to the current playlist entry.
    ///
    /// # Returns
    /// * `Some(&PlaylistEntry)` - Reference to current entry
    /// * `None` - If playlist is empty
    pub fn current_entry(&self) -> Option<&PlaylistEntry> {
        self.playlist.entries.get(self.current_index)
    }

    /// Updates player state based on elapsed time.
    ///
    /// This method handles automatic transitions between entries when their
    /// duration has elapsed. It does nothing if playback is paused.
    ///
    /// # Arguments
    /// * `delta` - Time elapsed since last update
    ///
    /// # Returns
    /// * `true` if the current entry changed
    /// * `false` if staying on same entry or playlist is empty/paused
    pub fn update(&mut self, delta: Duration) -> bool {
        if self.paused {
            return false;
        }

        let current_duration = match self.current_entry() {
            Some(current) => current.get_duration(),
            None => return false,
        };

        self.time_in_current += delta;

        if self.time_in_current >= current_duration {
            // Move to next entry
            self.current_index = (self.current_index + 1) % self.playlist.entries.len();
            self.time_in_current = Duration::ZERO;
            true
        } else {
            false
        }
    }

    /// Gets progress through current entry as a fraction.
    ///
    /// # Returns
    /// * A value between 0.0 and 1.0 representing progress
    /// * Returns 0.0 if playlist is empty
    pub fn current_progress(&self) -> f64 {
        let Some(current) = self.current_entry() else {
            return 0.0;
        };

        self.time_in_current.as_secs_f64() / current.duration as f64
    }

    /// Toggles between paused and playing states.
    ///
    /// When paused:
    /// - Time tracking stops
    /// - Automatic transitions are disabled
    /// - Manual navigation still works
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Returns whether playback is currently paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Manually advances to the next entry.
    ///
    /// If at the end of the playlist, wraps around to the first entry.
    /// Does nothing if playlist is empty.
    pub fn next_entry(&mut self) {
        if !self.playlist.entries.is_empty() {
            self.current_index = (self.current_index + 1) % self.playlist.entries.len();
            self.time_in_current = Duration::ZERO;
        }
    }

    /// Manually goes to the previous entry.
    ///
    /// If at the start of the playlist, wraps around to the last entry.
    /// Does nothing if playlist is empty.
    pub fn previous_entry(&mut self) {
        if !self.playlist.entries.is_empty() {
            self.current_index = if self.current_index == 0 {
                self.playlist.entries.len() - 1
            } else {
                self.current_index - 1
            };
            self.time_in_current = Duration::ZERO;
        }
    }

    /// Gets the total number of entries in the playlist.
    pub fn entry_count(&self) -> usize {
        self.playlist.entries.len()
    }

    /// Gets the index of the current entry.
    ///
    /// Note: This index is always valid unless the playlist is empty.
    pub fn current_index(&self) -> usize {
        self.current_index
    }
}
