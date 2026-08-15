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
fn accessibility_trusted() -> bool {
    unsafe { AXIsProcessTrusted() != 0 }
}

#[cfg(target_os = "macos")]
pub fn input_monitoring_granted() -> bool {
    unsafe { CGPreflightListenEventAccess() }
}

#[cfg(target_os = "macos")]
pub fn native_input_permissions_granted() -> bool {
    accessibility_trusted() && input_monitoring_granted()
}

// PlatformStatus historically exposes this as `accessibilityGranted`. Keep that
// field backward compatible, but on macOS make it mean "native switcher input is
// authorized" because Tab hold requires both TCC services.
#[cfg(target_os = "macos")]
pub fn accessibility_granted() -> bool {
    native_input_permissions_granted()
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
            thread::sleep(Duration::from_millis(750));
        }
    }

    if !input_monitoring_granted() {
        // CGEventTap has its own TCC service. Asking through Core Graphics first
        // gives macOS a chance to show the native Input Monitoring consent flow.
        // The settings pane is also opened because ad-hoc-signed community builds
        // can be treated as a new TCC identity after an application update.
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
