use super::scheduler::SceneScheduler;
use crate::pattern::REGISTRY;
use crate::playlist::Playlist;
use crate::themes;
use chrono::Timelike;
/// Automix system for seamless transitions between patterns, themes, and art
///
/// This module provides smooth, automatic transitions between different visual
/// elements to create an engaging, dynamic experience in playground mode.
use std::time::{Duration, Instant};

/// Transition types for smooth blending
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransitionType {
    /// Instant switch
    Cut,
    /// Smooth fade between states
    Crossfade,
    /// Morph parameters gradually
    Morph,
    /// Slide from one to another
    Slide,
}

/// Automix modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutomixMode {
    /// Follow a predefined playlist
    Playlist,
    /// Random selection with smart variety
    Random,
    /// Curated showcase of best combinations
    Showcase,
    /// Adaptive based on time of day
    Adaptive,
    /// Manual control only
    Off,
}

/// State of a visual element during transition
#[derive(Debug, Clone)]
struct TransitionState {
    /// Current blend factor (0.0 = source, 1.0 = target)
    blend: f32,
    /// Transition duration
    duration: Duration,
    /// Time when transition started
    start_time: Instant,
    /// Type of transition
    transition_type: TransitionType,
}

impl TransitionState {
    fn new(duration: Duration, transition_type: TransitionType) -> Self {
        Self {
            blend: 0.0,
            duration,
            start_time: Instant::now(),
            transition_type,
        }
    }

    /// Update transition progress, returns true when complete
    fn update(&mut self) -> bool {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let progress = (elapsed / self.duration.as_secs_f32()).min(1.0);

        // Apply easing curve based on transition type
        self.blend = match self.transition_type {
            TransitionType::Cut => 1.0,
            TransitionType::Crossfade => progress,
            TransitionType::Morph => ease_in_out_cubic(progress),
            TransitionType::Slide => ease_out_quart(progress),
        };

        progress >= 1.0
    }
}

/// Main automix controller
pub struct Automix {
    /// Current mode
    mode: AutomixMode,

    /// Scene scheduler for variety mode
    scheduler: SceneScheduler,

    /// Playlist for structured sequences
    playlist: Option<Playlist>,
    playlist_index: usize,

    /// Current scene timing
    scene_start: Instant,
    scene_duration: Duration,

    /// Transition state
    transition: Option<TransitionState>,

    /// Current and target states
    current_pattern: String,
    current_theme: String,
    current_art: Option<String>,

    target_pattern: Option<String>,
    target_theme: Option<String>,
    target_art: Option<String>,
    
    /// Queue of pending changes (to be applied one at a time)
    pending_changes: Vec<(String, String)>, // (type, value)

    /// Showcase sequences (curated combinations)
    showcase_sequences: Vec<ShowcaseSequence>,
    showcase_index: usize,

    /// Settings
    min_scene_duration: Duration,
    max_scene_duration: Duration,
    transition_duration: Duration,
    default_transition: TransitionType,
}

/// A curated showcase sequence
#[derive(Debug, Clone)]
struct ShowcaseSequence {
    name: String,
    pattern: String,
    theme: String,
    art: Option<String>,
    duration: Duration,
    params: Vec<(String, f64)>,
}

impl Automix {
    /// Create a new automix system
    pub fn new() -> Self {
        // Create default showcase sequences
        let showcase_sequences = vec![
            ShowcaseSequence {
                name: "Neon Dreams".to_string(),
                pattern: "plasma".to_string(),
                theme: "neon".to_string(),
                art: Some("cityscape".to_string()),
                duration: Duration::from_secs(15), // 15 seconds for testing
                params: vec![("scale".to_string(), 2.0), ("complexity".to_string(), 3.0)],
            },
            ShowcaseSequence {
                name: "Ocean Waves".to_string(),
                pattern: "wave".to_string(),
                theme: "ocean".to_string(),
                art: Some("waves".to_string()),
                duration: Duration::from_secs(12), // 12 seconds for testing
                params: vec![
                    ("frequency".to_string(), 1.5),
                    ("amplitude".to_string(), 0.8),
                ],
            },
            ShowcaseSequence {
                name: "Matrix Rain".to_string(),
                pattern: "rain".to_string(),
                theme: "matrix".to_string(),
                art: Some("matrix".to_string()),
                duration: Duration::from_secs(10), // 10 seconds for testing
                params: vec![("density".to_string(), 1.5), ("speed".to_string(), 2.0)],
            },
            ShowcaseSequence {
                name: "Aurora Borealis".to_string(),
                pattern: "aurora".to_string(),
                theme: "aurora".to_string(),
                art: Some("rainbow".to_string()),
                duration: Duration::from_secs(18), // 18 seconds for testing
                params: vec![("turbulence".to_string(), 0.3), ("layers".to_string(), 3.0)],
            },
            ShowcaseSequence {
                name: "Digital Spiral".to_string(),
                pattern: "spiral".to_string(),
                theme: "cyberpunk".to_string(),
                art: Some("blocks".to_string()),
                duration: Duration::from_secs(10), // 10 seconds for testing
                params: vec![("arms".to_string(), 5.0), ("tightness".to_string(), 0.5)],
            },
        ];

        Self {
            mode: AutomixMode::Off,
            scheduler: SceneScheduler::default(),
            playlist: None,
            playlist_index: 0,
            scene_start: Instant::now(),
            scene_duration: Duration::from_secs(10),
            transition: None,
            current_pattern: "diagonal".to_string(),
            current_theme: "rainbow".to_string(),
            current_art: Some("rainbow".to_string()),
            target_pattern: None,
            target_theme: None,
            target_art: None,
            pending_changes: Vec::new(),
            showcase_sequences,
            showcase_index: 0,
            min_scene_duration: Duration::from_secs(10),  // 10 seconds for testing
            max_scene_duration: Duration::from_secs(20), // 20 seconds for testing
            transition_duration: Duration::from_secs(5), // 5 second transitions for testing
            default_transition: TransitionType::Crossfade,
        }
    }

