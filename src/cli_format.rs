//! CLI formatting and styling helpers

/// Core UI styling and help text formatting for the CLI
pub struct CliFormat;

impl CliFormat {
    // Color codes
    pub const TITLE_1: &'static str = "\x1b[1;38;5;219m"; // Bright pink
    pub const TITLE_2: &'static str = "\x1b[1;38;5;213m"; // Magenta
    pub const CORE: &'static str = "\x1b[38;5;219m"; // Pink
    pub const ANIMATION: &'static str = "\x1b[38;5;213m"; // Magenta
    pub const GENERAL: &'static str = "\x1b[38;5;147m"; // Light purple
    pub const PATTERN: &'static str = "\x1b[38;5;117m"; // Light blue
    pub const RESET: &'static str = "\x1b[0m";

    // Help headings
    pub const HEADING_INPUT: &'static str = "📁 Input/Output";
    pub const HEADING_CORE: &'static str = "🎨 Core Options";
    pub const HEADING_ANIMATION: &'static str = "✨ Animation";
    pub const HEADING_GENERAL: &'static str = "⚙️ General";
    pub const HEADING_WAVE: &'static str = "🌊 Wave/Ripple";
    pub const HEADING_PLASMA: &'static str = "🌀 Plasma/Perlin";
    pub const HEADING_SPIRAL: &'static str = "💫 Spiral/Diamond";
    pub const HEADING_OTHER: &'static str = "📐 Other";
    pub const HEADING_PLAYLIST: &'static str = "📝 Playlist";

    // Add new color constants and heading constants
    pub const PARAM: &'static str = "\x1b[38;5;149m"; // Light green for parameters
    pub const PARAM_VALUE: &'static str = "\x1b[38;5;215m"; // Light orange for values
    pub const DESCRIPTION: &'static str = "\x1b[38;5;252m"; // Light gray for descriptions
    pub const SEPARATOR: &'static str = "\x1b[38;5;239m"; // Dark gray for separators

    pub const HEADING_PARAMS: &'static str = "🎯 Pattern Parameters";
    pub const HEADING_EXAMPLES: &'static str = "📚 Examples";

    pub fn wrap(color: &str, text: &str) -> String {
        format!("{}{}{}", color, text, Self::RESET)
    }

    pub fn core(text: &str) -> String {
        Self::wrap(Self::CORE, text)
    }

    pub fn animation(text: &str) -> String {
        Self::wrap(Self::ANIMATION, text)
    }

    pub fn general(text: &str) -> String {
        Self::wrap(Self::GENERAL, text)
    }

    pub fn pattern(text: &str) -> String {
        Self::wrap(Self::PATTERN, text)
    }

    pub fn param(text: &str) -> String {
        Self::wrap(Self::PARAM, text)
    }

    pub fn param_value(text: &str) -> String {
        Self::wrap(Self::PARAM_VALUE, text)
    }

    pub fn description(text: &str) -> String {
        Self::wrap(Self::DESCRIPTION, text)
    }

    pub fn separator(text: &str) -> String {
        Self::wrap(Self::SEPARATOR, text)
    }

    /// Highlights key terms in description text with meaningful colors
    pub fn highlight_description(text: &str) -> String {
        let highlights = [
            // Action words - use PARAM (light green) to highlight what the user can do
            ("select", Self::PARAM),
            ("enable", Self::PARAM),
            ("disable", Self::PARAM),
            ("customize", Self::PARAM),
            ("load", Self::PARAM),
            ("specify", Self::PARAM),
            ("use", Self::PARAM),
            ("reads", Self::PARAM),
            // Values and ranges - use PARAM_VALUE (light orange) to highlight configurable values
            ("0.0-1.0", Self::PARAM_VALUE),
            ("0.1-10.0", Self::PARAM_VALUE),
            ("0.1-2.0", Self::PARAM_VALUE),
            ("1-144", Self::PARAM_VALUE),
            ("true/false", Self::PARAM_VALUE),
            ("infinite", Self::PARAM_VALUE),
            // Core features - use CORE (pink) to highlight main functionality
            ("pattern", Self::CORE),
            ("theme", Self::CORE),
            ("gradient", Self::CORE),
            ("color", Self::CORE),
            // Animation terms - use ANIMATION (magenta) for movement-related terms
            ("animation", Self::ANIMATION),
            ("animated", Self::ANIMATION),
            ("transitions", Self::ANIMATION),
            ("fps", Self::ANIMATION),
            ("speed", Self::ANIMATION),
            ("duration", Self::ANIMATION),
            // File operations - use GENERAL (light purple) for I/O related terms
            ("stdin", Self::GENERAL),
            ("input", Self::GENERAL),
            ("output", Self::GENERAL),
            ("file", Self::GENERAL),
            // Parameters - use PATTERN (light blue) for parameter-related terms
            ("--param", Self::PATTERN),
            ("parameters", Self::PATTERN),
            ("key=value", Self::PATTERN),
            ("comma-separated", Self::PATTERN),
        ];

        let mut result = text.to_string();
        for (term, color) in highlights.iter() {
            // Use regex to match whole words only
            let pattern = format!(r"\b{}\b", regex::escape(term));
            if let Ok(re) = regex::Regex::new(&pattern) {
                result = re
                    .replace_all(&result, format!("{}{}{}", color, term, Self::RESET))
                    .to_string();
            }
        }

        // Wrap the entire string in description color
        format!("{}{}{}", Self::DESCRIPTION, result, Self::RESET)
    }
}

// Add this trait definition
pub trait PadToWidth {
    fn pad_to_width(&self, width: usize) -> String;
}

impl PadToWidth for String {
    fn pad_to_width(&self, width: usize) -> String {
        if self.len() >= width {
            self.clone()
        } else {
            format!("{:<width$}", self, width = width)
        }
    }
}

impl PadToWidth for &str {
    fn pad_to_width(&self, width: usize) -> String {
        if self.len() >= width {
            self.to_string()
        } else {
            format!("{:<width$}", self, width = width)
        }
    }
}
