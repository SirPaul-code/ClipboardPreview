use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const CONFIG_VERSION: u32 = 5;
pub const HISTORY_MAX_ITEMS: usize = 250;
pub const HISTORY_PERF_WARNING_ITEMS: usize = 150;
pub const HISTORY_MEMORY_BUDGET_MIB: usize = 192;
pub const TAB_HOLD_DELAY_MS: u64 = 180;
pub const IMAGE_PREVIEW_DELAY_MS: u64 = 650;

fn default_large_preview_panel() -> bool {
    true
}

fn default_previous_item() -> String {
    "ArrowUp".into()
}

fn default_next_item() -> String {
    "ArrowDown".into()
}

fn valid_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value[1..].chars().all(|value| value.is_ascii_hexdigit())
}

fn normalize_color(value: &mut String, fallback: &str) {
    if !valid_hex_color(value) {
        *value = fallback.to_string();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClipboardContentType {
    Text,
    Url,
    Code,
    Multiline,
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardMetadata {
    pub character_count: usize,
    pub line_count: usize,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub byte_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardItem {
    pub id: String,
    #[serde(rename = "type")]
    pub content_type: ClipboardContentType,
    #[serde(default)]
    pub content: Option<String>,
    pub preview: String,
    pub created_at: DateTime<Utc>,
    pub hash: String,
    pub metadata: ClipboardMetadata,
    #[serde(default)]
    pub thumbnail_data_url: Option<String>,
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
    #[serde(default = "default_previous_item")]
    pub previous_item: String,
    #[serde(default = "default_next_item")]
    pub next_item: String,
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
    #[serde(default = "default_large_preview_panel")]
    pub large_preview_panel: bool,
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
pub struct SwitcherTextStyle {
    pub font_size: u32,
    pub color: String,
}

impl SwitcherTextStyle {
    fn normalized(mut self, fallback_color: &str) -> Self {
        self.font_size = self.font_size.clamp(8, 28);
        normalize_color(&mut self.color, fallback_color);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SwitcherAppearanceSettings {
    pub header_title: SwitcherTextStyle,
    pub header_subtitle: SwitcherTextStyle,
    pub header_meta: SwitcherTextStyle,
    pub item_type: SwitcherTextStyle,
    pub item_content: SwitcherTextStyle,
    pub item_meta: SwitcherTextStyle,
    pub detail_content: SwitcherTextStyle,
    pub detail_meta: SwitcherTextStyle,
    pub footer: SwitcherTextStyle,
    pub panel_background: String,
    pub row_background: String,
    pub selected_background: String,
    pub border_color: String,
    pub selected_border_color: String,
    pub row_height: u32,
    pub thumbnail_size: u32,
}

impl Default for SwitcherAppearanceSettings {
    fn default() -> Self {
        Self {
            header_title: SwitcherTextStyle { font_size: 13, color: "#F3F4F1".into() },
            header_subtitle: SwitcherTextStyle { font_size: 10, color: "#8D938C".into() },
            header_meta: SwitcherTextStyle { font_size: 10, color: "#A6ACA5".into() },
            item_type: SwitcherTextStyle { font_size: 9, color: "#9DA39C".into() },
            item_content: SwitcherTextStyle { font_size: 12, color: "#F0F1EE".into() },
            item_meta: SwitcherTextStyle { font_size: 9, color: "#858B84".into() },
            detail_content: SwitcherTextStyle { font_size: 13, color: "#F0F1EE".into() },
            detail_meta: SwitcherTextStyle { font_size: 10, color: "#929890".into() },
            footer: SwitcherTextStyle { font_size: 9, color: "#858B84".into() },
            panel_background: "#151716".into(),
            row_background: "#151716".into(),
            selected_background: "#282C28".into(),
            border_color: "#343834".into(),
            selected_border_color: "#D5DAD2".into(),
            row_height: 48,
            thumbnail_size: 38,
        }
    }
}

impl SwitcherAppearanceSettings {
    fn normalized(mut self) -> Self {
        let defaults = Self::default();
        self.header_title = self.header_title.normalized(&defaults.header_title.color);
        self.header_subtitle = self.header_subtitle.normalized(&defaults.header_subtitle.color);
        self.header_meta = self.header_meta.normalized(&defaults.header_meta.color);
        self.item_type = self.item_type.normalized(&defaults.item_type.color);
        self.item_content = self.item_content.normalized(&defaults.item_content.color);
        self.item_meta = self.item_meta.normalized(&defaults.item_meta.color);
        self.detail_content = self.detail_content.normalized(&defaults.detail_content.color);
        self.detail_meta = self.detail_meta.normalized(&defaults.detail_meta.color);
        self.footer = self.footer.normalized(&defaults.footer.color);
        normalize_color(&mut self.panel_background, &defaults.panel_background);
        normalize_color(&mut self.row_background, &defaults.row_background);
        normalize_color(&mut self.selected_background, &defaults.selected_background);
        normalize_color(&mut self.border_color, &defaults.border_color);
        normalize_color(&mut self.selected_border_color, &defaults.selected_border_color);
        self.row_height = self.row_height.clamp(38, 88);
        self.thumbnail_size = self.thumbnail_size.clamp(26, 72).min(self.row_height.saturating_sub(6));
        self
    }
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
    #[serde(default)]
    pub switcher: SwitcherAppearanceSettings,
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
        let linux = cfg!(target_os = "linux");
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
                history_selector: if linux { "Ctrl+Alt+J".into() } else { "Tab".into() },
                previous_item: default_previous_item(),
                next_item: default_next_item(),
                open_settings: "Ctrl+Alt+P".into(),
                pause_monitoring: "Ctrl+Alt+Shift+P".into(),
            },
            preview: PreviewSettings {
                max_characters: 160,
                max_lines: 6,
                width: 480,
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
                max_items: 40,
                visible_items: 6,
                scroll_direction: ScrollDirection::Natural,
                wrap_selection: false,
                move_selected_to_top: true,
                persist_history: false,
                clear_on_exit: false,
                interaction_mode: if linux { InteractionMode::Sticky } else { InteractionMode::HoldRelease },
                large_preview_panel: true,
            },
            appearance: AppearanceSettings {
                theme: ThemePreference::System,
                overlay_opacity: 0.97,
                corner_radius: 16,
                compact_spacing: false,
                font_size: 14,
                reduced_motion: false,
                switcher: SwitcherAppearanceSettings::default(),
            },
            advanced: AdvancedSettings {
                debug_logging: false,
                clipboard_poll_interval_ms: 350,
            },
        }
    }
}

impl AppSettings {
    pub fn migrate(mut self) -> Self {
        if self.config_version < 2 {
            if self.shortcuts.history_selector.eq_ignore_ascii_case("Ctrl+Alt+J") && !cfg!(target_os = "linux") {
                self.shortcuts.history_selector = "Tab".into();
            }
            if self.history.max_items == 20 {
                self.history.max_items = 40;
            }
            if self.history.visible_items == 5 {
                self.history.visible_items = 6;
            }
        }
        if self.config_version < 4
            && cfg!(target_os = "linux")
            && self.shortcuts.history_selector.eq_ignore_ascii_case("Tab")
        {
            self.shortcuts.history_selector = "Ctrl+Alt+J".into();
            self.history.interaction_mode = InteractionMode::Sticky;
        }
        self.normalized()
    }

    pub fn normalized(mut self) -> Self {
        self.config_version = CONFIG_VERSION;
        self.preview.max_characters = self.preview.max_characters.clamp(20, 2000);
        self.preview.max_lines = self.preview.max_lines.clamp(1, 20);
        self.preview.width = self.preview.width.clamp(280, 760);
        self.preview.font_size = self.preview.font_size.clamp(11, 22);
        self.preview.auto_hide_ms = self.preview.auto_hide_ms.clamp(500, 8000);
        self.history.max_items = self.history.max_items.clamp(1, HISTORY_MAX_ITEMS);
        self.history.visible_items = self.history.visible_items.clamp(3, 12);
        self.appearance.overlay_opacity = self.appearance.overlay_opacity.clamp(0.72, 1.0);
        self.appearance.corner_radius = self.appearance.corner_radius.clamp(6, 24);
        self.appearance.font_size = self.appearance.font_size.clamp(11, 20);
        self.appearance.switcher = self.appearance.switcher.normalized();
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
    pub shortcut: String,
    pub previous_shortcut: String,
    pub next_shortcut: String,
    pub image_preview_delay_ms: u64,
    pub large_preview_panel: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePreviewPayload {
    pub id: String,
    pub data_url: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformStatus {
    pub os: String,
    pub accessibility_required: bool,
    pub accessibility_granted: bool,
    pub hold_release_available: bool,
    pub global_wheel_available: bool,
    pub tab_hold_available: bool,
    pub image_history_available: bool,
    pub history_memory_budget_mib: usize,
    pub history_performance_warning_items: usize,
    pub last_crash_available: bool,
    pub version: String,
    pub official_build: bool,
    pub updates_enabled: bool,
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
    fn defaults_use_platform_switcher() {
        let x = AppSettings::default();
        if cfg!(target_os = "linux") {
            assert_eq!(x.shortcuts.history_selector, "Ctrl+Alt+J");
            assert!(matches!(x.history.interaction_mode, InteractionMode::Sticky));
        } else {
            assert_eq!(x.shortcuts.history_selector, "Tab");
            assert!(matches!(x.history.interaction_mode, InteractionMode::HoldRelease));
        }
        assert_eq!(x.shortcuts.previous_item, "ArrowUp");
        assert_eq!(x.shortcuts.next_item, "ArrowDown");
        assert_eq!(x.history.visible_items, 6);
        assert!(x.history.large_preview_panel);
    }

    #[test]
    fn migrates_v1_defaults() {
        let mut x = AppSettings {
            config_version: 1,
            ..AppSettings::default()
        };
        x.shortcuts.history_selector = "Ctrl+Alt+J".into();
        x.history.max_items = 20;
        x.history.visible_items = 5;
        let x = x.migrate();
        assert_eq!(x.config_version, CONFIG_VERSION);
        if cfg!(target_os = "linux") {
            assert_eq!(x.shortcuts.history_selector, "Ctrl+Alt+J");
        } else {
            assert_eq!(x.shortcuts.history_selector, "Tab");
        }
        assert_eq!(x.history.max_items, 40);
        assert_eq!(x.history.visible_items, 6);
    }

    #[test]
    fn deserializes_old_settings_without_navigation_or_appearance() {
        let mut value = serde_json::to_value(AppSettings::default()).unwrap();
        value["configVersion"] = serde_json::json!(3);
        value["appearance"].as_object_mut().unwrap().remove("switcher");
        value["shortcuts"].as_object_mut().unwrap().remove("previousItem");
        value["shortcuts"].as_object_mut().unwrap().remove("nextItem");
        let x: AppSettings = serde_json::from_value(value).unwrap();
        assert_eq!(x.appearance.switcher, SwitcherAppearanceSettings::default());
        assert_eq!(x.shortcuts.previous_item, "ArrowUp");
        assert_eq!(x.shortcuts.next_item, "ArrowDown");
    }

    #[test]
    fn normalization_clamps() {
        let mut x = AppSettings::default();
        x.history.max_items = 999;
        x.preview.max_characters = 1;
        x.advanced.clipboard_poll_interval_ms = 10;
        x.appearance.switcher.row_height = 200;
        x.appearance.switcher.item_content.font_size = 80;
        x.appearance.switcher.item_content.color = "not-a-color".into();
        let x = x.normalized();
        assert_eq!(x.history.max_items, HISTORY_MAX_ITEMS);
        assert_eq!(x.preview.max_characters, 20);
        assert_eq!(x.advanced.clipboard_poll_interval_ms, 150);
        assert_eq!(x.appearance.switcher.row_height, 88);
        assert_eq!(x.appearance.switcher.item_content.font_size, 28);
        assert_eq!(x.appearance.switcher.item_content.color, "#F0F1EE");
    }
}
