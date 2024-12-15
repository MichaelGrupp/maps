use std::path::Path;
use std::process::exit;
use std::vec::Vec;

// CLI
use clap::Parser;
use env_logger;
use log::{debug, error, info};

// GUI
use eframe::egui;
use egui::Image;

use image::ImageReader;

mod meta;
use meta::{Meta, MetaYamlAnnotated};

#[derive(Parser, Debug)]
#[command(name = "rosmaps", version, author = "Michael Grupp")]
struct Args {
    #[clap(name = "yaml_files", help = "ROS map yaml files", required = true)]
    yaml_files: Vec<String>,
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

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 1000.0]),
        ..Default::default()
    };
    eframe::run_native(
        "rosmaps",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<RosMapsApp>::from(RosMapsApp::from(metas)))
        }),
    )
}

#[derive(Default)]
struct RosMapsApp {
    metas: Vec<Meta>,
}

impl RosMapsApp {
    fn from(metas: Vec<Meta>) -> RosMapsApp {
        RosMapsApp { metas }
    }
}

impl eframe::App for RosMapsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.add(egui::Image::new(format!(
                    "file://{0}",
                    self.metas[0].image_path.to_str().unwrap()
                )));
            });
        });
    }
}
