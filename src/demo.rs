//! Demo mode text generator for ChromaCat
//!
//! This module provides creative text patterns for demonstrating ChromaCat's capabilities.

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

/// Generates demo text content
pub struct DemoGenerator {
    patterns: Vec<DemoPattern>,
    rng: rand::rngs::ThreadRng,
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
        }
    }

    /// Generates the next frame of demo content
    pub fn generate(&mut self) -> String {
        // Always generate full content, regardless of animation mode
        let mut output = String::with_capacity(50000);

        // Collect patterns into a vector first to avoid borrowing issues
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

    fn generate_matrix(&mut self) -> String {
        let mut output = String::with_capacity(2000);
        let chars = "10 ";
        let chars: Vec<char> = chars.chars().collect();

        // Fill entire width with denser matrix
        for _ in 0..24 {
            for _ in 0..80 {
                if self.rng.gen_bool(0.7) {
                    // Increased density
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

        // Create smoother wave pattern
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

        // New approach using polar coordinates
        for y in 0..24 {
            for x in 0..80 {
                let center_x = 40.0;
                let center_y = 12.0;
                let dx = x as f64 - center_x;
                let dy = (y as f64 - center_y) * 2.0; // Adjust for terminal aspect ratio

                // Convert to polar coordinates
                let r = (dx * dx + dy * dy).sqrt();
                let theta = dy.atan2(dx);

                // Create spiral pattern
                let spiral = (r * 0.15 - theta).sin();
                let value = (spiral + 1.0) / 2.0; // Normalize to 0.0-1.0 range

                let idx = (value * char_count as f64) as usize;
                output.push(spiral_chars[idx.min(char_count)]);
            }
            output.push('\n');
        }

        // Add debug output to check dimensions
        for (i, line) in output.lines().enumerate() {
            let len = line.chars().count();
            if len != 80 {
                eprintln!("Line {} has incorrect length: {}", i, len);
            }
        }

        output
    }

    fn generate_boxes(&mut self) -> String {
        let mut output = String::with_capacity(2000);

        // Create a more structured box pattern
        for y in 0..24 {
            for x in 0..80 {
                let pattern_size = 6; // Smaller boxes
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
            "└───────────────────────────────────┘",
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

            // Calculate lines needed for this block
            let lines_needed: usize = block_size;
            let art_height: usize = art_lines.len();

            // Calculate top padding (ensure it's non-negative)
            let top_padding = lines_needed.saturating_sub(art_height) / 2;

            // Add top padding
            for _ in 0..top_padding {
                output.push_str(&" ".repeat(80));
                output.push('\n');
            }

            // Add the art with centering
            for line in art_lines {
                let line_len: usize = line.chars().count();
                let side_padding = if line_len < 80 {
                    (80 - line_len) / 2
                } else {
                    0
                };

                output.push_str(&" ".repeat(side_padding));
                output.push_str(line);

                // Add remaining padding to reach 80 chars
                let remaining = if side_padding + line_len < 80 {
                    80 - (side_padding + line_len)
                } else {
                    0
                };
                output.push_str(&" ".repeat(remaining));
                output.push('\n');
            }

            // Calculate and add bottom padding
            let current_height = output.lines().count() % block_size;
            if current_height < block_size {
                let bottom_padding = block_size - current_height;
                for _ in 0..bottom_padding {
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

    /// Check if the generator is in animated mode
    pub fn is_animated(&self) -> bool {
        true
    }
}

impl Default for DemoGenerator {
    fn default() -> Self {
        Self::new()
    }
}
