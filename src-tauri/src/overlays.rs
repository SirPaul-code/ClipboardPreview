use std::{
    sync::atomic::Ordering,
    thread,
    time::Duration,
};

use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Position, Size,
};

use crate::{
    models::{
        HistoryOverlayPayload, InteractionMode, OverlayPosition, QuickPreviewPayload,
        IMAGE_PREVIEW_DELAY_MS,
    },
    state::AppState,
};

fn position(
    app: &AppHandle,
    label: &str,
    width: u32,
    height: u32,
    position: &OverlayPosition,
) -> Result<(), String> {
    let window = app
        .get_webview_window(label)
        .ok_or_else(|| format!("Window {label} missing"))?;
    let cursor = app.cursor_position().map_err(|error| error.to_string())?;
    let monitor = app
        .monitor_from_point(cursor.x, cursor.y)
        .map_err(|error| error.to_string())?
        .or_else(|| app.primary_monitor().ok().flatten())
        .ok_or("No monitor available")?;

    let scale = monitor.scale_factor();
    let area = monitor.work_area();
    let area_x = area.position.x as f64 / scale;
    let area_y = area.position.y as f64 / scale;
    let area_width = area.size.width as f64 / scale;
    let area_height = area.size.height as f64 / scale;

    let (x, y) = match position {
        OverlayPosition::Cursor => (cursor.x / scale + 18.0, cursor.y / scale + 20.0),
        OverlayPosition::ScreenCenter => (
            area_x + (area_width - width as f64) / 2.0,
            area_y + (area_height - height as f64) / 2.0,
        ),
        OverlayPosition::TopCenter => (
            area_x + (area_width - width as f64) / 2.0,
            area_y + 48.0,
        ),
        OverlayPosition::BottomCenter => (
            area_x + (area_width - width as f64) / 2.0,
            area_y + area_height - height as f64 - 48.0,
        ),
    };

    let x = x.clamp(
        area_x + 8.0,
        (area_x + area_width - width as f64 - 8.0).max(area_x + 8.0),
    );
    let y = y.clamp(
        area_y + 8.0,
        (area_y + area_height - height as f64 - 8.0).max(area_y + 8.0),
    );

    window
        .set_size(Size::Logical(LogicalSize::new(width as f64, height as f64)))
        .map_err(|error| error.to_string())?;
    window
        .set_position(Position::Logical(LogicalPosition::new(x, y)))
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn show_quick(app: &AppHandle) -> Result<(), String> {
    let state = app
        .try_state::<AppState>()
        .ok_or("Application state is not ready")?;
    let settings = state.settings.read().clone();
    let item = state
        .history
        .lock()
        .get(0)
        .map(|entry| entry.frontend_item());
    let is_image = item.as_ref().is_some_and(|item| {
        matches!(
            &item.content_type,
            crate::models::ClipboardContentType::Image
        )
    });
    let payload = QuickPreviewPayload {
        item,
        settings: settings.preview.clone(),
        appearance: settings.appearance.clone(),
    };
    let height = if is_image {
        220
    } else {
        ((settings.preview.max_lines.min(6) as u32 * settings.preview.font_size * 3) / 2 + 52)
            .clamp(88, 240)
    };

    position(
        app,
        "quick-preview",
        settings.preview.width,
        height,
        &settings.preview.position,
    )?;
    let window = app
        .get_webview_window("quick-preview")
        .ok_or("Quick preview missing")?;
    window.set_focusable(false).map_err(|error| error.to_string())?;
    app.emit_to("quick-preview", "clipboard://quick-preview", payload)
        .map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;

    let generation = state.preview_generation.fetch_add(1, Ordering::SeqCst) + 1;
    let delay = settings.preview.auto_hide_ms;
    let handle = app.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(delay));
        let Some(state) = handle.try_state::<AppState>() else {
            return;
        };
        if state.preview_generation.load(Ordering::SeqCst) == generation {
            if let Some(window) = handle.get_webview_window("quick-preview") {
                let _ = window.hide();
            }
        }
    });
    Ok(())
}

fn history_payload(app: &AppHandle) -> HistoryOverlayPayload {
    let state = app.state::<AppState>();
    let settings = state.settings.read().clone();
    let history = state.history.lock();
    let total_items = history.len();
    let absolute = state
        .selector
        .lock()
        .selected_index
        .min(total_items.saturating_sub(1));
    let visible = settings.history.visible_items.max(1);
    let start = absolute
        .saturating_sub(visible.saturating_sub(1) / 2)
        .min(total_items.saturating_sub(visible));
    let end = (start + visible).min(total_items);
    let items = (start..end)
        .filter_map(|index| history.get(index).map(|entry| entry.frontend_item()))
        .collect();

    HistoryOverlayPayload {
        items,
        selected_index: absolute.saturating_sub(start),
        interaction_mode: settings.history.interaction_mode,
        appearance: settings.appearance,
        total_items,
        shortcut: settings.shortcuts.history_selector,
        image_preview_delay_ms: IMAGE_PREVIEW_DELAY_MS,
        large_preview_panel: settings.history.large_preview_panel,
    }
}

pub fn show_history(app: &AppHandle, focus: bool) -> Result<(), String> {
    let settings = app.state::<AppState>().settings.read().clone();

    // The CSS uses a 42 px header, a 30 px footer, six pixels of list padding,
    // and exact 48 px rows. Matching that geometry here keeps the WebView
    // document from becoming scrollable and prevents partial/extra rows.
    const SWITCHER_CHROME_HEIGHT: u32 = 78;
    const SWITCHER_ROW_HEIGHT: u32 = 48;
    let height = (SWITCHER_CHROME_HEIGHT
        + settings.history.visible_items as u32 * SWITCHER_ROW_HEIGHT)
        .clamp(222, 654);
    let width = if settings.history.large_preview_panel {
        760
    } else {
        500
    };

    position(app, "history-overlay", width, height, &settings.preview.position)?;
    let window = app
        .get_webview_window("history-overlay")
        .ok_or("History overlay missing")?;
    window.set_focusable(focus).map_err(|error| error.to_string())?;
    app.emit_to(
        "history-overlay",
        "clipboard://history-show",
        history_payload(app),
    )
    .map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;
    if focus {
        window.set_focus().map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn emit_selection(app: &AppHandle) -> Result<(), String> {
    app.emit_to(
        "history-overlay",
        "clipboard://history-selection",
        history_payload(app),
    )
    .map_err(|error| error.to_string())
}

pub fn begin(app: &AppHandle, mode: InteractionMode) -> Result<(), String> {
    {
        let state = app.state::<AppState>();
        let mut selector = state.selector.lock();
        selector.active = true;
        selector.selected_index = 0;
    }
    show_history(app, matches!(mode, InteractionMode::Sticky))
}

pub fn hide_history(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("history-overlay") {
        let _ = window.hide();
        let _ = window.set_focusable(false);
    }
}

pub fn open_settings(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("settings")
        .ok_or("Settings window missing")?;
    window.show().map_err(|error| error.to_string())?;
    let _ = window.unminimize();
    window.set_focus().map_err(|error| error.to_string())
}
