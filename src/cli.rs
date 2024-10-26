//! Command-line interface for ChromaCat
//!
//! This module provides the command-line argument parsing and validation for ChromaCat.
//! It handles all user input configuration and converts it into the internal configuration
//! types used by the pattern engine and renderer.

use crate::error::{ChromaCatError, Result};
use crate::pattern::{CommonParams, PatternConfig, PatternParams};
use crate::renderer::AnimationConfig;
use crate::themes::Theme;
use clap::{Parser, ValueEnum};
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

/// ChromaCat - A versatile command-line tool for applying animated color gradients to text
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Files to read. If none provided, reads from stdin
    #[arg(name = "FILES")]
    pub files: Vec<PathBuf>,

    /// Select pattern type for gradient effect
    #[arg(short = 'p', long, value_enum, default_value = "horizontal")]
    pub pattern: PatternKind,

    /// Select a built-in theme for the gradient
    #[arg(short = 't', long, default_value = "rainbow")]
    pub theme: String,

    /// Enable animation mode
    #[arg(short = 'a', long)]
    pub animate: bool,

    /// Animation frames per second (1-144)
    #[arg(long, default_value = "30")]
    pub fps: u32,

    /// Animation duration in seconds (0 for infinite)
    #[arg(long, default_value = "0")]
    pub duration: u64,

    /// Disable colored output
    #[arg(short = 'n', long = "no-color")]
    pub no_color: bool,

    /// Show available themes and patterns
    #[arg(short = 'l', long = "list")]
    pub list_available: bool,

    /// Enable smooth transitions
    #[arg(long)]
    pub smooth: bool,

    /// Base frequency of the pattern (0.1-10.0)
    #[arg(short = 'f', long, default_value = "1.0")]
    pub frequency: f64,

    /// Pattern amplitude (0.1-2.0)
    #[arg(short = 'm', long, default_value = "1.0")]
    pub amplitude: f64,

    /// Animation speed (0.0-1.0)
    #[arg(short = 's', long, default_value = "1.0")]
    pub speed: f64,

    // Pattern-specific parameters
    #[command(flatten)]
    pub pattern_params: PatternParameters,
}

/// Pattern-specific parameters grouped by pattern type
#[derive(Parser, Debug)]
pub struct PatternParameters {
    // Plasma parameters
    #[arg(long, help = "Plasma complexity (1.0-10.0)")]
    pub complexity: Option<f64>,

    #[arg(long, help = "Pattern scale (0.1-5.0)")]
    pub scale: Option<f64>,

    // Ripple parameters
    #[arg(long, help = "Ripple center X position (0.0-1.0)")]
    pub center_x: Option<f64>,

    #[arg(long, help = "Ripple center Y position (0.0-1.0)")]
    pub center_y: Option<f64>,

    #[arg(long, help = "Distance between ripples (0.1-5.0)")]
    pub wavelength: Option<f64>,

    #[arg(long, help = "Ripple fade-out rate (0.0-1.0)")]
    pub damping: Option<f64>,

    // Wave parameters
    #[arg(long, help = "Wave height (0.1-2.0)")]
    pub height: Option<f64>,

    #[arg(long, help = "Number of waves (0.1-5.0)")]
    pub count: Option<f64>,

    #[arg(long, help = "Wave phase shift (0.0-2π)")]
    pub phase: Option<f64>,

    #[arg(long, help = "Wave vertical offset (0.0-1.0)")]
    pub offset: Option<f64>,

    // Spiral parameters
    #[arg(long, help = "Spiral density (0.1-5.0)")]
    pub density: Option<f64>,

    #[arg(long, help = "Pattern rotation angle (0-360)")]
    pub rotation: Option<f64>,

    #[arg(long, help = "Spiral expansion rate (0.1-2.0)")]
    pub expansion: Option<f64>,

    #[arg(long, help = "Reverse spiral direction")]
    pub counterclockwise: bool,

    // Checker/Diamond parameters
    #[arg(long, help = "Pattern size (1-10)")]
    pub size: Option<usize>,

    #[arg(long, help = "Edge blur (0.0-1.0)")]
    pub blur: Option<f64>,

    #[arg(long, help = "Diamond edge sharpness (0.1-5.0)")]
    pub sharpness: Option<f64>,

    // Perlin parameters
    #[arg(long, help = "Perlin noise octaves (1-8)")]
    pub octaves: Option<u32>,

    #[arg(long, help = "Perlin persistence (0.0-1.0)")]
    pub persistence: Option<f64>,

    #[arg(long, help = "Random seed for noise")]
    pub seed: Option<u32>,

    // Diagonal parameters
    #[arg(long, help = "Gradient angle (0-360)")]
    pub angle: Option<i32>,
}

