import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();

const packageJsonPath = path.join(root, 'package.json');
const tauriConfPath = path.join(root, 'src-tauri', 'tauri.conf.json');
const cargoTomlPath = path.join(root, 'src-tauri', 'Cargo.toml');

const pkg = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
const version = pkg.version;

const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
tauriConf.version = version;
fs.writeFileSync(tauriConfPath, `${JSON.stringify(tauriConf, null, 2)}\n`);

const cargoToml = fs.readFileSync(cargoTomlPath, 'utf8');
const nextCargoToml = cargoToml.replace(/^version = ".*"$/m, `version = "${version}"`);

if (nextCargoToml === cargoToml) {
	throw new Error('Failed to sync version in src-tauri/Cargo.toml');
}

fs.writeFileSync(cargoTomlPath, nextCargoToml);

console.log(`Synced app version to ${version}`);
