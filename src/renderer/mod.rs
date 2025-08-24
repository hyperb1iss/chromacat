//! Terminal rendering system for ChromaCat
//!
//! This module provides functionality for rendering colored text output to the terminal,
//! handling both static and animated displays. It manages terminal state, text buffers,
//! color calculations, scrolling, and status information display.
//!
//! The rendering system is built around several key components:
//! - Terminal state management and interaction
//! - Double buffered text and color handling
//! - Pattern-based color generation
//! - Scrolling and viewport control
//! - Status bar rendering
//! - Frame timing and synchronization
//! - Playlist management and transitions

mod buffer;
mod config;
mod error;
mod scroll;
mod status_bar;
pub mod terminal;
pub mod control;
mod modulation;
mod scheduler;

pub use buffer::RenderBuffer;
pub use config::AnimationConfig;
pub use error::RendererError;
pub use scroll::{Action, ScrollState};
pub use status_bar::StatusBar;
pub use terminal::TerminalState;

use crate::pattern::PatternEngine;
use crate::playlist::{Playlist, PlaylistPlayer};
use crate::{themes, PatternConfig};
use crate::demo::art::DemoArt;
use crossterm::event::{KeyCode, MouseEvent};
use crossterm::style::{Color, SetForegroundColor, SetAttribute, Attribute, ResetColor};
use crossterm::event::KeyEvent;
use crossterm::queue;
use std::collections::HashMap;
use log::info;
use std::io::Write;
use std::time::{Duration, Instant};
use crate::input::InputReader;
use modulation::{Lfo, Modulator};
use scheduler::{Scene, SceneScheduler};
use crate::recipes::{Recipe, RouteRecipe, LfoRecipe};

/// Coordinates all rendering functionality for ChromaCat
pub struct Renderer {
    /// Pattern generation engine
    engine: PatternEngine,
    /// Animation configuration
    config: AnimationConfig,
    /// Double buffered text and colors
    buffer: RenderBuffer,
    /// Terminal state manager
    terminal: TerminalState,
    /// Scrolling state manager
    scroll: ScrollState,
    /// Status bar renderer
    status_bar: StatusBar,
    /// Available theme names
    available_themes: Vec<String>,
    /// Current theme index
    current_theme_index: usize,
    /// Available pattern IDs
    available_patterns: Vec<String>,
    /// Current pattern index
    current_pattern_index: usize,
    /// Optional secondary engine for transitions
    transition_engine: Option<PatternEngine>,
    /// Crossfade progress 0..1
    transition_alpha: f32,
    /// Whether a transition is active
    transitioning: bool,
    /// Last frame timestamp for timing
    last_frame: Option<Instant>,
    /// Frame counter for FPS calculation
    frame_count: u32,
    /// Last FPS update timestamp
    last_fps_update: Instant,
    /// Current FPS measurement
    current_fps: f64,
    /// Current playlist player if using a playlist
    playlist_player: Option<PlaylistPlayer>,
    /// Current content being displayed
    content: String,
    /// Whether running in demo mode
    demo_mode: bool,
    /// Whether the playground overlay is visible
    overlay_visible: bool,
    /// Whether the help overlay is visible
    help_visible: bool,
    /// Overlay focused section: 0=Patterns,1=Params,2=Themes
    overlay_section: u8,
    /// Overlay selections
    overlay_pattern_sel: usize,
    overlay_param_sel: usize,
    overlay_theme_sel: usize,
    /// Overlay scroll offsets to keep selection visible
    overlay_pattern_offset: usize,
    overlay_param_offset: usize,
    overlay_theme_offset: usize,
    /// Art selection and scroll
    overlay_art_sel: usize,
    overlay_art_offset: usize,
    /// Overlay param value cache (name -> value string)
    overlay_param_values: HashMap<String, String>,
    /// Demo art list (only used in demo mode)
    available_arts: Vec<String>,
    /// Simple modulator for demo routing
    modulator: Option<Modulator>,
    /// Whether modulation is enabled
    modulation_enabled: bool,
    /// Minimal pattern scheduler toggle
    scheduler_enabled: bool,
    /// Interval between automatic pattern switches
    scheduler_interval: Duration,
    /// Timestamp of last automatic switch
    scheduler_last_switch: Instant,
    /// Scene scheduler
    scene_scheduler: Option<SceneScheduler>,
    /// Overlay redraw throttle and invalidation
    overlay_dirty: bool,
    last_overlay_draw: Instant,
    overlay_refresh: Duration,
}

impl Renderer {
    /// Creates a new renderer with the given pattern engine and configuration
    pub fn new(
        engine: PatternEngine,
        config: AnimationConfig,
        playlist: Option<Playlist>,
        demo_mode: bool,
    ) -> Result<Self, RendererError> {
        let terminal = TerminalState::new()?;
        let term_size = terminal.size();
        let buffer = RenderBuffer::new(term_size);
        let scroll = ScrollState::new(term_size.1.saturating_sub(2));
        let mut status_bar = StatusBar::new(term_size);

        // Initialize available themes and patterns
        let available_themes = themes::all_themes()
            .iter()
            .map(|t| t.name.clone())
            .collect::<Vec<_>>();

        let available_patterns = crate::pattern::REGISTRY
            .list_patterns()
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        // Initialize playlist player if provided
        let playlist_player = playlist.map(PlaylistPlayer::new);

        // Get the initial engine configuration based on playlist or defaults
        let (initial_engine, initial_theme, initial_pattern) =
            if let Some(player) = &playlist_player {
                if let Some(entry) = player.current_entry() {
                    // Get configuration from first playlist entry
                    let entry_config = entry.to_pattern_config()?;
                    let entry_gradient = themes::get_theme(&entry.theme)?.create_gradient()?;

                    // Create new engine with playlist entry's configuration
                    let new_engine = PatternEngine::new(
                        entry_gradient,
                        entry_config,
                        term_size.0 as usize,
                        term_size.1 as usize,
                    );

                    (new_engine, entry.theme.clone(), entry.pattern.clone())
                } else {
                    let theme = engine
                        .config()
                        .common
                        .theme_name
                        .clone()
                        .unwrap_or_else(|| "rainbow".to_string());
                    let pattern = crate::pattern::REGISTRY
                        .get_pattern_id(&engine.config().params)
                        .unwrap_or("horizontal")
                        .to_string();
                    (engine, theme, pattern)
                }
            } else {
                let theme = engine
                    .config()
                    .common
                    .theme_name
                    .clone()
                    .unwrap_or_else(|| "rainbow".to_string());
                let pattern = crate::pattern::REGISTRY
                    .get_pattern_id(&engine.config().params)
                    .unwrap_or("horizontal")
                    .to_string();
                (engine, theme, pattern)
            };

        // Set initial theme and pattern in status bar
        status_bar.set_theme(&initial_theme);
        status_bar.set_pattern(&initial_pattern);

        // Find current indices
        let current_theme_index = available_themes
            .iter()
            .position(|t| t == &initial_theme)
            .unwrap_or(0);

        let current_pattern_index = available_patterns
            .iter()
            .position(|p| p == &initial_pattern)
            .unwrap_or(0);

        // Initialize timing state
        let now = Instant::now();
        let fps = config.fps as f64;

        Ok(Self {
            engine: initial_engine,
            config,
            buffer,
            terminal,
            scroll,
            status_bar,
            available_themes,
            current_theme_index,
            available_patterns,
            current_pattern_index,
            transition_engine: None,
            transition_alpha: 0.0,
            transitioning: false,
            last_frame: None,
            frame_count: 0,
            last_fps_update: now,
            current_fps: fps,
            playlist_player,
            content: String::new(),
            demo_mode,
            overlay_visible: false,
            help_visible: false,
            overlay_section: 0,
            overlay_pattern_sel: current_pattern_index,
            overlay_param_sel: 0,
            overlay_theme_sel: current_theme_index,
            overlay_pattern_offset: 0,
            overlay_param_offset: 0,
            overlay_theme_offset: 0,
            overlay_param_values: HashMap::new(),
            available_arts: if demo_mode { DemoArt::all_types().iter().map(|a| a.as_str().to_string()).collect() } else { Vec::new() },
            overlay_art_sel: 0,
            overlay_art_offset: 0,
            modulator: None,
            modulation_enabled: false,
            scheduler_enabled: false,
            scheduler_interval: Duration::from_secs(20),
            scheduler_last_switch: now,
            scene_scheduler: None,
            overlay_dirty: true,
            last_overlay_draw: now,
            overlay_refresh: Duration::from_millis(33),
        })
    }

