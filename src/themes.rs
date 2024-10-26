use crate::error::{ChromaCatError, Result};
use colorgrad::{Color, Gradient, GradientBuilder, LinearGradient};
use std::str::FromStr;

/// Available color gradient themes
#[derive(Debug, Clone)]
pub enum Theme {
    // Classic Themes
    Rainbow,
    Grayscale,
    Sepia,
    Monochrome,

    // Nature Themes
    Ocean,
    Forest,
    Autumn,
    Sunset,
    Desert,
    Arctic,
    Tropical,

    // Aesthetic Themes
    Pastel,
    Neon,
    Retrowave,
    Vaporwave,

    // Tech Themes
    Matrix,
    Cyberpunk,
    Terminal,
    Hackerman,

    // Space Themes
    Nebula,
    Galaxy,
    Aurora,
    Cosmos,

    // Abstract Themes
    Heat,
    Ice,
    Fire,
    Toxic,

    // Mood Themes
    Calm,
    Energy,
    Dream,

    // Party Themes
    Rave,
    Disco,
    Festival,

    // Color Theory Themes
    Complementary,
    Analogous,
    Triadic,

    // Special Effects
    Hologram,
    Glitch,
    Plasma,
    Lightning,
}

impl Theme {
    /// Returns the color stops for the theme
    pub fn get_colors(&self) -> Vec<Color> {
        match self {
            // Classic Themes
            Theme::Rainbow => vec![
                Color::new(1.0, 0.0, 0.0, 1.0),   // Red
                Color::new(1.0, 0.5, 0.0, 1.0),   // Orange
                Color::new(1.0, 1.0, 0.0, 1.0),   // Yellow
                Color::new(0.0, 1.0, 0.0, 1.0),   // Green
                Color::new(0.0, 0.0, 1.0, 1.0),   // Blue
                Color::new(0.29, 0.0, 0.51, 1.0), // Indigo
                Color::new(0.58, 0.0, 0.83, 1.0), // Violet
            ],
            Theme::Grayscale => vec![
                Color::new(0.1, 0.1, 0.1, 1.0),
                Color::new(0.5, 0.5, 0.5, 1.0),
                Color::new(0.9, 0.9, 0.9, 1.0),
            ],
            Theme::Sepia => vec![
                Color::new(0.85, 0.75, 0.60, 1.0),
                Color::new(0.70, 0.60, 0.45, 1.0),
                Color::new(0.55, 0.45, 0.30, 1.0),
            ],
            Theme::Monochrome => vec![
                Color::new(0.0, 0.0, 0.8, 1.0),
                Color::new(0.0, 0.0, 0.4, 1.0),
                Color::new(0.0, 0.0, 0.1, 1.0),
            ],

            // Nature Themes
            Theme::Ocean => vec![
                Color::new(0.0, 0.47, 0.75, 1.0),  // Deep blue
                Color::new(0.0, 0.71, 0.85, 1.0),  // Medium blue
                Color::new(0.28, 0.79, 0.89, 1.0), // Light blue
                Color::new(0.56, 0.88, 0.94, 1.0), // Sky blue
            ],
            Theme::Forest => vec![
                Color::new(0.08, 0.32, 0.16, 1.0), // Dark green
                Color::new(0.18, 0.54, 0.34, 1.0), // Forest green
                Color::new(0.13, 0.54, 0.13, 1.0), // Green
                Color::new(0.60, 0.80, 0.20, 1.0), // Yellow green
            ],
            Theme::Autumn => vec![
                Color::new(0.65, 0.16, 0.16, 1.0), // Brown
                Color::new(0.82, 0.41, 0.12, 1.0), // Chocolate
                Color::new(1.0, 0.27, 0.0, 1.0),   // Red-orange
                Color::new(1.0, 0.55, 0.0, 1.0),   // Dark orange
            ],
            Theme::Sunset => vec![
                Color::new(0.98, 0.31, 0.42, 1.0), // Coral
                Color::new(0.99, 0.62, 0.45, 1.0), // Peach
                Color::new(0.97, 0.76, 0.44, 1.0), // Light orange
                Color::new(0.56, 0.28, 0.58, 1.0), // Purple
            ],
            Theme::Desert => vec![
                Color::new(0.94, 0.76, 0.56, 1.0), // Sand
                Color::new(0.85, 0.60, 0.35, 1.0), // Tan
                Color::new(0.76, 0.44, 0.24, 1.0), // Terra cotta
                Color::new(0.67, 0.28, 0.13, 1.0), // Rust
            ],
            Theme::Arctic => vec![
                Color::new(0.88, 0.96, 1.0, 1.0),  // Ice blue
                Color::new(0.78, 0.92, 1.0, 1.0),  // Light blue
                Color::new(0.68, 0.85, 0.95, 1.0), // Pale blue
                Color::new(0.58, 0.78, 0.90, 1.0), // Sky blue
            ],
            Theme::Tropical => vec![
                Color::new(1.0, 0.88, 0.37, 1.0),  // Yellow
                Color::new(1.0, 0.48, 0.42, 1.0),  // Coral
                Color::new(0.94, 0.23, 0.37, 1.0), // Hot pink
                Color::new(0.13, 0.70, 0.67, 1.0), // Turquoise
            ],

            // Aesthetic Themes
            Theme::Pastel => vec![
                Color::new(1.0, 0.71, 0.76, 1.0),  // Light pink
                Color::new(1.0, 0.85, 0.73, 1.0),  // Peach
                Color::new(1.0, 1.0, 0.88, 1.0),   // Light yellow
                Color::new(0.69, 0.88, 0.90, 1.0), // Powder blue
            ],
            Theme::Neon => vec![
                Color::new(1.0, 0.0, 1.0, 1.0), // Magenta
                Color::new(0.0, 1.0, 1.0, 1.0), // Cyan
                Color::new(1.0, 1.0, 0.0, 1.0), // Yellow
                Color::new(0.0, 1.0, 0.0, 1.0), // Green
            ],
            Theme::Retrowave => vec![
                Color::new(0.93, 0.0, 1.0, 1.0),  // Hot pink
                Color::new(0.47, 0.0, 0.86, 1.0), // Purple
                Color::new(0.0, 0.72, 1.0, 1.0),  // Cyan
            ],
            Theme::Vaporwave => vec![
                Color::new(1.0, 0.0, 1.0, 1.0),   // Magenta
                Color::new(0.0, 1.0, 1.0, 1.0),   // Cyan
                Color::new(0.47, 0.0, 0.86, 1.0), // Purple
                Color::new(1.0, 0.41, 0.71, 1.0), // Pink
            ],

            // Tech Themes
            Theme::Matrix => vec![
                Color::new(0.0, 0.5, 0.0, 1.0), // Dark green
                Color::new(0.0, 0.8, 0.0, 1.0), // Medium green
                Color::new(0.0, 1.0, 0.0, 1.0), // Bright green
            ],
            Theme::Cyberpunk => vec![
                Color::new(1.0, 0.0, 0.4, 1.0),  // Hot pink
                Color::new(0.0, 1.0, 1.0, 1.0),  // Cyan
                Color::new(1.0, 0.92, 0.0, 1.0), // Yellow
            ],
            Theme::Terminal => vec![
                Color::new(0.0, 0.75, 0.0, 1.0), // Green
                Color::new(0.0, 0.55, 0.0, 1.0), // Medium green
                Color::new(0.0, 0.35, 0.0, 1.0), // Dark green
            ],
            Theme::Hackerman => vec![
                Color::new(0.0, 1.0, 0.0, 1.0), // Bright green
                Color::new(0.0, 0.0, 0.0, 1.0), // Black
                Color::new(0.0, 0.5, 0.0, 1.0), // Dark green
            ],

            // Space Themes
            Theme::Nebula => vec![
                Color::new(0.29, 0.0, 0.51, 1.0), // Deep purple
                Color::new(0.86, 0.0, 1.0, 1.0),  // Bright purple
                Color::new(0.0, 0.72, 1.0, 1.0),  // Bright blue
                Color::new(1.0, 0.0, 0.5, 1.0),   // Pink
            ],
            Theme::Galaxy => vec![
                Color::new(0.0, 0.0, 0.2, 1.0),   // Dark blue
                Color::new(0.29, 0.0, 0.51, 1.0), // Purple
                Color::new(0.86, 0.0, 1.0, 1.0),  // Bright purple
                Color::new(1.0, 1.0, 1.0, 1.0),   // White
            ],
            Theme::Aurora => vec![
                Color::new(0.0, 1.0, 0.5, 1.0), // Green
                Color::new(0.0, 0.5, 1.0, 1.0), // Blue
                Color::new(0.5, 0.0, 1.0, 1.0), // Purple
                Color::new(1.0, 0.0, 0.5, 1.0), // Pink
            ],
            Theme::Cosmos => vec![
                Color::new(0.0, 0.0, 0.0, 1.0),   // Black
                Color::new(0.29, 0.0, 0.51, 1.0), // Deep purple
                Color::new(0.0, 0.0, 1.0, 1.0),   // Blue
                Color::new(1.0, 1.0, 1.0, 1.0),   // White
            ],

            // Abstract Themes
            Theme::Heat => vec![
                Color::new(1.0, 0.0, 0.0, 1.0), // Red
                Color::new(1.0, 0.5, 0.0, 1.0), // Orange
                Color::new(1.0, 1.0, 0.0, 1.0), // Yellow
            ],
            Theme::Ice => vec![
                Color::new(1.0, 1.0, 1.0, 1.0), // White
                Color::new(0.8, 0.9, 1.0, 1.0), // Light blue
                Color::new(0.6, 0.8, 1.0, 1.0), // Blue
                Color::new(0.4, 0.7, 1.0, 1.0), // Deep blue
            ],
            Theme::Fire => vec![
                Color::new(1.0, 0.0, 0.0, 1.0), // Red
                Color::new(1.0, 0.5, 0.0, 1.0), // Orange
                Color::new(1.0, 0.8, 0.0, 1.0), // Yellow
                Color::new(1.0, 0.3, 0.0, 1.0), // Dark orange
            ],
            Theme::Toxic => vec![
                Color::new(0.0, 1.0, 0.0, 1.0), // Green
                Color::new(0.8, 1.0, 0.0, 1.0), // Yellow-green
                Color::new(0.4, 0.8, 0.0, 1.0), // Dark green
            ],

            // Mood Themes
            Theme::Calm => vec![
                Color::new(0.53, 0.81, 0.92, 1.0), // Light blue
                Color::new(0.53, 0.81, 0.76, 1.0), // Turquoise
                Color::new(0.53, 0.81, 0.61, 1.0), // Sea green
            ],
            Theme::Energy => vec![
                Color::new(1.0, 0.0, 0.0, 1.0), // Red
                Color::new(1.0, 0.5, 0.0, 1.0), // Orange
                Color::new(1.0, 1.0, 0.0, 1.0), // Yellow
                Color::new(1.0, 0.0, 0.5, 1.0), // Pink
            ],
            Theme::Dream => vec![
                Color::new(0.86, 0.0, 1.0, 1.0),   // Purple
                Color::new(0.53, 0.81, 0.92, 1.0), // Light blue
                Color::new(1.0, 0.71, 0.76, 1.0),  // Pink
                // Continuing from Theme::Dream...
                Color::new(0.86, 0.0, 0.5, 1.0), // Dark pink
            ],

            // Party Themes
            Theme::Rave => vec![
                Color::new(1.0, 0.0, 1.0, 1.0), // Magenta
                Color::new(0.0, 1.0, 0.0, 1.0), // Green
                Color::new(1.0, 1.0, 0.0, 1.0), // Yellow
                Color::new(0.0, 1.0, 1.0, 1.0), // Cyan
                Color::new(1.0, 0.0, 0.0, 1.0), // Red
            ],
            Theme::Disco => vec![
                Color::new(1.0, 0.0, 0.5, 1.0), // Hot pink
                Color::new(0.5, 0.0, 1.0, 1.0), // Purple
                Color::new(0.0, 0.5, 1.0, 1.0), // Blue
                Color::new(1.0, 0.8, 0.0, 1.0), // Gold
            ],
            Theme::Festival => vec![
                Color::new(1.0, 0.4, 0.0, 1.0), // Orange
                Color::new(1.0, 0.0, 0.6, 1.0), // Pink
                Color::new(0.6, 0.0, 1.0, 1.0), // Purple
                Color::new(0.0, 0.8, 1.0, 1.0), // Blue
                Color::new(0.0, 1.0, 0.4, 1.0), // Green
            ],

            // Color Theory Themes
            Theme::Complementary => vec![
                Color::new(1.0, 0.0, 0.0, 1.0), // Red
                Color::new(0.5, 0.0, 0.0, 1.0), // Dark red
                Color::new(0.0, 1.0, 0.0, 1.0), // Green
                Color::new(0.0, 0.5, 0.0, 1.0), // Dark green
            ],
            Theme::Analogous => vec![
                Color::new(1.0, 0.0, 0.0, 1.0), // Red
                Color::new(1.0, 0.5, 0.0, 1.0), // Orange
                Color::new(1.0, 0.0, 0.5, 1.0), // Pink
            ],
            Theme::Triadic => vec![
                Color::new(1.0, 0.0, 0.0, 1.0), // Red
                Color::new(0.0, 1.0, 0.0, 1.0), // Green
                Color::new(0.0, 0.0, 1.0, 1.0), // Blue
            ],

            // Special Effects
            Theme::Hologram => vec![
                Color::new(0.0, 1.0, 1.0, 1.0), // Cyan
                Color::new(1.0, 0.0, 1.0, 1.0), // Magenta
                Color::new(0.0, 0.8, 1.0, 1.0), // Light blue
                Color::new(1.0, 0.0, 0.8, 1.0), // Pink
            ],
            Theme::Glitch => vec![
                Color::new(1.0, 0.0, 0.0, 1.0), // Red
                Color::new(0.0, 1.0, 1.0, 1.0), // Cyan
                Color::new(1.0, 0.0, 1.0, 1.0), // Magenta
                Color::new(0.0, 0.0, 0.0, 1.0), // Black
            ],
            Theme::Plasma => vec![
                Color::new(1.0, 0.0, 1.0, 1.0), // Magenta
                Color::new(0.5, 0.0, 1.0, 1.0), // Purple
                Color::new(0.0, 0.0, 1.0, 1.0), // Blue
                Color::new(0.0, 1.0, 1.0, 1.0), // Cyan
            ],
            Theme::Lightning => vec![
                Color::new(1.0, 1.0, 1.0, 1.0), // White
                Color::new(0.0, 1.0, 1.0, 1.0), // Cyan
                Color::new(0.0, 0.0, 1.0, 1.0), // Blue
                Color::new(0.5, 0.0, 1.0, 1.0), // Purple
            ],
        }
    }

