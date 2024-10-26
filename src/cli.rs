use crate::error::{ChromaCatError, Result};
use clap::Parser;
use std::path::PathBuf;

/// ChromaCat - A versatile command-line tool for applying color gradients to text output
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Select a built-in theme for the gradient (rainbow, heat, ocean, forest, pastel, neon, autumn)
    #[arg(short, long, default_value = "rainbow")]
    pub theme: String,

    /// Enable infinite cycling of gradient colors
    #[arg(short, long)]
    pub cycle: bool,

    /// Input file to read text from (reads from stdin if not provided)
    #[arg(short, long)]
    pub input: Option<PathBuf>,

    /// Disable colored output
    #[arg(short = 'n', long = "no-color")]
    pub no_color: bool,

    /// Enable diagonal gradient mode
    #[arg(short, long)]
    pub diagonal: bool,

    /// Adjust the angle of the diagonal gradient in degrees (0-360)
    #[arg(short, long, default_value = "45")]
    pub angle: i32,

    /// Display available themes and exit
    #[arg(long = "list-themes")]
    pub list_themes: bool,
}

impl Cli {
    /// Get a list of available theme descriptions
    pub fn theme_descriptions() -> Vec<(&'static str, &'static str)> {
        vec![
            ("rainbow", "Classic rainbow colors (red, orange, yellow, green, blue, indigo, violet)"),
            ("heat", "Warm colors transitioning from red through orange to yellow"),
            ("ocean", "Cool blue tones reminiscent of ocean depths"),
            ("forest", "Natural green tones inspired by forests"),
            ("pastel", "Soft, muted colors for a gentle appearance"),
            ("neon", "Bright, vibrant colors that pop"),
            ("autumn", "Warm fall colors (browns, oranges, and red-oranges)"),
        ]
    }

    /// Validates the command line arguments
    pub fn validate(&self) -> Result<()> {
        // Skip validation if we're just listing themes
        if self.list_themes {
            return Ok(());
        }

        // Validate angle range
        if self.angle < 0 || self.angle > 360 {
            return Err(ChromaCatError::InvalidAngle(self.angle));
        }

        // Validate input file existence if specified
        if let Some(path) = &self.input {
            if !path.exists() {
                return Err(ChromaCatError::InputError(
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("Input file not found: {}", path.display()),
                    ),
                ));
            }
        }

        Ok(())
    }
}