/// Available pattern types for gradient effects
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum PatternKind {
    Horizontal,
    Diagonal,
    Plasma,
    Ripple,
    Wave,
    Spiral,
    Checkerboard,
    Diamond,
    Perlin,
}

impl Cli {
    /// Creates pattern configuration from CLI arguments
    pub fn create_pattern_config(&self) -> Result<PatternConfig> {
        let common = CommonParams {
            frequency: self.frequency.clamp(0.1, 10.0),
            amplitude: self.amplitude.clamp(0.1, 2.0),
            speed: self.speed.clamp(0.0, 1.0),
        };

        let params = match self.pattern {
            PatternKind::Horizontal => PatternParams::Horizontal,

            PatternKind::Diagonal => {
                let angle = self.pattern_params.angle.unwrap_or(45);
                self.validate_range("angle", angle as f64, 0.0, 360.0)?;
                PatternParams::Diagonal { angle }
            }

            PatternKind::Plasma => {
                let complexity = self.pattern_params.complexity.unwrap_or(3.0);
                let scale = self.pattern_params.scale.unwrap_or(1.0);
                self.validate_range("complexity", complexity, 1.0, 10.0)?;
                self.validate_range("scale", scale, 0.1, 5.0)?;
                PatternParams::Plasma { complexity, scale }
            }

            PatternKind::Ripple => {
                let center_x = self.pattern_params.center_x.unwrap_or(0.5);
                let center_y = self.pattern_params.center_y.unwrap_or(0.5);
                let wavelength = self.pattern_params.wavelength.unwrap_or(1.0);
                let damping = self.pattern_params.damping.unwrap_or(0.5);
                self.validate_range("center_x", center_x, 0.0, 1.0)?;
                self.validate_range("center_y", center_y, 0.0, 1.0)?;
                self.validate_range("wavelength", wavelength, 0.1, 5.0)?;
                self.validate_range("damping", damping, 0.0, 1.0)?;
                PatternParams::Ripple {
                    center_x,
                    center_y,
                    wavelength,
                    damping,
                }
            }

            PatternKind::Wave => {
                let height = self.pattern_params.height.unwrap_or(1.0);
                let count = self.pattern_params.count.unwrap_or(1.0);
                let phase = self.pattern_params.phase.unwrap_or(0.0);
                let offset = self.pattern_params.offset.unwrap_or(0.5);
                self.validate_range("height", height, 0.1, 2.0)?;
                self.validate_range("count", count, 0.1, 5.0)?;
                self.validate_range("phase", phase, 0.0, 6.28)?;
                self.validate_range("offset", offset, 0.0, 1.0)?;
                PatternParams::Wave {
                    amplitude: height,
                    frequency: count,
                    phase,
                    offset,
                }
            }

            PatternKind::Spiral => {
                let density = self.pattern_params.density.unwrap_or(1.0);
                let rotation = self.pattern_params.rotation.unwrap_or(0.0);
                let expansion = self.pattern_params.expansion.unwrap_or(1.0);
                self.validate_range("density", density, 0.1, 5.0)?;
                self.validate_range("rotation", rotation, 0.0, 360.0)?;
                self.validate_range("expansion", expansion, 0.1, 2.0)?;
                PatternParams::Spiral {
                    density,
                    rotation,
                    expansion,
                    clockwise: !self.pattern_params.counterclockwise,
                }
            }

            PatternKind::Checkerboard => {
                let size = self.pattern_params.size.unwrap_or(2);
                let blur = self.pattern_params.blur.unwrap_or(0.0);
                let rotation = self.pattern_params.rotation.unwrap_or(0.0);
                let scale = self.pattern_params.scale.unwrap_or(1.0);
                self.validate_range("size", size as f64, 1.0, 10.0)?;
                self.validate_range("blur", blur, 0.0, 1.0)?;
                self.validate_range("rotation", rotation, 0.0, 360.0)?;
                self.validate_range("scale", scale, 0.1, 5.0)?;
                PatternParams::Checkerboard {
                    size,
                    blur,
                    rotation,
                    scale,
                }
            }

            PatternKind::Diamond => {
                let size = self.pattern_params.size.unwrap_or(1).clamp(1, 10) as f64;
                let offset = self.pattern_params.offset.unwrap_or(0.0);
                let sharpness = self.pattern_params.sharpness.unwrap_or(1.0);
                let rotation = self.pattern_params.rotation.unwrap_or(0.0);
                self.validate_range("sharpness", sharpness, 0.1, 5.0)?;
                self.validate_range("rotation", rotation, 0.0, 360.0)?;
                PatternParams::Diamond {
                    size,
                    offset,
                    sharpness,
                    rotation,
                }
            }

            PatternKind::Perlin => {
                let octaves = self.pattern_params.octaves.unwrap_or(4);
                let persistence = self.pattern_params.persistence.unwrap_or(0.5);
                let scale = self.pattern_params.scale.unwrap_or(1.0);
                let seed = self.pattern_params.seed.unwrap_or(0);
                self.validate_range("octaves", octaves as f64, 1.0, 8.0)?;
                self.validate_range("persistence", persistence, 0.0, 1.0)?;
                self.validate_range("scale", scale, 0.1, 5.0)?;
                PatternParams::Perlin {
                    octaves,
                    persistence,
                    scale,
                    seed,
                }
            }
        };

        Ok(PatternConfig { common, params })
    }

