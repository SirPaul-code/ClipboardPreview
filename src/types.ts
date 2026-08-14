export type ClipboardContentType = 'text' | 'url' | 'code' | 'multiline';
export interface ClipboardItem {
  id: string; type: ClipboardContentType; content: string; preview: string; createdAt: string; hash: string;
  metadata: { characterCount: number; lineCount: number };
}
export type InteractionMode = 'hold_release' | 'sticky';
export interface AppSettings {
  configVersion: number; firstRunCompleted: boolean;
  general: { launchAtStartup: boolean; startHidden: boolean; showTrayIcon: boolean; monitoringPaused: boolean };
  shortcuts: { quickPreview: string; historySelector: string; openSettings: string; pauseMonitoring: string };
  preview: { maxCharacters: number; maxLines: number; width: number; fontSize: number; textWrapping: boolean; position: 'cursor'|'screen_center'|'top_center'|'bottom_center'; animation: boolean; animationSpeed: 'fast'|'normal'|'slow'; autoHideMs: number; showContentType: boolean; showCharacterCount: boolean };
  history: { maxItems: number; visibleItems: number; scrollDirection: 'natural'|'reversed'; wrapSelection: boolean; moveSelectedToTop: boolean; persistHistory: boolean; clearOnExit: boolean; interactionMode: InteractionMode };
  appearance: { theme: 'system'|'light'|'dark'; overlayOpacity: number; cornerRadius: number; compactSpacing: boolean; fontSize: number; reducedMotion: boolean };
  advanced: { debugLogging: boolean; clipboardPollIntervalMs: number };
}
export interface QuickPreviewPayload { item: ClipboardItem | null; settings: AppSettings['preview']; appearance: AppSettings['appearance'] }
export interface HistoryPayload { items: ClipboardItem[]; selectedIndex: number; interactionMode: InteractionMode; appearance: AppSettings['appearance']; totalItems: number }
export interface PlatformStatus { os: string; accessibilityRequired: boolean; accessibilityGranted: boolean; holdReleaseAvailable: boolean; globalWheelAvailable: boolean; version: string }
