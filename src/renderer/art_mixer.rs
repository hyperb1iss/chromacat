/// Art mixing system for smooth transitions between ASCII art
///
/// This module provides smooth blending between different ASCII art pieces
/// to create seamless transitions instead of jarring cuts.
use std::collections::HashMap;

/// Art mixing strategies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MixStrategy {
    /// Simple character-by-character blend
    CharacterBlend,
    /// Fade through empty space
    SpaceFade,
    /// Dissolve effect with random pixels
    Dissolve,
    /// Line-by-line wipe
    LineWipe,
    /// Morph between similar characters
    CharacterMorph,
}

/// Character similarity groups for morphing
struct CharacterGroups {
    groups: HashMap<char, Vec<char>>,
}

impl CharacterGroups {
    fn new() -> Self {
        let mut groups = HashMap::new();

        // Define character similarity groups for smooth morphing
        // Dots and periods
        groups.insert('.', vec![',', '·', '•', '°', 'o', 'O', '0']);
        groups.insert('·', vec!['.', '•', '°', '*']);

        // Lines and dashes
        groups.insert('-', vec!['_', '=', '—', '─', '━']);
        groups.insert('|', vec!['!', 'I', 'l', '1', '│', '┃']);
        groups.insert('/', vec!['\\', '╱', '╲']);
        groups.insert('\\', vec!['/', '╱', '╲']);

        // Blocks and squares
        groups.insert('#', vec!['█', '▓', '▒', '░', '@', '%']);
        groups.insert('█', vec!['▓', '▒', '░', '#']);
        groups.insert('▓', vec!['█', '▒', '░', '#']);
        groups.insert('▒', vec!['▓', '░', '#']);
        groups.insert('░', vec!['▒', ' ']);

        // Stars and asterisks
        groups.insert('*', vec!['+', 'x', '×', '✕', '✖', '★', '☆']);
        groups.insert('+', vec!['*', 'x', '×', '†']);

        // Waves and curves
        groups.insert('~', vec!['≈', '∼', '〜', '～']);
        groups.insert('(', vec!['[', '{', '⟨', '〈']);
        groups.insert(')', vec![']', '}', '⟩', '〉']);

        // Letters to similar shapes
        groups.insert('O', vec!['0', 'o', '○', '◯', '●']);
        groups.insert('o', vec!['0', 'O', '°', '·', '.']);
        groups.insert('0', vec!['O', 'o', '○', '◯']);

        Self { groups }
    }

    /// Get a fade-in/fade-out character based on density level
    /// Used for smooth transitions from/to space characters
    fn get_density_char(&self, progress: f32) -> char {
        // Fade sequence: ' ' → '░' → '▒' → '▓'
        if progress < 0.25 {
            ' '
        } else if progress < 0.5 {
            '░'
        } else if progress < 0.75 {
            '▒'
        } else {
            '▓'
        }
    }

    /// Get intermediate character for morphing
    fn get_morph_char(&self, from: char, to: char, progress: f32) -> char {
        if progress < 0.3 {
            return from;
        }
        if progress > 0.7 {
            return to;
        }

        // Check if characters are in same group
        if let Some(group) = self.groups.get(&from) {
            if group.contains(&to) {
                // They're similar, pick an intermediate
                let index = ((group.len() as f32) * progress) as usize;
                return group.get(index).copied().unwrap_or(to);
            }
        }

        // Not in same group, use space as intermediate
        if progress < 0.5 {
            from
        } else {
            to
        }
    }
}

/// Art mixer for blending between ASCII art
pub struct ArtMixer {
    strategy: MixStrategy,
    char_groups: CharacterGroups,
}

impl ArtMixer {
    /// Create a new art mixer
    pub fn new(strategy: MixStrategy) -> Self {
        Self {
            strategy,
            char_groups: CharacterGroups::new(),
        }
    }

    /// Mix two art pieces together based on blend factor
    /// blend: 0.0 = fully source, 1.0 = fully target
    pub fn mix(&self, source: &str, target: &str, blend: f32) -> String {
        match self.strategy {
            MixStrategy::CharacterBlend => self.character_blend(source, target, blend),
            MixStrategy::SpaceFade => self.space_fade(source, target, blend),
            MixStrategy::Dissolve => self.dissolve(source, target, blend),
            MixStrategy::LineWipe => self.line_wipe(source, target, blend),
            MixStrategy::CharacterMorph => self.character_morph(source, target, blend),
        }
    }

    /// Simple character-by-character blend
    fn character_blend(&self, source: &str, target: &str, blend: f32) -> String {
        let source_lines: Vec<&str> = source.lines().collect();
        let target_lines: Vec<&str> = target.lines().collect();
        let max_lines = source_lines.len().max(target_lines.len());

        let mut result = Vec::new();

        for i in 0..max_lines {
            let source_line = source_lines.get(i).unwrap_or(&"");
            let target_line = target_lines.get(i).unwrap_or(&"");

            let mut line = String::new();
            let max_len = source_line.len().max(target_line.len());

            for j in 0..max_len {
                let source_char = source_line.chars().nth(j).unwrap_or(' ');
                let target_char = target_line.chars().nth(j).unwrap_or(' ');

                // Simple threshold-based selection
                let char = if blend < 0.5 {
                    source_char
                } else {
                    target_char
                };

                line.push(char);
            }

            result.push(line);
        }

        result.join("\n")
    }

