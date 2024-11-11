//! Demo art generation implementation
//!
//! This module contains the actual art generation logic for each demo pattern.
//! It handles creating the visual patterns with appropriate sizing and formatting.

use super::art::{ArtSettings, DemoArt};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::f64::consts::PI;

/// Generator for demo art patterns
pub struct DemoArtGenerator {
    /// Generator settings
    settings: ArtSettings,
    /// Random number generator
    rng: StdRng,
    /// Cached generated content
    generated: Option<String>,
}

impl DemoArtGenerator {
    /// Create a new demo art generator.
    pub fn new(settings: ArtSettings) -> Self {
        Self {
            rng: StdRng::seed_from_u64(settings.seed),
            settings,
            generated: None,
        }
    }

    /// Generate content for the specified art type.
    pub fn generate(&mut self, art: DemoArt) -> String {
        // Return cached content if available
        if let Some(content) = &self.generated {
            return content.clone();
        }

        let mut output = String::new();

        match art {
            DemoArt::All => {
                // Generate all patterns with headers
                for art_type in DemoArt::all_types() {
                    if self.settings.include_headers {
                        output.push_str(&format!(
                            "\n{:=^width$}\n\n",
                            format!(" {} ", art_type.display_name()),
                            width = self.settings.width as usize
                        ));
                    }
                    output.push_str(&self.generate_art(*art_type));
                    output.push_str("\n\n");
                }
                // Add final divider
                output.push_str(&"=".repeat(self.settings.width as usize));
                output.push_str("\n\n");
            }
            _ => {
                // Generate single pattern without header
                output.push_str(&self.generate_art(art));
            }
        }

        // Cache the generated content
        self.generated = Some(output.clone());
        output
    }

    /// Generate a specific art pattern.
    fn generate_art(&mut self, art: DemoArt) -> String {
        match art {
            DemoArt::Matrix => self.generate_matrix(),
            DemoArt::Waves => self.generate_waves(),
            DemoArt::Spiral => self.generate_spiral(),
            DemoArt::Code => self.generate_code(),
            DemoArt::Ascii => self.generate_ascii(),
            DemoArt::Boxes => self.generate_boxes(),
            DemoArt::Plasma => self.generate_plasma(),
            DemoArt::Vortex => self.generate_vortex(),
            DemoArt::Cells => self.generate_cells(),
            DemoArt::Fluid => self.generate_fluid(),
            DemoArt::Maze => self.generate_maze(),
            DemoArt::Mandala => self.generate_mandala(),
            DemoArt::Logo => self.generate_logo(),
            DemoArt::All => unreachable!(),
        }
    }

    /// Generate matrix digital rain effect.
    fn generate_matrix(&mut self) -> String {
        let mut output =
            String::with_capacity((self.settings.width * self.settings.height) as usize);
        let chars = "10 ";
        let chars: Vec<char> = chars.chars().collect();

        for _ in 0..self.settings.height {
            for _ in 0..self.settings.width {
                output.push(if self.rng.gen_bool(0.7) {
                    chars[self.rng.gen_range(0..2)]
                } else {
                    ' '
                });
            }
            output.push('\n');
        }

        output
    }

    /// Generate wave interference pattern.
    fn generate_waves(&mut self) -> String {
        let mut output =
            String::with_capacity((self.settings.width * self.settings.height) as usize);
        let wave_chars = ['█', '▓', '▒', '░', ' '];
        let char_count = wave_chars.len() - 1;

        // Multiple wave parameters for more organic feel
        let waves = [
            // (frequency_x, frequency_y, amplitude, speed)
            (0.07, 0.03, 0.5, 0.8), // Primary wave
            (0.05, 0.04, 0.3, 1.2), // Secondary wave
            (0.03, 0.06, 0.2, 0.6), // Background wave
        ];

        // Add time variation for animation-like effect
        let time_offset = self.rng.gen_range(0.0..2.0 * PI);

        for y in 0..self.settings.height {
            for x in 0..self.settings.width {
                let mut value = 0.0;

                // Combine multiple waves
                for (freq_x, freq_y, amplitude, speed) in waves.iter() {
                    let phase = x as f64 * freq_x + time_offset * speed;
                    let y_offset = y as f64 * freq_y;

                    // Create wave pattern with vertical displacement
                    let wave = (phase + y_offset).sin() * amplitude;
                    value += wave;
                }

                // Normalize to [0, 1] range
                value = (value + 1.5) / 3.0;
                value = value.clamp(0.0, 1.0);

                let idx = (value * char_count as f64) as usize;
                output.push(wave_chars[idx.min(char_count)]);
            }
            output.push('\n');
        }

        output
    }

