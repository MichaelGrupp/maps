use std::io::Cursor;
use std::sync::{Arc, Mutex};

use image::{DynamicImage, ImageFormat};
use log::info;
use rfd::{AsyncFileDialog, FileHandle};

use crate::wasm::async_data::AsyncData;

#[cfg(target_arch = "wasm32")]
async fn save_image(
    data: &mut AsyncData,
    file_handle: FileHandle,
    image_name: &str,
    image: DynamicImage,
    format: ImageFormat,
) {
    let mut buf = Vec::new();
    match image.write_to(&mut Cursor::new(&mut buf), format) {
        Ok(_) => {}
        Err(e) => {
            data.error
                .clone_from(&format!("Failed to encode image {}: {:?}", image_name, e));
            return;
        }
    }

    match file_handle.write(&buf).await {
        Ok(_) => {
            info!(
                "Saved image file {} as bytes: {:?}",
                image_name,
                file_handle.file_name()
            );
        }
        Err(e) => {
            data.error
                .clone_from(&format!("Error saving image file {}: {:?}", image_name, e));
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn pick_save_png(data: Arc<Mutex<AsyncData>>, image_name: String, image: DynamicImage) {
    let dialog = AsyncFileDialog::new()
        .set_title("Save image file:")
        .add_filter("PNG", &["png"])
        .set_file_name(image_name.as_str());

    let future = dialog.save_file();

    wasm_bindgen_futures::spawn_local(async move {
        let Ok(mut locked_data) = data.try_lock() else {
            return;
        };

        let Some(file_handle) = future.await else {
            return;
        };
        save_image(
            &mut locked_data,
            file_handle,
            image_name.as_str(),
            image,
            ImageFormat::Png,
        )
        .await;
    });
}
