//! Minimal demo that loads a ROS map yaml and applies a colormap to the corresponding image.
//! The resulting image shows the same as when you publish the map with a ROS map_server and
//! display it in RViz.

use std::path::{Path, PathBuf};
use std::process::exit;

use image::DynamicImage;
use maps_io_ros::{ColorMap, Meta};

fn filename(path: &Path) -> &str {
    path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
}

fn main() {
    let meta_path = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: demo <map.yaml>");
        exit(1);
    }));

    // Load the map metadata from the yaml file.
    let meta = Meta::load_from_file(&meta_path).unwrap_or_else(|e| {
        eprintln!("Error loading map yaml {meta_path:?}: {e}");
        exit(1);
    });
    println!(
        "Loaded map metadata: {:?} (image path: {:?})",
        meta_path, meta.image_path
    );

    // Load the map image from the path specified in the metadata.
    let img = image::open(&meta.image_path).unwrap_or_else(|e| {
        eprintln!("Error loading map image {:?}: {}", meta.image_path, e);
        exit(1);
    });
    println!("Loaded map image from {:?}", meta.image_path);

    // Like in RViz, colormaps operate on RGBA image buffers.
    let mut img_rgba = DynamicImage::from(img.to_rgba8());

    // Interpret the image values according to the metadata and apply a colormap.
    // Here we use the classic RViz colormap.
    let colormap = ColorMap::RvizMap;
    println!("Applying value interpretation with colormap {colormap:?}");
    meta.value_interpretation
        .with_colormap(colormap)
        .apply(&mut img_rgba, img.has_alpha());

    // Save the resulting image to a temporary file.
    let out_path = std::env::temp_dir().join("processed_".to_owned() + filename(&meta.image_path));
    img_rgba.save(&out_path).unwrap_or_else(|e| {
        eprintln!("Error saving processed image to {out_path:?}: {e}");
        exit(1);
    });
    println!("Saved processed image to {out_path:?}");
}
