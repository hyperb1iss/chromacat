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
    /// Cityscape with sky and moon
    Cityscape,
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
            Mandala, Cityscape,
        ]
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
            Cityscape => "cityscape",
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
            Cityscape => "Night Cityscape",
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
            Cityscape => "Multi-layered cityscape with night sky and moon",
            All => "All available demo patterns in sequence",
        }
    }

    // Add a try_from_str method to maintain backward compatibility
    pub fn try_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "logo" => Some(Self::Logo),
            "matrix" => Some(Self::Matrix),
            "waves" => Some(Self::Waves),
            "spiral" => Some(Self::Spiral),
            "code" => Some(Self::Code),
            "ascii" => Some(Self::Ascii),
            "boxes" => Some(Self::Boxes),
            "plasma" => Some(Self::Plasma),
            "vortex" => Some(Self::Vortex),
            "cells" => Some(Self::Cells),
            "fluid" => Some(Self::Fluid),
            "maze" => Some(Self::Maze),
            "mandala" => Some(Self::Mandala),
            "cityscape" => Some(Self::Cityscape),
            "all" => Some(Self::All),
            _ => None,
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
        DemoArt::try_from_str(&s)
            .ok_or_else(|| serde::de::Error::custom(format!("Invalid art type: {s}")))
    }
}

// Add easy conversion from string types
impl FromStr for DemoArt {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "logo" => Ok(Self::Logo),
            "matrix" => Ok(Self::Matrix),
            "waves" => Ok(Self::Waves),
            "spiral" => Ok(Self::Spiral),
            "code" => Ok(Self::Code),
            "ascii" => Ok(Self::Ascii),
            "boxes" => Ok(Self::Boxes),
            "plasma" => Ok(Self::Plasma),
            "vortex" => Ok(Self::Vortex),
            "cells" => Ok(Self::Cells),
            "fluid" => Ok(Self::Fluid),
            "maze" => Ok(Self::Maze),
            "mandala" => Ok(Self::Mandala),
            "cityscape" => Ok(Self::Cityscape),
            "all" => Ok(Self::All),
            _ => Err(format!("Invalid art type: {s}")),
        }
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
