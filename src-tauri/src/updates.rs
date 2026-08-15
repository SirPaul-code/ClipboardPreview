use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::UpdaterExt;

const UPDATE_ENDPOINT: &str =
    "https://github.com/SirPaul-code/ClipboardPreview/releases/latest/download/latest.json";
const UPDATE_CHECK_DELAY: Duration = Duration::from_secs(5);
const UPDATE_TIMEOUT: Duration = Duration::from_secs(20);

static UPDATER_READY: AtomicBool = AtomicBool::new(false);
static NOTIFICATIONS_READY: AtomicBool = AtomicBool::new(false);

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

    #[cfg(not(target_os = "windows"))]
    app.restart();

    #[allow(unreachable_code)]
    Ok(())
}

pub fn schedule_background_check(app: AppHandle) {
    if !ready() {
        return;
    }

    thread::spawn(move || {
        thread::sleep(UPDATE_CHECK_DELAY);
        let check_app = app.clone();
        let result = tauri::async_runtime::block_on(async move { check_inner(&check_app).await });
        match result {
            Ok(status) if status.available => {
                let _ = app.emit("clipboard://update-available", status.clone());
                if NOTIFICATIONS_READY.load(Ordering::SeqCst) {
                    if let Some(version) = status.version {
                        let _ = app
                            .notification()
                            .builder()
                            .title("Clipboard Preview update available")
                            .body(format!("Version {version} is ready to install."))
                            .show();
                    }
                }
            }
            Ok(_) => {}
            Err(error) => {
                log::warn!("Background update check failed: {error}");
            }
        }
    });
}
