use eframe::egui;
use egui_file_dialog::DialogState;

use crate::app::{ActiveMovable, ActiveTool, AppState};
use crate::app_impl::screenshot;
use crate::movable::{DragDirection, Draggable, Rotatable};

impl AppState {
    fn dialogs_open(&self) -> bool {
        self.load_meta_file_dialog.state() == DialogState::Open
            || self.load_map_pose_file_dialog.state() == DialogState::Open
            || self.save_map_pose_file_dialog.state() == DialogState::Open
            || self.load_session_file_dialog.state() == DialogState::Open
            || self.save_session_file_dialog.state() == DialogState::Open
            || self.save_screenshot_dialog.state() == DialogState::Open
            || self.status.quit_modal_active
            || !self.status.error.is_empty()
    }

    fn text_editing(&self) -> bool {
        self.options.pose_edit.edit_map_frame || self.options.pose_edit.edit_root_frame
    }

    pub(crate) fn handle_key_shortcuts(&mut self, ui: &egui::Ui) {
        if self.dialogs_open() || self.text_editing() {
            self.status.move_action = None;
            return;
        }

        let mut screenshot_request: Option<screenshot::Viewport> = None;
        ui.input(|i| {
            if i.key_released(egui::Key::Escape) {
                self.options.menu_visible = false;
                self.options.settings_visible = false;
                self.options.help_visible = false;
                self.options.active_tool = ActiveTool::None;
            } else if i.key_released(egui::Key::L) || i.pointer.secondary_released() {
                if self.options.active_tool == ActiveTool::HoverLens {
                    self.options.active_tool = ActiveTool::None;
                } else {
                    self.options.active_tool = ActiveTool::HoverLens;
                }
            }
            if i.key_released(egui::Key::M) {
                self.options.menu_visible = !self.options.menu_visible;
            }
            if i.key_released(egui::Key::O) {
                self.options.settings_visible = !self.options.settings_visible;
            }
            if i.key_released(egui::Key::G) {
                self.options.grid.lines_visible = !self.options.grid.lines_visible;
            }

            // Screenshot shortcuts.
            if i.modifiers.shift && i.key_released(egui::Key::P) {
                screenshot_request = Some(screenshot::Viewport::Clipped);
            } else if i.key_released(egui::Key::P) {
                screenshot_request = Some(screenshot::Viewport::Full);
            }

            // Get the obects that can be currently dragged.
            let drag_amount = match self.options.active_movable {
                ActiveMovable::MapPose => self.options.pose_edit.movable_amounts.drag,
                ActiveMovable::Grid => self.options.grid.movable_amounts.drag,
                _ => Default::default(),
            };
            let draggable: Option<&mut dyn Draggable> = match self.options.active_movable {
                ActiveMovable::MapPose => {
                    match self
                        .data
                        .maps
                        .get_mut(self.options.pose_edit.selected_map.as_str())
                    {
                        Some(map) => Some(&mut map.pose),
                        None => None,
                    }
                }
                ActiveMovable::Grid => Some(&mut self.options.grid),
                _ => None,
            };

            if let Some(draggable) = draggable {
                let previous_offset = draggable.offset_rhs();
                if i.key_down(egui::Key::W) {
                    draggable.drag_directed(drag_amount, DragDirection::Up);
                }
                if i.key_down(egui::Key::A) {
                    draggable.drag_directed(drag_amount, DragDirection::Left);
                }
                if i.key_down(egui::Key::S) {
                    draggable.drag_directed(drag_amount, DragDirection::Down);
                }
                if i.key_down(egui::Key::D) {
                    draggable.drag_directed(drag_amount, DragDirection::Right);
                }
                let delta = draggable.offset_rhs() - previous_offset;
                if delta.x != 0. && delta.y == 0. {
                    self.status.move_action = Some("⬌".to_string());
                } else if delta.x == 0. && delta.y != 0. {
                    self.status.move_action = Some("⬍".to_string());
                } else if delta.x > 0. && delta.y > 0. {
                    self.status.move_action = Some("⬈".to_string());
                } else if delta.x < 0. && delta.y > 0. {
                    self.status.move_action = Some("⬉".to_string());
                } else if delta.x < 0. && delta.y < 0. {
                    self.status.move_action = Some("⬋".to_string());
                } else if delta.x > 0. && delta.y < 0. {
                    self.status.move_action = Some("⬊".to_string());
                } else {
                    self.status.move_action = None;
                }
            }

            let rotatable: Option<&mut dyn Rotatable> = match self.options.active_movable {
                ActiveMovable::MapPose => {
                    match self
                        .data
                        .maps
                        .get_mut(self.options.pose_edit.selected_map.as_str())
                    {
                        Some(map) => Some(&mut map.pose),
                        None => None,
                    }
                }
                _ => None,
            };

            let rotation_amount = self.options.pose_edit.movable_amounts.rotate;
            if let Some(rotatable) = rotatable {
                if i.key_down(egui::Key::Q) {
                    rotatable.rotate_directed(rotation_amount, DragDirection::Left);
                    self.status.move_action = Some("⟲".to_string());
                } else if i.key_down(egui::Key::E) {
                    rotatable.rotate_directed(rotation_amount, DragDirection::Right);
                    self.status.move_action = Some("⟳".to_string());
                }
            }

            if i.key_down(egui::Key::Minus) {
                self.options
                    .grid
                    .zoom(-self.options.grid.scroll_delta_percent);
                self.status.move_action = Some("-".to_string());
            }
            if i.key_down(egui::Key::Plus) {
                self.options
                    .grid
                    .zoom(self.options.grid.scroll_delta_percent);
                self.status.move_action = Some("+".to_string());
            }
        });

        if let Some(viewport) = screenshot_request {
            // Has to be called here outside of the input closure to not block.
            self.request_screenshot(ui, viewport);
        }
    }
}
