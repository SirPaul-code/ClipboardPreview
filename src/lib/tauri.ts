import { invoke } from '@tauri-apps/api/core';
import type {
  AppSettings,
  ClipboardItem,
  ImagePreviewPayload,
  PlatformStatus,
  UpdateStatus
} from '../types';

export const backend = {
  settings: () => invoke<AppSettings>('get_settings'),
  history: () => invoke<ClipboardItem[]>('get_history'),
  status: () => invoke<PlatformStatus>('platform_status'),
  imagePreview: (id: string) => invoke<ImagePreviewPayload | null>('get_image_preview', { id }),
  diagnostics: () => invoke<string>('diagnostics_report'),
  clearDiagnostics: () => invoke<void>('clear_diagnostics'),
  openDiagnosticsFolder: () => invoke<void>('open_diagnostics_folder'),
  openMacAccessibilitySettings: () => invoke<void>('open_macos_accessibility_settings'),
  openMacInputMonitoringSettings: () => invoke<void>('open_macos_input_monitoring_settings'),
  openExternal: (url: string) => invoke<void>('open_external', { url }),
  checkForUpdates: () => invoke<UpdateStatus>('check_for_updates'),
  installUpdate: () => invoke<void>('install_update'),
  saveSettings: (settings: AppSettings) => invoke<AppSettings>('save_settings', { settings }),
  clearHistory: () => invoke<void>('clear_history'),
  selectItem: (id: string) => invoke<void>('select_history_item', { id }),
  navigate: (delta: number) => invoke<void>('navigate_selection', { delta }),
  accept: () => invoke<void>('accept_selection'),
  cancel: () => invoke<void>('cancel_selection'),
  showHistory: () => invoke<void>('show_history'),
  openSettings: () => invoke<void>('open_settings'),
  reset: () => invoke<AppSettings>('reset_settings'),
  completeFirstRun: () => invoke<AppSettings>('complete_first_run'),
  quit: () => invoke<void>('quit_app')
};
