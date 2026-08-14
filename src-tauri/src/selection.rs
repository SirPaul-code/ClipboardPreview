use tauri::{AppHandle, Manager};

use crate::{
    clipboard,
    models::ScrollDirection,
    overlays, settings_store,
    state::AppState,
};

pub fn navigate(app: &AppHandle, mut delta: i32) -> Result<(), String> {
    let Some(state) = app.try_state::<AppState>() else {
        return Ok(());
    };
    let settings = state.settings.read().clone();
    if matches!(settings.history.scroll_direction, ScrollDirection::Reversed) {
        delta = -delta;
    }
    let len = state.history.lock().len();
    if len == 0 {
        return Ok(());
    }

    {
        let mut selector = state.selector.lock();
        if !selector.active {
            return Ok(());
        }
        let current = selector.selected_index as i32;
        let next = current + delta;
        if settings.history.wrap_selection {
            selector.selected_index = next.rem_euclid(len as i32) as usize;
        } else {
            selector.selected_index = next.clamp(0, len as i32 - 1) as usize;
        }
    }

    overlays::emit_selection(app)
}

pub fn accept(app: &AppHandle) -> Result<(), String> {
    let (active, settings, entry) = {
        let Some(state) = app.try_state::<AppState>() else {
            overlays::hide_history(app);
            return Ok(());
        };
        let (active, index) = {
            let mut selector = state.selector.lock();
            let current = (selector.active, selector.selected_index);
            selector.active = false;
            current
        };
        let settings = state.settings.read().clone();
        let entry = state.history.lock().get(index);
        (active, settings, entry)
    };

    if !active {
        overlays::hide_history(app);
        return Ok(());
    }

    let result = if let Some(entry) = entry {
        let write_result = clipboard::write_entry(app, &entry);
        if write_result.is_ok() && settings.history.move_selected_to_top {
            if let Some(state) = app.try_state::<AppState>() {
                state.history.lock().promote(&entry.item.id);
            }
            settings_store::schedule_history_save(app, settings.history.persist_history);
        }
        write_result
    } else {
        Ok(())
    };

    overlays::hide_history(app);
    result
}

pub fn cancel(app: &AppHandle) {
    if let Some(state) = app.try_state::<AppState>() {
        state.selector.lock().active = false;
    }
    overlays::hide_history(app);
}

#[cfg(test)]
mod tests {
    #[test]
    fn clamp_math() {
        let len = 3_i32;
        assert_eq!((1 + 10).clamp(0, len - 1), 2);
        assert_eq!((1 - 10).clamp(0, len - 1), 0);
    }

    #[test]
    fn wrap_math() {
        assert_eq!((-1_i32).rem_euclid(3), 2);
        assert_eq!(3_i32.rem_euclid(3), 0);
    }
}