    /// Generate spiral vortex pattern.
    fn generate_spiral(&mut self) -> String {
        let mut output =
            String::with_capacity((self.settings.width * self.settings.height) as usize);
        let chars = ['█', '▓', '▒', '░', ' '];
        let char_count = chars.len() - 1;

        let center_x = self.settings.width as f64 / 2.0;
        let center_y = self.settings.height as f64 / 2.0;

        for y in 0..self.settings.height {
            for x in 0..self.settings.width {
                let dx = x as f64 - center_x;
                let dy = (y as f64 - center_y) * 2.0; // Adjust for character aspect ratio
                let r = (dx * dx + dy * dy).sqrt();
                let theta = dy.atan2(dx);

                let spiral = (r * 0.15 - theta).sin();
                let value = (spiral + 1.0) / 2.0;
                let idx = (value * char_count as f64) as usize;
                output.push(chars[idx.min(char_count)]);
            }
            output.push('\n');
        }

        output
    }

    /// Generate styled code display.
    fn generate_code(&self) -> String {
        let code = [
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

        // Center the code box
        let mut output = String::new();
        let padding_top = (self.settings.height as usize - code.len()) / 2;

        // Add top padding
        for _ in 0..padding_top {
            output.push_str(&" ".repeat(self.settings.width as usize));
            output.push('\n');
        }

        // Add code with centering
        for line in code {
            let padding = (self.settings.width as usize - line.chars().count()) / 2;
            output.push_str(&" ".repeat(padding));
            output.push_str(line);
            output.push_str(
                &" ".repeat(self.settings.width as usize - padding - line.chars().count()),
            );
            output.push('\n');
        }

        // Fill remaining space
        while output.lines().count() < self.settings.height as usize {
            output.push_str(&" ".repeat(self.settings.width as usize));
            output.push('\n');
        }

        output
    }

    /// Generate ASCII art showcase.
    fn generate_ascii(&self) -> String {
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
            ┌─────────────────────────────────────────────────────────────┐
            │   ██████╗██╗  ██╗██████╗  ██████╗ ███╗   ███╗ █████╗  ██████╗█████╗ ████████╗  │
            │  ██╔════╝██║  ██║██╔══██╗██╔═══██╗████╗ ████║██╔══██╗██╔════╝██╔══██╗╚══██╔══╝ │
            │  ██║     ███████║██████╔╝██║   ██║██╔████╔██║███████║██║     ███████║   ██║    │
            │  ██║     ██╔══██║██╔══██╗██║   ██║██║╚██╔╝██║██╔══██║██║     ██╔══██║   ██║    │
            │  ╚██████╗██║  ██║██║  ██║╚██████╔╝██║ ╚═╝ ██║██║  ██║╚██████╗██║  ██║   ██║    │
            │   ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚═╝ ╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝   ╚═╝    │
            └──────────────────────────────────────────────────────────────┘"#,
            r#"
            ┌─────────────────────────────────────────────────────────────┐
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
            └─────────────────────────────────────────────────────────────┘"#,
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
                output.push_str(&" ".repeat(self.settings.width as usize));
                output.push('\n');
            }

            // Add the art with centering
            for line in art_lines {
                let line_len = line.chars().count().min(self.settings.width as usize);
                let side_padding = (self.settings.width as usize - line_len) / 2;

                output.push_str(&" ".repeat(side_padding));
                output.push_str(line);

                // Calculate remaining space safely
                let remaining =
                    (self.settings.width as usize).saturating_sub(side_padding + line_len);
                output.push_str(&" ".repeat(remaining));
                output.push('\n');
            }

            // Add bottom padding to complete the block
            let lines_so_far = output.lines().count() % block_size;
            if lines_so_far < block_size {
                let remaining_lines = block_size - lines_so_far;
                for _ in 0..remaining_lines {
                    output.push_str(&" ".repeat(self.settings.width as usize));
                    output.push('\n');
                }
            }

            // Add separator between arts (except for the last one)
            if i < arts.len() - 1 {
                output.push_str(&format!(
                    "{:=^width$}\n\n",
                    " Next Art ",
                    width = self.settings.width as usize
                ));
            }
        }

