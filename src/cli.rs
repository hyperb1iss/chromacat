//! Command-line interface for ChromaCat
//!
//! This module provides the command-line argument parsing and validation for ChromaCat.
//! It handles all user input configuration and converts it into the internal configuration
//! types used by the pattern engine and renderer.

use crate::cli_format::{CliFormat, PadToWidth};
use crate::demo::DemoArt;
use crate::error::{ChromaCatError, Result};
use crate::pattern::{CommonParams, ParamType, PatternConfig, REGISTRY};
use crate::renderer::AnimationConfig;
use crate::themes;

use clap::Parser;
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
        default_value = "diagonal",
        env = "CHROMACAT_PATTERN",
        help_heading = CliFormat::HEADING_CORE,
        value_name = "TYPE",
        help = CliFormat::highlight_description("Select pattern type for the color gradient")
    )]
    pub pattern: String,

    #[arg(
        short = 't',
        long,
        default_value = "rainbow",
        env = "CHROMACAT_THEME",
        help_heading = CliFormat::HEADING_CORE,
        value_name = "NAME",
        help = CliFormat::highlight_description("Select color theme (use --list to see available)")
    )]
    pub theme: String,

    #[arg(
        short = 'f',
        long,
        default_value = "1.0",
        env = "CHROMACAT_FREQUENCY",
        help_heading = CliFormat::HEADING_CORE,
        value_name = "NUM",
        help = CliFormat::highlight_description("Base frequency (0.1-10.0)")
    )]
    pub frequency: f64,

    #[arg(
        short = 'm',
        long,
        default_value = "1.0",
        env = "CHROMACAT_AMPLITUDE",
        help_heading = CliFormat::HEADING_CORE,
        value_name = "NUM",
        help = CliFormat::highlight_description("Pattern amplitude (0.1-2.0)")
    )]
    pub amplitude: f64,

    #[arg(
        short = 'n',
        long = "no-color",
        env = "NO_COLOR",
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

    #[arg(
        long = "param",
        value_name = "KEY=VALUE",
        help_heading = CliFormat::HEADING_CORE,
        help = CliFormat::highlight_description("Pattern-specific parameter (can be used multiple times)")
    )]
    pub params: Vec<String>,

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

    // Demo mode removed - playground is now the default
    #[arg(
        long,
        value_name = "FILE",
        help_heading = CliFormat::HEADING_PLAYLIST,
        help = CliFormat::highlight_description("Load and play a sequence of patterns (uses default if not specified in animation mode)")
    )]
    pub playlist: Option<PathBuf>,

    /// Demo art pattern to display
    #[arg(
        long = "art",
        value_name = "TYPE",
        env = "CHROMACAT_ART",
        help_heading = CliFormat::HEADING_DEMO,
        help = CliFormat::highlight_description("Select demo art pattern to display")
    )]
    pub art: Option<String>,

    /// List available demo art patterns
    #[arg(
        long = "list-art",
        help_heading = CliFormat::HEADING_DEMO,
        help = CliFormat::highlight_description("Show available art patterns")
    )]
    pub list_art: bool,

    /// Disable playground UI mode (enabled by default for interactive terminals)
    #[arg(
        long = "no-playground",
        env = "CHROMACAT_NO_PLAYGROUND",
        help_heading = CliFormat::HEADING_GENERAL,
        help = CliFormat::highlight_description("Disable playground UI for simple output")
    )]
    pub no_playground: bool,

    /// Enable debug logging to /tmp/chromacat_debug.log
    #[arg(
        long = "debug",
        env = "CHROMACAT_DEBUG",
        help_heading = CliFormat::HEADING_GENERAL,
        help = CliFormat::highlight_description("Enable debug logging for troubleshooting")
    )]
    pub debug: bool,
}

