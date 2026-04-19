import type { ModKind, ModRecord } from '$lib/desktop-api';

export function formatTimestamp(value: string) {
	const seconds = Number(value);
	if (!Number.isFinite(seconds) || seconds <= 0) {
		return 'Unknown';
	}

	return new Date(seconds * 1000).toLocaleString();
}

export function modKindLabel(modKind: ModKind) {
	if (modKind === 'json_data') return 'JSON';
	if (modKind === 'precompiled_overlay') return 'Precompiled';
	if (modKind === 'browser_raw') return 'Browser/Raw';
	if (modKind === 'asi') return 'ASI';
	if (modKind === 'bnk') return 'BNK';
	if (modKind === 'binary_patch') return 'Binary Patch';
	if (modKind === 'script_installer') return 'Script Installer';
	return 'Language';
}

export function fallbackKindForLanguageMod(mod: ModRecord): ModKind {
	if (mod.libraryPath.endsWith('/files') || mod.libraryPath.includes('/files/')) {
		return 'browser_raw';
	}

	return mod.targetFiles.every((target) => /^\d{4}$/.test(target))
		? 'precompiled_overlay'
		: 'json_data';
}
