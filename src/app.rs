//! ChromaCat application core
//!
//! This module provides the main application logic and coordinates all components
//! of ChromaCat. It handles initialization, input processing, and orchestrates
//! the pattern generation and rendering pipeline.

use crate::cli::Cli;
use crate::error::{ChromaCatError, Result};
use crate::input::InputReader;
use crate::pattern::PatternEngine;
use crate::renderer::Renderer;
use crate::themes::Theme;

use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use log::{debug, info};
use std::io::{stdout, Write};
use std::str::FromStr;
use std::time::{Duration, Instant};

/// Main application struct that coordinates ChromaCat functionality
pub struct ChromaCat {
    /// Command line interface configuration
    cli: Cli,
    /// Terminal dimensions (width, height)
    term_size: (u16, u16),
    /// Whether the application is in raw mode
    raw_mode: bool,
    /// Whether we're using the alternate screen
    alternate_screen: bool,
}

impl ChromaCat {
    /// Creates a new ChromaCat instance with the given CLI configuration
    pub fn new(cli: Cli) -> Self {
        Self {
            cli,
            term_size: (0, 0),
            raw_mode: false,
            alternate_screen: false,
        }
    }

    /// Runs the ChromaCat application
    pub fn run(&mut self) -> Result<()> {
        debug!("Starting ChromaCat with configuration: {:?}", self.cli);

        // Handle --list flag
        if self.cli.list_available {
            Cli::print_available_options();
            return Ok(());
        }

        // Validate CLI arguments
        self.cli.validate()?;

        // Initialize terminal
        self.setup_terminal()?;

        // Create pattern configuration
        info!("Creating pattern configuration");
        let pattern_config = self.cli.create_pattern_config()?;

        // Create theme and gradient
        info!("Creating theme and gradient");
        let theme = Theme::from_str(&self.cli.theme)?;
        let gradient = theme.create_gradient()?;

        info!("Initializing pattern engine");
        let engine = PatternEngine::new(
            gradient,
            pattern_config,
            self.term_size.0 as usize,
            self.term_size.1 as usize,
        );

        // Set up the renderer
        let animation_config = self.cli.create_animation_config();
        info!("Creating renderer with config: {:?}", animation_config);
        let mut renderer = Renderer::new(engine, animation_config)?;

        // Process input and render
        let result = self.process_input(&mut renderer);

        // Cleanup terminal
        self.cleanup_terminal()?;

        result
    }

    /// Returns true if running in a test environment
    fn is_test() -> bool {
        std::env::var("RUST_TEST").is_ok()
    }

    /// Sets up the terminal for rendering
    fn setup_terminal(&mut self) -> Result<()> {
        // Get terminal size
        self.term_size = crossterm::terminal::size()
            .map_err(|e| ChromaCatError::TerminalError(e.to_string()))?;

        // Skip terminal setup in test environment
        if Self::is_test() {
            return Ok(());
        }

        if self.cli.animate {
            // Enter raw mode for animation
            enable_raw_mode().map_err(|e| ChromaCatError::TerminalError(e.to_string()))?;
            self.raw_mode = true;

            // Enter alternate screen
            execute!(stdout(), EnterAlternateScreen, Hide)?;
            self.alternate_screen = true;
        }

        Ok(())
    }

    /// Restores terminal state
    fn cleanup_terminal(&mut self) -> Result<()> {
        let mut stdout = stdout();

        if self.alternate_screen {
            execute!(stdout, Show, LeaveAlternateScreen)?;
            self.alternate_screen = false;
        }

        if self.raw_mode {
            disable_raw_mode().map_err(|e| ChromaCatError::TerminalError(e.to_string()))?;
            self.raw_mode = false;
        }

        stdout.flush()?;
        Ok(())
    }

    /// Processes input from files or stdin
    fn process_input(&self, renderer: &mut Renderer) -> Result<()> {
        // If no files specified, read from stdin
        if self.cli.files.is_empty() {
            info!("No input files specified, reading from stdin");
            self.process_stdin(renderer)?;
            return Ok(());
        }

        // Process each input file
        for file in &self.cli.files {
            info!("Processing file: {}", file.display());
            let mut reader = InputReader::from_file(file)?;
            let mut buffer = String::new();
            reader.read_to_string(&mut buffer)?;

            if self.cli.animate {
                self.run_animation(renderer, &buffer)?;
            } else {
                renderer.render_static(&buffer)?;
            }
        }

        Ok(())
    }

    /// Processes input from stdin
    fn process_stdin(&self, renderer: &mut Renderer) -> Result<()> {
        let mut reader = InputReader::from_stdin()?;
        let mut buffer = String::new();
        reader.read_to_string(&mut buffer)?;

        if self.cli.animate {
            self.run_animation(renderer, &buffer)?;
        } else {
            renderer.render_static(&buffer)?;
        }

        Ok(())
    }

    /// Runs the animation loop
    fn run_animation(&self, renderer: &mut Renderer, content: &str) -> Result<()> {
        let start_time = Instant::now();
        let frame_duration = renderer.frame_duration();
        let mut last_frame = Instant::now();
        let paused = false;

        // Skip terminal setup and animation loop in test environment
        if Self::is_test() {
            // Just render one frame for testing
            renderer.render_frame(content, Duration::from_millis(100))?;
            return Ok(());
        }

        // Set up terminal
        enable_raw_mode()?;

        // Main animation loop
        'main: loop {
            // Handle input with minimal polling delay
            if event::poll(Duration::from_millis(1))? {
                match event::read()? {
                    Event::Key(key) => {
                        match renderer.handle_key_event(key) {
                            Ok(true) => continue 'main, // Continue running
                            Ok(false) => break 'main,   // Exit requested
                            Err(e) => {
                                // Handle error but continue running
                                eprintln!("Key handling error: {}", e);
                                continue 'main;
                            }
                        }
                    }
                    Event::Resize(width, height) => {
                        // Update terminal size if needed
                        // For now, we just continue
                        continue 'main;
                    }
                    _ => continue 'main,
                }
            }

            let now = Instant::now();

            // Update and render frame
            if !paused && now.duration_since(last_frame) >= frame_duration {
                let elapsed = now.duration_since(start_time);

                // Render frame, handle potential errors
                if let Err(e) = renderer.render_frame(content, elapsed) {
                    eprintln!("Render error: {}", e);
                    // Optionally break or continue based on error severity
                    continue 'main;
                }

                last_frame = now;
            } else {
                // Avoid busy-waiting
                std::thread::sleep(Duration::from_millis(1));
            }
        }

        // Clean up terminal
        disable_raw_mode()?;

        Ok(())
    }
}

impl Drop for ChromaCat {
    fn drop(&mut self) {
        // Attempt to cleanup terminal state if something went wrong
        if self.raw_mode || self.alternate_screen {
            if let Err(e) = self.cleanup_terminal() {
                eprintln!("Error cleaning up terminal: {}", e);
            }
        }
    }
}