impl Cli {
    /// Creates pattern configuration from CLI arguments
    pub fn create_pattern_config(&self) -> Result<PatternConfig> {
        let common = CommonParams {
            frequency: self.frequency,
            amplitude: self.amplitude,
            speed: 1.0,
            correct_aspect: !self.no_aspect_correction,
            aspect_ratio: self.aspect_ratio,
            theme_name: Some(self.theme.clone()),
        };

        // Get pattern params from registry
        let pattern_params = if self.params.is_empty() {
            // Use default parameters
            REGISTRY
                .create_pattern_params(&self.pattern)
                .ok_or_else(|| ChromaCatError::PatternError {
                    pattern: self.pattern.clone(),
                    param: String::new(),
                    message: "Unknown pattern type".to_string(),
                })?
        } else {
            // Parse provided parameters
            let params_str = self.params.join(",");
            REGISTRY
                .parse_params(&self.pattern, &params_str)
                .map_err(|e| ChromaCatError::PatternError {
                    pattern: self.pattern.clone(),
                    param: "params".to_string(),
                    message: e,
                })?
        };

        Ok(PatternConfig {
            common,
            params: pattern_params,
        })
    }

    /// Creates animation configuration with default values for playground mode
    pub fn create_animation_config(&self) -> AnimationConfig {
        AnimationConfig {
            fps: 30,
            cycle_duration: Duration::from_secs(u64::MAX),
            infinite: true,
            show_progress: true,
            smooth: false,
        }
    }

