import { useCallback, useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import {
  Clipboard,
  Command,
  Copy,
  FolderOpen,
  History,
  Image as ImageIcon,
  Info,
  Palette,
  Settings2,
  Shield,
  SlidersHorizontal,
  Trash2
} from 'lucide-react';
import { backend } from '../lib/tauri';
import type {
  AppSettings,
  ClipboardItem,
  PlatformStatus,
  UpdateProgress,
  UpdateStatus
} from '../types';
import { ClipboardCard } from '../components/ClipboardCard';
import { NumberField, Row, Section, Toggle } from '../components/SettingsControls';
import { ShortcutRecorder } from '../components/ShortcutRecorder';
import { SwitcherAppearanceEditor } from '../components/SwitcherAppearanceEditor';

type Tab = 'general' | 'switcher' | 'history' | 'appearance' | 'advanced' | 'about';
const tabs: Array<[Tab, string, typeof Clipboard]> = [
  ['general', 'General', Settings2],
  ['switcher', 'Switcher', Command],
  ['history', 'History', History],
  ['appearance', 'Appearance', Palette],
  ['advanced', 'Advanced', SlidersHorizontal],
  ['about', 'About', Info]
];

export function SettingsApp() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [history, setHistory] = useState<ClipboardItem[]>([]);
  const [status, setStatus] = useState<PlatformStatus | null>(null);
  const [tab, setTab] = useState<Tab>('general');
  const [message, setMessage] = useState('');
  const [loadError, setLoadError] = useState('');
  const [diagnostics, setDiagnostics] = useState('');
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus | null>(null);
  const [updateChecking, setUpdateChecking] = useState(false);
  const [updateProgress, setUpdateProgress] = useState<UpdateProgress | null>(null);

  const load = useCallback(async () => {
    try {
      const [nextSettings, nextHistory, nextStatus] = await Promise.all([
        backend.settings(),
        backend.history(),
        backend.status()
      ]);
      setSettings(nextSettings);
      setHistory(nextHistory);
      setStatus(nextStatus);
      setLoadError('');
    } catch (error) {
      setLoadError(String(error));
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    const offs: Array<() => void> = [];
    Promise.all([
      listen('clipboard://history-changed', () => void backend.history().then(setHistory)),
      listen('clipboard://status-changed', () => void backend.status().then(setStatus)),
      listen<UpdateStatus>('clipboard://update-available', (event) => setUpdateStatus(event.payload)),
      listen<UpdateProgress>('clipboard://update-progress', (event) => setUpdateProgress(event.payload))
    ]).then((listeners) => offs.push(...listeners));
    return () => offs.forEach((off) => off());
  }, []);

  const save = useCallback(
    async (next: AppSettings) => {
      setSettings(next);
      try {
        setSettings(await backend.saveSettings(next));
        setMessage('Saved');
        window.setTimeout(() => setMessage(''), 900);
      } catch (error) {
        setMessage(String(error));
        void load();
      }
    },
    [load]
  );

  const patch = useCallback(
    <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
      if (settings) void save({ ...settings, [key]: value });
    },
    [settings, save]
  );

  const loadDiagnostics = useCallback(async () => {
    setDiagnostics(await backend.diagnostics());
  }, []);

  const checkUpdates = useCallback(async () => {
    setUpdateChecking(true);
    try {
      const next = await backend.checkForUpdates();
      setUpdateStatus(next);
      setMessage(next.available ? `Update ${next.version} available` : next.enabled ? 'Up to date' : 'Updates disabled for source build');
      window.setTimeout(() => setMessage(''), 1800);
    } catch (error) {
      setMessage(`Update check failed: ${String(error)}`);
    } finally {
      setUpdateChecking(false);
    }
  }, []);

  const installUpdate = useCallback(async () => {
    setUpdateProgress({ downloaded: 0, total: null });
    setMessage('Downloading update…');
    try {
      await backend.installUpdate();
      setMessage('Update installed');
    } catch (error) {
      setMessage(`Update failed: ${String(error)}`);
      setUpdateProgress(null);
    }
  }, []);

  if (!settings) {
    return (
      <div className="loading">
        <div className="loading-card">
          <div className="brand-mark"><Clipboard size={18} /></div>
          <strong>Clipboard Preview</strong>
          <span>{loadError ? `Backend error: ${loadError}` : 'Starting…'}</span>
        </div>
      </div>
    );
  }

  if (!settings.firstRunCompleted) {
    return (
      <FirstRun
        settings={settings}
        status={status}
        onDone={async () => setSettings(await backend.completeFirstRun())}
      />
    );
  }

  const performanceThreshold = status?.historyPerformanceWarningItems ?? 150;
  const memoryBudget = status?.historyMemoryBudgetMib ?? 192;
  const tabHold = Boolean(status?.tabHoldAvailable && settings.shortcuts.historySelector.toLowerCase() === 'tab');
  const progressPercent = updateProgress?.total
    ? Math.min(100, Math.round((updateProgress.downloaded / updateProgress.total) * 100))
    : null;

  return (
    <div className={`settings-app theme-${settings.appearance.theme}`}>
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark"><Clipboard size={17} /></div>
          <div><strong>Clipboard Preview</strong><span>v{status?.version ?? '1.3.0'}</span></div>
        </div>
        <nav>
          {tabs.map(([id, label, Icon]) => (
            <button key={id} className={tab === id ? 'active' : ''} onClick={() => setTab(id)}>
              <Icon size={16} />{label}
            </button>
          ))}
        </nav>
        <div className="sidebar-foot"><Shield size={14} /> Local only</div>
      </aside>

      <main className="settings-main">
        <header className="page-header">
          <div>
            <span className="eyebrow">Clipboard Preview</span>
            <h1>{tabs.find(([id]) => id === tab)?.[1]}</h1>
          </div>
          <span className={message && message !== 'Saved' && message !== 'Up to date' ? 'save-state error' : 'save-state'}>{message}</span>
        </header>

        <div className="page-body">
          {updateStatus?.available ? (
            <div className="update-banner">
              <div><strong>Clipboard Preview {updateStatus.version} is available</strong><span>{updateStatus.body || 'A newer signed release is ready to install.'}</span></div>
              <div className="update-actions">
                {updateProgress ? <div className="update-progress" title={progressPercent === null ? 'Downloading update' : `${progressPercent}%`}><i style={{ width: `${progressPercent ?? 12}%` }} /></div> : null}
                <button className="primary" onClick={() => void installUpdate()} disabled={Boolean(updateProgress)}>Update now</button>
                <button onClick={() => setUpdateStatus(null)}>Later</button>
              </div>
            </div>
          ) : null}

          {status?.lastCrashAvailable ? (
            <div className="diagnostic-banner">
              <div><strong>Previous crash captured</strong><span>The next launch stayed alive and a local diagnostic report is available.</span></div>
              <button onClick={() => { setTab('advanced'); void loadDiagnostics(); }}>Open diagnostics</button>
            </div>
          ) : null}

          {status?.startupWarnings?.length ? (
            <div className="warning">{status.startupWarnings.join(' ')}</div>
          ) : null}

          {tab === 'general' && (
            <>
              <section className="switcher-hero">
                <div className="hero-copy-block">
                  <span className="eyebrow">Default gesture</span>
                  {tabHold ? (
                    <><h2>Hold <kbd>Tab</kbd>. Scroll. Release.</h2><p>A quick tap still behaves like a normal Tab. Holding it opens your clipboard switcher without taking over your workflow.</p></>
                  ) : (
                    <><h2>Press <kbd>{settings.shortcuts.historySelector}</kbd>. Browse. Select.</h2><p>This platform uses the reliable global-shortcut path. Scroll or use arrow keys, then press Enter to restore the selected item.</p></>
                  )}
                </div>
                <div className="hero-steps" aria-label="Clipboard switcher steps">
                  <span>{tabHold ? 'Hold' : 'Open'}</span><i>→</i><span>Scroll</span><i>→</i><span>{tabHold ? 'Release' : 'Select'}</span>
                </div>
              </section>

              <Section title="Application">
                <Row label="Launch at startup"><Toggle checked={settings.general.launchAtStartup} onChange={(value) => patch('general', { ...settings.general, launchAtStartup: value })} /></Row>
                <Row label="Start hidden" hint="Only applies to OS autostart. Manual launches always open Settings."><Toggle checked={settings.general.startHidden} onChange={(value) => patch('general', { ...settings.general, startHidden: value })} /></Row>
                <Row label="Tray / menu bar icon"><Toggle checked={settings.general.showTrayIcon} onChange={(value) => patch('general', { ...settings.general, showTrayIcon: value })} /></Row>
                <Row label="Pause clipboard monitoring" hint="Existing history remains available"><Toggle checked={settings.general.monitoringPaused} onChange={(value) => patch('general', { ...settings.general, monitoringPaused: value })} /></Row>
              </Section>

              <Section title="Recent clipboard">
                <div className="settings-history">
                  {history.slice(0, 5).map((item) => <ClipboardCard key={item.id} item={item} onClick={() => void backend.selectItem(item.id)} />)}
                  {!history.length ? <div className="empty-inline">Copy text or an image to populate history.</div> : null}
                </div>
              </Section>
            </>
          )}

          {tab === 'switcher' && (
            <>
              <Section title="Shortcuts" description={status?.tabHoldAvailable ? 'Tab is handled as tap-vs-hold so short taps continue to work normally.' : 'Use a modifier-based shortcut on this platform. Plain Tab hold is intentionally unavailable.'}>
                <Row label="Clipboard Switcher" hint={status?.tabHoldAvailable ? 'Default: Tab' : 'Default: Ctrl+Alt+J'}><ShortcutRecorder value={settings.shortcuts.historySelector} onChange={(value) => patch('shortcuts', { ...settings.shortcuts, historySelector: value })} /></Row>
                <Row label="Quick Preview"><ShortcutRecorder value={settings.shortcuts.quickPreview} onChange={(value) => patch('shortcuts', { ...settings.shortcuts, quickPreview: value })} /></Row>
                <Row label="Open Settings"><ShortcutRecorder value={settings.shortcuts.openSettings} onChange={(value) => patch('shortcuts', { ...settings.shortcuts, openSettings: value })} /></Row>
                <Row label="Pause / Resume"><ShortcutRecorder value={settings.shortcuts.pauseMonitoring} onChange={(value) => patch('shortcuts', { ...settings.shortcuts, pauseMonitoring: value })} /></Row>
              </Section>

              <Section title="Interaction">
                <Row label="Mode"><select value={settings.history.interactionMode} onChange={(event) => patch('history', { ...settings.history, interactionMode: event.target.value as AppSettings['history']['interactionMode'] })}><option value="hold_release">Hold → scroll → release</option><option value="sticky">Press → navigate → Enter</option></select></Row>
                <Row label="Reverse scroll direction"><Toggle checked={settings.history.scrollDirection === 'reversed'} onChange={(value) => patch('history', { ...settings.history, scrollDirection: value ? 'reversed' : 'natural' })} /></Row>
                <Row label="Wrap selection"><Toggle checked={settings.history.wrapSelection} onChange={(value) => patch('history', { ...settings.history, wrapSelection: value })} /></Row>
              </Section>

              <Section title="Layout" description="Choose between the split detail view and a smaller row-only switcher.">
                <Row label="Large preview panel" hint="Off keeps the switcher compact. Images still open a small floating preview after you pause on the selected row."><Toggle checked={settings.history.largePreviewPanel} onChange={(value) => patch('history', { ...settings.history, largePreviewPanel: value })} /></Row>
              </Section>

              <Section title="Quick Preview">
                <Row label="Maximum characters"><NumberField value={settings.preview.maxCharacters} min={20} max={2000} onChange={(value) => patch('preview', { ...settings.preview, maxCharacters: value })} /></Row>
                <Row label="Maximum lines"><NumberField value={settings.preview.maxLines} min={1} max={20} onChange={(value) => patch('preview', { ...settings.preview, maxLines: value })} /></Row>
                <Row label="Width"><NumberField value={settings.preview.width} min={280} max={760} suffix="px" onChange={(value) => patch('preview', { ...settings.preview, width: value })} /></Row>
                <Row label="Position"><select value={settings.preview.position} onChange={(event) => patch('preview', { ...settings.preview, position: event.target.value as AppSettings['preview']['position'] })}><option value="cursor">Near cursor</option><option value="screen_center">Screen center</option><option value="top_center">Top center</option><option value="bottom_center">Bottom center</option></select></Row>
              </Section>
            </>
          )}

          {tab === 'history' && (
            <>
              <Section title="History limits" description={`History is also bounded to about ${memoryBudget} MiB of in-memory payloads so image-heavy sessions stay lightweight.`}>
                <Row label="Maximum history items"><NumberField value={settings.history.maxItems} min={1} max={250} onChange={(value) => patch('history', { ...settings.history, maxItems: value })} /></Row>
                <Row label="Visible rows"><NumberField value={settings.history.visibleItems} min={3} max={12} onChange={(value) => patch('history', { ...settings.history, visibleItems: value })} /></Row>
                <Row label="Move selected item to top"><Toggle checked={settings.history.moveSelectedToTop} onChange={(value) => patch('history', { ...settings.history, moveSelectedToTop: value })} /></Row>
                <Row label="Persist between restarts" hint="Off by default for privacy. Images are persisted as compressed PNG only when enabled."><Toggle checked={settings.history.persistHistory} onChange={(value) => patch('history', { ...settings.history, persistHistory: value })} /></Row>
                <Row label="Clear history on exit"><Toggle checked={settings.history.clearOnExit} onChange={(value) => patch('history', { ...settings.history, clearOnExit: value })} /></Row>
                {settings.history.maxItems > performanceThreshold ? <div className="soft-warning">Large histories can use more memory and disk, especially when they contain screenshots or other images.</div> : null}
              </Section>

              <Section title="Image history" description="Images are compressed in memory, shown as thumbnails in the list, and the full preview is loaded only after you pause on an item.">
                <div className="feature-note"><ImageIcon size={18} /><div><strong>{status?.imageHistoryAvailable ? 'Image clipboard enabled' : 'Image clipboard unavailable on this platform'}</strong><span>Text and image entries share the same item count and memory budget.</span></div></div>
              </Section>

              <Section title="Clear history">
                <div className="action-row"><button className="danger-quiet" onClick={async () => { await backend.clearHistory(); setHistory([]); }}><Trash2 size={15} /> Clear clipboard history</button></div>
              </Section>
            </>
          )}

          {tab === 'appearance' && (
            <>
              <Section title="Application appearance">
                <Row label="Theme"><select value={settings.appearance.theme} onChange={(event) => patch('appearance', { ...settings.appearance, theme: event.target.value as AppSettings['appearance']['theme'] })}><option value="system">System</option><option value="light">Light</option><option value="dark">Dark</option></select></Row>
                <Row label="Overlay opacity"><NumberField value={Math.round(settings.appearance.overlayOpacity * 100)} min={72} max={100} suffix="%" onChange={(value) => patch('appearance', { ...settings.appearance, overlayOpacity: value / 100 })} /></Row>
                <Row label="Corner radius"><NumberField value={settings.appearance.cornerRadius} min={6} max={24} suffix="px" onChange={(value) => patch('appearance', { ...settings.appearance, cornerRadius: value })} /></Row>
                <Row label="Compact spacing"><Toggle checked={settings.appearance.compactSpacing} onChange={(value) => patch('appearance', { ...settings.appearance, compactSpacing: value })} /></Row>
                <Row label="Reduced motion"><Toggle checked={settings.appearance.reducedMotion} onChange={(value) => patch('appearance', { ...settings.appearance, reducedMotion: value })} /></Row>
              </Section>
              <Section title="Clipboard Switcher" description="This is a live preview of the real switcher. Tune every text layer, the row geometry, thumbnails, and the restrained surface colors here.">
                <SwitcherAppearanceEditor appearance={settings.appearance} onChange={(appearance) => patch('appearance', appearance)} />
              </Section>
            </>
          )}

          {tab === 'advanced' && (
            <>
              <Section title="Native integration">
                <Row label="Clipboard poll interval"><NumberField value={settings.advanced.clipboardPollIntervalMs} min={150} max={1500} step={50} suffix="ms" onChange={(value) => patch('advanced', { ...settings.advanced, clipboardPollIntervalMs: value })} /></Row>
                <Row label="Debug logging" hint="Clipboard contents are never intentionally logged"><Toggle checked={settings.advanced.debugLogging} onChange={(value) => patch('advanced', { ...settings.advanced, debugLogging: value })} /></Row>
                {status?.accessibilityRequired ? <div className={status.accessibilityGranted ? 'permission-ok' : 'warning'}>{status.accessibilityGranted ? 'macOS Accessibility access is available for Tab hold and global wheel selection.' : 'Grant macOS Accessibility access to use the default Tab hold gesture. Modifier-based sticky shortcuts remain available without it.'}</div> : null}
              </Section>

              <Section title="Diagnostics" description="Crash and startup diagnostics stay local and never intentionally contain clipboard contents.">
                <div className="diagnostic-actions">
                  <button onClick={() => void loadDiagnostics()}><SlidersHorizontal size={15} /> Load report</button>
                  <button onClick={() => void backend.openDiagnosticsFolder()}><FolderOpen size={15} /> Open folder</button>
                  {diagnostics ? <button onClick={async () => { await navigator.clipboard.writeText(diagnostics); setMessage('Diagnostics copied'); }}><Copy size={15} /> Copy report</button> : null}
                </div>
                {diagnostics ? <pre className="diagnostics-output">{diagnostics}</pre> : null}
                {status?.lastCrashAvailable ? <div className="action-row"><button className="quiet-button" onClick={async () => { await backend.clearDiagnostics(); setDiagnostics(''); setStatus(await backend.status()); }}>Dismiss previous crash report</button></div> : null}
              </Section>

              <Section title="Reset">
                <div className="action-row"><button className="danger-quiet" onClick={async () => { if (confirm('Reset Clipboard Preview to defaults?')) setSettings(await backend.reset()); }}><Trash2 size={15} /> Reset settings to defaults</button></div>
              </Section>
            </>
          )}

          {tab === 'about' && (
            <Section title="Clipboard Preview">
              <div className="about">
                <div className="about-icon"><Clipboard size={25} /></div>
                <h3>Clipboard Preview</h3>
                <p className="version">Version {status?.version ?? '1.3.0'}</p>
                <p>A lightweight text and image clipboard switcher for Windows, macOS, and Linux.</p>
                <p>Made by <a href="https://github.com/SirPaul-code" onClick={(event) => { event.preventDefault(); void backend.openExternal('https://github.com/SirPaul-code'); }}><strong>Pavol Duplinsky</strong></a> · <a href="https://github.com/SirPaul-code" onClick={(event) => { event.preventDefault(); void backend.openExternal('https://github.com/SirPaul-code'); }}>@SirPaul-code</a></p>
                <div className="privacy-note"><Shield size={18} /><div><strong>Local by default</strong><span>No telemetry, accounts, cloud sync, or remote clipboard API.</span></div></div>

                <div className="privacy-note">
                  <Info size={18} />
                  <div>
                    <strong>{status?.officialBuild ? 'Official release build' : 'Source / development build'}</strong>
                    <span className="update-status-copy">{status?.officialBuild ? (status.updatesEnabled ? 'Signed automatic update checks are enabled.' : 'Automatic updater is unavailable in this build.') : 'Automatic updates are intentionally disabled. Forks and local builds never require Clipboard Preview signing secrets.'}</span>
                  </div>
                </div>
                {status?.officialBuild && status.updatesEnabled ? (
                  <div className="action-row update-actions">
                    <button onClick={() => void checkUpdates()} disabled={updateChecking}>{updateChecking ? 'Checking…' : 'Check for updates'}</button>
                    {updateStatus?.available ? <button className="primary" onClick={() => void installUpdate()} disabled={Boolean(updateProgress)}>Update to {updateStatus.version}</button> : null}
                  </div>
                ) : null}

                <div className="about-links">
                  <a href="https://github.com/SirPaul-code/ClipboardPreview" onClick={(event) => { event.preventDefault(); void backend.openExternal('https://github.com/SirPaul-code/ClipboardPreview'); }}>GitHub</a>
                  <a href="https://github.com/SirPaul-code/ClipboardPreview/releases" onClick={(event) => { event.preventDefault(); void backend.openExternal('https://github.com/SirPaul-code/ClipboardPreview/releases'); }}>Releases</a>
                  <a href="https://github.com/SirPaul-code/ClipboardPreview/issues" onClick={(event) => { event.preventDefault(); void backend.openExternal('https://github.com/SirPaul-code/ClipboardPreview/issues'); }}>Report an issue</a>
                  <a href="https://github.com/SirPaul-code/ClipboardPreview/blob/main/LICENSE" onClick={(event) => { event.preventDefault(); void backend.openExternal('https://github.com/SirPaul-code/ClipboardPreview/blob/main/LICENSE'); }}>MIT License</a>
                </div>
              </div>
            </Section>
          )}
        </div>
      </main>
    </div>
  );
}

