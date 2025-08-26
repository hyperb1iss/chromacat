use crate::renderer::playground::PlaygroundUI;
use crate::renderer::RendererError;
/// Input handling for the renderer
/// Separates input logic from rendering logic
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};

/// Input event types
pub enum InputEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
}

/// Handles input for playground UI
pub struct PlaygroundInputHandler;

impl PlaygroundInputHandler {
    /// Handle keyboard input for playground
    pub fn handle_key(ui: &mut PlaygroundUI, key: KeyEvent) -> Result<InputAction, RendererError> {
        match key.code {
            // Quit keys
            KeyCode::Char('q') | KeyCode::Esc => Ok(InputAction::Quit),
            // Toggle overlay
            KeyCode::Char(';') => {
                ui.overlay_visible = !ui.overlay_visible;
                let msg = if ui.overlay_visible {
                    "Overlay: ON"
                } else {
                    "Overlay: OFF"
                };
                ui.show_toast(msg);
                Ok(InputAction::Redraw)
            }

            // Automix controls
            KeyCode::Char('a') | KeyCode::Char('A') => Ok(InputAction::AutomixToggle),
            KeyCode::Char('1') => Ok(InputAction::AutomixMode("off".to_string())),
            KeyCode::Char('2') => Ok(InputAction::AutomixMode("random".to_string())),
            KeyCode::Char('3') => Ok(InputAction::AutomixMode("showcase".to_string())),
            KeyCode::Char('4') => Ok(InputAction::AutomixMode("playlist".to_string())),
            KeyCode::Char('5') => Ok(InputAction::AutomixMode("adaptive".to_string())),
            KeyCode::Char('.') | KeyCode::Char('>') => Ok(InputAction::AutomixNext),
            KeyCode::Char(',') | KeyCode::Char('<') => Ok(InputAction::AutomixPrev),

            // Navigate sections
            KeyCode::Tab => {
                if ui.overlay_visible {
                    ui.active_section = (ui.active_section + 1) % 4;
                    Ok(InputAction::Redraw)
                } else {
                    Ok(InputAction::None)
                }
            }

            // Navigate within section
            KeyCode::Up => {
                if ui.overlay_visible {
                    match ui.active_section {
                        0 => {
                            if ui.pattern_sel > 0 {
                                ui.pattern_sel -= 1;
                            }
                        }
                        1 => {
                            if ui.param_sel > 0 {
                                ui.param_sel -= 1;
                            }
                        }
                        2 => {
                            if ui.theme_sel > 0 {
                                ui.theme_sel -= 1;
                            }
                        }
                        3 => {
                            if ui.art_sel > 0 {
                                ui.art_sel -= 1;
                            }
                        }
                        _ => {}
                    }
                    Ok(InputAction::Redraw)
                } else {
                    Ok(InputAction::None)
                }
            }

            KeyCode::Down => {
                if ui.overlay_visible {
                    match ui.active_section {
                        0 => {
                            if ui.pattern_sel + 1 < ui.pattern_names.len() {
                                ui.pattern_sel += 1;
                            }
                        }
                        1 => {
                            if ui.param_sel + 1 < ui.param_names.len() {
                                ui.param_sel += 1;
                            }
                        }
                        2 => {
                            if ui.theme_sel + 1 < ui.theme_names.len() {
                                ui.theme_sel += 1;
                            }
                        }
                        3 => {
                            if ui.art_sel + 1 < ui.art_names.len() {
                                ui.art_sel += 1;
                            }
                        }
                        _ => {}
                    }
                    Ok(InputAction::Redraw)
                } else {
                    Ok(InputAction::None)
                }
            }

            KeyCode::Enter => {
                if ui.overlay_visible {
                    match ui.active_section {
                        0 => {
                            if let Some(pattern) = ui.pattern_names.get(ui.pattern_sel) {
                                Ok(InputAction::ApplyPattern(pattern.clone()))
                            } else {
                                Ok(InputAction::None)
                            }
                        }
                        2 => {
                            if let Some(theme) = ui.theme_names.get(ui.theme_sel) {
                                Ok(InputAction::ApplyTheme(theme.clone()))
                            } else {
                                Ok(InputAction::None)
                            }
                        }
                        3 => {
                            if let Some(art) = ui.art_names.get(ui.art_sel) {
                                Ok(InputAction::ApplyArt(art.clone()))
                            } else {
                                Ok(InputAction::None)
                            }
                        }
                        _ => Ok(InputAction::None),
                    }
                } else {
                    Ok(InputAction::None)
                }
            }

            // Add arrow key navigation between columns
            KeyCode::Left => {
                if ui.overlay_visible && ui.active_section > 0 {
                    ui.active_section -= 1;
                    Ok(InputAction::Redraw)
                } else {
                    Ok(InputAction::None)
                }
            }

            KeyCode::Right => {
                if ui.overlay_visible && ui.active_section < 3 {
                    ui.active_section += 1;
                    Ok(InputAction::Redraw)
                } else {
                    Ok(InputAction::None)
                }
            }

            _ => Ok(InputAction::None),
        }
    }

