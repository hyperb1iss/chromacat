//! ChromaCat application core
//!
//! This module provides the main application logic and coordinates all components
//! of ChromaCat. It handles initialization, input processing, and orchestrates
//! the pattern generation and rendering pipeline.

use crate::cli::Cli;
use crate::error::{ChromaCatError, Result};
use crate::input::InputReader;
use crate::pattern::PatternEngine;
use crate::playlist::{load_default_playlist, Playlist};
use crate::renderer::Renderer;
use crate::streaming::StreamingInput;
use crate::themes;

use crossterm::cursor::{Hide, Show};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use log::{debug, info};
use std::io::{stdout, Write};
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag to track if terminal is in alternate screen mode
/// Used by panic hook to restore terminal state
static TERMINAL_IN_RAW_MODE: AtomicBool = AtomicBool::new(false);
static TERMINAL_IN_ALTERNATE_SCREEN: AtomicBool = AtomicBool::new(false);

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

/// Install a panic hook that restores terminal state before printing panic info
fn install_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Restore terminal state BEFORE printing panic message
        // This ensures the user can see the panic message in a usable terminal
        let mut stdout = stdout();

        if TERMINAL_IN_ALTERNATE_SCREEN.load(Ordering::SeqCst) {
            let _ = execute!(stdout, Show, DisableMouseCapture, LeaveAlternateScreen);
            TERMINAL_IN_ALTERNATE_SCREEN.store(false, Ordering::SeqCst);
        }

        if TERMINAL_IN_RAW_MODE.load(Ordering::SeqCst) {
            let _ = disable_raw_mode();
            TERMINAL_IN_RAW_MODE.store(false, Ordering::SeqCst);
        }

        let _ = stdout.flush();

        // Now run the original panic handler to print the message
        original_hook(panic_info);
    }));
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
        let playground = self.should_use_playground();
        if playground {
            crate::debug_log::init_debug_log();
            crate::debug_log::debug_log("Playground mode started").ok();
        }

        debug!("Starting ChromaCat with configuration: {:?}", self.cli);

        // Handle --list-art flag
        if self.cli.list_art {
            Cli::print_art_patterns();
            return Ok(());
        }

        // Handle --list flag
        if self.cli.list_available {
            Cli::print_available_options();
            return Ok(());
        }

        // Validate CLI arguments
        self.cli.validate()?;

        // Initialize terminal
        self.setup_terminal()?;

        let playground = self.should_use_playground();

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
        info!("Creating renderer with config: {animation_config:?}");

        // Load playlist if enabled
        let playlist = if let Some(playlist_path) = &self.cli.playlist {
            match Playlist::from_file(playlist_path) {
                Ok(p) => {
                    info!(
                        "Loaded playlist from {} with {} entries",
                        playlist_path.display(),
                        p.entries.len()
                    );
                    if let Some(first) = p.entries.first() {
                        info!(
                            "First entry: pattern={}, theme={}, art={:?}",
                            first.pattern, first.theme, first.art
                        );
                    }
                    // Validate all entries
                    for (index, entry) in p.entries.iter().enumerate() {
                        if let Err(e) = entry.validate() {
                            return Err(ChromaCatError::Other(format!(
                                "Invalid playlist entry {} ({}): {}",
                                index + 1,
                                entry.name,
                                e
                            )));
                        }
                    }
                    Some(p)
                }
                Err(e) => {
                    return Err(ChromaCatError::Other(format!(
                        "Failed to load playlist from {}: {}",
                        playlist_path.display(),
                        e
                    )));
                }
            }
        } else if playground {
            // Try loading default playlist in animation mode
            match load_default_playlist()? {
                Some(p) => {
                    info!("Loaded default playlist");
                    Some(p)
                }
                None => {
                    info!("No default playlist found");
                    None
                }
            }
        } else {
            None
        };

        info!("Creating renderer with playlist: {}", playlist.is_some());
        let mut renderer = Renderer::new(
            engine,
            animation_config,
            playlist,
            &self.cli.theme,
            &self.cli.pattern,
        )?;

        // Process input and render
        // If playground, show overlay and status, and seed scenes
        if playground {
            crate::debug_log::debug_log("Setting up playground mode").ok();
            renderer.set_overlay_visible(true);
            renderer.set_status_message(
                "Playground: A=automix 1-5=modes ,/.=prev/next ;=overlay q=quit",
            );
            renderer.enable_default_scenes();

            // Load the art specified by CLI or use default
            // Don't override what was already loaded from InputReader::from_demo
            if self.cli.art.is_none() {
                crate::debug_log::debug_log("No art specified, setting rainbow").ok();
                // Only set default if no art was specified
                let _ = renderer.set_demo_art("rainbow");
            } else {
                crate::debug_log::debug_log(&format!("Using CLI art: {:?}", self.cli.art)).ok();
            }
        }

        // Process input - handle playground mode separately since it takes ownership
        let result = if playground {
            // For playground mode, hand over control to the renderer
            self.process_playground_mode(renderer)
        } else {
            // For non-playground mode, use the old approach
            self.process_input(&mut renderer)
        };

        // Always cleanup terminal, even if there was an error
        let cleanup_result = self.cleanup_terminal();

        // Return the original error if there was one, otherwise the cleanup error
        result.and(cleanup_result)
    }

    /// Processes playground mode by loading demo content and handing over to renderer
    fn process_playground_mode(&self, mut renderer: Renderer) -> Result<()> {
        // Configure renderer for playground mode
        renderer.set_overlay_visible(true);
        renderer
            .set_status_message("Playground: A=automix 1-5=modes ,/.=prev/next ;=overlay q=quit");
        renderer.enable_default_scenes();

        // Load demo content
        let content = {
            // Always load demo content in playground mode
            crate::debug_log::debug_log(&format!(
                "Loading demo content with art: {:?}",
                self.cli.art
            ))
            .ok();

            let mut reader =
                InputReader::from_demo(/*animate*/ true, self.cli.art.as_deref(), None)?;
            let mut buffer = String::new();
            reader.read_to_string(&mut buffer)?;
            buffer
        };

        // Load the art specified by CLI if any
        if self.cli.art.is_none() {
            crate::debug_log::debug_log("No art specified, using default demo content").ok();
        } else {
            crate::debug_log::debug_log(&format!("Using CLI art: {:?}", self.cli.art)).ok();
            // The art is already loaded via InputReader::from_demo above
        }

        crate::debug_log::debug_log("Handing control to renderer.run()").ok();

        // Hand over control to the renderer's event loop
        let result = renderer.run(content);
        result.map_err(|e| e.into())
    }

    /// Returns true if running in a test environment
    fn is_test() -> bool {
        std::env::var("RUST_TEST").is_ok()
    }

    /// Returns true if playground mode should be used
    ///
    /// Playground is the default mode unless:
    /// - Explicitly disabled via --no-playground
    /// - Running in test environment (unless CHROMACAT_TEST_PLAYGROUND is set)
    fn should_use_playground(&self) -> bool {
        !self.cli.no_playground
            && (!Self::is_test() || std::env::var("CHROMACAT_TEST_PLAYGROUND").is_ok())
    }

    /// Sets up the terminal for rendering
    fn setup_terminal(&mut self) -> Result<()> {
        // Install panic hook to restore terminal on panic
        install_panic_hook();

        // Get terminal size
        if Self::is_test() {
            // Use fixed size for tests
            self.term_size = (80, 24);
        } else {
            match crossterm::terminal::size() {
                Ok(size) => {
                    self.term_size = size;
                }
                Err(e) => {
                    return Err(ChromaCatError::Other(format!(
                        "Failed to get terminal size: {e}"
                    )));
                }
            }
        }

        // Skip terminal setup in test environment
        if Self::is_test() {
            return Ok(());
        }

        let playground = self.should_use_playground();
        // Setup terminal for playground mode, but only if we can
        if playground {
            // For playground mode, we need BOTH raw mode AND alternate screen for ratatui
            // If we can't get both, we should fall back to non-playground mode

            match enable_raw_mode() {
                Ok(()) => {
                    self.raw_mode = true;
                    TERMINAL_IN_RAW_MODE.store(true, Ordering::SeqCst);

                    // Now try alternate screen - ratatui NEEDS this
                    let mut stdout = stdout();

                    // Try to enter alternate screen
                    if let Err(_e) =
                        execute!(stdout, EnterAlternateScreen, Hide, EnableMouseCapture)
                    {
                        // Disable raw mode since we can't use it
                        let _ = disable_raw_mode();
                        self.raw_mode = false;
                        TERMINAL_IN_RAW_MODE.store(false, Ordering::SeqCst);

                        return Err(ChromaCatError::Other(
                            "Playground mode requires terminal features that are not available. \
                             Try running with --no-playground or in a different terminal."
                                .to_string(),
                        ));
                    }

                    self.alternate_screen = true;
                    TERMINAL_IN_ALTERNATE_SCREEN.store(true, Ordering::SeqCst);
                }
                Err(e) => {
                    return Err(ChromaCatError::Other(format!(
                        "Cannot initialize terminal for playground mode: {e}. \
                                Try running with --no-playground"
                    )));
                }
            }
        }

        Ok(())
    }

    /// Restores terminal state
    fn cleanup_terminal(&mut self) -> Result<()> {
        let mut stdout = stdout();

        if self.alternate_screen {
            crossterm::execute!(stdout, Show, DisableMouseCapture, LeaveAlternateScreen)?;
            self.alternate_screen = false;
            TERMINAL_IN_ALTERNATE_SCREEN.store(false, Ordering::SeqCst);
        }

        if self.raw_mode {
            disable_raw_mode()
                .map_err(|e| ChromaCatError::Other(format!("Failed to disable raw mode: {e}")))?;
            self.raw_mode = false;
            TERMINAL_IN_RAW_MODE.store(false, Ordering::SeqCst);
        }

        stdout.flush()?;
        Ok(())
    }

    /// Processes input from files or stdin
    fn process_input(&self, renderer: &mut Renderer) -> Result<()> {
        let playground = self.should_use_playground();

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

            if playground {
                self.run_playground(renderer, &buffer)?;
            } else {
                renderer.render_static(&buffer)?;
            }
        }

        Ok(())
    }

    /// Processes input from stdin
    fn process_stdin(&self, renderer: &mut Renderer) -> Result<()> {
        let playground = self.should_use_playground();

        if playground {
            // Always load demo content in playground mode, regardless of stdin type
            crate::debug_log::debug_log(&format!(
                "Loading demo content with art: {:?}",
                self.cli.art
            ))
            .ok();
            let mut reader =
                InputReader::from_demo(/*animate*/ true, self.cli.art.as_deref(), None)?;
            let mut buffer = String::new();
            reader.read_to_string(&mut buffer)?;
            crate::debug_log::debug_log(&format!("Loaded {} chars of content", buffer.len())).ok();

            self.run_playground(renderer, &buffer)?;
        } else if atty::is(atty::Stream::Stdin) {
            debug!("Processing stdin in terminal mode");
            // Terminal input - use normal processing
            let mut reader = InputReader::from_stdin()?;
            let mut buffer = String::new();
            reader.read_to_string(&mut buffer)?;

            renderer.render_static(&buffer)?
        } else {
            debug!("Processing stdin in streaming mode");
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
        info!("Streaming complete: processed {lines} lines ({bytes} bytes) at {rate:.2} lines/sec");

        result
    }

    /// Runs the playground UI loop
    fn run_playground(&self, renderer: &mut Renderer, content: &str) -> Result<()> {
        crate::debug_log::debug_log("run_playground started").ok();

        // For now, just render static to see if it works
        renderer.set_overlay_visible(true);
        let non_empty = if content.is_empty() { "\n" } else { content };
        renderer.render_static(non_empty).map_err(Into::into)
    }
}

impl Drop for ChromaCat {
    fn drop(&mut self) {
        // Attempt to cleanup terminal state if something went wrong
        if self.raw_mode || self.alternate_screen {
            if let Err(e) = self.cleanup_terminal() {
                eprintln!("Error cleaning up terminal: {e}");
            }
        }
    }
}
