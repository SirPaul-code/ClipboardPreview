use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const CONFIG_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClipboardContentType {
    Text,
    Url,
    Code,
    Multiline,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardMetadata {
    pub character_count: usize,
    pub line_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardItem {
    pub id: String,
    #[serde(rename = "type")]
    pub content_type: ClipboardContentType,
    pub content: String,
    pub preview: String,
    pub created_at: DateTime<Utc>,
    pub hash: String,
    pub metadata: ClipboardMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub config_version: u32,
    pub first_run_completed: bool,
    pub general: GeneralSettings,
    pub shortcuts: ShortcutSettings,
    pub preview: PreviewSettings,
    pub history: HistorySettings,
    pub appearance: AppearanceSettings,
    pub advanced: AdvancedSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GeneralSettings {
    pub launch_at_startup: bool,
    pub start_hidden: bool,
    pub show_tray_icon: bool,
    pub monitoring_paused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutSettings {
    pub quick_preview: String,
    pub history_selector: String,
    pub open_settings: String,
    pub pause_monitoring: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PreviewSettings {
    pub max_characters: usize,
    pub max_lines: usize,
    pub width: u32,
    pub font_size: u32,
    pub text_wrapping: bool,
    pub position: OverlayPosition,
    pub animation: bool,
    pub animation_speed: AnimationSpeed,
    pub auto_hide_ms: u64,
    pub show_content_type: bool,
    pub show_character_count: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverlayPosition {
    Cursor,
    ScreenCenter,
    TopCenter,
    BottomCenter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnimationSpeed {
    Fast,
    Normal,
    Slow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HistorySettings {
    pub max_items: usize,
    pub visible_items: usize,
    pub scroll_direction: ScrollDirection,
    pub wrap_selection: bool,
    pub move_selected_to_top: bool,
    pub persist_history: bool,
    pub clear_on_exit: bool,
    pub interaction_mode: InteractionMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScrollDirection {
    Natural,
    Reversed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InteractionMode {
    HoldRelease,
    Sticky,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppearanceSettings {
    pub theme: ThemePreference,
    pub overlay_opacity: f64,
    pub corner_radius: u32,
    pub compact_spacing: bool,
    pub font_size: u32,
    pub reduced_motion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreference {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdvancedSettings {
    pub debug_logging: bool,
    pub clipboard_poll_interval_ms: u64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            config_version: CONFIG_VERSION,
            first_run_completed: false,
            general: GeneralSettings {
                launch_at_startup: false,
                start_hidden: true,
                show_tray_icon: true,
                monitoring_paused: false,
            },
            shortcuts: ShortcutSettings {
                quick_preview: "Ctrl+Alt+K".into(),
                history_selector: "Ctrl+Alt+J".into(),
                open_settings: "Ctrl+Alt+P".into(),
                pause_monitoring: "Ctrl+Alt+Shift+P".into(),
            },
            preview: PreviewSettings {
                max_characters: 120,
                max_lines: 4,
                width: 460,
                font_size: 14,
                text_wrapping: true,
                position: OverlayPosition::Cursor,
                animation: true,
                animation_speed: AnimationSpeed::Fast,
                auto_hide_ms: 1800,
                show_content_type: true,
                show_character_count: false,
            },
            history: HistorySettings {
                max_items: 20,
                visible_items: 5,
                scroll_direction: ScrollDirection::Natural,
                wrap_selection: false,
                move_selected_to_top: true,
                persist_history: false,
                clear_on_exit: false,
                interaction_mode: InteractionMode::HoldRelease,
            },
            appearance: AppearanceSettings {
                theme: ThemePreference::System,
                overlay_opacity: 0.96,
                corner_radius: 13,
                compact_spacing: false,
                font_size: 14,
                reduced_motion: false,
            },
            advanced: AdvancedSettings {
                debug_logging: false,
                clipboard_poll_interval_ms: 350,
            },
        }
    }
}

impl AppSettings {
    pub fn normalized(mut self) -> Self {
        self.config_version = CONFIG_VERSION;
        self.preview.max_characters = self.preview.max_characters.clamp(20, 2000);
        self.preview.max_lines = self.preview.max_lines.clamp(1, 20);
        self.preview.width = self.preview.width.clamp(280, 760);
        self.preview.font_size = self.preview.font_size.clamp(11, 22);
        self.preview.auto_hide_ms = self.preview.auto_hide_ms.clamp(500, 8000);
        self.history.max_items = self.history.max_items.clamp(1, 100);
        self.history.visible_items = self.history.visible_items.clamp(3, 12);
        self.appearance.overlay_opacity = self.appearance.overlay_opacity.clamp(0.72, 1.0);
        self.appearance.corner_radius = self.appearance.corner_radius.clamp(6, 24);
        self.appearance.font_size = self.appearance.font_size.clamp(11, 20);
        self.advanced.clipboard_poll_interval_ms =
            self.advanced.clipboard_poll_interval_ms.clamp(150, 1500);
        self
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickPreviewPayload {
    pub item: Option<ClipboardItem>,
    pub settings: PreviewSettings,
    pub appearance: AppearanceSettings,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryOverlayPayload {
    pub items: Vec<ClipboardItem>,
    pub selected_index: usize,
    pub interaction_mode: InteractionMode,
    pub appearance: AppearanceSettings,
    pub total_items: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformStatus {
    pub os: String,
    pub accessibility_required: bool,
    pub accessibility_granted: bool,
    pub hold_release_available: bool,
    pub global_wheel_available: bool,
    pub version: String,
    pub startup_warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_round_trip() {
        let x = AppSettings::default();
        let y: AppSettings = serde_json::from_str(&serde_json::to_string(&x).unwrap()).unwrap();
        assert_eq!(x, y);
    }

    #[test]
    fn normalization_clamps() {
        let mut x = AppSettings::default();
        x.history.max_items = 999;
        x.preview.max_characters = 1;
        x.advanced.clipboard_poll_interval_ms = 10;
        let x = x.normalized();
        assert_eq!(x.history.max_items, 100);
        assert_eq!(x.preview.max_characters, 20);
        assert_eq!(x.advanced.clipboard_poll_interval_ms, 150);
    }
}
