# Changelog

All notable changes to Clipboard Preview are documented here. The project follows semantic versioning and a Keep a Changelog-style structure.

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
