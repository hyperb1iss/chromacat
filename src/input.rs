use crate::demo::DemoGenerator;
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

    /// Creates a new InputReader in demo mode
    pub fn from_demo(_is_animated: bool) -> Result<Self> {
        let generator = DemoGenerator::new();
        Ok(Self {
            source: Box::new(DemoInput::new(generator)),
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

/// Demo mode input source
struct DemoInput {
    generator: DemoGenerator,
    buffer: Vec<u8>,
    position: usize,
}

impl DemoInput {
    fn new(generator: DemoGenerator) -> Self {
        Self {
            generator,
            buffer: Vec::new(),
            position: 0,
        }
    }
}

impl Read for DemoInput {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // Generate content if buffer is empty
        if self.buffer.is_empty() {
            let content = self.generator.generate();
            eprintln!("Generated content length: {}", content.len());
            self.buffer = content.into_bytes();
            self.position = 0;
        }

        // If we've read everything, return 0
        if self.position >= self.buffer.len() {
            return Ok(0);
        }

        // Copy data to the output buffer
        let available = self.buffer.len() - self.position;
        let to_copy = available.min(buf.len());
        buf[..to_copy].copy_from_slice(&self.buffer[self.position..self.position + to_copy]);
        self.position += to_copy;

        Ok(to_copy)
    }
}

impl BufRead for DemoInput {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        // Generate content if buffer is empty
        if self.buffer.is_empty() {
            let content = self.generator.generate();
            eprintln!("BufRead generated content length: {}", content.len());
            self.buffer = content.into_bytes();
            self.position = 0;
        }

        // Return remaining unread portion of the buffer
        Ok(&self.buffer[self.position..])
    }

    fn consume(&mut self, amt: usize) {
        self.position = (self.position + amt).min(self.buffer.len());
    }
}
