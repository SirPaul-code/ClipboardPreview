#![cfg(target_os = "macos")]

use std::{
    ffi::c_void,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
    thread,
    time::{Duration, SystemTime},
};

use tauri::{AppHandle, Emitter, Manager};

use crate::{
    models::{InteractionMode, TAB_HOLD_DELAY_MS},
    overlays, permissions, selection,
    state::AppState,
};

type CGEventRef = *mut c_void;
type CGEventSourceRef = *mut c_void;
type CFMachPortRef = *mut c_void;
type CFRunLoopSourceRef = *mut c_void;
type CFRunLoopRef = *mut c_void;
type CFStringRef = *const c_void;
type CGEventTapProxy = *mut c_void;
type CGEventType = u32;
type CGEventFlags = u64;
type CGEventMask = u64;
type CGKeyCode = u16;
type UniCharCount = usize;

type CGEventTapCallback = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    event_type: CGEventType,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef;

const CG_SESSION_EVENT_TAP: u32 = 1;
const CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
const CG_EVENT_TAP_OPTION_DEFAULT: u32 = 0;
const CG_EVENT_NULL: CGEventType = 0;
const CG_EVENT_KEY_DOWN: CGEventType = 10;
const CG_EVENT_KEY_UP: CGEventType = 11;
const CG_EVENT_FLAGS_CHANGED: CGEventType = 12;
const CG_EVENT_SCROLL_WHEEL: CGEventType = 22;
const CG_EVENT_TAP_DISABLED_BY_TIMEOUT: CGEventType = u32::MAX - 1;
const CG_EVENT_TAP_DISABLED_BY_USER_INPUT: CGEventType = u32::MAX;
const CG_KEYBOARD_EVENT_KEYCODE: u32 = 9;
const CG_SCROLL_WHEEL_EVENT_DELTA_AXIS_1: u32 = 11;
const CG_SCROLL_WHEEL_EVENT_DELTA_AXIS_2: u32 = 12;
const CG_EVENT_FLAG_MASK_SHIFT: CGEventFlags = 1 << 17;
const CG_EVENT_FLAG_MASK_CONTROL: CGEventFlags = 1 << 18;
const CG_EVENT_FLAG_MASK_ALTERNATE: CGEventFlags = 1 << 19;
const CG_EVENT_FLAG_MASK_COMMAND: CGEventFlags = 1 << 20;
const CG_EVENT_SOURCE_STATE_HID_SYSTEM: i32 = 1;
const TAB_KEY_CODE: CGKeyCode = 48;

const MAC_ACCESSIBILITY_WARNING: &str = "Clipboard Switcher needs macOS Accessibility permission for global keyboard and wheel capture. Enable Clipboard Preview in System Settings → Privacy & Security → Accessibility; this message disappears automatically when native input is ready.";

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: CGEventMask,
        callback: Option<CGEventTapCallback>,
        user_info: *mut c_void,
    ) -> CFMachPortRef;
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    fn CGEventGetFlags(event: CGEventRef) -> CGEventFlags;
    fn CGEventKeyboardGetUnicodeString(
        event: CGEventRef,
        max_string_length: UniCharCount,
        actual_string_length: *mut UniCharCount,
        unicode_string: *mut u16,
    );
    fn CGEventSourceCreate(state_id: i32) -> CGEventSourceRef;
    fn CGEventCreateKeyboardEvent(
        source: CGEventSourceRef,
        virtual_key: CGKeyCode,
        key_down: bool,
    ) -> CGEventRef;
    fn CGEventPost(tap: u32, event: CGEventRef);
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    static kCFRunLoopCommonModes: CFStringRef;
    fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: CFMachPortRef,
        order: isize,
    ) -> CFRunLoopSourceRef;
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopAddSource(run_loop: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
    fn CFRunLoopRun();
    fn CFRelease(value: *const c_void);
}

struct CallbackContext {
    app: AppHandle,
    tap: AtomicPtr<c_void>,
}

pub fn start(app: AppHandle) {
    thread::spawn(move || {
        if !permissions::native_input_permissions_granted() {
            push_warning(&app, MAC_ACCESSIBILITY_WARNING);
            if let Err(error) = permissions::wait_for_native_input_permissions() {
                let message = format!("Could not complete macOS Accessibility setup: {error}");
                log::warn!("{message}");
                push_warning(&app, &message);
                return;
            }
            remove_warning(&app, MAC_ACCESSIBILITY_WARNING);
        }

        if let Err(error) = run_event_tap(app.clone()) {
            if let Some(state) = app.try_state::<AppState>() {
                state.native_input_ready.store(false, Ordering::SeqCst);
            }
            let message = format!(
                "macOS native input capture could not start: {error}. Clipboard Switcher shortcuts cannot intercept keys until this is fixed."
            );
            log::warn!("{message}");
            push_warning(&app, &message);
        }
    });
}

