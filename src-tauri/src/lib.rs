mod clipboard;
mod commands;
mod diagnostics;
mod global_input;
mod history;
#[cfg(target_os = "macos")]
mod macos_event_tap;
mod models;
mod overlays;
mod permissions;
mod selection;
mod settings_store;
mod shortcuts;
mod state;
mod tray;
mod updates;

use history::ClipboardHistory;
use state::AppState;
use tauri::{Manager, WindowEvent};

fn push_startup_warning(app: &tauri::AppHandle, warning: impl Into<String>) {
    if let Some(state) = app.try_state::<AppState>() {
        let warning = warning.into();
        let mut warnings = state.startup_warnings.write();
        if !warnings.iter().any(|existing| existing == &warning) {
            warnings.push(warning);
        }
    }
}

fn create_configured_windows(app: &mut tauri::App) {
    let configs = app.config().app.windows.clone();
    for config in configs {
        diagnostics::mark(&format!("creating webview window {}", config.label));
        let result = tauri::WebviewWindowBuilder::from_config(app.handle(), &config)
            .map(|builder| {
                builder.on_navigation(|url| {
                    url.scheme() == "tauri"
                        || matches!(
                            url.host_str(),
                            Some("tauri.localhost" | "localhost" | "127.0.0.1")
                        )
                })
            })
            .and_then(|builder| builder.build());
        if let Err(error) = result {
            let message = format!(
                "The {} window could not be created: {error}. See Diagnostics for the startup log.",
                config.label
            );
            diagnostics::mark(&message);
            push_startup_warning(app.handle(), message);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    diagnostics::install_panic_hook();
    diagnostics::mark("process entry");

    let hidden_launch = std::env::args().any(|argument| argument == "--hidden");

    let builder = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(shortcuts::plugin())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .setup(move |app| {
            diagnostics::mark("setup entered");

            let settings = settings_store::load_settings(app.handle());
            let history = ClipboardHistory::from_entries(
                settings_store::load_history(
                    app.handle(),
                    settings.history.persist_history,
                    settings.history.max_items,
                ),
                settings.history.max_items,
            );

            if !app.manage(AppState::new(settings.clone(), history)) {
                diagnostics::mark("FATAL: AppState was already managed unexpectedly");
                return Ok(());
            }
            diagnostics::mark("AppState managed before any webview creation");

            if let Err(error) = settings_store::save_settings(app.handle(), &settings) {
                push_startup_warning(
                    app.handle(),
                    format!("Settings migration could not be saved: {error}"),
                );
            }

            if diagnostics::last_crash_available() {
                push_startup_warning(
                    app.handle(),
                    "Clipboard Preview recorded a previous application panic. Open Advanced → Diagnostics to copy the report for the developer.",
                );
            }

            if let Err(error) = shortcuts::register_all(app.handle()) {
                log::error!("Global shortcut registration failed: {error}");
                push_startup_warning(
                    app.handle(),
                    format!(
                        "Global shortcuts could not be registered: {error}. Change the conflicting shortcut in Settings."
                    ),
                );
            }

            if let Err(error) = tray::build(app.handle()) {
                log::error!("Tray/menu bar initialization failed: {error}");
                push_startup_warning(
                    app.handle(),
                    format!(
                        "Tray/menu bar initialization failed: {error}. Settings will remain available when the window can be opened."
                    ),
                );
            }

            if let Err(error) = updates::register_notifications(app.handle()) {
                log::warn!("Native notifications unavailable: {error}");
            }
            if let Err(error) = updates::register_updater(app.handle()) {
                log::error!("Updater initialization failed: {error}");
                if updates::official_build() {
                    push_startup_warning(
                        app.handle(),
                        format!("Automatic updates are unavailable: {error}"),
                    );
                }
            }

            // Critical startup ordering invariant: AppState and optional native integrations
            // are initialized before any frontend can issue IPC commands.
            create_configured_windows(app);

            clipboard::start(app.handle().clone());
            #[cfg(target_os = "macos")]
            {
                // Keep the legacy rdev entry point referenced for shared state/reset code,
                // but never execute its kCGHIDEventTap grab path on macOS. A normal
                // desktop process uses the native session-level Core Graphics event tap.
                let _legacy_global_input_start: fn(tauri::AppHandle) = global_input::start;
                macos_event_tap::start(app.handle().clone());
            }
            #[cfg(not(target_os = "macos"))]
            global_input::start(app.handle().clone());
            updates::schedule_background_check(app.handle().clone());

            let has_startup_warning = app
                .state::<AppState>()
                .startup_warnings
                .read()
                .iter()
                .any(|warning| !warning.contains("recorded a previous application panic"));

            if hidden_launch
                && settings.first_run_completed
                && settings.general.start_hidden
                && !has_startup_warning
            {
                diagnostics::mark("autostart --hidden launch: settings remain hidden");
            } else if let Some(window) = app.get_webview_window("settings") {
                diagnostics::mark("showing settings window");
                let _ = window.show();
                let _ = window.set_focus();
            }

            diagnostics::mark("setup completed");
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "settings" {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::get_history,
            commands::get_image_preview,
            commands::save_settings,
            commands::clear_history,
            commands::select_history_item,
            commands::navigate_selection,
            commands::accept_selection,
            commands::cancel_selection,
            commands::show_history,
            commands::open_settings,
            commands::toggle_monitoring,
            commands::platform_status,
            commands::diagnostics_report,
            commands::clear_diagnostics,
            commands::open_diagnostics_folder,
            commands::open_external,
            commands::complete_first_run,
            commands::reset_settings,
            updates::check_for_updates,
            updates::install_update,
            commands::quit_app
        ]);

    let app = match builder.build(tauri::generate_context!()) {
        Ok(app) => app,
        Err(error) => {
            diagnostics::mark(&format!("FATAL: Tauri build failed: {error}"));
            return;
        }
    };

    diagnostics::mark("entering Tauri run loop");
    app.run(|handle, event| {
        if let tauri::RunEvent::ExitRequested { .. } = event {
            let Some(state) = handle.try_state::<AppState>() else {
                return;
            };
            let settings = state.settings.read().clone();
            if settings.history.clear_on_exit {
                settings_store::clear_history_file(handle);
            } else if settings.history.persist_history {
                let entries = state.history.lock().entries();
                if let Err(error) = settings_store::save_history(handle, true, &entries) {
                    log::warn!("Could not persist clipboard history during shutdown: {error}");
                }
            }
        }
    });
    diagnostics::mark_clean_shutdown();
}
