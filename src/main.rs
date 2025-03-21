use std::env;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::vec::Vec;

// CLI
use clap::Parser;
use log::{error, info, Level};

// GUI
use eframe::egui;

use maps::app::{AppOptions, AppState, ViewMode, CUSTOM_TITLEBAR};
use maps::map_pose::MapPose;
use maps::meta::Meta;
use maps::persistence::load_app_options;
use strum::VariantNames;

const MIN_SIZE: egui::Vec2 = egui::vec2(300., 200.);

#[cfg(not(target_arch = "wasm32"))]
#[derive(Parser, Debug)]
#[command(name = "maps", version, author = "Michael Grupp", about)]
struct Args {
    #[clap(name = "yaml_files", help = "ROS map yaml files", required = false)]
    yaml_files: Vec<String>,
    #[clap(
        short,
        long,
        help = "File path of a saved maps session that will be loaded on startup."
    )]
    session: Option<PathBuf>,
    #[clap(
        short,
        long,
        help = "Map pose YAML file that will be applied to all maps that are loaded via CLI."
    )]
    pose: Option<PathBuf>,
    #[clap(
        short,
        long,
        help = "Initial alpha value for all maps. 0. is transparent, 1.0 is opaque."
    )]
    alpha: Option<f32>,
    #[clap(
        short,
        long,
        num_args = 2,
        value_names = &["width", "height"],
        default_values_t = Vec::from(&[1500., 1000.]),
        help = "Initial window width and height in screen points."
    )]
    window_size: Vec<f32>,
    #[clap(
        short,
        long,
        help = format!("Initial view mode. Possible values: {}", ViewMode::VARIANTS.join(", ")),
    )]
    view_mode: Option<ViewMode>,
    #[clap(
        short,
        long,
        help = "Custom configuration file path for loading and saving app options.\n\
        Will be created on startup with defaults if it does not exist."
    )]
    config: Option<PathBuf>,
    #[clap(
        short,
        long,
        default_value = "info",
        help = "Log level. Possible values: trace, debug, info, warn, error.\n\
        Has no effect if a RUST_LOG environment variable is already defined."
    )]
    log_level: Level,
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

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let args = Args::parse();
    let build_info = build_info_string();

    // Use env_logger to log to stderr when executing: RUST_LOG=debug maps
    // To show only logs of this app: RUST_LOG=maps=debug maps
    if env::var("RUST_LOG").is_err() {
        env::set_var(
            "RUST_LOG",
            format!("maps={}", args.log_level.as_str().to_lowercase()),
        );
    }
    env_logger::init();
    info!("{}", build_info);

    let mut metas: Vec<Meta> = Vec::new();

    for yaml_file in args.yaml_files {
        let yaml_path = Path::new(&yaml_file);
        if !yaml_path.exists() {
            error!("YAML file does not exist: {}", yaml_file);
            exit(1);
        }
        let Some(yaml_path_str) = yaml_path.to_str() else {
            error!("Invalid unicode paths are not supported: {:?}", yaml_path);
            exit(1);
        };
        info!("Loading map YAML {}", yaml_path_str);
        if let Ok(meta) = Meta::load_from_file(&yaml_path.to_path_buf()) {
            if !meta.image_path.exists() {
                error!(
                    "Metadata from {} points to an image that does not exist: {}",
                    yaml_path_str,
                    meta.image_path.to_str().unwrap_or("<invalid unicode path>")
                );
                exit(1);
            }
            metas.push(meta);
        } else {
            error!(
                "Error parsing map YAML file {}. \
                 In case you want to load a session file, use the -s / --session flag.",
                yaml_path_str
            );
            exit(1);
        }
    }

    let map_pose = match &args.pose {
        Some(pose_path) => {
            info!("Loading map pose from {:?}", pose_path);
            match MapPose::from_yaml_file(pose_path) {
                Ok(pose) => Some(pose),
                Err(e) => {
                    error!("Error loading pose {:?}: {}", pose_path, e.message);
                    exit(1);
                }
            }
        }
        None => None,
    };

    let mut options: AppOptions = load_app_options(&args.config);
    options.version = built_info::PKG_VERSION.to_string();
    options.persistence.custom_config_path = args.config;
    options.view_mode = args.view_mode.unwrap_or(options.view_mode);

    // Looks like there is no faster way to edit just the alpha value of a Color32.
    if let Some(alpha) = args.alpha {
        let mut color = options.tint_settings.tint_for_all;
        let new_alpha = (alpha * 255.) as u8;
        color = egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), new_alpha);
        options.tint_settings.tint_for_all = color;
    }

    let mut app_state = match AppState::init(metas, options) {
        Ok(state) => Box::new(state.with_build_info(build_info)),
        Err(e) => {
            error!("Fatal error during initialization. {}", e.message);
            exit(1);
        }
    };

    if let Some(session) = args.session {
        app_state.load_session(session);
    }

    if let Some(pose) = map_pose {
        for (name, map) in app_state.data.maps.iter_mut() {
            info!("Applying pose to map: {}", name);
            map.pose = pose.clone();
        }
    }

    let size = egui::Vec2::from([args.window_size[0], args.window_size[1]]);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(load_icon())
            .with_inner_size(size)
            .with_min_inner_size(MIN_SIZE)
            .with_fullsize_content_view(CUSTOM_TITLEBAR)
            .with_titlebar_shown(!CUSTOM_TITLEBAR)
            .with_title_shown(!CUSTOM_TITLEBAR),
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

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;
    extern crate console_error_panic_hook;
    use std::panic;

    panic::set_hook(Box::new(console_error_panic_hook::hook));

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("maps_canvas_id")
            .expect("Failed to find maps_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("maps_canvas_id was not a HtmlCanvasElement");

        let app_state = AppState::init(Vec::new(), AppOptions::default())
            .expect("Failed to initialize AppState")
            .with_build_info(build_info_string());

        let _ = eframe::WebRunner::new()
            .start(canvas, web_options, Box::new(|_cc| Ok(Box::new(app_state))))
            .await;
    });
}
