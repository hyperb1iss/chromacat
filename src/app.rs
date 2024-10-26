use crate::cli::Cli;
use crate::colorizer::Colorizer;
use crate::error::Result;
use crate::gradient::GradientConfig;
use crate::input::InputHandler;
use crate::themes::Theme;
use std::str::FromStr;

/// Main application struct that coordinates all ChromaCat functionality
pub struct ChromaCat {
    cli: Cli,
}

impl ChromaCat {
    /// Creates a new ChromaCat instance
    pub fn new(cli: Cli) -> Self {
        Self { cli }
    }

    /// Runs the ChromaCat application
    pub fn run(&self) -> Result<()> {
        // Validate CLI arguments
        self.cli.validate()?;

        // Set up the gradient
        let theme = Theme::from_str(&self.cli.theme).unwrap_or(Theme::Rainbow);
        let gradient = theme.create_gradient()?;

        // Configure the gradient engine
        let config = GradientConfig {
            diagonal: self.cli.diagonal,
            angle: self.cli.angle,
            cycle: self.cli.cycle,
        };

        // Set up the colorizer with gradient configuration
        let mut colorizer = Colorizer::new(gradient, config, self.cli.no_color);

        // Process input
        let mut input = InputHandler::new(self.cli.input.as_ref())?;
        colorizer.colorize(input.reader())?;

        Ok(())
    }
}