import fs from 'node:fs';

const pkg = JSON.parse(fs.readFileSync(new URL('../package.json', import.meta.url), 'utf8'));
const cargo = fs.readFileSync(new URL('../src-tauri/Cargo.toml', import.meta.url), 'utf8');
const tauri = JSON.parse(fs.readFileSync(new URL('../src-tauri/tauri.conf.json', import.meta.url), 'utf8'));

const rustVersion = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
const versions = { package: pkg.version, cargo: rustVersion, tauri: tauri.version };
if (!rustVersion || new Set(Object.values(versions)).size !== 1) {
  console.error(`Version mismatch: ${JSON.stringify(versions)}`);
  process.exit(1);
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

if (/^\s*panic\s*=\s*"abort"\s*$/m.test(cargo)) {
  console.error('Release panic=abort is forbidden: crash diagnostics require unwind-capable Rust panics.');
  process.exit(1);
}

console.log(`Version ${pkg.version} is synchronized and startup invariants are preserved.`);
