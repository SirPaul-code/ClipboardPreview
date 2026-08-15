import fs from 'node:fs';

const [manifestPath, releasePath, explicitTag] = process.argv.slice(2);

function fail(message) {
  console.error(`Updater release verification failed: ${message}`);
  process.exit(1);
}

if (!manifestPath || !releasePath) {
  fail('usage: node scripts/verify-updater-release.mjs <latest.json> <release.json> [vX.Y.Z]');
}

const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
const release = JSON.parse(fs.readFileSync(releasePath, 'utf8'));
const refTag = process.env.GITHUB_REF_NAME;
const tag = explicitTag ?? (/^v\d+\.\d+\.\d+$/.test(refTag ?? '') ? refTag : release.tag_name);

if (!tag || !/^v\d+\.\d+\.\d+$/.test(tag)) {
  fail(`expected a vX.Y.Z release tag, got ${tag ?? '<unset>'}`);
}

const expectedVersion = tag.slice(1);

if (release.tag_name !== tag) {
  fail(`draft release tag mismatch: expected ${tag}, got ${release.tag_name ?? '<missing>'}`);
}
if (release.draft !== true) {
  fail('release must remain draft until updater verification finishes');
}
if (manifest.version !== expectedVersion) {
  fail(`latest.json version mismatch: expected ${expectedVersion}, got ${manifest.version ?? '<missing>'}`);
}

const requiredPlatforms = [
  'windows-x86_64',
  'darwin-aarch64',
  'darwin-x86_64',
  'linux-x86_64'
];
const releaseAssetIds = new Set((release.assets ?? []).map((asset) => String(asset.id)));
const releaseAssetNames = new Set((release.assets ?? []).map((asset) => asset.name));

for (const platform of requiredPlatforms) {
  const entry = manifest.platforms?.[platform];
  if (!entry) {
    fail(`latest.json is missing ${platform}`);
  }
  if (typeof entry.signature !== 'string' || !entry.signature.trim()) {
    fail(`${platform} has an empty updater signature`);
  }
  if (typeof entry.url !== 'string' || !entry.url.trim()) {
    fail(`${platform} has an empty updater URL`);
  }

  let url;
  try {
    url = new URL(entry.url);
  } catch {
    fail(`${platform} has an invalid updater URL: ${entry.url}`);
  }
  if (url.protocol !== 'https:' || !url.hostname.endsWith('github.com')) {
    fail(`${platform} updater URL must use GitHub over HTTPS: ${entry.url}`);
  }

  const apiAsset = url.pathname.match(/\/repos\/SirPaul-code\/ClipboardPreview\/releases\/assets\/(\d+)$/);
  if (apiAsset && !releaseAssetIds.has(apiAsset[1])) {
    fail(`${platform} points at release asset ${apiAsset[1]}, but that asset is not attached to this release`);
  }
}

const escapeRegex = (value) => value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const version = escapeRegex(expectedVersion);
const requiredAssets = [
  ['Windows NSIS installer', new RegExp(`^ClipboardPreview-${version}-windows-x64-setup\\.exe$`, 'i')],
  ['Windows MSI installer', new RegExp(`^ClipboardPreview-${version}-windows-x64\\.msi$`, 'i')],
  ['macOS Apple Silicon DMG', new RegExp(`^ClipboardPreview-${version}-darwin-aarch64\\.dmg$`, 'i')],
  ['macOS Intel DMG', new RegExp(`^ClipboardPreview-${version}-darwin-x64\\.dmg$`, 'i')],
  ['Linux AppImage', new RegExp(`^ClipboardPreview-${version}-linux-(?:x64|amd64)\\.AppImage$`, 'i')],
  ['Linux Debian package', new RegExp(`^ClipboardPreview-${version}-linux-(?:x64|amd64)\\.deb$`, 'i')],
  ['updater manifest', /^latest\.json$/]
];

for (const [label, pattern] of requiredAssets) {
  if (![...releaseAssetNames].some((name) => pattern.test(name))) {
    fail(`missing ${label}`);
  }
}

const signatureCount = [...releaseAssetNames].filter((name) => name.endsWith('.sig')).length;
if (signatureCount < requiredPlatforms.length) {
  fail(`expected at least ${requiredPlatforms.length} updater signature assets, found ${signatureCount}`);
}

console.log(
  `Updater release ${tag} is complete: ${requiredPlatforms.length} signed targets, ${releaseAssetNames.size} assets.`
);
