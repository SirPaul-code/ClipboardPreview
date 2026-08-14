# Manual desktop validation

CI can prove frontend checks, Rust tests, and native builds compile. It cannot truthfully prove foreground focus behavior, real OS clipboard ownership, global input hooks, permission dialogs, or multi-monitor placement. Use this checklist for release validation and record only what was actually tested.

## Windows 10/11

- [ ] Install the generated installer on a current Windows account.
- [ ] App launches and first-run window renders without a white flash.
- [ ] Closing settings hides the window but leaves the process/tray icon running.
- [ ] Tray menu opens Settings/History, pauses/resumes, clears history, toggles startup, opens About, and quits.
- [ ] Distinct text creates ordered history; consecutive duplicates do not create duplicate rows.
- [ ] Pausing prevents new history entries; resuming works normally.
- [ ] Quick Preview appears without stealing focus from the foreground application.
- [ ] Hold the history shortcut, scroll, release, then `Ctrl+V`; the highlighted item is pasted.
- [ ] Sticky mode supports wheel, arrows/J/K, Enter, and Escape.
- [ ] Selecting a history item does not create a duplicate history row.
- [ ] Rebinding each shortcut works; conflicting bindings are rejected.
- [ ] Overlay follows the cursor to a second monitor and stays inside its work area at mixed DPI/scaling.
- [ ] Memory-only history disappears after restart; persisted history survives; corrupted JSON falls back safely.
- [ ] Launch-at-startup and tray visibility toggles behave as configured.

## macOS

- [ ] Install the generated DMG/app and launch it.
- [ ] Menu-bar icon/menu works and closing Settings keeps the utility running.
- [ ] Text monitoring, deduplication, pause/resume, persistence, and clear behavior match Windows.
- [ ] Quick Preview appears without taking focus from the current app.
- [ ] Without Accessibility permission, the app reports the limitation and sticky selector remains usable.
- [ ] Grant Accessibility permission, restart Clipboard Preview, then verify hold → global wheel → release and `Cmd+V`.
- [ ] Rebinding shortcuts works and conflicts are rejected.
- [ ] Multi-monitor/Retina positioning stays on the cursor's active display.
- [ ] Launch at login works after enabling it.
- [ ] Explicit Quit terminates the background process.

## Resource sanity

Observe the utility after several idle minutes and during normal copy/paste activity. Version 1 intentionally sleeps between clipboard reads and performs no network loop. Capture measurements only from a real build/environment; do not invent CPU or memory numbers in documentation.
