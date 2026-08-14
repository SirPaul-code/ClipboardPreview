# Releasing Clipboard Preview

The release path is deliberately small.

## 1. Validate

- CI is green on `main`.
- Complete the applicable checklist in `docs/manual-testing.md` on platforms you can genuinely test.
- Review unsigned-build limitations if signing credentials are not configured.

## 2. Update the version

Keep `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` synchronized. `npm run check:version` rejects a mismatch. Update `CHANGELOG.md` in the same release commit.

## 3. Commit and tag

```bash
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json CHANGELOG.md
git commit -m "release: prepare vX.Y.Z"
git tag vX.Y.Z
git push origin main
git push origin vX.Y.Z
```

## 4. GitHub Actions

The tag starts `.github/workflows/release.yml`. It builds Windows x64 plus macOS Apple Silicon and Intel bundles, creates the GitHub Release, and attaches readable installer names.

## 5. Verify

Download generated assets and smoke-install them before announcing the release. A successful CI bundle build is not the same as a manual installation test.

## Signing

No signing certificate, Apple Developer identity, notarization secret, or private key is committed. If signing is configured later, provide credentials through GitHub Actions secrets according to Tauri's official signing guidance. Until then, community builds may trigger Windows SmartScreen or macOS Gatekeeper warnings.
