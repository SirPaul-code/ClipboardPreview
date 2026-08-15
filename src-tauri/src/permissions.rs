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
    fn CGRequestListenEventAccess() -> bool;
}

#[cfg(target_os = "macos")]
fn accessibility_only_granted() -> bool {
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
pub fn accessibility_granted() -> bool {
    // Clipboard Preview's native switcher capture is an active CGEventTap. On
    // current macOS releases it needs both Accessibility (to filter/replay input)
    // and Input Monitoring (to receive global keyboard events reliably).
    accessibility_only_granted() && input_monitoring_granted()
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
    if !accessibility_only_granted() {
        open_accessibility_settings()?;
        while !accessibility_only_granted() {
            thread::sleep(Duration::from_millis(750));
        }
    }

    if !input_monitoring_granted() {
        // This is the Core Graphics API intended to trigger the Input Monitoring
        // consent flow. The settings pane is also opened because ad-hoc-signed
        // community builds can lose TCC identity across application updates.
        unsafe {
            let _ = CGRequestListenEventAccess();
        }
        if !input_monitoring_granted() {
            open_input_monitoring_settings()?;
            while !input_monitoring_granted() {
                thread::sleep(Duration::from_millis(750));
            }
        }
    }

    Ok(())
}
