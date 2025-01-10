use eframe::egui;

#[derive(Debug, PartialEq)]
pub enum DragDirection {
    Up,
    Down,
    Left,
    Right,
}

pub trait Draggable {
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
