use std::io::Cursor;
use std::sync::{Arc, Mutex};

use image::{DynamicImage, ImageFormat};
use rfd::AsyncFileDialog;

use crate::wasm::async_data::AsyncData;

#[cfg(target_arch = "wasm32")]
pub fn pick_save_png(data: Arc<Mutex<AsyncData>>, image_name: String, image: DynamicImage) {
    let dialog = AsyncFileDialog::new()
        .set_title("Save image file:")
        .add_filter("PNG", &["png"])
        .set_file_name(image_name.as_str());

    let future = dialog.save_file();

    wasm_bindgen_futures::spawn_local(async move {
        if data.try_lock().is_err() {
            return;
        }

        let Some(file_handle) = future.await else {
            return;
        };

        let result = {
            let mut buf = Vec::new();
            match image.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png) {
                Ok(_) => match file_handle.write(&buf).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!("Error saving image file {}: {:?}", image_name, e)),
                },
                Err(e) => Err(format!("Failed to encode image {}: {:?}", image_name, e)),
            }
        };
        if let Err(err_msg) = result {
            if let Ok(mut locked_data) = data.try_lock() {
                locked_data.error.clone_from(&err_msg);
            }
        }
    });
}