    /// Returns a description of the theme
    pub fn description(&self) -> &'static str {
        match self {
            // Classic Themes
            Theme::Rainbow => "Classic rainbow colors (red through violet)",
            Theme::Grayscale => "Smooth transitions between black and white",
            Theme::Sepia => "Vintage brownscale reminiscent of old photographs",
            Theme::Monochrome => "Single color variations in intensity",

            // Nature Themes
            Theme::Ocean => "Cool blue tones reminiscent of ocean depths",
            Theme::Forest => "Natural green tones inspired by forests",
            Theme::Autumn => "Warm fall colors (browns, oranges, reds)",
            Theme::Sunset => "Warm evening sky colors with purples",
            Theme::Desert => "Warm earth tones inspired by desert landscapes",
            Theme::Arctic => "Cool, crisp colors of polar regions",
            Theme::Tropical => "Vibrant colors inspired by tropical paradise",

            // Aesthetic Themes
            Theme::Pastel => "Soft, muted colors for a gentle appearance",
            Theme::Neon => "Bright, vibrant colors that pop",
            Theme::Retrowave => "80s-inspired synthwave aesthetic",
            Theme::Vaporwave => "90s-inspired aesthetic with pink and cyan",

            // Tech Themes
            Theme::Matrix => "Digital green inspired by The Matrix",
            Theme::Cyberpunk => "High-tech urban future aesthetic",
            Theme::Terminal => "Classic computer terminal green",
            Theme::Hackerman => "Retro hacker aesthetic with deep greens",

            // Space Themes
            Theme::Nebula => "Cosmic colors of stellar nurseries",
            Theme::Galaxy => "Deep space with stellar highlights",
            Theme::Aurora => "Northern lights inspired colors",
            Theme::Cosmos => "Deep space with stars",

            // Abstract Themes
            Theme::Heat => "Warm colors transitioning from red through orange to yellow",
            Theme::Ice => "Cool, frozen tones of glaciers and ice",
            Theme::Fire => "Hot, intense flames and embers",
            Theme::Toxic => "Radioactive greens and acid colors",

            // Mood Themes
            Theme::Calm => "Soothing blues and greens",
            Theme::Energy => "Vibrant, energetic colors",
            Theme::Dream => "Soft, dreamy pastels with purple",

            // Party Themes
            Theme::Rave => "Intense party colors that pulse",
            Theme::Disco => "70s disco-inspired colors",
            Theme::Festival => "Vibrant festival color palette",

            // Color Theory Themes
            Theme::Complementary => "Opposite colors on the color wheel",
            Theme::Analogous => "Adjacent colors on the color wheel",
            Theme::Triadic => "Three colors equally spaced on the color wheel",

            // Special Effects
            Theme::Hologram => "Futuristic holographic effect",
            Theme::Glitch => "Digital glitch artifact colors",
            Theme::Plasma => "Electric plasma-like effect",
            Theme::Lightning => "Electric discharge colors",
        }
    }

    /// Creates a gradient from the theme
    pub fn create_gradient(&self) -> Result<Box<dyn Gradient + Send + Sync>> {
        let colors = self.get_colors();
        let gradient = GradientBuilder::new()
            .colors(&colors)
            .build::<LinearGradient>()
            .map_err(|e| ChromaCatError::GradientError(e.to_string()))?;
        Ok(Box::new(gradient))
    }

    /// Returns a list of all available themes
    pub fn list_all() -> Vec<(String, &'static str)> {
        vec![
            // Classic Themes
            ("rainbow".to_string(), Theme::Rainbow.description()),
            ("grayscale".to_string(), Theme::Grayscale.description()),
            ("sepia".to_string(), Theme::Sepia.description()),
            ("monochrome".to_string(), Theme::Monochrome.description()),
            // Nature Themes
            ("ocean".to_string(), Theme::Ocean.description()),
            ("forest".to_string(), Theme::Forest.description()),
            ("autumn".to_string(), Theme::Autumn.description()),
            ("sunset".to_string(), Theme::Sunset.description()),
            ("desert".to_string(), Theme::Desert.description()),
            ("arctic".to_string(), Theme::Arctic.description()),
            ("tropical".to_string(), Theme::Tropical.description()),
            // Aesthetic Themes
            ("pastel".to_string(), Theme::Pastel.description()),
            ("neon".to_string(), Theme::Neon.description()),
            ("retrowave".to_string(), Theme::Retrowave.description()),
            ("vaporwave".to_string(), Theme::Vaporwave.description()),
            // Tech Themes
            ("matrix".to_string(), Theme::Matrix.description()),
            ("cyberpunk".to_string(), Theme::Cyberpunk.description()),
            ("terminal".to_string(), Theme::Terminal.description()),
            ("hackerman".to_string(), Theme::Hackerman.description()),
            // Space Themes
            ("nebula".to_string(), Theme::Nebula.description()),
            ("galaxy".to_string(), Theme::Galaxy.description()),
            ("aurora".to_string(), Theme::Aurora.description()),
            ("cosmos".to_string(), Theme::Cosmos.description()),
            // Abstract Themes
            ("heat".to_string(), Theme::Heat.description()),
            ("ice".to_string(), Theme::Ice.description()),
            ("fire".to_string(), Theme::Fire.description()),
            ("toxic".to_string(), Theme::Toxic.description()),
            // Mood Themes
            ("calm".to_string(), Theme::Calm.description()),
            ("energy".to_string(), Theme::Energy.description()),
            ("dream".to_string(), Theme::Dream.description()),
            // Party Themes
            ("rave".to_string(), Theme::Rave.description()),
            ("disco".to_string(), Theme::Disco.description()),
            ("festival".to_string(), Theme::Festival.description()),
            // Color Theory Themes
            (
                "complementary".to_string(),
                Theme::Complementary.description(),
            ),
            ("analogous".to_string(), Theme::Analogous.description()),
            ("triadic".to_string(), Theme::Triadic.description()),
            // Special Effects
            ("hologram".to_string(), Theme::Hologram.description()),
            ("glitch".to_string(), Theme::Glitch.description()),
            ("plasma".to_string(), Theme::Plasma.description()),
            ("lightning".to_string(), Theme::Lightning.description()),
        ]
    }
}

