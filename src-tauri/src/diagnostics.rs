use std::{
    backtrace::Backtrace,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

#[cfg(any(target_os = "windows", target_os = "macos"))]
const IDENTIFIER: &str = "io.github.sirpaulcode.clipboardpreview";
const MAX_BOOTSTRAP_LOG_BYTES: u64 = 512 * 1024;

pub fn diagnostics_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        return PathBuf::from(local_app_data)
            .join(IDENTIFIER)
            .join("diagnostics");
    }

    #[cfg(target_os = "macos")]
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join("Library")
            .join("Logs")
            .join(IDENTIFIER);
    }

    std::env::temp_dir().join("ClipboardPreview-diagnostics")
}

fn bootstrap_path() -> PathBuf {
    diagnostics_dir().join("bootstrap.log")
}

fn last_crash_path() -> PathBuf {
    diagnostics_dir().join("last-crash.log")
}

fn ensure_dir() {
    let _ = fs::create_dir_all(diagnostics_dir());
}

fn append(path: PathBuf, message: &str) {
    ensure_dir();
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let _ = writeln!(
            file,
            "{timestamp} pid={} thread={} | {message}",
            std::process::id(),
            std::thread::current().name().unwrap_or("<unnamed>")
        );
        let _ = file.flush();
        let _ = file.sync_data();
    }
}

fn rotate_bootstrap_if_needed() {
    let path = bootstrap_path();
    let should_rotate = fs::metadata(&path)
        .map(|metadata| metadata.len() > MAX_BOOTSTRAP_LOG_BYTES)
        .unwrap_or(false);
    if should_rotate {
        let _ = fs::rename(&path, diagnostics_dir().join("bootstrap.previous.log"));
    }
}

pub fn mark(message: &str) {
    append(bootstrap_path(), message);
}

pub fn install_panic_hook() {
    ensure_dir();
    rotate_bootstrap_if_needed();
    let previous = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|location| format!("{}:{}:{}", location.file(), location.line(), location.column()))
            .unwrap_or_else(|| "<unknown>".to_string());
        let payload = if let Some(value) = info.payload().downcast_ref::<&str>() {
            (*value).to_string()
        } else if let Some(value) = info.payload().downcast_ref::<String>() {
            value.clone()
        } else {
            "<non-string panic payload>".to_string()
        };
        let report = format!(
            "Clipboard Preview panic\nLocation: {location}\nMessage: {payload}\n\nBacktrace:\n{}",
            Backtrace::force_capture()
        );

        let _ = fs::write(last_crash_path(), &report);
        mark(&format!("PANIC: {location}: {payload}"));
        previous(info);
    }));

    mark("panic hook installed");
}

pub fn last_crash_available() -> bool {
    fs::metadata(last_crash_path())
        .map(|metadata| metadata.len() > 0)
        .unwrap_or(false)
}

pub fn clear_last_crash() -> Result<(), String> {
    let path = last_crash_path();
    if path.exists() {
        fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn report(version: &str, os: &str) -> String {
    let mut report = format!(
        "Clipboard Preview diagnostics\n=============================\nVersion: {version}\nOS: {os}\nGenerated: {}\nDiagnostics directory: {}\n\n",
        chrono::Utc::now().to_rfc3339(),
        diagnostics_dir().display()
    );

    report.push_str("Last crash\n----------\n");
    match fs::read_to_string(last_crash_path()) {
        Ok(contents) if !contents.trim().is_empty() => report.push_str(&contents),
        _ => report.push_str("No recorded application panic.\n"),
    }

    report.push_str("\n\nBootstrap log\n-------------\n");
    match fs::read_to_string(bootstrap_path()) {
        Ok(contents) if !contents.trim().is_empty() => {
            let tail: Vec<&str> = contents.lines().rev().take(250).collect();
            for line in tail.into_iter().rev() {
                report.push_str(line);
                report.push('\n');
            }
        }
        _ => report.push_str("No bootstrap log is available.\n"),
    }

    report.push_str(
        "\nPrivacy note: Clipboard contents are not intentionally written to diagnostic logs.\n",
    );
    report
}

pub fn mark_clean_shutdown() {
    mark("clean shutdown");
}
