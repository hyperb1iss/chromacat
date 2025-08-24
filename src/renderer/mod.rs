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

use crate::debug_log::debug_log;
use crate::pattern::PatternEngine;
use crate::playlist::{Playlist, PlaylistPlayer};
use crate::{themes, PatternConfig};
use crate::demo::art::DemoArt;
use crossterm::event::{KeyCode, MouseEvent};
#[cfg(not(feature = "playground-ui"))]
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
#[cfg(feature = "playground-ui")]
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color as TuiColor, Modifier, Style},
    text::{Line as TuiLine, Span as TuiSpan},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Terminal as TuiTerminal,
};
use rand::Rng;

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
    /// Crossfade duration in seconds
    transition_duration: f32,
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
    #[cfg(feature = "playground-ui")]
    tui: Option<TuiTerminal<CrosstermBackend<std::io::Stdout>>>,
    /// Toast message shown above the overlay for short duration
    toast_text: Option<String>,
    toast_time: Option<Instant>,
    toast_duration: Duration,
    /// Theme advance behavior: 0=Scene, 1=Locked (manual), 2=Random per scene
    theme_mode: u8,
    /// If Locked, this holds the chosen theme name
    locked_theme: Option<String>,
    /// Debug overlay for internal state
    debug_mode: bool,
}

