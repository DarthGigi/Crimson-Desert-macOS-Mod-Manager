import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const localKeyPath = path.join(root, '.keys', 'updater.key');

const env = { ...process.env };
if (!env.TAURI_SIGNING_PRIVATE_KEY && !env.TAURI_SIGNING_PRIVATE_KEY_PATH && fs.existsSync(localKeyPath)) {
	env.TAURI_SIGNING_PRIVATE_KEY_PATH = localKeyPath;
}

const command = process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm';
const child = spawn(command, ['exec', 'tauri', ...process.argv.slice(2)], {
	stdio: 'inherit',
	env,
	cwd: root,
	shell: false
});

child.on('exit', (code, signal) => {
	if (signal) {
		process.kill(process.pid, signal);
	}
	process.exit(code ?? 1);
});
