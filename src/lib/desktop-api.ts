import { invoke } from '@tauri-apps/api/core';

export type GameInstallInfo = {
	packagesPath: string;
	appPath: string | null;
	metaExists: boolean;
	pamtExists: boolean;
	writable: boolean;
	detected: boolean;
};

export type ModKind = 'json_data' | 'precompiled_overlay' | 'browser_raw' | 'language';

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

export type ApplyPreviewFile = {
	gameFile: string;
	sourceGroup: string;
	sourcePazIndex: number | null;
	changeCount: number;
	overlapCount: number;
	sourceMods: string[];
	resolved: boolean;
	reason: string | null;
};

export type ApplyPreview = {
	modCount: number;
	jsonModCount: number;
	precompiledModCount: number;
	browserRawModCount: number;
	targetFileCount: number;
	estimatedGroupCount: number;
	selectedLanguage: string | null;
	files: ApplyPreviewFile[];
};

export type ModPatchSummary = {
	modId: string;
	patchIndex: number;
	title: string;
	sourceGroup: string;
	gameFile: string;
	changeCount: number;
	enabled: boolean;
};

export type PathcLookupResult = {
	virtualPath: string;
	keyHash: number;
	found: boolean;
	directDdsIndex: number | null;
	width: number | null;
	height: number | null;
	mipCount: number | null;
	formatLabel: string | null;
	m1: number | null;
	m2: number | null;
	m3: number | null;
	m4: number | null;
};

export type PathcSummary = {
	path: string;
	ddsTemplateCount: number;
	hashCount: number;
	collisionPathCount: number;
	directMappingCount: number;
	collisionMappingCount: number;
	unknownMappingCount: number;
	lookups: PathcLookupResult[];
};

export type PathcRepackResult = {
	pathcPath: string;
	backupPath: string;
	processedCount: number;
	updatedCount: number;
	addedTemplateCount: number;
};

export type ExtractPreview = {
	virtualPath: string;
	sourceGroup: string;
	resolved: boolean;
	resolvedGameFile: string | null;
	sourcePazIndex: number | null;
	compressedSize: number | null;
	decompressedSize: number | null;
	flags: number | null;
	reason: string | null;
};

export type ExtractResult = {
	virtualPath: string;
	sourceGroup: string;
	outputPath: string;
	decompressedSize: number;
};

export type StatusSummary = {
	gameInstall: GameInstallInfo | null;
	selectedLanguage: string | null;
	recoveryPending: boolean;
	pendingOperation: string | null;
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

export async function moveModInLoadOrder(modId: string, direction: 'up' | 'down') {
	return invoke<DashboardData>('move_mod_in_load_order_command', { modId, direction });
}

export async function getModPatchSummaries(modId: string) {
	return invoke<ModPatchSummary[]>('get_mod_patch_summaries_command', { modId });
}

export async function setPatchEnabled(modId: string, patchIndex: number, enabled: boolean) {
	return invoke<DashboardData>('set_patch_enabled_command', { modId, patchIndex, enabled });
}

export async function applyMods() {
	return invoke<ApplyResult>('apply_mods_command');
}

export async function getApplyPreview() {
	return invoke<ApplyPreview>('get_apply_preview_command');
}

export async function restoreVanilla() {
	return invoke<DashboardData>('restore_vanilla_command');
}

export async function resetActiveMods() {
	return invoke<DashboardData>('reset_active_mods_command');
}

export async function fixEverything() {
	return invoke<DashboardData>('fix_everything_command');
}

export async function launchGame() {
	return invoke<LaunchResult>('launch_game_command');
}

export async function getPathcSummary(path: string | null, lookups: string[]) {
	return invoke<PathcSummary>('get_pathc_summary_command', { path, lookups });
}

export async function repackPathc(path: string | null, folderPath: string) {
	return invoke<PathcRepackResult>('repack_pathc_command', { path, folderPath });
}

export async function getVirtualFilePreview(virtualPath: string, sourceGroup: string | null) {
	return invoke<ExtractPreview>('get_virtual_file_preview_command', { virtualPath, sourceGroup });
}

export async function extractVirtualFile(virtualPath: string, sourceGroup: string | null, outputDir: string) {
	return invoke<ExtractResult>('extract_virtual_file_command', { virtualPath, sourceGroup, outputDir });
}
