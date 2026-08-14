# Clipboard Preview

**Your clipboard, one shortcut away.**

Instantly preview what you copied and jump back through recent clipboard history without leaving the app you're working in.

**Windows 10/11 · macOS 11+ · Tauri 2 · Rust · React**

[![CI](https://github.com/SirPaul-code/ClipboardPreview/actions/workflows/ci.yml/badge.svg)](https://github.com/SirPaul-code/ClipboardPreview/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/SirPaul-code/ClipboardPreview?display_name=tag)](https://github.com/SirPaul-code/ClipboardPreview/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Demo

<p align="center">
  <img src="assets/demo.gif" alt="Clipboard Preview demo recording placeholder" width="900">
</p>

> `assets/demo.gif` is intentionally an honest placeholder. Replace it with the real product recording; the repository does not ship simulated demo footage.

## Why Clipboard Preview?

Normal copy/paste is fast until you need to check what you copied or go back one item. Clipboard Preview keeps that recovery path short: summon a tiny overlay, scroll to an older item, release, then paste normally.

The core fast path is:

```text
Copy A → Copy B → hold history shortcut → scroll once → release → paste → A
```

No accounts, cloud sync, analytics, or remote clipboard API are involved.

## Features

- **Instant Quick Preview** — see the active text clipboard in a small, non-focus-stealing overlay.
- **Fast recent history** — navigate the full configured history while only rendering a compact visible window.
- **Hold → scroll → release** — select with a global shortcut and mouse wheel without clicking the overlay.
- **Sticky fallback mode** — press the shortcut, scroll or use arrow keys, press Enter; Escape cancels.
- **Configurable shortcuts** — record shortcuts directly in the settings UI; conflicts are rejected.
- **Local-first history** — memory-only by default, with optional local JSON persistence and clear-on-exit.
- **Tray / menu bar utility** — settings, history, pause/resume, clear, startup, about, and quit actions.
- **Cross-monitor overlays** — positioning follows the cursor's current monitor and work area.
- **System/light/dark themes** — plus opacity, spacing, radius, font size, animation speed, and reduced motion.
- **No clipboard logging** — log messages never intentionally include clipboard contents.

### Clipboard formats

Version 1 focuses deliberately on **plain text, URLs, code-like text, and multiline text**. The history model is typed so image/file/rich-text support can be added without putting native clipboard logic into React.

## Download

Download the latest compiled installers from **[GitHub Releases](https://github.com/SirPaul-code/ClipboardPreview/releases/latest)**.

Typical release assets:

- Windows x64: NSIS `.exe` and/or MSI installer produced by Tauri
- macOS Apple Silicon: `.dmg`
- macOS Intel: `.dmg`

The public community builds are unsigned unless signing credentials are configured by the maintainer. Windows SmartScreen or macOS Gatekeeper may therefore warn before first launch. The project does not contain fake certificates or signing secrets.

## Default shortcuts

| Action | Windows | macOS |
| --- | --- | --- |
| Quick Preview | `Ctrl + Alt + K` | `Control + Option + K` |
| Clipboard History | `Ctrl + Alt + J` | `Control + Option + J` |
| Open Settings | `Ctrl + Alt + P` | `Control + Option + P` |
| Pause / Resume | `Ctrl + Alt + Shift + P` | `Control + Option + Shift + P` |

The defaults avoid normal copy/paste, the Windows clipboard manager, Spotlight, and common paste-without-formatting shortcuts. Every shortcut is configurable from **Settings → Shortcuts**.

## How it works

### Quick Preview

Press the Quick Preview shortcut. Clipboard Preview reads the newest local history item and shows a frameless overlay on the monitor under the cursor. It is configured as non-focusable so the application you are working in stays active.

### Clipboard History

With **Hold → scroll → release** mode:

1. press and hold the history shortcut;
2. scroll the global mouse wheel to move through history;
3. release the shortcut;
4. the selected text becomes the active OS clipboard;
5. paste normally with `Ctrl+V` / `Cmd+V`.

On macOS, global mouse-wheel observation requires **Accessibility** permission. Clipboard Preview detects the permission state, explains why it is needed, and keeps the sticky selector usable when permission is unavailable.

### Clipboard monitoring

The app uses a conservative configurable poll interval instead of a tight loop. It hashes consecutive text values for deduplication, records nothing while paused, and marks application-generated clipboard writes so selecting history does not create feedback-loop duplicates.

## Customization

The settings app separates persistent configuration from transient overlay state. Available controls include:

- startup, hidden launch, tray/menu visibility, pause/resume;
- shortcut recording;
- preview character/line limits, width, font size, wrapping, position, and auto-hide delay;
- history size, visible rows, scroll direction, wrapping, ordering behavior, persistence, and interaction mode;
- system/light/dark appearance, overlay opacity, radius, spacing, animation speed, and reduced motion;
- clipboard poll interval, debug logging, clear history, and reset to defaults.

Settings are stored in the operating system's application data directory with a `configVersion` field so future migrations have a clean path. Corrupt settings/history files are ignored safely instead of crashing startup.

## Screenshots

The repository is ready for real screenshots and the demo GIF. No mock screenshot is presented as a shipped build.

## Installation

### Windows

1. Open the latest GitHub Release.
2. Download the Windows x64 installer.
3. Run it and follow the installer prompts.
4. If the unsigned community build triggers SmartScreen, review the publisher/source before choosing to continue.

### macOS

1. Open the latest GitHub Release.
2. Download the Apple Silicon (`arm64`) or Intel (`x64`) DMG for your Mac.
3. Drag Clipboard Preview to Applications.
4. If Gatekeeper blocks an unsigned community build, use macOS Privacy & Security to review and allow it.
5. Grant Accessibility permission only if you want global wheel selection in hold/release mode.

## Build from source

### Prerequisites

- Node.js **20.19+** (Node 22 LTS recommended for CI parity)
- Rust stable via `rustup`
- Tauri 2 platform prerequisites
- Windows: Microsoft C++ Build Tools and WebView2
- macOS: Xcode Command Line Tools

Then:

```bash
git clone https://github.com/SirPaul-code/ClipboardPreview.git
cd ClipboardPreview
npm ci
npm run tauri dev
```

Production bundle:

```bash
npm run tauri build
```

Quality checks:

```bash
npm run check
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

## Architecture

```text
Native / Rust
├── clipboard.rs       clipboard polling + internal-write suppression
├── history.rs         typed history, hashing, limits, classification
├── shortcuts.rs       configurable global shortcuts + press/release events
├── global_input.rs    platform global wheel listener
├── selection.rs       selection navigation + clipboard activation
├── overlays.rs        multi-monitor positioning + window lifecycle
├── settings_store.rs  local settings/history persistence
├── permissions.rs     macOS Accessibility state
├── tray.rs            Windows tray / macOS menu-bar commands
└── commands.rs        narrow Tauri IPC boundary

React / TypeScript
├── SettingsApp        normal configuration window
├── QuickPreview       non-focusable one-shot overlay
├── HistoryOverlay     selector UI + sticky keyboard fallback
└── typed backend wrappers / shortcut recorder / reusable controls
```

System-level clipboard and shortcut responsibilities stay in Rust; React does not own native clipboard monitoring.

## Privacy

**Clipboard Preview processes clipboard content locally on your device.**

By default history is memory-only. Optional persisted history is written to local app storage. There is no account system, analytics, telemetry, cloud sync, remote API, or network request loop. Production logging intentionally avoids clipboard contents.

Clipboard data can be sensitive. Review the persistence and clear-on-exit settings for your threat model, and see [SECURITY.md](SECURITY.md).

## Testing

Deterministic Rust tests cover history deduplication, history limits, classification, settings normalization/serialization, item promotion, and selector boundary behavior. CI also compiles the native Tauri application on Windows and macOS.

Some desktop interactions require real foreground applications, real clipboard ownership, global input, multiple monitors, and OS permission dialogs. Those are explicitly listed in [docs/manual-testing.md](docs/manual-testing.md) rather than being falsely marked as tested by headless CI.

## Known limitations

- Version 1 records text clipboard content only.
- Hold/release global wheel selection requires macOS Accessibility permission; sticky mode does not.
- The clipboard watcher uses low-frequency polling for portable behavior in v1 rather than platform-specific clipboard change notifications.
- Community release artifacts are unsigned until real Windows/Apple signing credentials are configured.
- The project does not attempt unreliable “password detection.” Per-application exclusions and sensitive-content rules are future work.

## Roadmap

Possible follow-up work after the core interaction remains stable:

- image clipboard previews;
- file clipboard history;
- pinned items and search;
- per-application exclusions;
- temporary/sensitive clipboard mode;
- export/import settings.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, code style, test expectations, and pull-request guidance.

Release maintainers should read [docs/releasing.md](docs/releasing.md).

## License

[MIT](LICENSE)
