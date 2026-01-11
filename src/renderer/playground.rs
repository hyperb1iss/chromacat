/// Playground UI implementation using ratatui
/// This module handles all playground-specific rendering and interaction
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color as TuiColor, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Terminal,
};
use std::io::Stdout;
use std::time::{Duration, Instant};

use crate::debug_log::debug_log;
use crate::pattern::PatternEngine;
use crate::renderer::{
    blend_engine::{BlendEngine, TransitionEffect},
    error::RendererError,
    pattern_widget::PatternWidget,
};

/// Data needed for rendering overlay (to avoid borrow issues)
#[derive(Clone)]
struct OverlayRenderData {
    pattern_names: Vec<String>,
    theme_names: Vec<String>,
    art_names: Vec<String>,
    param_names: Vec<String>,
    pattern_sel: usize,
    theme_sel: usize,
    art_sel: usize,
    param_sel: usize,
    pattern_offset: usize,
    theme_offset: usize,
    art_offset: usize,
    param_offset: usize,
    active_section: usize,
}

/// Manages the playground UI state and rendering
pub struct PlaygroundUI {
    /// Ratatui terminal instance
    terminal: Option<Terminal<CrosstermBackend<Stdout>>>,

    /// Pattern selection state
    pub pattern_sel: usize,
    pub pattern_offset: usize,
    pub pattern_names: Vec<String>,

    /// Theme selection state  
    pub theme_sel: usize,
    pub theme_offset: usize,
    pub theme_names: Vec<String>,

    /// Art selection state
    pub art_sel: usize,
    pub art_offset: usize,
    pub art_names: Vec<String>,

    /// Parameter selection state
    pub param_sel: usize,
    pub param_offset: usize,
    pub param_names: Vec<String>,
    pub param_values: std::collections::HashMap<String, String>,

    /// Current active section (0=patterns, 1=params, 2=themes, 3=art)
    pub active_section: usize,

    /// Overlay visibility
    pub overlay_visible: bool,

    /// Help visibility
    pub help_visible: bool,

    /// Toast message
    toast_message: Option<(String, Instant)>,
    toast_duration: Duration,

    /// Terminal size for mouse handling
    pub terminal_size: (u16, u16),

    /// Current active pattern (for display)
    pub current_pattern: String,
    /// Current active theme (for display)
    pub current_theme: String,
    /// Current active art (for display)
    pub current_art: Option<String>,
    /// Current automix mode (for status bar)
    pub automix_mode: String,
    /// Scene progress (0.0-1.0) for automix display
    pub scene_progress: f32,
    /// Whether a transition is active
    pub is_transitioning: bool,
    /// Whether theme is locked (prevents automix from changing it)
    pub theme_locked: bool,
    /// Whether modulation (LFO parameter automation) is enabled
    pub modulation_enabled: bool,
}

impl Default for PlaygroundUI {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaygroundUI {
    pub fn new() -> Self {
        Self {
            terminal: None,
            pattern_sel: 0,
            pattern_offset: 0,
            pattern_names: Vec::new(),
            theme_sel: 0,
            theme_offset: 0,
            theme_names: Vec::new(),
            art_sel: 0,
            art_offset: 0,
            art_names: Vec::new(),
            param_sel: 0,
            param_offset: 0,
            param_names: Vec::new(),
            param_values: std::collections::HashMap::new(),
            active_section: 0,
            overlay_visible: true,
            help_visible: false,
            toast_message: None,
            toast_duration: Duration::from_secs(2),
            terminal_size: (80, 24),
            current_pattern: "diagonal".to_string(),
            current_theme: "rainbow".to_string(),
            current_art: Some("rainbow".to_string()),
            automix_mode: "Off".to_string(),
            scene_progress: 0.0,
            is_transitioning: false,
            theme_locked: false,
            modulation_enabled: false,
        }
    }

    /// Ensure terminal is initialized
    pub fn ensure_terminal(&mut self) -> Result<(), RendererError> {
        if self.terminal.is_none() {
            let backend = CrosstermBackend::new(std::io::stdout());
            let terminal = Terminal::new(backend)
                .map_err(|e| RendererError::Other(format!("Failed to create terminal: {e}")))?;
            self.terminal = Some(terminal);
        }

        // Update terminal size
        if let Some(term) = &self.terminal {
            if let Ok(size) = term.size() {
                self.terminal_size = (size.width, size.height);
            }
        }

        Ok(())
    }

    /// Show a toast message
    pub fn show_toast(&mut self, message: impl Into<String>) {
        self.toast_message = Some((message.into(), Instant::now()));
    }

