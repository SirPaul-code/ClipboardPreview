# Manual desktop validation

CI proves frontend checks, Rust tests, strict clippy, and distributable bundle creation. It cannot truthfully prove foreground focus behavior, real OS clipboard ownership, global input interception/replay, permission dialogs, installer behavior, update UX, or multi-monitor placement. Use this checklist for release validation and record only what was actually tested.

## Startup invariant — every platform/build

- [ ] Manual launch renders Settings and does not terminate during initial frontend IPC.
- [ ] Backend state exists before any configured WebView is created; no `state() called before manage()` panic appears in diagnostics.
- [ ] Closing Settings hides the window but leaves the background utility running when tray/menu operation is enabled.
- [ ] OS autostart with `--hidden` may start hidden; a normal manual launch must not inherit that behavior.
- [ ] A shortcut/tray/native-input/updater/notification initialization failure produces a visible warning or capability fallback rather than terminating the process.
- [ ] Advanced → Diagnostics can load/copy a report and open the local diagnostics directory.
- [ ] GitHub/Releases/Issue/License/author links open the system browser and never replace the Settings WebView.

## Clipboard Switcher behavior

- [ ] Clipboard rows show capture date/time rather than character counts or image dimensions.
- [ ] Mouse wheel changes selection while the switcher is open.
- [ ] `ArrowUp` and `ArrowDown` change selection by default.
- [ ] Rebind Previous/Next to another supported pair (for example `J` / `K` or `Ctrl+J` / `Ctrl+K`) and verify the new bindings work.
- [ ] Previous/Next bindings are not consumed while the switcher is closed; the foreground application still receives those keys normally.
- [ ] Previous/Next key presses do not leak into the foreground application while hold-mode switcher selection is active.
- [ ] Sticky mode uses the same configured Previous/Next bindings as hold mode.
- [ ] Wheel direction, wrap-selection, and move-selected-to-top settings still behave as configured.
- [ ] Clicking a row remains a valid selection path.

## Switcher appearance editor

- [ ] Settings → Appearance → Clipboard Switcher renders a live preview without opening the real overlay.
- [ ] Changing header title/subtitle/meta size or color updates the preview and the real switcher.
- [ ] Changing item type/content/date size or color updates the preview and the real switcher.
- [ ] Changing detail content/meta and footer size/color updates the preview and the real switcher.
- [ ] Panel, row, selected-row, border, and selected-border colors apply consistently.
- [ ] Row height changes native overlay sizing rather than clipping or leaving empty slots.
- [ ] Thumbnail size is clamped to fit the configured row height.
- [ ] Invalid/out-of-range persisted appearance values normalize safely instead of breaking rendering.

## Windows 10/11

- [ ] Install both the generated MSI and NSIS setup build on a normal Windows account (one at a time).
- [ ] On a machine without WebView2, verify the installer bootstrapper handles the missing runtime before first launch.
- [ ] Launch from the installed Windows application entry and verify the app stays alive after Settings is closed.
- [ ] Tray menu opens Settings/History, pauses/resumes, clears history, toggles startup, opens About, and quits.
- [ ] Short `Tab` taps still move focus/indent normally in representative apps (browser form, Notepad/editor, IDE).
- [ ] Hold `Tab` for the switcher, use wheel and `ArrowUp`/`ArrowDown`, release, then `Ctrl+V`; the highlighted text item is pasted.
- [ ] Scroll while the switcher is active does not also scroll the foreground application.
- [ ] Copy an image/screenshot: a thumbnail appears in history and selecting it restores an image to the clipboard.
- [ ] Pause on a selected image: the full preview appears after the configured delay; navigate away before the delay and the old full preview does not appear.
- [ ] Consecutive duplicate text or identical images do not create duplicate top history rows.
- [ ] Pausing prevents new history entries; resuming works normally.
- [ ] Quick Preview appears without stealing focus and shows capture time rather than dimensions/counts.
- [ ] Selecting text or image history does not immediately feed back as a duplicate history row.
- [ ] Rebinding the switcher away from Tab registers a conventional global shortcut; conflicting bindings are rejected.
- [ ] Overlay follows the cursor to a second monitor and stays inside its work area at mixed DPI/scaling.
- [ ] Memory-only history disappears after restart; persisted text/image history survives; corrupted JSON falls back safely.
- [ ] Configure more than 150 history items and verify the UI displays the performance warning.
- [ ] Launch-at-startup and tray visibility toggles behave as configured.
- [ ] Test an elevated application separately and record any Windows UIPI/input-boundary limitation rather than treating it as ordinary desktop behavior.