fn run_event_tap(app: AppHandle) -> Result<(), String> {
    let context = Box::new(CallbackContext {
        app: app.clone(),
        tap: AtomicPtr::new(ptr::null_mut()),
    });
    let context_ptr = Box::into_raw(context);

    // kCGHIDEventTap is root-only. Clipboard Preview runs as the signed-in user,
    // so use an active session event tap. Accessibility authorizes this modifying
    // tap and lets us suppress the trigger key while the switcher is active.
    let tap = unsafe {
        CGEventTapCreate(
            CG_SESSION_EVENT_TAP,
            CG_HEAD_INSERT_EVENT_TAP,
            CG_EVENT_TAP_OPTION_DEFAULT,
            u64::from(u32::MAX),
            Some(raw_callback),
            context_ptr.cast(),
        )
    };
    if tap.is_null() {
        unsafe { drop(Box::from_raw(context_ptr)) };
        return Err(
            "CGEventTapCreate(kCGSessionEventTap) returned NULL even though Accessibility is enabled"
                .into(),
        );
    }

    unsafe { (*context_ptr).tap.store(tap, Ordering::SeqCst) };

    let source = unsafe { CFMachPortCreateRunLoopSource(ptr::null(), tap, 0) };
    if source.is_null() {
        unsafe {
            CFRelease(tap.cast_const());
            drop(Box::from_raw(context_ptr));
        }
        return Err("CFMachPortCreateRunLoopSource returned NULL".into());
    }

    let run_loop = unsafe { CFRunLoopGetCurrent() };
    if run_loop.is_null() {
        unsafe {
            CFRelease(source.cast_const());
            CFRelease(tap.cast_const());
            drop(Box::from_raw(context_ptr));
        }
        return Err("CFRunLoopGetCurrent returned NULL".into());
    }

    unsafe {
        CFRunLoopAddSource(run_loop, source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, true);
    }

    if let Some(state) = app.try_state::<AppState>() {
        state.native_input_ready.store(true, Ordering::SeqCst);
    }
    remove_capture_failure_warnings(&app);
    let _ = app.emit("clipboard://status-changed", ());

    unsafe { CFRunLoopRun() };

    if let Some(state) = app.try_state::<AppState>() {
        state.native_input_ready.store(false, Ordering::SeqCst);
    }
    let _ = app.emit("clipboard://status-changed", ());

    unsafe {
        CFRelease(source.cast_const());
        CFRelease(tap.cast_const());
        drop(Box::from_raw(context_ptr));
    }

    Err("the macOS input run loop exited unexpectedly".into())
}

unsafe extern "C" fn raw_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    if user_info.is_null() || event.is_null() {
        return event;
    }

    let context = unsafe { &*(user_info as *const CallbackContext) };

    if matches!(
        event_type,
        CG_EVENT_TAP_DISABLED_BY_TIMEOUT | CG_EVENT_TAP_DISABLED_BY_USER_INPUT
    ) {
        let tap = context.tap.load(Ordering::SeqCst);
        if !tap.is_null() {
            unsafe { CGEventTapEnable(tap, true) };
        }
        return event;
    }

    let Some(native_event) = convert_event(event_type, event) else {
        return event;
    };

    let fallback = native_event.clone();
    let result = catch_unwind(AssertUnwindSafe(|| handle_event(&context.app, native_event)));
    match result {
        Ok(Some(_)) => event,
        Ok(None) => ptr::null_mut(),
        Err(_) => {
            log::error!("macOS event-tap callback panicked; passing the event through");
            let _ = fallback;
            event
        }
    }
}

fn convert_event(event_type: CGEventType, event: CGEventRef) -> Option<rdev::Event> {
    use rdev::EventType;

    let event_type = match event_type {
        CG_EVENT_KEY_DOWN => {
            let code = key_code(event)?;
            EventType::KeyPress(key_from_code(code))
        }
        CG_EVENT_KEY_UP => {
            let code = key_code(event)?;
            EventType::KeyRelease(key_from_code(code))
        }
        CG_EVENT_FLAGS_CHANGED => {
            let code = key_code(event)?;
            let key = key_from_code(code);
            let flags = unsafe { CGEventGetFlags(event) };
            if modifier_is_pressed(key, flags)? {
                EventType::KeyPress(key)
            } else {
                EventType::KeyRelease(key)
            }
        }
        CG_EVENT_SCROLL_WHEEL => {
            let delta_y = unsafe { CGEventGetIntegerValueField(event, CG_SCROLL_WHEEL_EVENT_DELTA_AXIS_1) };
            let delta_x = unsafe { CGEventGetIntegerValueField(event, CG_SCROLL_WHEEL_EVENT_DELTA_AXIS_2) };
            EventType::Wheel { delta_x, delta_y }
        }
        CG_EVENT_NULL => return None,
        _ => return None,
    };

    let name = match event_type {
        EventType::KeyPress(_) => unicode_name(event),
        _ => None,
    };

    Some(rdev::Event {
        event_type,
        time: SystemTime::now(),
        name,
    })
}

