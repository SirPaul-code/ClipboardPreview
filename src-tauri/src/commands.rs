use std::{collections::HashSet, io::Cursor, process::Command};

#[cfg(any(target_os = "windows", target_os = "macos"))]
use std::sync::atomic::Ordering;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::ImageOutputFormat;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;

use crate::{
    clipboard, diagnostics, global_input,
    models::{
        AppSettings, ImagePreviewPayload, InteractionMode, PlatformStatus,
        HISTORY_MEMORY_BUDGET_MIB, HISTORY_PERF_WARNING_ITEMS,
    },
    overlays, permissions, selection, settings_store, shortcuts,
    state::AppState,
    updates,
};

const IMAGE_PREVIEW_MAX_WIDTH: u32 = 1280;
const IMAGE_PREVIEW_MAX_HEIGHT: u32 = 900;
const EXTERNAL_URLS: [&str; 5] = [
    "https://github.com/SirPaul-code",
    "https://github.com/SirPaul-code/ClipboardPreview",
    "https://github.com/SirPaul-code/ClipboardPreview/releases",
    "https://github.com/SirPaul-code/ClipboardPreview/issues",
    "https://github.com/SirPaul-code/ClipboardPreview/blob/main/LICENSE",
];

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
        &settings.shortcuts.previous_item,
        &settings.shortcuts.next_item,
        &settings.shortcuts.open_settings,
        &settings.shortcuts.pause_monitoring,
    ];
    let set: HashSet<_> = shortcuts
        .iter()
        .filter(|shortcut| !shortcut.trim().is_empty())
        .map(|shortcut| shortcut.to_lowercase())
        .collect();
    set.len()
        == shortcuts
            .iter()
            .filter(|shortcut| !shortcut.trim().is_empty())
            .count()
}

fn apply_pause_transition(app: &AppHandle, was_paused: bool, is_paused: bool) {
    if was_paused == is_paused {
        return;
    }

    global_input::reset_state(app);

    if is_paused {
        selection::cancel(app);
        if let Some(window) = app.get_webview_window("quick-preview") {
            let _ = window.hide();
        }
    }

    updates::show_runtime_notice(
        app,
        if is_paused {
            "Clipboard paused"
        } else {
            "Clipboard resumed"
        },
    );
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: AppSettings) -> Result<AppSettings, String> {
    let next = settings.normalized();
    if !unique_shortcuts(&next) {
        return Err("Each shortcut action needs a unique key binding".into());
    }
    if !global_input::navigation_shortcut_supported(&next.shortcuts.previous_item)
        || !global_input::navigation_shortcut_supported(&next.shortcuts.next_item)
    {
        return Err("Switcher navigation supports letters, numbers, arrows, Home/End, PageUp/PageDown, Enter, Space, Escape, Backspace, Delete, Insert and F1-F12, with optional modifiers.".into());
    }
    if cfg!(target_os = "linux") {
        if next.shortcuts.history_selector.eq_ignore_ascii_case("Tab") {
            return Err("Plain Tab hold is not available on Linux. Choose a modifier-based switcher shortcut such as Ctrl+Alt+J.".into());
        }
        if matches!(next.history.interaction_mode, InteractionMode::HoldRelease) {
            return Err("Hold/release mode is not available on Linux. Use sticky mode so wheel and keyboard navigation remain reliable.".into());
        }
    }

    let state = app.state::<AppState>();
    let old = state.settings.read().clone();
    let was_paused = old.general.monitoring_paused;
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

    apply_pause_transition(&app, was_paused, next.general.monitoring_paused);
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
    let mut next = app.state::<AppState>().settings.read().clone();
    next.general.monitoring_paused = !next.general.monitoring_paused;
    let saved = save_settings(app, next)?;
    Ok(saved.general.monitoring_paused)
}

#[tauri::command]
pub fn platform_status(app: AppHandle) -> PlatformStatus {
    let windows = cfg!(target_os = "windows");
    let mac = cfg!(target_os = "macos");
    let linux = cfg!(target_os = "linux");
    let accessibility_granted = permissions::accessibility_granted();
    let input_monitoring_granted = permissions::input_monitoring_granted();
    let state = app.state::<AppState>();

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    let native_input_ready = state.native_input_ready.load(Ordering::SeqCst);
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let native_input_ready = false;

    let startup_warnings = state.startup_warnings.read().clone();

    PlatformStatus {
        os: std::env::consts::OS.into(),
        accessibility_required: mac,
        accessibility_granted,
        input_monitoring_required: false,
        input_monitoring_granted,
        native_input_ready,
        hold_release_available: windows || (mac && native_input_ready),
        global_wheel_available: windows || (mac && native_input_ready),
        tab_hold_available: windows || (mac && native_input_ready),
        image_history_available: windows || mac || linux,
        history_memory_budget_mib: HISTORY_MEMORY_BUDGET_MIB,
        history_performance_warning_items: HISTORY_PERF_WARNING_ITEMS,
        last_crash_available: diagnostics::last_crash_available(),
        version: app.package_info().version.to_string(),
        official_build: updates::official_build(),
        updates_enabled: updates::ready(),
        startup_warnings,
    }
}

#[tauri::command]
pub fn open_macos_accessibility_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        permissions::open_accessibility_settings()
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("macOS Accessibility settings are only available on macOS".into())
    }
}

#[tauri::command]
pub fn open_macos_input_monitoring_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        permissions::open_input_monitoring_settings()
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("macOS Input Monitoring settings are only available on macOS".into())
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
pub fn open_external(url: String) -> Result<(), String> {
    if !EXTERNAL_URLS.contains(&url.as_str()) {
        return Err("Blocked external URL".into());
    }

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("rundll32");
        command.arg("url.dll,FileProtocolHandler").arg(&url);
        command
    };
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(&url);
        command
    };
    #[cfg(target_os = "linux")]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(&url);
        command
    };
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return Err("Opening external URLs is not supported on this platform".into());

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    {
        command
            .spawn()
            .map_err(|error| format!("Could not open system browser: {error}"))?;
        Ok(())
    }
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
    let was_paused = state.settings.read().general.monitoring_paused;
    *state.settings.write() = defaults.clone();
    shortcuts::register_all(&app)?;
    state.startup_warnings.write().clear();
    app.autolaunch()
        .disable()
        .map_err(|error| error.to_string())?;
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_visible(defaults.general.show_tray_icon);
    }
    settings_store::save_settings(&app, &defaults)?;
    settings_store::clear_history_file(&app);
    apply_pause_transition(&app, was_paused, defaults.general.monitoring_paused);
    Ok(defaults)
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0)
}
