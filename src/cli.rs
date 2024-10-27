//! Command-line interface for ChromaCat
//!
//! This module provides the command-line argument parsing and validation for ChromaCat.
//! It handles all user input configuration and converts it into the internal configuration
//! types used by the pattern engine and renderer.

use crate::error::{ChromaCatError, Result};
use crate::pattern::{CommonParams, PatternConfig, PatternParams};
use crate::renderer::AnimationConfig;
use crate::themes;
use crate::cli_format::CliFormat;

use clap::{Parser, ValueEnum};
use std::f64::consts::TAU;
use std::path::PathBuf;
use std::time::Duration;

/// ChromaCat - A versatile command-line tool for applying animated color gradients to text
#[derive(Parser, Debug)]
#[command(
    author, 
    version,
    about = format!("😺 {}Chroma{}Cat{} - Create magical color gradients for your text ✨", 
        CliFormat::TITLE_1, CliFormat::TITLE_2, CliFormat::RESET),
    long_about = None,
    help_template = "{about}\n\nUsage: {usage}\n\n{options}",
    styles = clap::builder::Styles::styled()
        .header(anstyle::AnsiColor::BrightMagenta.on_default())
        .usage(anstyle::AnsiColor::BrightCyan.on_default())
        .literal(anstyle::AnsiColor::BrightYellow.on_default())
)]
pub struct Cli {
    #[arg(
        name = "FILES",
        help_heading = CliFormat::HEADING_INPUT,
        value_name = "FILE",
        help = CliFormat::general("Input files (reads from stdin if none provided)")
    )]
    pub files: Vec<PathBuf>,

    #[arg(
        short = 'p',
        long,
        value_enum,
        default_value = "horizontal",
        help_heading = CliFormat::HEADING_CORE,
        value_name = "TYPE",
        help = CliFormat::core("Select pattern type")
    )]
    pub pattern: PatternKind,

    #[arg(
        short = 't',
        long,
        default_value = "rainbow",
        help_heading = CliFormat::HEADING_CORE,
        value_name = "NAME",
        help = CliFormat::core("Select color theme (use --list to see available)")
    )]
    pub theme: String,

    #[arg(
        short = 'f',
        long,
        default_value = "1.0",
        help_heading = CliFormat::HEADING_CORE,
        value_name = "NUM",
        help = CliFormat::core("Base frequency (0.1-10.0)")
    )]
    pub frequency: f64,

    #[arg(
        short = 'm',
        long,
        default_value = "1.0",
        help_heading = CliFormat::HEADING_CORE,
        value_name = "NUM",
        help = CliFormat::core("Pattern amplitude (0.1-2.0)")
    )]
    pub amplitude: f64,

    #[arg(
        short = 'a',
        long,
        help_heading = CliFormat::HEADING_ANIMATION,
        help = CliFormat::animation("Enable animation mode")
    )]
    pub animate: bool,

    #[arg(
        long,
        default_value = "30",
        help_heading = CliFormat::HEADING_ANIMATION,
        value_name = "NUM",
        help = CliFormat::animation("Frames per second (1-144)")
    )]
    pub fps: u32,

    #[arg(
        long,
        default_value = "0",
        help_heading = CliFormat::HEADING_ANIMATION,
        value_name = "SECS",
        help = CliFormat::animation("Duration in seconds (0 = infinite)")
    )]
    pub duration: u64,

    #[arg(
        short = 's',
        long,
        default_value = "1.0",
        help_heading = CliFormat::HEADING_ANIMATION,
        value_name = "NUM",
        help = CliFormat::animation("Animation speed (0.0-1.0)")
    )]
    pub speed: f64,

    #[arg(
        long,
        help_heading = CliFormat::HEADING_ANIMATION,
        help = CliFormat::animation("Enable smooth transitions")
    )]
    pub smooth: bool,

    #[arg(
        short = 'n',
        long = "no-color",
        help_heading = CliFormat::HEADING_GENERAL,
        help = CliFormat::general("Disable colored output")
    )]
    pub no_color: bool,

    #[arg(
        short = 'l',
        long = "list",
        help_heading = CliFormat::HEADING_GENERAL,
        help = CliFormat::general("Show available themes and patterns")
    )]
    pub list_available: bool,

    #[arg(
        long = "theme-file",
        value_name = "FILE",
        help_heading = CliFormat::HEADING_CORE,
        help = CliFormat::core("Load custom theme from YAML file")
    )]
    pub theme_file: Option<PathBuf>,

    #[command(flatten)]
    pub pattern_params: PatternParameters,
}