impl FromStr for Theme {
    type Err = ChromaCatError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            // Classic Themes
            "rainbow" => Ok(Theme::Rainbow),
            "grayscale" => Ok(Theme::Grayscale),
            "sepia" => Ok(Theme::Sepia),
            "monochrome" => Ok(Theme::Monochrome),

            // Nature Themes
            "ocean" => Ok(Theme::Ocean),
            "forest" => Ok(Theme::Forest),
            "autumn" => Ok(Theme::Autumn),
            "sunset" => Ok(Theme::Sunset),
            "desert" => Ok(Theme::Desert),
            "arctic" => Ok(Theme::Arctic),
            "tropical" => Ok(Theme::Tropical),

            // Aesthetic Themes
            "pastel" => Ok(Theme::Pastel),
            "neon" => Ok(Theme::Neon),
            "retrowave" => Ok(Theme::Retrowave),
            "vaporwave" => Ok(Theme::Vaporwave),

            // Tech Themes
            "matrix" => Ok(Theme::Matrix),
            "cyberpunk" => Ok(Theme::Cyberpunk),
            "terminal" => Ok(Theme::Terminal),
            "hackerman" => Ok(Theme::Hackerman),

            // Space Themes
            "nebula" => Ok(Theme::Nebula),
            "galaxy" => Ok(Theme::Galaxy),
            "aurora" => Ok(Theme::Aurora),
            "cosmos" => Ok(Theme::Cosmos),

            // Abstract Themes
            "heat" => Ok(Theme::Heat),
            "ice" => Ok(Theme::Ice),
            "fire" => Ok(Theme::Fire),
            "toxic" => Ok(Theme::Toxic),

            // Mood Themes
            "calm" => Ok(Theme::Calm),
            "energy" => Ok(Theme::Energy),
            "dream" => Ok(Theme::Dream),

            // Party Themes
            "rave" => Ok(Theme::Rave),
            "disco" => Ok(Theme::Disco),
            "festival" => Ok(Theme::Festival),

            // Color Theory Themes
            "complementary" => Ok(Theme::Complementary),
            "analogous" => Ok(Theme::Analogous),
            "triadic" => Ok(Theme::Triadic),

            // Special Effects
            "hologram" => Ok(Theme::Hologram),
            "glitch" => Ok(Theme::Glitch),
            "plasma" => Ok(Theme::Plasma),
            "lightning" => Ok(Theme::Lightning),

            // Invalid theme
            _ => Err(ChromaCatError::InvalidTheme(s.to_string())),
        }
    }
}
