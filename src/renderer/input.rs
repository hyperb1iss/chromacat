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
        _ui: &mut PlaygroundUI,
        _event: MouseEvent,
    ) -> Result<InputAction, RendererError> {
        // TODO: Implement mouse handling
        Ok(InputAction::None)
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
    Quit,
}
