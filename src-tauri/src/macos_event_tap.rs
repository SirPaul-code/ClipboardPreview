#![cfg(target_os = "macos")]

use core::ptr::NonNull;
use std::{
    ffi::c_void,
    panic::{catch_unwind, AssertUnwindSafe},
    thread,
    time::{Duration, SystemTime},
};

use objc2_core_foundation::{CFMachPort, CFRunLoop, kCFRunLoopCommonModes};
use objc2_core_graphics::{
    CGEvent, CGEventField, CGEventFlags, CGEventSource, CGEventSourceStateID, CGEventTapCallBack,
    CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy, CGEventType,
    CGKeyCode, kCGEventMaskForAllEvents,
};
use objc2_foundation::NSAutoreleasePool;
use tauri::{AppHandle, Emitter, Manager};

use crate::{global_input, state::AppState};

const TAB_KEY_CODE: CGKeyCode = 48;

pub fn run(app: AppHandle) -> Result<(), String> {
    unsafe {
        let _pool = NSAutoreleasePool::new();
        let callback: CGEventTapCallBack = Some(raw_callback);
        let app_ptr = &app as *const AppHandle as *mut c_void;

        // kCGHIDEventTap is root-only. Clipboard Preview is a normal user app, so
        // the active filter must live at the session event tap. Accessibility
        // permission authorizes this modifying event tap and lets us suppress the
        // trigger key while the switcher is active.
        let tap = CGEvent::tap_create(
            CGEventTapLocation::SessionEventTap,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            kCGEventMaskForAllEvents.into(),
            callback,
            app_ptr,
        )
        .ok_or_else(|| {
            "macOS could not create the session event tap. Confirm Accessibility permission for Clipboard Preview, then quit and reopen the app."
                .to_string()
        })?;

        let source = CFMachPort::new_run_loop_source(None, Some(&tap), 0)
            .ok_or_else(|| "macOS could not create the Clipboard Preview input run-loop source".to_string())?;
        let run_loop = CFRunLoop::current()
            .ok_or_else(|| "macOS could not access the Clipboard Preview input run loop".to_string())?;
        run_loop.add_source(Some(&source), kCFRunLoopCommonModes);
        CGEvent::tap_enable(&tap, true);

        if let Some(state) = app.try_state::<AppState>() {
            state
                .native_input_ready
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
        let _ = app.emit("clipboard://status-changed", ());

        CFRunLoop::run();

        if let Some(state) = app.try_state::<AppState>() {
            state
                .native_input_ready
                .store(false, std::sync::atomic::Ordering::SeqCst);
        }
        let _ = app.emit("clipboard://status-changed", ());
    }

    Ok(())
}

pub fn replay_tab() -> Result<(), String> {
    unsafe {
        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .ok_or_else(|| "Could not create a macOS keyboard event source".to_string())?;
        let press = CGEvent::new_keyboard_event(Some(&source), TAB_KEY_CODE, true)
            .ok_or_else(|| "Could not create a macOS Tab key-down event".to_string())?;
        let release = CGEvent::new_keyboard_event(Some(&source), TAB_KEY_CODE, false)
            .ok_or_else(|| "Could not create a macOS Tab key-up event".to_string())?;

        CGEvent::post(CGEventTapLocation::SessionEventTap, Some(&press));
        thread::sleep(Duration::from_millis(8));
        CGEvent::post(CGEventTapLocation::SessionEventTap, Some(&release));
    }
    Ok(())
}

unsafe extern "C-unwind" fn raw_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    cg_event: NonNull<CGEvent>,
    user_info: *mut c_void,
) -> *mut CGEvent {
    let original = cg_event.as_ptr();
    if user_info.is_null() {
        return original;
    }

    let Some(event) = convert_event(event_type, cg_event) else {
        return original;
    };

    let app = unsafe { &*(user_info as *const AppHandle) };
    let result = catch_unwind(AssertUnwindSafe(|| global_input::handle_event(app, event)));
    match result {
        Ok(Some(_)) => original,
        Ok(None) => {
            unsafe { CGEvent::set_type(Some(cg_event.as_ref()), CGEventType::Null) };
            original
        }
        Err(_) => {
            log::error!("macOS input callback panicked; passing the event through");
            original
        }
    }
}

fn convert_event(event_type: CGEventType, cg_event: NonNull<CGEvent>) -> Option<rdev::Event> {
    use rdev::EventType;

    let event = unsafe { cg_event.as_ref() };
    let event_type = match event_type {
        CGEventType::KeyDown => {
            let code = key_code(event)?;
            EventType::KeyPress(key_from_code(code))
        }
        CGEventType::KeyUp => {
            let code = key_code(event)?;
            EventType::KeyRelease(key_from_code(code))
        }
        CGEventType::FlagsChanged => {
            let code = key_code(event)?;
            let key = key_from_code(code);
            let flags = CGEvent::flags(Some(event));
            if modifier_is_pressed(key, flags)? {
                EventType::KeyPress(key)
            } else {
                EventType::KeyRelease(key)
            }
        }
        CGEventType::ScrollWheel => {
            let delta_y = CGEvent::integer_value_field(
                Some(event),
                CGEventField::ScrollWheelEventDeltaAxis1,
            );
            let delta_x = CGEvent::integer_value_field(
                Some(event),
                CGEventField::ScrollWheelEventDeltaAxis2,
            );
            EventType::Wheel { delta_x, delta_y }
        }
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

fn key_code(event: &CGEvent) -> Option<CGKeyCode> {
    CGEvent::integer_value_field(Some(event), CGEventField::KeyboardEventKeycode)
        .try_into()
        .ok()
}

fn unicode_name(event: &CGEvent) -> Option<String> {
    let mut buffer = [0_u16; 8];
    let mut length = 0usize;
    unsafe {
        CGEvent::keyboard_get_unicode_string(
            Some(event),
            buffer.len(),
            Some(&mut length),
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
        Key::ShiftLeft | Key::ShiftRight => Some(flags.contains(CGEventFlags::MaskShift)),
        Key::ControlLeft | Key::ControlRight => Some(flags.contains(CGEventFlags::MaskControl)),
        Key::Alt | Key::AltGr => Some(flags.contains(CGEventFlags::MaskAlternate)),
        Key::MetaLeft | Key::MetaRight => Some(flags.contains(CGEventFlags::MaskCommand)),
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
