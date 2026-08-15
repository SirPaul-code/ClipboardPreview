#[cfg(target_os = "macos")]
use std::{process::Command, thread, time::Duration};

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> std::ffi::c_uchar;
}

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightListenEventAccess() -> bool;
}

#[cfg(target_os = "macos")]
fn accessibility_trusted() -> bool {
    unsafe { AXIsProcessTrusted() != 0 }
}

#[cfg(target_os = "macos")]
pub fn input_monitoring_granted() -> bool {
    unsafe { CGPreflightListenEventAccess() }
}

#[cfg(not(target_os = "macos"))]
pub fn input_monitoring_granted() -> bool {
    true
}

#[cfg(target_os = "macos")]
pub fn native_input_permissions_granted() -> bool {
    // Clipboard Preview uses an active kCGSessionEventTap. Apple authorizes a
    // modifying event tap through Accessibility. Input Monitoring is only needed
    // for passive listen-only taps, so it is intentionally not a prerequisite.
    accessibility_trusted()
}

#[cfg(not(target_os = "macos"))]
pub fn native_input_permissions_granted() -> bool {
    true
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
fn open_privacy_settings(pane: &str, label: &str) -> Result<(), String> {
    Command::new("open")
        .arg(format!(
            "x-apple.systempreferences:com.apple.preference.security?{pane}"
        ))
        .spawn()
        .map_err(|error| format!("Could not open macOS {label} settings: {error}"))?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn open_accessibility_settings() -> Result<(), String> {
    open_privacy_settings("Privacy_Accessibility", "Accessibility")
}

#[cfg(target_os = "macos")]
pub fn open_input_monitoring_settings() -> Result<(), String> {
    open_privacy_settings("Privacy_ListenEvent", "Input Monitoring")
}

#[cfg(target_os = "macos")]
pub fn wait_for_native_input_permissions() -> Result<(), String> {
    if !accessibility_trusted() {
        open_accessibility_settings()?;
        while !accessibility_trusted() {
            thread::sleep(Duration::from_millis(500));
        }
    }
    Ok(())
}
