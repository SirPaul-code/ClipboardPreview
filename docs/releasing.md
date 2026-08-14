# Releasing Clipboard Preview

The release path is deliberately small and uses the same source that contributors build locally.

## 1. Validate the release branch

- Open a PR into `main` and require the complete CI matrix to pass.
- CI must pass frontend checks plus Rust test/clippy/native build on Windows x64, macOS Apple Silicon, and macOS Intel.
- `npm run check` also enforces the startup-order invariant: configured WebViews must remain `create=false`, and release `panic = "abort"` is forbidden.
- Complete the applicable checklist in `docs/manual-testing.md` on platforms you can genuinely test.
- Review unsigned-build limitations if signing credentials are not configured.

## 2. Update the version

Keep `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` synchronized. `npm run check:version` rejects a mismatch. Update `CHANGELOG.md` in the same release branch.

## 3. Merge, then tag the exact main commit

After the PR is green and merged:

```bash
git checkout main
git pull --ff-only
git tag vX.Y.Z
git push origin vX.Y.Z
```

Do not add one-off publish workflows for normal releases. `.github/workflows/release.yml` is the canonical tag-driven release path.

## 4. GitHub Actions release build

The `vX.Y.Z` tag starts `.github/workflows/release.yml`. It checks version/tag consistency, builds Windows x64 plus macOS Apple Silicon and Intel bundles, creates the GitHub Release, and attaches readable installer names.

The release build must come from the tagged merged commit; do not point a release at an older build or copy assets between versions.

## 5. Verify artifacts

Before announcing the release:

- verify the GitHub Release is non-draft and points at the expected tag/commit;
- verify Windows MSI/NSIS and both macOS architecture assets are attached;
- download and smoke-install artifacts on every platform you can genuinely access;
- record untested runtime platforms as CI-compiled/bundled, not manually verified.

A successful CI or bundle build is not the same as a manual installation/runtime test.

## Signing

No signing certificate, Apple Developer identity, notarization secret, or private key is committed. If signing is configured later, provide credentials through GitHub Actions secrets according to Tauri's official signing guidance. Until then, community builds may trigger Windows SmartScreen or macOS Gatekeeper warnings.
