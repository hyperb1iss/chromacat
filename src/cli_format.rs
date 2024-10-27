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
}