/// Pattern-specific parameters grouped by pattern type
#[derive(Parser, Debug, Default)]
#[command(next_help_heading = "Pattern-Specific Options")]
pub struct PatternParameters {
    // Wave & Ripple
    #[arg(
        long, 
        value_name = "NUM",
        help = format!("{} (0.1-2.0)", CliFormat::pattern("Wave height or ripple amplitude")),
        help_heading = CliFormat::HEADING_WAVE
    )]
    pub height: Option<f64>,

    #[arg(
        long, 
        value_name = "NUM", 
        help = format!("{} (0.1-5.0)", CliFormat::pattern("Number of waves")), 
        help_heading = CliFormat::HEADING_WAVE
    )]
    pub count: Option<f64>,

    #[arg(
        long, 
        value_name = "NUM", 
        help = format!("{} (0.0-2π)", CliFormat::pattern("Wave/ripple phase")), 
        help_heading = CliFormat::HEADING_WAVE
    )]
    pub phase: Option<f64>,

    #[arg(
        long, 
        value_name = "NUM", 
        help = format!("{} (0.0-1.0)", CliFormat::pattern("Vertical offset")), 
        help_heading = CliFormat::HEADING_WAVE
    )]
    pub offset: Option<f64>,

    #[arg(
        long, 
        value_name = "X", 
        help = format!("{} (0.0-1.0)", CliFormat::pattern("Ripple center X position")), 
        help_heading = CliFormat::HEADING_WAVE
    )]
    pub center_x: Option<f64>,

    #[arg(
        long, 
        value_name = "Y", 
        help = format!("{} (0.0-1.0)", CliFormat::pattern("Ripple center Y position")), 
        help_heading = CliFormat::HEADING_WAVE
    )]
    pub center_y: Option<f64>,

    #[arg(
        long, 
        value_name = "NUM", 
        help = format!("{} (0.1-5.0)", CliFormat::pattern("Distance between ripples")), 
        help_heading = CliFormat::HEADING_WAVE
    )]
    pub wavelength: Option<f64>,

    #[arg(
        long, 
        value_name = "NUM", 
        help = format!("{} (0.0-1.0)", CliFormat::pattern("Ripple fade-out rate")), 
        help_heading = CliFormat::HEADING_WAVE
    )]
    pub damping: Option<f64>,

    // Plasma & Perlin
    #[arg(
        long, 
        value_name = "NUM", 
        help = format!("{} (1.0-10.0)", CliFormat::pattern("Pattern complexity")), 
        help_heading = CliFormat::HEADING_PLASMA
    )]
    pub complexity: Option<f64>,

    #[arg(
        long, 
        value_name = "NUM", 
        help = format!("{} (0.1-5.0)", CliFormat::pattern("Pattern scale")), 
        help_heading = CliFormat::HEADING_PLASMA
    )]
    pub scale: Option<f64>,

    #[arg(
        long, 
        value_name = "NUM", 
        help = format!("{} (1-8)", CliFormat::pattern("Noise octaves")), 
        help_heading = CliFormat::HEADING_PLASMA
    )]
    pub octaves: Option<u32>,

    #[arg(
        long, 
        value_name = "NUM", 
        help = format!("{} (0.0-1.0)", CliFormat::pattern("Noise persistence")), 
        help_heading = CliFormat::HEADING_PLASMA
    )]
    pub persistence: Option<f64>,

    #[arg(
        long, 
        value_name = "NUM", 
        help = format!("{}", CliFormat::pattern("Random seed")), 
        help_heading = CliFormat::HEADING_PLASMA
    )]
    pub seed: Option<u32>,

    // Spiral & Diamond
    #[arg(
        long, 
        value_name = "NUM", 
        help = format!("{} (0.1-5.0)", CliFormat::pattern("Pattern density")), 
        help_heading = CliFormat::HEADING_SPIRAL
    )]
    pub density: Option<f64>,

    #[arg(
        long, 
        value_name = "DEG", 
        help = format!("{} (0-360)", CliFormat::pattern("Rotation angle")), 
        help_heading = CliFormat::HEADING_SPIRAL
    )]
    pub rotation: Option<f64>,

    #[arg(
        long, 
        value_name = "NUM", 
        help = format!("{} (0.1-2.0)", CliFormat::pattern("Expansion rate")), 
        help_heading = CliFormat::HEADING_SPIRAL
    )]
    pub expansion: Option<f64>,

    #[arg(
        long, 
        help = CliFormat::pattern("Reverse spiral direction"), 
        help_heading = CliFormat::HEADING_SPIRAL
    )]
    pub counterclockwise: bool,

    #[arg(
        long, 
        value_name = "NUM", 
        help = format!("{} (1-10)", CliFormat::pattern("Pattern size")), 
        help_heading = CliFormat::HEADING_SPIRAL
    )]
    pub size: Option<usize>,

    #[arg(
        long, 
        value_name = "NUM", 
        help = format!("{} (0.0-1.0)", CliFormat::pattern("Edge blur")), 
        help_heading = CliFormat::HEADING_SPIRAL
    )]
    pub blur: Option<f64>,

    #[arg(
        long, 
        value_name = "NUM", 
        help = format!("{} (0.1-5.0)", CliFormat::pattern("Diamond edge sharpness")), 
        help_heading = CliFormat::HEADING_SPIRAL
    )]
    pub sharpness: Option<f64>,

    // Diagonal & Checkerboard
    #[arg(
        long, 
        value_name = "DEG", 
        help = format!("{} (0-360)", CliFormat::pattern("Gradient angle")), 
        help_heading = CliFormat::HEADING_OTHER
    )]
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

        for category in themes::list_categories() {  // Now iterating over Vec<String>
            println!("\n\x1b[1;38;5;147m{}\x1b[0m", category);
            if let Some(theme_names) = themes::list_category(&category) {
                for name in theme_names {
                    if let Ok(theme) = themes::get_theme(&name) {
                        let preview = Self::create_theme_preview(&theme);
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
