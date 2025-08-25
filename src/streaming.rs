//! Streaming input processing for ChromaCat
//!
//! This module provides functionality for processing continuous input streams,
//! such as pipes or real-time logs, applying color patterns while maintaining
//! efficient throughput and memory usage.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crossterm::style::Color;
use log::{debug, trace};

use crate::error::{ChromaCatError, Result};
use crate::pattern::{PatternConfig, PatternEngine};
use crate::themes;

/// Default buffer capacity for streaming input
const DEFAULT_BUFFER_CAPACITY: usize = 8192;

/// Minimum sleep duration when no data is available (milliseconds)
const MIN_SLEEP_MS: u64 = 10;

/// Statistics for stream processing
#[derive(Debug, Default)]
struct StreamStats {
    /// Number of lines processed
    lines_processed: usize,
    /// Number of bytes processed
    bytes_processed: usize,
    /// Start time of processing
    start_time: Option<Instant>,
}

impl StreamStats {
    /// Starts tracking statistics
    fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// Updates statistics with processed data
    fn update(&mut self, bytes: usize) {
        self.lines_processed += 1;
        self.bytes_processed += bytes;
    }

    /// Returns the current lines per second
    fn lines_per_second(&self) -> f64 {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed().as_secs_f64();
            if elapsed > 0.0 {
                return self.lines_processed as f64 / elapsed;
            }
        }
        0.0
    }
}

/// Handles streaming input processing and colorization
pub struct StreamingInput {
    /// Pattern engine for color generation
    engine: PatternEngine,
    /// Whether colors are enabled
    colors_enabled: bool,
    /// Signal to stop processing
    stop_signal: Arc<AtomicBool>,
    /// Buffer capacity for reading
    buffer_capacity: usize,
    /// Processing statistics
    stats: StreamStats,
}

impl StreamingInput {
    /// Creates a new streaming input processor with the given configuration
    ///
    /// # Arguments
    /// * `config` - Pattern configuration for color generation
    /// * `theme_name` - Name of the color theme to use
    ///
    /// # Returns
    /// A new StreamingInput instance or an error if initialization fails
    pub fn new(config: PatternConfig, theme_name: &str) -> Result<Self> {
        debug!("Creating StreamingInput with theme: {theme_name}");

        let theme = themes::get_theme(theme_name)?;
        let gradient = theme.create_gradient()?;

        // Create pattern engine with default terminal size
        // Actual dimensions don't matter much for streaming since we process line by line
        let engine = PatternEngine::new(gradient, config, 80, 24);

        Ok(Self {
            engine,
            colors_enabled: true,
            stop_signal: Arc::new(AtomicBool::new(false)),
            buffer_capacity: DEFAULT_BUFFER_CAPACITY,
            stats: StreamStats::default(),
        })
    }

    /// Processes input from a reader and writes colored output
    ///
    /// # Arguments
    /// * `reader` - The input reader to process
    ///
    /// # Returns
    /// Ok(()) if processing completes successfully, Error otherwise
    pub fn process_stream<R: Read>(&mut self, reader: R) -> Result<()> {
        debug!("Starting stream processing");
        self.stats.start();

        let mut stdout = io::stdout();
        let buf_reader = BufReader::with_capacity(self.buffer_capacity, reader);

        for line in buf_reader.lines() {
            // Check stop signal
            if self.stop_signal.load(Ordering::Relaxed) {
                debug!("Stop signal received, ending stream processing");
                break;
            }

            let line = line?;
            self.process_line(&line, &mut stdout)?;

            trace!("Processed line: {} characters", line.len());
            self.stats.update(line.len());
        }

        debug!(
            "Stream processing complete. Processed {} lines at {:.2} lines/sec",
            self.stats.lines_processed,
            self.stats.lines_per_second()
        );

        Ok(())
    }

    /// Processes a single line of input
    ///
    /// # Arguments
    /// * `line` - The line to process
    /// * `writer` - The output writer
    ///
    /// # Returns
    /// Ok(()) if successful, Error otherwise
    fn process_line<W: Write>(&mut self, line: &str, writer: &mut W) -> Result<()> {
        // Trim any trailing whitespace/newlines
        let line = line.trim_end();

        // Skip empty lines
        if line.is_empty() {
            return Ok(());
        }

        if !self.colors_enabled {
            writeln!(writer, "{line}")?;
            return Ok(());
        }

        // Strip existing ANSI escape sequences
        let line = line
            .replace("\x1B[33m", "") // Remove yellow color
            .replace("\x1B[0m", "") // Remove reset
            .replace("#033[33m", "") // Remove yellow (alternate form)
            .replace("#033[0m", ""); // Remove reset (alternate form)

        // Generate colors for each character
        let mut current_color = None;

        for (x, ch) in line.chars().enumerate() {
            let pattern_value = self.engine.get_value_at(x, 0)?;
            let gradient_color = self.engine.gradient().at(pattern_value as f32);

            // Convert to RGB
            let color = Color::Rgb {
                r: (gradient_color.r * 255.0) as u8,
                g: (gradient_color.g * 255.0) as u8,
                b: (gradient_color.b * 255.0) as u8,
            };

            // Only output color code if it changed
            if current_color != Some(color) {
                match color {
                    Color::Rgb { r, g, b } => {
                        write!(writer, "\x1b[38;2;{r};{g};{b}m")?;
                    }
                    _ => unreachable!("We only use RGB colors"),
                }
                current_color = Some(color);
            }

            // Write character
            write!(writer, "{ch}")?;
        }

        // Reset color and add newline
        writeln!(writer, "\x1b[0m")?;
        writer.flush()?;

        // Advance pattern slightly for next line
        self.engine.update(0.1);

        Ok(())
    }

