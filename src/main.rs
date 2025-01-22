use std::path::Path;
use std::process::exit;
use std::vec::Vec;

// CLI
use clap::Parser;
use log::{debug, error, info};

// GUI
use eframe::egui;

use maps::app::{AppOptions, AppState, ViewMode};
use maps::meta::Meta;
use strum::VariantNames;

const MIN_SIZE: egui::Vec2 = egui::vec2(300., 200.);

#[derive(Parser, Debug)]
#[command(name = "maps", version, author = "Michael Grupp")]
struct Args {
    #[clap(name = "yaml_files", help = "ROS map yaml files", required = false)]
    yaml_files: Vec<String>,
    #[clap(
        short,
        long,
        num_args = 2,
        value_names = &["width", "height"],
        default_values_t = Vec::from(&[1500., 1000.]),
        help = "Initial window width and height in pixels."
    )]
    window_size: Vec<f32>,
    #[clap(
        short,
        long,
        default_value_t = ViewMode::default(),
        help = format!("Initial view mode. Possible values: {}", ViewMode::VARIANTS.join(", ")),
    )]
    view_mode: ViewMode,
    #[clap(
        short,
        long,
        default_value_t = 1.,
        help = "Initial alpha value for maps. 0. is transparent, 1.0 is opaque."
    )]
    alpha: f32,
}

// Gather build information from build.rs during compile time.
pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn build_info_string() -> String {
    format!(
        "maps v{} rev:{}{} | {} | {}",
        built_info::PKG_VERSION,
        built_info::GIT_VERSION.unwrap_or("unknown"),
        if built_info::GIT_DIRTY.unwrap_or(false) {
            "(+ uncommited changes)"
        } else {
            ""
        },
        built_info::TARGET,
        built_info::PROFILE,
    )
}

fn load_icon() -> egui::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let icon = include_bytes!("../data/icon.png");
        let image = image::load_from_memory(icon)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    egui::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

fn main() -> eframe::Result {
    let args = Args::parse();
    let build_info = build_info_string();

    // Use env_logger to log to stderr when executing: RUST_LOG=debug maps
    // To show only logs of this app: RUST_LOG=maps=debug maps
    env_logger::init();
    info!("{}", build_info);

    let mut metas: Vec<Meta> = Vec::new();

    for yaml_file in args.yaml_files {
        let yaml_path = Path::new(&yaml_file);
        if !yaml_path.exists() {
            error!("YAML file does not exist: {}", yaml_file);
            exit(1);
        }
        info!("Loading {}", yaml_path.to_str().unwrap());
        if let Ok(meta) = Meta::load_from_file(yaml_path.to_path_buf()) {
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

    let mut options = AppOptions {
        view_mode: args.view_mode,
        ..Default::default()
    };

    // Looks like there is no faster way to edit just the alpha value of a Color32.
    let mut color = options.tint_settings.tint_for_all;
    let new_alpha = (args.alpha * 255.) as u8;
    color = egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), new_alpha);
    options.tint_settings.tint_for_all = color;

    let app_state = match AppState::init(metas, options) {
        Ok(state) => Box::new(state.with_build_info(build_info)),
        Err(e) => {
            error!("Fatal error during initialization. {}", e.message);
            exit(1);
        }
    };

    let size = egui::Vec2::from([args.window_size[0], args.window_size[1]]);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(load_icon())
            .with_inner_size(size)
            .with_min_inner_size(MIN_SIZE),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "maps",
        options,
        Box::new(|cc| {
            // This gives us image support:
            // TODO: still needed?
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(app_state)
        }),
    )
}