fn key_code(event: CGEventRef) -> Option<CGKeyCode> {
    unsafe { CGEventGetIntegerValueField(event, CG_KEYBOARD_EVENT_KEYCODE) }
        .try_into()
        .ok()
}

fn unicode_name(event: CGEventRef) -> Option<String> {
    let mut buffer = [0_u16; 8];
    let mut length = 0usize;
    unsafe {
        CGEventKeyboardGetUnicodeString(
            event,
            buffer.len(),
            &mut length,
            buffer.as_mut_ptr(),
        );
    }
    if length == 0 {
        return None;
    }
    String::from_utf16(&buffer[..length.min(buffer.len())])
        .ok()
        .filter(|value| !value.is_empty())
}

fn modifier_is_pressed(key: rdev::Key, flags: CGEventFlags) -> Option<bool> {
    use rdev::Key;
    match key {
        Key::ShiftLeft | Key::ShiftRight => Some(flags & CG_EVENT_FLAG_MASK_SHIFT != 0),
        Key::ControlLeft | Key::ControlRight => Some(flags & CG_EVENT_FLAG_MASK_CONTROL != 0),
        Key::Alt | Key::AltGr => Some(flags & CG_EVENT_FLAG_MASK_ALTERNATE != 0),
        Key::MetaLeft | Key::MetaRight => Some(flags & CG_EVENT_FLAG_MASK_COMMAND != 0),
        _ => None,
    }
}

