// File: src/demo.rs
//! Demo mode text generator for ChromaCat
//!
//! This module provides creative text patterns for demonstrating ChromaCat's capabilities
//! by generating a comprehensive display of various pattern types. Content is generated
//! once and cached for efficiency.

use rand::{thread_rng, Rng};

/// Different types of demo patterns
#[derive(Debug, Clone, Copy)]
enum DemoPattern {
    Matrix,
    Waves,
    Spiral,
    Code,
    Ascii,
    Boxes,
    Mandala,
}

/// Generates and caches demo text content
pub struct DemoGenerator {
    /// Available pattern types
    patterns: Vec<DemoPattern>,
    /// Random number generator
    rng: rand::rngs::ThreadRng,
    /// Cached generated content
    generated_content: Option<String>,
}

impl DemoGenerator {
    /// Creates a new demo generator
    pub fn new() -> Self {
        Self {
            patterns: vec![
                DemoPattern::Matrix,
                DemoPattern::Waves,
                DemoPattern::Spiral,
                DemoPattern::Code,
                DemoPattern::Mandala,
                DemoPattern::Ascii,
                DemoPattern::Boxes,
            ],
            rng: thread_rng(),
            generated_content: None,
        }
    }

    /// Generates or returns cached demo content
    pub fn generate(&mut self) -> String {
        // Return cached content if available
        if let Some(content) = &self.generated_content {
            log::debug!("Returning cached demo content");
            return content.clone();
        }

        log::info!("Generating demo content for the first time");

        // Generate content once
        let mut output = String::with_capacity(50000);
        let patterns: Vec<DemoPattern> = self.patterns.clone();

        // Generate each pattern once
        for pattern in patterns.iter() {
            // Add a header for the pattern
            output.push_str(&format!(
                "\n{:=^80}\n",
                format!(" {} ", format!("{:?}", pattern))
            ));
            output.push('\n');

            // Generate the pattern content
            output.push_str(&self.generate_pattern(*pattern));
            output.push_str("\n\n");
        }

        // Add a final divider
        output.push_str(&"=".repeat(80));
        output.push_str("\n\n");

        // Cache the generated content
        self.generated_content = Some(output.clone());
        output
    }

    fn generate_matrix(&mut self) -> String {
        let mut output = String::with_capacity(2000);
        let chars = "10 ";
        let chars: Vec<char> = chars.chars().collect();

        // Fill entire width with denser matrix
        for _ in 0..24 {
            for _ in 0..80 {
                if self.rng.gen_bool(0.7) {
                    output.push(chars[self.rng.gen_range(0..2)]);
                } else {
                    output.push(' ');
                }
            }
            output.push('\n');
        }

        output
    }

    fn generate_waves(&mut self) -> String {
        let mut output = String::with_capacity(2000);
        let wave_chars: Vec<char> = "█▓▒░ ".chars().collect();
        let char_count = wave_chars.len() - 1;

        for y in 0..24 {
            for x in 0..80 {
                let phase = (x as f64 * 0.2 + y as f64 * 0.1).sin();
                let second = (x as f64 * 0.1 - y as f64 * 0.15).cos();
                let value = (phase + second) * 0.5 + 0.5;
                let idx = (value * char_count as f64) as usize;
                output.push(wave_chars[idx.min(char_count)]);
            }
            output.push('\n');
        }

        output
    }

    fn generate_spiral(&mut self) -> String {
        let mut output = String::with_capacity(2000);
        let spiral_chars: Vec<char> = "█▓▒░ ".chars().collect();
        let char_count = spiral_chars.len() - 1;

        for y in 0..24 {
            for x in 0..80 {
                let center_x = 40.0;
                let center_y = 12.0;
                let dx = x as f64 - center_x;
                let dy = (y as f64 - center_y) * 2.0;

                let r = (dx * dx + dy * dy).sqrt();
                let theta = dy.atan2(dx);

                let spiral = (r * 0.15 - theta).sin();
                let value = (spiral + 1.0) / 2.0;

                let idx = (value * char_count as f64) as usize;
                output.push(spiral_chars[idx.min(char_count)]);
            }
            output.push('\n');
        }

        output
    }

