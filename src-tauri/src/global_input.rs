use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    sync::atomic::Ordering,
    thread,
    time::Duration,
};

use tauri::{AppHandle, Emitter, Manager};

use crate::{
    models::{InteractionMode, TAB_HOLD_DELAY_MS},
    overlays, permissions, selection,
    state::AppState,
};

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn start(app: AppHandle) {
    if cfg!(target_os = "macos") && !permissions::accessibility_granted() {
        push_warning(
            &app,
            "Tab hold and global wheel selection need macOS Accessibility permission. Grant it or choose a modifier-based History shortcut in Settings.",
        );
        return;
    }

    thread::spawn(move || {
        if let Some(state) = app.try_state::<AppState>() {
            state.native_input_ready.store(true, Ordering::SeqCst);
        }

        let callback_app = app.clone();
        let result = rdev::grab(move |event| {
            let fallback = event.clone();
            match catch_unwind(AssertUnwindSafe(|| handle_event(&callback_app, event))) {
                Ok(value) => value,
                Err(_) => {
                    log::error!("Global input callback panicked; passing the input event through");
                    Some(fallback)
                }
            }
        });

        if let Some(state) = app.try_state::<AppState>() {
            state.native_input_ready.store(false, Ordering::SeqCst);
        }

        if let Err(error) = result {
            let message = format!(
                "Global input capture stopped: {error:?}. Tab hold is unavailable; choose another History shortcut if needed."
            );
            log::warn!("{message}");
            push_warning(&app, &message);
        }
    });
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn handle_event(app: &AppHandle, event: rdev::Event) -> Option<rdev::Event> {
    use rdev::{EventType, Key};

    let Some(state) = app.try_state::<AppState>() else {
        return Some(event);
    };

    if state.replaying_tab.load(Ordering::SeqCst) {
        return Some(event);
    }

    match event.event_type {
        EventType::Wheel { delta_y, .. } if delta_y != 0 => {
            if state.selector.lock().active {
                let _ = selection::navigate(app, if delta_y < 0 { 1 } else { -1 });
                return None;
            }
        }
        EventType::KeyPress(Key::Tab) if history_uses_tab(&state) => {
            if !state.tab_down.swap(true, Ordering::SeqCst) {
                state.tab_hold_triggered.store(false, Ordering::SeqCst);
                schedule_tab_hold(app.clone());
            }
            return None;
        }
        EventType::KeyRelease(Key::Tab) if history_uses_tab(&state) => {
            state.tab_down.store(false, Ordering::SeqCst);
            if state.tab_hold_triggered.swap(false, Ordering::SeqCst) {
                let mode = state.settings.read().history.interaction_mode.clone();
                if matches!(mode, InteractionMode::HoldRelease) {
                    let _ = selection::accept(app);
                }
            } else {
                replay_tab(app.clone());
            }
            return None;
        }
        _ => {}
    }

    Some(event)
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn schedule_tab_hold(app: AppHandle) {
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(TAB_HOLD_DELAY_MS));
        let Some(state) = app.try_state::<AppState>() else {
            return;
        };
        if !state.tab_down.load(Ordering::SeqCst) || !history_uses_tab(&state) {
            return;
        }
        if state.tab_hold_triggered.swap(true, Ordering::SeqCst) {
            return;
        }

        let mode = state.settings.read().history.interaction_mode.clone();
        drop(state);
        if let Err(error) = overlays::begin(&app, mode) {
            if let Some(state) = app.try_state::<AppState>() {
                state.tab_hold_triggered.store(false, Ordering::SeqCst);
            }
            log::warn!("Could not open clipboard switcher from Tab hold: {error}");
        }
    });
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn replay_tab(app: AppHandle) {
    thread::spawn(move || {
        use rdev::{simulate, EventType, Key};

        let Some(state) = app.try_state::<AppState>() else {
            return;
        };
        state.replaying_tab.store(true, Ordering::SeqCst);
        drop(state);

        let press = simulate(&EventType::KeyPress(Key::Tab));
        thread::sleep(Duration::from_millis(8));
        let release = simulate(&EventType::KeyRelease(Key::Tab));

        if let Some(state) = app.try_state::<AppState>() {
            state.replaying_tab.store(false, Ordering::SeqCst);
        }

        if press.is_err() || release.is_err() {
            log::warn!("Could not replay a short Tab press to the foreground application");
        }
    });
}

fn history_uses_tab(state: &AppState) -> bool {
    state
        .settings
        .read()
        .shortcuts
        .history_selector
        .eq_ignore_ascii_case("Tab")
}

fn push_warning(app: &AppHandle, message: &str) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let mut warnings = state.startup_warnings.write();
    if !warnings.iter().any(|warning| warning == message) {
        warnings.push(message.to_string());
        let _ = app.emit("clipboard://status-changed", ());
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn start(_app: AppHandle) {}