function FirstRun({ settings, status, onDone }: { settings: AppSettings; status: PlatformStatus | null; onDone: () => void }) {
  const tabHold = Boolean(status?.tabHoldAvailable && settings.shortcuts.historySelector.toLowerCase() === 'tab');
  return (
    <main className="first-run">
      <div className="first-card">
        <div className="hero-mark"><Clipboard size={27} /></div>
        <span className="eyebrow">Clipboard Preview</span>
        <h1>{tabHold ? 'Hold Tab. Pick anything.' : 'Your clipboard, one shortcut away.'}</h1>
        <p className="hero-copy">{tabHold ? 'Text and images stay local. Tap Tab normally, or hold it to open the switcher.' : `Text and images stay local. Use ${settings.shortcuts.historySelector} to open the switcher on this platform.`}</p>
        <div className="first-shortcuts">
          <div><span>Clipboard Switcher</span><kbd>{settings.shortcuts.historySelector}</kbd></div>
          <div><span>Quick Preview</span><kbd>{settings.shortcuts.quickPreview}</kbd></div>
        </div>
        {status?.startupWarnings?.length ? <div className="warning">{status.startupWarnings.join(' ')}</div> : null}
        {status?.accessibilityRequired && !status.accessibilityGranted ? <p className="permission-copy">On macOS, the default Tab hold gesture and global wheel capture require Accessibility permission. You can also choose a modifier-based shortcut in Settings.</p> : null}
        <button className="primary" onClick={onDone}>Start Clipboard Preview</button>
        <p className="local-copy"><Shield size={14} /> Everything stays on this device.</p>
      </div>
    </main>
  );
}
