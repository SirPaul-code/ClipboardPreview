import type { PlatformStatus } from '../types';
import { backend } from '../lib/tauri';

export function MacPermissionPanel({ status }: { status: PlatformStatus | null }) {
  if (!status || status.os !== 'macos') return null;

  const ready = status.nativeInputReady;

  return (
    <section className={`mac-permission-panel ${ready ? 'ready' : 'waiting'}`}>
      <div className="mac-permission-summary">
        <div>
          <strong>{ready ? 'macOS switcher input is ready' : 'Finish macOS input setup'}</strong>
          <span>
            {ready
              ? 'Clipboard Preview is receiving global keyboard and wheel events. Tab and your configured switcher shortcut are ready to use.'
              : status.accessibilityGranted
                ? 'Accessibility is granted. Clipboard Preview is starting its native input capture; this status turns green only when the event tap is actually running.'
                : 'Clipboard Preview needs Accessibility access before it can intercept the switcher trigger globally.'}
          </span>
        </div>
        <span className={`permission-state ${ready ? 'ok' : 'pending'}`}>
          {ready ? 'READY' : 'NOT READY'}
        </span>
      </div>

      <div className="mac-permission-row">
        <div>
          <strong>Accessibility</strong>
          <span>Required for the active global switcher input hook.</span>
        </div>
        <div className="mac-permission-actions">
          <span className={`permission-state ${status.accessibilityGranted ? 'ok' : 'pending'}`}>
            {status.accessibilityGranted ? 'GRANTED' : 'REQUIRED'}
          </span>
          <button onClick={() => void backend.openMacAccessibilitySettings()}>Open Accessibility</button>
        </div>
      </div>

      <div className="mac-permission-row">
        <div>
          <strong>Input Monitoring</strong>
          <span>Shown separately for clarity. The v1.3.3 active session event tap does not require it.</span>
        </div>
        <div className="mac-permission-actions">
          <span className={`permission-state ${status.inputMonitoringGranted ? 'ok' : 'optional'}`}>
            {status.inputMonitoringGranted ? 'GRANTED' : 'OPTIONAL'}
          </span>
          <button onClick={() => void backend.openMacInputMonitoringSettings()}>Open Input Monitoring</button>
        </div>
      </div>
    </section>
  );
}