## macOS

- [ ] Install both generated architecture bundles on matching hardware where available (Apple Silicon and Intel).
- [ ] Verify the app bundle passes the expected community/ad-hoc signature validation for the build pipeline.
- [ ] Menu-bar icon/menu works and closing Settings keeps the utility running.
- [ ] Text/image monitoring, deduplication, pause/resume, persistence, and clear behavior match Windows.
- [ ] Quick Preview appears without taking focus from the current app.
- [ ] Without Accessibility permission, the app stays alive and explains that Tab hold/global wheel/keyboard capture is unavailable.
- [ ] Without Accessibility permission, configure a modifier-based switcher shortcut and verify sticky mode remains usable.
- [ ] Grant Accessibility permission, restart Clipboard Preview, then verify short Tab replay plus hold Tab → wheel/Previous/Next keys → release and `Cmd+V`.
- [ ] Copy a screenshot/image, verify thumbnail, delayed full preview, selection, and clipboard restore.
- [ ] Rebinding Previous/Next and conventional shortcuts works; conflicts are rejected.
- [ ] Multi-monitor/Retina positioning stays on the cursor's active display.
- [ ] Launch at login works after enabling it; manual launch still shows Settings.
- [ ] Explicit Quit terminates the background process.

## Linux x64

- [ ] Install the `.deb` on an Ubuntu/Debian-family test system and separately launch the AppImage.
- [ ] Default `Ctrl+Alt+J` opens the sticky Clipboard Switcher; plain Tab remains untouched.
- [ ] Wheel, configured Previous/Next keys, Enter, and Escape work while the focused sticky overlay is open.
- [ ] Attempting to configure plain Tab or hold/release interaction is rejected with a clear settings error rather than producing a broken mode.
- [ ] Text and image clipboard monitoring/restoration work under the desktop session being tested.
- [ ] Tray behavior is verified on a desktop with AppIndicator support; missing tray support does not terminate startup.
- [ ] GitHub links use `xdg-open` and the Settings WebView remains in the application.
- [ ] AppImage starts with the libraries expected by the Ubuntu 22.04 CI baseline; `.deb` dependency installation is clean on the target distribution.

## Official updater

These checks apply only after the official updater public/private key has been configured. Source/fork builds must show that updates are disabled and must not need any signing secret.

- [ ] Source build shows `Source / development build` in About and performs no automatic official update request.
- [ ] Official build shows `Official release build` and `Check for updates` when updater initialization succeeds.
- [ ] With no newer release, a manual check reports `Up to date` without an error popup.
- [ ] Network/GitHub failure during check leaves the app running and reports a recoverable error.
- [ ] A newer published release shows an in-app banner; native notification failure does not break the banner/update path.
- [ ] `Update now` reports progress and rejects modified/invalid-signature updater artifacts.
- [ ] Windows update installation completes through the configured NSIS updater path.
- [ ] macOS updates use the correct architecture-specific updater archive/signature.
- [ ] Linux AppImage update uses the signed AppImage updater artifact; `.deb` remains a manual package path.
- [ ] A draft/partial GitHub Release is never returned as the latest updater release.

## History/resource sanity

- [ ] Default 40-item history remains responsive with a mixed text/image workload.
- [ ] Count limit removes the oldest items first.
- [ ] Image-heavy history remains bounded by the internal payload budget rather than growing indefinitely.
- [ ] Persisted history writes are debounced during rapid copy operations.
- [ ] Diagnostics and standard logs do not intentionally contain copied text or image payloads.
- [ ] Idle clipboard polling does not create a network loop.

Capture CPU/memory measurements only from a real build/environment; do not invent performance numbers in documentation.
