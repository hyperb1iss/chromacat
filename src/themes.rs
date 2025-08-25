//! Theme system for ChromaCat
//!
//! Provides gradient theme definitions, loading, and color interpolation for terminal
//! colorization effects. Themes are loaded from YAML files at compile time and provide
//! various distribution patterns, repeat modes, and easing functions.

use crate::error::{ChromaCatError, Result};
use colorgrad::{Color, Gradient, GradientBuilder, LinearGradient};
use lazy_static::lazy_static;
use serde::de::{self, Deserializer, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_yaml::from_str;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::fmt;
use std::path::Path;
use std::sync::RwLock;

/// Color stop with RGB values and optional position/name
#[derive(Debug, Clone, Serialize)]
pub struct ColorStop {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    #[serde(default)]
    pub position: Option<f32>,
    #[serde(default)]
    pub name: Option<String>,
}

// Custom deserializer implementation for ColorStop
impl<'de> Deserialize<'de> for ColorStop {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ColorStopVisitor;

        impl<'de> Visitor<'de> for ColorStopVisitor {
            type Value = ColorStop;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a color stop array [r, g, b] or [r, g, b, position, name] or color stop object")
            }

            fn visit_seq<A>(self, mut seq: A) -> std::result::Result<ColorStop, A::Error>
            where
                A: SeqAccess<'de>,
            {
                // First three elements are always r,g,b
                let r = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let g = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let b = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;

                // Try to read optional position and name
                let position = seq.next_element()?;
                let name = seq.next_element()?;

                Ok(ColorStop {
                    r,
                    g,
                    b,
                    position,
                    name,
                })
            }

            fn visit_map<M>(self, map: M) -> std::result::Result<ColorStop, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                // Delegate to default derived implementation for structured format
                Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
            }
        }

        deserializer.deserialize_any(ColorStopVisitor)
    }
}

/// How colors are distributed across the gradient
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Distribution {
    Even,
    Front,
    Back,
    Center,
    Alt,
}

/// How the gradient repeats or cycles
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Repeat {
    Named(RepeatMode),
    Function(String, f32), // (name, rate)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepeatMode {
    None,
    Mirror,
    Repeat,
}

// Custom deserializer for Repeat
impl<'de> Deserialize<'de> for Repeat {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RepeatVisitor;

        impl Visitor<'_> for RepeatVisitor {
            type Value = Repeat;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string repeat mode or function notation (pulse/rotate)")
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Handle function notation like "pulse(1.0)" or "rotate(0.5)"
                if value.starts_with("pulse(") && value.ends_with(")") {
                    let rate = value[6..value.len() - 1]
                        .parse::<f32>()
                        .map_err(|_| E::custom("invalid pulse rate"))?;
                    Ok(Repeat::Function("pulse".to_string(), rate))
                } else if value.starts_with("rotate(") && value.ends_with(")") {
                    let rate = value[7..value.len() - 1]
                        .parse::<f32>()
                        .map_err(|_| E::custom("invalid rotation rate"))?;
                    Ok(Repeat::Function("rotate".to_string(), rate))
                } else {
                    // Handle simple mode names
                    match value {
                        "none" => Ok(Repeat::Named(RepeatMode::None)),
                        "mirror" => Ok(Repeat::Named(RepeatMode::Mirror)),
                        "repeat" => Ok(Repeat::Named(RepeatMode::Repeat)),
                        _ => Err(E::custom(format!("unknown repeat mode: {value}"))),
                    }
                }
            }
        }

        deserializer.deserialize_str(RepeatVisitor)
    }
}

/// Easing function for color transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Easing {
    Linear,
    Smooth,
    Smoother,
    Sine,
    Exp,
    Elastic,
}

/// Complete theme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeDefinition {
    pub name: String,
    pub desc: String,
    pub colors: Vec<ColorStop>,
    #[serde(default = "default_distribution")]
    pub dist: Distribution,
    #[serde(default = "default_repeat")]
    pub repeat: Repeat,
    #[serde(default = "default_speed")]
    pub speed: f32,
    #[serde(default = "default_easing")]
    pub ease: Easing,
}

fn default_distribution() -> Distribution {
    Distribution::Even
}

fn default_repeat() -> Repeat {
    Repeat::Named(RepeatMode::None)
}

fn default_speed() -> f32 {
    1.0
}

fn default_easing() -> Easing {
    Easing::Linear
}

