import { useCallback, useEffect, useState } from 'react';
import { Clipboard, Command, Eye, History, Info, Palette, Settings2, Shield, SlidersHorizontal, Trash2 } from 'lucide-react';
import { backend } from '../lib/tauri';
import type { AppSettings, ClipboardItem, PlatformStatus } from '../types';
import { ClipboardCard } from '../components/ClipboardCard';
import { NumberField, Row, Section, Toggle } from '../components/SettingsControls';
import { ShortcutRecorder } from '../components/ShortcutRecorder';

type Tab = 'general' | 'shortcuts' | 'preview' | 'history' | 'appearance' | 'advanced' | 'about';
const tabs: Array<[Tab, string, typeof Clipboard]> = [
  ['general', 'General', Settings2], ['shortcuts', 'Shortcuts', Command], ['preview', 'Preview', Eye],
  ['history', 'History', History], ['appearance', 'Appearance', Palette], ['advanced', 'Advanced', SlidersHorizontal], ['about', 'About', Info],
];

export function SettingsApp() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [history, setHistory] = useState<ClipboardItem[]>([]);
  const [status, setStatus] = useState<PlatformStatus | null>(null);
  const [tab, setTab] = useState<Tab>('general');
  const [message, setMessage] = useState('');
  const [loadError, setLoadError] = useState('');

  const load = useCallback(async () => {
    try {
      const [s, h, p] = await Promise.all([backend.settings(), backend.history(), backend.status()]);
      setSettings(s); setHistory(h); setStatus(p); setLoadError('');
    } catch (error) {
      setLoadError(String(error));
    }
  }, []);
  useEffect(() => { void load(); }, [load]);

  const save = useCallback(async (next: AppSettings) => {
    setSettings(next);
    try {
      setSettings(await backend.saveSettings(next));
      setMessage('Saved'); window.setTimeout(() => setMessage(''), 800);
    } catch (error) {
      setMessage(String(error)); void load();
    }
  }, [load]);

  const patch = useCallback(<K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    if (settings) void save({ ...settings, [key]: value });
  }, [settings, save]);

  if (!settings) return <div className="loading">{loadError ? `Clipboard Preview backend failed to initialize: ${loadError}` : 'Loading Clipboard Preview…'}</div>;
  if (!settings.firstRunCompleted) return <FirstRun settings={settings} status={status} onDone={async () => setSettings(await backend.completeFirstRun())} />;

  return <div className={`settings-app theme-${settings.appearance.theme}`}>
    <aside className="sidebar">
      <div className="brand"><div className="brand-mark"><Clipboard size={18}/></div><div><strong>Clipboard Preview</strong><span>Local utility</span></div></div>
      <nav>{tabs.map(([id, label, Icon]) => <button key={id} className={tab === id ? 'active' : ''} onClick={() => setTab(id)}><Icon size={17}/>{label}</button>)}</nav>
      <div className="sidebar-foot"><Shield size={15}/> Clipboard data stays local</div>
    </aside>
    <main className="settings-main">
      <header className="page-header"><div><h1>{tabs.find(x => x[0] === tab)?.[1]}</h1><p>Fast, local clipboard behavior without unnecessary background services.</p></div><span className={message && message !== 'Saved' ? 'save-state error' : 'save-state'}>{message}</span></header>
      <div className="page-body">
        {status?.startupWarnings?.length ? <div className="warning">{status.startupWarnings.join(' ')}</div> : null}
        {tab === 'general' && <>
          <Section title="Application">
            <Row label="Launch at startup"><Toggle checked={settings.general.launchAtStartup} onChange={v => patch('general', { ...settings.general, launchAtStartup: v })}/></Row>
            <Row label="Start hidden"><Toggle checked={settings.general.startHidden} onChange={v => patch('general', { ...settings.general, startHidden: v })}/></Row>
            <Row label="Tray / menu bar icon"><Toggle checked={settings.general.showTrayIcon} onChange={v => patch('general', { ...settings.general, showTrayIcon: v })}/></Row>
            <Row label="Pause clipboard monitoring" hint="Existing history remains available"><Toggle checked={settings.general.monitoringPaused} onChange={v => patch('general', { ...settings.general, monitoringPaused: v })}/></Row>
          </Section>
          <Section title="Recent clipboard"><div className="settings-history">{history.slice(0, 4).map(item => <ClipboardCard key={item.id} item={item} onClick={() => void backend.selectItem(item.id)}/>)}{!history.length && <div className="empty-inline">Copy some text to populate history.</div>}</div></Section>
        </>}
        {tab === 'shortcuts' && <Section title="Global shortcuts" description="Click a shortcut and press a new combination. Duplicate or unavailable shortcuts are rejected.">
          <Row label="Quick Preview"><ShortcutRecorder value={settings.shortcuts.quickPreview} onChange={v => patch('shortcuts', { ...settings.shortcuts, quickPreview: v })}/></Row>
          <Row label="Clipboard History"><ShortcutRecorder value={settings.shortcuts.historySelector} onChange={v => patch('shortcuts', { ...settings.shortcuts, historySelector: v })}/></Row>
          <Row label="Open Settings"><ShortcutRecorder value={settings.shortcuts.openSettings} onChange={v => patch('shortcuts', { ...settings.shortcuts, openSettings: v })}/></Row>
          <Row label="Pause / Resume"><ShortcutRecorder value={settings.shortcuts.pauseMonitoring} onChange={v => patch('shortcuts', { ...settings.shortcuts, pauseMonitoring: v })}/></Row>
        </Section>}
        {tab === 'preview' && <Section title="Quick Preview">
          <Row label="Maximum characters"><NumberField value={settings.preview.maxCharacters} min={20} max={2000} onChange={v => patch('preview', { ...settings.preview, maxCharacters: v })}/></Row>
          <Row label="Maximum lines"><NumberField value={settings.preview.maxLines} min={1} max={20} onChange={v => patch('preview', { ...settings.preview, maxLines: v })}/></Row>
          <Row label="Width"><NumberField value={settings.preview.width} min={280} max={760} suffix="px" onChange={v => patch('preview', { ...settings.preview, width: v })}/></Row>
          <Row label="Font size"><NumberField value={settings.preview.fontSize} min={11} max={22} suffix="px" onChange={v => patch('preview', { ...settings.preview, fontSize: v })}/></Row>
          <Row label="Text wrapping"><Toggle checked={settings.preview.textWrapping} onChange={v => patch('preview', { ...settings.preview, textWrapping: v })}/></Row>
          <Row label="Character count"><Toggle checked={settings.preview.showCharacterCount} onChange={v => patch('preview', { ...settings.preview, showCharacterCount: v })}/></Row>
          <Row label="Auto-hide"><NumberField value={settings.preview.autoHideMs} min={500} max={8000} step={100} suffix="ms" onChange={v => patch('preview', { ...settings.preview, autoHideMs: v })}/></Row>
          <Row label="Position"><select value={settings.preview.position} onChange={e => patch('preview', { ...settings.preview, position: e.target.value as AppSettings['preview']['position'] })}><option value="cursor">Near cursor</option><option value="screen_center">Screen center</option><option value="top_center">Top center</option><option value="bottom_center">Bottom center</option></select></Row>
        </Section>}
        {tab === 'history' && <Section title="Clipboard History">
          <Row label="Maximum history items"><NumberField value={settings.history.maxItems} min={1} max={100} onChange={v => patch('history', { ...settings.history, maxItems: v })}/></Row>
          <Row label="Visible rows"><NumberField value={settings.history.visibleItems} min={3} max={12} onChange={v => patch('history', { ...settings.history, visibleItems: v })}/></Row>
          <Row label="Interaction mode"><select value={settings.history.interactionMode} onChange={e => patch('history', { ...settings.history, interactionMode: e.target.value as AppSettings['history']['interactionMode'] })}><option value="hold_release">Hold → scroll → release</option><option value="sticky">Press → navigate → Enter</option></select></Row>
          <Row label="Reverse scroll direction"><Toggle checked={settings.history.scrollDirection === 'reversed'} onChange={v => patch('history', { ...settings.history, scrollDirection: v ? 'reversed' : 'natural' })}/></Row>
          <Row label="Wrap selection"><Toggle checked={settings.history.wrapSelection} onChange={v => patch('history', { ...settings.history, wrapSelection: v })}/></Row>
          <Row label="Move selected item to top"><Toggle checked={settings.history.moveSelectedToTop} onChange={v => patch('history', { ...settings.history, moveSelectedToTop: v })}/></Row>
          <Row label="Persist between restarts" hint="Off by default for privacy"><Toggle checked={settings.history.persistHistory} onChange={v => patch('history', { ...settings.history, persistHistory: v })}/></Row>
          <Row label="Clear history on exit"><Toggle checked={settings.history.clearOnExit} onChange={v => patch('history', { ...settings.history, clearOnExit: v })}/></Row>
          <div className="action-row"><button className="danger-quiet" onClick={async () => { await backend.clearHistory(); setHistory([]); }}><Trash2 size={16}/> Clear clipboard history</button></div>
        </Section>}
        {tab === 'appearance' && <Section title="Appearance">
          <Row label="Theme"><select value={settings.appearance.theme} onChange={e => patch('appearance', { ...settings.appearance, theme: e.target.value as AppSettings['appearance']['theme'] })}><option value="system">System</option><option value="light">Light</option><option value="dark">Dark</option></select></Row>
          <Row label="Overlay opacity"><NumberField value={Math.round(settings.appearance.overlayOpacity * 100)} min={72} max={100} suffix="%" onChange={v => patch('appearance', { ...settings.appearance, overlayOpacity: v / 100 })}/></Row>
          <Row label="Corner radius"><NumberField value={settings.appearance.cornerRadius} min={6} max={24} suffix="px" onChange={v => patch('appearance', { ...settings.appearance, cornerRadius: v })}/></Row>
          <Row label="Compact spacing"><Toggle checked={settings.appearance.compactSpacing} onChange={v => patch('appearance', { ...settings.appearance, compactSpacing: v })}/></Row>
          <Row label="Reduced motion"><Toggle checked={settings.appearance.reducedMotion} onChange={v => patch('appearance', { ...settings.appearance, reducedMotion: v })}/></Row>
        </Section>}
        {tab === 'advanced' && <>
          <Section title="Native integration">
            <Row label="Clipboard poll interval"><NumberField value={settings.advanced.clipboardPollIntervalMs} min={150} max={1500} step={50} suffix="ms" onChange={v => patch('advanced', { ...settings.advanced, clipboardPollIntervalMs: v })}/></Row>
            <Row label="Debug logging" hint="Clipboard content is never intentionally logged"><Toggle checked={settings.advanced.debugLogging} onChange={v => patch('advanced', { ...settings.advanced, debugLogging: v })}/></Row>
            {status?.accessibilityRequired && <div className={status.accessibilityGranted ? 'permission-ok' : 'warning'}>{status.accessibilityGranted ? 'macOS Accessibility permission is available for global wheel selection.' : 'macOS Accessibility permission is not granted. Sticky mode remains usable.'}</div>}
          </Section>
          <Section title="Reset"><div className="action-row"><button className="danger-quiet" onClick={async () => { if (confirm('Reset Clipboard Preview to defaults?')) setSettings(await backend.reset()); }}><Trash2 size={16}/> Reset settings to defaults</button></div></Section>
        </>}
        {tab === 'about' && <Section title="Clipboard Preview"><div className="about"><div className="about-icon"><Clipboard size={28}/></div><h3>Clipboard Preview</h3><p className="version">Version {status?.version ?? '1.0.1'}</p><p>A fast, minimal clipboard preview and history utility for Windows and macOS.</p><div className="privacy-note"><Shield size={18}/><div><strong>Local by default</strong><span>No telemetry, accounts, cloud sync, or remote clipboard API.</span></div></div><div className="about-links"><a href="https://github.com/SirPaul-code/ClipboardPreview">GitHub</a><a href="https://github.com/SirPaul-code/ClipboardPreview/issues">Report an issue</a><span>MIT License</span></div></div></Section>}
      </div>
    </main>
  </div>;
}

function FirstRun({ settings, status, onDone }: { settings: AppSettings; status: PlatformStatus | null; onDone: () => void }) {
  return <main className="first-run"><div className="first-card"><div className="hero-mark"><Clipboard size={28}/></div><h1>Clipboard Preview</h1><p className="hero-copy">Your clipboard, one shortcut away.</p><div className="first-shortcuts"><div><span>Quick Preview</span><kbd>{settings.shortcuts.quickPreview}</kbd></div><div><span>Clipboard History</span><kbd>{settings.shortcuts.historySelector}</kbd></div></div>{status?.startupWarnings?.length ? <div className="warning">{status.startupWarnings.join(' ')}</div> : null}{status?.accessibilityRequired && !status.accessibilityGranted && <p className="permission-copy">On macOS, hold-and-scroll selection needs Accessibility permission for global mouse-wheel input. Sticky mode works without it.</p>}<button className="primary" onClick={onDone}>Start Clipboard Preview</button><p className="local-copy"><Shield size={14}/> Everything stays local.</p></div></main>;
}
