use std::{collections::HashSet, io::Cursor, process::Command};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::ImageOutputFormat;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;

use crate::{
    clipboard, diagnostics,
    models::{
        AppSettings, ImagePreviewPayload, InteractionMode, PlatformStatus,
        HISTORY_MEMORY_BUDGET_MIB, HISTORY_PERF_WARNING_ITEMS,
    },
    overlays, permissions, selection, settings_store, shortcuts,
    state::AppState,
};

const IMAGE_PREVIEW_MAX_WIDTH: u32 = 1280;
const IMAGE_PREVIEW_MAX_HEIGHT: u32 = 900;

#[tauri::command]
pub fn get_settings(app: AppHandle) -> AppSettings {
    app.state::<AppState>().settings.read().clone()
}

#[tauri::command]
pub fn get_history(app: AppHandle) -> Vec<crate::models::ClipboardItem> {
    app.state::<AppState>().history.lock().items()
}

#[tauri::command]
pub fn get_image_preview(app: AppHandle, id: String) -> Result<Option<ImagePreviewPayload>, String> {
    let state = app.state::<AppState>();
    let entry = state.history.lock().find(&id);
    let Some(entry) = entry else {
        return Ok(None);
    };
    let Some(png) = entry.image_png else {
        return Ok(None);
    };

    let decoded = image::load_from_memory_with_format(&png, image::ImageFormat::Png)
        .map_err(|error| format!("Could not decode clipboard image preview: {error}"))?;
    let preview = decoded.thumbnail(IMAGE_PREVIEW_MAX_WIDTH, IMAGE_PREVIEW_MAX_HEIGHT);
    let (preview_width, preview_height) = (preview.width(), preview.height());
    let mut encoded = Vec::new();
    preview
        .write_to(&mut Cursor::new(&mut encoded), ImageOutputFormat::Png)
        .map_err(|error| format!("Could not encode clipboard image preview: {error}"))?;

    Ok(Some(ImagePreviewPayload {
        id,
        data_url: format!("data:image/png;base64,{}", STANDARD.encode(encoded)),
        width: preview_width,
        height: preview_height,
    }))
}

fn unique_shortcuts(settings: &AppSettings) -> bool {
    let shortcuts = [
        &settings.shortcuts.quick_preview,
        &settings.shortcuts.history_selector,
        &settings.shortcuts.open_settings,
        &settings.shortcuts.pause_monitoring,
    ];
    let set: HashSet<_> = shortcuts
        .iter()
        .filter(|shortcut| !shortcut.trim().is_empty())
        .map(|shortcut| shortcut.to_lowercase())
        .collect();
    set.len() == shortcuts.iter().filter(|shortcut| !shortcut.trim().is_empty()).count()
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: AppSettings) -> Result<AppSettings, String> {
    let next = settings.normalized();
    if !unique_shortcuts(&next) {
        return Err("Each action needs a unique shortcut".into());
    }

    let state = app.state::<AppState>();
    let old = state.settings.read().clone();
    *state.settings.write() = next.clone();

    if let Err(error) = shortcuts::register_all(&app) {
        *state.settings.write() = old.clone();
        let _ = shortcuts::register_all(&app);
        return Err(error);
    }

    state
        .startup_warnings
        .write()
        .retain(|warning| !warning.starts_with("Global shortcuts could not be registered:"));

    let startup_result = if next.general.launch_at_startup {
        app.autolaunch().enable()
    } else {
        app.autolaunch().disable()
    };
    if let Err(error) = startup_result {
        *state.settings.write() = old;
        let _ = shortcuts::register_all(&app);
        return Err(error.to_string());
    }

    if let Some(tray) = app.tray_by_id("main-tray") {
        tray.set_visible(next.general.show_tray_icon)
            .map_err(|error| error.to_string())?;
    }

    state.history.lock().truncate(next.history.max_items);
    settings_store::save_settings(&app, &next)?;
    if next.history.persist_history {
        let entries = state.history.lock().entries();
        settings_store::save_history(&app, true, &entries)?;
    } else {
        settings_store::clear_history_file(&app);
    }
    Ok(next)
}

#[tauri::command]
pub fn clear_history(app: AppHandle) -> Result<(), String> {
    app.state::<AppState>().history.lock().clear();
    settings_store::clear_history_file(&app);
    Ok(())
}

