import fs from 'node:fs';

const pkg = JSON.parse(fs.readFileSync(new URL('../package.json', import.meta.url), 'utf8'));
const cargo = fs.readFileSync(new URL('../src-tauri/Cargo.toml', import.meta.url), 'utf8');
const tauri = JSON.parse(fs.readFileSync(new URL('../src-tauri/tauri.conf.json', import.meta.url), 'utf8'));
const releaseTauri = JSON.parse(fs.readFileSync(new URL('../src-tauri/tauri.release.conf.json', import.meta.url), 'utf8'));
const updaterPublicKey = fs.readFileSync(new URL('../src-tauri/updater.pub', import.meta.url), 'utf8').trim();

const rustVersion = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
const versions = { package: pkg.version, cargo: rustVersion, tauri: tauri.version };
if (!rustVersion || new Set(Object.values(versions)).size !== 1) {
  console.error(`Version mismatch: ${JSON.stringify(versions)}`);
  process.exit(1);
}

if (process.env.GITHUB_REF_TYPE === 'tag') {
  const expectedTag = `v${pkg.version}`;
  if (process.env.GITHUB_REF_NAME !== expectedTag) {
    console.error(`Release tag mismatch: expected ${expectedTag}, got ${process.env.GITHUB_REF_NAME}`);
    process.exit(1);
  }
}

const eagerWindows = (tauri.app?.windows ?? []).filter((window) => window.create !== false);
if (eagerWindows.length) {
  console.error(
    `Startup invariant violated: configured WebViews must use create=false so AppState can be managed first. Offenders: ${eagerWindows
      .map((window) => window.label)
      .join(', ')}`
  );
  process.exit(1);
}

if (tauri.bundle?.createUpdaterArtifacts) {
  console.error('Clone invariant violated: base tauri.conf.json must not create updater artifacts or require signing secrets.');
  process.exit(1);
}

if (releaseTauri.bundle?.createUpdaterArtifacts !== true) {
  console.error('Release invariant violated: tauri.release.conf.json must enable updater artifacts.');
  process.exit(1);
}

if (/^\s*panic\s*=\s*"abort"\s*$/m.test(cargo)) {
  console.error('Release panic=abort is forbidden: crash diagnostics require unwind-capable Rust panics.');
  process.exit(1);
}

const officialBuild = process.env.CLIPBOARD_PREVIEW_OFFICIAL_BUILD === '1';
if (officialBuild) {
  if (!process.env.TAURI_SIGNING_PRIVATE_KEY?.trim()) {
    console.error('Official release invariant violated: TAURI_SIGNING_PRIVATE_KEY is missing.');
    process.exit(1);
  }
  if (!updaterPublicKey || updaterPublicKey.includes('PLACEHOLDER')) {
    console.error('Official release invariant violated: src-tauri/updater.pub still contains the placeholder.');
    process.exit(1);
  }
}

console.log(
  `Version ${pkg.version} is synchronized. Startup, clone-safe updater, and ${officialBuild ? 'official release' : 'source build'} invariants are preserved.`
);
