use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    thread,
    time::Duration,
};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::UpdaterExt;

use crate::overlays;

const UPDATE_ENDPOINT: &str =
    "https://github.com/SirPaul-code/ClipboardPreview/releases/latest/download/latest.json";
const UPDATE_CHECK_DELAY: Duration = Duration::from_secs(5);
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60);
const UPDATE_TIMEOUT: Duration = Duration::from_secs(20);

static UPDATER_READY: AtomicBool = AtomicBool::new(false);
static NOTIFICATIONS_READY: AtomicBool = AtomicBool::new(false);
static LAST_NOTIFIED_UPDATE: Mutex<Option<String>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub enabled: bool,
    pub available: bool,
    pub current_version: String,
    pub version: Option<String>,
    pub body: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateProgress {
    downloaded: u64,
    total: Option<u64>,
}

pub fn official_build() -> bool {
    option_env!("CLIPBOARD_PREVIEW_OFFICIAL_BUILD") == Some("1")
}

pub fn ready() -> bool {
    official_build() && UPDATER_READY.load(Ordering::SeqCst)
}

pub fn register_notifications(app: &AppHandle) -> Result<(), String> {
    app.plugin(tauri_plugin_notification::init())
        .map_err(|error| error.to_string())?;
    NOTIFICATIONS_READY.store(true, Ordering::SeqCst);
    Ok(())
}

pub fn show_runtime_notice(app: &AppHandle, message: &str) {
    if !NOTIFICATIONS_READY.load(Ordering::SeqCst) {
        return;
    }
    if let Err(error) = app
        .notification()
        .builder()
        .title("Clipboard Preview")
        .body(message)
        .show()
    {
        log::warn!("Could not show Clipboard Preview notification: {error}");
    }
}

pub fn register_updater(app: &AppHandle) -> Result<(), String> {
    if !official_build() {
        return Ok(());
    }

    let public_key = include_str!("../updater.pub").trim();
    if public_key.is_empty() || public_key.contains("PLACEHOLDER") {
        return Err("Official build updater public key is not configured".into());
    }

    app.plugin(
        tauri_plugin_updater::Builder::new()
            .pubkey(public_key.to_string())
            .build(),
    )
    .map_err(|error| error.to_string())?;
    UPDATER_READY.store(true, Ordering::SeqCst);
    Ok(())
}

fn disabled_status(app: &AppHandle) -> UpdateStatus {
    UpdateStatus {
        enabled: false,
        available: false,
        current_version: app.package_info().version.to_string(),
        version: None,
        body: None,
        date: None,
    }
}

async fn check_inner(app: &AppHandle) -> Result<UpdateStatus, String> {
    if !ready() {
        return Ok(disabled_status(app));
    }

    let endpoint = UPDATE_ENDPOINT
        .parse()
        .map_err(|error| format!("Invalid updater endpoint: {error}"))?;
    let updater = app
        .updater_builder()
        .endpoints(vec![endpoint])
        .map_err(|error| error.to_string())?
        .timeout(UPDATE_TIMEOUT)
        .build()
        .map_err(|error| error.to_string())?;

    let current_version = app.package_info().version.to_string();
    match updater.check().await.map_err(|error| error.to_string())? {
        Some(update) => Ok(UpdateStatus {
            enabled: true,
            available: true,
            current_version,
            version: Some(update.version.clone()),
            body: update.body.clone(),
            date: update.date.map(|date| date.to_string()),
        }),
        None => Ok(UpdateStatus {
            enabled: true,
            available: false,
            current_version,
            version: None,
            body: None,
            date: None,
        }),
    }
}

#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<UpdateStatus, String> {
    check_inner(&app).await
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    if !ready() {
        return Err("Automatic updates are disabled for this source/development build".into());
    }

    let endpoint = UPDATE_ENDPOINT
        .parse()
        .map_err(|error| format!("Invalid updater endpoint: {error}"))?;
    let updater = app
        .updater_builder()
        .endpoints(vec![endpoint])
        .map_err(|error| error.to_string())?
        .timeout(UPDATE_TIMEOUT)
        .build()
        .map_err(|error| error.to_string())?;
    let update = updater
        .check()
        .await
        .map_err(|error| error.to_string())?
        .ok_or("No update is currently available")?;

    let progress_app = app.clone();
    let mut downloaded = 0u64;
    update
        .download_and_install(
            move |chunk_size, total| {
                downloaded = downloaded.saturating_add(chunk_size as u64);
                let _ = progress_app.emit(
                    "clipboard://update-progress",
                    UpdateProgress { downloaded, total },
                );
            },
            || {},
        )
        .await
        .map_err(|error| error.to_string())?;

    // The updater plugin restarts Windows through the installer itself. Other
    // platforms return here after installation and need an explicit restart.
    #[cfg(not(target_os = "windows"))]
    app.restart();

    #[allow(unreachable_code)]
    Ok(())
}

fn first_notice_for(version: &str) -> bool {
    let Ok(mut last) = LAST_NOTIFIED_UPDATE.lock() else {
        return true;
    };
    if last.as_deref() == Some(version) {
        return false;
    }
    *last = Some(version.to_string());
    true
}

fn surface_available_update(app: &AppHandle, status: UpdateStatus) {
    let first_notice = status
        .version
        .as_deref()
        .is_some_and(first_notice_for);

    // Keep the frontend state fresh even if the release was published after the
    // app had already been running for hours.
    let _ = app.emit("clipboard://update-available", status.clone());

    if !first_notice {
        return;
    }

    if NOTIFICATIONS_READY.load(Ordering::SeqCst) {
        if let Some(version) = status.version.as_deref() {
            if let Err(error) = app
                .notification()
                .builder()
                .title("Clipboard Preview update available")
                .body(format!("Version {version} is ready to install."))
                .show()
            {
                log::warn!("Could not show update notification: {error}");
            }
        }
    }

    // Native notifications can be suppressed by macOS or desktop notification
    // settings. Opening Settings once per discovered version gives the signed
    // in-app update banner a deterministic fallback instead of silently relying
    // on an OS notification that may never be visible.
    if let Err(error) = overlays::open_settings(app) {
        log::warn!("Could not surface the available update in Settings: {error}");
    }
}

pub fn schedule_background_check(app: AppHandle) {
    if !ready() {
        return;
    }

    thread::spawn(move || {
        let mut delay = UPDATE_CHECK_DELAY;
        loop {
            thread::sleep(delay);
            delay = UPDATE_CHECK_INTERVAL;

            let check_app = app.clone();
            let result =
                tauri::async_runtime::block_on(async move { check_inner(&check_app).await });
            match result {
                Ok(status) if status.available => surface_available_update(&app, status),
                Ok(_) => {}
                Err(error) => {
                    log::warn!("Background update check failed: {error}");
                }
            }
        }
    });
}
