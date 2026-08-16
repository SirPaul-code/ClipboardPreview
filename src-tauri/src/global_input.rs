use tauri::AppHandle;

#[cfg(any(target_os = "windows", target_os = "macos"))]
use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    sync::atomic::Ordering,
    thread,
    time::Duration,
};

#[cfg(target_os = "macos")]
use std::time::UNIX_EPOCH;

#[cfg(any(target_os = "windows", target_os = "macos"))]
use tauri::{Emitter, Manager};

#[cfg(any(target_os = "windows", target_os = "macos"))]
use crate::{
    models::{InteractionMode, TAB_HOLD_DELAY_MS},
    overlays, selection,
    state::AppState,
};

#[cfg(target_os = "macos")]
use crate::{permissions, shortcuts};

#[cfg(target_os = "macos")]
const MAC_NATIVE_INPUT_WARNING: &str = "Clipboard Switcher keyboard, arrow-key, mouse-wheel, and trackpad capture needs macOS Accessibility permission. Clipboard Preview will request access and open Privacy & Security → Accessibility; capture activates automatically after permission is granted.";

#[cfg(any(target_os = "windows", target_os = "macos"))]
const MOD_ALT: u8 = 1 << 0;
#[cfg(any(target_os = "windows", target_os = "macos"))]
const MOD_CONTROL: u8 = 1 << 1;
#[cfg(any(target_os = "windows", target_os = "macos"))]
const MOD_SHIFT: u8 = 1 << 2;
#[cfg(any(target_os = "windows", target_os = "macos"))]
const MOD_META: u8 = 1 << 3;

#[cfg(target_os = "macos")]
const MAC_WHEEL_NAVIGATION_INTERVAL_MS: u64 = 80;

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
    thread::spawn(move || {
        #[cfg(target_os = "macos")]
        {
            if !permissions::native_input_permissions_granted() {
                push_warning(&app, MAC_NATIVE_INPUT_WARNING);
                if let Err(error) = permissions::wait_for_native_input_permissions() {
                    log::warn!("Could not complete macOS native input permission flow: {error}");
                    return;
                }
                remove_warning(&app, MAC_NATIVE_INPUT_WARNING);
            }
        }

        run_grab(app);
    });
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn run_grab(app: AppHandle) {
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
            "Global input capture stopped: {error:?}. Clipboard Switcher keyboard/wheel capture is unavailable until the app is restarted."
        );
        log::warn!("{message}");
        push_warning(&app, &message);
    }
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
    #[cfg(target_os = "macos")]
    {
        *state.mac_history_trigger_key.lock() = None;
        state.mac_history_trigger_modifiers.store(0, Ordering::SeqCst);
        state.mac_last_wheel_navigation_ms.store(0, Ordering::SeqCst);
    }
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

    // Every macOS history shortcut is handled in this same native stream. Process
    // its release/repeat before the active-selector input firewall so Hold/Release
    // can finish cleanly without leaking the trigger back to the foreground app.
    #[cfg(target_os = "macos")]
    if selector_active && handle_macos_history_trigger(app, &state, &event) {
        return None;
    }

    // Plain Tab has the special tap-vs-hold replay behavior. Handle it before the
    // selector firewall for the same reason as the macOS native trigger above.
    match event.event_type {
        EventType::KeyPress(Key::Tab) if history_uses_tab(&state) && !modifier_pressed(&state) => {
            if !state.tab_down.swap(true, Ordering::SeqCst) {
                state.tab_hold_triggered.store(false, Ordering::SeqCst);
                #[cfg(target_os = "macos")]
                {
                    state.mac_history_trigger_modifiers.store(0, Ordering::SeqCst);
                    state.mac_last_wheel_navigation_ms.store(0, Ordering::SeqCst);
                }
                schedule_tab_hold(app.clone());
            }
            return None;
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
            #[cfg(target_os = "macos")]
            {
                state.mac_history_trigger_modifiers.store(0, Ordering::SeqCst);
                state.mac_last_wheel_navigation_ms.store(0, Ordering::SeqCst);
            }
            return None;
        }
        _ => {}
    }

    if selector_active {
        #[cfg(target_os = "macos")]
        let mode = state.settings.read().history.interaction_mode.clone();
        match event.event_type {
            EventType::Wheel { delta_x, delta_y } => {
                if delta_y != 0 && delta_y.abs() >= delta_x.abs() {
                    let should_navigate = {
                        #[cfg(target_os = "macos")]
                        {
                            mac_wheel_navigation_ready(&state, &event)
                        }
                        #[cfg(not(target_os = "macos"))]
                        {
                            true
                        }
                    };
                    if should_navigate {
                        let _ = selection::navigate(app, if delta_y < 0 { 1 } else { -1 });
                    }
                    return None;
                }

                // Trackpad gestures can contain a horizontal component. While the
                // macOS switcher is active, consume the whole scroll gesture so the
                // foreground application never scrolls behind the preview.
                #[cfg(target_os = "macos")]
                {
                    return None;
                }
            }
            EventType::KeyPress(key) => {
                if let Some(delta) = navigation_delta(&state, &event, key) {
                    let _ = selection::navigate(app, delta);
                    return None;
                }

                #[cfg(target_os = "macos")]
                {
                    if matches!(mode, InteractionMode::Sticky) {
                        match key {
                            Key::Return | Key::KpReturn => {
                                let _ = selection::accept(app);
                            }
                            Key::Escape => selection::cancel(app),
                            _ => {}
                        }
                    }

                    // The switcher owns keyboard input while visible. Do not let
                    // unrelated app shortcuts, arrows, or typed characters fire in
                    // the application underneath it.
                    return None;
                }
            }
            EventType::KeyRelease(key) => {
                if navigation_delta(&state, &event, key).is_some() {
                    return None;
                }

                #[cfg(target_os = "macos")]
                {
                    return None;
                }
            }
            EventType::ButtonPress(_) | EventType::ButtonRelease(_) => {
                #[cfg(target_os = "macos")]
                if matches!(mode, InteractionMode::HoldRelease) {
                    // Hold/Release overlays are deliberately non-focusable. Swallow
                    // clicks while they are visible so the app underneath cannot be
                    // accidentally activated or clicked during selection.
                    return None;
                }
            }
            _ => {}
        }
    }

    #[cfg(target_os = "macos")]
    if handle_macos_history_trigger(app, &state, &event) {
        return None;
    }

    Some(event)
}

