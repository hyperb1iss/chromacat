use crate::pattern::config::PatternParams;
use crate::pattern::params::PatternParam;
use crate::pattern::patterns::*;
use std::collections::HashMap;
use std::sync::Arc; // Import all pattern types

/// Metadata about a pattern including its name, description, and parameters
pub struct PatternMetadata {
    /// Unique identifier for the pattern
    pub id: &'static str,
    /// Display name for the pattern
    pub name: &'static str,
    /// Description of what the pattern does
    pub description: &'static str,
    /// Default parameters for this pattern
    default_params: Arc<Box<dyn PatternParam + Send + Sync>>,
}

impl Clone for PatternMetadata {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name,
            description: self.description,
            default_params: Arc::clone(&self.default_params),
        }
    }
}

// Add this impl block after the Clone impl
impl PatternMetadata {
    /// Gets the parameters for this pattern
    pub fn params(&self) -> &dyn PatternParam {
        &**self.default_params
    }
}

/// Macro to define pattern registration entries
macro_rules! define_pattern_registry {
    ($(
        $id:expr => {
            variant: $variant:ident,
            params: $params:ident
        }
    ),* $(,)?) => {
        impl PatternRegistry {
            fn create_default() -> HashMap<String, PatternMetadata> {
                let mut patterns = HashMap::new();
                $(
                    let default_params = Box::new($params::default());
                    patterns.insert($id.to_string(), PatternMetadata {
                        id: $id,
                        name: default_params.name(),
                        description: default_params.description(),
                        default_params: Arc::new(default_params),
                    });
                )*
                patterns
            }

            fn pattern_to_params(&self, id: &str, params: Box<dyn PatternParam>) -> Result<PatternParams, String> {
                match id {
                    $(
                        $id => Ok(PatternParams::$variant(
                            params.as_any()
                                .downcast_ref::<$params>()
                                .ok_or_else(|| format!("Invalid parameter type for {}", $id))?
                                .clone()
                        )),
                    )*
                    _ => Err(format!("Unknown pattern: {}", id))
                }
            }

            /// Gets the pattern ID corresponding to the given parameters
            pub fn get_pattern_id(&self, params: &PatternParams) -> Option<&str> {
                match params {
                    $(PatternParams::$variant(_) => Some($id),)*
                }
            }
        }
    };
}

// Define all available patterns with simplified entries
define_pattern_registry! {
    "horizontal" => {
        variant: Horizontal,
        params: HorizontalParams
    },
    "diagonal" => {
        variant: Diagonal,
        params: DiagonalParams
    },
    "plasma" => {
        variant: Plasma,
        params: PlasmaParams
    },
    "ripple" => {
        variant: Ripple,
        params: RippleParams
    },
    "wave" => {
        variant: Wave,
        params: WaveParams
    },
    "spiral" => {
        variant: Spiral,
        params: SpiralParams
    },
    "checkerboard" => {
        variant: Checkerboard,
        params: CheckerboardParams
    },
    "diamond" => {
        variant: Diamond,
        params: DiamondParams
    },
    "perlin" => {
        variant: Perlin,
        params: PerlinParams
    },
    "rain" => {
        variant: PixelRain,
        params: PixelRainParams
    },
    "fire" => {
        variant: Fire,
        params: FireParams
    },
    "aurora" => {
        variant: Aurora,
        params: AuroraParams
    },
    "kaleidoscope" => {
        variant: Kaleidoscope,
        params: KaleidoscopeParams
    },
}

/// Registry for managing available patterns
pub struct PatternRegistry {
    patterns: HashMap<String, PatternMetadata>,
}

impl Default for PatternRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternRegistry {
    /// Creates a new pattern registry with all available patterns
    pub fn new() -> Self {
        Self {
            patterns: Self::create_default(),
        }
    }

    /// Gets metadata for a specific pattern
    pub fn get_pattern(&self, id: &str) -> Option<&PatternMetadata> {
        self.patterns.get(id)
    }

    /// Lists all available pattern IDs
    pub fn list_patterns(&self) -> Vec<&str> {
        self.patterns.keys().map(|s| s.as_str()).collect()
    }

    /// Creates default parameters for a pattern
    pub fn create_pattern_params(&self, id: &str) -> Option<PatternParams> {
        self.get_pattern(id).map(|metadata| {
            self.pattern_to_params(id, metadata.default_params.as_ref().clone_param())
                .unwrap_or_else(|_| PatternParams::Horizontal(Default::default()))
        })
    }

    /// Validates parameters for a pattern
    pub fn validate_params(&self, id: &str, params: &str) -> Result<(), String> {
        if let Some(metadata) = self.get_pattern(id) {
            metadata.default_params.validate(params)
        } else {
            Err(format!("Unknown pattern: {}", id))
        }
    }

    /// Parses parameters for a pattern
    pub fn parse_params(&self, id: &str, params: &str) -> Result<PatternParams, String> {
        if let Some(metadata) = self.get_pattern(id) {
            let parsed = metadata.default_params.parse(params)?;
            self.pattern_to_params(id, parsed)
        } else {
            Err(format!("Unknown pattern: {}", id))
        }
    }
}

// Create a lazy static instance for global access
lazy_static::lazy_static! {
    pub static ref REGISTRY: PatternRegistry = PatternRegistry::new();
}
