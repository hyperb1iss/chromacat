use crate::error::Result;
use crate::gradient::{GradientConfig, GradientEngine};
use colorgrad::Gradient;
use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use unicode_segmentation::UnicodeSegmentation;

/// Handles the colorization of text using gradients
pub struct Colorizer {
    engine: GradientEngine,
    stdout: StandardStream,
    no_color: bool,
}

impl Colorizer {
    /// Creates a new Colorizer instance with the specified gradient and configuration
    pub fn new(gradient: Box<dyn Gradient + Send + Sync>, config: GradientConfig, no_color: bool) -> Self {
        let choice = if no_color {
            ColorChoice::Never
        } else {
            ColorChoice::Auto
        };

        Self {
            engine: GradientEngine::new(gradient, config),
            stdout: StandardStream::stdout(choice),
            no_color,
        }
    }

    /// Processes and colorizes text input
    pub fn colorize<R: io::BufRead>(&mut self, reader: R) -> Result<()> {
        let lines: Vec<String> = reader.lines().collect::<io::Result<Vec<String>>>()?;
        
        self.engine.set_total_lines(lines.len());

        for (i, line) in lines.iter().enumerate() {
            self.engine.set_current_line(i);
            self.render_line(line)?;
        }

        Ok(())
    }

    /// Renders a single line of text with the specified gradient
    fn render_line(&mut self, text: &str) -> Result<()> {
        let graphemes: Vec<&str> = text.graphemes(true).collect();
        let line_length = graphemes.len();

        for (i, grapheme) in graphemes.iter().enumerate() {
            if self.no_color {
                write!(self.stdout, "{}", grapheme)?;
                continue;
            }

            let gradient_color = self.engine.get_color_at(i, line_length)?;
            let color = Color::Rgb(
                (gradient_color.r * 255.0) as u8,
                (gradient_color.g * 255.0) as u8,
                (gradient_color.b * 255.0) as u8,
            );

            self.stdout.set_color(ColorSpec::new().set_fg(Some(color)))?;
            write!(self.stdout, "{}", grapheme)?;
        }

        writeln!(self.stdout)?;
        if !self.no_color {
            self.stdout.reset()?;
        }

        Ok(())
    }
}