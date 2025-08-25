//! Playlist entry definitions and validation
//!
//! This module defines the structure for playlist entries, which specify patterns,
//! themes, demo art, and other configuration for ChromaCat's animation sequences.
//! Each entry represents a single step in the playlist that can be rendered with
//! specific visual effects and timing.

use crate::demo::DemoArt;
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
/// - A display name (optional)
/// - The pattern type and theme to use
/// - How long to display it
/// - Pattern-specific parameters (optional)
/// - Demo art to display (optional)
///
/// # Example
/// ```yaml
/// name: "Digital Rain"
/// pattern: "rain"
/// theme: "matrix"
/// duration: 30
/// art: "matrix"
/// params:
///   speed: 2.0
///   density: 1.5
/// ```
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

    /// Demo art to display (only used in demo mode)
    #[serde(default)]
    pub art: Option<DemoArt>,
}

impl PlaylistEntry {
    /// Creates a new entry with the required fields.
    ///
    /// # Arguments
    /// * `pattern` - Pattern type to use
    /// * `theme` - Theme to apply
    /// * `duration` - How long to display in seconds
    ///
    /// # Example
    /// ```
    /// use chromacat::playlist::PlaylistEntry;
    /// use chromacat::demo::DemoArt;
    ///
    /// let entry = PlaylistEntry::new("wave", "ocean", 30)
    ///     .with_name("Ocean Waves")
    ///     .with_art(DemoArt::Waves);
    /// ```
    pub fn new(pattern: impl Into<String>, theme: impl Into<String>, duration: u64) -> Self {
        Self {
            name: String::new(),
            pattern: pattern.into(),
            theme: theme.into(),
            duration,
            params: None,
            art: None,
        }
    }

    /// Adds a display name to the entry.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Adds pattern-specific parameters to the entry.
    pub fn with_params(mut self, params: serde_yaml::Value) -> Self {
        self.params = Some(params);
        self
    }

    /// Sets the demo art pattern to display.
    pub fn with_art(mut self, art: DemoArt) -> Self {
        self.art = Some(art);
        self
    }

    /// Returns a human-readable description of this entry.
    pub fn description(&self) -> String {
        let mut desc = if self.name.is_empty() {
            format!("{} with {} theme", self.pattern, self.theme)
        } else {
            self.name.clone()
        };

        if let Some(art) = &self.art {
            desc.push_str(&format!(" ({})", art.display_name()));
        }

        desc
    }

    /// Validates that all references and parameters exist and are valid.
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

        // Validate art type if present
        if let Some(art) = &self.art {
            // Ensure the art type is valid by checking against available types
            if !DemoArt::all_types().contains(art) && *art != DemoArt::All {
                return Err(ChromaCatError::InvalidArt(format!(
                    "Art type '{}' is not valid",
                    art.as_str()
                )));
            }
        }

        Ok(())
    }

    /// Converts this entry into a pattern configuration that can be rendered.
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
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            ChromaCatError::InputError(format!("Failed to read playlist file: {e}"))
        })?;

        contents.parse()
    }
}

impl FromStr for Playlist {
    type Err = ChromaCatError;

    fn from_str(contents: &str) -> std::result::Result<Self, Self::Err> {
        let playlist: Playlist = serde_yaml::from_str(contents)
            .map_err(|e| ChromaCatError::InputError(format!("Invalid playlist format: {e}")))?;

        // Validate all entries
        for entry in &playlist.entries {
            entry.validate()?;
        }

        Ok(playlist)
    }
}

/// Converts YAML parameters to the string format expected by the registry.
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
                            "Invalid parameter value for '{key_str}': must be number, boolean, or string"
                        )))
                    }
                };

                param_strings.push(format!("{key_str}={value_str}"));
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

/// Example playlist yaml for documentation
#[doc = include_str!("../../docs/sample-playlist.yaml")]
const _EXAMPLE: &str = "";
