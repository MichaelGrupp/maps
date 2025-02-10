/// Value thresholding options.
/// Mostly follows ROS: wiki.ros.org/map_server#Value_Interpretation
use serde::{Deserialize, Serialize};

const TRINARY_FREE: u8 = 0;
const TRINARY_OCCUPIED: u8 = 100;
const TRINARY_UNKNOWN: u8 = 255;

const SCALE_MAGIC: f32 = 99.;

use image::{DynamicImage, Rgba};
use imageproc::map::map_colors_mut;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Raw,
    Trinary,
    Scale,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct ValueInterpretation {
    free: f32,
    occupied: f32,
    negate: bool,
    mode: Mode,
}

impl ValueInterpretation {
    pub fn new(free: f32, occupied: f32, negate: bool, mode: Option<Mode>) -> Self {
        ValueInterpretation {
            free,
            occupied,
            negate,
            mode: mode.unwrap_or(Mode::default()),
        }
    }

    /// Modifies the image according to the value interpretation.
    /// Note that the output needs a color map for visualization.
    /// Without color map, the image corresponds to the "raw" display of RViz.
    pub fn apply(&self, img: &mut DynamicImage) {
        match self.mode {
            Mode::Raw => {}
            Mode::Trinary | Mode::Scale => {
                map_colors_mut(img, |c| self.interpret(&c));
            }
        }
    }

    fn to_avg_float(&self, pixel: &Rgba<u8>) -> f32 {
        let avg = (pixel[0] as f32 + pixel[1] as f32 + pixel[2] as f32) / 3.0;
        if self.negate {
            return avg / 255.;
        }
        (255. - avg) / 255.
    }

    fn interpret(&self, pixel: &Rgba<u8>) -> Rgba<u8> {
        let p = self.to_avg_float(pixel);
        if p > self.occupied {
            Rgba([
                TRINARY_OCCUPIED,
                TRINARY_OCCUPIED,
                TRINARY_OCCUPIED,
                pixel[3],
            ])
        } else if p < self.free {
            Rgba([TRINARY_FREE, TRINARY_FREE, TRINARY_FREE, pixel[3]])
        } else if self.mode == Mode::Trinary || pixel[3] != 255 {
            Rgba([TRINARY_UNKNOWN, TRINARY_UNKNOWN, TRINARY_UNKNOWN, pixel[3]])
        }
        // Scale
        else {
            let scaled = (SCALE_MAGIC * (p - self.free) / (self.occupied - self.free)) as u8;
            Rgba([scaled, scaled, scaled, pixel[3]])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-3;

    #[test]
    fn avg_float() {
        let thresholding = ValueInterpretation::new(0.196, 0.65, false, None);
        let pixel = Rgba([128, 128, 128, 255]);
        assert!(thresholding.to_avg_float(&pixel) - 0.5 < EPS);

        let pixel = Rgba([255, 255, 255, 255]);
        assert_eq!(thresholding.to_avg_float(&pixel), 0.);

        let pixel = Rgba([0, 0, 0, 255]);
        assert_eq!(thresholding.to_avg_float(&pixel), 1.);
    }

    #[test]
    fn trinary() {
        use image::{GenericImage, GenericImageView};

        let mut img = DynamicImage::new_rgba8(1, 1);
        img.put_pixel(0, 0, Rgba([128, 128, 128, 255]));
        let thresholding = ValueInterpretation::new(0.196, 0.65, false, Some(Mode::Trinary));
        thresholding.apply(&mut img);
        assert_eq!(
            img.get_pixel(0, 0),
            Rgba([TRINARY_UNKNOWN, TRINARY_UNKNOWN, TRINARY_UNKNOWN, 255])
        );

        img.put_pixel(0, 0, Rgba([255, 255, 255, 255]));
        thresholding.apply(&mut img);
        assert_eq!(
            img.get_pixel(0, 0),
            Rgba([TRINARY_FREE, TRINARY_FREE, TRINARY_FREE, 255])
        );

        img.put_pixel(0, 0, Rgba([60, 60, 60, 255]));
        thresholding.apply(&mut img);
        assert_eq!(
            img.get_pixel(0, 0),
            Rgba([TRINARY_OCCUPIED, TRINARY_OCCUPIED, TRINARY_OCCUPIED, 255])
        );
    }

    #[test]
    fn scale() {
        use image::{GenericImage, GenericImageView};

        let mut img = DynamicImage::new_rgba8(1, 1);
        img.put_pixel(0, 0, Rgba([128, 128, 128, 255]));
        let thresholding = ValueInterpretation::new(0.196, 0.65, false, Some(Mode::Scale));
        thresholding.apply(&mut img);
        assert_eq!(img.get_pixel(0, 0), Rgba([50, 50, 50, 255]));

        // Any pixel with alpha is considered unknown here.
        img.put_pixel(0, 0, Rgba([1, 2, 3, 100]));
        thresholding.apply(&mut img);
        assert_eq!(
            img.get_pixel(0, 0),
            Rgba([TRINARY_UNKNOWN, TRINARY_UNKNOWN, TRINARY_UNKNOWN, 255])
        );

        img.put_pixel(0, 0, Rgba([60, 60, 60, 255]));
        thresholding.apply(&mut img);
        assert_eq!(
            img.get_pixel(0, 0),
            Rgba([TRINARY_OCCUPIED, TRINARY_OCCUPIED, TRINARY_OCCUPIED, 255])
        );

        img.put_pixel(0, 0, Rgba([0, 0, 0, 255]));
        thresholding.apply(&mut img);
        assert_eq!(
            img.get_pixel(0, 0),
            Rgba([TRINARY_FREE, TRINARY_FREE, TRINARY_FREE, 255])
        );
    }
}
