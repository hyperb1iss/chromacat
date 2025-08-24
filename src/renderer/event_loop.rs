use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, MouseEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
/// Event loop for the renderer - pure ratatui implementation
/// Handles the main rendering loop with proper event handling
use std::io;
use std::time::{Duration, Instant};

use crate::debug_log::debug_log;
use crate::renderer::{Renderer, RendererError};

/// Main event loop for the renderer
pub struct EventLoop {
    /// The renderer instance
    renderer: Renderer,

    /// Target frame rate
    target_fps: u32,

    /// Last frame time for delta calculation
    last_frame: Instant,
}

impl EventLoop {
    /// Create a new event loop
    pub fn new(renderer: Renderer, target_fps: u32) -> Self {
        Self {
            renderer,
            target_fps,
            last_frame: Instant::now(),
        }
    }

    /// Run the main event loop
    pub fn run(mut self) -> Result<(), RendererError> {
        // Setup terminal
        enable_raw_mode()
            .map_err(|e| RendererError::Other(format!("Failed to enable raw mode: {}", e)))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).map_err(|e| {
            RendererError::Other(format!("Failed to enter alternate screen: {}", e))
        })?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)
            .map_err(|e| RendererError::Other(format!("Failed to create terminal: {}", e)))?;

        // Clear the terminal
        terminal
            .clear()
            .map_err(|e| RendererError::Other(format!("Failed to clear terminal: {}", e)))?;

        let frame_duration = Duration::from_secs_f64(1.0 / self.target_fps as f64);

        loop {
            let now = Instant::now();
            let delta = now.duration_since(self.last_frame).as_secs_f64();
            self.last_frame = now;

            // Check for events (non-blocking with timeout)
            if event::poll(Duration::from_millis(1))
                .map_err(|e| RendererError::Other(format!("Event poll failed: {}", e)))?
            {
                match event::read()
                    .map_err(|e| RendererError::Other(format!("Event read failed: {}", e)))?
                {
                    Event::Key(key) => {
                        if !self.handle_key(key)? {
                            break; // Exit requested
                        }
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse(mouse)?;
                    }
                    Event::Resize(width, height) => {
                        let _ = debug_log(&format!("Terminal resized to {}x{}", width, height));
                        self.renderer.handle_resize(width, height)?;
                    }
                    _ => {}
                }
            }

            // Render frame
            self.renderer.render(delta)?;

            // Frame rate limiting
            let elapsed = Instant::now().duration_since(now);
            if let Some(sleep_duration) = frame_duration.checked_sub(elapsed) {
                std::thread::sleep(sleep_duration);
            }
        }

        // Cleanup terminal
        disable_raw_mode()
            .map_err(|e| RendererError::Other(format!("Failed to disable raw mode: {}", e)))?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(|e| {
            RendererError::Other(format!("Failed to leave alternate screen: {}", e))
        })?;
        terminal
            .show_cursor()
            .map_err(|e| RendererError::Other(format!("Failed to show cursor: {}", e)))?;

        Ok(())
    }

    /// Handle keyboard events
    fn handle_key(&mut self, key: KeyEvent) -> Result<bool, RendererError> {
        // Check for quit
        if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
            return Ok(false); // Signal exit
        }

        // Pass to renderer
        self.renderer.handle_key(key)
    }

    /// Handle mouse events
    fn handle_mouse(&mut self, mouse: MouseEvent) -> Result<(), RendererError> {
        self.renderer.handle_mouse(mouse)?;
        Ok(())
    }
}