    /// Set the automix mode
    pub fn set_mode(&mut self, mode: AutomixMode) {
        self.mode = mode;
        self.scene_start = Instant::now();

        match mode {
            AutomixMode::Random => {
                self.init_random_mode();
            }
            AutomixMode::Showcase => {
                self.showcase_index = 0;
                self.apply_showcase_sequence(0);
            }
            AutomixMode::Playlist => {
                if self.playlist.is_some() {
                    self.playlist_index = 0;
                    self.apply_playlist_entry(0);
                }
            }
            _ => {}
        }
    }

    /// Initialize random mode with variety
    fn init_random_mode(&mut self) {
        let patterns = REGISTRY
            .list_patterns()
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        let themes = themes::all_themes()
            .iter()
            .map(|t| t.name.clone())
            .collect::<Vec<_>>();

        self.scheduler.reseed_variety(&patterns, &themes, 10);
        self.scene_duration = Duration::from_secs(15); // 15 seconds for testing
    }

    /// Apply a showcase sequence
    fn apply_showcase_sequence(&mut self, index: usize) {
        if let Some(seq) = self.showcase_sequences.get(index) {
            self.start_transition(
                Some(seq.pattern.clone()),
                Some(seq.theme.clone()),
                seq.art.clone(),
                seq.duration,
                self.default_transition,
            );
        }
    }

    /// Apply a playlist entry
    fn apply_playlist_entry(&mut self, index: usize) {
        if let Some(playlist) = &self.playlist {
            if let Some(entry) = playlist.entries.get(index) {
                self.start_transition(
                    Some(entry.pattern.clone()),
                    Some(entry.theme.clone()),
                    entry.art.as_ref().map(|a| a.as_str().to_string()),
                    entry.get_duration(),
                    self.default_transition,
                );
            }
        }
    }

    /// Start a transition to new values
    fn start_transition(
        &mut self,
        pattern: Option<String>,
        theme: Option<String>,
        art: Option<String>,
        duration: Duration,
        transition_type: TransitionType,
    ) {
        self.target_pattern = pattern.clone();
        self.target_theme = theme.clone();
        self.target_art = art.clone();
        self.scene_duration = duration;
        self.scene_start = Instant::now();

        self.transition = Some(TransitionState::new(
            self.transition_duration,
            transition_type,
        ));
        
        // Store all pending changes to return immediately for overlapping transitions
        self.pending_changes.clear();
        if let Some(p) = pattern {
            self.pending_changes.push(("pattern".to_string(), p));
        }
        if let Some(t) = theme {
            self.pending_changes.push(("theme".to_string(), t));
        }
        if let Some(a) = art {
            self.pending_changes.push(("art".to_string(), a));
        }
    }

