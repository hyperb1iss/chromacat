//! Playlist entry and playlist file format definitions.
//!
//! This module provides the core data structures for defining playlists and their entries.
//! Playlists are typically loaded from YAML files with the following format:
//!
//! ```yaml
//! entries:
//!   - name: "Cool Effect"
//!     pattern: "plasma"
//!     theme: "rainbow"
//!     duration: 30
//!     params:
//!       complexity: 3.0
//!       scale: 1.5
//! ```

use crate::error::{ChromaCatError, Result};
use crate::pattern::{PatternConfig, REGISTRY};
use crate::themes;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

/// A single entry in a playlist, describing a pattern configuration and duration.
///
/// Each entry specifies:
/// - An optional name for the sequence
/// - The pattern type to display
/// - The theme to apply
/// - How long to display it (in seconds)
/// - Optional pattern-specific parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistEntry {
    /// Optional name for this sequence
    #[serde(default)]
    pub name: String,

    /// Pattern type to use (must be a valid registered pattern)
    pub pattern: String,

    /// Theme to use (must be a valid theme name)
    pub theme: String,

    /// Duration in seconds to display this entry
    pub duration: u64,

    /// Pattern-specific parameters as key-value pairs
    #[serde(default)]
    pub params: Option<serde_yaml::Value>,
}

impl PlaylistEntry {
    /// Creates a new playlist entry with required values.
    ///
    /// # Arguments
    /// * `pattern` - The pattern type to use
    /// * `theme` - The theme to apply
    /// * `duration` - How long to display this entry in seconds
    pub fn new(pattern: impl Into<String>, theme: impl Into<String>, duration: u64) -> Self {
        Self {
            name: String::new(),
            pattern: pattern.into(),
            theme: theme.into(),
            duration,
            params: None,
        }
    }

    /// Validates that all references and parameters exist and are valid.
    ///
    /// Checks:
    /// - Pattern exists in the registry
    /// - Theme exists
    /// - Parameters are valid for the specified pattern
    ///
    /// # Returns
    /// * `Ok(())` if validation passes
    /// * `Err(ChromaCatError)` if any validation fails
    pub fn validate(&self) -> Result<()> {
        // Check pattern exists
        if !REGISTRY.list_patterns().contains(&self.pattern.as_str()) {
            return Err(ChromaCatError::InvalidPattern(format!(
                "Pattern '{}' does not exist",
                self.pattern
            )));
        }

        // Check theme exists
        themes::get_theme(&self.theme)?;

        // Validate parameters if present
        if let Some(params) = &self.params {
            let param_str = params_to_string(params)?;
            REGISTRY.validate_params(&self.pattern, &param_str)?;
        }

        Ok(())
    }

    /// Converts this entry into a pattern configuration that can be rendered.
    ///
    /// # Returns
    /// * `Ok(PatternConfig)` with the entry's settings applied
    /// * `Err(ChromaCatError)` if conversion fails
    pub fn to_pattern_config(&self) -> Result<PatternConfig> {
        // Start with default parameters for the pattern
        let mut pattern_config = PatternConfig {
            common: Default::default(),
            params: REGISTRY
                .create_pattern_params(&self.pattern)
                .ok_or_else(|| ChromaCatError::InvalidPattern(self.pattern.clone()))?,
        };

        // Set theme name
        pattern_config.common.theme_name = Some(self.theme.clone());

        // Apply custom parameters if present
        if let Some(params) = &self.params {
            let param_str = params_to_string(params)?;
            pattern_config.params = REGISTRY.parse_params(&self.pattern, &param_str)?;
        }

        Ok(pattern_config)
    }

    /// Gets this entry's duration as a std::time::Duration
    pub fn get_duration(&self) -> Duration {
        Duration::from_secs(self.duration)
    }
}

/// A complete playlist containing multiple entries to be played in sequence.
///
/// Playlists can be loaded from YAML files or strings and provide validation
/// of all entries before use.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Playlist {
    /// List of entries to play in sequence
    pub entries: Vec<PlaylistEntry>,
}

impl Playlist {
    /// Creates a new empty playlist
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Creates a playlist with the given entries
    pub fn with_entries(entries: Vec<PlaylistEntry>) -> Self {
        Self { entries }
    }

    /// Loads a playlist from a file.
    ///
    /// # Arguments
    /// * `path` - Path to the YAML playlist file
    ///
    /// # Returns
    /// * `Ok(Playlist)` if loading and validation succeed
    /// * `Err(ChromaCatError)` if loading or validation fails
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            ChromaCatError::InputError(format!("Failed to read playlist file: {}", e))
        })?;

        contents.parse()
    }
}

impl FromStr for Playlist {
    type Err = ChromaCatError;

    /// Parses a playlist from a YAML string.
    ///
    /// # Arguments
    /// * `contents` - YAML string containing playlist definition
    ///
    /// # Returns
    /// * `Ok(Playlist)` if parsing and validation succeed
    /// * `Err(ChromaCatError)` if parsing or validation fails
    fn from_str(contents: &str) -> std::result::Result<Self, Self::Err> {
        let playlist: Playlist = serde_yaml::from_str(contents)
            .map_err(|e| ChromaCatError::InputError(format!("Invalid playlist format: {}", e)))?;

        // Validate all entries
        for entry in &playlist.entries {
            entry.validate()?;
        }

        Ok(playlist)
    }
}

/// Converts YAML parameters to the string format expected by the registry.
///
/// # Arguments
/// * `params` - YAML value containing parameter key-value pairs
///
/// # Returns
/// * `Ok(String)` containing parameters in "key=value,key2=value2" format
/// * `Err(ChromaCatError)` if parameter format is invalid
fn params_to_string(params: &serde_yaml::Value) -> Result<String> {
    let mut param_strings = Vec::new();

    match params {
        serde_yaml::Value::Mapping(map) => {
            for (key, value) in map {
                let key_str = key.as_str().ok_or_else(|| {
                    ChromaCatError::InputError("Parameter key must be a string".to_string())
                })?;

                let value_str = match value {
                    serde_yaml::Value::Number(n) => n.to_string(),
                    serde_yaml::Value::Bool(b) => b.to_string(),
                    serde_yaml::Value::String(s) => s.clone(),
                    _ => {
                        return Err(ChromaCatError::InputError(format!(
                            "Invalid parameter value for '{}': must be number, boolean, or string",
                            key_str
                        )))
                    }
                };

                param_strings.push(format!("{}={}", key_str, value_str));
            }
        }
        _ => {
            return Err(ChromaCatError::InputError(
                "Parameters must be a mapping of key-value pairs".to_string(),
            ))
        }
    }

    Ok(param_strings.join(","))
}