        output
    }

    /// Generate box drawing pattern.
    fn generate_boxes(&mut self) -> String {
        let mut output =
            String::with_capacity((self.settings.width * self.settings.height) as usize);

        for y in 0..self.settings.height {
            for x in 0..self.settings.width {
                let pattern_size = 6;
                let is_border = x % pattern_size == 0 || y % pattern_size == 0;
                let is_corner = x % pattern_size == 0 && y % pattern_size == 0;

                let ch = if is_corner {
                    // Create corner characters based on position
                    match ((y / pattern_size) % 2, (x / pattern_size) % 2) {
                        (0, 0) => '┌',
                        (0, _) => '┐',
                        (_, 0) => '└',
                        (_, _) => '┘',
                    }
                } else if is_border {
                    // Create border lines
                    if x % pattern_size == 0 {
                        '│'
                    } else {
                        '─'
                    }
                } else if ((x / pattern_size) + (y / pattern_size)) % 2 == 0 {
                    '█'
                } else {
                    ' '
                };

                output.push(ch);
            }
            output.push('\n');
        }

        output
    }

    /// Generate mandala pattern.
    fn generate_mandala(&mut self) -> String {
        let mut output =
            String::with_capacity((self.settings.width * self.settings.height) as usize);
        let chars = ['█', '▓', '▒', '░', ' '];
        let char_count = chars.len() - 1;

        let center_x = self.settings.width as f64 / 2.0;
        let center_y = self.settings.height as f64 / 2.0;

        for y in 0..self.settings.height {
            for x in 0..self.settings.width {
                let dx = x as f64 - center_x;
                let dy = (y as f64 - center_y) * 2.0;
                let distance = (dx * dx + dy * dy).sqrt() * 0.15;
                let angle = dy.atan2(dx) * 6.0;
                let value = (distance + angle).sin().abs();
                let idx = (value * char_count as f64) as usize;
                output.push(chars[idx.min(char_count)]);
            }
            output.push('\n');
        }

        output
    }

    /// Generate ChromaCat logo.
    fn generate_logo(&self) -> String {
        let logo = [
            r#"                              ,----,         ,----,                "#,
            r#"                            ,/   .`|       ,/   .`|                "#,
            r#"                          ,`   .'  :     ,`   .'  :     ,---,     "#,
            r#"                        ;    ;     /   ;    ;     /   .'  .' `\   "#,
            r#"                      .'___,/    ,' .'___,/    ,' ,---.'     \  "#,
            r#"                      |    :     |  |    :     |  |   |  '  |   "#,
            r#"                      ;    |.';  ;  ;    |.';  ;  :   : |  '  |   "#,
            r#"                      `----'  |  |  `----'  |  |  |   ' '  ;  :   "#,
            r#"                          '   :  ;      '   :  ;  '   | ;  .  |   "#,
            r#"                          |   |  '      |   |  '  |   | :  |  '   "#,
            r#"                          '   :  |      '   :  |  '   : | /  ;    "#,
            r#"                          ;   |.'       ;   |.'   |   | '` ,/     "#,
            r#"                          '---'         '---'     ;   :  .'       "#,
            r#"                                                  |   ,.'         "#,
            r#"               ✨ ChromaCat - Terminal Artistry ✨ '---'           "#,
        ];

        let mut output = String::new();
        let padding_top = (self.settings.height as usize - logo.len()) / 2;

        // Add top padding
        for _ in 0..padding_top {
            output.push_str(&" ".repeat(self.settings.width as usize));
            output.push('\n');
        }

        // Add logo with centering
        for line in logo {
            let padding = (self.settings.width as usize - line.chars().count()) / 2;
            output.push_str(&" ".repeat(padding));
            output.push_str(line);
            output.push_str(
                &" ".repeat(self.settings.width as usize - padding - line.chars().count()),
            );
            output.push('\n');
        }

        // Fill remaining space
        while output.lines().count() < self.settings.height as usize {
            output.push_str(&" ".repeat(self.settings.width as usize));
            output.push('\n');
        }

        output
    }

    /// Generate complex plasma effect with organic motion
    fn generate_plasma(&mut self) -> String {
        let mut output =
            String::with_capacity((self.settings.width * self.settings.height) as usize);
        let chars = ['█', '▓', '▒', '░', ' '];
        let char_count = chars.len() - 1;

        let freq_x = 0.1;
        let freq_y = 0.08;
        let freq_t = self.rng.gen_range(0.0..=1.0) * PI;

        for y in 0..self.settings.height {
            for x in 0..self.settings.width {
                let x_norm = x as f64 / self.settings.width as f64;
                let y_norm = y as f64 / self.settings.height as f64;

                // Generate multiple plasma layers
                let v1 = ((x_norm * freq_x + freq_t).sin() + (y_norm * freq_y).cos()) * 0.5;
                let v2 = ((x_norm * freq_y - freq_t).cos() + (y_norm * freq_x).sin()) * 0.5;
                let v3 = ((x_norm * y_norm * 4.0 + freq_t).sin()) * 0.25;

                let value = (v1 + v2 + v3 + 1.5) / 3.0;
                let idx = (value * char_count as f64) as usize;
                output.push(chars[idx.min(char_count)]);
            }
            output.push('\n');
        }

        output
    }

    /// Generate a mesmerizing vortex tunnel effect
    fn generate_vortex(&mut self) -> String {
        let mut output =
            String::with_capacity((self.settings.width * self.settings.height) as usize);
        let chars = ['█', '▓', '▒', '░', ' '];
        let char_count = chars.len() - 1;

        let center_x = self.settings.width as f64 / 2.0;
        let center_y = self.settings.height as f64 / 2.0;
        let time_offset = self.rng.gen_range(0.0..=2.0 * PI);

        for y in 0..self.settings.height {
            for x in 0..self.settings.width {
                let dx = (x as f64 - center_x) / center_x;
                let dy = (y as f64 - center_y) / center_y * 2.0; // Adjust for character aspect ratio

                let distance = (dx * dx + dy * dy).sqrt();
                let angle = dy.atan2(dx);

                // Create spinning vortex effect
                let spiral = (distance * 5.0 - angle * 2.0 + time_offset).sin();
                let tunnel = 1.0 / (distance + 0.5) * 0.5;

                let value = (spiral * tunnel + 1.0) / 2.0;
                let idx = (value * char_count as f64) as usize;
                output.push(chars[idx.min(char_count)]);
            }
            output.push('\n');
        }

        output
    }

    /// Generate a cellular automaton pattern
    fn generate_cells(&mut self) -> String {
        let mut output =
            String::with_capacity((self.settings.width * self.settings.height) as usize);
        let chars = ['█', '▓', '▒', '░', ' '];
        let char_count = chars.len() - 1;

        // Initialize random cells
        let mut grid =
            vec![vec![false; self.settings.width as usize]; self.settings.height as usize];
        for row in grid.iter_mut() {
            for cell in row.iter_mut() {
                *cell = self.rng.gen_bool(0.3);
            }
        }

        // Run cellular automaton rules
        let generations = 5;
        for _ in 0..generations {
            let mut new_grid = grid.clone();
            for y in 1..self.settings.height - 1 {
                for x in 1..self.settings.width - 1 {
                    let mut neighbors = 0;
                    // Convert coordinates to i32 for negative range
                    let y = y as i32;
                    let x = x as i32;

                    for dy in -1_i32..=1_i32 {
                        for dx in -1_i32..=1_i32 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let ny = (y + dy) as usize;
                            let nx = (x + dx) as usize;
                            if grid[ny][nx] {
                                neighbors += 1;
                            }
                        }
                    }
                    // Conway's Game of Life rules
                    new_grid[y as usize][x as usize] = matches!(
                        (grid[y as usize][x as usize], neighbors),
                        (true, 2) | (true, 3) | (false, 3)
                    );
                }
            }
            grid = new_grid;
        }

        // Render with smooth transitions
        for y in 0..self.settings.height {
            for x in 0..self.settings.width {
                let mut value = 0.0;
                let radius = 2;

                // Calculate smoothed value based on neighborhood
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        let ny = (y as i32 + dy).clamp(0, self.settings.height as i32 - 1) as usize;
                        let nx = (x as i32 + dx).clamp(0, self.settings.width as i32 - 1) as usize;
                        if grid[ny][nx] {
                            let dist = (dx * dx + dy * dy) as f64;
                            value += (-dist / 4.0).exp();
                        }
                    }
                }

                let idx = ((value * 0.5) * char_count as f64) as usize;
                output.push(chars[idx.min(char_count)]);
            }
            output.push('\n');
        }

        output
    }

    /// Generate a fluid simulation effect
    fn generate_fluid(&mut self) -> String {
        let mut output =
            String::with_capacity((self.settings.width * self.settings.height) as usize);
        let chars = ['█', '▓', '▒', '░', ' '];
        let char_count = chars.len() - 1;

        let time = self.rng.gen_range(0.0..=2.0 * PI);

        // Multiple frequency layers for more organic motion
        let frequencies = [
            (0.03, 0.02, 1.2), // Slow-moving large features
            (0.07, 0.05, 0.8), // Medium features
            (0.15, 0.11, 0.4), // Small detail features
        ];

        for y in 0..self.settings.height {
            for x in 0..self.settings.width {
                let mut value = 0.0;

                // Combine multiple noise layers with different frequencies
                for (freq_x, freq_y, amplitude) in frequencies.iter() {
                    let x_norm = x as f64 * freq_x;
                    let y_norm = y as f64 * freq_y;

                    // Create swirling motion
                    let swirl_x = x_norm + (y_norm * 0.5 + time).sin() * 0.5;
                    let swirl_y = y_norm - (x_norm * 0.5 + time).cos() * 0.5;

                    // Generate fluid-like patterns
                    let n1 = (swirl_x.sin() * swirl_y.cos()) * amplitude;
                    let n2 = ((swirl_x + time).cos() * (swirl_y - time).sin()) * amplitude;

                    // Add turbulence
                    let turbulence = ((swirl_x * swirl_y + time).sin() * 0.5) * amplitude;

                    value += n1 + n2 + turbulence;
                }

                // Normalize and add bias for better distribution
                value = (value + 2.0) / 4.0;
                value = value.clamp(0.0, 1.0);

                let idx = (value * char_count as f64) as usize;
                output.push(chars[idx.min(char_count)]);
            }
            output.push('\n');
        }

        output
    }

    /// Generate an intricate maze pattern using box-drawing characters
    fn generate_maze(&mut self) -> String {
        let mut canvas =
            vec![vec![' '; self.settings.width as usize]; self.settings.height as usize];
        let mut output =
            String::with_capacity((self.settings.width * self.settings.height) as usize);

        // Create a grid for maze generation (true = wall, false = path)
        let cell_width = (self.settings.width as usize - 4) / 2;
        let cell_height = (self.settings.height as usize - 4) / 2;
        let mut maze = vec![vec![true; cell_width]; cell_height];
        let mut visited = vec![vec![false; cell_width]; cell_height];

        // Start from top center for entrance
        let start_x = cell_width / 2;
        let start_y = 0;
        let mut stack = vec![(start_x, start_y)];
        visited[start_y][start_x] = true;
        maze[start_y][start_x] = false;

        // Track the longest path for exit placement
        let mut longest_path = vec![(start_x, start_y)];
        let mut current_path = vec![(start_x, start_y)];

        while let Some(&(current_x, current_y)) = stack.last() {
            // Get unvisited neighbors
            let mut neighbors = Vec::new();
            for &(dx, dy) in &[(0, -2), (2, 0), (0, 2), (-2, 0)] {
                let new_x = current_x as i32 + dx;
                let new_y = current_y as i32 + dy;

                if new_x >= 0
                    && new_x < cell_width as i32
                    && new_y >= 0
                    && new_y < cell_height as i32
                    && !visited[new_y as usize][new_x as usize]
                {
                    neighbors.push((new_x as usize, new_y as usize));
                }
            }

            if neighbors.is_empty() {
                stack.pop();
                if current_path.len() > longest_path.len() {
                    longest_path = current_path.clone();
                }
                current_path.pop();
            } else {
                // Choose random neighbor with bias towards straight paths
                let next_idx = if neighbors.len() > 1 && self.rng.gen_bool(0.7) {
                    // Try to continue in current direction if possible
                    let last_dx = if current_path.len() >= 2 {
                        (current_x as i32 - current_path[current_path.len() - 2].0 as i32).signum()
                    } else {
                        0
                    };
                    let last_dy = if current_path.len() >= 2 {
                        (current_y as i32 - current_path[current_path.len() - 2].1 as i32).signum()
                    } else {
                        0
                    };

                    neighbors
                        .iter()
                        .position(|&(nx, ny)| {
                            (nx as i32 - current_x as i32).signum() == last_dx
                                && (ny as i32 - current_y as i32).signum() == last_dy
                        })
                        .unwrap_or_else(|| self.rng.gen_range(0..neighbors.len()))
                } else {
                    self.rng.gen_range(0..neighbors.len())
                };

                let (next_x, next_y) = neighbors[next_idx];

                // Carve path to neighbor
                maze[next_y][next_x] = false;
                maze[(current_y + next_y) / 2][(current_x + next_x) / 2] = false;

                visited[next_y][next_x] = true;
                stack.push((next_x, next_y));
                current_path.push((next_x, next_y));
            }
        }

        // Place exit at the end of the longest path
        let exit_x = longest_path.last().unwrap().0; // Only get the x coordinate since we don't use y

        // Box drawing characters for different styles
        let styles = [
            // Regular weight
            ['─', '│', '┌', '┐', '└', '┘', '├', '┤', '┬', '┴', '┼'],
            // Double line
            ['═', '║', '╔', '╗', '╚', '╝', '╠', '╣', '╦', '╩', '╬'],
            // Heavy weight
            ['━', '┃', '┏', '┓', '┗', '┛', '┣', '┫', '┳', '┻', '╋'],
        ];

        // Choose random style
        let style = &styles[self.rng.gen_range(0..styles.len())];

        // Convert maze to box drawing characters
        for y in 0..cell_height {
            for x in 0..cell_width {
                if !maze[y][x] {
                    // For each path cell, determine appropriate box drawing character
                    let mut connections = 0u8;

                    // Check adjacent cells (up, right, down, left)
                    if y > 0 && !maze[y - 1][x] {
                        connections |= 0b0001;
                    }
                    if x < cell_width - 1 && !maze[y][x + 1] {
                        connections |= 0b0010;
                    }
                    if y < cell_height - 1 && !maze[y + 1][x] {
                        connections |= 0b0100;
                    }
                    if x > 0 && !maze[y][x - 1] {
                        connections |= 0b1000;
                    }

                    // Map connection pattern to appropriate box drawing character
                    let ch = match connections {
                        0b0000 => '█',                        // isolated cell becomes solid wall
                        0b0010 | 0b1000 | 0b1010 => style[0], // horizontal ─
                        0b0001 | 0b0100 | 0b0101 => style[1], // vertical │
                        0b0110 => style[2],                   // top-left corner ┌
                        0b0011 => style[3],                   // top-right corner ┐
                        0b1100 => style[4],                   // bottom-left corner └
                        0b1001 => style[5],                   // bottom-right corner ┘
                        0b0111 => style[6],                   // left T-junction ├
                        0b1011 => style[7],                   // right T-junction ┤
                        0b1101 => style[8],                   // top T-junction ┬
                        0b1110 => style[9],                   // bottom T-junction ┴
                        0b1111 => style[10],                  // crossroads ┼
                        _ => '█', // any unexpected pattern becomes solid wall
                    };

                    // Scale up to final size
                    let canvas_x = x * 2 + 2;
                    let canvas_y = y * 2 + 2;
                    canvas[canvas_y][canvas_x] = ch;

                    // Add connecting lines and fill walls
                    if connections & 0b0001 != 0 {
                        canvas[canvas_y - 1][canvas_x] = style[1]; // up
                    } else {
                        canvas[canvas_y - 1][canvas_x] = '█'; // wall
                    }
                    if connections & 0b0010 != 0 {
                        canvas[canvas_y][canvas_x + 1] = style[0]; // right
                    } else {
                        canvas[canvas_y][canvas_x + 1] = '█'; // wall
                    }
                    if connections & 0b0100 != 0 {
                        canvas[canvas_y + 1][canvas_x] = style[1]; // down
                    } else {
                        canvas[canvas_y + 1][canvas_x] = '█'; // wall
                    }
                    if connections & 0b1000 != 0 {
                        canvas[canvas_y][canvas_x - 1] = style[0]; // left
                    } else {
                        canvas[canvas_y][canvas_x - 1] = '█'; // wall
                    }

                    // Fill diagonal walls
                    if (connections & 0b0011) == 0 {
                        canvas[canvas_y - 1][canvas_x + 1] = '█';
                    } // top-right
                    if (connections & 0b1001) == 0 {
                        canvas[canvas_y - 1][canvas_x - 1] = '█';
                    } // top-left
                    if (connections & 0b0110) == 0 {
                        canvas[canvas_y + 1][canvas_x + 1] = '█';
                    } // bottom-right
                    if (connections & 0b1100) == 0 {
                        canvas[canvas_y + 1][canvas_x - 1] = '█';
                    } // bottom-left
                }
            }
        }

        // Fill the entire canvas with walls first
        for row in &mut canvas {
            for ch in row.iter_mut() {
                if *ch == ' ' {
                    *ch = '█';
                }
            }
        }

        // Add entrance and exit
        let entrance_x = start_x * 2 + 2;
        let exit_x = exit_x * 2 + 2;

        // Clear entrance path
        canvas[0][entrance_x] = style[1]; // vertical line for entrance
        canvas[1][entrance_x] = style[1]; // extend entrance
        canvas[0][entrance_x - 1] = '█'; // wall on left
        canvas[0][entrance_x + 1] = '█'; // wall on right
        canvas[1][entrance_x - 1] = '█'; // wall on left
        canvas[1][entrance_x + 1] = '█'; // wall on right

        // Clear exit path
        canvas[self.settings.height as usize - 2][exit_x] = style[1]; // vertical line for exit
        canvas[self.settings.height as usize - 1][exit_x] = style[1]; // extend exit
        canvas[self.settings.height as usize - 2][exit_x - 1] = '█'; // wall on left
        canvas[self.settings.height as usize - 2][exit_x + 1] = '█'; // wall on right
        canvas[self.settings.height as usize - 1][exit_x - 1] = '█'; // wall on left
        canvas[self.settings.height as usize - 1][exit_x + 1] = '█'; // wall on right

        // Add markers for entrance and exit
        canvas[0][entrance_x] = '▼'; // entrance marker
        canvas[self.settings.height as usize - 1][exit_x] = '▲'; // exit marker

        // Draw border with solid corners
        for x in 0..self.settings.width as usize {
            canvas[0][x] = style[0]; // top border
            canvas[self.settings.height as usize - 1][x] = style[0]; // bottom border
        }
        for row in canvas.iter_mut() {
            row[0] = style[1]; // left border
            row[self.settings.width as usize - 1] = style[1]; // right border
        }

        // Add proper corners
        canvas[0][0] = style[2]; // top-left ┌
        canvas[0][self.settings.width as usize - 1] = style[3]; // top-right ┐
        canvas[self.settings.height as usize - 1][0] = style[4]; // bottom-left └
        canvas[self.settings.height as usize - 1][self.settings.width as usize - 1] = style[5]; // bottom-right ┘

        // Convert canvas to string
        for row in canvas {
            for ch in row {
                output.push(ch);
            }
            output.push('\n');
        }

        output
    }
}
