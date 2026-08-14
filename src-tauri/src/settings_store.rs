use std::{
    fs,
    path::PathBuf,
    sync::atomic::Ordering,
    thread,
    time::Duration,
};

use tauri::{AppHandle, Manager};

use crate::{
    history::{HistoryEntry, PersistedHistoryEntry},
    models::{AppSettings, ClipboardItem},
    state::AppState,
};

fn dir(app: &AppHandle) -> Result<PathBuf, String> {
    let path = app.path().app_data_dir().map_err(|error| error.to_string())?;
    fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    Ok(path)
}

fn path(app: &AppHandle, name: &str) -> Result<PathBuf, String> {
    Ok(dir(app)?.join(name))
}

pub fn load_settings(app: &AppHandle) -> AppSettings {
    let Ok(path) = path(app, "settings.json") else {
        return AppSettings::default();
    };
    let Ok(raw) = fs::read_to_string(&path) else {
        return AppSettings::default();
    };

    match serde_json::from_str::<AppSettings>(&raw) {
        Ok(settings) => settings.migrate(),
        Err(error) => {
            log::warn!("Invalid settings; using defaults: {error}");
            let _ = fs::rename(&path, path.with_extension("json.corrupt"));
            AppSettings::default()
        }
    }
}

pub fn save_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    fs::write(path(app, "settings.json")?, raw).map_err(|error| error.to_string())
}

pub fn load_history(app: &AppHandle, enabled: bool, max: usize) -> Vec<HistoryEntry> {
    if !enabled {
        return Vec::new();
    }

    let Ok(path) = path(app, "history.json") else {
        return Vec::new();
    };
    let Ok(raw) = fs::read_to_string(&path) else {
        return Vec::new();
    };

    if let Ok(values) = serde_json::from_str::<Vec<PersistedHistoryEntry>>(&raw) {
        return values
            .into_iter()
            .filter_map(HistoryEntry::from_persisted)
            .take(max)
            .collect();
    }

    // v1.0.x stored ClipboardItem directly. Keep text history across the migration.
    if let Ok(values) = serde_json::from_str::<Vec<ClipboardItem>>(&raw) {
        return values
            .into_iter()
            .filter(|item| item.content.is_some())
            .map(HistoryEntry::from_legacy_item)
            .take(max)
            .collect();
    }

    log::warn!("Invalid history file; ignoring it");
    let _ = fs::rename(&path, path.with_extension("json.corrupt"));
    Vec::new()
}

pub fn save_history(app: &AppHandle, enabled: bool, entries: &[HistoryEntry]) -> Result<(), String> {
    let path = path(app, "history.json")?;
    if !enabled {
        let _ = fs::remove_file(path);
        return Ok(());
    }

    let persisted: Vec<PersistedHistoryEntry> =
        entries.iter().map(HistoryEntry::to_persisted).collect();
    let raw = serde_json::to_string(&persisted).map_err(|error| error.to_string())?;
    fs::write(path, raw).map_err(|error| error.to_string())
}

pub fn schedule_history_save(app: &AppHandle, enabled: bool) {
    if !enabled {
        clear_history_file(app);
        return;
    }

    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let generation = state
        .history_save_generation
        .fetch_add(1, Ordering::SeqCst)
        + 1;
    let handle = app.clone();

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(750));
        let Some(state) = handle.try_state::<AppState>() else {
            return;
        };
        if state.history_save_generation.load(Ordering::SeqCst) != generation {
            return;
        }
        let entries = state.history.lock().entries();
        if let Err(error) = save_history(&handle, true, &entries) {
            log::warn!("Could not persist clipboard history: {error}");
        }
    });
}

pub fn clear_history_file(app: &AppHandle) {
    if let Ok(path) = path(app, "history.json") {
        let _ = fs::remove_file(path);
    }
}
