#[cfg(target_os = "macos")]
use std::{ffi::c_void, process::Command, ptr, thread, time::Duration};

#[cfg(target_os = "macos")]
type CFDictionaryRef = *const c_void;

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> std::ffi::c_uchar;
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> std::ffi::c_uchar;
    static kAXTrustedCheckOptionPrompt: *const c_void;
}

#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFDictionaryCreate(
        allocator: *const c_void,
        keys: *const *const c_void,
        values: *const *const c_void,
        num_values: isize,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> CFDictionaryRef;
    fn CFRelease(value: *const c_void);
    static kCFBooleanTrue: *const c_void;
}

#[cfg(target_os = "macos")]
fn accessibility_trusted() -> bool {
    unsafe { AXIsProcessTrusted() != 0 }
}

#[cfg(target_os = "macos")]
fn request_accessibility_prompt() -> bool {
    unsafe {
        // AXIsProcessTrustedWithOptions is the documented way to make macOS
        // inform an untrusted user and register the app in Accessibility. The
        // key and value are process-lifetime Core Foundation constants, so the
        // temporary dictionary does not need retain/release callbacks.
        let keys = [kAXTrustedCheckOptionPrompt];
        let values = [kCFBooleanTrue];
        let options = CFDictionaryCreate(
            ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            ptr::null(),
            ptr::null(),
        );
        if options.is_null() {
            return accessibility_trusted();
        }
        let trusted = AXIsProcessTrustedWithOptions(options) != 0;
        CFRelease(options);
        trusted
    }
}

#[cfg(target_os = "macos")]
pub fn native_input_permissions_granted() -> bool {
    accessibility_trusted()
}

#[cfg(target_os = "macos")]
pub fn accessibility_granted() -> bool {
    accessibility_trusted()
}

#[cfg(not(target_os = "macos"))]
pub fn accessibility_granted() -> bool {
    true
}

#[cfg(target_os = "macos")]
pub fn open_accessibility_settings() -> Result<(), String> {
    Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn()
        .map_err(|error| format!("Could not open macOS Accessibility settings: {error}"))?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn wait_for_native_input_permissions() -> Result<(), String> {
    if accessibility_trusted() {
        return Ok(());
    }

    // Prompt first so Clipboard Preview is registered as the requesting app.
    // The prompt is asynchronous, so also open the exact pane as a deterministic
    // fallback and wait until TCC reports the process as trusted.
    let _ = request_accessibility_prompt();
    if !accessibility_trusted() {
        open_accessibility_settings()?;
        while !accessibility_trusted() {
            thread::sleep(Duration::from_millis(750));
        }
    }

    Ok(())
}
