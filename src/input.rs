use crate::error::Result;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

/// Handles reading input from either stdin or a file
pub struct InputReader {
    source: Box<dyn BufRead>,
}

impl InputReader {
    /// Creates a new InputReader from a file path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        Ok(Self {
            source: Box::new(BufReader::new(file)),
        })
    }

    /// Creates a new InputReader from stdin
    pub fn from_stdin() -> Result<Self> {
        Ok(Self {
            source: Box::new(BufReader::new(io::stdin())),
        })
    }

    /// Returns a mutable reference to the underlying reader
    pub fn reader(&mut self) -> &mut dyn BufRead {
        &mut *self.source
    }

    /// Reads all content into a String
    pub fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
        self.source.read_to_string(buf).map_err(Into::into)
    }

    /// Returns an iterator over the lines of this reader
    pub fn lines(self) -> impl Iterator<Item = Result<String>> {
        self.source.lines().map(|line| line.map_err(Into::into))
    }
}
