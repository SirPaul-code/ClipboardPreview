use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};
use tauri_plugin_autostart::ManagerExt;

use crate::{commands, overlays, state::AppState};

pub fn build(app: &AppHandle) -> Result<(), String> {
    let open = MenuItem::with_id(app, "open", "Open Settings", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let history = MenuItem::with_id(app, "history", "Show Clipboard History", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let paused = app
        .state::<AppState>()
        .settings
        .read()
        .general
        .monitoring_paused;
    let pause = CheckMenuItem::with_id(
        app,
        "pause",
        "Pause Monitoring",
        true,
        paused,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    let clear = MenuItem::with_id(app, "clear", "Clear History", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let startup = app.autolaunch().is_enabled().unwrap_or(false);
    let auto = CheckMenuItem::with_id(
        app,
        "startup",
        "Launch at Startup",
        true,
        startup,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    let about = MenuItem::with_id(
        app,
        "about",
        "About Clipboard Preview",
        true,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let sep1 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let sep2 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let menu = Menu::with_items(
        app,
        &[
            &open, &history, &pause, &clear, &sep1, &auto, &sep2, &about, &quit,
        ],
    )
    .map_err(|e| e.to_string())?;
    let icon = tauri::include_image!("./icons/32x32.png");
    let tray = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .tooltip("Clipboard Preview")
        .icon(icon)
        .icon_as_template(cfg!(target_os = "macos"))
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" | "about" => {
                let _ = overlays::open_settings(app);
            }
            "history" => {
                let paused = app
                    .state::<AppState>()
                    .settings
                    .read()
                    .general
                    .monitoring_paused;
                if !paused {
                    let _ = overlays::begin(app, crate::models::InteractionMode::Sticky);
                }
            }
            "pause" => {
                let _ = commands::toggle_monitoring(app.clone());
            }
            "clear" => {
                let _ = commands::clear_history(app.clone());
            }
            "startup" => {
                let state = app.state::<AppState>();
                let mut settings = state.settings.write();
                settings.general.launch_at_startup = !settings.general.launch_at_startup;
                if settings.general.launch_at_startup {
                    let _ = app.autolaunch().enable();
                } else {
                    let _ = app.autolaunch().disable();
                }
                let _ = crate::settings_store::save_settings(app, &settings);
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)
        .map_err(|e| e.to_string())?;
    let visible = app
        .state::<AppState>()
        .settings
        .read()
        .general
        .show_tray_icon;
    tray.set_visible(visible).map_err(|e| e.to_string())?;
    Ok(())
}
