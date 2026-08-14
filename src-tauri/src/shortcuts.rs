use std::str::FromStr;

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::{models::InteractionMode, overlays, selection, state::AppState};

fn parse(value: &str) -> Option<Shortcut> {
    Shortcut::from_str(value).ok()
}

fn action(app: &AppHandle, shortcut: &Shortcut, event_state: ShortcutState) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let settings = state.settings.read().clone();
    let quick_preview = parse(&settings.shortcuts.quick_preview);
    let history = parse(&settings.shortcuts.history_selector);
    let open_settings = parse(&settings.shortcuts.open_settings);
    let pause = parse(&settings.shortcuts.pause_monitoring);

    if quick_preview.as_ref() == Some(shortcut) && matches!(event_state, ShortcutState::Pressed) {
        let _ = overlays::show_quick(app);
    } else if !settings.shortcuts.history_selector.eq_ignore_ascii_case("Tab")
        && history.as_ref() == Some(shortcut)
    {
        match event_state {
            ShortcutState::Pressed => {
                if !state.selector.lock().active {
                    let _ = overlays::begin(app, settings.history.interaction_mode.clone());
                }
            }
            ShortcutState::Released => {
                if matches!(settings.history.interaction_mode, InteractionMode::HoldRelease) {
                    let _ = selection::accept(app);
                }
            }
        }
    } else if open_settings.as_ref() == Some(shortcut)
        && matches!(event_state, ShortcutState::Pressed)
    {
        let _ = overlays::open_settings(app);
    } else if pause.as_ref() == Some(shortcut) && matches!(event_state, ShortcutState::Pressed) {
        let mut settings = state.settings.write();
        settings.general.monitoring_paused = !settings.general.monitoring_paused;
        let _ = crate::settings_store::save_settings(app, &settings);
    }
}

pub fn plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, shortcut, event| action(app, shortcut, event.state()))
        .build()
}

pub fn register_all(app: &AppHandle) -> Result<(), String> {
    let Some(state) = app.try_state::<AppState>() else {
        return Err("Application state is not ready for shortcut registration".into());
    };
    let settings = state.settings.read().clone();
    drop(state);

    let manager = app.global_shortcut();
    manager.unregister_all().map_err(|error| error.to_string())?;

    let mut shortcuts = vec![
        settings.shortcuts.quick_preview.as_str(),
        settings.shortcuts.open_settings.as_str(),
        settings.shortcuts.pause_monitoring.as_str(),
    ];
    if !settings.shortcuts.history_selector.eq_ignore_ascii_case("Tab") {
        shortcuts.push(settings.shortcuts.history_selector.as_str());
    }

    for shortcut in shortcuts {
        if shortcut.trim().is_empty() {
            continue;
        }
        if let Err(error) = manager.register(shortcut) {
            let _ = manager.unregister_all();
            return Err(format!("Could not register {shortcut}: {error}"));
        }
    }

    Ok(())
}