    /// Update selection and scroll offset for a section
    /// visible_height should be approximate panel content height
    pub fn update_selection(&mut self, section: usize, new_sel: usize) {
        // Estimate visible items based on panel height (1/4 of terminal, minus headers/footers)
        let panel_height = (self.terminal_size.1 / 4).clamp(10, 20);
        let visible_items = panel_height.saturating_sub(5) as usize; // 3 header + 2 footer

        match section {
            0 => {
                self.pattern_sel = new_sel;
                self.pattern_offset =
                    Self::calculate_offset(new_sel, self.pattern_offset, visible_items);
            }
            1 => {
                self.param_sel = new_sel;
                self.param_offset =
                    Self::calculate_offset(new_sel, self.param_offset, visible_items);
            }
            2 => {
                self.theme_sel = new_sel;
                self.theme_offset =
                    Self::calculate_offset(new_sel, self.theme_offset, visible_items);
            }
            3 => {
                self.art_sel = new_sel;
                self.art_offset = Self::calculate_offset(new_sel, self.art_offset, visible_items);
            }
            _ => {}
        }
    }

    /// Render the complete playground frame with blending support
    pub fn render_with_blending(
        &mut self,
        content: &str,
        engine: &PatternEngine,
        blend_engine: &BlendEngine,
        transition_effect: TransitionEffect,
        time: f64,
    ) -> Result<(), RendererError> {
        self.ensure_terminal()?;

        // Check if toast should expire
        if let Some((_, start)) = &self.toast_message {
            if start.elapsed() > self.toast_duration {
                self.toast_message = None;
            }
        }

        // Prepare data for rendering to avoid borrow issues
        let overlay_visible = self.overlay_visible;
        let help_visible = self.help_visible;
        let toast_message = self.toast_message.clone();

        // Clone data needed for overlay rendering
        let overlay_data = if overlay_visible {
            Some(OverlayRenderData {
                pattern_names: self.pattern_names.clone(),
                theme_names: self.theme_names.clone(),
                art_names: self.art_names.clone(),
                param_names: self.param_names.clone(),
                pattern_sel: self.pattern_sel,
                theme_sel: self.theme_sel,
                art_sel: self.art_sel,
                param_sel: self.param_sel,
                pattern_offset: self.pattern_offset,
                theme_offset: self.theme_offset,
                art_offset: self.art_offset,
                param_offset: self.param_offset,
                active_section: self.active_section,
            })
        } else {
            None
        };

        let term = self
            .terminal
            .as_mut()
            .ok_or_else(|| RendererError::Other("Terminal not initialized".into()))?;

        term.draw(|f| {
            let size = f.area();

            // First render the pattern as background with blending (only if transitioning)
            let pattern_widget = if blend_engine.is_transitioning() {
                PatternWidget::with_blending(content, engine, blend_engine, transition_effect, time)
            } else {
                PatternWidget::new(content, engine, time)
            };
            f.render_widget(pattern_widget, size);

            // Then render overlay on top if visible
            if let Some(ref data) = overlay_data {
                Self::render_overlay_static(f, size, data);
            }

            // Render toast if present
            if let Some((ref text, _)) = toast_message {
                Self::render_toast_static(f, size, text);
            }

            // Render help modal if visible (on top of everything except status bar)
            if help_visible {
                Self::render_help_modal_static(f, size);
            }

            // Render status bar at bottom with current state
            Self::render_status_bar_with_state(
                f,
                size,
                &self.current_pattern,
                &self.current_theme,
                self.current_art.as_deref(),
                &self.automix_mode,
                self.scene_progress,
                self.is_transitioning,
                self.theme_locked,
                self.modulation_enabled,
            );
        })
        .map_err(|e| {
            let _ = debug_log(&format!("Ratatui draw error: {e}"));
            RendererError::Other(format!("Draw failed: {e}"))
        })?;

        Ok(())
    }

