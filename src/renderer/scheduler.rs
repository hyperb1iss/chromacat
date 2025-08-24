#[derive(Debug, Clone)]
pub struct Scene {
    pub pattern_id: String,
    pub theme_name: String,
    pub duration_secs: f32,
}

#[derive(Debug, Default, Clone)]
pub struct SceneScheduler {
    scenes: Vec<Scene>,
    current_index: usize,
    elapsed: f32,
    enabled: bool,
}

impl SceneScheduler {
    pub fn new(scenes: Vec<Scene>) -> Self {
        Self { scenes, current_index: 0, elapsed: 0.0, enabled: true }
    }

    pub fn is_enabled(&self) -> bool { self.enabled }
    pub fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }

    pub fn current(&self) -> Option<&Scene> { self.scenes.get(self.current_index) }

    /// Advances time; returns next scene when duration completes
    pub fn tick(&mut self, dt: f32) -> Option<&Scene> {
        if !self.enabled || self.scenes.is_empty() { return None; }
        self.elapsed += dt;
        let cur = &self.scenes[self.current_index];
        if self.elapsed >= cur.duration_secs {
            self.elapsed = 0.0;
            self.current_index = (self.current_index + 1) % self.scenes.len();
            return self.scenes.get(self.current_index);
        }
        None
    }
}

impl From<SceneScheduler> for Vec<crate::recipes::SceneRecipe> {
    fn from(s: SceneScheduler) -> Self {
        s.scenes
            .into_iter()
            .map(|sc| crate::recipes::SceneRecipe { pattern_id: sc.pattern_id, theme_name: sc.theme_name, duration_secs: sc.duration_secs })
            .collect()
    }
}