    /// Renders the overlay with three columns: patterns, params, themes
    fn render_overlay(&self, stdout: &mut std::io::StdoutLock) -> Result<(), RendererError> {
        let size = self.terminal.size();
        let width = size.0 as usize;
        let height_u16 = size.1;
        let overlay_height = ((height_u16 / 2).max(10)).min(height_u16.saturating_sub(3));
        let start_y = height_u16.saturating_sub(overlay_height + 2);

        for i in 0..overlay_height as usize {
            queue!(stdout, crossterm::cursor::MoveTo(0, start_y + i as u16), crossterm::style::Print("\x1b[K"))?;
        }
        let sep_line = "─".repeat(width.min(240));
        queue!(stdout, crossterm::cursor::MoveTo(0, start_y), crossterm::style::Print(&sep_line))?;

        let titles_y = start_y + 1;
        let colw = (width / 4).max(18);
        let x_pat = 0u16;
        let x_prm = colw as u16;
        let x_thm = (colw * 2) as u16;
        let x_mod = (colw * 3) as u16;
        // Colored titles (SilkCircuit palette)
        let mut print_title = |x: u16, text: &str, color: u8, selected: bool| -> Result<(), RendererError> {
            let pad = format!("{:<w$}", text, w=colw);
            if selected {
                queue!(stdout, crossterm::cursor::MoveTo(x, titles_y), SetAttribute(Attribute::Bold), SetForegroundColor(Color::AnsiValue(color)), crossterm::style::Print(&pad), ResetColor, SetAttribute(Attribute::Reset))?;
            } else {
                queue!(stdout, crossterm::cursor::MoveTo(x, titles_y), SetForegroundColor(Color::AnsiValue(color)), crossterm::style::Print(&pad), ResetColor)?;
            }
            Ok(())
        };

        print_title(x_pat, if self.overlay_section == 0 { "▶ Patterns" } else { "  Patterns" }, 219, self.overlay_section == 0)?;
        print_title(x_prm, if self.overlay_section == 1 { "▶ Params" } else { "  Params" }, 117, self.overlay_section == 1)?;
        print_title(x_thm, if self.overlay_section == 2 { "▶ Themes" } else { "  Themes" }, 147, self.overlay_section == 2)?;
        print_title(x_mod, if self.overlay_section == 3 { "▶ Art" } else { "  Art" }, 149, self.overlay_section == 3)?;

        let list_rows = overlay_height.saturating_sub(3) as usize;
        let pat_start = self.overlay_pattern_offset;
        for i in 0..list_rows {
            let row_y = titles_y + 1 + i as u16;
            let idx = pat_start + i;
            let name = self.available_patterns.get(idx).map(|s| s.as_str()).unwrap_or("");
            let is_sel = idx == self.overlay_pattern_sel;
            let line = format!(" {} {:<w$}", if is_sel {"➤"} else {" "}, name, w=colw.saturating_sub(3));
            let printed = if is_sel { format!("\x1b[7m{}\x1b[0m", line) } else { line };
            queue!(stdout, crossterm::cursor::MoveTo(x_pat, row_y), crossterm::style::Print(printed))?;
        }

        let param_names = self.current_param_names();
        let prm_start = self.overlay_param_offset;
        for i in 0..list_rows {
            let row_y = titles_y + 1 + i as u16;
            let idx = prm_start + i;
            let name = param_names.get(idx).map(|s| s.as_str()).unwrap_or("");
            let is_sel = idx == self.overlay_param_sel;
            // draw value slider if numeric
            let mut val_str = String::new();
            if let Some((n, ptype)) = self.param_meta_at(idx) {
                match ptype {
                    crate::pattern::ParamType::Number { min, max } => {
                        let cur = self.overlay_param_values.get(&n).and_then(|s| s.parse::<f64>().ok()).unwrap_or_else(|| self.param_default_value(&n).and_then(|v| v.parse::<f64>().ok()).unwrap_or(min));
                        let t = ((cur - min) / (max - min)).clamp(0.0, 1.0);
                        let bars = (t * 10.0).round() as usize;
                        val_str = format!(" [{}{}] {:.2}", "█".repeat(bars), "░".repeat(10 - bars), cur);
                    }
                    _ => {}
                }
            }
            let name_w = colw.saturating_sub(14);
            let line = format!(" {} {:<nw$}{}", if is_sel {"➤"} else {" "}, name, val_str, nw=name_w);
            let printed = if is_sel { format!("\x1b[7m{}\x1b[0m", line) } else { line };
            queue!(stdout, crossterm::cursor::MoveTo(x_prm, row_y), crossterm::style::Print(printed))?;
        }

        let thm_start = self.overlay_theme_offset;
        for i in 0..list_rows {
            let row_y = titles_y + 1 + i as u16;
            let idx = thm_start + i;
            let name = self.available_themes.get(idx).map(|s| s.as_str()).unwrap_or("");
            let is_sel = idx == self.overlay_theme_sel;
            let line = format!(" {} {:<w$}", if is_sel {"➤"} else {" "}, name, w=colw.saturating_sub(3));
            let printed = if is_sel { format!("\x1b[7m{}\x1b[0m", line) } else { line };
            queue!(stdout, crossterm::cursor::MoveTo(x_thm, row_y), crossterm::style::Print(printed))?;
        }

        // Art column (demo)
        let art_x = x_mod;
        if self.demo_mode && !self.available_arts.is_empty() {
            let start = self.overlay_art_offset;
            for i in 0..list_rows {
                let row_y = titles_y + 1 + i as u16;
                let idx = start + i;
                let is_sel = idx == self.overlay_art_sel;
                let name = self.available_arts.get(idx).map(|s| s.as_str()).unwrap_or("");
                let line = format!(" {} {:<w$}", if is_sel {"➤"} else {" "}, name, w=colw.saturating_sub(3));
                let printed = if is_sel { format!("\x1b[7m{}\x1b[0m", line) } else { line };
                queue!(stdout, crossterm::cursor::MoveTo(art_x, row_y), crossterm::style::Print(printed))?;
            }
        } else {
            queue!(stdout, crossterm::cursor::MoveTo(art_x, titles_y + 1), crossterm::style::Print(" (demo only)"))?;
        }

        let footer = "[Click] select/apply  [Wheel] scroll  [Tab] section  [Enter] apply  [-/=] adjust  [b] bind LFO  [m] mod  [S] schedule  [R/L] save/load  [E] export  [;] overlay  [?] help";
        let mut footer_trunc = footer.to_string();
        let maxw = width.saturating_sub(1);
        if footer_trunc.len() > maxw { footer_trunc.truncate(maxw); }
        queue!(stdout, crossterm::cursor::MoveTo(0, start_y + overlay_height - 1), crossterm::style::Print("\x1b[K"), SetForegroundColor(Color::AnsiValue(239)), crossterm::style::Print(footer_trunc), ResetColor)?;

        // Help modal overlay
        if self.help_visible {
            self.render_help_modal(stdout)?;
        }
        Ok(())
    }