#[cfg(target_os = "macos")]
fn handle_macos_history_trigger(app: &AppHandle, state: &AppState, event: &rdev::Event) -> bool {
    use rdev::EventType;

    let (shortcut, mode) = {
        let settings = state.settings.read();
        (
            settings.shortcuts.history_selector.clone(),
            settings.history.interaction_mode.clone(),
        )
    };

    // Plain Tab keeps its dedicated tap-vs-hold replay path. Every other macOS
    // history selector stays in this native stream so the trigger, navigation,
    // and input suppression cannot race a second global-hotkey implementation.
    if shortcut.eq_ignore_ascii_case("Tab")
        || shortcut.trim().is_empty()
        || !shortcuts::history_uses_native_capture(&shortcut)
    {
        return false;
    }

    match event.event_type {
        EventType::KeyPress(key) if shortcut_matches(state, event, key, &shortcut) => {
            let mut trigger = state.mac_history_trigger_key.lock();
            if trigger.is_none() {
                *trigger = Some(key);
                state
                    .mac_history_trigger_modifiers
                    .store(current_modifier_mask(state), Ordering::SeqCst);
                state.mac_last_wheel_navigation_ms.store(0, Ordering::SeqCst);
                drop(trigger);
                if !state.selector.lock().active {
                    if let Err(error) = overlays::begin(app, mode) {
                        *state.mac_history_trigger_key.lock() = None;
                        state.mac_history_trigger_modifiers.store(0, Ordering::SeqCst);
                        log::warn!("Could not open Clipboard Switcher from macOS native shortcut: {error}");
                    }
                }
            }
            true
        }
        EventType::KeyPress(key)
            if state
                .mac_history_trigger_key
                .lock()
                .as_ref()
                .is_some_and(|trigger| *trigger == key) =>
        {
            // Consume key-repeat events while the trigger is held.
            true
        }
        EventType::KeyRelease(key) => {
            let matched = state
                .mac_history_trigger_key
                .lock()
                .as_ref()
                .is_some_and(|trigger| *trigger == key);
            if !matched {
                return false;
            }

            *state.mac_history_trigger_key.lock() = None;
            state.mac_history_trigger_modifiers.store(0, Ordering::SeqCst);
            state.mac_last_wheel_navigation_ms.store(0, Ordering::SeqCst);
            if matches!(mode, InteractionMode::HoldRelease) && state.selector.lock().active {
                let _ = selection::accept(app);
            }
            true
        }
        _ => false,
    }
}

