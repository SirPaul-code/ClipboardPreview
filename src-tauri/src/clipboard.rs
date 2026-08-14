use std::{borrow::Cow, thread, time::Duration};

use arboard::{Clipboard, ImageData};
use image::ImageFormat;
use tauri::{AppHandle, Emitter, Manager};

use crate::{
    history::{hash_image, hash_text, HistoryEntry},
    settings_store,
    state::AppState,
};

pub fn start(app: AppHandle) {
    thread::spawn(move || {
        let mut clipboard: Option<Clipboard> = None;

        loop {
            let (paused, interval, max, persist) = {
                let Some(state) = app.try_state::<AppState>() else {
                    thread::sleep(Duration::from_millis(250));
                    continue;
                };
                let settings = state.settings.read().clone();
                (
                    settings.general.monitoring_paused,
                    settings.advanced.clipboard_poll_interval_ms,
                    settings.history.max_items,
                    settings.history.persist_history,
                )
            };

            if !paused {
                if clipboard.is_none() {
                    clipboard = Clipboard::new().ok();
                }

                if let Some(clipboard) = clipboard.as_mut() {
                    let mut image_was_available = false;

                    if let Ok(image) = clipboard.get_image() {
                        image_was_available = true;
                        let width = image.width;
                        let height = image.height;
                        let rgba = image.bytes.into_owned();
                        let hash = hash_image(width, height, &rgba);

                        if !consume_internal_write(&app, &hash) {
                            let result = app
                                .try_state::<AppState>()
                                .map(|state| state.history.lock().add_image(width, height, rgba, max));

                            match result {
                                Some(Ok(true)) => history_changed(&app, persist),
                                Some(Ok(false)) | None => {}
                                Some(Err(error)) => log::warn!("Clipboard image was not added: {error}"),
                            }
                        }
                    }

                    if !image_was_available {
                        if let Ok(text) = clipboard.get_text() {
                            let hash = hash_text(&text);
                            if !consume_internal_write(&app, &hash) {
                                let changed = app
                                    .try_state::<AppState>()
                                    .map(|state| state.history.lock().add_text(text, max))
                                    .unwrap_or(false);
                                if changed {
                                    history_changed(&app, persist);
                                }
                            }
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(interval));
        }
    });
}

fn consume_internal_write(app: &AppHandle, hash: &str) -> bool {
    let Some(state) = app.try_state::<AppState>() else {
        return false;
    };
    let mut token = state.internal_write_hash.lock();
    if token.as_deref() == Some(hash) {
        token.take();
        true
    } else {
        false
    }
}

fn history_changed(app: &AppHandle, persist: bool) {
    settings_store::schedule_history_save(app, persist);
    let _ = app.emit("clipboard://history-changed", ());
}

pub fn write_entry(app: &AppHandle, entry: &HistoryEntry) -> Result<(), String> {
    if let Some(image_png) = entry.image_png.as_deref() {
        return write_image_png(app, &entry.item.hash, image_png);
    }

    if let Some(text) = entry.item.content.as_deref() {
        return write_text(app, text);
    }

    Err("Clipboard history item has no restorable payload".into())
}

pub fn write_text(app: &AppHandle, text: &str) -> Result<(), String> {
    let hash = hash_text(text);
    if let Some(state) = app.try_state::<AppState>() {
        *state.internal_write_hash.lock() = Some(hash);
    }

    let mut clipboard = Clipboard::new().map_err(|error| format!("Clipboard unavailable: {error}"))?;
    if let Err(error) = clipboard.set_text(text.to_owned()) {
        if let Some(state) = app.try_state::<AppState>() {
            *state.internal_write_hash.lock() = None;
        }
        return Err(format!("Unable to write clipboard: {error}"));
    }
    Ok(())
}

fn write_image_png(app: &AppHandle, hash: &str, png: &[u8]) -> Result<(), String> {
    let image = image::load_from_memory_with_format(png, ImageFormat::Png)
        .map_err(|error| format!("Unable to decode clipboard image: {error}"))?
        .to_rgba8();
    let (width, height) = image.dimensions();

    if let Some(state) = app.try_state::<AppState>() {
        *state.internal_write_hash.lock() = Some(hash.to_string());
    }

    let mut clipboard = Clipboard::new().map_err(|error| format!("Clipboard unavailable: {error}"))?;
    let result = clipboard.set_image(ImageData {
        width: width as usize,
        height: height as usize,
        bytes: Cow::Owned(image.into_raw()),
    });

    if let Err(error) = result {
        if let Some(state) = app.try_state::<AppState>() {
            *state.internal_write_hash.lock() = None;
        }
        return Err(format!("Unable to write clipboard image: {error}"));
    }
    Ok(())
}
