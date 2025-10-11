//! Minimal demo that loads a ROS map yaml and applies a colormap to the corresponding image.
//! The resulting image shows the same as when you publish the map with a ROS map_server and
//! display it in RViz.

use std::path::{Path, PathBuf};
use std::process::exit;

use image::DynamicImage;
use maps_io_ros::{ColorMap, Meta, load_image, save_image};

fn filename(path: &Path) -> &str {
    path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
}

fn demo(meta_path: &Path) -> maps_io_ros::Result<()> {
    // Load the map metadata from the yaml file.
    let meta = Meta::load_from_file(&meta_path)?;
    println!(
        "Loaded map metadata: {:?} (image path: {:?})",
        meta_path, meta.image_path
    );

    // Load the map image from the path specified in the metadata.
    let img = load_image(&meta.image_path)?;
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

    println!("Saved processed image to {out_path:?}");
    save_image(&out_path, &img_rgba)?;

    Ok(())
}

fn main() {
    let meta_path = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: demo <map.yaml>");
        exit(1);
    }));

    #[cfg(debug_assertions)]
    println!("You're running the demo in debug mode! Expect poor performance.");

    if let Err(e) = demo(&meta_path) {
        eprintln!("{e}");
        exit(1);
    }
}