    /// Render the overlay UI (static version)
    fn render_overlay_static(f: &mut ratatui::Frame, size: Rect, data: &OverlayRenderData) {
        // Create a bottom panel that takes up 1/4 of the screen height
        let panel_height = (size.height / 4).clamp(10, 20); // 1/4 height, min 10, max 20
        let panel_y = size.height.saturating_sub(panel_height).saturating_sub(1); // Above status bar

        let panel_area = Rect {
            x: 0,
            y: panel_y,
            width: size.width,
            height: panel_height,
        };

        // Clear the panel area and draw a border
        f.render_widget(Clear, panel_area);
        let panel_block = Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .title(" ChromaCat Playground ")
            .style(Style::default().bg(TuiColor::Black).fg(TuiColor::White));
        f.render_widget(panel_block, panel_area);

        // Layout for inner content
        let inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header with titles
                Constraint::Min(3),    // Lists
                Constraint::Length(1), // Footer with controls
            ])
            .split(panel_area.inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 1,
            }));

        // Header with column titles
        let titles = ["Patterns", "Params", "Themes", "Art"];
        let title_style = |i| {
            if i == data.active_section {
                Style::default().fg(TuiColor::Yellow)
            } else {
                Style::default().fg(TuiColor::DarkGray)
            }
        };

        let header = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(inner[0]);

        for (i, title) in titles.iter().enumerate() {
            let p = Paragraph::new(*title).style(title_style(i));
            f.render_widget(p, header[i]);
        }

        // Content area with 4 columns
        let content_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(inner[1]);

        // Render each list with auto-scrolling
        let pattern_offset = Self::calculate_offset(
            data.pattern_sel,
            data.pattern_offset,
            content_area[0].height as usize,
        );
        let param_offset = Self::calculate_offset(
            data.param_sel,
            data.param_offset,
            content_area[1].height as usize,
        );
        let theme_offset = Self::calculate_offset(
            data.theme_sel,
            data.theme_offset,
            content_area[2].height as usize,
        );
        let art_offset = Self::calculate_offset(
            data.art_sel,
            data.art_offset,
            content_area[3].height as usize,
        );

        Self::render_list_static(
            f,
            content_area[0],
            &data.pattern_names,
            data.pattern_sel,
            pattern_offset,
            data.active_section == 0,
        );
        Self::render_list_static(
            f,
            content_area[1],
            &data.param_names,
            data.param_sel,
            param_offset,
            data.active_section == 1,
        );
        Self::render_list_static(
            f,
            content_area[2],
            &data.theme_names,
            data.theme_sel,
            theme_offset,
            data.active_section == 2,
        );
        Self::render_list_static(
            f,
            content_area[3],
            &data.art_names,
            data.art_sel,
            art_offset,
            data.active_section == 3,
        );

        // Footer with controls
        let footer_text =
            "[Tab] switch • [↑↓] select • [Enter] apply • [;] toggle overlay • [q] quit";
        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(TuiColor::Cyan))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(footer, inner[2]);
    }

    /// Calculate scroll offset to keep selection visible
    fn calculate_offset(selected: usize, current_offset: usize, visible_items: usize) -> usize {
        if selected < current_offset {
            selected
        } else if selected >= current_offset + visible_items {
            selected.saturating_sub(visible_items - 1)
        } else {
            current_offset
        }
    }

    /// Render a list with selection (static version)
    fn render_list_static(
        f: &mut ratatui::Frame,
        area: Rect,
        items: &[String],
        selected: usize,
        offset: usize,
        is_active: bool,
    ) {
        let visible_items = area.height as usize;

        let list_items: Vec<ListItem> = items
            .iter()
            .skip(offset)
            .take(visible_items)
            .enumerate()
            .map(|(i, name)| {
                let is_selected = offset + i == selected;
                let style = if is_selected && is_active {
                    Style::default().bg(TuiColor::Blue).fg(TuiColor::White)
                } else if is_selected {
                    Style::default().fg(TuiColor::Cyan)
                } else {
                    Style::default()
                };
                ListItem::new(name.as_str()).style(style)
            })
            .collect();

        let list = List::new(list_items);
        f.render_widget(list, area);
    }

    /// Render toast message (static version)
    fn render_toast_static(f: &mut ratatui::Frame, size: Rect, text: &str) {
        let toast_rect = Rect {
            x: (size.width / 2).saturating_sub(text.len() as u16 / 2),
            y: 2,
            width: text.len() as u16 + 4,
            height: 3,
        };

        f.render_widget(Clear, toast_rect);

        let toast_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(TuiColor::Black).fg(TuiColor::White));
        f.render_widget(toast_block, toast_rect);

        let toast_inner = toast_rect.inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 1,
        });
        let toast_text = Paragraph::new(text).style(Style::default().fg(TuiColor::Yellow));
        f.render_widget(toast_text, toast_inner);
    }

    /// Render help modal overlay
    fn render_help_modal_static(f: &mut ratatui::Frame, size: Rect) {
        // Centered modal with keyboard shortcuts
        let modal_width = 52.min(size.width.saturating_sub(4));
        let modal_height = 20.min(size.height.saturating_sub(4));
        let modal_x = (size.width.saturating_sub(modal_width)) / 2;
        let modal_y = (size.height.saturating_sub(modal_height)) / 2;

        let modal_rect = Rect {
            x: modal_x,
            y: modal_y,
            width: modal_width,
            height: modal_height,
        };

        f.render_widget(Clear, modal_rect);

        let modal_block = Block::default()
            .borders(Borders::ALL)
            .title(" Keyboard Shortcuts ")
            .title_style(Style::default().fg(TuiColor::Cyan).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(TuiColor::Black).fg(TuiColor::White));
        f.render_widget(modal_block, modal_rect);

        let inner = modal_rect.inner(ratatui::layout::Margin {
            horizontal: 2,
            vertical: 1,
        });

        // Help text with key bindings
        let help_lines = vec![
            Line::from(vec![
                Span::styled("Navigation", Style::default().fg(TuiColor::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  ;       ", Style::default().fg(TuiColor::Cyan)),
                Span::raw("Toggle overlay panel"),
            ]),
            Line::from(vec![
                Span::styled("  Tab     ", Style::default().fg(TuiColor::Cyan)),
                Span::raw("Cycle sections"),
            ]),
            Line::from(vec![
                Span::styled("  ↑/↓     ", Style::default().fg(TuiColor::Cyan)),
                Span::raw("Navigate items"),
            ]),
            Line::from(vec![
                Span::styled("  Enter   ", Style::default().fg(TuiColor::Cyan)),
                Span::raw("Apply selection"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Controls", Style::default().fg(TuiColor::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  -/=     ", Style::default().fg(TuiColor::Cyan)),
                Span::raw("Adjust parameter"),
            ]),
            Line::from(vec![
                Span::styled("  a/A     ", Style::default().fg(TuiColor::Cyan)),
                Span::raw("Toggle automix (a=random, A=playlist)"),
            ]),
            Line::from(vec![
                Span::styled("  z       ", Style::default().fg(TuiColor::Cyan)),
                Span::raw("Cycle crossfade duration"),
            ]),
            Line::from(vec![
                Span::styled("  Space   ", Style::default().fg(TuiColor::Cyan)),
                Span::raw("Skip to next scene"),
            ]),
            Line::from(vec![
                Span::styled("  t/y     ", Style::default().fg(TuiColor::Cyan)),
                Span::raw("Next/prev theme"),
            ]),
            Line::from(vec![
                Span::styled("  u       ", Style::default().fg(TuiColor::Cyan)),
                Span::raw("Lock/unlock theme"),
            ]),
            Line::from(vec![
                Span::styled("  m       ", Style::default().fg(TuiColor::Cyan)),
                Span::raw("Toggle modulation"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Recipes", Style::default().fg(TuiColor::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  R       ", Style::default().fg(TuiColor::Cyan)),
                Span::raw("Save current state"),
            ]),
            Line::from(vec![
                Span::styled("  L       ", Style::default().fg(TuiColor::Cyan)),
                Span::raw("Load saved recipe"),
            ]),
            Line::from(vec![
                Span::styled("  q/Esc   ", Style::default().fg(TuiColor::Cyan)),
                Span::raw("Quit"),
            ]),
        ];

        let help_paragraph = Paragraph::new(help_lines);
        f.render_widget(help_paragraph, inner);
    }

    /// Handle terminal resize
    pub fn resize(&mut self) -> Result<(), RendererError> {
        if let Some(term) = &mut self.terminal {
            term.autoresize()
                .map_err(|e| RendererError::Other(format!("Failed to resize: {e}")))?;
            // Update cached terminal size
            if let Ok(size) = term.size() {
                self.terminal_size = (size.width, size.height);
            }
        }
        Ok(())
    }

    /// Render status bar at bottom with current state
    fn render_status_bar_with_state(
        f: &mut ratatui::Frame,
        size: Rect,
        pattern: &str,
        theme: &str,
        art: Option<&str>,
        automix_mode: &str,
        scene_progress: f32,
        is_transitioning: bool,
        theme_locked: bool,
        modulation_enabled: bool,
    ) {
        let status_area = Rect {
            x: 0,
            y: size.height.saturating_sub(1),
            width: size.width,
            height: 1,
        };

        // Clear the status bar area
        f.render_widget(Clear, status_area);

        // Build status text with current state
        let art_str = art.unwrap_or("none");
        let lock_indicator = if theme_locked { " [locked]" } else { "" };
        let mod_indicator = if modulation_enabled { " [mod]" } else { "" };
        let automix_str = if automix_mode != "Off" {
            // Show progress bar and transition indicator
            let progress_bar = Self::make_progress_bar(scene_progress, 8);
            let transition_indicator = if is_transitioning { "~" } else { "" };
            format!(" • Automix: {automix_mode} [{progress_bar}]{transition_indicator}")
        } else {
            String::new()
        };

        let status_text = format!(
            " Pattern: {pattern}{mod_indicator} • Theme: {theme}{lock_indicator} • Art: {art_str}{automix_str} • [?] help • [q] quit "
        );

        let status = Paragraph::new(status_text)
            .style(Style::default().bg(TuiColor::Blue).fg(TuiColor::White))
            .alignment(ratatui::layout::Alignment::Center);

        f.render_widget(status, status_area);
    }

    /// Create a text-based progress bar
    fn make_progress_bar(progress: f32, width: usize) -> String {
        let filled = (progress * width as f32).round() as usize;
        let empty = width.saturating_sub(filled);
        format!("{}{}", "█".repeat(filled), "░".repeat(empty))
    }
}
