/// Playground UI implementation using ratatui
/// This module handles all playground-specific rendering and interaction
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color as TuiColor, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Terminal,
};
use std::io::Stdout;
use std::time::{Duration, Instant};

use crate::debug_log::debug_log;
use crate::pattern::PatternEngine;
use crate::renderer::{error::RendererError, pattern_widget::PatternWidget};

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

    /// Toast message
    toast_message: Option<(String, Instant)>,
    toast_duration: Duration,
    
    /// Terminal size for mouse handling
    pub terminal_size: (u16, u16),
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
            toast_message: None,
            toast_duration: Duration::from_secs(2),
            terminal_size: (80, 24),
        }
    }

    /// Ensure terminal is initialized
    pub fn ensure_terminal(&mut self) -> Result<(), RendererError> {
        if self.terminal.is_none() {
            let backend = CrosstermBackend::new(std::io::stdout());
            let terminal = Terminal::new(backend)
                .map_err(|e| RendererError::Other(format!("Failed to create terminal: {}", e)))?;
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

    /// Render the complete playground frame
    pub fn render(
        &mut self,
        content: &str,
        engine: &PatternEngine,
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
            let size = f.size();

            // First render the pattern as background
            let pattern_widget = PatternWidget::new(content, engine, time);
            f.render_widget(pattern_widget, size);

            // Then render overlay on top if visible
            if let Some(ref data) = overlay_data {
                Self::render_overlay_static(f, size, data);
            }

            // Render toast if present
            if let Some((ref text, _)) = toast_message {
                Self::render_toast_static(f, size, text);
            }

            // Render status bar at bottom
            Self::render_status_bar(f, size);
        })
        .map_err(|e| {
            let _ = debug_log(&format!("Ratatui draw error: {}", e));
            RendererError::Other(format!("Draw failed: {}", e))
        })?;

        Ok(())
    }

    /// Render the overlay UI (static version)
    fn render_overlay_static(f: &mut ratatui::Frame, size: Rect, data: &OverlayRenderData) {
        // Create a bottom panel that takes up 1/4 of the screen height
        let panel_height = (size.height / 4).max(10).min(20); // 1/4 height, min 10, max 20
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

    /// Handle terminal resize
    pub fn resize(&mut self) -> Result<(), RendererError> {
        if let Some(term) = &mut self.terminal {
            term.autoresize()
                .map_err(|e| RendererError::Other(format!("Failed to resize: {}", e)))?;
        }
        Ok(())
    }

    /// Render status bar at bottom
    fn render_status_bar(f: &mut ratatui::Frame, size: Rect) {
        let status_area = Rect {
            x: 0,
            y: size.height.saturating_sub(1),
            width: size.width,
            height: 1,
        };

        // Clear the status bar area
        f.render_widget(Clear, status_area);

        // Create status text
        let status_text =
            format!(" ChromaCat • Pattern: diagonal • Theme: terminal • [?] help • [q] quit ");

        let status = Paragraph::new(status_text)
            .style(Style::default().bg(TuiColor::Blue).fg(TuiColor::White))
            .alignment(ratatui::layout::Alignment::Center);

        f.render_widget(status, status_area);
    }
}
