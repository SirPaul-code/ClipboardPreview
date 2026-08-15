# Changelog

All notable changes to Clipboard Preview are documented here. The project follows semantic versioning and a Keep a Changelog-style structure.

## [1.3.0] - Unreleased

### Added

- Linux x64 support with AppImage and Debian package targets.
- Source-safe updater architecture: local clones and forks never require Clipboard Preview signing secrets and do not query official releases for updates.
- Official-build update checks with an in-app availability banner, manual check button, download progress, and native update notification where available.
- Signed Tauri updater artifact pipeline using GitHub Releases and `latest.json`; publishing is gated until every platform release job succeeds.
- Live Clipboard Switcher appearance editor in Settings with the real switcher layout as the preview surface.
- Per-layer switcher text customization for header title/subtitle/meta, item type/content/date, detail content/meta, and footer.
- Switcher surface customization for panel, rows, selected state, borders, row height, and thumbnail size.
- Configurable Previous/Next navigation bindings, defaulting to `ArrowUp` and `ArrowDown`.
- Keyboard navigation in hold mode on Windows/macOS; configured navigation keys are consumed only while the Clipboard Switcher is open.

### Changed

- Clipboard list metadata now shows the item creation date/time instead of image dimensions or character counts.
- Linux defaults to a modifier-based `Ctrl+Alt+J` switcher shortcut and sticky interaction rather than pretending to support the native Tab hold hook.
- CI now validates fresh-clone builds and distributable bundles on Windows x64, macOS Apple Silicon, macOS Intel, and Linux x64 using committed npm/Cargo lockfiles.
- Windows installers embed the WebView2 bootstrapper so missing WebView2 is handled by the installer rather than becoming an unexplained first-launch failure.
- macOS community CI/release builds use ad-hoc signing when an Apple Developer identity is not configured.

### Fixed

- GitHub, Releases, issue, author, and license links no longer navigate the Settings WebView away from Clipboard Preview. They open the system browser, and native WebViews reject external navigation as a second line of defense.
- Updater, notification, tray, shortcut, and global-input integration failures are nonfatal startup capabilities and surface through warnings/diagnostics instead of intentionally terminating the process.

## [1.2.1] - 2026-08-14

### Changed

- Clipboard Switcher height follows the actual number of visible history entries rather than reserving empty rows.
- Large preview mode has more usable preview space and scales against the active monitor work area.
- Compact mode hugs the real list height and delayed image previews adapt to the available overlay geometry.

## [1.2.0] - 2026-08-14

### Changed

- Reworked the UI into a flatter, restrained desktop-utility design without glassmorphism, decorative gradients, or card-in-card styling.
- Refined the Clipboard Switcher row layout and compact/large-preview presentation.
- Added About attribution and project links.

## [1.1.0] - 2026-08-14

### Added

- Unified Clipboard Switcher for recent text and images.
- Default `Tab` tap-vs-hold gesture: a short tap is replayed to the foreground application, while holding Tab opens the switcher; scroll changes selection and releasing selects it.
- Clipboard image history with compressed PNG payloads, list thumbnails, delayed full previews, and image restore back to the system clipboard.
- Local crash diagnostics with an early panic hook, bootstrap log, previous-crash indicator, copyable report, and diagnostics-folder shortcut.
- History memory budgeting in addition to item-count limits; large configured histories display a performance warning.
- Settings migration from v1 defaults to the new v2 switcher defaults.

### Changed

- Redesigned Settings and overlay surfaces with a quieter minimalist layout, tighter hierarchy, and a split list/preview switcher.
- Default history size is 40 items, with a configurable hard maximum of 250 items and an approximately 192 MiB in-memory payload budget.
- Clipboard persistence remains opt-in; when enabled, image history is stored locally as compressed PNG data.
- Manual launches always show Settings. `Start hidden` is honored only for the real operating-system autostart launch.
- Optional native integrations report warnings instead of being allowed to terminate startup.

### Fixed

- Fixed the confirmed startup crash where a configured WebView could invoke `get_settings` before `AppState` had been managed. Configured WebViews are now created explicitly only after backend state initialization.
- Removed release `panic = "abort"` behavior so recoverable Rust panics are not deliberately converted into immediate process aborts.
- Native global-input callbacks defensively catch Rust panics before they can cross the Windows/macOS callback boundary.
- Removed the one-off v1.0.1 publication workflow; releases are again produced only by the normal tag-driven workflow.

## [1.0.1] - 2026-08-14

### Fixed

- Prevented the application from exiting during startup when one or more global shortcuts cannot be registered on Windows or macOS.
- Global-shortcut registration now rolls back partial registrations instead of leaving a half-registered shortcut set.
- Startup integration failures are retained as diagnostic warnings instead of being treated as fatal initialization errors.
- The settings window stays visible when startup integration warnings occur so shortcut conflicts can be corrected.

## [1.0.0] - 2026-08-14

### Added

- Quick clipboard preview overlay with configurable size, text limits, wrapping, position, and auto-hide.
- Recent clipboard history with deduplication, configurable limits, optional persistence, and clear-on-exit.
- Hold → scroll → release history interaction plus sticky keyboard/mouse fallback mode.
- Configurable global shortcuts for preview, history, settings, and pause/resume.
- Windows tray and macOS menu-bar controls.
- Launch-at-startup integration.
- Multi-monitor cursor-relative overlay positioning.
- System/light/dark appearance options and reduced motion groundwork.
- macOS Accessibility detection for global wheel capture.
- First-run experience, settings dashboard, privacy messaging, and local-only architecture.
- Rust tests for deterministic history/settings/selection logic.
- Windows/macOS CI and tag-driven GitHub Release workflow.
- Contributor, security, issue, PR, manual-testing, and release documentation.