    /// Handle mouse input for playground
    pub fn handle_mouse(
        ui: &mut PlaygroundUI,
        event: MouseEvent,
    ) -> Result<InputAction, RendererError> {
        use crossterm::event::{MouseButton, MouseEventKind};

        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if ui.overlay_visible {
                    // Calculate which section was clicked based on coordinates
                    // This is simplified - actual implementation would need to know the exact layout
                    let section = Self::get_section_from_coords(event.column, event.row, ui);
                    if let Some(section) = section {
                        ui.active_section = section;

                        // Calculate which item in the list was clicked
                        if let Some(item_index) =
                            Self::get_item_from_coords(event.column, event.row, ui, section)
                        {
                            match section {
                                0 => {
                                    ui.pattern_sel = item_index;
                                    if let Some(pattern) = ui.pattern_names.get(item_index) {
                                        return Ok(InputAction::ApplyPattern(pattern.clone()));
                                    }
                                }
                                1 => {
                                    ui.param_sel = item_index;
                                }
                                2 => {
                                    ui.theme_sel = item_index;
                                    if let Some(theme) = ui.theme_names.get(item_index) {
                                        return Ok(InputAction::ApplyTheme(theme.clone()));
                                    }
                                }
                                3 => {
                                    ui.art_sel = item_index;
                                    if let Some(art) = ui.art_names.get(item_index) {
                                        return Ok(InputAction::ApplyArt(art.clone()));
                                    }
                                }
                                _ => {}
                            }
                        }
                        return Ok(InputAction::Redraw);
                    }
                }
                Ok(InputAction::None)
            }

            MouseEventKind::ScrollUp => {
                if ui.overlay_visible {
                    // Scroll up in the active section
                    match ui.active_section {
                        0 => {
                            if ui.pattern_sel > 0 {
                                ui.pattern_sel -= 1;
                            }
                        }
                        1 => {
                            if ui.param_sel > 0 {
                                ui.param_sel -= 1;
                            }
                        }
                        2 => {
                            if ui.theme_sel > 0 {
                                ui.theme_sel -= 1;
                            }
                        }
                        3 => {
                            if ui.art_sel > 0 {
                                ui.art_sel -= 1;
                            }
                        }
                        _ => {}
                    }
                    Ok(InputAction::Redraw)
                } else {
                    Ok(InputAction::None)
                }
            }

            MouseEventKind::ScrollDown => {
                if ui.overlay_visible {
                    // Scroll down in the active section
                    match ui.active_section {
                        0 => {
                            if ui.pattern_sel + 1 < ui.pattern_names.len() {
                                ui.pattern_sel += 1;
                            }
                        }
                        1 => {
                            if ui.param_sel + 1 < ui.param_names.len() {
                                ui.param_sel += 1;
                            }
                        }
                        2 => {
                            if ui.theme_sel + 1 < ui.theme_names.len() {
                                ui.theme_sel += 1;
                            }
                        }
                        3 => {
                            if ui.art_sel + 1 < ui.art_names.len() {
                                ui.art_sel += 1;
                            }
                        }
                        _ => {}
                    }
                    Ok(InputAction::Redraw)
                } else {
                    Ok(InputAction::None)
                }
            }

            _ => Ok(InputAction::None),
        }
    }

    /// Helper to determine which section was clicked
    fn get_section_from_coords(x: u16, y: u16, ui: &PlaygroundUI) -> Option<usize> {
        // Calculate overlay area (bottom 1/4 of screen)
        let panel_height = (ui.terminal_size.1 / 4).clamp(10, 20);
        let panel_y = ui
            .terminal_size
            .1
            .saturating_sub(panel_height)
            .saturating_sub(1);

        // Check if we're in the overlay area
        if y >= panel_y && y < panel_y + panel_height {
            // Divide width into 4 equal columns
            let column_width = ui.terminal_size.0 / 4;
            let section = (x / column_width) as usize;
            if section < 4 {
                return Some(section);
            }
        }
        None
    }

    /// Helper to determine which item in a list was clicked
    fn get_item_from_coords(_x: u16, y: u16, ui: &PlaygroundUI, section: usize) -> Option<usize> {
        // Calculate overlay area dimensions
        let panel_height = (ui.terminal_size.1 / 4).clamp(10, 20);
        let panel_y = ui
            .terminal_size
            .1
            .saturating_sub(panel_height)
            .saturating_sub(1);

        // Account for borders and headers (3 lines from top of panel)
        let list_start_y = panel_y + 3;
        let list_height = panel_height.saturating_sub(5); // 3 for header, 2 for footer

        // Check if y is within the list area
        if y >= list_start_y && y < list_start_y + list_height {
            let relative_y = (y - list_start_y) as usize;

            // Get the offset for the section
            let offset = match section {
                0 => ui.pattern_offset,
                1 => ui.param_offset,
                2 => ui.theme_offset,
                3 => ui.art_offset,
                _ => 0,
            };

            // Calculate actual item index
            let item_index = offset + relative_y;

            // Validate it's within bounds
            let max_items = match section {
                0 => ui.pattern_names.len(),
                1 => ui.param_names.len(),
                2 => ui.theme_names.len(),
                3 => ui.art_names.len(),
                _ => 0,
            };

            if item_index < max_items {
                Some(item_index)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Actions resulting from input
pub enum InputAction {
    None,
    Redraw,
    ApplyPattern(String),
    ApplyTheme(String),
    ApplyArt(String),
    AdjustParam { name: String, value: f64 },
    // Automix controls
    AutomixToggle,
    AutomixMode(String),
    AutomixNext,
    AutomixPrev,
    Quit,
}
