use fastrand::Rng;

#[derive(Debug, Clone)]
pub struct Scene {
    pub pattern_id: String,
    pub theme_name: String,
    pub duration_secs: f32,
}

#[derive(Debug, Clone)]
pub struct SceneScheduler {
    scenes: Vec<Scene>,
    current_index: usize,
    elapsed: f32,
    enabled: bool,
    rng: Rng,
}

impl Default for SceneScheduler {
    fn default() -> Self {
        Self {
            scenes: Vec::new(),
            current_index: 0,
            elapsed: 0.0,
            enabled: true,
            rng: Rng::new(),
        }
    }
}

impl SceneScheduler {
    pub fn new(scenes: Vec<Scene>) -> Self {
        Self {
            scenes,
            current_index: 0,
            elapsed: 0.0,
            enabled: true,
            rng: Rng::new(),
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

    /// Reseed with a varied list of scenes using random selection with variety tracking
    pub fn reseed_variety(&mut self, patterns: &[String], themes: &[String], count: usize) {
        if patterns.is_empty() || themes.is_empty() {
            return;
        }

        let count = count.max(2);
        let mut scenes = Vec::with_capacity(count);
        let mut last_pattern: Option<&str> = None;
        let mut last_theme: Option<&str> = None;

        for _ in 0..count {
            // Pick a random pattern, avoiding immediate repeats
            let pattern = loop {
                let p = &patterns[self.rng.usize(0..patterns.len())];
                if last_pattern.is_none_or(|lp| lp != p) {
                    break p;
                }
            };

            // Pick a random theme, avoiding immediate repeats
            let theme = loop {
                let t = &themes[self.rng.usize(0..themes.len())];
                if last_theme.is_none_or(|lt| lt != t) {
                    break t;
                }
            };

            // Randomize durations between 8-18 seconds
            let duration = 8.0 + self.rng.f32() * 10.0;

            scenes.push(Scene {
                pattern_id: pattern.clone(),
                theme_name: theme.clone(),
                duration_secs: duration,
            });

            last_pattern = Some(pattern);
            last_theme = Some(theme);
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
