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
console.log(`Version ${pkg.version} is synchronized.`);