// Include theme files at compile time
const SPACE_THEMES: &str = include_str!("../themes/space.yaml");
const TECH_THEMES: &str = include_str!("../themes/tech.yaml");
const NATURE_THEMES: &str = include_str!("../themes/nature.yaml");
const AESTHETIC_THEMES: &str = include_str!("../themes/aesthetic.yaml");
const MOOD_THEMES: &str = include_str!("../themes/mood.yaml");
const PARTY_THEMES: &str = include_str!("../themes/party.yaml");
const ABSTRACT_THEMES: &str = include_str!("../themes/abstract.yaml");
const PRIDE_THEMES: &str = include_str!("../themes/pride.yaml");
const THEORY_THEMES: &str = include_str!("../themes/theory.yaml");

lazy_static! {
    static ref THEME_REGISTRY: RwLock<ThemeRegistry> = RwLock::new(ThemeRegistry::new());
}

#[derive(Debug)]
pub struct ThemeRegistry {
    themes: HashMap<String, ThemeDefinition>,
    categories: HashMap<String, Vec<String>>,
}

impl ThemeRegistry {
    fn new() -> Self {
        let mut registry = Self {
            themes: HashMap::new(),
            categories: HashMap::new(),
        };

        // Add default rainbow theme
        let rainbow_theme = ThemeDefinition {
            name: "rainbow".to_string(),
            desc: "Default rainbow gradient".to_string(),
            colors: vec![
                ColorStop {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    position: Some(0.0),
                    name: None,
                },
                ColorStop {
                    r: 1.0,
                    g: 1.0,
                    b: 0.0,
                    position: Some(0.2),
                    name: None,
                },
                ColorStop {
                    r: 0.0,
                    g: 1.0,
                    b: 0.0,
                    position: Some(0.4),
                    name: None,
                },
                ColorStop {
                    r: 0.0,
                    g: 1.0,
                    b: 1.0,
                    position: Some(0.6),
                    name: None,
                },
                ColorStop {
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                    position: Some(0.8),
                    name: None,
                },
                ColorStop {
                    r: 1.0,
                    g: 0.0,
                    b: 1.0,
                    position: Some(1.0),
                    name: None,
                },
            ],
            dist: Distribution::Even,
            repeat: Repeat::Named(RepeatMode::None),
            speed: 1.0,
            ease: Easing::Linear,
        };

        registry.themes.insert("rainbow".to_string(), rainbow_theme);

        // Create default category
        registry
            .categories
            .insert("default".to_string(), vec!["rainbow".to_string()]);

        // Load all theme categories
        registry.load_category("space", SPACE_THEMES);
        registry.load_category("tech", TECH_THEMES);
        registry.load_category("nature", NATURE_THEMES);
        registry.load_category("aesthetic", AESTHETIC_THEMES);
        registry.load_category("mood", MOOD_THEMES);
        registry.load_category("party", PARTY_THEMES);
        registry.load_category("abstract", ABSTRACT_THEMES);
        registry.load_category("pride", PRIDE_THEMES);
        registry.load_category("theory", THEORY_THEMES);

        registry
    }

    fn load_category(&mut self, category: &str, content: &str) {
        match from_str::<Vec<ThemeDefinition>>(content) {
            Ok(themes) => {
                let mut category_themes = Vec::new();

                for theme in themes {
                    if let Err(e) = theme.validate() {
                        eprintln!("Warning: Invalid theme '{}': {}", theme.name, e);
                        continue;
                    }
                    category_themes.push(theme.name.clone());
                    self.themes.insert(theme.name.clone(), theme);
                }

                self.categories
                    .insert(category.to_string(), category_themes);
            }
            Err(e) => {
                eprintln!("Warning: Failed to load {category} themes: {e}");
            }
        }
    }

    // Add new method to load a custom theme file
    pub fn load_theme_file(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ChromaCatError::InputError(format!("Failed to read theme file: {e}")))?;

        let themes = from_str::<Vec<ThemeDefinition>>(&content).map_err(|e| {
            ChromaCatError::InvalidTheme(format!("Invalid theme file format: {e}"))
        })?;

        for theme in themes {
            if let Err(e) = theme.validate() {
                return Err(ChromaCatError::InvalidTheme(format!(
                    "Invalid theme '{}': {}",
                    theme.name, e
                )));
            }
            self.themes.insert(theme.name.clone(), theme);
        }

        Ok(())
    }
}

impl ThemeDefinition {
    pub fn validate(&self) -> Result<()> {
        if self.colors.len() < 2 {
            return Err(ChromaCatError::GradientError(
                "Theme must have at least 2 colors".to_string(),
            ));
        }

        for color in &self.colors {
            if color.r < 0.0
                || color.r > 1.0
                || color.g < 0.0
                || color.g > 1.0
                || color.b < 0.0
                || color.b > 1.0
            {
                return Err(ChromaCatError::GradientError(
                    "Color components must be between 0.0 and 1.0".to_string(),
                ));
            }

            if let Some(p) = color.position {
                if !(0.0..=1.0).contains(&p) {
                    return Err(ChromaCatError::GradientError(
                        "Color positions must be between 0.0 and 1.0".to_string(),
                    ));
                }
            }
        }

        if self.speed <= 0.0 {
            return Err(ChromaCatError::GradientError(
                "Speed must be positive".to_string(),
            ));
        }

        Ok(())
    }

