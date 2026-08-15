use std::sync::atomic::AtomicU64;

#[cfg(any(target_os = "windows", target_os = "macos"))]
use std::sync::atomic::AtomicBool;

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
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub tab_down: AtomicBool,
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub tab_hold_triggered: AtomicBool,
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub replaying_tab: AtomicBool,
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub native_input_ready: AtomicBool,
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub alt_down: AtomicBool,
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub control_down: AtomicBool,
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub shift_down: AtomicBool,
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub meta_down: AtomicBool,
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
            #[cfg(any(target_os = "windows", target_os = "macos"))]
            tab_down: AtomicBool::new(false),
            #[cfg(any(target_os = "windows", target_os = "macos"))]
            tab_hold_triggered: AtomicBool::new(false),
            #[cfg(any(target_os = "windows", target_os = "macos"))]
            replaying_tab: AtomicBool::new(false),
            #[cfg(any(target_os = "windows", target_os = "macos"))]
            native_input_ready: AtomicBool::new(false),
            #[cfg(any(target_os = "windows", target_os = "macos"))]
            alt_down: AtomicBool::new(false),
            #[cfg(any(target_os = "windows", target_os = "macos"))]
            control_down: AtomicBool::new(false),
            #[cfg(any(target_os = "windows", target_os = "macos"))]
            shift_down: AtomicBool::new(false),
            #[cfg(any(target_os = "windows", target_os = "macos"))]
            meta_down: AtomicBool::new(false),
        }
    }
}
