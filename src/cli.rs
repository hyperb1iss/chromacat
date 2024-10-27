//! Command-line interface for ChromaCat
//!
//! This module provides the command-line argument parsing and validation for ChromaCat.
//! It handles all user input configuration and converts it into the internal configuration
//! types used by the pattern engine and renderer.

use crate::error::{ChromaCatError, Result};
use crate::pattern::{
    CommonParams, PatternConfig, PatternParams, PatternParam, ParamType,
    CheckerboardParams, DiagonalParams, DiamondParams, HorizontalParams,
    PerlinParams, PlasmaParams, RippleParams, SpiralParams, WaveParams,
};
use crate::renderer::AnimationConfig;
use crate::themes;
use crate::cli_format::CliFormat;

use clap::{Parser, ValueEnum};
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
    #[arg(
        long = "param",
        value_name = "KEY=VALUE",
        help = "Pattern-specific parameter (can be used multiple times)",
        value_parser = parse_param_value
    )]
    pub params: Vec<String>,
}

fn parse_param_value(s: &str) -> std::result::Result<String, String> {
    if !s.contains('=') {
        return Err("Parameter must be in format key=value".to_string());
    }
    let parts: Vec<&str> = s.split('=').collect();
    if parts.len() != 2 {
        return Err("Parameter must be in format key=value".to_string());
    }
    Ok(s.to_string())
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

        // Get default parameters for the selected pattern
        let default_params = self.pattern.default_params();
        
        // Validate and parse any provided parameter values
        let params_str = self.pattern_params.params.join(",");
        if !params_str.is_empty() {
            default_params.validate(&params_str)?;
        }

        let pattern_params = if !params_str.is_empty() {
            default_params.parse(&params_str)?
        } else {
            default_params
        };

        // Convert to the correct PatternParams variant
        let params = match self.pattern {
            PatternKind::Horizontal => PatternParams::Horizontal(
                pattern_params.as_any().downcast_ref::<HorizontalParams>()
                    .expect("Failed to downcast horizontal parameters")
                    .clone()
            ),
            PatternKind::Diagonal => PatternParams::Diagonal(
                pattern_params.as_any().downcast_ref::<DiagonalParams>()
                    .expect("Failed to downcast diagonal parameters")
                    .clone()
            ),
            PatternKind::Plasma => PatternParams::Plasma(
                pattern_params.as_any().downcast_ref::<PlasmaParams>()
                    .expect("Failed to downcast plasma parameters")
                    .clone()
            ),
            PatternKind::Ripple => PatternParams::Ripple(
                pattern_params.as_any().downcast_ref::<RippleParams>()
                    .expect("Failed to downcast ripple parameters")
                    .clone()
            ),
            PatternKind::Wave => PatternParams::Wave(
                pattern_params.as_any().downcast_ref::<WaveParams>()
                    .expect("Failed to downcast wave parameters")
                    .clone()
            ),
            PatternKind::Spiral => PatternParams::Spiral(
                pattern_params.as_any().downcast_ref::<SpiralParams>()
                    .expect("Failed to downcast spiral parameters")
                    .clone()
            ),
            PatternKind::Checkerboard => PatternParams::Checkerboard(
                pattern_params.as_any().downcast_ref::<CheckerboardParams>()
                    .expect("Failed to downcast checkerboard parameters")
                    .clone()
            ),
            PatternKind::Diamond => PatternParams::Diamond(
                pattern_params.as_any().downcast_ref::<DiamondParams>()
                    .expect("Failed to downcast diamond parameters")
                    .clone()
            ),
            PatternKind::Perlin => PatternParams::Perlin(
                pattern_params.as_any().downcast_ref::<PerlinParams>()
                    .expect("Failed to downcast perlin parameters")
                    .clone()
            ),
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

        // Print patterns section with their parameters
        println!("\x1b[1;38;5;39m🎮 Patterns\x1b[0m");
        println!("\x1b[38;5;239m{}\x1b[0m", "─".repeat(80));

        for pattern_kind in [
            PatternKind::Horizontal,
            PatternKind::Diagonal,
            PatternKind::Plasma,
            PatternKind::Ripple,
            PatternKind::Wave,
            PatternKind::Spiral,
            PatternKind::Checkerboard,
            PatternKind::Diamond,
            PatternKind::Perlin,
        ] {
            let params = pattern_kind.default_params();
            println!("\n\x1b[1;38;5;75m{}\x1b[0m", params.name());
            println!("  {}", params.description());
            
            // Print parameter details
            for param in params.sub_params() {
                let range = match param.param_type() {
                    ParamType::Number { min, max } => format!(" ({}-{})", min, max),
                    ParamType::Boolean => " (true/false)".to_string(),
                    ParamType::Enum { options } => format!(" ({})", options.join("/")),
                    _ => String::new(),
                };
                println!("  \x1b[38;5;147m--param {}={}\x1b[0m{}", 
                    param.name(),
                    param.default_value(),
                    range
                );
                println!("    {}", param.description());
            }
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

impl PatternKind {
    fn default_params(&self) -> Box<dyn PatternParam> {
        match self {
            Self::Horizontal => Box::new(HorizontalParams::default()),
            Self::Diagonal => Box::new(DiagonalParams::default()),
            Self::Plasma => Box::new(PlasmaParams::default()),
            Self::Ripple => Box::new(RippleParams::default()),
            Self::Wave => Box::new(WaveParams::default()),
            Self::Spiral => Box::new(SpiralParams::default()),
            Self::Checkerboard => Box::new(CheckerboardParams::default()),
            Self::Diamond => Box::new(DiamondParams::default()),
            Self::Perlin => Box::new(PerlinParams::default()),
        }
    }
}
