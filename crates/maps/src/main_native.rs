use std::{
    env,
    path::{Path, PathBuf},
    process::exit,
};

use {
    clap::Parser,
    eframe::egui,
    log::{LevelFilter, error, info, warn},
    strum::VariantNames,
};

use crate::{
    app::ViewMode,
    persistence::{load_app_options, save_session},
};
use maps_io_ros::{MapPose, Meta};

use crate::app::{AppOptions, AppState};

#[cfg(target_os = "linux")]
use crate::os_helpers::write_desktop_file;

const MIN_SIZE: egui::Vec2 = egui::vec2(450., 200.);
const APP_ID: &str = "maps";

#[derive(Parser, Debug)]
#[command(name = APP_ID, version, author = "Michael Grupp", about)]
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
        help = "Map pose YAML file that will be applied to all maps that are loaded via CLI.\n\
        Note that this is not applied to maps that are loaded from a session file."
    )]
    pose: Option<PathBuf>,
    #[clap(
        short,
        long,
        help = "Initial alpha value for all maps. 0. is transparent, 1.0 is opaque."
    )]
    alpha: Option<f32>,
    #[clap(
        long,
        value_parser = parse_hex_color,
        help = "Hex-color that will be set to transparent in all maps. Example: #FF0012"
    )]
    color_to_alpha: Option<egui::Color32>,
    #[clap(
        short,
        long,
        value_parser = parse_hex_color,
        help = "Hex-color for color tint in all maps."
    )]
    tint_color: Option<egui::Color32>,
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
    log_level: LevelFilter,
    #[clap(
        long,
        help = "Exit after a dry-run initialization without starting the GUI.\n\
        Only load input metadata files, initialize the app state without actually loading images,\n\
        save/update a session file if specified. Can be used to test files or to build a session\n\
        file, e.g. using a script."
    )]
    init_only: bool,
    #[cfg(target_os = "linux")]
    #[clap(
        long,
        help = "Write a .desktop file for easier launching of maps from application menus, and exit.\n\
        Only has an effect on Linux systems using the freedesktop.org standards.\n\
        Overwrites a previous maps desktop file if it exists."
    )]
    write_desktop_file: bool,
}

fn load_icon() -> egui::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "macos")] {
                // Icon composer PNG export from maps_icon.icon,
                // with 100px padding to match size of other macOS icons.
                let icon = include_bytes!("../data/icon_mac.png");
            } else {
                // No padding on other platforms.
                let icon = include_bytes!("../data/icon.png");
            }
        }
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

fn parse_hex_color(hex_str: &str) -> std::result::Result<egui::Color32, std::io::Error> {
    match egui::Color32::from_hex(hex_str) {
        Ok(color) => Ok(color),
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "failed to parse hex string",
        )),
    }
}

pub fn main_native() -> eframe::Result {
    let args = Args::parse();
    let build_info = crate::build_info_string();

    // Use env_logger to log to stderr when executing: RUST_LOG=debug maps
    // To show only logs of this app: RUST_LOG=maps=debug maps
    if env::var("RUST_LOG").is_err() {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Off)
            .filter_module(APP_ID, args.log_level)
            .init();
    } else {
        env_logger::init();
    }
    info!("{build_info}");

    #[cfg(target_os = "linux")]
    if crate::built_info::GIT_VERSION.is_some() && !args.write_desktop_file {
        info!(
            "Development build detected, not writing a .desktop file. \
            Use --write-desktop-file to force this."
        );
    } else {
        if let Err(e) = write_desktop_file(APP_ID, args.write_desktop_file) {
            warn!("Failed to write .desktop file: {e}");
        }
        if args.write_desktop_file {
            exit(0);
        }
    }

    let mut metas: Vec<Meta> = Vec::new();

    for yaml_file in args.yaml_files {
        let yaml_path = Path::new(&yaml_file);
        info!("Loading map YAML {}", yaml_path.display());
        let meta = Meta::load_from_file(yaml_path)
            .map_err(crate::error::Error::from)
            .unwrap_or_else(|e| {
                error!("{e}");
                if matches!(
                    e,
                    crate::error::Error::Core(maps_io_ros::Error::Yaml { .. })
                ) {
                    warn!("In case you want to load a session file, use the -s / --session flag.");
                }
                exit(1);
            });
        metas.push(meta);
    }

    let map_pose = args.pose.as_ref().map(|pose_path| {
        info!("Loading map pose from {pose_path:?}");
        MapPose::from_yaml_file(pose_path).unwrap_or_else(|e| {
            error!("{e}");
            exit(1);
        })
    });

    let mut options: AppOptions = load_app_options(&args.config).with_custom_titlebar();
    options.version = crate::built_info::PKG_VERSION.to_string();
    options.persistence.custom_config_path = args.config;
    options.view_mode = args.view_mode.unwrap_or(options.view_mode);
    options.advanced.dry_run = args.init_only;

    if let Some(tint_color) = args.tint_color {
        options.tint_settings.tint_for_all = tint_color;
    }

    // Looks like there is no faster way to edit just the alpha value of a Color32.
    if let Some(alpha) = args.alpha {
        let mut color = options.tint_settings.tint_for_all;
        let new_alpha = (alpha * 255.) as u8;
        color = egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), new_alpha);
        options.tint_settings.tint_for_all = color;
    }

    if let Some(color_to_alpha) = args.color_to_alpha {
        options.tint_settings.color_to_alpha_for_all = Some(color_to_alpha);
    }

    let mut app_state = match AppState::init(metas, options) {
        Ok(state) => Box::new(state.with_build_info(build_info)),
        Err(e) => {
            error!("Fatal error during initialization. {e}");
            exit(1);
        }
    };

    if let Some(pose) = map_pose {
        for (name, map) in app_state.data.maps.iter_mut() {
            info!("Applying pose to map: {name}");
            map.pose = pose.clone();
        }
    }

    if let Some(session) = &args.session {
        app_state.load_session(session).unwrap_or_else(|e| {
            if !args.init_only {
                // Ignore missing session file in init_only mode to allow creating it in a script.
                error!("{e}");
                exit(1);
            }
        });

        if args.init_only {
            // In init_only mode, directly save the (possibly updated) session.
            save_session(session, &app_state.data).unwrap_or_else(|e| {
                error!("{e}");
                exit(1);
            });
        }
    }

    if args.init_only {
        info!("Exiting without GUI due to --init-only flag.");
        exit(0);
    }

    let size = egui::Vec2::from([args.window_size[0], args.window_size[1]]);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id(APP_ID)
            .with_icon(load_icon())
            .with_inner_size(size)
            .with_min_inner_size(MIN_SIZE)
            .with_fullsize_content_view(app_state.options.custom_titlebar())
            .with_titlebar_shown(!app_state.options.custom_titlebar())
            .with_title_shown(!app_state.options.custom_titlebar()),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(APP_ID, options, Box::new(|_cc| Ok(app_state)))
}
