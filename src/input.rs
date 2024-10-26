use crate::error::Result;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

/// Handles reading input from either stdin or a file
pub struct InputHandler {
    source: Box<dyn BufRead>,
}

impl InputHandler {
    /// Creates a new InputHandler from a file path or stdin
    /// 
    /// If no path is provided, reads from stdin.
    /// If stdin is not connected to a terminal (i.e., being piped),
    /// reads from stdin. Otherwise, uses an empty reader.
    pub fn new<P: AsRef<Path>>(path: Option<P>) -> Result<Self> {
        let source: Box<dyn BufRead> = match path {
            Some(path) => {
                let file = File::open(path)?;
                Box::new(BufReader::new(file))
            }
            None => {
                if atty::is(atty::Stream::Stdin) {
                    // No input file and stdin is a terminal
                    Box::new(BufReader::new(io::empty()))
                } else {
                    // Stdin has content (being piped)
                    Box::new(BufReader::new(io::stdin()))
                }
            }
        };

        Ok(Self { source })
    }

    /// Returns a mutable reference to the underlying reader
    pub fn reader(&mut self) -> &mut dyn BufRead {
        &mut *self.source
    }
}