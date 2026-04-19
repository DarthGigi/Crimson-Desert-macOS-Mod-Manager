import { invoke } from '@tauri-apps/api/core';

export type GameInstallInfo = {
	packagesPath: string;
	appPath: string | null;
	metaExists: boolean;
	pamtExists: boolean;
	writable: boolean;
	detected: boolean;
};

export type ModKind =
	| 'json_data'
	| 'precompiled_overlay'
	| 'browser_raw'
	| 'language'
	| 'asi'
	| 'bnk'
	| 'binary_patch'
	| 'script_installer';

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

export type HistoryEntry = {
	id: number;
	action: string;
	status: string;
	message: string;
	detailsJson: string | null;
	createdAt: string;
};

export type ModProfile = {
	id: number;
	name: string;
	description: string | null;
	createdAt: string;
	updatedAt: string;
};

export type IsolationSession = {
	suspects: string[];
	currentTestSet: string[];
	rounds: number;
	lastResult: boolean | null;
	resolvedModId: string | null;
};

export type VerifyGameStateResult = {
	packagesPath: string;
	metaExists: boolean;
	pamtExists: boolean;
	backupExists: boolean;
	managedGroupCount: number;
	recoveryPending: boolean;
	enabledModCount: number;
	disabledModCount: number;
};

export type VirtualFileMatch = {
	sourceGroup: string;
	virtualPath: string;
	sourcePazIndex: number;
	compressedSize: number;
	decompressedSize: number;
	flags: number;
};

export type XmlPreview = {
	virtualPath: string;
	sourceGroup: string;
	sourcePazIndex: number;
	encrypted: boolean;
	compressed: boolean;
	compressedSize: number;
	decompressedSize: number;
	extractedPath: string;
};

export type XmlRepackResult = {
	virtualPath: string;
	sourceGroup: string;
	modifiedPath: string;
	targetCompSize: number;
	newCompSize: number;
	exactFit: boolean;
	patchedInPlace: boolean;
	outputPath: string | null;
};

export type AsiPluginInfo = {
	name: string;
	enabled: boolean;
	path: string;
	iniFiles: string[];
	hookTargets: string[];
};

export type ExternalFileInfo = {
	name: string;
	path: string;
	kind: string;
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

export async function removeMod(modId: string) {
	return invoke<DashboardData>('remove_mod_command', { modId });
}

export async function getAsiPlugins() {
	return invoke<AsiPluginInfo[]>('get_asi_plugins_command');
}

export async function installAsiMod(modId: string) {
	return invoke<DashboardData>('install_asi_mod_command', { modId });
}

export async function setAsiEnabled(pluginName: string, enabled: boolean) {
	return invoke<AsiPluginInfo[]>('set_asi_enabled_command', { pluginName, enabled });
}

export async function removeAsiPlugin(pluginName: string) {
	return invoke<AsiPluginInfo[]>('remove_asi_plugin_command', { pluginName });
}

export async function installBnkMod(modId: string) {
	return invoke<DashboardData>('install_bnk_mod_command', { modId });
}

export async function getBnkFiles() {
	return invoke<ExternalFileInfo[]>('get_bnk_files_command');
}

export async function removeBnkFile(name: string) {
	return invoke<ExternalFileInfo[]>('remove_bnk_file_command', { name });
}

export async function installScriptMod(modId: string) {
	return invoke<DashboardData>('install_script_mod_command', { modId });
}

export async function applyBinaryPatch(modId: string, targetFile: string, outputFile: string) {
	return invoke<DashboardData>('apply_binary_patch_command', { modId, targetFile, outputFile });
}

export async function runScriptInstaller(modId: string, workingDir: string) {
	return invoke<DashboardData>('run_script_installer_command', { modId, workingDir });
}

export async function setSelectedLanguage(language: string | null) {
	return invoke<DashboardData>('set_selected_language_command', { language });
}

export async function setModClassification(
	modId: string,
	modKind: ModKind,
	language: string | null
) {
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

export async function extractVirtualFile(
	virtualPath: string,
	sourceGroup: string | null,
	outputDir: string
) {
	return invoke<ExtractResult>('extract_virtual_file_command', {
		virtualPath,
		sourceGroup,
		outputDir
	});
}

export async function getHistory(limit = 50) {
	return invoke<HistoryEntry[]>('get_history_command', { limit });
}

export async function getProfiles() {
	return invoke<ModProfile[]>('get_profiles_command');
}

export async function createProfile(name: string, description: string | null) {
	return invoke<ModProfile[]>('create_profile_command', { name, description });
}

export async function saveProfile(profileId: number) {
	return invoke<ModProfile[]>('save_profile_command', { profileId });
}

export async function applyProfile(profileId: number) {
	return invoke<DashboardData>('apply_profile_command', { profileId });
}

export async function deleteProfile(profileId: number) {
	return invoke<ModProfile[]>('delete_profile_command', { profileId });
}

export async function getProblemModIsolation() {
	return invoke<IsolationSession | null>('get_problem_mod_isolation_command');
}

export async function startProblemModIsolation() {
	return invoke<DashboardData>('start_problem_mod_isolation_command');
}

export async function reportProblemModIsolation(crashed: boolean) {
	return invoke<DashboardData>('report_problem_mod_isolation_command', { crashed });
}

export async function clearProblemModIsolation() {
	return invoke<DashboardData>('clear_problem_mod_isolation_command');
}

export async function verifyGameState() {
	return invoke<VerifyGameStateResult>('verify_game_state_command');
}

export async function exportDiagnosticReport(outputPath: string) {
	return invoke<string>('export_diagnostic_report_command', { outputPath });
}

export async function searchVirtualFiles(query: string, sourceGroup: string | null, limit = 100) {
	return invoke<VirtualFileMatch[]>('search_virtual_files_command', { query, sourceGroup, limit });
}

export async function extractXmlEntry(
	virtualPath: string,
	sourceGroup: string | null,
	outputDir: string
) {
	return invoke<XmlPreview>('extract_xml_entry_command', { virtualPath, sourceGroup, outputDir });
}

export async function repackXmlEntry(
	virtualPath: string,
	sourceGroup: string | null,
	modifiedPath: string,
	outputPath: string | null
) {
	return invoke<XmlRepackResult>('repack_xml_entry_command', {
		virtualPath,
		sourceGroup,
		modifiedPath,
		outputPath
	});
}