    pub fn create_gradient(&self) -> Result<Box<dyn Gradient + Send + Sync>> {
        let mut colors = Vec::with_capacity(self.colors.len());
        let mut positions = Vec::with_capacity(self.colors.len());

        for color in &self.colors {
            colors.push(Color::new(color.r, color.g, color.b, 1.0));
            if let Some(p) = color.position {
                positions.push(p);
            }
        }

        let mut builder = GradientBuilder::new();
        builder.colors(&colors);

        if positions.len() == colors.len() {
            builder.domain(&positions);
        }

        let gradient = builder
            .mode(colorgrad::BlendMode::Rgb)
            .build::<LinearGradient>()
            .map_err(|e| ChromaCatError::GradientError(e.to_string()))?;

        Ok(Box::new(gradient))
    }

    pub fn apply_distribution(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self.dist {
            Distribution::Even => t,
            Distribution::Front => t * t,
            Distribution::Back => 1.0 - (1.0 - t) * (1.0 - t),
            Distribution::Center => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0) * (-2.0 * t + 2.0) / 2.0
                }
            }
            Distribution::Alt => (t * PI).sin() * 0.5 + 0.5,
        }
    }

    pub fn apply_repeat(&self, t: f32, time: f32) -> f32 {
        match &self.repeat {
            Repeat::Named(mode) => match mode {
                RepeatMode::None => t.clamp(0.0, 1.0),
                RepeatMode::Mirror => {
                    let t = t % 2.0;
                    if t > 1.0 {
                        2.0 - t
                    } else {
                        t
                    }
                }
                RepeatMode::Repeat => t.fract(),
            },
            Repeat::Function(name, rate) => match name.as_str() {
                "rotate" => (t + time * rate).fract(),
                "pulse" => {
                    let phase = (time * rate * PI).sin();
                    (t + phase) * 0.5
                }
                _ => t, // fallback
            },
        }
    }

    pub fn apply_easing(&self, t: f32) -> f32 {
        match self.ease {
            Easing::Linear => t,
            Easing::Smooth => t * t * (3.0 - 2.0 * t),
            Easing::Smoother => t * t * t * (t * (t * 6.0 - 15.0) + 10.0),
            Easing::Sine => (t * PI - PI / 2.0).sin() * 0.5 + 0.5,
            Easing::Exp => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    (2.0_f32).powf(10.0 * t - 10.0)
                }
            }
            Easing::Elastic => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    let t = t - 1.0;
                    -(2.0_f32.powf(10.0 * t) * (t * PI * 4.5).sin())
                }
            }
        }
    }
}

// Public interface for accessing themes
pub fn get_theme(name: &str) -> Result<ThemeDefinition> {
    THEME_REGISTRY
        .read()
        .map_err(|e| ChromaCatError::Other(format!("Failed to read theme registry: {e}")))?
        .themes
        .get(name)
        .cloned()
        .ok_or_else(|| ChromaCatError::InvalidTheme(name.to_string()))
}

pub fn list_category(category: &str) -> Option<Vec<String>> {
    THEME_REGISTRY.read().ok().and_then(|registry| {
        registry.categories.get(category).map(|themes| {
            let mut themes = themes.clone();
            themes.sort(); // Sort themes alphabetically
            themes
        })
    })
}

pub fn list_categories() -> Vec<String> {
    THEME_REGISTRY
        .read()
        .map(|registry| {
            let mut categories: Vec<String> = registry.categories.keys().cloned().collect();
            categories.sort(); // Sort categories alphabetically
            categories
        })
        .unwrap_or_default()
}

pub fn all_themes() -> Vec<ThemeDefinition> {
    THEME_REGISTRY
        .read()
        .map(|registry| registry.themes.values().cloned().collect())
        .unwrap_or_default()
}

pub fn theme_count() -> usize {
    THEME_REGISTRY
        .read()
        .map(|registry| registry.themes.len())
        .unwrap_or(0)
}

// Modify public interface
pub fn load_theme_file(path: &Path) -> Result<()> {
    let mut registry = THEME_REGISTRY
        .write()
        .map_err(|e| ChromaCatError::Other(format!("Failed to lock theme registry: {e}")))?;

    registry.load_theme_file(path)
}