fn key_from_code(code: CGKeyCode) -> rdev::Key {
    use rdev::Key;

    match code {
        58 => Key::Alt,
        61 => Key::AltGr,
        51 => Key::Backspace,
        57 => Key::CapsLock,
        59 => Key::ControlLeft,
        62 => Key::ControlRight,
        125 => Key::DownArrow,
        53 => Key::Escape,
        122 => Key::F1,
        120 => Key::F2,
        99 => Key::F3,
        118 => Key::F4,
        96 => Key::F5,
        97 => Key::F6,
        98 => Key::F7,
        100 => Key::F8,
        101 => Key::F9,
        109 => Key::F10,
        103 => Key::F11,
        111 => Key::F12,
        123 => Key::LeftArrow,
        55 => Key::MetaLeft,
        54 => Key::MetaRight,
        36 => Key::Return,
        124 => Key::RightArrow,
        56 => Key::ShiftLeft,
        60 => Key::ShiftRight,
        49 => Key::Space,
        TAB_KEY_CODE => Key::Tab,
        126 => Key::UpArrow,
        50 => Key::BackQuote,
        18 => Key::Num1,
        19 => Key::Num2,
        20 => Key::Num3,
        21 => Key::Num4,
        23 => Key::Num5,
        22 => Key::Num6,
        26 => Key::Num7,
        28 => Key::Num8,
        25 => Key::Num9,
        29 => Key::Num0,
        27 => Key::Minus,
        24 => Key::Equal,
        12 => Key::KeyQ,
        13 => Key::KeyW,
        14 => Key::KeyE,
        15 => Key::KeyR,
        17 => Key::KeyT,
        16 => Key::KeyY,
        32 => Key::KeyU,
        34 => Key::KeyI,
        31 => Key::KeyO,
        35 => Key::KeyP,
        33 => Key::LeftBracket,
        30 => Key::RightBracket,
        0 => Key::KeyA,
        1 => Key::KeyS,
        2 => Key::KeyD,
        3 => Key::KeyF,
        5 => Key::KeyG,
        4 => Key::KeyH,
        38 => Key::KeyJ,
        40 => Key::KeyK,
        37 => Key::KeyL,
        41 => Key::SemiColon,
        39 => Key::Quote,
        42 => Key::BackSlash,
        6 => Key::KeyZ,
        7 => Key::KeyX,
        8 => Key::KeyC,
        9 => Key::KeyV,
        11 => Key::KeyB,
        45 => Key::KeyN,
        46 => Key::KeyM,
        43 => Key::Comma,
        47 => Key::Dot,
        44 => Key::Slash,
        114 => Key::Insert,
        115 => Key::Home,
        116 => Key::PageUp,
        117 => Key::Delete,
        119 => Key::End,
        121 => Key::PageDown,
        other => Key::Unknown(other.into()),
    }
}

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

    if handle_history_trigger(app, &state, &event) {
        return None;
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

fn handle_history_trigger(app: &AppHandle, state: &AppState, event: &rdev::Event) -> bool {
    use rdev::EventType;

    let (shortcut, mode) = {
        let settings = state.settings.read();
        (
            settings.shortcuts.history_selector.clone(),
            settings.history.interaction_mode.clone(),
        )
    };

    // Plain Tab keeps its tap-vs-hold replay path below. Every other macOS
    // history shortcut is handled by the same native event tap, including
    // layout-specific printable keys such as § and modifier combinations.
    if shortcut.eq_ignore_ascii_case("Tab") || shortcut.trim().is_empty() {
        return false;
    }

    match event.event_type {
        EventType::KeyPress(key) if shortcut_matches(state, event, key, &shortcut) => {
            let mut trigger = state.mac_history_trigger_key.lock();
            if trigger.is_none() {
                *trigger = Some(key);
                drop(trigger);
                if !state.selector.lock().active {
                    if let Err(error) = overlays::begin(app, mode) {
                        *state.mac_history_trigger_key.lock() = None;
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
            if matches!(mode, InteractionMode::HoldRelease) && state.selector.lock().active {
                let _ = selection::accept(app);
            }
            true
        }
        _ => false,
    }
}

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

fn modifier_pressed(state: &AppState) -> bool {
    state.alt_down.load(Ordering::SeqCst)
        || state.control_down.load(Ordering::SeqCst)
        || state.shift_down.load(Ordering::SeqCst)
        || state.meta_down.load(Ordering::SeqCst)
}

fn reset_native_state(state: &AppState) {
    state.tab_down.store(false, Ordering::SeqCst);
    state.tab_hold_triggered.store(false, Ordering::SeqCst);
    state.replaying_tab.store(false, Ordering::SeqCst);
    state.alt_down.store(false, Ordering::SeqCst);
    state.control_down.store(false, Ordering::SeqCst);
    state.shift_down.store(false, Ordering::SeqCst);
    state.meta_down.store(false, Ordering::SeqCst);
    *state.mac_history_trigger_key.lock() = None;
}

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

fn replay_tab(app: AppHandle) {
    thread::spawn(move || {
        {
            let Some(state) = app.try_state::<AppState>() else {
                return;
            };
            state.replaying_tab.store(true, Ordering::SeqCst);
        }

        let result = replay_tab_native();

        if let Some(state) = app.try_state::<AppState>() {
            state.replaying_tab.store(false, Ordering::SeqCst);
        }

        if let Err(error) = result {
            log::warn!("Could not replay a short Tab press to the foreground application: {error}");
        }
    });
}

fn replay_tab_native() -> Result<(), String> {
    let source = unsafe { CGEventSourceCreate(CG_EVENT_SOURCE_STATE_HID_SYSTEM) };
    if source.is_null() {
        return Err("CGEventSourceCreate returned NULL".into());
    }

    let press = unsafe { CGEventCreateKeyboardEvent(source, TAB_KEY_CODE, true) };
    let release = unsafe { CGEventCreateKeyboardEvent(source, TAB_KEY_CODE, false) };
    if press.is_null() || release.is_null() {
        unsafe {
            if !press.is_null() {
                CFRelease(press.cast_const());
            }
            if !release.is_null() {
                CFRelease(release.cast_const());
            }
            CFRelease(source.cast_const());
        }
        return Err("CGEventCreateKeyboardEvent returned NULL".into());
    }

    unsafe {
        CGEventPost(CG_SESSION_EVENT_TAP, press);
        thread::sleep(Duration::from_millis(8));
        CGEventPost(CG_SESSION_EVENT_TAP, release);
        CFRelease(press.cast_const());
        CFRelease(release.cast_const());
        CFRelease(source.cast_const());
    }
    Ok(())
}

fn history_uses_tab(state: &AppState) -> bool {
    let settings = state.settings.read();
    !settings.general.monitoring_paused
        && settings.shortcuts.history_selector.eq_ignore_ascii_case("Tab")
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

fn remove_capture_failure_warnings(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let mut warnings = state.startup_warnings.write();
    let before = warnings.len();
    warnings.retain(|warning| !warning.starts_with("macOS native input capture could not start:"));
    if warnings.len() != before {
        let _ = app.emit("clipboard://status-changed", ());
    }
}
