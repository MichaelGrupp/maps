//! Extensions for MapPose to add GUI interaction capabilities.

use eframe::emath;
use maps_io_ros::MapPose;

use crate::movable::{Draggable, Rotatable};

impl Draggable for MapPose {
    fn offset_rhs(&self) -> emath::Vec2 {
        emath::vec2(self.translation.x, self.translation.y)
    }

    fn drag(&mut self, delta: emath::Vec2) {
        self.translation.x += delta.x;
        self.translation.y -= delta.y;
    }
}

impl Rotatable for MapPose {
    fn rotate(&mut self, delta: f32) {
        self.rotation.yaw = emath::normalized_angle(self.rotation.yaw + delta);
    }
}