impl Renderer {
    /// Generate fallback content if none was provided (e.g., playground without demo)
    fn generate_default_content(&self) -> String {
        // Try demo art first for something interesting; fall back to simple banner
        if let Ok(mut reader) = InputReader::from_demo(true, None, None) {
            let mut s = String::new();
            if reader.read_to_string(&mut s).is_ok() && !s.is_empty() { return s; }
        }
        // Minimal placeholder content
        let mut lines = String::new();
        for i in 0..64 {
            lines.push_str(&format!("ChromaCat Playground {:03}\n", i));
        }
        lines
    }
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
            transition_duration: 1.0,
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
            #[cfg(feature = "playground-ui")]
            tui: None, // Don't create immediately - will be created on first render
            toast_text: None,
            toast_time: None,
            toast_duration: Duration::from_millis(2200),
            theme_mode: 0,
            locked_theme: None,
            debug_mode: false,
        })
    }

    /// Ensures TUI terminal is healthy and ready for rendering
    #[cfg(feature = "playground-ui")]
    fn ensure_tui_healthy(&mut self) -> bool {
        let _ = debug_log(&format!("ensure_tui_healthy called, tui.is_some()={}", self.tui.is_some()));
        if self.tui.is_none() {
            let _ = debug_log("Creating CrosstermBackend...");
            // Try to create TUI terminal once if missing
            let backend = CrosstermBackend::new(std::io::stdout());
            let _ = debug_log("Creating TuiTerminal...");
            match TuiTerminal::new(backend) {
                Ok(t) => {
                    let _ = debug_log("TUI terminal created successfully");
                    self.tui = Some(t);
                    true
                }
                Err(e) => {
                    let _ = debug_log(&format!("TUI terminal creation failed: {}", e));
                    self.status_bar.set_custom_text(Some(&format!("TUI init failed: {}", e)));
                    self.overlay_visible = false; // Disable overlay on TUI failure
                    false
                }
            }
        } else {
            true
        }
    }

    /// Renders the overlay with three columns: patterns, params, themes
    fn render_overlay(&mut self) -> Result<(), RendererError> {
        let _ = debug_log(&format!("render_overlay called, overlay_visible={}", self.overlay_visible));
        #[cfg(feature = "playground-ui")]
        {
            // Ensure TUI is healthy before attempting to render
            if !self.ensure_tui_healthy() {
                let _ = debug_log("TUI not healthy, cannot render overlay");
                return Err(RendererError::Other("TUI terminal not available".to_string()));
            }
            let term_size = self.terminal.size();
            let width = term_size.0 as u16;
            let height = term_size.1 as u16;
            // Compact floating panel
            let overlay_height = (12u16).min(height.saturating_sub(4));
            let panel_w = ((width as u32 * 90) / 100) as u16; // 90% width
            let panel_w = panel_w.max(48).min(width.saturating_sub(2));
            let start_y = height.saturating_sub(overlay_height + 2);
            let start_x = ((width.saturating_sub(panel_w)) / 2).max(1);
            let overlay_rect = Rect { x: start_x, y: start_y, width: panel_w, height: overlay_height };
            let visible_rows: usize = overlay_height.saturating_sub(3) as usize;
            let pat_off = self.overlay_pattern_offset;
            let prm_off = self.overlay_param_offset;
            let thm_off = self.overlay_theme_offset;
            let art_off = self.overlay_art_offset;
            let pat_names = &self.available_patterns;
            let param_names = self.current_param_names();
            let thm_names = &self.available_themes;
            let art_names = &self.available_arts;

            let pat_sel = self.overlay_pattern_sel;
            let prm_sel = self.overlay_param_sel;
            let thm_sel = self.overlay_theme_sel;
            let art_sel = self.overlay_art_sel;
            // Precompute slices and labels to avoid borrowing `self` in draw closure
            let pat_end = (pat_off + visible_rows).min(pat_names.len());
            let pat_slice = &pat_names[pat_off..pat_end];
            let pat_high = pat_sel.saturating_sub(pat_off).min(pat_slice.len().saturating_sub(1));

            let prm_end = (prm_off + visible_rows).min(param_names.len());
            let mut prm_labels: Vec<String> = Vec::new();
            for (i, n) in param_names[prm_off..prm_end].iter().enumerate() {
                let global_i = prm_off + i;
                if global_i == prm_sel {
                    // Selected label with numeric bar if applicable
                    if let Some((name, ptype)) = self.param_meta_at(prm_sel) {
                        if let crate::pattern::ParamType::Number { min, max } = ptype {
                            let cur = self.overlay_param_values.get(&name).and_then(|s| s.parse::<f64>().ok()).unwrap_or_else(|| self.param_default_value(&name).and_then(|v| v.parse::<f64>().ok()).unwrap_or(min));
                            let t = ((cur - min) / (max - min)).clamp(0.0, 1.0);
                            let bars = (t * 10.0).round() as usize;
                            prm_labels.push(format!("{}  [{}{}] {:.2}", n, "█".repeat(bars), "░".repeat(10 - bars), cur));
                            continue;
                        }
                    }
                }
                prm_labels.push(n.clone());
            }
            let theme_mode_label: String = match self.theme_mode {
                0 => "Theme: Scene".to_string(),
                1 => "Theme: Locked".to_string(),
                2 => "Theme: Random".to_string(),
                _ => "Theme".to_string(),
            };
            let footer_text: String = format!(
                " [Tab] switch  [Enter] apply  [-/=] adjust  [n/b] step  [t] lock  [u] unlock  [Y] {}  [S] scenes  [E] export  [?] help",
                theme_mode_label
            );
            let prm_high = prm_sel.saturating_sub(prm_off).min(prm_labels.len().saturating_sub(1));

            let thm_end = (thm_off + visible_rows).min(thm_names.len());
            let thm_slice = &thm_names[thm_off..thm_end];
            let thm_high = thm_sel.saturating_sub(thm_off).min(thm_slice.len().saturating_sub(1));

            let art_end = (art_off + visible_rows).min(art_names.len());
            let art_slice = &art_names[art_off..art_end];
            let art_high = art_sel.saturating_sub(art_off).min(art_slice.len().saturating_sub(1));

            // After ensure_tui_healthy succeeds, tui should be Some
            let term = self.tui.as_mut().ok_or_else(|| {
                let _ = debug_log("CRITICAL: TUI is None after ensure_tui_healthy returned true!");
                RendererError::Other("TUI unexpectedly None".to_string())
            })?;
            
            term.draw(|f: &mut ratatui::Frame| {
                // Only clear inside the compact panel; leave surroundings untouched for a floating look
                f.render_widget(Clear, overlay_rect);

                // Inner layout with a 1-cell margin and 4 floating windows
                let inner = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(100)])
                    .margin(1)
                    .split(overlay_rect)[0];

                let cols = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(25),
                        Constraint::Length(1), // gutter
                        Constraint::Percentage(25),
                        Constraint::Length(1),
                        Constraint::Percentage(25),
                        Constraint::Length(1),
                        Constraint::Percentage(25),
                    ])
                    .split(inner);

                // Helper to build list with highlight and slice
                let render_list = |f: &mut ratatui::Frame, rect: Rect, title: &str, names: &[String], highlight_index: usize, focused: bool| {
                    let pulse = (Instant::now().elapsed().as_millis() / 450) % 2 == 0;
                    let focus_color = if pulse { TuiColor::Rgb(255, 130, 200) } else { TuiColor::LightMagenta };
                    let title_style = if focused { Style::default().fg(focus_color).add_modifier(Modifier::BOLD) } else { Style::default().fg(TuiColor::Magenta).add_modifier(Modifier::BOLD) };
                    let border_style = if focused { Style::default().fg(focus_color) } else { Style::default().fg(TuiColor::DarkGray) };
                    // Subtle shadow behind panel
                    let shadow = Rect { x: rect.x.saturating_add(1), y: rect.y.saturating_add(1), width: rect.width, height: rect.height };
                    let shadow_block = Block::default().style(Style::default().bg(TuiColor::Rgb(10, 10, 12)));
                    f.render_widget(shadow_block, shadow);
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(border_style)
                        .title(TuiLine::from(TuiSpan::styled(title, title_style)))
                        .style(Style::default().bg(TuiColor::Rgb(16, 16, 20)));
                    let items: Vec<ListItem> = names.iter().enumerate().map(|(i, n)| {
                        let is_sel = i == highlight_index;
                        let sel_glyph = if is_sel { "›" } else { " " };
                        let style = if is_sel { Style::default().fg(focus_color).add_modifier(Modifier::BOLD) } else { Style::default().fg(TuiColor::Gray) };
                        ListItem::new(TuiLine::from(TuiSpan::styled(format!(" {} {}", sel_glyph, n), style)))
                    }).collect();
                    let list = List::new(items).block(block);
                    f.render_widget(list, rect);
                };

                render_list(f, cols[0], if self.overlay_section == 0 { "▶ Patterns" } else { "Patterns" }, pat_slice, pat_high, self.overlay_section == 0);
                render_list(f, cols[2], if self.overlay_section == 1 { "▶ Params" } else { "Params" }, &prm_labels, prm_high, self.overlay_section == 1);
                render_list(f, cols[4], if self.overlay_section == 2 { "▶ Themes" } else { "Themes" }, thm_slice, thm_high, self.overlay_section == 2);
                render_list(f, cols[6], if self.overlay_section == 3 { "▶ Art" } else { "Art" }, art_slice, art_high, self.overlay_section == 3);

                // Footer line
                let footer = Paragraph::new(TuiLine::from(
                    TuiSpan::styled(
                        format!("{}   [Z] crossfade", footer_text),
                        Style::default().fg(TuiColor::DarkGray),
                    ),
                ));
                let footer_rect = Rect { x: start_x, y: start_y + overlay_height - 1, width: panel_w, height: 1 };
                f.render_widget(footer, footer_rect);

                // Toast above the panel if active
                if let (Some(text), Some(t0)) = (&self.toast_text, self.toast_time) {
                    if Instant::now().duration_since(t0) <= self.toast_duration {
                        let toast_w = (text.len() as u16 + 4).min(width.saturating_sub(2));
                        let tx = start_x + panel_w.saturating_sub(toast_w) / 2;
                        let ty = start_y.saturating_sub(2).max(1);
                        let toast_rect = Rect { x: tx, y: ty, width: toast_w, height: 1 };
                        let toast = Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(TuiColor::LightCyan))
                            .title(TuiLine::from(TuiSpan::styled(text.clone(), Style::default().fg(TuiColor::White))));
                        f.render_widget(toast, toast_rect);
                    }
                }
            }).map_err(|e| {
                let _ = debug_log(&format!("TUI draw error: {}", e));
                eprintln!("TUI draw error (overlay will retry): {}", e);
                RendererError::Other(format!("TUI draw failed: {}", e))
            })?;

            return Ok(());
        }
        #[cfg(not(feature = "playground-ui"))]
        { 
            /* Legacy overlay removed */ 
            Ok(())
        }
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
        // Clamp offsets to keep selection within the visible page
        let clamp_offset = |sel: usize, mut off: usize, len: usize| -> usize {
            if len == 0 { return 0; }
            if sel < off { off = sel; }
            let max_off = len.saturating_sub(visible_rows.max(1));
            if off > max_off { off = max_off; }
            off
        };
        self.overlay_pattern_offset = clamp_offset(self.overlay_pattern_sel, self.overlay_pattern_offset, self.available_patterns.len());
        self.overlay_param_offset = clamp_offset(self.overlay_param_sel, self.overlay_param_offset, self.current_param_names().len());
        self.overlay_theme_offset = clamp_offset(self.overlay_theme_sel, self.overlay_theme_offset, self.available_themes.len());
        self.overlay_art_offset = clamp_offset(self.overlay_art_sel, self.overlay_art_offset, self.available_arts.len());
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
            KeyCode::Enter => { match self.overlay_section { 0 => { if let Some(p) = self.available_patterns.get(self.overlay_pattern_sel) { let pat = p.clone(); self.set_pattern_by_id(&pat)?; if let Some(s) = self.scene_scheduler.as_mut() { s.set_current_pattern(&pat); } self.overlay_param_sel = 0; } }
                2 => { if let Some(t) = self.available_themes.get(self.overlay_theme_sel) { let theme = t.clone(); self.set_theme_by_name(&theme)?; if let Some(s) = self.scene_scheduler.as_mut() { s.set_current_theme(&theme); } } }
                3 => { if self.demo_mode { if let Some(a) = self.available_arts.get(self.overlay_art_sel) { let art = a.clone(); self.set_demo_art(&art)?; } } }
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
                            self.overlay_param_values.insert(name.clone(), format!("{new_value:.3}"));
                            // Toast on change
                            let msg = format!("{} = {:.3}", name, new_value);
                            self.toast_text = Some(msg);
                            self.toast_time = Some(Instant::now());
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
            KeyCode::Char(';') => {
                // Let toggle fall through to main handler
                return Ok(false);
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
        Recipe { current_theme, current_pattern: current_id, scenes, routes, theme_mode: Some(self.theme_mode), crossfade_seconds: Some(self.transition_duration) }
    }

    fn apply_recipe(&mut self, recipe: Recipe) -> Result<(), RendererError> {
        if let Some(t) = recipe.current_theme.as_ref() { let _ = self.set_theme_by_name(t); }
        if let Some(p) = recipe.current_pattern.as_ref() { let _ = self.set_pattern_by_id(p); }
        // Scenes
        if !recipe.scenes.is_empty() {
            let scenes = recipe.scenes.into_iter().map(|s| Scene { pattern_id: s.pattern_id, theme_name: s.theme_name, duration_secs: s.duration_secs }).collect::<Vec<_>>() ;
            self.scene_scheduler = Some(SceneScheduler::new(scenes));
        }
        // Theme mode and crossfade
        if let Some(mode) = recipe.theme_mode { self.theme_mode = mode; }
        if let Some(sec) = recipe.crossfade_seconds { self.transition_duration = sec.max(0.1); }
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

    /// Internal helper to apply a theme, optionally locking into manual mode
    fn apply_theme_by_name(&mut self, theme: &str, lock: bool) -> Result<(), RendererError> {
        let new_gradient = themes::get_theme(theme)?.create_gradient()?;
        self.engine.update_gradient(new_gradient);
        if let Some(idx) = self.available_themes.iter().position(|t| t == theme) {
            self.current_theme_index = idx;
        }
        self.status_bar.set_theme(theme);
        if lock {
            self.locked_theme = Some(theme.to_string());
            self.theme_mode = 1;
        }
        Ok(())
    }

    /// Sets the current theme by name (locks the theme until toggled)
    pub fn set_theme_by_name(&mut self, theme: &str) -> Result<(), RendererError> {
        self.apply_theme_by_name(theme, true)
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

        // First-time initialization (ensure we have content)
        if !self.buffer.has_content() {
            self.terminal.enter_alternate_screen()?;
            let mut initial = if text.is_empty() { self.generate_default_content() } else { text.to_string() };
            if initial.trim().is_empty() { initial = self.generate_default_content(); }
            self.buffer.prepare_text(&initial)?;
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
            let dur = self.transition_duration.max(0.1);
            self.transition_alpha += (delta_seconds as f32) / dur;
            if self.transition_alpha >= 1.0 {
                // Finish transition: swap to new engine
                if let Some(new_engine) = self.transition_engine.take() {
                    self.engine = new_engine;
                }
                self.transition_alpha = 0.0;
                self.transitioning = false;
                // If a scene scheduler exists, lock in the chosen pattern/theme so manual changes persist
                if let Some(s) = &mut self.scene_scheduler {
                    let cur_id = crate::pattern::REGISTRY.get_pattern_id(&self.engine.config().params).unwrap_or("horizontal");
                    s.set_current_pattern(cur_id);
                    if self.theme_mode == 1 {
                        if let Some(theme) = self.available_themes.get(self.current_theme_index) { s.set_current_theme(theme); }
                    }
                }
            }
        }

        // Update status bar
        self.status_bar.render(&mut stdout, &self.scroll)?;

        // Flush main content before optional overlay
        stdout.flush()?;
        drop(stdout);

        // Render overlay if enabled (always draw after content to avoid flicker)
        if self.overlay_visible {
            #[cfg(feature = "playground-ui")]
            let _ = debug_log(&format!("render_frame: About to render overlay, tui.is_some()={}", self.tui.is_some()));
            #[cfg(not(feature = "playground-ui"))]
            let _ = debug_log("render_frame: About to render overlay");
            // Don't fail the entire frame if overlay has issues
            if let Err(e) = self.render_overlay() {
                let _ = debug_log(&format!("render_frame: Overlay render error: {}", e));
                eprintln!("Overlay render warning: {}", e);
                // Only disable overlay for persistent TUI failures
                if format!("{}", e).contains("TUI terminal not available") {
                    self.overlay_visible = false;
                }
            } else {
                self.last_overlay_draw = Instant::now();
                self.overlay_dirty = false;
            }
        }

        // Scene scheduler
        if let Some(next) = self
            .scene_scheduler
            .as_mut()
            .and_then(|s| if s.is_enabled() { s.tick(delta_seconds as f32).cloned() } else { None })
        {
            // Determine theme based on mode
            match self.theme_mode {
                0 => { // Scene mode
                    self.apply_theme_by_name(&next.theme_name, false)?;
                }
                1 => { // Locked mode
                    if let Some(theme) = self.locked_theme.clone() {
                        self.apply_theme_by_name(&theme, false)?;
                    }
                }
                2 => { // Random mode
                    if !self.available_themes.is_empty() {
                        let mut rng = rand::thread_rng();
                        let idx = rng.gen_range(0..self.available_themes.len());
                        let theme = self.available_themes[idx].clone();
                        self.apply_theme_by_name(&theme, false)?;
                    }
                }
                _ => {}
            }
            if let Some(params) = crate::pattern::REGISTRY.create_pattern_params(&next.pattern_id) {
                let mut new_engine = self.engine.clone();
                new_engine.update_pattern_config(PatternConfig { common: self.engine.config().common.clone(), params });
                self.transition_engine = Some(new_engine);
                self.transition_alpha = 0.0;
                self.transitioning = true;
                self.status_bar.set_pattern(&next.pattern_id);
            }
        } else if self.scheduler_enabled && self.scene_scheduler.is_none() && now.duration_since(self.scheduler_last_switch) >= self.scheduler_interval {
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
                let msg = if self.overlay_visible { "Overlay: ON" } else { "Overlay: OFF" };
                self.status_bar.set_custom_text(Some(msg));
                // TUI terminal will be created/validated when render_overlay is called
                self.draw_full_screen()?;
                Ok(true)
            }
            KeyCode::Char('?') => {
                self.help_visible = !self.help_visible;
                if self.help_visible { self.status_bar.set_custom_text(Some("? Help: ; overlay | Tab navigate | Enter apply | -= adjust | b bind | m mod | S scenes | V reseed | n/b step | t lock theme | u unlock | Y theme mode | R save | L load | q quit")); } else { self.status_bar.set_custom_text(None); }
                self.draw_full_screen()?;
                Ok(true)
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                self.next_theme()?; // manual action keeps lock semantics
                if let Some(name) = self.available_themes.get(self.current_theme_index).cloned() {
                    self.locked_theme = Some(name.clone());
                    self.theme_mode = 1;
                    self.toast_text = Some(format!("Theme locked: {}", name));
                    self.toast_time = Some(Instant::now());
                }
                self.draw_full_screen()?;
                Ok(true)
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                // Cycle theme advance mode: 0 Scene -> 1 Locked -> 2 Random -> 0
                self.theme_mode = (self.theme_mode + 1) % 3;
                let label = match self.theme_mode { 0 => "Theme Mode: Scene", 1 => "Theme Mode: Locked", 2 => "Theme Mode: Random", _ => "Theme Mode" };
                self.status_bar.set_custom_text(Some(label));
                self.toast_text = Some(label.to_string());
                self.toast_time = Some(Instant::now());
                // If switching to Locked and no lock yet, lock current
                if self.theme_mode == 1 && self.locked_theme.is_none() {
                    if let Some(name) = self.available_themes.get(self.current_theme_index).cloned() { self.locked_theme = Some(name); }
                }
                self.draw_full_screen()?;
                Ok(true)
            }
            KeyCode::Char('u') | KeyCode::Char('U') => {
                // Unlock theme and return to Scene mode
                self.theme_mode = 0;
                self.locked_theme = None;
                self.status_bar.set_custom_text(Some("Theme Mode: Scene (unlocked)"));
                self.toast_text = Some("Theme unlocked".to_string());
                self.toast_time = Some(Instant::now());
                self.draw_full_screen()?;
                Ok(true)
            }
            KeyCode::Char('z') | KeyCode::Char('Z') => {
                // Cycle crossfade duration: 0.5 -> 1.0 -> 2.0 seconds
                let next = if self.transition_duration < 0.75 { 1.0 } else if self.transition_duration < 1.5 { 2.0 } else { 0.5 };
                self.transition_duration = next;
                let label = match next as u32 { 0 => "Crossfade: Short", 1 => "Crossfade: Medium", 2 => "Crossfade: Long", _ => "Crossfade" };
                let msg = format!("{} ({:.1}s)", label, next);
                self.status_bar.set_custom_text(Some(&msg));
                self.toast_text = Some(msg);
                self.toast_time = Some(Instant::now());
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
                    // Seed a richer cycle with alternating patterns/themes
                    let mut sched = SceneScheduler::new(vec![]);
                    let mut pats = self.available_patterns.clone();
                    let plen = pats.len();
                    if plen > 1 { let k = self.current_pattern_index % plen; pats.rotate_left(k); }
                    let mut ths = self.available_themes.clone();
                    let tlen = ths.len();
                    if tlen > 1 { let k = self.current_theme_index % tlen; ths.rotate_left(k); }
                    sched.reseed_variety(&pats, &ths, 8);
                    self.scene_scheduler = Some(sched);
                }
                let enabled = if let Some(s) = self.scene_scheduler.as_mut() { let en = !s.is_enabled(); s.set_enabled(en); en } else { false };
                let msg = if enabled { "Scene Scheduler: ON" } else { "Scene Scheduler: OFF" };
                self.status_bar.set_custom_text(Some(msg));
                self.draw_full_screen()?;
                Ok(true)
            }
            KeyCode::Char('v') | KeyCode::Char('V') => {
                // Force reseed with a varied multi-scene cycle
                let mut sched = self.scene_scheduler.take().unwrap_or_else(|| SceneScheduler::new(vec![]));
                let mut pats = self.available_patterns.clone();
                let plen = pats.len();
                if plen > 1 { let k = self.current_pattern_index % plen; pats.rotate_left(k); }
                let mut ths = self.available_themes.clone();
                let tlen = ths.len();
                if tlen > 1 { let k = self.current_theme_index % tlen; ths.rotate_left(k); }
                sched.reseed_variety(&pats, &ths, 10);
                self.scene_scheduler = Some(sched);
                self.status_bar.set_custom_text(Some("Variety cycle reseeded"));
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
                    let msg = "Export: run `cargo run --bin webp-generator --features build-tools --release` for full control";
                    self.status_bar.set_custom_text(Some(msg));
                    self.toast_text = Some(msg.to_string());
                    self.toast_time = Some(Instant::now());
                }
                #[cfg(not(feature = "build-tools"))]
                {
                    let msg = "Export disabled. Build with --features build-tools for WebP export";
                    self.status_bar.set_custom_text(Some(msg));
                    self.toast_text = Some(msg.to_string());
                    self.toast_time = Some(Instant::now());
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
            KeyCode::Char('n') | KeyCode::Char('.') | KeyCode::Char('>') => {
                if let Some(s) = self.scene_scheduler.as_mut() {
                    if let Some(next) = s.jump_next().cloned() {
                        self.set_theme_by_name(&next.theme_name)?;
                        if let Some(params) = crate::pattern::REGISTRY.create_pattern_params(&next.pattern_id) {
                            let mut new_engine = self.engine.clone();
                            new_engine.update_pattern_config(PatternConfig { common: self.engine.config().common.clone(), params });
                            self.transition_engine = Some(new_engine);
                            self.transition_alpha = 0.0;
                            self.transitioning = true;
                            self.status_bar.set_pattern(&next.pattern_id);
                        }
                    }
                    self.draw_full_screen()?;
                    return Ok(true);
                }
                Ok(true)
            }
            KeyCode::Char('b') | KeyCode::Char(',') | KeyCode::Char('<') => {
                if let Some(s) = self.scene_scheduler.as_mut() {
                    if let Some(prev) = s.jump_prev().cloned() {
                        self.set_theme_by_name(&prev.theme_name)?;
                        if let Some(params) = crate::pattern::REGISTRY.create_pattern_params(&prev.pattern_id) {
                            let mut new_engine = self.engine.clone();
                            new_engine.update_pattern_config(PatternConfig { common: self.engine.config().common.clone(), params });
                            self.transition_engine = Some(new_engine);
                            self.transition_alpha = 0.0;
                            self.transitioning = true;
                            self.status_bar.set_pattern(&prev.pattern_id);
                        }
                    }
                    self.draw_full_screen()?;
                    return Ok(true);
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
        
        // Validate both overlay visibility and TUI health before processing
        if !self.overlay_visible { return Ok(()); }
        
        #[cfg(feature = "playground-ui")]
        {
            // Don't process mouse events if TUI is not available
            if self.tui.is_none() {
                self.overlay_visible = false; // Disable overlay if TUI missing
                return Ok(());
            }
        }
        let size = self.terminal.size();
        // Mirror the compact floating panel geometry from the ratatui path
        let overlay_height = (12u16).min(size.1.saturating_sub(4));
        let panel_w = (((size.0 as u32) * 90) / 100) as u16;
        let panel_w = panel_w.max(48).min(size.0.saturating_sub(2));
        let start_y = size.1.saturating_sub(overlay_height + 2);
        let start_x = ((size.0.saturating_sub(panel_w)) / 2).max(1);
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
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left) => {
                // Determine column based on x using floating panel geometry
                let size = self.terminal.size();
                let _overlay_height = (12u16).min(size.1.saturating_sub(4));
                let panel_w = (((size.0 as u32) * 90) / 100) as u16;
                let panel_w = panel_w.max(48).min(size.0.saturating_sub(2));
                let start_x = ((size.0.saturating_sub(panel_w)) / 2).max(1);
                let colw = (panel_w as usize / 4).max(18) as u16;
                let rel_x = x.saturating_sub(start_x);
                let col = if rel_x < colw { 0 } else if rel_x < colw * 2 + 1 { 1 } else if rel_x < colw * 3 + 2 { 2 } else { 3 };
                if y >= titles_y + 1 && y < titles_y + 1 + list_rows as u16 {
                    let row_index = (y - (titles_y + 1)) as usize;
                    match col {
                        0 => { self.overlay_section = 0; let idx = self.overlay_pattern_offset + row_index; if idx < self.available_patterns.len() { self.overlay_pattern_sel = idx; let apply = self.available_patterns.get(idx).cloned(); if let Some(pat) = apply { let _ = self.set_pattern_by_id(&pat); if let Some(s) = self.scene_scheduler.as_mut() { s.set_current_pattern(&pat); } } } }
                        1 => { self.overlay_section = 1; let idx = self.overlay_param_offset + row_index; if idx < self.current_param_names().len() { self.overlay_param_sel = idx; } }
                        2 => { self.overlay_section = 2; let idx = self.overlay_theme_offset + row_index; if idx < self.available_themes.len() { self.overlay_theme_sel = idx; let apply = self.available_themes.get(idx).cloned(); if let Some(theme) = apply { let _ = self.set_theme_by_name(&theme); if let Some(s) = self.scene_scheduler.as_mut() { s.set_current_theme(&theme); } } } }
                        3 => { self.overlay_section = 3; let idx = self.overlay_art_offset + row_index; if idx < self.available_arts.len() { self.overlay_art_sel = idx; if self.demo_mode { if let Some(a) = self.available_arts.get(idx) { let art = a.clone(); let _ = self.set_demo_art(&art); } } } }
                        _ => {}
                    }
                    if !matches!(me.kind, MouseEventKind::Drag(_)) { self.draw_full_screen()?; }
                }

                // Click in params column slider area to set numeric value
                if self.overlay_section == 1 && y >= titles_y + 1 {
                    let x_prm = start_x + colw + 1; // params column start with gutter
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
                            if !matches!(me.kind, MouseEventKind::Drag(_)) { self.draw_full_screen()?; }
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

        // Flush before drawing overlay to release borrow of stdout
        stdout.flush()?;
        drop(stdout);

        if self.overlay_visible {
            // Draw overlay last every frame (TUI health check happens inside render_overlay)
            // Don't fail the entire draw if overlay has issues
            if let Err(e) = self.render_overlay() {
                eprintln!("Overlay render warning in draw_full_screen: {}", e);
                // Only disable overlay for persistent TUI failures
                if format!("{}", e).contains("TUI terminal not available") {
                    self.overlay_visible = false;
                }
            } else {
                self.overlay_dirty = false;
                self.last_overlay_draw = Instant::now();
            }
        }
        Ok(())
    }

    fn redraw_overlay_only(&mut self) -> Result<(), RendererError> {
        if !self.overlay_visible { return Ok(()); }
        // TUI health check happens inside render_overlay
        self.render_overlay()?;
        let mut stdout = self.terminal.stdout();
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

    /// Apply demo art content into the buffer (only in demo mode)
    pub fn set_demo_art(&mut self, art: &str) -> Result<(), RendererError> {
        let _ = debug_log(&format!("set_demo_art called with art='{}'", art));
        if !self.demo_mode { return Ok(()); }
        let demo = DemoArt::try_from_str(art).ok_or_else(|| RendererError::Other("Unknown demo art".to_string()))?;
        let mut reader = InputReader::from_demo(true, None, Some(&demo))?;
        let mut new_content = String::new();
        reader.read_to_string(&mut new_content)?;
        self.content = new_content;
        self.buffer.prepare_text(&self.content)?;
        self.scroll.set_total_lines(self.buffer.line_count());
        // Ensure overlay remains visible and is drawn immediately after art switch
        let _ = debug_log("Setting overlay_visible=true after art switch");
        self.overlay_visible = true;
        self.overlay_dirty = true;
        // Repaint everything so the overlay floats above the new content
        // Be more resilient - don't kill overlay on minor rendering issues
        if let Err(e) = self.draw_full_screen() {
            // Log the error but don't disable overlay unless it's a TUI-specific failure
            let _ = debug_log(&format!("draw_full_screen error in set_demo_art: {}", e));
            eprintln!("Warning: Render issue after art switch: {}", e);
            // Only disable overlay if TUI specifically failed
            if format!("{}", e).contains("TUI") {
                self.overlay_visible = false;
                self.tui = None;
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

