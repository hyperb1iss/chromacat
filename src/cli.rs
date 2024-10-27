//! Command-line interface for ChromaCat
//!
//! This module provides the command-line argument parsing and validation for ChromaCat.
//! It handles all user input configuration and converts it into the internal configuration
//! types used by the pattern engine and renderer.

use crate::error::{ChromaCatError, Result};
use crate::pattern::{CommonParams, PatternConfig, PatternParams};
use crate::renderer::AnimationConfig;
use crate::themes;

use clap::{Parser, ValueEnum};
use std::f64::consts::TAU;
use std::path::PathBuf;
use std::time::Duration;

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

    /// Select a theme for the gradient effect
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
#[derive(Parser, Debug, Default)]
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
            frequency: self.frequency,
            amplitude: self.amplitude,
            speed: self.speed,
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
                self.validate_range("phase", phase, 0.0, TAU)?;
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

    /// Prints available themes and patterns
    pub fn print_available_options() {
        println!("\n\x1b[1;38;5;213m✨ ChromaCat Theme Gallery ✨\x1b[0m\n");

        // Print patterns section
        println!("\x1b[1;38;5;39m🎮 Patterns\x1b[0m");
        println!("\x1b[38;5;239m{}\x1b[0m", "─".repeat(80));

        let patterns = [
            ("horizontal", "Simple left-to-right gradient"),
            ("diagonal", "Gradient at specified angle"),
            ("plasma", "Psychedelic plasma effect"),
            ("ripple", "Concentric ripple effect"),
            ("wave", "Wave distortion pattern"),
            ("spiral", "Spiral gradient pattern"),
            ("checkerboard", "Checkerboard pattern"),
            ("diamond", "Diamond pattern"),
            ("perlin", "Perlin noise pattern"),
        ];

        for (name, desc) in patterns {
            println!("  \x1b[1;38;5;75m{:<15}\x1b[0m {}", name, desc);
        }

        // Print theme categories
        println!("\n\x1b[1;38;5;213m🎨 Themes\x1b[0m");
        println!("\x1b[38;5;239m{}\x1b[0m", "─".repeat(80));

        for category in themes::list_categories() {
            println!("\n\x1b[1;38;5;147m{}\x1b[0m", category);
            if let Some(theme_names) = themes::list_category(category) {
                for name in theme_names {
                    if let Ok(theme) = themes::get_theme(name) {
                        let preview = Self::create_theme_preview(theme);
                        println!(
                            "  \x1b[1;38;5;75m{:<15}\x1b[0m {} \x1b[38;5;239m│\x1b[0m {}",
                            name, preview, theme.desc
                        );
                    }
                }
            }
        }

        Self::print_usage_examples();
    }

    fn create_theme_preview(theme: &themes::ThemeDefinition) -> String {
        if let Ok(gradient) = theme.create_gradient() {
            let mut preview = String::new();
            for i in 0..30 {
                let t = i as f32 / 29.0;
                let color = gradient.at(t);
                let r = (color.r * 255.0) as u8;
                let g = (color.g * 255.0) as u8;
                let b = (color.b * 255.0) as u8;
                preview.push_str(&format!("\x1b[48;2;{};{};{}m \x1b[0m", r, g, b));
            }
            preview
        } else {
            " ".repeat(30)
        }
    }

    fn print_usage_examples() {
        println!("\n\x1b[1;38;5;39m📚 Usage Examples\x1b[0m");
        println!("\x1b[38;5;239m{}\x1b[0m", "─".repeat(80));

        let examples = [
            ("Basic file colorization:", "chromacat input.txt"),
            ("Using a specific theme:", "chromacat -t ocean input.txt"),
            ("Animated output:", "chromacat -a --fps 60 input.txt"),
            ("Pipe from another command:", "ls -la | chromacat -t neon"),
            (
                "Pattern with custom parameters:",
                "chromacat -p wave --height 1.5 --count 3 input.txt",
            ),
            ("Multiple files with animation:", "chromacat -a *.txt"),
            (
                "Custom diagonal gradient:",
                "chromacat -p diagonal --angle 45 --speed 0.8 input.txt",
            ),
            (
                "Interactive plasma effect:",
                "chromacat -p plasma --complexity 3.0 --scale 1.5 -a input.txt",
            ),
        ];

        for (desc, cmd) in examples {
            println!("  \x1b[38;5;75m{}\x1b[0m", desc);
            println!(
                "    \x1b[1;38;5;239m$\x1b[0m \x1b[38;5;222m{}\x1b[0m\n",
                cmd
            );
        }

        // Print pattern-specific parameters
        println!("\n\x1b[1;38;5;39m🔧 Pattern Parameters\x1b[0m");
        println!("\x1b[38;5;239m{}\x1b[0m", "─".repeat(80));

        let parameter_docs = [
            (
                "Plasma Pattern",
                vec![
                    ("--complexity <1.0-10.0>", "Number of plasma layers"),
                    ("--scale <0.1-5.0>", "Pattern scale factor"),
                ],
            ),
            (
                "Wave Pattern",
                vec![
                    ("--height <0.1-2.0>", "Wave amplitude"),
                    ("--count <0.1-5.0>", "Number of waves"),
                    ("--phase <0.0-2π>", "Wave phase shift"),
                ],
            ),
            (
                "Ripple Pattern",
                vec![
                    ("--center-x/y <0.0-1.0>", "Ripple center position"),
                    ("--wavelength <0.1-5.0>", "Distance between ripples"),
                    ("--damping <0.0-1.0>", "Ripple fade-out rate"),
                ],
            ),
            (
                "Spiral Pattern",
                vec![
                    ("--density <0.1-5.0>", "Spiral density"),
                    ("--expansion <0.1-2.0>", "Spiral growth rate"),
                    ("--counterclockwise", "Reverse spiral direction"),
                ],
            ),
        ];

        for (pattern, params) in parameter_docs {
            println!("\n  \x1b[1;38;5;75m{}\x1b[0m", pattern);
            for (param, desc) in params {
                println!("    \x1b[38;5;222m{:<25}\x1b[0m {}", param, desc);
            }
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

        // Validate input files exist
        for path in &self.files {
            if !path.exists() {
                return Err(ChromaCatError::InputError(format!(
                    "Input file not found: {}",
                    path.display()
                )));
            }
        }

        // Validate theme exists
        themes::get_theme(&self.theme)?;

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
