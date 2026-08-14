use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize};

use parking_lot::{Mutex, RwLock};

use crate::{history::ClipboardHistory, models::AppSettings};

#[derive(Debug, Default)]
pub struct SelectorState {
    pub active: bool,
    pub selected_index: usize,
}

pub struct AppState {
    pub settings: RwLock<AppSettings>,
    pub history: Mutex<ClipboardHistory>,
    pub selector: Mutex<SelectorState>,
    pub internal_write_hash: Mutex<Option<String>>,
    pub preview_generation: AtomicU64,
    pub history_save_generation: AtomicU64,
    pub startup_warnings: RwLock<Vec<String>>,
    pub tab_down: AtomicBool,
    pub tab_hold_triggered: AtomicBool,
    pub replay_tab_events: AtomicUsize,
    pub native_input_ready: AtomicBool,
}

impl AppState {
    pub fn new(settings: AppSettings, history: ClipboardHistory) -> Self {
        Self {
            settings: RwLock::new(settings),
            history: Mutex::new(history),
            selector: Mutex::new(SelectorState::default()),
            internal_write_hash: Mutex::new(None),
            preview_generation: AtomicU64::new(0),
            history_save_generation: AtomicU64::new(0),
            startup_warnings: RwLock::new(Vec::new()),
            tab_down: AtomicBool::new(false),
            tab_hold_triggered: AtomicBool::new(false),
            replay_tab_events: AtomicUsize::new(0),
            native_input_ready: AtomicBool::new(false),
        }
    }
}