    /// Creates animation configuration from CLI arguments
    pub fn create_animation_config(&self) -> AnimationConfig {
        AnimationConfig {
            fps: self.fps.clamp(1, 144),
            cycle_duration: if self.duration == 0 {
                Duration::from_secs(u64::MAX)
            } else {
                Duration::from_secs(self.duration)
            },
            infinite: self.duration == 0,
            show_progress: true,
            smooth: self.smooth,
        }
    }

    /// Creates a colored preview string for a theme
    fn create_theme_preview(theme: &Theme) -> String {
        let gradient = theme
            .create_gradient()
            .unwrap_or_else(|_| Theme::Rainbow.create_gradient().unwrap());

        let mut preview = String::new();
        // Create a gradient preview using background colors
        for i in 0..30 {
            let t = i as f32 / 30.0;
            let color = gradient.at(t);
            let r = (color.r * 255.0) as u8;
            let g = (color.g * 255.0) as u8;
            let b = (color.b * 255.0) as u8;
            // Use space with background color instead of foreground colored block
            preview.push_str(&format!("\x1b[48;2;{};{};{}m \x1b[0m", r, g, b));
        }
        preview
    }

    /// Prints available themes and patterns
    pub fn print_available_options() {
        // Simple, elegant header
        println!("\n\x1b[1;38;5;213m✨ ChromaCat Theme Gallery ✨\x1b[0m");

        // Print patterns section with styled header
        println!("\n\x1b[1;38;5;39m🎮 Available Patterns\x1b[0m");
        println!("\x1b[38;5;239m{}\x1b[0m", "─".repeat(50));

        for pattern in [
            ("horizontal", "Simple left-to-right gradient"),
            ("diagonal", "Gradient at specified angle"),
            ("plasma", "Psychedelic plasma effect using sine waves"),
            ("ripple", "Concentric circles emanating from center"),
            ("wave", "Flowing wave distortion pattern"),
            ("spiral", "Spiral pattern from center"),
            ("checkerboard", "Alternating gradient colors in a grid"),
            ("diamond", "Diamond-shaped gradient pattern"),
            ("perlin", "Organic, cloud-like noise pattern"),
        ] {
            println!("  \x1b[1;38;5;75m{:<12}\x1b[0m │ {}", pattern.0, pattern.1);
        }

        println!("\n\x1b[1;38;5;213m🎨 Theme Collection\x1b[0m");
        println!("\x1b[38;5;239m{}\x1b[0m", "─".repeat(50));

        let categories = [
            ("Classic Themes", "🌈", 226),   // Yellow
            ("Nature Themes", "🌿", 84),     // Green
            ("Tech Themes", "💻", 39),       // Blue
            ("Aesthetic Themes", "✨", 213), // Pink
            ("Space Themes", "🌌", 99),      // Purple
            ("Abstract Themes", "🎯", 203),  // Coral
            ("Mood Themes", "🎭", 147),      // Lavender
            ("Party Themes", "🎉", 214),     // Orange
            ("Color Theory", "🎨", 159),     // Cyan
            ("Special Effects", "⚡", 227),  // Light Yellow
        ];

        let themes = Theme::list_all();

        for (category, emoji, color) in categories {
            // Print category header with custom styling
            println!(
                "\n\x1b[1;38;5;{}m{} {} {}\x1b[0m",
                color, emoji, category, emoji
            );
            println!("\x1b[38;5;239m{}\x1b[0m", "─".repeat(40));

            let category_themes: Vec<_> = themes
                .iter()
                .filter(|(name, _)| {
                    matches!(
                        (category, name.as_str()),
                        (
                            "Classic Themes",
                            "rainbow" | "grayscale" | "sepia" | "monochrome"
                        ) | (
                            "Nature Themes",
                            "ocean"
                                | "forest"
                                | "autumn"
                                | "sunset"
                                | "desert"
                                | "arctic"
                                | "tropical"
                        ) | (
                            "Tech Themes",
                            "matrix" | "cyberpunk" | "terminal" | "hackerman"
                        ) | (
                            "Aesthetic Themes",
                            "pastel" | "neon" | "retrowave" | "vaporwave"
                        ) | ("Space Themes", "nebula" | "galaxy" | "aurora" | "cosmos")
                            | ("Abstract Themes", "heat" | "ice" | "fire" | "toxic")
                            | ("Mood Themes", "calm" | "energy" | "dream")
                            | ("Party Themes", "rave" | "disco" | "festival")
                            | ("Color Theory", "complementary" | "analogous" | "triadic")
                            | (
                                "Special Effects",
                                "hologram" | "glitch" | "plasma" | "lightning"
                            )
                    )
                })
                .collect();

            // Print themes in this category with color previews
            for (name, description) in category_themes {
                let theme = Theme::from_str(name).unwrap();
                let preview = Self::create_theme_preview(&theme);
                println!(
                    "  \x1b[1;38;5;{}m{:<12}\x1b[0m {} \x1b[38;5;239m│\x1b[0m {}",
                    color, name, preview, description
                );
            }
        }

        Self::print_styled_examples();
    }

