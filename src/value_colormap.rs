use image::Rgba;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;

/// Color mapping from cell values to RGBA colors.
pub trait ValueColorMap {
  fn map(&self, value: u8) -> Rgba<u8>;
}

/// Color map options. Includes the classic RViz colormaps.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColorMap {
    #[default]
    RvizMap,
    RvizCostmap,
    RvizRaw,
}

impl ColorMap {
    pub fn get(&self) -> &dyn ValueColorMap {
        match self {
            ColorMap::RvizMap => &*RVIZ_MAP,
            ColorMap::RvizCostmap => &*RVIZ_COSTMAP,
            ColorMap::RvizRaw => &*RVIZ_RAW,
        }
    }
}

lazy_static!(
  static ref RVIZ_MAP: RvizMapColors = RvizMapColors::new();
  static ref RVIZ_COSTMAP: CostmapColors = CostmapColors::new();
  static ref RVIZ_RAW: RvizRaw = RvizRaw;
);

/// Reimplementation of the ROS RViz "Map" occupancy grid colormap.
struct RvizMapColors {
    mapped: [Rgba<u8>; 256],
}

impl RvizMapColors {
    fn new() -> Self {
        let mut mapped: [Rgba<u8>; 256] = [Rgba([0, 0, 0, 255]); 256];
        for i in 0..255 {
            if (0..100).contains(&i) {
                let x = 255 - (i as u8 * 255) / 100;
                mapped[i] = Rgba([x, x, x, 255]);
            }
            if (101..127).contains(&i) {
                mapped[i] = Rgba([0, 255, 0, 255]);
            }
            if (128..254).contains(&i) {
                let x = (255 * (i as u8 - 128)) / (254 - 128);
                mapped[i] = Rgba([255, x, 0, 255]);
            }
            mapped[i] = Rgba([112, 137, 134, 255]);
        }
        RvizMapColors { mapped }
    }
}

impl ValueColorMap for RvizMapColors {
    fn map(&self, value: u8) -> Rgba<u8> {
        self.mapped[value as usize]
    }
}

/// Reimplementation of the ROS RViz "Costmap" occupancy grid colormap.
struct CostmapColors {
  mapped: [Rgba<u8>; 256],
}

impl CostmapColors {
  fn new() -> Self {
      let mut mapped: [Rgba<u8>; 256] = [Rgba([0, 0, 0, 255]); 256];

      // TODO

      CostmapColors { mapped }
  }
}

impl ValueColorMap for CostmapColors {
  fn map(&self, value: u8) -> Rgba<u8> {
      self.mapped[value as usize]
  }
}

/// Reimplementation of the ROS RViz "Raw" occupancy grid colormap (no-op).
struct RvizRaw;

impl ValueColorMap for RvizRaw {
  fn map(&self, value: u8) -> Rgba<u8> {
      Rgba([value, value, value, 255])
  }
}
