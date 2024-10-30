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
        help = CliFormat::highlight_description("Input files (reads from stdin if none provided)")
    )]
    pub files: Vec<PathBuf>,

    #[arg(
        short = 'p',
        long,
        value_enum,
        default_value = "diagonal",
        help_heading = CliFormat::HEADING_CORE,
        value_name = "TYPE",
        help = CliFormat::highlight_description("Select pattern type for the color gradient")
    )]
    pub pattern: PatternKind,

    #[arg(
        short = 't',
        long,
        default_value = "rainbow",
        help_heading = CliFormat::HEADING_CORE,
        value_name = "NAME",
        help = CliFormat::highlight_description("Select color theme (use --list to see available)")
    )]
    pub theme: String,

    #[arg(
        short = 'f',
        long,
        default_value = "1.0",
        help_heading = CliFormat::HEADING_CORE,
        value_name = "NUM",
        help = CliFormat::highlight_description("Base frequency (0.1-10.0)")
    )]
    pub frequency: f64,

    #[arg(
        short = 'm',
        long,
        default_value = "1.0",
        help_heading = CliFormat::HEADING_CORE,
        value_name = "NUM",
        help = CliFormat::highlight_description("Pattern amplitude (0.1-2.0)")
    )]
    pub amplitude: f64,

    #[arg(
        short = 'a',
        long,
        help_heading = CliFormat::HEADING_ANIMATION,
        help = CliFormat::highlight_description("Enable animation mode")
    )]
    pub animate: bool,

    #[arg(
        long,
        default_value = "30",
        help_heading = CliFormat::HEADING_ANIMATION,
        value_name = "NUM",
        help = CliFormat::highlight_description("Frames per second (1-144)")
    )]
    pub fps: u32,

    #[arg(
        long,
        default_value = "0",
        help_heading = CliFormat::HEADING_ANIMATION,
        value_name = "SECS",
        help = CliFormat::highlight_description("Duration in seconds (0 = infinite)")
    )]
    pub duration: u64,

    #[arg(
        short = 's',
        long,
        default_value = "1.0",
        help_heading = CliFormat::HEADING_ANIMATION,
        value_name = "NUM",
        help = CliFormat::highlight_description("Animation speed (0.0-1.0)")
    )]
    pub speed: f64,

    #[arg(
        long,
        help_heading = CliFormat::HEADING_ANIMATION,
        help = CliFormat::highlight_description("Enable smooth transitions")
    )]
    pub smooth: bool,

    #[arg(
        short = 'n',
        long = "no-color",
        help_heading = CliFormat::HEADING_GENERAL,
        help = CliFormat::highlight_description("Disable colored output")
    )]
    pub no_color: bool,

    #[arg(
        short = 'l',
        long = "list",
        help_heading = CliFormat::HEADING_GENERAL,
        help = CliFormat::highlight_description("Show available themes and patterns")
    )]
    pub list_available: bool,

    #[arg(
        long = "theme-file",
        value_name = "FILE",
        help_heading = CliFormat::HEADING_CORE,
        help = CliFormat::highlight_description("Load custom theme from YAML file")
    )]
    pub theme_file: Option<PathBuf>,

    #[command(flatten)]
    pub pattern_params: PatternParameters,

    #[arg(
        long = "pattern-help",
        help_heading = CliFormat::HEADING_GENERAL,
        help = CliFormat::highlight_description("Show detailed help for pattern parameters")
    )]
    pub pattern_help: bool,

    #[arg(
        long = "no-aspect-correction",
        help_heading = CliFormat::HEADING_GENERAL,
        help = CliFormat::highlight_description("Disable terminal character aspect ratio correction")
    )]
    pub no_aspect_correction: bool,

    #[arg(
        long = "aspect-ratio",
        value_name = "RATIO",
        default_value = "0.5",
        help_heading = CliFormat::HEADING_GENERAL,
        help = CliFormat::highlight_description("Set terminal character aspect ratio (width/height, default: 0.5)")
    )]
    pub aspect_ratio: f64,

    #[arg(
        long = "buffer-size",
        value_name = "BYTES",
        help_heading = CliFormat::HEADING_CORE,
        help = CliFormat::highlight_description("Set input buffer size for streaming mode (default: 8192)")
    )]
    pub buffer_size: Option<usize>,

    #[arg(
        long = "demo",
        help_heading = CliFormat::HEADING_GENERAL,
        help = CliFormat::highlight_description("Run in demo mode with generated patterns")
    )]
    pub demo: bool,
}