    /// Prints styled example usage
    fn print_styled_examples() {
        println!("\n\x1b[1;38;5;39m📚 Example Usage\x1b[0m");
        println!("\x1b[38;5;239m{}\x1b[0m", "─".repeat(50));

        let examples = [
            (
                "Basic usage with file input:",
                "chromacat file1.txt file2.txt",
            ),
            (
                "Plasma effect with animation:",
                "chromacat -p plasma --complexity 3.0 --scale 1.5 -a input.txt",
            ),
            (
                "Ocean theme with ripple effect:",
                "chromacat -p ripple --wavelength 0.5 --damping 0.3 -t ocean *.txt",
            ),
            (
                "Pipe from another command:",
                "ls -l | chromacat -p wave --height 1.5 --count 3",
            ),
            (
                "Infinite animation:",
                "chromacat -p spiral --density 2.0 -a --duration 0 file.txt",
            ),
        ];

        for (description, command) in examples {
            println!("  \x1b[38;5;75m{}\x1b[0m", description);
            println!(
                "  \x1b[1;38;5;239m$\x1b[0m \x1b[38;5;222m{}\x1b[0m\n",
                command
            );
        }
    }

    /// Validates the CLI arguments
    pub fn validate(&self) -> Result<()> {
        // Skip validation if just listing options
        if self.list_available {
            return Ok(());
        }

        // Validate animation parameters
        if self.fps < 1 || self.fps > 144 {
            return Err(ChromaCatError::InvalidParameter {
                name: "fps".to_string(),
                value: self.fps as f64,
                min: 1.0,
                max: 144.0,
            });
        }

        // Validate input files
        for path in &self.files {
            if !path.exists() {
                return Err(ChromaCatError::InputError(format!(
                    "Input file not found: {}",
                    path.display()
                )));
            }
        }

        // Validate theme
        Theme::from_str(&self.theme)
            .map_err(|_| ChromaCatError::InvalidTheme(self.theme.clone()))?;

        // Validate common parameters
        self.validate_range("frequency", self.frequency, 0.1, 10.0)?;
        self.validate_range("amplitude", self.amplitude, 0.1, 2.0)?;
        self.validate_range("speed", self.speed, 0.0, 1.0)?;

        Ok(())
    }

    /// Validates a parameter is within the specified range
    fn validate_range(&self, name: &str, value: f64, min: f64, max: f64) -> Result<()> {
        if value < min || value > max {
            return Err(ChromaCatError::InvalidParameter {
                name: name.to_string(),
                value,
                min,
                max,
            });
        }
        Ok(())
    }
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            pattern: PatternKind::Horizontal,
            theme: "rainbow".to_string(),
            animate: false,
            fps: 30,
            duration: 0,
            no_color: false,
            list_available: false,
            smooth: false,
            frequency: 1.0,
            amplitude: 1.0,
            speed: 1.0,
            pattern_params: PatternParameters::default(),
        }
    }
}

impl Default for PatternParameters {
    fn default() -> Self {
        Self {
            complexity: None,
            scale: None,
            center_x: None,
            center_y: None,
            wavelength: None,
            damping: None,
            height: None,
            count: None,
            phase: None,
            offset: None,
            density: None,
            rotation: None,
            expansion: None,
            counterclockwise: false,
            size: None,
            blur: None,
            sharpness: None,
            octaves: None,
            persistence: None,
            seed: None,
            angle: None,
        }
    }
}

impl std::fmt::Display for PatternKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Horizontal => write!(f, "horizontal"),
            Self::Diagonal => write!(f, "diagonal"),
            Self::Plasma => write!(f, "plasma"),
            Self::Ripple => write!(f, "ripple"),
            Self::Wave => write!(f, "wave"),
            Self::Spiral => write!(f, "spiral"),
            Self::Checkerboard => write!(f, "checkerboard"),
            Self::Diamond => write!(f, "diamond"),
            Self::Perlin => write!(f, "perlin"),
        }
    }
}
