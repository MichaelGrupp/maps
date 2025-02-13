/// Value thresholding options.
/// Mostly follows ROS: wiki.ros.org/map_server#Value_Interpretation
/// Since the nav map_server slightly deviates from that documentation,
/// it's also possible to mimic its behavior.
use serde::{Deserialize, Serialize};

const TRINARY_FREE: u8 = 0;
const TRINARY_OCCUPIED: u8 = 100;
const TRINARY_UNKNOWN: u8 = 255;

const MAP_SERVER_FREE_DEFAULT: f32 = 0.196;
const MAP_SERVER_OCCUPIED_DEFAULT: f32 = 0.65;

use image::{DynamicImage, Rgba};
use imageproc::{integral_image::ArrayData, map::map_colors_mut};

use crate::meta::MetaYaml;
use crate::value_colormap::ColorMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Raw,
    Trinary,
    Scale,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Quirks {
    Ros1Wiki, // Interpret values as documented in ROS 1 Wiki.

    // ROS 1 map_server behaves differently than documented.
    // At this point, probably everyone is used to the map_server quirks.
    #[default]
    Ros1MapServer,
    Ros2MapServer, // TODO: same as ROS 1?
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValueInterpretation {
    pub free: f32,
    pub occupied: f32,
    pub negate: bool,
    pub mode: Mode,
    pub quirks: Quirks,
    #[serde(default)]
    pub colormap: ColorMap,
}

impl Default for ValueInterpretation {
    fn default() -> Self {
        ValueInterpretation {
            free: MAP_SERVER_FREE_DEFAULT,
            occupied: MAP_SERVER_OCCUPIED_DEFAULT,
            negate: false,
            mode: Mode::default(),
            quirks: Quirks::default(),
            colormap: ColorMap::default(),
        }
    }
}

impl ValueInterpretation {
    pub fn new(free: f32, occupied: f32, negate: bool, mode: Option<Mode>) -> Self {
        ValueInterpretation {
            free,
            occupied,
            negate,
            mode: mode.unwrap_or_default(),
            quirks: Quirks::default(),
            colormap: ColorMap::default(),
        }
    }

    pub fn from_meta_yaml(meta: &MetaYaml) -> Self {
        ValueInterpretation::new(
            meta.free_thresh,
            meta.occupied_thresh,
            meta.negate != 0,
            meta.mode,
        )
    }

    /// Allows to mimic the wonderful undocumented behaviors of map_server.
    pub fn with_quirks(mut self, quirks: Quirks) -> Self {
        self.quirks = quirks;
        self
    }

    pub fn with_colormap(mut self, colormap: ColorMap) -> Self {
        self.colormap = colormap;
        self
    }

    /// Modifies the image according to the value interpretation.
    /// Note that the output needs a color map for visualization.
    /// Without color map, the image corresponds to the "raw" display of RViz.
    ///
    /// The "original_has_alpha" parameter is used to determine if the source
    /// image had an alpha channel. This is necessary for some implementation quirks.
    pub fn apply(&self, img: &mut DynamicImage, original_has_alpha: bool) {
        match self.mode {
            Mode::Raw => {}
            Mode::Trinary | Mode::Scale => {
                map_colors_mut(img, |c| {
                    self.colormap
                        .get()
                        .map(self.interpret(&c, original_has_alpha)[0])
                });
            }
        }
    }

    fn avg_float(&self, pixel: &Rgba<u8>, has_alpha: bool) -> f32 {
        let num_channels = match self.quirks {
            // Nothing documented about alpha averaging in ROS 1 Wiki.
            Quirks::Ros1Wiki => 3,
            // "Alpha will be averaged in with color channels when using trinary mode."
            // ROS 1: https://github.com/ros-planning/navigation/blob/9ad644198e132d0e950579a3bc72c29da46e60b0/map_server/src/image_loader.cpp#L106C3-L106C76
            // ROS 2: https://github.com/ros-navigation/navigation2/blob/088c423deb97a76f5a5f4ca133cb122338576fe1/nav2_map_server/src/map_io.cpp#L236
            Quirks::Ros1MapServer | Quirks::Ros2MapServer => {
                if self.mode == Mode::Trinary && has_alpha {
                    4
                } else {
                    3
                }
            }
        };
        let sum = pixel.data()[0..num_channels]
            .iter()
            .map(|&v| v as f32)
            .sum::<f32>();
        let avg = sum / num_channels as f32;
        if self.negate {
            return avg / 255.;
        }
        (255. - avg) / 255.
    }

    fn interpret(&self, pixel: &Rgba<u8>, has_alpha: bool) -> Rgba<u8> {
        let p = self.avg_float(pixel, has_alpha);
        let alpha = pixel[3];

        // In scale mode, any pixel with transparency is considered unknown.
        let scale_unknown = self.mode == Mode::Scale && alpha != 255;

        if p > self.occupied && !scale_unknown {
            Rgba([TRINARY_OCCUPIED, TRINARY_OCCUPIED, TRINARY_OCCUPIED, alpha])
        } else if p < self.free && !scale_unknown {
            Rgba([TRINARY_FREE, TRINARY_FREE, TRINARY_FREE, alpha])
        } else if self.mode == Mode::Trinary || scale_unknown {
            // In trinary mode, any pixel that is not occupied or free is considered unknown.
            // In scale mode, any pixel with transparency is considered unknown.
            Rgba([TRINARY_UNKNOWN, TRINARY_UNKNOWN, TRINARY_UNKNOWN, alpha])
        }
        // Scale
        else {
            let scaled = match self.quirks {
                Quirks::Ros1Wiki => {
                    // wiki.ros.org/map_server#Value_Interpretation
                    (99. * (p - self.free) / (self.occupied - self.free)) as u8
                }
                Quirks::Ros1MapServer | Quirks::Ros2MapServer => {
                    // ROS 1: https://github.com/ros-planning/navigation/blob/9ad644198e132d0e950579a3bc72c29da46e60b0/map_server/src/image_loader.cpp#L155
                    // ROS 2:
                    (1. + 98. * (p - self.free) / (self.occupied - self.free)) as u8
                }
            };
            Rgba([scaled, scaled, scaled, alpha])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GenericImage, GenericImageView};

    const EPS: f32 = 1e-3;

    #[test]
    fn avg_float() {
        let thresholding = ValueInterpretation::new(0.196, 0.65, false, None);

        let pixel = Rgba([128, 128, 128, 255]);
        assert!(thresholding.avg_float(&pixel, false) - 0.5 < EPS);

        let pixel = Rgba([255, 255, 255, 255]);
        assert_eq!(thresholding.avg_float(&pixel, false), 0.);

        let pixel = Rgba([0, 0, 0, 255]);
        assert_eq!(thresholding.avg_float(&pixel, false), 1.);
    }

    #[test]
    fn trinary() {
        let thresholding = ValueInterpretation::new(0.196, 0.65, false, Some(Mode::Trinary));
        let mut img = DynamicImage::new_rgba8(1, 1);

        img.put_pixel(0, 0, Rgba([128, 128, 128, 255]));
        thresholding.apply(&mut img, false);
        assert_eq!(
            img.get_pixel(0, 0),
            Rgba([TRINARY_UNKNOWN, TRINARY_UNKNOWN, TRINARY_UNKNOWN, 255])
        );

        img.put_pixel(0, 0, Rgba([255, 255, 255, 255]));
        thresholding.apply(&mut img, false);
        assert_eq!(
            img.get_pixel(0, 0),
            Rgba([TRINARY_FREE, TRINARY_FREE, TRINARY_FREE, 255])
        );

        img.put_pixel(0, 0, Rgba([60, 60, 60, 255]));
        thresholding.apply(&mut img, false);
        assert_eq!(
            img.get_pixel(0, 0),
            Rgba([TRINARY_OCCUPIED, TRINARY_OCCUPIED, TRINARY_OCCUPIED, 255])
        );
    }

    #[test]
    fn scale() {
        let thresholding = ValueInterpretation::new(0.196, 0.65, false, Some(Mode::Scale));
        let mut img = DynamicImage::new_rgba8(1, 1);

        img.put_pixel(0, 0, Rgba([128, 128, 128, 255]));
        thresholding.apply(&mut img, true);
        assert_eq!(img.get_pixel(0, 0), Rgba([65, 65, 65, 255]));

        img.put_pixel(0, 0, Rgba([60, 60, 60, 255]));
        thresholding.apply(&mut img, true);
        assert_eq!(
            img.get_pixel(0, 0),
            Rgba([TRINARY_OCCUPIED, TRINARY_OCCUPIED, TRINARY_OCCUPIED, 255])
        );

        img.put_pixel(0, 0, Rgba([255, 255, 255, 255]));
        thresholding.apply(&mut img, true);
        assert_eq!(
            img.get_pixel(0, 0),
            Rgba([TRINARY_FREE, TRINARY_FREE, TRINARY_FREE, 255])
        );

        // Any pixel with transparency is considered unknown here.
        img.put_pixel(0, 0, Rgba([1, 2, 3, 100]));
        thresholding.apply(&mut img, true);
        assert_eq!(
            img.get_pixel(0, 0),
            Rgba([TRINARY_UNKNOWN, TRINARY_UNKNOWN, TRINARY_UNKNOWN, 255])
        );
    }

    #[test]
    fn scale_map_server_quirks() {
        let thresholding = ValueInterpretation::new(0.196, 0.65, false, Some(Mode::Scale))
            .with_quirks(Quirks::Ros1MapServer);
        let mut img = DynamicImage::new_rgba8(1, 1);

        img.put_pixel(0, 0, Rgba([128, 128, 128, 255]));
        thresholding.apply(&mut img, true);
        assert_eq!(img.get_pixel(0, 0), Rgba([66, 66, 66, 255]));

        img.put_pixel(0, 0, Rgba([60, 60, 60, 255]));
        thresholding.apply(&mut img, true);
        assert_eq!(
            img.get_pixel(0, 0),
            Rgba([TRINARY_OCCUPIED, TRINARY_OCCUPIED, TRINARY_OCCUPIED, 255])
        );

        img.put_pixel(0, 0, Rgba([255, 255, 255, 255]));
        thresholding.apply(&mut img, true);
        assert_eq!(
            img.get_pixel(0, 0),
            Rgba([TRINARY_FREE, TRINARY_FREE, TRINARY_FREE, 255])
        );

        // Any pixel with transparency is considered unknown here.
        img.put_pixel(0, 0, Rgba([1, 2, 3, 100]));
        thresholding.apply(&mut img, true);
        assert_eq!(
            img.get_pixel(0, 0),
            Rgba([TRINARY_UNKNOWN, TRINARY_UNKNOWN, TRINARY_UNKNOWN, 255])
        );
    }
}
