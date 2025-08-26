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
        Self {
            scenes,
            current_index: 0,
            elapsed: 0.0,
            enabled: true,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn current(&self) -> Option<&Scene> {
        self.scenes.get(self.current_index)
    }

    /// Mutates the current scene's pattern if available
    pub fn set_current_pattern(&mut self, pattern_id: &str) {
        if let Some(sc) = self.scenes.get_mut(self.current_index) {
            sc.pattern_id = pattern_id.to_string();
        }
    }

    /// Mutates the current scene's theme if available
    pub fn set_current_theme(&mut self, theme_name: &str) {
        if let Some(sc) = self.scenes.get_mut(self.current_index) {
            sc.theme_name = theme_name.to_string();
        }
    }

    /// Reseed with a varied list of scenes using simple strided selection to avoid repeats
    pub fn reseed_variety(&mut self, patterns: &[String], themes: &[String], count: usize) {
        if patterns.is_empty() || themes.is_empty() {
            return;
        }
        let plen = patterns.len();
        let tlen = themes.len();
        let mut scenes = Vec::with_capacity(count.max(2));
        for i in 0..count.max(2) {
            let p = &patterns[(i * 3) % plen];
            let t = &themes[(i * 5 + 7) % tlen];
            // Durations vary 8..17 seconds in a repeating pattern
            let duration = 8.0 + ((i % 4) as f32) * 3.0;
            scenes.push(Scene {
                pattern_id: p.clone(),
                theme_name: t.clone(),
                duration_secs: duration,
            });
        }
        self.scenes = scenes;
        self.current_index = 0;
        self.elapsed = 0.0;
        self.enabled = true;
    }

    /// Advances time; returns next scene when duration completes
    pub fn tick(&mut self, dt: f32) -> Option<&Scene> {
        if !self.enabled || self.scenes.is_empty() {
            return None;
        }
        self.elapsed += dt;
        let cur = &self.scenes[self.current_index];
        if self.elapsed >= cur.duration_secs {
            self.elapsed = 0.0;
            self.current_index = (self.current_index + 1) % self.scenes.len();
            return self.scenes.get(self.current_index);
        }
        None
    }

    /// Jump to next scene immediately
    pub fn jump_next(&mut self) -> Option<&Scene> {
        if self.scenes.is_empty() {
            return None;
        }
        self.elapsed = 0.0;
        self.current_index = (self.current_index + 1) % self.scenes.len();
        self.scenes.get(self.current_index)
    }

    /// Jump to previous scene immediately
    pub fn jump_prev(&mut self) -> Option<&Scene> {
        if self.scenes.is_empty() {
            return None;
        }
        self.elapsed = 0.0;
        if self.current_index == 0 {
            self.current_index = self.scenes.len() - 1;
        } else {
            self.current_index -= 1;
        }
        self.scenes.get(self.current_index)
    }
}

impl From<SceneScheduler> for Vec<crate::recipes::SceneRecipe> {
    fn from(s: SceneScheduler) -> Self {
        s.scenes
            .into_iter()
            .map(|sc| crate::recipes::SceneRecipe {
                pattern_id: sc.pattern_id,
                theme_name: sc.theme_name,
                duration_secs: sc.duration_secs,
            })
            .collect()
    }
}
