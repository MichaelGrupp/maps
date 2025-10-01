use eframe::wasm_bindgen::JsCast as _;
extern crate console_error_panic_hook;
use std::panic;

use crate::app::{AppOptions, AppState};

pub fn main_wasm() {
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

        let app_state = AppState::init(Vec::new(), AppOptions::default().with_dark_theme())
            .expect("Failed to initialize AppState")
            .with_build_info(crate::build_info_string());

        let _ = eframe::WebRunner::new()
            .start(canvas, web_options, Box::new(|_cc| Ok(Box::new(app_state))))
            .await;
    });
}
