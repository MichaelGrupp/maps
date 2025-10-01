//! Color map implementations.
//! Includes reimplementations of the classic RViz colormaps for occupancy grids.

use image::Rgba;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

/// Trait for color mapping from cell values to RGBA colors.
pub trait ValueColorMap {
    fn map(&self, value: u8) -> Rgba<u8>;
}

/// Color map options. Includes the classic RViz colormaps.
#[derive(Debug, Display, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColorMap {
    /// "Raw" occupancy grid colormap (no-op).
    #[strum(to_string = "Raw")]
    Raw,
    /// Reimplementation of the ROS RViz "Map" occupancy grid colormap.
    #[default]
    #[strum(to_string = "RViz \"Map\"")]
    RvizMap,
    /// Reimplementation of the ROS RViz "Costmap" occupancy grid colormap.
    #[strum(to_string = "RViz \"Costmap\"")]
    RvizCostmap,
    /// An alternative costmap color map with less screaming colors.
    #[strum(to_string = "Cool Costmap")]
    CoolCostmap,
}

impl ColorMap {
    /// Gives access to the corresponding color map trait implementations
    /// that are implemented in this module.
    pub fn get(&self) -> &dyn ValueColorMap {
        match self {
            ColorMap::RvizMap => &*RVIZ_MAP,
            ColorMap::RvizCostmap => &*RVIZ_COSTMAP,
            ColorMap::Raw => &*RAW,
            ColorMap::CoolCostmap => &*COOL_COSTMAP,
        }
    }
}

lazy_static! {
    static ref RVIZ_MAP: RvizMapColors = RvizMapColors::new();
    static ref RVIZ_COSTMAP: CostmapColors = CostmapColors::new();
    static ref RAW: Raw = Raw;
    static ref COOL_COSTMAP: CoolCostmapColors = CoolCostmapColors::new();
}

struct RvizMapColors {
    mapped: [Rgba<u8>; 256],
}

impl RvizMapColors {
    fn new() -> Self {
        let mut mapped: [Rgba<u8>; 256] = [Rgba([0, 0, 0, 255]); 256];
        for (i, value) in mapped.iter_mut().enumerate() {
            if (0..101).contains(&i) {
                let x = (255. - (i as f32 * 255.) / 100.) as u8;
                *value = Rgba([x, x, x, 255]);
            } else if (101..128).contains(&i) {
                *value = Rgba([0, 255, 0, 255]);
            } else if (128..255).contains(&i) {
                let x = ((255. * (i as f32 - 128.)) / (254. - 128.)) as u8;
                *value = Rgba([255, x, 0, 255]);
            } else {
                *value = Rgba([112, 137, 134, 255]);
            }
        }
        RvizMapColors { mapped }
    }
}

impl ValueColorMap for RvizMapColors {
    fn map(&self, value: u8) -> Rgba<u8> {
        self.mapped[value as usize]
    }
}

struct CostmapColors {
    mapped: [Rgba<u8>; 256],
}

impl CostmapColors {
    fn new() -> Self {
        let mut mapped: [Rgba<u8>; 256] = [Rgba([0, 0, 0, 255]); 256];
        for (i, value) in mapped.iter_mut().enumerate() {
            if i == 0 {
                *value = Rgba([0, 0, 0, 0]);
            } else if (1..99).contains(&i) {
                let x = (i as f32 * 255. / 100.) as u8;
                *value = Rgba([x, 0, 255 - x, 255]);
            } else if i == 99 {
                *value = Rgba([0, 255, 255, 255]);
            } else if i == 100 {
                *value = Rgba([255, 0, 255, 255]);
            } else if (101..128).contains(&i) {
                *value = Rgba([0, 255, 0, 255]);
            } else if (128..255).contains(&i) {
                let x = ((255. * (i as f32 - 128.)) / (254. - 128.)) as u8;
                *value = Rgba([255, x, 0, 255]);
            } else {
                *value = Rgba([112, 137, 134, 255]);
            }
        }
        CostmapColors { mapped }
    }
}

impl ValueColorMap for CostmapColors {
    fn map(&self, value: u8) -> Rgba<u8> {
        self.mapped[value as usize]
    }
}

struct Raw;

impl ValueColorMap for Raw {
    fn map(&self, value: u8) -> Rgba<u8> {
        Rgba([value, value, value, 255])
    }
}

struct CoolCostmapColors {
    mapped: [Rgba<u8>; 256],
}

impl CoolCostmapColors {
    pub fn new() -> Self {
        let mut mapped: [Rgba<u8>; 256] = [Rgba([0, 0, 0, 255]); 256];
        for (i, value) in mapped.iter_mut().enumerate() {
            if i == 0 {
                *value = Rgba([0, 0, 0, 0]);
            } else if (1..99).contains(&i) {
                let r = (38. + (i as f32 * 129.) / 100.) as u8;
                let g = (55. + (i as f32 * 171.) / 100.) as u8;
                let b = (59. + (i as f32 * 134.) / 100.) as u8;
                *value = Rgba([r, g, b, 255]);
            } else if i == 99 {
                *value = Rgba([38, 55, 59, 255]);
            } else if i == 100 {
                *value = Rgba([255, 252, 93, 255]);
            } else if (101..128).contains(&i) {
                *value = Rgba([0, 255, 0, 255]);
            } else if (128..255).contains(&i) {
                let x = ((255. * (i as f32 - 128.)) / (254. - 128.)) as u8;
                *value = Rgba([255, x, 0, 255]);
            } else {
                *value = Rgba([112, 137, 134, 255]);
            }
        }
        CoolCostmapColors { mapped }
    }
}

impl ValueColorMap for CoolCostmapColors {
    fn map(&self, value: u8) -> Rgba<u8> {
        self.mapped[value as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rviz_map_colors() {
        let palette = RvizMapColors::new();
        assert_eq!(palette.map(0), Rgba([255, 255, 255, 255]));
        assert_eq!(palette.map(100), Rgba([0, 0, 0, 255]));
        assert_eq!(palette.map(101), Rgba([0, 255, 0, 255]));
        assert_eq!(palette.map(128), Rgba([255, 0, 0, 255]));
        assert_eq!(palette.map(254), Rgba([255, 255, 0, 255]));
        assert_eq!(palette.map(255), Rgba([112, 137, 134, 255]));
    }

    #[test]
    fn test_costmap_colors() {
        let palette = CostmapColors::new();
        assert_eq!(palette.map(0), Rgba([0, 0, 0, 0]));
        assert_eq!(palette.map(1), Rgba([2, 0, 253, 255]));
        assert_eq!(palette.map(99), Rgba([0, 255, 255, 255]));
        assert_eq!(palette.map(100), Rgba([255, 0, 255, 255]));
        assert_eq!(palette.map(101), Rgba([0, 255, 0, 255]));
        assert_eq!(palette.map(128), Rgba([255, 0, 0, 255]));
        assert_eq!(palette.map(234), Rgba([255, 214, 0, 255]));
        assert_eq!(palette.map(254), Rgba([255, 255, 0, 255]));
        assert_eq!(palette.map(255), Rgba([112, 137, 134, 255]));
    }

    #[test]
    fn test_rviz_raw_colors() {
        let palette = Raw;
        assert_eq!(palette.map(0), Rgba([0, 0, 0, 255]));
        assert_eq!(palette.map(100), Rgba([100, 100, 100, 255]));
        assert_eq!(palette.map(255), Rgba([255, 255, 255, 255]));
    }
}
