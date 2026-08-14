# Contributing to Clipboard Preview

Thanks for improving Clipboard Preview. The project intentionally keeps the core small: native clipboard/OS responsibilities belong in Rust, UI responsibilities belong in React, and new dependencies need a concrete benefit.

## Development setup

1. Install the Tauri 2 prerequisites for your platform.
2. Install Node.js 20.19+ and Rust stable.
3. Fork/clone the repository.
4. Install dependencies and start the app:

```bash
npm install
npm run tauri dev
```

## Before opening a pull request

```bash
npm run check
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

For clipboard behavior, overlays, global shortcuts, tray/menu bar, startup, or permissions, also run the relevant checklist in `docs/manual-testing.md`. Never claim a platform was tested if it was not.

## Code style

- Keep TypeScript typed and components focused.
- Keep native clipboard, shortcut, window, and OS integration logic out of React components.
- Prefer small Rust modules with explicit responsibilities.
- Do not log clipboard contents.
- Avoid telemetry, network services, accounts, or cloud storage.
- Add deterministic tests for deterministic behavior; use documented manual validation for OS integration.

## Pull requests

Use a focused branch such as `fix/history-dedup` or `feat/image-preview`. Keep commits meaningful and avoid generated build output. Explain what changed, why, how it was tested, and which platforms were actually exercised.

## Bugs and features

Use the repository templates. Before attaching logs or screenshots, remove clipboard data, credentials, tokens, and other private material. For features, describe the user problem first; speed, reliability, privacy, and a small always-running footprint are primary design constraints.
