//! Demo art types and generation
//!
//! This module provides the core types and generation logic for ChromaCat's
//! demo art system. It defines the available art patterns and handles their
//! generation with configurable parameters.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Available demo art patterns.
///
/// Each variant represents a unique visual art pattern that can be generated
/// and displayed in demo mode. These patterns showcase different aspects of
/// ChromaCat's capabilities.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DemoArt {
    /// ChromaCat ASCII logo art
    Logo,
    /// Matrix-style digital rain effect
    Matrix,
    /// Wave interference patterns
    Waves,
    /// Spiral vortex visualization
    Spiral,
    /// Source code display
    Code,
    /// ASCII art showcase
    Ascii,
    /// Box drawing pattern art
    Boxes,
    /// Organic plasma effect
    Plasma,
    /// Hypnotic vortex tunnel
    Vortex,
    /// Cellular automaton patterns
    Cells,
    /// Fluid simulation effect
    Fluid,
    /// Intricate maze pattern
    Maze,
    /// Mandala pattern
    Mandala,
    /// All demo patterns in sequence
    All,
}

impl DemoArt {
    /// Returns a list of all available art types.
    ///
    /// # Returns
    /// A slice containing all art variants except `All`.
    pub fn all_types() -> &'static [DemoArt] {
        use DemoArt::*;
        &[
            Logo, Matrix, Waves, Spiral, Code, Ascii, Boxes, Plasma, Vortex, Cells, Fluid, Maze,
            Mandala,
        ]
    }

    /// Parse art type from string.
    ///
    /// # Arguments
    /// * `s` - String representation of an art type
    ///
    /// # Returns
    /// * `Some(DemoArt)` if the string matches a known art type
    /// * `None` if the string doesn't match any art type
    pub fn from_str(s: &str) -> Option<Self> {
        use DemoArt::*;
        match s.to_lowercase().as_str() {
            "logo" => Some(Logo),
            "matrix" => Some(Matrix),
            "waves" => Some(Waves),
            "spiral" => Some(Spiral),
            "code" => Some(Code),
            "ascii" => Some(Ascii),
            "boxes" => Some(Boxes),
            "plasma" => Some(Plasma),
            "vortex" => Some(Vortex),
            "cells" => Some(Cells),
            "fluid" => Some(Fluid),
            "maze" => Some(Maze),
            "mandala" => Some(Mandala),
            "all" => Some(All),
            _ => None,
        }
    }

    /// Get string representation of the art type.
    pub fn as_str(&self) -> &'static str {
        use DemoArt::*;
        match self {
            Logo => "logo",
            Matrix => "matrix",
            Waves => "waves",
            Spiral => "spiral",
            Code => "code",
            Ascii => "ascii",
            Boxes => "boxes",
            Plasma => "plasma",
            Vortex => "vortex",
            Cells => "cells",
            Fluid => "fluid",
            Maze => "maze",
            Mandala => "mandala",
            All => "all",
        }
    }

    /// Get human-readable display name for the art type.
    pub fn display_name(&self) -> &'static str {
        use DemoArt::*;
        match self {
            Logo => "ChromaCat Logo",
            Matrix => "Digital Rain",
            Waves => "Wave Interference",
            Spiral => "Spiral Vortex",
            Code => "Source Code",
            Ascii => "ASCII Art",
            Boxes => "Box Drawing",
            Plasma => "Organic Plasma",
            Vortex => "Hypnotic Vortex",
            Cells => "Cellular Patterns",
            Fluid => "Fluid Motion",
            Maze => "Intricate Maze",
            Mandala => "Mandala Pattern",
            All => "All Patterns",
        }
    }

    /// Get description of the art pattern.
    pub fn description(&self) -> &'static str {
        use DemoArt::*;
        match self {
            Logo => "ChromaCat ASCII logo with styled text",
            Matrix => "Digital rain effect inspired by The Matrix",
            Waves => "Wave interference patterns with varying frequencies",
            Spiral => "Hypnotic spiral vortex visualization",
            Code => "Styled source code display",
            Ascii => "Collection of ASCII art pieces",
            Boxes => "Box drawing characters forming patterns",
            Plasma => "Organic plasma effect with smooth transitions",
            Vortex => "Mesmerizing vortex tunnel visualization",
            Cells => "Cellular automaton with emergent patterns",
            Fluid => "Fluid simulation with dynamic motion",
            Maze => "Intricate maze pattern with box-drawing characters",
            Mandala => "Symmetrical mandala pattern",
            All => "All available demo patterns in sequence",
        }
    }
}

/// Enable serialization for playlist integration
impl Serialize for DemoArt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Enable deserialization for playlist loading
impl<'de> Deserialize<'de> for DemoArt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DemoArt::from_str(&s)
            .ok_or_else(|| serde::de::Error::custom(format!("Invalid art type: {}", s)))
    }
}

// Add easy conversion from string types
impl FromStr for DemoArt {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s).ok_or_else(|| format!("Invalid art type: {}", s))
    }
}

impl std::fmt::Display for DemoArt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Generator settings for demo art patterns
#[derive(Debug, Clone)]
pub struct ArtSettings {
    /// Width of the output area
    pub width: u16,
    /// Height of the output area
    pub height: u16,
    /// Whether to include headers between sections
    pub include_headers: bool,
    /// Random seed for consistent generation
    pub seed: u64,
}

impl Default for ArtSettings {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
            include_headers: true,
            seed: 42,
        }
    }
}

impl ArtSettings {
    /// Create new settings with the given dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width: width.max(40),   // Ensure minimum width
            height: height.max(10), // Ensure minimum height
            ..Default::default()
        }
    }

    /// Set whether to include section headers.
    pub fn with_headers(mut self, include_headers: bool) -> Self {
        self.include_headers = include_headers;
        self
    }

    /// Set random seed for consistent generation.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }
}
