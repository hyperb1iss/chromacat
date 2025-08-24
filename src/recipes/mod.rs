use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneRecipe {
    pub pattern_id: String,
    pub theme_name: String,
    pub duration_secs: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LfoRecipe {
    pub frequency_hz: f64,
    pub min: f64,
    pub max: f64,
    pub depth: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRecipe {
    pub target_param: String,
    pub lfo: LfoRecipe,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Recipe {
    pub current_theme: Option<String>,
    pub current_pattern: Option<String>,
    pub scenes: Vec<SceneRecipe>,
    pub routes: Vec<RouteRecipe>,
    pub theme_mode: Option<u8>,
    pub crossfade_seconds: Option<f32>,
}