#[cfg(target_os = "macos")]
fn mac_wheel_navigation_ready(state: &AppState, event: &rdev::Event) -> bool {
    let now_ms = event
        .time
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or(0);
    let last = state.mac_last_wheel_navigation_ms.load(Ordering::SeqCst);
    if last != 0 && now_ms.saturating_sub(last) < MAC_WHEEL_NAVIGATION_INTERVAL_MS {
        return false;
    }
    state
        .mac_last_wheel_navigation_ms
        .store(now_ms.max(1), Ordering::SeqCst);
    true
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn navigation_delta(state: &AppState, event: &rdev::Event, key: rdev::Key) -> Option<i32> {
    let settings = state.settings.read();
    let ignored_modifiers = {
        #[cfg(target_os = "macos")]
        {
            state.mac_history_trigger_modifiers.load(Ordering::SeqCst)
        }
        #[cfg(not(target_os = "macos"))]
        {
            0
        }
    };

    if shortcut_matches_with_ignored_modifiers(
        state,
        event,
        key,
        &settings.shortcuts.previous_item,
        ignored_modifiers,
    ) {
        Some(-1)
    } else if shortcut_matches_with_ignored_modifiers(
        state,
        event,
        key,
        &settings.shortcuts.next_item,
        ignored_modifiers,
    ) {
        Some(1)
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
fn shortcut_matches(state: &AppState, event: &rdev::Event, key: rdev::Key, shortcut: &str) -> bool {
    shortcut_matches_with_ignored_modifiers(state, event, key, shortcut, 0)
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn shortcut_matches_with_ignored_modifiers(
    state: &AppState,
    event: &rdev::Event,
    key: rdev::Key,
    shortcut: &str,
    ignored_modifiers: u8,
) -> bool {
    let parts: Vec<_> = shortcut
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    let Some(expected_key) = parts.last() else {
        return false;
    };

    let Some(actual_key) = canonical_key(event, key) else {
        return false;
    };
    if !actual_key.eq_ignore_ascii_case(expected_key) {
        return false;
    }

    let expected_modifiers = shortcut_modifier_mask(&parts);
    let actual_modifiers = current_modifier_mask(state);

    #[cfg(target_os = "macos")]
    let ignored = {
        let mut ignored = ignored_modifiers;
        // Printable symbols can require Shift/Option purely to produce the active
        // keyboard-layout character (for example $ or §). When Event.name already
        // proves the requested character matched, do not reject such implicit
        // layout modifiers unless the shortcut explicitly requested them.
        let printable_name_matches = expected_key.chars().count() == 1
            && event
                .name
                .as_ref()
                .is_some_and(|name| name.eq_ignore_ascii_case(expected_key));
        if printable_name_matches {
            if expected_modifiers & MOD_SHIFT == 0 {
                ignored |= MOD_SHIFT;
            }
            if expected_modifiers & MOD_ALT == 0 {
                ignored |= MOD_ALT;
            }
        }
        ignored
    };

    #[cfg(not(target_os = "macos"))]
    let ignored = ignored_modifiers;

    modifier_masks_match(actual_modifiers, expected_modifiers, ignored)
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn shortcut_modifier_mask(parts: &[&str]) -> u8 {
    let mut mask = 0;
    if parts.iter().any(|part| part.eq_ignore_ascii_case("Ctrl")) {
        mask |= MOD_CONTROL;
    }
    if parts.iter().any(|part| part.eq_ignore_ascii_case("Alt")) {
        mask |= MOD_ALT;
    }
    if parts.iter().any(|part| part.eq_ignore_ascii_case("Shift")) {
        mask |= MOD_SHIFT;
    }
    if parts.iter().any(|part| {
        part.eq_ignore_ascii_case("Cmd")
            || part.eq_ignore_ascii_case("Meta")
            || part.eq_ignore_ascii_case("Super")
    }) {
        mask |= MOD_META;
    }
    mask
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn current_modifier_mask(state: &AppState) -> u8 {
    let mut mask = 0;
    if state.alt_down.load(Ordering::SeqCst) {
        mask |= MOD_ALT;
    }
    if state.control_down.load(Ordering::SeqCst) {
        mask |= MOD_CONTROL;
    }
    if state.shift_down.load(Ordering::SeqCst) {
        mask |= MOD_SHIFT;
    }
    if state.meta_down.load(Ordering::SeqCst) {
        mask |= MOD_META;
    }
    mask
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn modifier_masks_match(actual: u8, expected: u8, ignored: u8) -> bool {
    (actual & !ignored) == (expected & !ignored)
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
            let mode = state.settings.read().history.interaction_mode.clone();
            mode
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

#[cfg(target_os = "macos")]
fn remove_warning(app: &AppHandle, message: &str) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let mut warnings = state.startup_warnings.write();
    let before = warnings.len();
    warnings.retain(|warning| warning != message);
    if warnings.len() != before {
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

#[cfg(all(test, any(target_os = "windows", target_os = "macos")))]
mod tests {
    use super::*;

    #[test]
    fn selector_owned_modifiers_are_ignored_for_navigation() {
        assert!(modifier_masks_match(
            MOD_META | MOD_SHIFT,
            0,
            MOD_META | MOD_SHIFT
        ));
    }

    #[test]
    fn unrelated_modifiers_still_block_navigation() {
        assert!(!modifier_masks_match(MOD_META | MOD_ALT, 0, MOD_META));
    }

    #[test]
    fn explicit_navigation_modifier_is_still_required() {
        assert!(!modifier_masks_match(0, MOD_CONTROL, 0));
        assert!(modifier_masks_match(MOD_CONTROL, MOD_CONTROL, 0));
    }
}
