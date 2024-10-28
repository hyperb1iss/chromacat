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
use crate::streaming::StreamingInput;
use crate::themes;

use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use log::{debug, info};
use std::io::{stdout, Write};
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

        // Load custom theme file if specified
        if let Some(theme_file) = &self.cli.theme_file {
            themes::load_theme_file(theme_file)?;
        }

        // Create theme and gradient
        info!("Creating theme and gradient");
        let theme = themes::get_theme(&self.cli.theme)?;
        let gradient = theme.create_gradient()?;

        // Create pattern configuration
        info!("Creating pattern configuration");
        let pattern_config = self.cli.create_pattern_config()?;

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
        if Self::is_test() {
            // Use fixed size for tests
            self.term_size = (80, 24);
        } else {
            self.term_size = crossterm::terminal::size().map_err(|e| {
                ChromaCatError::Other(format!("Failed to get terminal size: {}", e))
            })?;
        }

        // Skip terminal setup in test environment
        if Self::is_test() {
            return Ok(());
        }

        if self.cli.animate {
            // Enter raw mode for animation
            enable_raw_mode()
                .map_err(|e| ChromaCatError::Other(format!("Failed to enable raw mode: {}", e)))?;
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
            disable_raw_mode()
                .map_err(|e| ChromaCatError::Other(format!("Failed to disable raw mode: {}", e)))?;
            self.raw_mode = false;
        }

        stdout.flush()?;
        Ok(())
    }

    /// Processes input from files or stdin
    fn process_input(&self, renderer: &mut Renderer) -> Result<()> {
        // Handle demo mode
        if self.cli.demo {
            info!("Running in demo mode");
            let mut reader = InputReader::from_demo(self.cli.animate)?;
            
            if self.cli.animate {
                // For animated demo, we'll keep generating new content
                let mut buffer = String::new();
                reader.read_to_string(&mut buffer)?;
                self.run_animation(renderer, &buffer)?;
            } else {
                // For static demo, read all generated content
                let mut buffer = String::new();
                reader.read_to_string(&mut buffer)?;
                renderer.render_static(&buffer)?;
            }
            return Ok(());
        }

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
        // Check if stdin is a terminal or a pipe
        if atty::is(atty::Stream::Stdin) {
            debug!("Processing stdin in terminal mode");
            // Terminal input - use normal processing
            let mut reader = InputReader::from_stdin()?;
            let mut buffer = String::new();
            reader.read_to_string(&mut buffer)?;

            if self.cli.animate {
                self.run_animation(renderer, &buffer)?;
            } else {
                renderer.render_static(&buffer)?;
            }
        } else {
            debug!("Processing stdin in streaming mode");
            if self.cli.animate {
                return Err(ChromaCatError::Other(
                    "Animation mode is not supported for streaming input. Please use static mode for pipes and real-time logs.".to_string()
                ));
            }
            // Streaming input - use streaming processor
            self.process_streaming()?;
        }

        Ok(())
    }

    /// Processes streaming input (e.g., from pipes)
    fn process_streaming(&self) -> Result<()> {
        info!("Starting streaming input processing");
        let pattern_config = self.cli.create_pattern_config()?;

        // Create streaming processor
        let mut processor = StreamingInput::new(pattern_config, &self.cli.theme)?;

        // Set color state
        processor.set_colors_enabled(!self.cli.no_color);

        // Set custom buffer size if specified
        if let Some(buffer_size) = self.cli.buffer_size {
            processor.set_buffer_capacity(buffer_size);
        }

        // Process stdin
        let result = processor.process_stdin();

        // Log processing statistics
        let (lines, bytes, rate) = processor.stats();
        info!(
            "Streaming complete: processed {} lines ({} bytes) at {:.2} lines/sec",
            lines, bytes, rate
        );

        result
    }

    /// Runs the animation loop
    fn run_animation(&self, renderer: &mut Renderer, content: &str) -> Result<()> {
        let frame_duration = renderer.frame_duration();
        let mut last_frame = Instant::now();
        let mut paused = false;
        let start_time = Instant::now();

        // Skip terminal setup and animation loop in test environment
        if Self::is_test() {
            renderer.render_frame(content, 0.016)?;
            return Ok(());
        }

        // Set up terminal
        enable_raw_mode()?;

        // Main animation loop
        'main: loop {
            // Add duration check
            if self.cli.duration > 0 && start_time.elapsed() >= Duration::from_secs(self.cli.duration) {
                break 'main;
            }

            // Handle input with minimal polling delay
            if event::poll(Duration::from_millis(1))? {
                match event::read()? {
                    Event::Key(key) => {
                        use crossterm::event::KeyCode;
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => break 'main,
                            KeyCode::Char(' ') => paused = !paused,
                            _ => {
                                match renderer.handle_key_event(key) {
                                    Ok(true) => continue 'main,
                                    Ok(false) => break 'main,
                                    Err(e) => {
                                        eprintln!("Key handling error: {}", e);
                                        continue 'main;
                                    }
                                }
                            }
                        }
                    }
                    Event::Resize(width, height) => {
                        if let Err(e) = renderer.handle_resize(width, height) {
                            eprintln!("Resize error: {}", e);
                        }
                        continue 'main;
                    }
                    _ => continue 'main,
                }
            }

            let now = Instant::now();

            // Update and render frame
            if !paused && now.duration_since(last_frame) >= frame_duration {
                let delta_seconds = now.duration_since(last_frame).as_secs_f64();

                if let Err(e) = renderer.render_frame(content, delta_seconds) {
                    eprintln!("Render error: {}", e);
                    continue 'main;
                }

                last_frame = now;
            } else {
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