    /// Validates the CLI arguments
    pub fn validate(&self) -> Result<()> {
        // Skip validation if just listing options
        if self.list_available {
            Self::print_available_options();
            std::process::exit(0);
        }

        // Handle --list-art flag
        if self.list_art {
            Self::print_art_patterns();
            std::process::exit(0);
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

        // Validate pattern exists and its parameters
        if !self.params.is_empty() {
            let params_str = self.params.join(",");
            REGISTRY
                .validate_params(&self.pattern, &params_str)
                .map_err(|e| ChromaCatError::PatternError {
                    pattern: self.pattern.clone(),
                    param: "params".to_string(),
                    message: e,
                })?;
        }

        // Validate aspect ratio
        self.validate_range("aspect-ratio", self.aspect_ratio, 0.1, 2.0)?;

        // Warn about demo mode overriding playlist
        // Playlist is handled properly now

        // Validate art selection if specified
        if let Some(art) = &self.art {
            if DemoArt::try_from_str(art).is_none() {
                return Err(ChromaCatError::InputError(format!(
                    "Invalid art type '{art}'. Use --list-art to see available options."
                )));
            }
        }

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

    /// Prints available themes and patterns
    pub fn print_available_options() {
        // Title and introduction
        println!(
            "\n{}",
            CliFormat::wrap(CliFormat::TITLE_1, "✨ ChromaCat Help ✨")
        );
        println!("{}", CliFormat::separator(&"═".repeat(90)));
        println!("\n{}", CliFormat::highlight_description(
            "ChromaCat is a command-line tool that adds beautiful color gradients to text output. \
            It supports various patterns, themes, and animated effects to make your terminal more colorful."
        ));

        // Patterns section
        println!("\n{}", CliFormat::core("Available Patterns:"));
        println!("{}", CliFormat::separator(&"─".repeat(85)));

        for pattern_id in REGISTRY.list_patterns() {
            if let Some(metadata) = REGISTRY.get_pattern(pattern_id) {
                println!(
                    "  {} {}",
                    CliFormat::param(&format!("{:<12}", metadata.name)),
                    CliFormat::description(metadata.description)
                );
            }
        }

        println!(
            "\n{}",
            CliFormat::general("Use --pattern-help for detailed pattern parameters")
        );

        Self::print_themes();
        Self::print_usage_examples();
    }

    pub fn print_pattern_help() {
        // Title and introduction
        println!(
            "\n{}",
            CliFormat::wrap(CliFormat::TITLE_1, "✨ ChromaCat Pattern Reference ✨")
        );
        println!("{}", CliFormat::separator(&"═".repeat(90)));
        println!("\n{}", CliFormat::highlight_description(
            "Each pattern supports specific parameters that can be customized using the --param flag. \
            Multiple parameters can be specified using comma separation: --param key1=value1,key2=value2"
        ));

        for pattern_id in REGISTRY.list_patterns() {
            if let Some(metadata) = REGISTRY.get_pattern(pattern_id) {
                // Pattern header
                println!(
                    "\n{} {}",
                    CliFormat::core(&format!("▶ {}", metadata.name)),
                    CliFormat::description(metadata.description)
                );

                // Parameter table
                let params = metadata.params().sub_params();
                if !params.is_empty() {
                    println!("{}", CliFormat::separator(&"─".repeat(85)));
                    println!(
                        "  {}  {}  {}",
                        CliFormat::param(&"Parameter".pad_to_width(20)),
                        CliFormat::param_value(&"Value Range".pad_to_width(20)),
                        CliFormat::param("Description")
                    );
                    println!("{}", CliFormat::separator(&"".repeat(85)));

                    for param in params {
                        let range = match param.param_type() {
                            ParamType::Number { min, max } => format!("{min} to {max}"),
                            ParamType::Boolean => "true/false".to_string(),
                            ParamType::Enum { options } => options.join(", "),
                            _ => String::new(),
                        };

                        println!(
                            "  {}  {}  {}",
                            CliFormat::param(&format!("{}=", param.name()).pad_to_width(20)),
                            CliFormat::param_value(&range.pad_to_width(20)),
                            CliFormat::description(param.description())
                        );
                    }
                }

                // Example usage
                println!(
                    "\n  {} {}",
                    CliFormat::param("Example:"),
                    CliFormat::param_value(&format!(
                        "chromacat -p {pattern_id} --param frequency=1.5 input.txt"
                    ))
                );
                println!("{}", CliFormat::separator(&"─".repeat(85)));
            }
        }
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
                            CliFormat::param_value(&format!("{name:<15}")),
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
                preview.push_str(&format!("\x1b[48;2;{r};{g};{b}m \x1b[0m"));
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
            ("Playground mode:", "chromacat input.txt"),
            ("Pipe from another command:", "ls -la | chromacat -t neon"),
            (
                "Pattern with parameters:",
                "chromacat -p wave --param amplitude=1.5,frequency=2.0 input.txt",
            ),
            ("Multiple files:", "chromacat *.txt"),
            (
                "Custom diagonal gradient:",
                "chromacat -p diagonal --param angle=45,speed=0.8 input.txt",
            ),
            (
                "Interactive plasma:",
                "chromacat -p plasma --param complexity=3.0,scale=1.5 input.txt",
            ),
        ];

        for (desc, cmd) in examples {
            println!(
                "  {} {}",
                CliFormat::param(&format!("{desc:<25}")),
                CliFormat::param_value(cmd)
            );
        }

        println!("\n{}", CliFormat::core("🎵 Playlist Examples"));
        println!("{}", CliFormat::separator(&"─".repeat(85)));

        let playlist_examples = [
            ("Play default playlist:", "chromacat"),
            (
                "Use custom playlist:",
                "chromacat --playlist my-playlist.yaml",
            ),
        ];

        for (desc, cmd) in playlist_examples {
            println!(
                "  {} {}",
                CliFormat::param(&format!("{desc:<25}")),
                CliFormat::param_value(cmd)
            );
        }
    }

    /// Print available demo art patterns
    pub fn print_art_patterns() {
        println!(
            "\n{}",
            CliFormat::wrap(CliFormat::TITLE_1, "✨ ChromaCat Demo Art ✨")
        );
        println!("{}", CliFormat::separator(&"═".repeat(90)));
        println!("\n{}", CliFormat::highlight_description(
            "ChromaCat's demo art patterns showcase different artistic effects and capabilities.\n\
             Use these patterns with --demo mode to create ambient displays and visualizations."
        ));

        println!("\n{}", CliFormat::core("Available Patterns:"));
        println!("{}", CliFormat::separator(&"─".repeat(85)));

        for art in DemoArt::all_types() {
            println!(
                "  {} {} - {}",
                CliFormat::param(&format!("{:<12}", art.as_str())),
                CliFormat::param_value(art.display_name()),
                CliFormat::description(art.description())
            );
        }

        println!("\n{}", CliFormat::param("Special Values:"));
        println!(
            "  {} {} - {}",
            CliFormat::param(&format!("{:<12}", "all")),
            CliFormat::param_value("All Patterns"),
            CliFormat::description("Show all patterns in sequence")
        );

        println!("\n{}", CliFormat::general("Examples:"));
        println!(
            "  {} {}",
            CliFormat::param("Basic demo:"),
            CliFormat::description("chromacat --demo")
        );
        println!(
            "  {} {}",
            CliFormat::param("Specific art:"),
            CliFormat::description("chromacat --demo --art matrix")
        );
        println!(
            "  {} {}",
            CliFormat::param("With playlist:"),
            CliFormat::description("chromacat --demo --playlist my-playlist.yaml")
        );
    }
}
