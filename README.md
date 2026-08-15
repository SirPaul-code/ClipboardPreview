# Clipboard Preview

**Hold Tab. Pick anything. Keep working.**

Clipboard Preview is a lightweight local clipboard switcher for **Windows 10/11 and macOS 11+**. It keeps recent text and images close to the app you are already using, without accounts, analytics, cloud sync, or a remote clipboard service.

<p align="center">
  <img src="assets/cc7261d8-8fd0-492a-8aa6-e5a7d65f3394.gif" alt="Clipboard Preview demo" width="1100">
</p>

**Tauri 2 · Rust · React · TypeScript**

[![CI](https://github.com/SirPaul-code/ClipboardPreview/actions/workflows/ci.yml/badge.svg)](https://github.com/SirPaul-code/ClipboardPreview/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/SirPaul-code/ClipboardPreview?display_name=tag)](https://github.com/SirPaul-code/ClipboardPreview/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## The fast path

The default v1.1 interaction is intentionally small:

```text
copy several things → hold Tab → scroll → release → paste
```

A **short Tab press is replayed to the foreground application**, so Tab keeps behaving normally in editors, browsers, forms, and other everyday desktop apps. Holding Tab for roughly 180 ms opens the Clipboard Switcher instead.

The switcher combines the old one-item preview and history workflow into one surface: recent text and images appear in a compact list, scrolling changes the selection, and releasing the hold-mode shortcut restores the selected item to the system clipboard.

## Features

- **Unified Clipboard Switcher** — recent text, URLs, code, multiline text, and images in one overlay.
- **Tap-vs-hold Tab gesture** — short Tab remains a normal Tab; hold Tab to open history, scroll to choose, release to select.
- **Image history** — images are compressed locally as PNG, represented by small thumbnails, and restored to the OS clipboard like text.
- **Delayed image preview** — pausing on an image loads the full preview only after a short delay; scrolling away cancels that preview path.
- **Quick Preview** — an optional separate one-shot preview shortcut remains available.
- **Sticky fallback** — use a normal shortcut, wheel/arrow keys, Enter, and Escape when hold/release input is unavailable or undesired.
- **Bounded history** — configurable 1–250 items plus an approximately 192 MiB in-memory payload budget for image-heavy sessions.
- **Local-first persistence** — history is memory-only by default. Optional persistence stays in local app storage and includes compressed image data when enabled.
- **Crash-safe startup diagnostics** — bootstrap logging and panic reports are available inside Settings without intentionally logging clipboard contents.
- **Tray / menu bar utility** — settings, history, pause/resume, clear, startup, about, and quit actions.
- **Multi-monitor overlays** — positioning follows the cursor's active monitor and work area.
- **System/light/dark appearance** — restrained settings for opacity, radius, spacing, and reduced motion.

## Download

Download compiled installers from **[GitHub Releases](https://github.com/SirPaul-code/ClipboardPreview/releases/latest)**.

Release targets:

- **Windows x64** — Tauri NSIS `.exe` and MSI installer
- **macOS Apple Silicon** — `.dmg` / app bundle
- **macOS Intel** — `.dmg` / app bundle

Release artifacts and source builds use the **same Rust/React source and startup architecture**. CI compiles and tests that source on Windows x64, macOS Apple Silicon, and macOS Intel before a release is cut.

Community builds are unsigned unless real Windows/Apple signing credentials are configured. Windows SmartScreen or macOS Gatekeeper can therefore warn before first launch.

## Default shortcuts

| Action | Default |
| --- | --- |
| Clipboard Switcher | `Tab` hold (short taps remain normal Tab) |
| Quick Preview | `Ctrl + Alt + K` |
| Open Settings | `Ctrl + Alt + P` |
| Pause / Resume | `Ctrl + Alt + Shift + P` |

Every shortcut is configurable. If the Clipboard Switcher shortcut is changed away from `Tab`, it is registered as a conventional global shortcut instead of using the native tap-vs-hold Tab path.

### macOS Accessibility

The default Tab hold gesture and global wheel interception require **Accessibility** permission on macOS. Clipboard Preview reports the permission state instead of crashing. A modifier-based shortcut with sticky interaction remains the fallback when Accessibility is not granted.

## Clipboard formats

v1.1 supports:

- plain text;
- URLs;
- code-like text;
- multiline text;
- raster clipboard images exposed by the operating system clipboard APIs.

Images are normalized to compressed PNG for history storage. The list receives only a small thumbnail; the full image payload is requested by the UI only when a selected image has remained selected long enough to need a detailed preview.

File-copy history and rich document formats are not implemented yet.

## History and memory behavior

Clipboard Preview is meant to remain a small utility rather than an archive.

- default history: **40 items**;
- configurable hard item cap: **250**;
- UI warns when configured above **150 items**;
- approximate in-memory history payload budget: **192 MiB**;
- individual text entries above the internal safety limit are not added;
- very large images are rejected from history instead of letting one clipboard item grow the process without bound;
- when either the count or memory budget is exceeded, the **oldest** entries are discarded first.

Optional persistence is debounced rather than rewriting the history file on every clipboard poll. Persistence remains disabled by default for privacy.

## Startup reliability

The native backend state is initialized **before any configured WebView is created**. This ordering is important: frontend IPC must never be able to call a command before `AppState` exists.

Optional integrations such as tray creation, shortcut registration, permissions, and native global input are handled as recoverable capabilities. Failure in one of them should produce a warning/diagnostic state rather than intentionally killing the entire desktop process.

Manual application launches show Settings. `Start hidden` applies only to the real OS autostart invocation (`--hidden`).

## Diagnostics

**Settings → Advanced → Diagnostics** can:

- display the local bootstrap / previous-panic report;
- copy the report for a developer;
- open the diagnostics directory;
- dismiss a previous-crash marker after it has been reviewed.

The panic hook is installed before the Tauri application is built so failures that occur before the normal log plugin is fully useful still have a local trace. Clipboard contents are not intentionally included in diagnostics.

## Installation

### Windows

1. Open the latest GitHub Release.
2. Download the Windows x64 MSI or setup EXE.
3. Install and launch Clipboard Preview from the normal Windows app entry.
4. Review any SmartScreen warning if the community build is unsigned.

### macOS

1. Open the latest GitHub Release.
2. Download the Apple Silicon (`aarch64`) or Intel (`x64`) build for your Mac.
3. Install Clipboard Preview in Applications.
4. Review Gatekeeper if the community build is unsigned.
5. Grant Accessibility if you want the default Tab hold + global wheel interaction.

## Build from source

### Prerequisites

- Node.js **20.19+** (Node 22 LTS recommended for CI parity)
- Rust stable via `rustup`
- Tauri 2 platform prerequisites
- Windows: Microsoft C++ Build Tools and WebView2 Runtime
- macOS: Xcode Command Line Tools

```bash
git clone https://github.com/SirPaul-code/ClipboardPreview.git
cd ClipboardPreview
npm install
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
├── diagnostics.rs     early panic + bootstrap diagnostics
├── clipboard.rs       text/image polling + clipboard restore
├── history.rs         bounded typed history + PNG thumbnails/payloads
├── global_input.rs    Tab tap-vs-hold + global wheel interception
├── shortcuts.rs       conventional configurable global shortcuts
├── selection.rs       selector navigation + activation
├── overlays.rs        multi-monitor window positioning / payloads
├── settings_store.rs  migration + local/debounced persistence
├── permissions.rs     macOS Accessibility state
├── tray.rs            Windows tray / macOS menu-bar commands
└── commands.rs        Tauri IPC boundary

React / TypeScript
├── SettingsApp        settings + diagnostics
├── HistoryOverlay     unified list/detail Clipboard Switcher
├── QuickPreview       optional one-shot preview
└── typed IPC wrappers / controls / shortcut recorder
```

System clipboard, global input, persistence, and native lifecycle responsibilities stay in Rust. React renders state and requests explicit actions through the IPC boundary.

## Privacy

**Clipboard Preview processes clipboard content locally on your device.**

There is no account system, telemetry, analytics, cloud clipboard sync, or remote API loop. History persistence is off by default. When enabled, persisted history is written to the application's local data directory; image entries include locally encoded PNG data.

Clipboard data can be sensitive. Choose persistence and clear-on-exit settings according to your threat model and see [SECURITY.md](SECURITY.md).

## Testing

Rust tests cover history deduplication, count limits, text classification, image metadata/encoding paths, settings migration/normalization, promotion, and selector boundary math. Frontend checks compile and lint the React/TypeScript app.

CI performs native test, clippy, and release-mode no-bundle compilation on:

- Windows x64;
- macOS Apple Silicon;
- macOS Intel.

Real desktop behavior still requires real OS testing for foreground-app interaction, permissions, global input, clipboard ownership, installation, and multi-monitor behavior. These checks are tracked in [docs/manual-testing.md](docs/manual-testing.md).

## Platform boundaries

Clipboard Preview targets normal desktop applications on Windows and macOS. No desktop utility can promise interception/injection across every OS security boundary: Windows secure desktop/UAC and higher-integrity processes can restrict synthetic input, and macOS can restrict global input until Accessibility is granted. Those cases should fail as capabilities rather than crash the application.

Linux is not currently a release target.

## Roadmap

Potential follow-up work:

- file clipboard history;
- pinned items and search;
- per-application exclusions;
- temporary/sensitive clipboard modes;
- native clipboard-change notifications where they materially improve efficiency;
- signed Windows/macOS releases when signing infrastructure is configured.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, code style, test expectations, and pull-request guidance.

Release maintainers should read [docs/releasing.md](docs/releasing.md).

## License

[MIT](LICENSE)