/// Pattern-specific parameters grouped by pattern type
#[derive(Parser, Debug, Default)]
#[command(next_help_heading = "Pattern-Specific Options")]
pub struct PatternParameters {
    #[arg(
        long = "param",
        value_name = "KEY=VALUE",
        help = CliFormat::highlight_description("Pattern-specific parameter (can be used multiple times)"),
        value_parser = parse_param_value
    )]
    pub params: Vec<String>,
}

fn parse_param_value(s: &str) -> std::result::Result<String, String> {
    // Split by commas first to handle multiple parameters
    let param_pairs: Vec<&str> = s.split(',').collect();

    // Reject empty string
    if s.trim().is_empty() {
        return Err("Parameter cannot be empty".to_string());
    }

    // Validate each key=value pair
    for pair in &param_pairs {
        let pair = pair.trim();

        // Reject empty pairs from multiple commas
        if pair.is_empty() {
            return Err("Empty parameter pair is not allowed".to_string());
        }

        if !pair.contains('=') {
            return Err(format!("Parameter '{}' must be in format key=value", pair));
        }

        let parts: Vec<&str> = pair.split('=').collect();
        if parts.len() != 2 {
            return Err(format!("Parameter '{}' must be in format key=value", pair));
        }

        // Check for empty key or value
        let key = parts[0].trim();
        let value = parts[1].trim();
        if key.is_empty() {
            return Err("Parameter key cannot be empty".to_string());
        }
        if value.is_empty() {
            return Err("Parameter value cannot be empty".to_string());
        }
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

impl Default for PatternKind {
    fn default() -> Self {
        Self::Diagonal
    }
}

trait PadToWidth {
    fn pad_to_width(&self, width: usize) -> String;
}

impl PadToWidth for String {
    fn pad_to_width(&self, width: usize) -> String {
        if self.len() >= width {
            self.clone()
        } else {
            format!("{:<width$}", self, width = width)
        }
    }
}

impl PadToWidth for &str {
    fn pad_to_width(&self, width: usize) -> String {
        if self.len() >= width {
            self.to_string()
        } else {
            format!("{:<width$}", self, width = width)
        }
    }
}

impl Cli {
    /// Creates pattern configuration from CLI arguments
    pub fn create_pattern_config(&self) -> Result<PatternConfig> {
        let common = CommonParams {
            frequency: self.frequency,
            amplitude: self.amplitude,
            speed: self.speed,
            correct_aspect: !self.no_aspect_correction,
            aspect_ratio: self.aspect_ratio,
            theme_name: Some(self.theme.clone()),
        };

        // Get default parameters for the selected pattern
        let default_params = self.pattern.default_params();

        // If we have parameter overrides, validate and parse them
        if !self.pattern_params.params.is_empty() {
            // Collect all parameters into a single comma-separated string
            let all_params: Vec<String> = self.pattern_params.params.iter()
                .flat_map(|p| p.split(','))
                .map(|s| s.trim().to_string())
                .collect();

            let params_str = all_params.join(",");

            // Validate parameters
            if let Err(e) = default_params.validate(&params_str) {
                return Err(ChromaCatError::PatternError {
                    pattern: self.pattern.to_string(),
                    param: "params".to_string(),
                    message: e,
                });
            }

            // Parse parameters
            let parsed = default_params.parse(&params_str)
                .map_err(|e| {
                    ChromaCatError::PatternError {
                        pattern: self.pattern.to_string(),
                        param: "params".to_string(),
                        message: e,
                    }
                })?;

            // Convert to PatternParams using pattern-specific conversion
            let params = match self.pattern {
                PatternKind::Horizontal => {
                    let p = parsed.as_any().downcast_ref::<HorizontalParams>()
                        .ok_or_else(|| ChromaCatError::Other("Failed to parse horizontal parameters".to_string()))?;
                    PatternParams::Horizontal(p.clone())
                },
                PatternKind::Diagonal => {
                    let p = parsed.as_any().downcast_ref::<DiagonalParams>()
                        .ok_or_else(|| ChromaCatError::Other("Failed to parse diagonal parameters".to_string()))?;
                    PatternParams::Diagonal(p.clone())
                },
                PatternKind::Plasma => {
                    let p = parsed.as_any().downcast_ref::<PlasmaParams>()
                        .ok_or_else(|| ChromaCatError::Other("Failed to parse plasma parameters".to_string()))?;
                    PatternParams::Plasma(p.clone())
                },
                PatternKind::Ripple => {
                    let p = parsed.as_any().downcast_ref::<RippleParams>()
                        .ok_or_else(|| ChromaCatError::Other("Failed to parse ripple parameters".to_string()))?;
                    PatternParams::Ripple(p.clone())
                },
                PatternKind::Wave => {
                    let p = parsed.as_any().downcast_ref::<WaveParams>()
                        .ok_or_else(|| ChromaCatError::Other("Failed to parse wave parameters".to_string()))?;
                    PatternParams::Wave(p.clone())
                },
                PatternKind::Spiral => {
                    let p = parsed.as_any().downcast_ref::<SpiralParams>()
                        .ok_or_else(|| ChromaCatError::Other("Failed to parse spiral parameters".to_string()))?;
                    PatternParams::Spiral(p.clone())
                },
                PatternKind::Checkerboard => {
                    let p = parsed.as_any().downcast_ref::<CheckerboardParams>()
                        .ok_or_else(|| ChromaCatError::Other("Failed to parse checkerboard parameters".to_string()))?;
                    PatternParams::Checkerboard(p.clone())
                },
                PatternKind::Diamond => {
                    let p = parsed.as_any().downcast_ref::<DiamondParams>()
                        .ok_or_else(|| ChromaCatError::Other("Failed to parse diamond parameters".to_string()))?;
                    PatternParams::Diamond(p.clone())
                },
                PatternKind::Perlin => {
                    let p = parsed.as_any().downcast_ref::<PerlinParams>()
                        .ok_or_else(|| ChromaCatError::Other("Failed to parse perlin parameters".to_string()))?;
                    PatternParams::Perlin(p.clone())
                },
            };

            return Ok(PatternConfig { common, params });
        }

        // If no parameters provided, use defaults
        let params = match self.pattern {
            PatternKind::Horizontal => PatternParams::Horizontal(HorizontalParams::default()),
            PatternKind::Diagonal => PatternParams::Diagonal(DiagonalParams::default()),
            PatternKind::Plasma => PatternParams::Plasma(PlasmaParams::default()),
            PatternKind::Ripple => PatternParams::Ripple(RippleParams::default()),
            PatternKind::Wave => PatternParams::Wave(WaveParams::default()),
            PatternKind::Spiral => PatternParams::Spiral(SpiralParams::default()),
            PatternKind::Checkerboard => PatternParams::Checkerboard(CheckerboardParams::default()),
            PatternKind::Diamond => PatternParams::Diamond(DiamondParams::default()),
            PatternKind::Perlin => PatternParams::Perlin(PerlinParams::default()),
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
        // Title and introduction
        println!("\n{}", CliFormat::wrap(CliFormat::TITLE_1, "✨ ChromaCat Help ✨"));
        println!("{}", CliFormat::separator(&"═".repeat(90)));
        println!("\n{}", CliFormat::highlight_description(
            "ChromaCat is a command-line tool that adds beautiful color gradients to text output. \
            It supports various patterns, themes, and animated effects to make your terminal more colorful."
        ));

        // Patterns section
        println!("\n{}", CliFormat::core("Available Patterns:"));
        println!("{}", CliFormat::separator(&"─".repeat(85)));
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
            println!("  {} {}",
                CliFormat::param(&format!("{:<12}", pattern_kind)),
                CliFormat::description(params.description())
            );
        }
        println!("\n{}", CliFormat::general("Use --pattern-help for detailed pattern parameters"));

        Self::print_themes();
        Self::print_usage_examples();
    }

    fn print_themes() {
        println!("\n{}", CliFormat::core("🎨 Available Themes"));
        println!("{}", CliFormat::separator(&"─".repeat(85)));

        for category in themes::list_categories() {
            println!("\n  {}", CliFormat::param(&category));
            if let Some(theme_names) = themes::list_category(&category) {
                for name in theme_names {
                    if let Ok(theme) = themes::get_theme(&name) {
                        let preview = Self::create_theme_preview(&theme);
                        println!(
                            "    {} {} {}",
                            CliFormat::param_value(&format!("{:<15}", name)),
                            preview,
                            CliFormat::description(&theme.desc)
                        );
                    }
                }
            }
        }
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
        println!("\n{}", CliFormat::core("📚 Usage Examples"));
        println!("{}", CliFormat::separator(&"─".repeat(85)));

        let examples = [
            ("Basic file colorization:", "chromacat input.txt"),
            ("Using a specific theme:", "chromacat -t ocean input.txt"),
            ("Animated output:", "chromacat -a --fps 60 input.txt"),
            ("Pipe from another command:", "ls -la | chromacat -t neon"),
            ("Pattern with parameters:", "chromacat -p wave --param amplitude=1.5,frequency=2.0 input.txt"),
            ("Multiple files:", "chromacat -a *.txt"),
            ("Custom diagonal gradient:", "chromacat -p diagonal --param angle=45,speed=0.8 input.txt"),
            ("Interactive plasma:", "chromacat -p plasma --param complexity=3.0,scale=1.5 -a input.txt"),
        ];

        for (desc, cmd) in examples {
            println!("  {} {}",
                CliFormat::param(&format!("{:<25}", desc)),
                CliFormat::param_value(cmd)
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

        // Validate pattern-specific parameters
        if !self.pattern_params.params.is_empty() {
            let default_params = self.pattern.default_params();

            // Validate each parameter individually
            for param in &self.pattern_params.params {
                if let Err(e) = default_params.validate(param) {
                    return Err(ChromaCatError::PatternError {
                        pattern: self.pattern.to_string(),
                        param: "params".to_string(),
                        message: e,
                    });
                }
            }
        }

        // Validate aspect ratio
        self.validate_range("aspect-ratio", self.aspect_ratio, 0.1, 2.0)?;

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

    pub fn print_pattern_help() {
        // Title and introduction
        println!("\n{}", CliFormat::wrap(CliFormat::TITLE_1, "✨ ChromaCat Pattern Reference ✨"));
        println!("{}", CliFormat::separator(&"═".repeat(90)));
        println!("\n{}", CliFormat::highlight_description(
            "Each pattern supports specific parameters that can be customized using the --param flag. \
            Multiple parameters can be specified using comma separation: --param key1=value1,key2=value2"
        ));

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

            // Pattern header
            println!("\n{} {}",
                CliFormat::core(&format!("▶ {}", params.name())),
                CliFormat::description(params.description())
            );

            // Parameter table
            if !params.sub_params().is_empty() {
                println!("{}", CliFormat::separator(&"─".repeat(85)));
                println!("  {}  {}  {}",
                    CliFormat::param(&"Parameter".pad_to_width(20)),
                    CliFormat::param_value(&"Value Range".pad_to_width(20)),
                    CliFormat::param("Description")
                );
                println!("{}", CliFormat::separator(&"".repeat(85)));

                for param in params.sub_params() {
                    let range = match param.param_type() {
                        ParamType::Number { min, max } => format!("{} to {}", min, max),
                        ParamType::Boolean => "true/false".to_string(),
                        ParamType::Enum { options } => options.join(", "),
                        _ => String::new(),
                    };

                    println!("  {}  {}  {}",
                        CliFormat::param(&format!("{}=", param.name()).pad_to_width(20)),
                        CliFormat::param_value(&range.pad_to_width(20)),
                        CliFormat::description(param.description())
                    );
                }
            }

            // Example usage
            println!("\n  {} {}",
                CliFormat::param("Example:"),
                CliFormat::param_value(&format!("chromacat -p {} --param frequency=1.5 input.txt", pattern_kind))
            );
            println!("{}", CliFormat::separator(&"─".repeat(85)));
        }
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
