export type ClipboardContentType = 'text' | 'url' | 'code' | 'multiline' | 'image';

export interface ClipboardItem {
  id: string;
  type: ClipboardContentType;
  content: string | null;
  preview: string;
  createdAt: string;
  hash: string;
  metadata: {
    characterCount: number;
    lineCount: number;
    width: number | null;
    height: number | null;
    byteSize: number;
  };
  thumbnailDataUrl: string | null;
}

export type InteractionMode = 'hold_release' | 'sticky';

export interface SwitcherTextStyle {
  fontSize: number;
  color: string;
}

export interface SwitcherAppearance {
  headerTitle: SwitcherTextStyle;
  headerSubtitle: SwitcherTextStyle;
  headerMeta: SwitcherTextStyle;
  itemType: SwitcherTextStyle;
  itemContent: SwitcherTextStyle;
  itemMeta: SwitcherTextStyle;
  detailContent: SwitcherTextStyle;
  detailMeta: SwitcherTextStyle;
  footer: SwitcherTextStyle;
  panelBackground: string;
  rowBackground: string;
  selectedBackground: string;
  borderColor: string;
  selectedBorderColor: string;
  rowHeight: number;
  thumbnailSize: number;
}

export interface AppSettings {
  configVersion: number;
  firstRunCompleted: boolean;
  general: {
    launchAtStartup: boolean;
    startHidden: boolean;
    showTrayIcon: boolean;
    monitoringPaused: boolean;
  };
  shortcuts: {
    quickPreview: string;
    historySelector: string;
    openSettings: string;
    pauseMonitoring: string;
  };
  preview: {
    maxCharacters: number;
    maxLines: number;
    width: number;
    fontSize: number;
    textWrapping: boolean;
    position: 'cursor' | 'screen_center' | 'top_center' | 'bottom_center';
    animation: boolean;
    animationSpeed: 'fast' | 'normal' | 'slow';
    autoHideMs: number;
    showContentType: boolean;
    showCharacterCount: boolean;
  };
  history: {
    maxItems: number;
    visibleItems: number;
    scrollDirection: 'natural' | 'reversed';
    wrapSelection: boolean;
    moveSelectedToTop: boolean;
    persistHistory: boolean;
    clearOnExit: boolean;
    interactionMode: InteractionMode;
    largePreviewPanel: boolean;
  };
  appearance: {
    theme: 'system' | 'light' | 'dark';
    overlayOpacity: number;
    cornerRadius: number;
    compactSpacing: boolean;
    fontSize: number;
    reducedMotion: boolean;
    switcher: SwitcherAppearance;
  };
  advanced: { debugLogging: boolean; clipboardPollIntervalMs: number };
}

export interface QuickPreviewPayload {
  item: ClipboardItem | null;
  settings: AppSettings['preview'];
  appearance: AppSettings['appearance'];
}

export interface HistoryPayload {
  items: ClipboardItem[];
  selectedIndex: number;
  interactionMode: InteractionMode;
  appearance: AppSettings['appearance'];
  totalItems: number;
  shortcut: string;
  imagePreviewDelayMs: number;
  largePreviewPanel: boolean;
}

export interface ImagePreviewPayload {
  id: string;
  dataUrl: string;
  width: number;
  height: number;
}

export interface PlatformStatus {
  os: string;
  accessibilityRequired: boolean;
  accessibilityGranted: boolean;
  holdReleaseAvailable: boolean;
  globalWheelAvailable: boolean;
  tabHoldAvailable: boolean;
  imageHistoryAvailable: boolean;
  historyMemoryBudgetMib: number;
  historyPerformanceWarningItems: number;
  lastCrashAvailable: boolean;
  version: string;
  officialBuild: boolean;
  updatesEnabled: boolean;
  startupWarnings: string[];
}

export interface UpdateStatus {
  enabled: boolean;
  available: boolean;
  currentVersion: string;
  version: string | null;
  body: string | null;
  date: string | null;
}

export interface UpdateProgress {
  downloaded: number;
  total: number | null;
}
