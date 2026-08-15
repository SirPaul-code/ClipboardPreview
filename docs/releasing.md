# Releasing Clipboard Preview

Clipboard Preview uses the same source for local builds and official releases, but **updater signing is intentionally release-only**. A fresh clone must build without any Clipboard Preview secrets.

## 1. Validate the release branch

- Open a PR into `main` and require the complete CI matrix to pass.
- CI must run from committed `package-lock.json` and `src-tauri/Cargo.lock` files.
- CI must pass frontend checks, Rust tests, strict clippy, and a real distributable bundle build on:
  - Windows x64 (`.msi` + NSIS setup);
  - macOS Apple Silicon (`.app` + `.dmg`);
  - macOS Intel (`.app` + `.dmg`);
  - Linux x64 (`.AppImage` + `.deb`).
- `npm run check:version` enforces synchronized versions, startup ordering, clone-safe updater configuration, and release-only updater-artifact generation.
- Complete the applicable checklist in `docs/manual-testing.md` on platforms you can genuinely test.

A successful CI bundle is evidence that the source and packaging pipeline build on that OS. It is not a substitute for a real interactive smoke test on the target machine.

## 2. Updater signing key

Tauri updater signatures are separate from Windows Authenticode and Apple Developer signing.

Generate the updater key once with a current Tauri CLI:

```bash
npm run tauri signer generate -- -w ~/.tauri/clipboard-preview-updater.key
```

This creates a private key and a public key. The rules are strict:

- **never commit the private key**;
- back it up securely; installed clients trust this key lineage for future updates;
- commit only the public key content to `src-tauri/updater.pub`;
- store the private key content in the GitHub Actions repository secret `TAURI_SIGNING_PRIVATE_KEY`;
- if the key has a password, store it in `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`; otherwise the secret may be empty.

`src-tauri/updater.pub` deliberately contains a placeholder until this one-time setup is complete. Official-release validation fails clearly while the placeholder/private secret is missing. Normal source builds do not fail and do not need either secret.

## 3. Official vs source builds

Normal clone/fork builds use only `src-tauri/tauri.conf.json` and **do not create updater artifacts**. The runtime sees `CLIPBOARD_PREVIEW_OFFICIAL_BUILD` as unset, so it does not register or query the official updater.

The tag-driven release workflow sets:

```text
CLIPBOARD_PREVIEW_OFFICIAL_BUILD=1
TAURI_SIGNING_PRIVATE_KEY=<GitHub Actions secret>
TAURI_SIGNING_PRIVATE_KEY_PASSWORD=<optional GitHub Actions secret>
```

and overlays `src-tauri/tauri.release.conf.json`, which enables `createUpdaterArtifacts`.

This keeps forks and local development independent from Clipboard Preview's release infrastructure.

## 4. Update the version

Keep these versions synchronized:

- `package.json`;
- root package entry in `package-lock.json`;
- `src-tauri/Cargo.toml`;
- package entry in `src-tauri/Cargo.lock`;
- `src-tauri/tauri.conf.json`.

Update `CHANGELOG.md` in the same release branch. `npm run check:version` rejects a mismatch between the primary version sources.

## 5. Merge, then tag the exact main commit

After the PR is green and merged:

```bash
git checkout main
git pull --ff-only
git tag vX.Y.Z
git push origin vX.Y.Z
```

Do not add one-off publish workflows for normal releases. `.github/workflows/release.yml` is the canonical tag-driven release path.

## 6. GitHub Actions release build

The `vX.Y.Z` tag starts `.github/workflows/release.yml`.

The workflow:

1. validates the version/tag/startup/signing invariants;
2. builds from the committed npm/Cargo lockfiles;
3. builds Windows x64, macOS Apple Silicon, macOS Intel, and Linux x64;
4. creates updater signatures and `latest.json`;
5. uploads all bundles to a **draft** GitHub Release;
6. publishes the release only after every platform job succeeds.

The updater endpoint is the stable GitHub URL:

```text
https://github.com/SirPaul-code/ClipboardPreview/releases/latest/download/latest.json
```

Keeping the release draft until every job is green prevents running clients from seeing an incomplete update.

## 7. Platform packaging

### Windows

- Build MSI and NSIS setup EXE.
- The installer embeds the Microsoft WebView2 bootstrapper and installs WebView2 when it is missing.
- Updater installation uses Tauri's passive Windows flow.
- Authenticode signing is a separate future credential; SmartScreen may still warn until it is configured.

### macOS

- Build Apple Silicon and Intel DMGs/app bundles separately.
- Community builds use ad-hoc code signing (`APPLE_SIGNING_IDENTITY=-`) when an Apple Developer identity is not configured.
- Proper Developer ID signing/notarization can be added later without changing the updater key lineage.

### Linux

- Build AppImage and Debian package on Ubuntu 22.04 x64 for a conservative glibc baseline.
- The Tauri updater uses the signed AppImage artifact. The `.deb` remains available as a normal package download.
- Linux uses a modifier-based sticky switcher workflow instead of claiming native global Tab-hold interception.

## 8. Verify artifacts

Before announcing a release:

- verify the GitHub Release is non-draft and points at the expected tag/commit;
- verify Windows MSI + NSIS, both macOS architecture assets, Linux AppImage + DEB, updater `.sig` files, and `latest.json` are attached;
- inspect `latest.json` and confirm its platform URLs/signatures correspond to the current release assets;
- download and smoke-install artifacts on every platform you can genuinely access;
- record untested runtime platforms as CI-compiled/bundled, not manually verified.

## Runtime failure policy

Network failure, GitHub outage, missing notifications, updater check failure, tray failure, shortcut conflict, or unavailable native-input integration must not intentionally terminate application startup. These are recoverable capability states and should be surfaced in Settings/Diagnostics.

The early panic/bootstrap diagnostic path remains independent of the normal log plugin so a failure before normal logging still leaves actionable local evidence.