    fn generate_boxes(&mut self) -> String {
        let mut output = String::with_capacity(2000);

        for y in 0..24 {
            for x in 0..80 {
                let pattern_size = 6;
                let is_border = x % pattern_size == 0 || y % pattern_size == 0;
                let is_corner = x % pattern_size == 0 && y % pattern_size == 0;

                if is_corner {
                    let corner_type = match (y / pattern_size % 2, x / pattern_size % 2) {
                        (0, 0) => '┌',
                        (0, _) => '┐',
                        (_, 0) => '└',
                        (_, _) => '┘',
                    };
                    output.push(corner_type);
                } else if is_border {
                    output.push(if x % pattern_size == 0 { '│' } else { '─' });
                } else {
                    let fill = if (x / pattern_size + y / pattern_size) % 2 == 0 {
                        '█'
                    } else {
                        ' '
                    };
                    output.push(fill);
                }
            }
            output.push('\n');
        }

        output
    }

    fn generate_mandala(&mut self) -> String {
        let mut output = String::with_capacity(2000);
        let mandala_chars: Vec<char> = "█▓▒░ ".chars().collect();
        let char_count = mandala_chars.len() - 1;

        for y in 0..24 {
            for x in 0..80 {
                let center_x = 40.0;
                let center_y = 12.0;
                let dx = x as f64 - center_x;
                let dy = (y as f64 - center_y) * 2.0;
                let distance = (dx * dx + dy * dy).sqrt() * 0.15;
                let angle = dy.atan2(dx) * 6.0;
                let value = (distance + angle).sin().abs();
                let idx = (value * char_count as f64) as usize;
                output.push(mandala_chars[idx.min(char_count)]);
            }
            output.push('\n');
        }

        output
    }

    fn generate_code(&mut self) -> String {
        let code_snippets = [
            "┌─ ChromaCat Demo ────────────────────┐",
            "│                                     │",
            "│  fn main() {                       │",
            "│      let cat = ChromaCat::new();   │",
            "│      cat.run(Pattern::Rainbow)     │",
            "│         .with_colors(vec![         │",
            "│             \"#FF0000\",           │",
            "│             \"#00FF00\",           │",
            "│             \"#0000FF\",           │",
            "│         ])                         │",
            "         .animate()                 │",
            "│  }                                 │",
            "│                                    │",
            "│  // Create beautiful gradients     │",
            "│  // for your terminal output!      │",
            "│                                    │",
            "└────────────────────────────┘",
        ];

        // Center and pad the code box
        let mut output = String::new();
        let padding_top = (24 - code_snippets.len()) / 2;

        // Add top padding
        for _ in 0..padding_top {
            output.push_str(&" ".repeat(80));
            output.push('\n');
        }

        // Add code with centering
        for line in code_snippets {
            let padding = (80 - line.chars().count()) / 2;
            output.push_str(&" ".repeat(padding));
            output.push_str(line);
            output.push_str(&" ".repeat(80 - padding - line.chars().count()));
            output.push('\n');
        }

        // Fill remaining space
        while output.lines().count() < 24 {
            output.push_str(&" ".repeat(80));
            output.push('\n');
        }

        output
    }