    /// Fade through empty space
    fn space_fade(&self, source: &str, target: &str, blend: f32) -> String {
        let source_lines: Vec<&str> = source.lines().collect();
        let target_lines: Vec<&str> = target.lines().collect();
        let max_lines = source_lines.len().max(target_lines.len());

        let mut result = Vec::new();

        for i in 0..max_lines {
            let source_line = source_lines.get(i).unwrap_or(&"");
            let target_line = target_lines.get(i).unwrap_or(&"");

            let mut line = String::new();
            let max_len = source_line.len().max(target_line.len());

            for j in 0..max_len {
                let source_char = source_line.chars().nth(j).unwrap_or(' ');
                let target_char = target_line.chars().nth(j).unwrap_or(' ');

                let char = if blend < 0.33 {
                    // Fade out source
                    if source_char != ' ' && fastrand::f32() < blend * 3.0 {
                        ' '
                    } else {
                        source_char
                    }
                } else if blend > 0.66 {
                    // Fade in target
                    if target_char != ' ' && fastrand::f32() < (1.0 - blend) * 3.0 {
                        ' '
                    } else {
                        target_char
                    }
                } else {
                    // Middle phase - mostly empty
                    ' '
                };

                line.push(char);
            }

            result.push(line);
        }

        result.join("\n")
    }

    /// Dissolve effect with random pixels
    fn dissolve(&self, source: &str, target: &str, blend: f32) -> String {
        let source_lines: Vec<&str> = source.lines().collect();
        let target_lines: Vec<&str> = target.lines().collect();
        let max_lines = source_lines.len().max(target_lines.len());

        let mut result = Vec::new();

        // Use position-based pseudo-random for consistent dissolve
        for i in 0..max_lines {
            let source_line = source_lines.get(i).unwrap_or(&"");
            let target_line = target_lines.get(i).unwrap_or(&"");

            let mut line = String::new();
            let max_len = source_line.len().max(target_line.len());

            for j in 0..max_len {
                let source_char = source_line.chars().nth(j).unwrap_or(' ');
                let target_char = target_line.chars().nth(j).unwrap_or(' ');

                // Create position-based threshold
                let threshold = ((i * 31 + j * 17) % 100) as f32 / 100.0;

                let char = if blend > threshold {
                    target_char
                } else {
                    source_char
                };

                line.push(char);
            }

            result.push(line);
        }

        result.join("\n")
    }

    /// Line-by-line wipe effect
    fn line_wipe(&self, source: &str, target: &str, blend: f32) -> String {
        let source_lines: Vec<&str> = source.lines().collect();
        let target_lines: Vec<&str> = target.lines().collect();
        let max_lines = source_lines.len().max(target_lines.len());

        let mut result = Vec::new();
        let wipe_line = (blend * max_lines as f32) as usize;

        for i in 0..max_lines {
            let line = if i < wipe_line {
                target_lines.get(i).unwrap_or(&"")
            } else {
                source_lines.get(i).unwrap_or(&"")
            };

            result.push(line.to_string());
        }

        result.join("\n")
    }

    /// Character morphing for smooth transitions
    fn character_morph(&self, source: &str, target: &str, blend: f32) -> String {
        let source_lines: Vec<&str> = source.lines().collect();
        let target_lines: Vec<&str> = target.lines().collect();
        let max_lines = source_lines.len().max(target_lines.len());

        let mut result = Vec::new();

        for i in 0..max_lines {
            let source_line = source_lines.get(i).unwrap_or(&"");
            let target_line = target_lines.get(i).unwrap_or(&"");

            let mut line = String::new();
            let max_len = source_line.len().max(target_line.len());

            for j in 0..max_len {
                let source_char = source_line.chars().nth(j).unwrap_or(' ');
                let target_char = target_line.chars().nth(j).unwrap_or(' ');

                if source_char == target_char {
                    // Same character, no morphing needed
                    line.push(source_char);
                } else if source_char == ' ' {
                    // Fade in target: space → density chars → target
                    if blend < 0.7 {
                        // Show density character during fade-in
                        line.push(self.char_groups.get_density_char(blend));
                    } else {
                        // Final 30%: show target
                        line.push(target_char);
                    }
                } else if target_char == ' ' {
                    // Fade out source: source → density chars → space
                    if blend < 0.3 {
                        // First 30%: show source
                        line.push(source_char);
                    } else {
                        // Show density character during fade-out (reversed)
                        line.push(self.char_groups.get_density_char(1.0 - blend));
                    }
                } else {
                    // Morph between characters
                    let morph_char =
                        self.char_groups
                            .get_morph_char(source_char, target_char, blend);
                    line.push(morph_char);
                }
            }

            result.push(line);
        }

        result.join("\n")
    }
}

impl Default for ArtMixer {
    fn default() -> Self {
        Self::new(MixStrategy::CharacterMorph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_blend() {
        let mixer = ArtMixer::new(MixStrategy::CharacterBlend);
        let source = "AAA\nBBB";
        let target = "XXX\nYYY";

        let result1 = mixer.mix(source, target, 0.25);
        assert_eq!(result1, "AAA\nBBB");

        let result2 = mixer.mix(source, target, 0.75);
        assert_eq!(result2, "XXX\nYYY");
    }

    #[test]
    fn test_line_wipe() {
        let mixer = ArtMixer::new(MixStrategy::LineWipe);
        let source = "111\n222\n333\n444";
        let target = "AAA\nBBB\nCCC\nDDD";

        let result = mixer.mix(source, target, 0.5);
        assert_eq!(result, "AAA\nBBB\n333\n444");
    }
}