#[tauri::command]
pub fn select_history_item(app: AppHandle, id: String) -> Result<(), String> {
    let (settings, entry) = {
        let state = app.state::<AppState>();
        let settings = state.settings.read().clone();
        let entry = state
            .history
            .lock()
            .find(&id)
            .ok_or("History item not found")?;
        (settings, entry)
    };

    clipboard::write_entry(&app, &entry)?;
    if settings.history.move_selected_to_top {
        app.state::<AppState>().history.lock().promote(&id);
        settings_store::schedule_history_save(&app, settings.history.persist_history);
    }
    overlays::hide_history(&app);
    Ok(())
}

#[tauri::command]
pub fn navigate_selection(app: AppHandle, delta: i32) -> Result<(), String> {
    selection::navigate(&app, delta)
}

#[tauri::command]
pub fn accept_selection(app: AppHandle) -> Result<(), String> {
    selection::accept(&app)
}

#[tauri::command]
pub fn cancel_selection(app: AppHandle) -> Result<(), String> {
    selection::cancel(&app);
    Ok(())
}

#[tauri::command]
pub fn show_history(app: AppHandle) -> Result<(), String> {
    overlays::begin(&app, InteractionMode::Sticky)
}

#[tauri::command]
pub fn open_settings(app: AppHandle) -> Result<(), String> {
    overlays::open_settings(&app)
}

#[tauri::command]
pub fn toggle_monitoring(app: AppHandle) -> Result<bool, String> {
    let state = app.state::<AppState>();
    let value = {
        let mut settings = state.settings.write();
        settings.general.monitoring_paused = !settings.general.monitoring_paused;
        settings_store::save_settings(&app, &settings)?;
        settings.general.monitoring_paused
    };
    Ok(value)
}

#[tauri::command]
pub fn platform_status(app: AppHandle) -> PlatformStatus {
    let mac = cfg!(target_os = "macos");
    let accessibility_granted = permissions::accessibility_granted();
    let state = app.state::<AppState>();
    let startup_warnings = state.startup_warnings.read().clone();

    PlatformStatus {
        os: std::env::consts::OS.into(),
        accessibility_required: mac,
        accessibility_granted,
        hold_release_available: !mac || accessibility_granted,
        global_wheel_available: !mac || accessibility_granted,
        tab_hold_available: !mac || accessibility_granted,
        image_history_available: cfg!(any(target_os = "windows", target_os = "macos")),
        history_memory_budget_mib: HISTORY_MEMORY_BUDGET_MIB,
        history_performance_warning_items: HISTORY_PERF_WARNING_ITEMS,
        last_crash_available: diagnostics::last_crash_available(),
        version: app.package_info().version.to_string(),
        startup_warnings,
    }
}

#[tauri::command]
pub fn diagnostics_report(app: AppHandle) -> String {
    diagnostics::report(
        &app.package_info().version.to_string(),
        std::env::consts::OS,
    )
}

#[tauri::command]
pub fn clear_diagnostics() -> Result<(), String> {
    diagnostics::clear_last_crash()
}

#[tauri::command]
pub fn open_diagnostics_folder() -> Result<(), String> {
    let path = diagnostics::diagnostics_dir();
    std::fs::create_dir_all(&path).map_err(|error| error.to_string())?;

    #[cfg(target_os = "windows")]
    let mut command = Command::new("explorer");
    #[cfg(target_os = "macos")]
    let mut command = Command::new("open");
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let mut command = Command::new("xdg-open");

    command
        .arg(path)
        .spawn()
        .map_err(|error| format!("Could not open diagnostics folder: {error}"))?;
    Ok(())
}

#[tauri::command]
pub fn complete_first_run(app: AppHandle) -> Result<AppSettings, String> {
    let state = app.state::<AppState>();
    let mut settings = state.settings.write();
    settings.first_run_completed = true;
    settings_store::save_settings(&app, &settings)?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn reset_settings(app: AppHandle) -> Result<AppSettings, String> {
    let defaults = AppSettings::default().normalized();
    let state = app.state::<AppState>();
    *state.settings.write() = defaults.clone();
    shortcuts::register_all(&app)?;
    state.startup_warnings.write().clear();
    app.autolaunch().disable().map_err(|error| error.to_string())?;
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_visible(defaults.general.show_tray_icon);
    }
    settings_store::save_settings(&app, &defaults)?;
    settings_store::clear_history_file(&app);
    Ok(defaults)
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0)
}
