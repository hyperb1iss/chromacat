use crate::error::{ChromaCatError, Result};
use crate::themes::Theme;
use clap::Parser;
use std::path::PathBuf;

/// ChromaCat - A versatile command-line tool for applying color gradients to text output
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Select a theme for the gradient (use --list-themes to see all options)
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
    pub fn theme_descriptions() -> Vec<(String, &'static str)> {
        Theme::list_all()
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