    fn current_param_names(&self) -> Vec<String> {
        let current_id = crate::pattern::REGISTRY.get_pattern_id(&self.engine.config().params).unwrap_or("horizontal");
        if let Some(meta) = crate::pattern::REGISTRY.get_pattern(current_id) {
            let subs = meta.params().sub_params();
            subs.into_iter().map(|p| p.name().to_string()).collect()
        } else { Vec::new() }
    }

    fn handle_overlay_key(&mut self, key: KeyEvent) -> Result<bool, RendererError> {
        // Compute visible rows for scrolling logic
        let term_h = self.terminal.size().1;
        let overlay_height = ((term_h / 2).max(10)).min(term_h.saturating_sub(3));
        let visible_rows = overlay_height.saturating_sub(3) as usize;
        match key.code {
            KeyCode::Tab => { self.overlay_section = (self.overlay_section + 1) % 4; self.overlay_dirty = true; self.redraw_overlay_only()?; return Ok(true); }
            KeyCode::Up => { match self.overlay_section { 0 => { if self.overlay_pattern_sel > 0 { self.overlay_pattern_sel -= 1; if self.overlay_pattern_sel < self.overlay_pattern_offset { self.overlay_pattern_offset = self.overlay_pattern_sel; } } }
                1 => { if self.overlay_param_sel > 0 { self.overlay_param_sel -= 1; if self.overlay_param_sel < self.overlay_param_offset { self.overlay_param_offset = self.overlay_param_sel; } } }
                2 => { if self.overlay_theme_sel > 0 { self.overlay_theme_sel -= 1; if self.overlay_theme_sel < self.overlay_theme_offset { self.overlay_theme_offset = self.overlay_theme_sel; } } }
                3 => { if self.overlay_art_sel > 0 { self.overlay_art_sel -= 1; if self.overlay_art_sel < self.overlay_art_offset { self.overlay_art_offset = self.overlay_art_sel; } } }
                _ => {} } self.overlay_dirty = true; self.redraw_overlay_only()?; return Ok(true); }
            KeyCode::Down => { match self.overlay_section { 0 => { if self.overlay_pattern_sel + 1 < self.available_patterns.len() { self.overlay_pattern_sel += 1; let vis = self.overlay_pattern_sel - self.overlay_pattern_offset; if vis >= visible_rows { self.overlay_pattern_offset += 1; } } }
                1 => { let max = self.current_param_names().len(); if self.overlay_param_sel + 1 < max { self.overlay_param_sel += 1; let vis = self.overlay_param_sel - self.overlay_param_offset; if vis >= visible_rows { self.overlay_param_offset += 1; } } }
                2 => { if self.overlay_theme_sel + 1 < self.available_themes.len() { self.overlay_theme_sel += 1; let vis = self.overlay_theme_sel - self.overlay_theme_offset; if vis >= visible_rows { self.overlay_theme_offset += 1; } } }
                3 => { let max = self.available_arts.len(); if self.overlay_art_sel + 1 < max { self.overlay_art_sel += 1; let vis = self.overlay_art_sel - self.overlay_art_offset; if vis >= visible_rows { self.overlay_art_offset += 1; } } }
                _ => {} } self.overlay_dirty = true; self.redraw_overlay_only()?; return Ok(true); }
            KeyCode::PageUp => { match self.overlay_section { 0 => { let step = visible_rows.max(1); self.overlay_pattern_sel = self.overlay_pattern_sel.saturating_sub(step); self.overlay_pattern_offset = self.overlay_pattern_offset.saturating_sub(step); }
                1 => { let step = visible_rows.max(1); self.overlay_param_sel = self.overlay_param_sel.saturating_sub(step); self.overlay_param_offset = self.overlay_param_offset.saturating_sub(step); }
                2 => { let step = visible_rows.max(1); self.overlay_theme_sel = self.overlay_theme_sel.saturating_sub(step); self.overlay_theme_offset = self.overlay_theme_offset.saturating_sub(step); }
                3 => { let step = visible_rows.max(1); self.overlay_art_sel = self.overlay_art_sel.saturating_sub(step); self.overlay_art_offset = self.overlay_art_offset.saturating_sub(step); }
                _ => {} } self.overlay_dirty = true; self.redraw_overlay_only()?; return Ok(true); }
            KeyCode::PageDown => { match self.overlay_section { 0 => { let step = visible_rows.max(1); self.overlay_pattern_sel = (self.overlay_pattern_sel + step).min(self.available_patterns.len().saturating_sub(1)); self.overlay_pattern_offset = (self.overlay_pattern_offset + step).min(self.available_patterns.len().saturating_sub(visible_rows)); }
                1 => { let step = visible_rows.max(1); let max = self.current_param_names().len(); self.overlay_param_sel = (self.overlay_param_sel + step).min(max.saturating_sub(1)); self.overlay_param_offset = (self.overlay_param_offset + step).min(max.saturating_sub(visible_rows)); }
                2 => { let step = visible_rows.max(1); self.overlay_theme_sel = (self.overlay_theme_sel + step).min(self.available_themes.len().saturating_sub(1)); self.overlay_theme_offset = (self.overlay_theme_offset + step).min(self.available_themes.len().saturating_sub(visible_rows)); }
                3 => { let step = visible_rows.max(1); let max = self.available_arts.len(); self.overlay_art_sel = (self.overlay_art_sel + step).min(max.saturating_sub(1)); self.overlay_art_offset = (self.overlay_art_offset + step).min(max.saturating_sub(visible_rows)); }
                _ => {} } self.overlay_dirty = true; self.redraw_overlay_only()?; return Ok(true); }
            KeyCode::Char('g') => { match self.overlay_section { 0 => { self.overlay_pattern_sel = 0; self.overlay_pattern_offset = 0; }
                1 => { self.overlay_param_sel = 0; self.overlay_param_offset = 0; }
                2 => { self.overlay_theme_sel = 0; self.overlay_theme_offset = 0; }
                3 => { self.overlay_art_sel = 0; self.overlay_art_offset = 0; }
                _ => {} } self.overlay_dirty = true; self.redraw_overlay_only()?; return Ok(true); }
            KeyCode::Char('G') => { match self.overlay_section { 0 => { if !self.available_patterns.is_empty() { self.overlay_pattern_sel = self.available_patterns.len() - 1; self.overlay_pattern_offset = self.overlay_pattern_sel.saturating_sub(visible_rows.saturating_sub(1)); } }
                1 => { let max = self.current_param_names().len(); if max > 0 { self.overlay_param_sel = max - 1; self.overlay_param_offset = self.overlay_param_sel.saturating_sub(visible_rows.saturating_sub(1)); } }
                2 => { if !self.available_themes.is_empty() { self.overlay_theme_sel = self.available_themes.len() - 1; self.overlay_theme_offset = self.overlay_theme_sel.saturating_sub(visible_rows.saturating_sub(1)); } }
                3 => { let max = self.available_arts.len(); if max > 0 { self.overlay_art_sel = max - 1; self.overlay_art_offset = self.overlay_art_sel.saturating_sub(visible_rows.saturating_sub(1)); } }
                _ => {} } self.overlay_dirty = true; self.redraw_overlay_only()?; return Ok(true); }
            // Modulation editing hotkeys
            KeyCode::Char('x') => {
                if self.overlay_section == 3 {
                    if let Some(m) = self.modulator.as_mut() {
                        if self.overlay_art_sel < m.routes.len() { m.routes.remove(self.overlay_art_sel); }
                        let max = m.routes.len();
                        if self.overlay_art_sel >= max { self.overlay_art_sel = max.saturating_sub(1); }
                        self.draw_full_screen()?;
                    }
                }
            }
            KeyCode::Char('F') | KeyCode::Char('f') => {
                if self.overlay_section == 3 {
                    let delta = if matches!(key.code, KeyCode::Char('F')) { 0.05 } else { -0.05 };
                    if let Some(m) = self.modulator.as_mut() {
                        if let Some(r) = m.routes.get(self.overlay_art_sel).cloned() {
                            if let Some(lfo) = m.lfos.get_mut(r.source_index) { lfo.frequency_hz = (lfo.frequency_hz + delta).max(0.01); }
                            self.draw_full_screen()?;
                        }
                    }
                }
            }
            KeyCode::Char('D') | KeyCode::Char('d') => {
                if self.overlay_section == 3 {
                    let delta = if matches!(key.code, KeyCode::Char('D')) { 0.1 } else { -0.1 };
                    if let Some(m) = self.modulator.as_mut() {
                        if let Some(r) = m.routes.get_mut(self.overlay_art_sel) { r.depth = (r.depth + delta).clamp(0.0, 4.0); }
                        self.draw_full_screen()?;
                    }
                }
            }
            KeyCode::Enter => { match self.overlay_section { 0 => { if let Some(p) = self.available_patterns.get(self.overlay_pattern_sel) { let pat = p.clone(); self.set_pattern_by_id(&pat)?; self.overlay_param_sel = 0; } }
                2 => { if let Some(t) = self.available_themes.get(self.overlay_theme_sel) { let theme = t.clone(); self.set_theme_by_name(&theme)?; } }
                _ => {} } self.draw_full_screen()?; return Ok(true); }
            KeyCode::Char('b') => {
                // Bind current param to first LFO route
                if let Some((name, ptype)) = self.param_meta_at(self.overlay_param_sel) {
                    if let crate::pattern::ParamType::Number { min, max } = ptype {
                        if self.modulator.is_none() {
                            let mut m = Modulator::new();
                            let idx = m.add_lfo(Lfo::new(0.15, min, max));
                            m.add_route(name, idx, 1.0);
                            self.modulator = Some(m);
                            self.modulation_enabled = true;
                        } else if let Some(m) = self.modulator.as_mut() {
                            let idx = if m.lfos.is_empty() { m.add_lfo(Lfo::new(0.15, min, max)) } else { 0 };
                            m.add_route(name, idx, 1.0);
                            self.modulation_enabled = true;
                        }
                        self.status_bar.set_custom_text(Some("Bound LFO → param"));
                        self.draw_full_screen()?;
                        return Ok(true);
                    }
                }
            }
            KeyCode::Char('-') | KeyCode::Char('=') => {
                if self.overlay_section == 1 {
                    // Adjust numeric param by small step
                    let step = if matches!(key.code, KeyCode::Char('=')) { 0.05 } else { -0.05 };
                    if let Some((name, ptype)) = self.param_meta_at(self.overlay_param_sel) {
                        if let crate::pattern::ParamType::Number { min, max } = ptype {
                            let current = self.overlay_param_values
                                .get(&name)
                                .and_then(|s| s.parse::<f64>().ok())
                                .unwrap_or_else(|| {
                                    // try default value from metadata
                                    self.param_default_value(&name).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.5)
                                });
                            let new_value = (current + step).clamp(min, max);
                            let csv = format!("{}={}", name, new_value);
                            self.update_params_from_str(&csv)?;
                            self.overlay_param_values.insert(name, format!("{new_value:.3}"));
                            self.draw_full_screen()?;
                            return Ok(true);
                        }
                    }
                }
            }
            KeyCode::Char(' ') => {
                if self.overlay_section == 1 {
                    if let Some((name, ptype)) = self.param_meta_at(self.overlay_param_sel) {
                        match ptype {
                            crate::pattern::ParamType::Boolean => {
                                // Toggle
                                let current = self.overlay_param_values.get(&name).map(|s| s == "true").unwrap_or(false);
                                let new_val = if current { "false" } else { "true" };
                                let csv = format!("{}={}", name, new_val);
                                self.update_params_from_str(&csv)?;
                                self.overlay_param_values.insert(name, new_val.to_string());
                                self.draw_full_screen()?;
                                return Ok(true);
                            }
                            _ => {}
                        }
                    }
                }
            }
            KeyCode::Left | KeyCode::Right => {
                if self.overlay_section == 1 {
                    if let Some((name, ptype)) = self.param_meta_at(self.overlay_param_sel) {
                        if let crate::pattern::ParamType::Enum { options } = ptype {
                            if options.is_empty() { return Ok(true); }
                            let cur_index = self.overlay_param_values.get(&name)
                                .and_then(|s| options.iter().position(|o| o == s))
                                .unwrap_or(0);
                            let next = match key.code { KeyCode::Left => cur_index.saturating_sub(1), KeyCode::Right => (cur_index + 1) % options.len(), _ => cur_index };
                            let value = options[next];
                            let csv = format!("{}={}", name, value);
                            self.update_params_from_str(&csv)?;
                            self.overlay_param_values.insert(name, value.to_string());
                            self.draw_full_screen()?;
                            return Ok(true);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn param_meta_at(&self, idx: usize) -> Option<(String, crate::pattern::ParamType)> {
        let current_id = crate::pattern::REGISTRY.get_pattern_id(&self.engine.config().params)?;
        let meta = crate::pattern::REGISTRY.get_pattern(current_id)?;
        let subs = meta.params().sub_params();
        if idx >= subs.len() { return None; }
        let p = &subs[idx];
        Some((p.name().to_string(), p.param_type()))
    }

    fn param_default_value(&self, name: &str) -> Option<String> {
        let current_id = crate::pattern::REGISTRY.get_pattern_id(&self.engine.config().params)?;
        let meta = crate::pattern::REGISTRY.get_pattern(current_id)?;
        let subs = meta.params().sub_params();
        for p in subs {
            if p.name() == name { return Some(p.default_value()); }
        }
        None
    }

    fn first_numeric_param_range(&self) -> Option<(String, f64, f64)> {
        let current_id = crate::pattern::REGISTRY.get_pattern_id(&self.engine.config().params)?;
        let meta = crate::pattern::REGISTRY.get_pattern(current_id)?;
        for p in meta.params().sub_params() {
            if let crate::pattern::ParamType::Number { min, max } = p.param_type() {
                return Some((p.name().to_string(), min, max));
            }
        }
        None
    }

    fn render_help_modal(&self, stdout: &mut std::io::StdoutLock) -> Result<(), RendererError> {
        let size = self.terminal.size();
        let w = size.0 as usize;
        let h = size.1 as usize;
        let bw = w.min(64);
        let bh = h.min(12);
        let x0 = ((w - bw) / 2) as u16;
        let y0 = ((h - bh) / 2) as u16;

        // Draw border box
        let top = format!("╔{}╗", "═".repeat(bw.saturating_sub(2)));
        let mid = format!("║{}║", " ".repeat(bw.saturating_sub(2)));
        let bot = format!("╚{}╝", "═".repeat(bw.saturating_sub(2)));
        queue!(stdout, crossterm::cursor::MoveTo(x0, y0), crossterm::style::Print(&top))?;
        for i in 1..(bh.saturating_sub(1)) {
            queue!(stdout, crossterm::cursor::MoveTo(x0, y0 + i as u16), crossterm::style::Print(&mid))?;
        }
        queue!(stdout, crossterm::cursor::MoveTo(x0, y0 + (bh.saturating_sub(1)) as u16), crossterm::style::Print(&bot))?;

        // Title and content
        let write_line = |s: &str, off: u16, out: &mut std::io::StdoutLock| -> Result<(), RendererError> {
            let mut line = s.to_string();
            if line.len() > bw.saturating_sub(4) { line.truncate(bw.saturating_sub(4)); }
            queue!(out, crossterm::cursor::MoveTo(x0 + 2, y0 + off), crossterm::style::Print(&line))?;
            Ok(())
        };
        write_line("ChromaCat Help ▸ keys", 1, stdout)?;
        write_line("; overlay  |  Tab switch  |  ↑/↓ move", 3, stdout)?;
        write_line("Enter apply  |  -= adjust  |  b bind LFO", 4, stdout)?;
        write_line("m toggle modulation  |  S scene scheduler", 5, stdout)?;
        write_line("R save recipe  |  L load recipe", 6, stdout)?;
        write_line("t next theme  |  p next pattern  |  q quit", 7, stdout)?;
        write_line("Press ? to close", 9, stdout)?;
        Ok(())
    }

    /// Public: show or hide the overlay
    pub fn set_overlay_visible(&mut self, visible: bool) {
        self.overlay_visible = visible;
    }

    /// Public: set a status message in the status bar
    pub fn set_status_message(&mut self, msg: &str) {
        self.status_bar.set_custom_text(Some(msg));
    }

    /// Public: enable scene scheduler with a default seed of two scenes
    pub fn enable_default_scenes(&mut self) {
        if self.scene_scheduler.is_none() {
            let current_id = crate::pattern::REGISTRY
                .get_pattern_id(&self.engine.config().params)
                .unwrap_or("horizontal")
                .to_string();
            let current_theme = self
                .available_themes
                .get(self.current_theme_index)
                .cloned()
                .unwrap_or_else(|| "rainbow".to_string());
            let alt_index = (self.current_pattern_index + 1) % self.available_patterns.len();
            let alt_pattern = self
                .available_patterns
                .get(alt_index)
                .cloned()
                .unwrap_or_else(|| current_id.clone());
            self.scene_scheduler = Some(SceneScheduler::new(vec![
                Scene { pattern_id: current_id, theme_name: current_theme, duration_secs: 12.0 },
                Scene { pattern_id: alt_pattern, theme_name: "rainbow".to_string(), duration_secs: 12.0 },
            ]));
        }
        if let Some(s) = self.scene_scheduler.as_mut() { s.set_enabled(true); }
    }

    fn create_recipe_snapshot(&self) -> Recipe {
        let current_id = crate::pattern::REGISTRY.get_pattern_id(&self.engine.config().params).map(|s| s.to_string());
        let current_theme = self.available_themes.get(self.current_theme_index).cloned();
        let mut routes = Vec::new();
        if let Some(m) = &self.modulator {
            for r in &m.routes {
                if let Some(lfo) = m.lfos.get(r.source_index) {
                    routes.push(RouteRecipe { target_param: r.target_param.clone(), lfo: LfoRecipe { frequency_hz: lfo.frequency_hz, min: lfo.min, max: lfo.max, depth: r.depth } });
                }
            }
        }
        let scenes = if let Some(s) = &self.scene_scheduler { s.clone().into() } else { Vec::new() };
        Recipe { current_theme, current_pattern: current_id, scenes, routes }
    }

    fn apply_recipe(&mut self, recipe: Recipe) -> Result<(), RendererError> {
        if let Some(t) = recipe.current_theme.as_ref() { let _ = self.set_theme_by_name(t); }
        if let Some(p) = recipe.current_pattern.as_ref() { let _ = self.set_pattern_by_id(p); }
        // Scenes
        if !recipe.scenes.is_empty() {
            let scenes = recipe.scenes.into_iter().map(|s| Scene { pattern_id: s.pattern_id, theme_name: s.theme_name, duration_secs: s.duration_secs }).collect::<Vec<_>>() ;
            self.scene_scheduler = Some(SceneScheduler::new(scenes));
        }
        // Routes
        if !recipe.routes.is_empty() {
            let mut m = Modulator::new();
            for r in recipe.routes {
                let idx = m.add_lfo(Lfo { frequency_hz: r.lfo.frequency_hz, phase: 0.0, min: r.lfo.min, max: r.lfo.max });
                m.add_route(r.target_param, idx, r.lfo.depth);
            }
            self.modulator = Some(m);
            self.modulation_enabled = true;
        }
        Ok(())
    }
    /// Returns the frame duration based on configured FPS
    #[inline]
    pub fn frame_duration(&self) -> Duration {
        self.config.frame_duration()
    }

    /// Returns whether animation is set to run indefinitely
    #[inline]
    pub fn is_infinite(&self) -> bool {
        self.config.infinite
    }

    /// Returns the configured animation cycle duration
    #[inline]
    pub fn cycle_duration(&self) -> Duration {
        self.config.cycle_duration
    }

    /// Renders static text with pattern-based colors
    pub fn render_static(&mut self, text: &str) -> Result<(), RendererError> {
        // Prepare the full content
        self.buffer.prepare_text(text)?;

        // Update colors
        self.buffer.update_colors_static(&self.engine)?;

        // Get a stdout lock for efficient writing
        let mut stdout = self.terminal.stdout();

        // Render the entire buffer content
        self.buffer.render_region(
            &mut stdout,
            0,
            self.buffer.total_lines(),
            self.terminal.colors_enabled(),
            false,
        )?;

        stdout.flush()?;
        Ok(())
    }

    /// Sets the current theme by name
    pub fn set_theme_by_name(&mut self, theme: &str) -> Result<(), RendererError> {
        let new_gradient = themes::get_theme(theme)?.create_gradient()?;
        self.engine.update_gradient(new_gradient);
        if let Some(idx) = self
            .available_themes
            .iter()
            .position(|t| t == theme)
        {
            self.current_theme_index = idx;
        }
        self.status_bar.set_theme(theme);
        Ok(())
    }

    /// Sets the current pattern by ID and resets params to defaults for that pattern
    pub fn set_pattern_by_id(&mut self, pattern_id: &str) -> Result<(), RendererError> {
        let new_params = crate::pattern::REGISTRY
            .create_pattern_params(pattern_id)
            .ok_or_else(|| RendererError::InvalidPattern(pattern_id.to_string()))?;

        let new_config = PatternConfig {
            common: self.engine.config().common.clone(),
            params: new_params,
        };

        self.engine.update_pattern_config(new_config);
        if let Some(idx) = self
            .available_patterns
            .iter()
            .position(|p| p == pattern_id)
        {
            self.current_pattern_index = idx;
        }
        self.status_bar.set_pattern(pattern_id);
        Ok(())
    }

    /// Updates current pattern parameters from a `key=value[,key=value]` string
    pub fn update_params_from_str(&mut self, params_csv: &str) -> Result<(), RendererError> {
        // Determine current pattern id
        let current_id = crate::pattern::REGISTRY
            .get_pattern_id(&self.engine.config().params)
            .unwrap_or("horizontal");

        let updated_params = crate::pattern::REGISTRY
            .parse_params(current_id, params_csv)
            .map_err(RendererError::InvalidParams)?;

        let new_config = PatternConfig {
            common: self.engine.config().common.clone(),
            params: updated_params,
        };
        self.engine.update_pattern_config(new_config);
        Ok(())
    }

    /// Renders a single animation frame
    pub fn render_frame(&mut self, text: &str, delta_seconds: f64) -> Result<(), RendererError> {
        let frame_time = Duration::from_secs_f64(delta_seconds);

        // Handle playlist updates if active
        let needs_update = if let Some(player) = &mut self.playlist_player {
            info!(
                "Updating playlist: current_entry={:?}, time={:?}",
                player.current_entry().map(|e| &e.pattern),
                frame_time
            );
            player.update(frame_time)
        } else {
            false
        };

        if needs_update {
            info!("Playlist entry changed, updating configuration");
            self.update_playlist_entry()?;
        }

        // Update playlist status display
        if let Some(player) = &self.playlist_player {
            if let Some(entry) = player.current_entry() {
                let status = if player.is_paused() {
                    "Paused"
                } else {
                    "Playing"
                };
                self.status_bar.set_custom_text(Some(&format!(
                    "{} - {} [{:.0}%]",
                    status,
                    entry.name,
                    player.current_progress() * 100.0
                )));
            }
        }

        // First-time initialization
        if !self.buffer.has_content() {
            self.terminal.enter_alternate_screen()?;
            self.buffer.prepare_text(text)?;
            self.scroll.set_total_lines(self.buffer.line_count());
            let visible_range = self.scroll.get_visible_range();
            self.buffer.update_colors(&self.engine, visible_range.0)?;
            self.draw_full_screen()?;
            self.last_frame = Some(Instant::now());
            self.last_fps_update = Instant::now();
            return Ok(());
        }

        // Update pattern animation
        self.engine.update(delta_seconds);

        // Apply demo modulation if enabled
        if self.modulation_enabled {
            // Initialize a simple modulator lazily
            if self.modulator.is_none() {
                if let Some((name, min, max)) = self.first_numeric_param_range() {
                    let mut m = Modulator::new();
                    let lfo_idx = m.add_lfo(Lfo::new(0.15, min, max));
                    m.add_route(name, lfo_idx, 1.0);
                    self.modulator = Some(m);
                }
            }
            if let Some(modulator) = self.modulator.as_mut() {
                let updates = modulator.advance(delta_seconds);
                if !updates.is_empty() {
                    // Apply first update for now
                    let (name, value) = &updates[0];
                    let _ = self.update_params_from_str(&format!("{}={}", name, value));
                }
            }
        }

        // Update colors and render
        let visible_range = self.scroll.get_visible_range();
        // Update colors, possibly blending during transitions
        if self.transitioning {
            if let Some(trans) = &self.transition_engine {
                self.buffer.update_colors_blend(&self.engine, trans, self.transition_alpha, visible_range.0)?;
            } else {
                self.buffer.update_colors(&self.engine, visible_range.0)?;
            }
        } else {
        self.buffer.update_colors(&self.engine, visible_range.0)?;
        }

        let mut stdout = self.terminal.stdout();
        self.buffer.render_region(
            &mut stdout,
            visible_range.0,
            visible_range.1,
            self.terminal.colors_enabled(),
            true,
        )?;

        // Update FPS counter
        self.frame_count += 1;
        let now = Instant::now();
        if now.duration_since(self.last_fps_update) >= Duration::from_secs(1) {
            self.current_fps = self.frame_count as f64;
            self.frame_count = 0;
            self.last_fps_update = now;
            self.status_bar.set_fps(self.current_fps);
        }

        // Progress transition
        if self.transitioning {
            self.transition_alpha += (delta_seconds as f32) / 1.0; // 1s crossfade
            if self.transition_alpha >= 1.0 {
                // Finish transition: swap to new engine
                if let Some(new_engine) = self.transition_engine.take() {
                    self.engine = new_engine;
                }
                self.transition_alpha = 0.0;
                self.transitioning = false;
            }
        }

        // Update status bar
        self.status_bar.render(&mut stdout, &self.scroll)?;

        // Render overlay if enabled (always draw after content to avoid flicker)
        if self.overlay_visible {
            self.render_overlay(&mut stdout)?;
            self.last_overlay_draw = Instant::now();
            self.overlay_dirty = false;
        }

        stdout.flush()?;
        drop(stdout);

        // Scene scheduler
        if let Some(next) = self
            .scene_scheduler
            .as_mut()
            .and_then(|s| if s.is_enabled() { s.tick(delta_seconds as f32).cloned() } else { None })
        {
            // Apply theme outside of scheduler borrow
            self.set_theme_by_name(&next.theme_name)?;
            if let Some(params) = crate::pattern::REGISTRY.create_pattern_params(&next.pattern_id) {
                let mut new_engine = self.engine.clone();
                new_engine.update_pattern_config(PatternConfig { common: self.engine.config().common.clone(), params });
                self.transition_engine = Some(new_engine);
                self.transition_alpha = 0.0;
                self.transitioning = true;
                self.status_bar.set_pattern(&next.pattern_id);
            }
        } else if self.scheduler_enabled && now.duration_since(self.scheduler_last_switch) >= self.scheduler_interval {
            // Fallback: time-interval scheduler if no scene scheduler configured
            self.next_pattern()?;
            self.scheduler_last_switch = now;
        }

        self.last_frame = Some(now);

        Ok(())
    }

    /// Handles terminal resize events
    pub fn handle_resize(&mut self, new_width: u16, new_height: u16) -> Result<(), RendererError> {
        self.terminal.resize(new_width, new_height)?;
        self.scroll.update_viewport(new_height.saturating_sub(2));
        self.buffer.resize((new_width, new_height))?;
        self.status_bar.resize((new_width, new_height));
        self.scroll.validate_viewport();
        self.draw_full_screen()?;
        Ok(())
    }

    /// Handles keyboard input events
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool, RendererError> {
        if self.overlay_visible {
            if self.handle_overlay_key(key)? { return Ok(true); }
        }
        match key.code {
            KeyCode::Char(';') => {
                self.overlay_visible = !self.overlay_visible;
                self.draw_full_screen()?;
                Ok(true)
            }
            KeyCode::Char('?') => {
                self.help_visible = !self.help_visible;
                if self.help_visible { self.status_bar.set_custom_text(Some("? Help: ; overlay | Tab navigate | Enter apply | -= adjust | b bind | m mod | S scenes | R save | L load | q quit")); }
                self.draw_full_screen()?;
                Ok(true)
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                self.next_theme()?;
                self.draw_full_screen()?;
                Ok(true)
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.next_pattern()?;
                self.draw_full_screen()?;
                Ok(true)
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                if self.scene_scheduler.is_none() {
                    // Seed a small scene list using current pattern/theme
                    let current_id = crate::pattern::REGISTRY.get_pattern_id(&self.engine.config().params).unwrap_or("horizontal").to_string();
                    let current_theme = self.available_themes.get(self.current_theme_index).cloned().unwrap_or_else(|| "rainbow".to_string());
                    let alt_index = (self.current_pattern_index + 1) % self.available_patterns.len();
                    let alt_pattern = self.available_patterns.get(alt_index).cloned().unwrap_or_else(|| current_id.clone());
                    self.scene_scheduler = Some(SceneScheduler::new(vec![
                        Scene { pattern_id: current_id, theme_name: current_theme, duration_secs: 12.0 },
                        Scene { pattern_id: alt_pattern, theme_name: "rainbow".to_string(), duration_secs: 12.0 },
                    ]));
                }
                let enabled = if let Some(s) = self.scene_scheduler.as_mut() { let en = !s.is_enabled(); s.set_enabled(en); en } else { false };
                let msg = if enabled { "Scene Scheduler: ON" } else { "Scene Scheduler: OFF" };
                self.status_bar.set_custom_text(Some(msg));
                self.draw_full_screen()?;
                Ok(true)
            }
            KeyCode::Char('m') | KeyCode::Char('M') => {
                self.modulation_enabled = !self.modulation_enabled;
                if !self.modulation_enabled { self.modulator = None; }
                self.status_bar.set_custom_text(Some(if self.modulation_enabled { "Modulation: ON" } else { "Modulation: OFF" }));
                self.draw_full_screen()?;
                Ok(true)
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                // Save recipe to chromacat_recipe.yaml in CWD
                let recipe = self.create_recipe_snapshot();
                let yaml = serde_yaml::to_string(&recipe).map_err(|e| RendererError::Other(format!("Failed to serialize recipe: {e}")))?;
                std::fs::write("chromacat_recipe.yaml", yaml).map_err(|e| RendererError::Other(format!("Failed to write recipe: {e}")))?;
                self.status_bar.set_custom_text(Some("Recipe saved ▶ chromacat_recipe.yaml"));
                self.draw_full_screen()?;
                Ok(true)
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                // Minimal export: if build-tools enabled, suggest using webp-generator; otherwise hint
                #[cfg(feature = "build-tools")]
                {
                    self.status_bar.set_custom_text(Some("Export: run `cargo run --bin webp-generator --features build-tools --release` for full control"));
                }
                #[cfg(not(feature = "build-tools"))]
                {
                    self.status_bar.set_custom_text(Some("Export disabled. Build with --features build-tools for WebP export"));
                }
                self.draw_full_screen()?;
                Ok(true)
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                // Load recipe from chromacat_recipe.yaml
                match std::fs::read_to_string("chromacat_recipe.yaml") {
                    Ok(s) => match serde_yaml::from_str::<Recipe>(&s) {
                        Ok(recipe) => { self.apply_recipe(recipe)?; self.draw_full_screen()?; }
                        Err(e) => { self.status_bar.set_custom_text(Some(&format!("Failed to parse recipe: {e}"))); }
                    },
                    Err(e) => { self.status_bar.set_custom_text(Some(&format!("Failed to read recipe: {e}"))); }
                }
                Ok(true)
            }
            // Playlist controls
            KeyCode::Char(' ') if self.playlist_player.is_some() => {
                if let Some(player) = &mut self.playlist_player {
                    player.toggle_pause();
                    if let Some(entry) = player.current_entry() {
                        let status = if player.is_paused() {
                            "Paused"
                        } else {
                            "Playing"
                        };
                        self.status_bar.set_custom_text(Some(&format!(
                            "{} - {} [{:.0}%]",
                            status,
                            entry.name,
                            player.current_progress() * 100.0
                        )));
                    }
                }
                Ok(true)
            }
            KeyCode::Right if self.playlist_player.is_some() => {
                if let Some(player) = &mut self.playlist_player {
                    player.next_entry();
                    self.update_playlist_entry()?;
                }
                Ok(true)
            }
            KeyCode::Left if self.playlist_player.is_some() => {
                if let Some(player) = &mut self.playlist_player {
                    player.previous_entry();
                    self.update_playlist_entry()?;
                }
                Ok(true)
            }
            _ => match self.scroll.handle_key_event(key) {
                Action::Continue => {
                    let visible_range = self.scroll.get_visible_range();
                    self.buffer.update_colors(&self.engine, visible_range.0)?;
                    let mut stdout = self.terminal.stdout();
                    self.buffer.render_region(
                        &mut stdout,
                        visible_range.0,
                        visible_range.1,
                        self.terminal.colors_enabled(),
                        true,
                    )?;
                    self.status_bar.render(&mut stdout, &self.scroll)?;
                    stdout.flush()?;
                    Ok(true)
                }
                Action::Exit => Ok(false),
                Action::NoChange => Ok(true),
            },
        }
    }

    /// Handles mouse events for overlay interaction
    pub fn handle_mouse_event(&mut self, me: MouseEvent) -> Result<(), RendererError> {
        use crossterm::event::{MouseEventKind, MouseButton};
        if !self.overlay_visible { return Ok(()); }
        let size = self.terminal.size();
        let overlay_height = 8u16.min(size.1.saturating_sub(2));
        let start_y = size.1.saturating_sub(overlay_height + 2);
        let titles_y = start_y + 1;
        let list_rows = overlay_height.saturating_sub(3) as usize;
        let x = me.column;
        let y = me.row;
        match me.kind {
            MouseEventKind::ScrollUp => {
                match self.overlay_section {
                    0 => { self.overlay_pattern_sel = self.overlay_pattern_sel.saturating_sub(1); if self.overlay_pattern_sel < self.overlay_pattern_offset { self.overlay_pattern_offset = self.overlay_pattern_sel; } }
                    1 => { self.overlay_param_sel = self.overlay_param_sel.saturating_sub(1); if self.overlay_param_sel < self.overlay_param_offset { self.overlay_param_offset = self.overlay_param_sel; } }
                    2 => { self.overlay_theme_sel = self.overlay_theme_sel.saturating_sub(1); if self.overlay_theme_sel < self.overlay_theme_offset { self.overlay_theme_offset = self.overlay_theme_sel; } }
                    3 => { self.overlay_art_sel = self.overlay_art_sel.saturating_sub(1); if self.overlay_art_sel < self.overlay_art_offset { self.overlay_art_offset = self.overlay_art_sel; } }
                    _ => {}
                }
                self.draw_full_screen()?;
            }
            MouseEventKind::ScrollDown => {
                match self.overlay_section {
                    0 => { if self.overlay_pattern_sel + 1 < self.available_patterns.len() { self.overlay_pattern_sel += 1; let vis = self.overlay_pattern_sel - self.overlay_pattern_offset; if vis >= list_rows { self.overlay_pattern_offset += 1; } } }
                    1 => { let max = self.current_param_names().len(); if self.overlay_param_sel + 1 < max { self.overlay_param_sel += 1; let vis = self.overlay_param_sel - self.overlay_param_offset; if vis >= list_rows { self.overlay_param_offset += 1; } } }
                    2 => { if self.overlay_theme_sel + 1 < self.available_themes.len() { self.overlay_theme_sel += 1; let vis = self.overlay_theme_sel - self.overlay_theme_offset; if vis >= list_rows { self.overlay_theme_offset += 1; } } }
                    3 => { let max = self.modulator.as_ref().map(|m| m.routes.len()).unwrap_or(0); if self.overlay_art_sel + 1 < max { self.overlay_art_sel += 1; let vis = self.overlay_art_sel - self.overlay_art_offset; if vis >= list_rows { self.overlay_art_offset += 1; } } }
                    _ => {}
                }
                self.draw_full_screen()?;
            }
            MouseEventKind::Down(MouseButton::Left) => {
                // Determine column based on x
                // Use dynamic columns
                let width = self.terminal.size().0 as usize;
                let colw = (width / 4).max(18) as u16;
                let col = if x < colw { 0 } else if x < colw * 2 { 1 } else if x < colw * 3 { 2 } else { 3 };
                if y >= titles_y + 1 && y < titles_y + 1 + list_rows as u16 {
                    let row_index = (y - (titles_y + 1)) as usize;
                    match col {
                        0 => { self.overlay_section = 0; let idx = self.overlay_pattern_offset + row_index; if idx < self.available_patterns.len() { self.overlay_pattern_sel = idx; let apply = self.available_patterns.get(idx).cloned(); if let Some(pat) = apply { let _ = self.set_pattern_by_id(&pat); } } }
                        1 => { self.overlay_section = 1; let idx = self.overlay_param_offset + row_index; if idx < self.current_param_names().len() { self.overlay_param_sel = idx; } }
                        2 => { self.overlay_section = 2; let idx = self.overlay_theme_offset + row_index; if idx < self.available_themes.len() { self.overlay_theme_sel = idx; let apply = self.available_themes.get(idx).cloned(); if let Some(theme) = apply { let _ = self.set_theme_by_name(&theme); } } }
                        3 => { self.overlay_section = 3; let idx = self.overlay_art_offset + row_index; if idx < self.available_arts.len() { self.overlay_art_sel = idx; } }
                        _ => {}
                    }
                    self.draw_full_screen()?;
                }

                // Click in params column slider area to set numeric value
                if self.overlay_section == 1 && y >= titles_y + 1 {
                    let width = self.terminal.size().0 as u16;
                    let colw = (width / 4).max(18) as u16;
                    let x_prm = colw; // params column start
                    if x >= x_prm && x < x_prm + colw {
                    if let Some((name, ptype)) = self.param_meta_at(self.overlay_param_sel) {
                        if let crate::pattern::ParamType::Number { min, max } = ptype {
                            let slider_w: u16 = colw.saturating_sub(6); // some space for marker and label
                            let rel_x = (x.saturating_sub(x_prm + 2)).min(slider_w) as f64;
                            let t = (rel_x / (slider_w.max(1) as f64)).clamp(0.0, 1.0);
                            let val = min + t * (max - min);
                            let csv = format!("{}={}", name, val);
                            let _ = self.update_params_from_str(&csv);
                            self.overlay_param_values.insert(name, format!("{val:.3}"));
                            self.draw_full_screen()?;
                        }
                    }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    // Private helper methods

    fn draw_full_screen(&mut self) -> Result<(), RendererError> {
        let mut stdout = self.terminal.stdout();
        let visible_range = self.scroll.get_visible_range();

        self.buffer.render_region(
            &mut stdout,
            visible_range.0,
            visible_range.1,
            self.terminal.colors_enabled(),
            true,
        )?;
        self.status_bar.render(&mut stdout, &self.scroll)?;

        if self.overlay_visible {
            self.render_overlay(&mut stdout)?;
            self.overlay_dirty = false;
            self.last_overlay_draw = Instant::now();
        }

        stdout.flush()?;
        Ok(())
    }

    fn redraw_overlay_only(&mut self) -> Result<(), RendererError> {
        if !self.overlay_visible { return Ok(()); }
        let mut stdout = self.terminal.stdout();
        self.render_overlay(&mut stdout)?;
        stdout.flush()?;
        self.overlay_dirty = false;
        self.last_overlay_draw = Instant::now();
        Ok(())
    }

    fn update_playlist_entry(&mut self) -> Result<(), RendererError> {
        if let Some(player) = &mut self.playlist_player {
            if let Some(entry) = player.current_entry() {
                let new_config = entry.to_pattern_config()?;
                let new_gradient = themes::get_theme(&entry.theme)?.create_gradient()?;

                self.engine.update_gradient(new_gradient);
                self.engine.update_pattern_config(new_config);

                // Update art type for demo mode
                if self.demo_mode {
                    if let Some(art) = entry.art {
                        // Create new input reader with the entry's art type
                        let mut reader = InputReader::from_demo(true, None, Some(&art))?;
                        let mut new_content = String::new();
                        reader.read_to_string(&mut new_content)?;
                        self.content = new_content;
                        
                        // Prepare the new content for rendering
                        self.buffer.prepare_text(&self.content)?;
                        self.scroll.set_total_lines(self.buffer.line_count());
                    }
                }

                // Update status bar
                self.status_bar.set_pattern(&entry.pattern);
                self.status_bar.set_theme(&entry.theme);
            }
        }
        Ok(())
    }

    /// Switches to the next available theme
    fn next_theme(&mut self) -> Result<(), RendererError> {
        // Increment theme index
        self.current_theme_index = (self.current_theme_index + 1) % self.available_themes.len();
        let new_theme = &self.available_themes[self.current_theme_index];

        // Update theme
        let new_gradient = themes::get_theme(new_theme)?.create_gradient()?;
        self.engine.update_gradient(new_gradient);

        // Update status bar
        self.status_bar.set_theme(new_theme);

        Ok(())
    }

    /// Switches to the next available pattern
    fn next_pattern(&mut self) -> Result<(), RendererError> {
        // Increment pattern index
        self.current_pattern_index =
            (self.current_pattern_index + 1) % self.available_patterns.len();
        let new_pattern = &self.available_patterns[self.current_pattern_index];

        // Create new pattern config for transition engine
        let new_params = crate::pattern::REGISTRY
                .create_pattern_params(new_pattern)
            .ok_or_else(|| RendererError::InvalidPattern(new_pattern.clone()))?;

        let mut new_engine = self.engine.clone();
        new_engine.update_pattern_config(PatternConfig { common: self.engine.config().common.clone(), params: new_params });

        self.transition_engine = Some(new_engine);
        self.transition_alpha = 0.0;
        self.transitioning = true;
        self.status_bar.set_pattern(new_pattern);
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        if let Err(e) = self.terminal.cleanup() {
            eprintln!("Error cleaning up terminal: {}", e);
        }
    }
}