    fn generate_ascii_art(&mut self) -> String {
        let arts = [
            r#"
    ╔══════════════════════════════════════════════════════════════╗
    ║                      Welcome to ChromaCat                     ║
    ║                                                              ║
    ║                         /\___/\                              ║
    ║                        (  o o  )                             ║
    ║                        (  =^=  )                             ║
    ║                         (______)                             ║
    ║                                                              ║
    ║                  Create Magical Color Gradients              ║
    ║                     For Your Terminal Text                   ║
    ║                                                              ║
    ╚══════════════════════════════════════════════════════════════╝"#,
            r#"
        ┌──────────────────────────────────────────────────────────────┐
        │   ██████╗██╗  ██╗██████╗  ██████╗ ███╗   ███╗ █████╗  ██████╗█████╗ ████████╗  │
        │  ██╔════╝██║  ██║██╔══██╗██╔═══██╗████╗ ████║██╔══██╗██╔════╝██╔══██╗╚══██╔══╝ │
        │  ██║     ███████║██████╔╝██║   ██║██╔████╔██║███████║██║     ███████║   ██║    │
        │  ██║     ██╔══██║██╔══██╗██║   ██║██║╚██╔╝██║██╔══██║██║     ██╔══██║   ██║    │
        │  ╚██████╗██║  ██║██║  ██║╚██████╔╝██║ ╚═╝ ██║██║  ██║╚██████╗██║  ██║   ██║    │
        │   ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝   ╚═╝    │
        └──────────────────────────────────────────────────────────────┘"#,
            r#"
        ┌──────────────────────────────────────────────────────────────┐
        │                     🎨 Terminal Artistry 🎨                   │
        │                                                              │
        │              ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░             │
        │              ░░  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  ░░             │
        │              ░░  ▒▒  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓  ▒▒  ░░             │
        │              ░░  ▒▒  ▓▓  ████████  ▓▓  ▒▒  ░░             │
        │              ░░  ▒▒  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓  ▒▒  ░░             │
        │              ░░  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  ░░             │
        │              ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░             │
        │                                                              │
        │                    Paint Your Terminal World                 │
        └──────────────────────────────────────────────────────────────┘"#,
        ];

        let mut output = String::new();
        let block_size: usize = 24; // Height of each block

        for (i, art) in arts.iter().enumerate() {
            // Get non-empty lines
            let art_lines: Vec<&str> = art.lines().filter(|line| !line.trim().is_empty()).collect();
            let art_height = art_lines.len();

            // Calculate padding while avoiding overflow
            let padding = if art_height < block_size {
                (block_size - art_height) / 2
            } else {
                0
            };

            // Add top padding
            for _ in 0..padding {
                output.push_str(&" ".repeat(80));
                output.push('\n');
            }

            // Add the art with centering
            for line in art_lines {
                let line_len = line.chars().count().min(80) as u32;
                let side_padding = ((80 - line_len) / 2) as u32;

                output.push_str(&" ".repeat(side_padding as usize));
                output.push_str(line);

                // Calculate remaining space safely with explicit types
                let remaining = (80_u32).saturating_sub(side_padding + line_len) as usize;
                output.push_str(&" ".repeat(remaining));
                output.push('\n');
            }

            // Add bottom padding to complete the block
            let lines_so_far = output.lines().count() % block_size;
            if lines_so_far < block_size {
                let remaining_lines = block_size - lines_so_far;
                for _ in 0..remaining_lines {
                    output.push_str(&" ".repeat(80));
                    output.push('\n');
                }
            }

            // Add separator between arts (except for the last one)
            if i < arts.len() - 1 {
                output.push_str(&format!("{:=^80}\n", " Next Art "));
                output.push('\n');
            }
        }

        output
    }

    fn generate_pattern(&mut self, pattern: DemoPattern) -> String {
        match pattern {
            DemoPattern::Matrix => self.generate_matrix(),
            DemoPattern::Waves => self.generate_waves(),
            DemoPattern::Spiral => self.generate_spiral(),
            DemoPattern::Code => self.generate_code(),
            DemoPattern::Ascii => self.generate_ascii_art(),
            DemoPattern::Boxes => self.generate_boxes(),
            DemoPattern::Mandala => self.generate_mandala(),
        }
    }
}

impl Default for DemoGenerator {
    fn default() -> Self {
        Self::new()
    }
}
