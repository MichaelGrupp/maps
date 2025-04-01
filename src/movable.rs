use eframe::egui;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
pub enum DragDirection {
    Up,
    Down,
    Left,
    Right,
}

pub trait Draggable {
    // Offset in a RHS coordinate system.
    fn offset_rhs(&self) -> egui::Vec2;

    fn drag(&mut self, delta: egui::Vec2);

    fn drag_directed(&mut self, amount: f32, direction: DragDirection) {
        let delta = match direction {
            DragDirection::Up => egui::vec2(0., -amount),
            DragDirection::Down => egui::vec2(0., amount),
            DragDirection::Left => egui::vec2(-amount, 0.),
            DragDirection::Right => egui::vec2(amount, 0.),
        };
        self.drag(delta);
    }
}

pub trait Rotatable {
    fn rotate(&mut self, delta: f32);

    fn rotate_directed(&mut self, amount: f32, direction: DragDirection) {
        let delta = match direction {
            DragDirection::Up => amount,
            DragDirection::Down => -amount,
            DragDirection::Left => amount,
            DragDirection::Right => -amount,
        };
        self.rotate(delta);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MovableAmounts {
    pub drag: f32,
    pub rotate: f32,
}

impl Default for MovableAmounts {
    fn default() -> Self {
        MovableAmounts::PRESET_COARSE
    }
}

impl MovableAmounts {
    pub const PRESET_FINE: Self = Self {
        drag: 0.01,
        rotate: 0.001,
    };
    pub const PRESET_MEDIUM: Self = Self {
        drag: 0.1,
        rotate: 0.01,
    };
    pub const PRESET_COARSE: Self = Self {
        drag: 1.0,
        rotate: 0.1,
    };
}
