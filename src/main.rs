use std::path::Path;
use std::process::exit;
use std::vec::Vec;

// CLI
use clap::Parser;
use env_logger;
use log::{debug, error, info};

// GUI
use eframe::egui;

use rosmaps::app::AppState;
use rosmaps::meta::{Meta, MetaYamlAnnotated};

#[derive(Parser, Debug)]
#[command(name = "rosmaps", version, author = "Michael Grupp")]
struct Args {
    #[clap(name = "yaml_files", help = "ROS map yaml files", required = true)]
    yaml_files: Vec<String>,
    #[clap(
        short,
        long,
        num_args = 2,
        value_names = &["width", "height"],
        default_values_t = Vec::from(&[750.0, 750.0]),
        help = "Initial window width and height in pixels."
    )]
    window_size: Vec<f32>,
}

fn main() -> eframe::Result {
    let args = Args::parse();

    // Use env_logger to log to stderr when executing: RUST_LOG=info rosmaps
    // To show only logs of this app: RUST_LOG=rosmaps rosmaps
    env_logger::init();

    let mut metas: Vec<Meta> = Vec::new();

    for yaml_file in args.yaml_files {
        let yaml_path = Path::new(&yaml_file);
        if !yaml_path.exists() {
            error!("YAML file does not exist: {}", yaml_file);
            exit(1);
        }
        info!("Loading {}", yaml_path.to_str().unwrap());
        if let Ok(meta_yaml_annotated) = MetaYamlAnnotated::from(yaml_path.to_path_buf()) {
            let meta = Meta::from(meta_yaml_annotated);
            debug!("Parsed metadata: {:?}", meta);
            if !meta.image_path.exists() {
                error!(
                    "Metadata from {} points to an image that does not exist: {}",
                    yaml_path.to_str().unwrap(),
                    meta.image_path.to_str().unwrap()
                );
                exit(1);
            }
            metas.push(meta);
        } else {
            error!("Error parsing YAML file: {}", yaml_path.to_str().unwrap());
            exit(1);
        }
    }

    let size = egui::Vec2::from([args.window_size[0], args.window_size[1]]);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(size),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "rosmaps",
        options,
        Box::new(|cc| {
            // This gives us image support:
            // TODO: still needed?
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<AppState>::from(AppState::init(metas)))
        }),
    )
}
