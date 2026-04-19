import { invoke } from '@tauri-apps/api/core';

export type GameInstallInfo = {
	packagesPath: string;
	appPath: string | null;
	metaExists: boolean;
	pamtExists: boolean;
	writable: boolean;
	detected: boolean;
};

export type ModKind = 'json_data' | 'precompiled_overlay' | 'language';

export type ModRecord = {
	id: string;
	modKind: ModKind;
	name: string;
	description: string | null;
	fileName: string;
	sourcePath: string | null;
	libraryPath: string;
	enabled: boolean;
	loadOrder: number;
	language: string | null;
	installGroup: string | null;
	patchCount: number;
	changeCount: number;
	targetFiles: string[];
	importedAt: string;
	updatedAt: string;
};

export type ScanResult = {
	path: string;
	modKind: ModKind;
	fileName: string;
	name: string;
	description: string | null;
	patchCount: number;
	changeCount: number;
	targetFiles: string[];
	resolvableFiles: number;
	missingFiles: string[];
};

export type ApplyFileResult = {
	gameFile: string;
	sourceGroup: string;
	sourcePazIndex: number;
	appliedChanges: number;
	skippedChanges: number;
	overlapCount: number;
	reason: string | null;
};

export type ApplyResult = {
	modCount: number;
	targetFileCount: number;
	overlayFileCount: number;
	pazSize: number;
	pamtSize: number;
	createdGroups: string[];
	files: ApplyFileResult[];
	message: string;
};

export type StatusSummary = {
	gameInstall: GameInstallInfo | null;
	selectedLanguage: string | null;
	overlayActive: boolean;
	backupExists: boolean;
	totalMods: number;
	enabledMods: number;
	disabledMods: number;
};

export type DashboardData = {
	status: StatusSummary;
	available: ModRecord[];
	enabled: ModRecord[];
	disabled: ModRecord[];
};

type LaunchResult = {
	launched: boolean;
};

export async function getDashboard() {
	return invoke<DashboardData>('get_dashboard');
}

export async function detectGameInstall() {
	return invoke<GameInstallInfo | null>('detect_game_install_command');
}

export async function setGameInstall(path: string) {
	return invoke<GameInstallInfo>('set_game_install_command', { path });
}

export async function scanModFolder(folderPath: string) {
	return invoke<ScanResult[]>('scan_mod_folder_command', { folderPath });
}

export async function importModVariant(path: string, enable = true) {
	return invoke<DashboardData>('import_mod_variant_command', { path, enable });
}

export async function setModEnabled(modId: string, enabled: boolean) {
	return invoke<DashboardData>('set_mod_enabled_command', { modId, enabled });
}

export async function setSelectedLanguage(language: string | null) {
	return invoke<DashboardData>('set_selected_language_command', { language });
}

export async function setModClassification(modId: string, modKind: ModKind, language: string | null) {
	return invoke<DashboardData>('set_mod_classification_command', { modId, modKind, language });
}

export async function applyMods() {
	return invoke<ApplyResult>('apply_mods_command');
}

export async function restoreVanilla() {
	return invoke<DashboardData>('restore_vanilla_command');
}

export async function resetActiveMods() {
	return invoke<DashboardData>('reset_active_mods_command');
}

export async function launchGame() {
	return invoke<LaunchResult>('launch_game_command');
}
