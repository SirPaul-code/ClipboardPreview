# Clipboard Preview

**Your clipboard, one shortcut away.**

Clipboard Preview is a lightweight local clipboard switcher for **Windows 10/11, macOS 11+, and Linux x64**. It keeps recent text and images close to the app you are already using, without accounts, analytics, cloud sync, or a remote clipboard service.

<p align="center">
  <img src="assets/cc7261d8-8fd0-492a-8aa6-e5a7d65f3394.gif" alt="Clipboard Preview demo" width="1100">
</p>

**Tauri 2 · Rust · React · TypeScript**

[![CI](https://github.com/SirPaul-code/ClipboardPreview/actions/workflows/ci.yml/badge.svg)](https://github.com/SirPaul-code/ClipboardPreview/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/SirPaul-code/ClipboardPreview?display_name=tag)](https://github.com/SirPaul-code/ClipboardPreview/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## The fast path

On Windows and macOS with native input access, the default interaction is:

```text
copy several things → hold Tab → wheel / ↑ ↓ → release → paste
```

**No clicking is required.** Hold `Tab` to open the switcher, use the mouse wheel or the configurable Previous/Next keys (`ArrowUp` / `ArrowDown` by default), and release `Tab` when the item you want is selected. That item is immediately restored to the system clipboard and is ready to paste.

A short Tab press is replayed to the foreground application, so Tab continues to behave normally in editors, browsers, forms, and other desktop apps. Holding it for roughly 180 ms opens Clipboard Preview instead.

You can also simply hold the switcher open to inspect recent clipboard content and release without navigating. Clicking items remains available if you prefer it.

### Linux interaction

Linux uses the reliable global-shortcut/sticky path instead of pretending to provide the same low-level Tab interception as Windows/macOS:

```text
Ctrl+Alt+J → wheel / ↑ ↓ → Enter
```

The switcher shortcut and Previous/Next keys are configurable.

## Features

- **Unified Clipboard Switcher** — recent text, URLs, code, multiline text, and images in one overlay.
- **Mouse + keyboard browsing** — scroll wheel and configurable Previous/Next keys use the same selection model.
- **Scoped navigation keys** — `ArrowUp` / `ArrowDown` are consumed only while the switcher is open; they behave normally everywhere else.
- **Image history** — local PNG history with small list thumbnails and delayed larger previews.
- **Compact or large preview layout** — keep the switcher row-only or show a larger detail panel.
- **Dynamic sizing** — the overlay grows with the real number of visible history entries instead of reserving empty rows.
- **Creation timestamps** — rows show when an item was captured rather than low-value character counts or image dimensions.
- **Live Switcher customization** — Settings contains a real preview of the switcher and lets you tune text sizes/colors, row geometry, thumbnail size, panel/row/selection/border colors, opacity, radius, and more.
- **Bounded history** — configurable 1–250 items plus an approximately 192 MiB in-memory payload budget for image-heavy sessions.
- **Local-first persistence** — history is memory-only by default. Optional persistence stays in local app storage.
- **Crash-safe diagnostics** — early bootstrap/panic diagnostics remain available even if the normal logger is not useful yet.
- **Tray / menu bar utility** — open settings/history, pause/resume, clear, and quit without keeping Settings visible.
- **Multi-monitor overlays** — positioning follows the active cursor monitor and work area.
- **Production updater architecture** — official releases can check GitHub Releases for signed updates; source/fork builds intentionally do not.

## Download

Download compiled builds from **[GitHub Releases](https://github.com/SirPaul-code/ClipboardPreview/releases/latest)**.

Release targets:

- **Windows x64** — MSI + NSIS setup EXE
- **macOS Apple Silicon** — DMG + app/updater archive
- **macOS Intel** — DMG + app/updater archive
- **Linux x64** — AppImage + Debian package

The Windows installer embeds the Microsoft WebView2 bootstrapper, so a missing WebView2 runtime is handled during installation instead of becoming a mysterious first-launch failure.

## Default controls

| Action | Windows / macOS | Linux |
| --- | --- | --- |
| Open Clipboard Switcher | hold `Tab` | `Ctrl + Alt + J` |
| Previous item | `ArrowUp` | `ArrowUp` |
| Next item | `ArrowDown` | `ArrowDown` |
| Quick Preview | `Ctrl + Alt + K` | `Ctrl + Alt + K` |
| Open Settings | `Ctrl + Alt + P` | `Ctrl + Alt + P` |
| Pause / Resume | `Ctrl + Alt + Shift + P` | `Ctrl + Alt + Shift + P` |

Previous/Next bindings are active only while the switcher is open. The conventional application shortcuts remain global shortcuts.

### macOS Accessibility

The default Tab hold gesture plus global wheel/keyboard interception requires **Accessibility** permission on macOS. Clipboard Preview reports missing permission as a recoverable capability instead of crashing. A modifier-based sticky shortcut remains available as the fallback.

## Clipboard formats

Current releases support:

- plain text;
- URLs;
- code-like text;
- multiline text;
- raster clipboard images exposed by the platform clipboard API.

Images are normalized to compressed PNG for bounded local history storage. The list receives a small thumbnail; the full image is decoded for the UI only when a selected image has remained selected long enough to need a detailed preview.

File-copy history and rich-document formats are not implemented yet.

## History and memory behavior

Clipboard Preview is designed as a fast utility rather than an archive.

- default history: **40 items**;
- configurable hard cap: **250 items**;
- performance warning above **150 items**;
- approximate in-memory history payload budget: **192 MiB**;
- very large entries are rejected instead of allowing one clipboard item to grow the process without bound;
- when limits are exceeded, the oldest entries are discarded first;
- persistence is disabled by default and remains local when enabled.

## Customize the switcher

Open **Settings → Appearance → Clipboard Switcher**.

The preview there uses the same CSS-variable contract as the real overlay. You can independently change:

- header title;
- header subtitle;
- item count / shortcut text;
- item type label;
- item content;
- item date/time;
- large-preview content;
- large-preview metadata;
- footer hint;
- panel, row, selection, border, and selected-border colors;
- row height;
- thumbnail size.

Each text layer has its own **font size and color** control. Changes are visible in the preview immediately and are persisted through the normal settings model.

## Updates

Official Clipboard Preview builds use Tauri's signed updater artifacts hosted on GitHub Releases.

The production flow is:

```text
app starts
  ↓
background update check
  ↓
new signed release? ── no → nothing shown
  ↓ yes
Settings banner / native notification
  ↓
Update now
  ↓
download + signature verification + install
```

A release is kept as a GitHub **draft** while Windows, both macOS architectures, and Linux are being built. It is published only after every release job succeeds, so running clients do not see a half-built update.

Updater signing is separate from Windows Authenticode or Apple Developer signing.

### Source builds do not require release secrets

A normal clone or fork can run:

```bash
npm ci
npm run tauri dev
```

or:

```bash
npm ci
npm run tauri build
```

without `TAURI_SIGNING_PRIVATE_KEY` or any Clipboard Preview release credential.

Source/development builds identify themselves as such in **Settings → About**, do not query official Clipboard Preview updates, and do not generate updater artifacts. The private updater key is used only by the official tag-driven GitHub Actions release workflow.

## Startup reliability

The native backend state is initialized **before any configured WebView is created**. Frontend IPC cannot intentionally run against an uninitialized `AppState`.

Optional integrations are treated as recoverable capabilities:

- updater/network failure;
- notifications;
- tray/menu bar creation;
- global shortcut conflicts;
- macOS Accessibility;
- native input capture.

A failure in one of those integrations should surface through Settings/Diagnostics rather than intentionally terminating the application.

External project links also open in the **system browser**. The Settings WebView rejects external navigation, preventing a GitHub page from replacing the app UI.

## Diagnostics

**Settings → Advanced → Diagnostics** can:

- display the bootstrap / previous-panic report;
- copy the report for a developer;
- open the diagnostics directory;
- dismiss a reviewed previous-crash marker.

The panic hook is installed before the Tauri application is built, so failures that happen before the normal log plugin is useful can still leave a local trace. Clipboard contents are not intentionally included in diagnostics.

## Installation

### Windows

1. Open the latest GitHub Release.
2. Download the Windows x64 MSI or setup EXE.
3. Install normally. WebView2 is bootstrapped when missing.
4. Windows SmartScreen can still warn while Authenticode signing is not configured.

### macOS

1. Download the Apple Silicon (`aarch64`) or Intel (`x64`) DMG.
2. Install Clipboard Preview in Applications.
3. Community builds use ad-hoc signing until Developer ID/notarization credentials are configured, so macOS may still require approval in Privacy & Security.
4. Grant Accessibility if you want the native Tab hold + global wheel/keyboard flow.

### Linux

1. Download the x64 AppImage or `.deb`.
2. AppImage users may need to make the file executable before launching.
3. Use `Ctrl+Alt+J` by default to open the sticky Clipboard Switcher, then wheel/arrow through history and press Enter.

## Build from source

### Prerequisites

- Node.js **20.19+** (Node 22 LTS recommended for CI parity)
- Rust stable via `rustup`
- Tauri 2 platform prerequisites
- Windows: Microsoft C++ Build Tools; installer/runtime uses WebView2
- macOS: Xcode Command Line Tools
- Linux: WebKitGTK 4.1 and the standard Tauri Linux build dependencies

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
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

## CI / reproducibility

Pull requests are validated from the committed lockfiles and build **real distributable bundles** on:

- Windows x64;
- macOS Apple Silicon;
- macOS Intel;
- Ubuntu 22.04 / Linux x64.

This is deliberately stricter than merely compiling one developer machine. CI still does not replace interactive runtime testing for permissions, foreground-app input, clipboard ownership, installer UX, and multi-monitor behavior.

## Architecture

```text
Native / Rust
├── diagnostics.rs     early panic + bootstrap diagnostics
├── clipboard.rs       text/image polling + clipboard restore
├── history.rs         bounded typed history + PNG thumbnails/payloads
├── global_input.rs    Tab hold + scoped wheel/keyboard navigation on Win/mac
├── shortcuts.rs       conventional configurable global shortcuts
├── selection.rs       selector navigation + activation
├── overlays.rs        dynamic multi-monitor window sizing/positioning
├── settings_store.rs  migration + local/debounced persistence
├── permissions.rs     macOS Accessibility state
├── updates.rs         official-build-only signed update flow
├── tray.rs            tray/menu-bar commands
└── commands.rs        Tauri IPC boundary + system-browser links

React / TypeScript
├── SettingsApp                settings, updates, diagnostics
├── SwitcherAppearanceEditor   live real-layout customization preview
├── HistoryOverlay             unified list/detail Clipboard Switcher
├── QuickPreview               optional one-shot preview
└── typed IPC / formatting / shortcut helpers
```

System clipboard, native lifecycle, persistence, release-update trust, and low-level input responsibilities stay in Rust. React renders state and requests explicit actions through the IPC boundary.

## Privacy

**Clipboard Preview processes clipboard content locally on your device.**

There is no account system, telemetry, analytics, cloud clipboard sync, or remote clipboard API. History persistence is off by default. When enabled, persisted history is written to the application's local data directory and image entries include locally encoded PNG data.

The only intentional network request in an **official release build** is the update check against this project's GitHub Release metadata. Source/fork builds keep updater checks disabled.

Clipboard data can be sensitive. Choose persistence and clear-on-exit settings according to your threat model and see [SECURITY.md](SECURITY.md).

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

Release maintainers should read [docs/releasing.md](docs/releasing.md). The release documentation describes the updater key boundary and the four-platform release gate.

## Credits

Created by **Pavol Duplinsky** · [@SirPaul-code](https://github.com/SirPaul-code)

## License

[MIT](LICENSE)
