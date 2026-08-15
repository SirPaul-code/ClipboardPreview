use tauri::AppHandle;

#[cfg(any(target_os = "windows", target_os = "macos"))]
use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    sync::atomic::Ordering,
    thread,
    time::Duration,
};

#[cfg(any(target_os = "windows", target_os = "macos"))]
use tauri::{Emitter, Manager};

#[cfg(any(target_os = "windows", target_os = "macos"))]
use crate::{
    models::{InteractionMode, TAB_HOLD_DELAY_MS},
    overlays, permissions, selection,
    state::AppState,
};

pub fn navigation_shortcut_supported(value: &str) -> bool {
    let Some(key) = value.split('+').next_back() else {
        return false;
    };
    let key = key.trim();
    if key.chars().count() == 1 {
        return true;
    }
    matches!(
        key.to_ascii_lowercase().as_str(),
        "arrowup"
            | "arrowdown"
            | "arrowleft"
            | "arrowright"
            | "pageup"
            | "pagedown"
            | "home"
            | "end"
            | "enter"
            | "space"
            | "escape"
            | "backspace"
            | "delete"
            | "insert"
            | "tab"
            | "f1"
            | "f2"
            | "f3"
            | "f4"
            | "f5"
            | "f6"
            | "f7"
            | "f8"
            | "f9"
            | "f10"
            | "f11"
            | "f12"
    )
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn start(app: AppHandle) {
    if cfg!(target_os = "macos") && !permissions::accessibility_granted() {
        push_warning(
            &app,
            "Tab hold and global wheel/keyboard selection need macOS Accessibility permission. Grant it or choose a modifier-based History shortcut in Settings.",
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
                "Global input capture stopped: {error:?}. Tab hold, wheel and hold-mode navigation keys are unavailable; choose sticky mode or another History shortcut if needed."
            );
            log::warn!("{message}");
            push_warning(&app, &message);
        }
    });
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn reset_native_state(state: &AppState) {
    state.tab_down.store(false, Ordering::SeqCst);
    state.tab_hold_triggered.store(false, Ordering::SeqCst);
    state.replaying_tab.store(false, Ordering::SeqCst);
    state.alt_down.store(false, Ordering::SeqCst);
    state.control_down.store(false, Ordering::SeqCst);
    state.shift_down.store(false, Ordering::SeqCst);
    state.meta_down.store(false, Ordering::SeqCst);
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn reset_state(app: &AppHandle) {
    if let Some(state) = app.try_state::<AppState>() {
        reset_native_state(&state);
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn handle_event(app: &AppHandle, event: rdev::Event) -> Option<rdev::Event> {
    use rdev::{EventType, Key};

    let Some(state) = app.try_state::<AppState>() else {
        return Some(event);
    };

    if state.settings.read().general.monitoring_paused {
        reset_native_state(&state);
        return Some(event);
    }

    update_modifier_state(&state, &event.event_type);

    if state.replaying_tab.load(Ordering::SeqCst) {
        return Some(event);
    }

    let selector_active = state.selector.lock().active;
    if selector_active {
        match event.event_type {
            EventType::Wheel { delta_y, .. } if delta_y != 0 => {
                let _ = selection::navigate(app, if delta_y < 0 { 1 } else { -1 });
                return None;
            }
            EventType::KeyPress(key) => {
                if let Some(delta) = navigation_delta(&state, &event, key) {
                    let _ = selection::navigate(app, delta);
                    return None;
                }
            }
            EventType::KeyRelease(key)
                if navigation_delta(&state, &event, key).is_some() =>
            {
                return None;
            }
            _ => {}
        }
    }

    match event.event_type {
        EventType::KeyPress(Key::Tab) if history_uses_tab(&state) && !modifier_pressed(&state) => {
            if !state.tab_down.swap(true, Ordering::SeqCst) {
                state.tab_hold_triggered.store(false, Ordering::SeqCst);
                schedule_tab_hold(app.clone());
            }
            None
        }
        EventType::KeyRelease(Key::Tab)
            if history_uses_tab(&state) && state.tab_down.load(Ordering::SeqCst) =>
        {
            state.tab_down.store(false, Ordering::SeqCst);
            if state.tab_hold_triggered.swap(false, Ordering::SeqCst) {
                let mode = state.settings.read().history.interaction_mode.clone();
                if matches!(mode, InteractionMode::HoldRelease) {
                    let _ = selection::accept(app);
                }
            } else {
                replay_tab(app.clone());
            }
            None
        }
        _ => Some(event),
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn navigation_delta(state: &AppState, event: &rdev::Event, key: rdev::Key) -> Option<i32> {
    let settings = state.settings.read();
    if shortcut_matches(state, event, key, &settings.shortcuts.previous_item) {
        Some(-1)
    } else if shortcut_matches(state, event, key, &settings.shortcuts.next_item) {
        Some(1)
    } else {
        None
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn shortcut_matches(state: &AppState, event: &rdev::Event, key: rdev::Key, shortcut: &str) -> bool {
    let parts: Vec<_> = shortcut
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    let Some(expected_key) = parts.last() else {
        return false;
    };

    let expected_ctrl = parts.iter().any(|part| part.eq_ignore_ascii_case("Ctrl"));
    let expected_alt = parts.iter().any(|part| part.eq_ignore_ascii_case("Alt"));
    let expected_shift = parts.iter().any(|part| part.eq_ignore_ascii_case("Shift"));
    let expected_meta = parts.iter().any(|part| {
        part.eq_ignore_ascii_case("Cmd")
            || part.eq_ignore_ascii_case("Meta")
            || part.eq_ignore_ascii_case("Super")
    });

    if expected_ctrl != state.control_down.load(Ordering::SeqCst)
        || expected_alt != state.alt_down.load(Ordering::SeqCst)
        || expected_shift != state.shift_down.load(Ordering::SeqCst)
        || expected_meta != state.meta_down.load(Ordering::SeqCst)
    {
        return false;
    }

    canonical_key(event, key)
        .is_some_and(|actual| actual.eq_ignore_ascii_case(expected_key))
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn canonical_key(event: &rdev::Event, key: rdev::Key) -> Option<String> {
    use rdev::Key;

    let special = match key {
        Key::UpArrow => Some("ArrowUp"),
        Key::DownArrow => Some("ArrowDown"),
        Key::LeftArrow => Some("ArrowLeft"),
        Key::RightArrow => Some("ArrowRight"),
        Key::PageUp => Some("PageUp"),
        Key::PageDown => Some("PageDown"),
        Key::Home => Some("Home"),
        Key::End => Some("End"),
        Key::Return | Key::KpReturn => Some("Enter"),
        Key::Space => Some("Space"),
        Key::Escape => Some("Escape"),
        Key::Backspace => Some("Backspace"),
        Key::Delete | Key::KpDelete => Some("Delete"),
        Key::Insert => Some("Insert"),
        Key::Tab => Some("Tab"),
        Key::F1 => Some("F1"),
        Key::F2 => Some("F2"),
        Key::F3 => Some("F3"),
        Key::F4 => Some("F4"),
        Key::F5 => Some("F5"),
        Key::F6 => Some("F6"),
        Key::F7 => Some("F7"),
        Key::F8 => Some("F8"),
        Key::F9 => Some("F9"),
        Key::F10 => Some("F10"),
        Key::F11 => Some("F11"),
        Key::F12 => Some("F12"),
        _ => None,
    };
    if let Some(value) = special {
        return Some(value.into());
    }

    if let Some(name) = event
        .name
        .as_ref()
        .filter(|name| !name.is_empty() && name.chars().count() == 1)
    {
        return Some(name.to_uppercase());
    }

    let physical = match key {
        Key::KeyA => Some("A"),
        Key::KeyB => Some("B"),
        Key::KeyC => Some("C"),
        Key::KeyD => Some("D"),
        Key::KeyE => Some("E"),
        Key::KeyF => Some("F"),
        Key::KeyG => Some("G"),
        Key::KeyH => Some("H"),
        Key::KeyI => Some("I"),
        Key::KeyJ => Some("J"),
        Key::KeyK => Some("K"),
        Key::KeyL => Some("L"),
        Key::KeyM => Some("M"),
        Key::KeyN => Some("N"),
        Key::KeyO => Some("O"),
        Key::KeyP => Some("P"),
        Key::KeyQ => Some("Q"),
        Key::KeyR => Some("R"),
        Key::KeyS => Some("S"),
        Key::KeyT => Some("T"),
        Key::KeyU => Some("U"),
        Key::KeyV => Some("V"),
        Key::KeyW => Some("W"),
        Key::KeyX => Some("X"),
        Key::KeyY => Some("Y"),
        Key::KeyZ => Some("Z"),
        Key::Num0 | Key::Kp0 => Some("0"),
        Key::Num1 | Key::Kp1 => Some("1"),
        Key::Num2 | Key::Kp2 => Some("2"),
        Key::Num3 | Key::Kp3 => Some("3"),
        Key::Num4 | Key::Kp4 => Some("4"),
        Key::Num5 | Key::Kp5 => Some("5"),
        Key::Num6 | Key::Kp6 => Some("6"),
        Key::Num7 | Key::Kp7 => Some("7"),
        Key::Num8 | Key::Kp8 => Some("8"),
        Key::Num9 | Key::Kp9 => Some("9"),
        _ => None,
    };
    physical.map(str::to_string)
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn update_modifier_state(state: &AppState, event_type: &rdev::EventType) {
    use rdev::{EventType, Key};

    let (key, pressed) = match event_type {
        EventType::KeyPress(key) => (*key, true),
        EventType::KeyRelease(key) => (*key, false),
        _ => return,
    };

    match key {
        Key::Alt | Key::AltGr => state.alt_down.store(pressed, Ordering::SeqCst),
        Key::ControlLeft | Key::ControlRight => {
            state.control_down.store(pressed, Ordering::SeqCst)
        }
        Key::ShiftLeft | Key::ShiftRight => state.shift_down.store(pressed, Ordering::SeqCst),
        Key::MetaLeft | Key::MetaRight => state.meta_down.store(pressed, Ordering::SeqCst),
        _ => {}
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn modifier_pressed(state: &AppState) -> bool {
    state.alt_down.load(Ordering::SeqCst)
        || state.control_down.load(Ordering::SeqCst)
        || state.shift_down.load(Ordering::SeqCst)
        || state.meta_down.load(Ordering::SeqCst)
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn schedule_tab_hold(app: AppHandle) {
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(TAB_HOLD_DELAY_MS));
        let mode = {
            let Some(state) = app.try_state::<AppState>() else {
                return;
            };
            if !state.tab_down.load(Ordering::SeqCst) || !history_uses_tab(&state) {
                return;
            }
            if state.tab_hold_triggered.swap(true, Ordering::SeqCst) {
                return;
            }
            state.settings.read().history.interaction_mode.clone()
        };

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

        {
            let Some(state) = app.try_state::<AppState>() else {
                return;
            };
            state.replaying_tab.store(true, Ordering::SeqCst);
        }

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

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn history_uses_tab(state: &AppState) -> bool {
    let settings = state.settings.read();
    !settings.general.monitoring_paused
        && settings.shortcuts.history_selector.eq_ignore_ascii_case("Tab")
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
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
pub fn start(_app: AppHandle) {
    // The hold threshold remains a shared setting even though Linux intentionally
    // does not compile native Tab interception.
    let _hold_delay_is_shared = crate::models::TAB_HOLD_DELAY_MS;
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn reset_state(_app: &AppHandle) {}
