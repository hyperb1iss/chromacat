//! Playlist support for ChromaCat
//!
//! This module provides functionality for defining and playing sequences of
//! patterns and themes. Playlists can be loaded from YAML files and provide
//! a way to create automated transitions between different visual effects.
//!
//! # Example Playlist File
//! ```yaml
//! entries:
//!   - name: "Digital Rain"
//!     pattern: "rain"
//!     theme: "matrix"
//!     duration: 20
//!     params:
//!       speed: 2.0
//!       density: 1.5
//! ```

use crate::error::Result;
use std::path::PathBuf;

mod entry;
mod player;

// Re-export the types from the submodules
pub use self::entry::{Playlist, PlaylistEntry};
pub use self::player::PlaylistPlayer;

/// Default directory for ChromaCat configuration
pub const CONFIG_DIR: &str = ".config/chromacat";

/// Default playlist filename
pub const DEFAULT_PLAYLIST: &str = "playlist.yaml";

/// Returns the path to the user's ChromaCat config directory
pub fn get_config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(CONFIG_DIR)
}

/// Returns the path to the default playlist file
pub fn get_default_playlist_path() -> PathBuf {
    get_config_dir().join(DEFAULT_PLAYLIST)
}

/// Loads the default playlist if it exists
pub fn load_default_playlist() -> Result<Option<Playlist>> {
    let path = get_default_playlist_path();
    if path.exists() {
        Ok(Some(Playlist::from_file(path)?))
    } else {
        Ok(None)
    }
}
