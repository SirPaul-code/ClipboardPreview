# Manual desktop validation

CI proves frontend checks, Rust tests, clippy, and native compilation. It cannot truthfully prove foreground focus behavior, real OS clipboard ownership, global input interception/replay, permission dialogs, installer behavior, or multi-monitor placement. Use this checklist for release validation and record only what was actually tested.

## Startup invariant — every platform/build

- [ ] Manual launch renders Settings and does not terminate during initial frontend IPC.
- [ ] Backend state exists before any configured WebView is created; no `state() called before manage()` panic appears in diagnostics.
- [ ] Closing Settings hides the window but leaves the background utility running when tray/menu operation is enabled.
- [ ] OS autostart with `--hidden` may start hidden; a normal manual launch must not inherit that behavior.
- [ ] A shortcut/tray/native-input initialization failure produces a visible warning or capability fallback rather than terminating the process.
- [ ] Advanced → Diagnostics can load/copy a report and open the local diagnostics directory.

## Windows 10/11

- [ ] Install both the generated MSI and NSIS setup build on a normal Windows account (one at a time).
- [ ] Launch from the installed Windows application entry and verify the app stays alive after Settings is closed.
- [ ] Tray menu opens Settings/History, pauses/resumes, clears history, toggles startup, opens About, and quits.
- [ ] Short `Tab` taps still move focus/indent normally in representative apps (browser form, Notepad/editor, IDE).
- [ ] Hold `Tab` for the switcher, scroll, release, then `Ctrl+V`; the highlighted text item is pasted.
- [ ] Scroll while the switcher is active does not also scroll the foreground application.
- [ ] Copy an image/screenshot: a thumbnail appears in history and selecting it restores an image to the clipboard.
- [ ] Pause on a selected image: the full preview appears after the configured delay; scroll away before the delay and the old full preview does not appear.
- [ ] Consecutive duplicate text or identical images do not create duplicate top history rows.
- [ ] Pausing prevents new history entries; resuming works normally.
- [ ] Quick Preview appears without stealing focus from the foreground application.
- [ ] Sticky mode supports wheel, arrows/J/K, Enter, and Escape.
- [ ] Selecting text or image history does not immediately feed back as a duplicate history row.
- [ ] Rebinding the switcher away from Tab registers a conventional global shortcut; conflicting bindings are rejected.
- [ ] Overlay follows the cursor to a second monitor and stays inside its work area at mixed DPI/scaling.
- [ ] Memory-only history disappears after restart; persisted text/image history survives; corrupted JSON falls back safely.
- [ ] Configure more than 150 history items and verify the UI displays the performance warning.
- [ ] Launch-at-startup and tray visibility toggles behave as configured.
- [ ] Test an elevated application separately and record any Windows UIPI/input-boundary limitation rather than treating it as ordinary desktop behavior.

## macOS

- [ ] Install both generated architecture bundles on matching hardware where available (Apple Silicon and Intel).
- [ ] Menu-bar icon/menu works and closing Settings keeps the utility running.
- [ ] Text/image monitoring, deduplication, pause/resume, persistence, and clear behavior match Windows.
- [ ] Quick Preview appears without taking focus from the current app.
- [ ] Without Accessibility permission, the app stays alive and explains that Tab hold/global wheel capture is unavailable.
- [ ] Without Accessibility permission, configure a modifier-based switcher shortcut and verify sticky mode remains usable.
- [ ] Grant Accessibility permission, restart Clipboard Preview, then verify short Tab replay plus hold Tab → global wheel → release and `Cmd+V`.
- [ ] Copy a screenshot/image, verify thumbnail, delayed full preview, selection, and clipboard restore.
- [ ] Rebinding shortcuts works and conflicts are rejected.
- [ ] Multi-monitor/Retina positioning stays on the cursor's active display.
- [ ] Launch at login works after enabling it; manual launch still shows Settings.
- [ ] Explicit Quit terminates the background process.

## History/resource sanity

- [ ] Default 40-item history remains responsive with a mixed text/image workload.
- [ ] Count limit removes the oldest items first.
- [ ] Image-heavy history remains bounded by the internal payload budget rather than growing indefinitely.
- [ ] Persisted history writes are debounced during rapid copy operations.
- [ ] Diagnostics and standard logs do not intentionally contain copied text or image payloads.
- [ ] Idle clipboard polling does not create a network loop.

Capture CPU/memory measurements only from a real build/environment; do not invent performance numbers in documentation.