    /// Update the automix system
    pub fn update(&mut self, delta: f64) -> AutomixUpdate {
        let mut update = AutomixUpdate::default();
        
        // Return ALL pending changes at once for overlapping transitions
        if !self.pending_changes.is_empty() {
            // Process all pending changes
            while let Some((change_type, value)) = self.pending_changes.pop() {
                match change_type.as_str() {
                    "pattern" => update.new_pattern = Some(value),
                    "theme" => update.new_theme = Some(value),
                    "art" => update.new_art = Some(value),
                    _ => {}
                }
            }
            return update;
        }

        // Update transition if active
        let transition_complete = if let Some(ref mut transition) = self.transition {
            let complete = transition.update();
            update.transition_blend = Some(transition.blend);
            complete
        } else {
            false
        };

        if transition_complete {
            // Transition complete - update current values
            if let Some(pattern) = self.target_pattern.take() {
                self.current_pattern = pattern;
            }
            if let Some(theme) = self.target_theme.take() {
                self.current_theme = theme;
            }
            if let Some(art) = self.target_art.take() {
                self.current_art = Some(art);
            }
            self.transition = None;
        }

        // Check if current scene has expired
        if self.mode != AutomixMode::Off && self.scene_start.elapsed() >= self.scene_duration {
            match self.mode {
                AutomixMode::Random => {
                    let scene_info = self.scheduler.tick(delta as f32).map(|scene| {
                        (
                            scene.pattern_id.clone(),
                            scene.theme_name.clone(),
                            scene.duration_secs,
                        )
                    });

                    if let Some((pattern, theme, duration)) = scene_info {
                        self.start_transition(
                            Some(pattern),
                            Some(theme),
                            None,
                            Duration::from_secs_f32(duration),
                            self.default_transition,
                        );
                    }
                }
                AutomixMode::Showcase => {
                    self.showcase_index = (self.showcase_index + 1) % self.showcase_sequences.len();
                    self.apply_showcase_sequence(self.showcase_index);
                }
                AutomixMode::Playlist => {
                    if let Some(playlist) = &self.playlist {
                        if !playlist.entries.is_empty() {
                            self.playlist_index =
                                (self.playlist_index + 1) % playlist.entries.len();
                            self.apply_playlist_entry(self.playlist_index);
                        }
                    }
                }
                AutomixMode::Adaptive => {
                    // Time-based selection (could be enhanced)
                    let hour = chrono::Local::now().hour();
                    let (pattern, theme) = match hour {
                        6..=9 => ("sunrise", "warm"),
                        10..=16 => ("wave", "ocean"),
                        17..=20 => ("sunset", "autumn"),
                        _ => ("stars", "midnight"),
                    };
                    self.start_transition(
                        Some(pattern.to_string()),
                        Some(theme.to_string()),
                        None,
                        Duration::from_secs(20),
                        TransitionType::Morph,
                    );
                }
                _ => {}
            }
        }

        update.is_transitioning = self.transition.is_some();
        update.scene_progress =
            self.scene_start.elapsed().as_secs_f32() / self.scene_duration.as_secs_f32();

        update
    }

    /// Load a playlist for automix
    pub fn load_playlist(&mut self, playlist: Playlist) {
        self.playlist = Some(playlist);
        self.playlist_index = 0;
    }

    /// Get current mode
    pub fn mode(&self) -> AutomixMode {
        self.mode
    }

    /// Skip to next item
    pub fn skip_next(&mut self) {
        match self.mode {
            AutomixMode::Random => {
                self.scheduler.jump_next();
            }
            AutomixMode::Showcase => {
                self.showcase_index = (self.showcase_index + 1) % self.showcase_sequences.len();
                self.apply_showcase_sequence(self.showcase_index);
            }
            AutomixMode::Playlist => {
                if let Some(playlist) = &self.playlist {
                    if !playlist.entries.is_empty() {
                        self.playlist_index = (self.playlist_index + 1) % playlist.entries.len();
                        self.apply_playlist_entry(self.playlist_index);
                    }
                }
            }
            _ => {}
        }
    }

    /// Skip to previous item
    pub fn skip_prev(&mut self) {
        match self.mode {
            AutomixMode::Random => {
                self.scheduler.jump_prev();
            }
            AutomixMode::Showcase => {
                if self.showcase_index == 0 {
                    self.showcase_index = self.showcase_sequences.len() - 1;
                } else {
                    self.showcase_index -= 1;
                }
                self.apply_showcase_sequence(self.showcase_index);
            }
            AutomixMode::Playlist => {
                if let Some(playlist) = &self.playlist {
                    if !playlist.entries.is_empty() {
                        if self.playlist_index == 0 {
                            self.playlist_index = playlist.entries.len() - 1;
                        } else {
                            self.playlist_index -= 1;
                        }
                        self.apply_playlist_entry(self.playlist_index);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Update information from automix system
#[derive(Debug, Default)]
pub struct AutomixUpdate {
    /// New pattern to apply
    pub new_pattern: Option<String>,
    /// New theme to apply
    pub new_theme: Option<String>,
    /// New art to apply
    pub new_art: Option<String>,
    /// Current transition blend value (0.0-1.0)
    pub transition_blend: Option<f32>,
    /// Whether a transition is active
    pub is_transitioning: bool,
    /// Progress through current scene (0.0-1.0)
    pub scene_progress: f32,
}

// Easing functions for smooth transitions

fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

fn ease_out_quart(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(4)
}

impl Default for Automix {
    fn default() -> Self {
        Self::new()
    }
}