    /// Sets the buffer capacity for reading
    ///
    /// # Arguments
    /// * `capacity` - New buffer capacity in bytes
    pub fn set_buffer_capacity(&mut self, capacity: usize) {
        self.buffer_capacity = capacity;
    }

    /// Enables or disables color output
    ///
    /// # Arguments
    /// * `enabled` - Whether colors should be enabled
    pub fn set_colors_enabled(&mut self, enabled: bool) {
        self.colors_enabled = enabled;
    }

    /// Processes input from stdin with non-blocking reads
    ///
    /// # Returns
    /// Ok(()) if processing completes successfully, Error otherwise
    pub fn process_stdin(&mut self) -> Result<()> {
        debug!("Starting stdin processing");
        self.stats.start();

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        // Create non-blocking stdin reader
        let mut reader = BufReader::with_capacity(self.buffer_capacity, stdin);
        let mut buffer = String::with_capacity(self.buffer_capacity);

        loop {
            // Check stop signal
            if self.stop_signal.load(Ordering::Relaxed) {
                debug!("Stop signal received, ending stdin processing");
                break;
            }

            // Try to read a line
            match reader.read_line(&mut buffer) {
                Ok(0) => {
                    debug!("Reached EOF on stdin");
                    break; // EOF
                }
                Ok(n) => {
                    trace!("Read {n} bytes from stdin");
                    self.process_line(&buffer, &mut stdout)?;
                    self.stats.update(n);
                    buffer.clear();
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No data available, sleep briefly
                    thread::sleep(Duration::from_millis(MIN_SLEEP_MS));
                    continue;
                }
                Err(e) => {
                    debug!("Error reading from stdin: {e}");
                    return Err(ChromaCatError::IoError(e));
                }
            }
        }

        debug!(
            "Stdin processing complete. Processed {} lines at {:.2} lines/sec",
            self.stats.lines_processed,
            self.stats.lines_per_second()
        );

        Ok(())
    }

    /// Returns the current processing statistics
    pub fn stats(&self) -> (usize, usize, f64) {
        (
            self.stats.lines_processed,
            self.stats.bytes_processed,
            self.stats.lines_per_second(),
        )
    }

    /// Signals the processor to stop
    pub fn stop(&self) {
        debug!("Stop signal set");
        self.stop_signal.store(true, Ordering::Relaxed);
    }
}

impl Drop for StreamingInput {
    fn drop(&mut self) {
        // Ensure we stop processing when dropped
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::{CommonParams, HorizontalParams, PatternParams};
    use std::io::Cursor;

    fn create_test_config() -> PatternConfig {
        PatternConfig {
            common: CommonParams::default(),
            params: PatternParams::Horizontal(HorizontalParams::default()),
        }
    }

    #[test]
    fn test_streaming_basic() {
        let input = "Line 1\nLine 2\nLine 3\n";
        let reader = Cursor::new(input);

        let mut processor = StreamingInput::new(create_test_config(), "rainbow")
            .expect("Failed to create processor");

        // Disable colors for testing
        processor.set_colors_enabled(false);

        processor
            .process_stream(reader)
            .expect("Failed to process stream");

        let (lines, bytes, _) = processor.stats();
        assert_eq!(lines, 3);
        assert_eq!(bytes, input.len() - 3); // -3 for the newlines
    }

    #[test]
    fn test_streaming_empty() {
        let input = "";
        let reader = Cursor::new(input);

        let mut processor = StreamingInput::new(create_test_config(), "rainbow")
            .expect("Failed to create processor");

        processor
            .process_stream(reader)
            .expect("Failed to process stream");

        let (lines, bytes, _) = processor.stats();
        assert_eq!(lines, 0);
        assert_eq!(bytes, 0);
    }

    #[test]
    fn test_streaming_unicode() {
        let input = "Hello, 世界\n";
        let reader = Cursor::new(input);

        let mut processor = StreamingInput::new(create_test_config(), "rainbow")
            .expect("Failed to create processor");

        processor.set_colors_enabled(false);

        processor
            .process_stream(reader)
            .expect("Failed to process stream");

        let (lines, bytes, _) = processor.stats();
        assert_eq!(lines, 1);
        assert_eq!(bytes, input.len() - 1); // -1 for the newline
    }

    #[test]
    fn test_buffer_capacity() {
        let mut processor = StreamingInput::new(create_test_config(), "rainbow")
            .expect("Failed to create processor");

        processor.set_buffer_capacity(4096);
        assert_eq!(processor.buffer_capacity, 4096);
    }

    #[test]
    fn test_stop_signal() {
        let processor = StreamingInput::new(create_test_config(), "rainbow")
            .expect("Failed to create processor");

        assert!(!processor.stop_signal.load(Ordering::Relaxed));
        processor.stop();
        assert!(processor.stop_signal.load(Ordering::Relaxed));
    }
}
