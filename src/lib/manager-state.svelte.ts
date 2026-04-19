import { goto } from '$app/navigation';
import { toast } from 'svelte-sonner';
import {
	applyMods,
	detectGameInstall,
	getApplyPreview,
	getDashboard,
	getHistory,
	getModPatchSummaries,
	getPathcSummary,
	extractXmlEntry,
	searchVirtualFiles,
	getVirtualFilePreview,
	importModVariant,
	launchGame,
	moveModInLoadOrder,
	type ApplyPreview,
	type ApplyResult,
	type DashboardData,
	type ExtractPreview,
	type ExtractResult,
	type HistoryEntry,
	type ModKind,
	type ModPatchSummary,
	type ModRecord,
	type PathcRepackResult,
	type PathcSummary,
	type ScanResult,
	type VirtualFileMatch,
	type XmlPreview,
	type XmlRepackResult,
	fixEverything,
	repackPathc,
	resetActiveMods,
	restoreVanilla,
	scanModFolder,
	setModClassification,
	setGameInstall,
	setPatchEnabled,
	setSelectedLanguage,
	setModEnabled,
	extractVirtualFile,
	repackXmlEntry
} from '$lib/desktop-api';

export type ManagerMessage = {
	kind: 'error' | 'success' | 'info';
	title: string;
	message: string;
};

class ManagerState {
	dashboard = $state<DashboardData | null>(null);
	applyPreview = $state<ApplyPreview | null>(null);
	pathcSummary = $state<PathcSummary | null>(null);
	pathcResult = $state<PathcRepackResult | null>(null);
	extractPreview = $state<ExtractPreview | null>(null);
	extractResult = $state<ExtractResult | null>(null);
	xmlPreview = $state<XmlPreview | null>(null);
	xmlRepackResult = $state<XmlRepackResult | null>(null);
	historyEntries = $state<HistoryEntry[]>([]);
	virtualFileMatches = $state<VirtualFileMatch[]>([]);
	patchSummaries = $state<ModPatchSummary[]>([]);
	scanResults = $state<ScanResult[]>([]);
	lastApplyResult = $state<ApplyResult | null>(null);
	selectedPatchModId = $state<string | null>(null);
	message = $state<ManagerMessage | null>(null);
	busy = $state({
		boot: false,
		detectingGame: false,
		settingGame: false,
		scanningMods: false,
		importing: false,
		previewing: false,
		patches: false,
		applying: false,
		restoring: false,
		resetting: false,
		launching: false,
		fixing: false,
		history: false,
		pathc: false,
		repackingPathc: false,
		extracting: false,
		searchingFiles: false,
		xml: false,
		toggling: ''
	});

	get allMods() {
		return this.dashboard?.available ?? [];
	}

	get install() {
		return this.dashboard?.status.gameInstall ?? null;
	}

	get selectedLanguage() {
		return this.dashboard?.status.selectedLanguage ?? null;
	}

	get enabledCount() {
		return this.dashboard?.status.enabledMods ?? 0;
	}

	get disabledCount() {
		return this.dashboard?.status.disabledMods ?? 0;
	}

	get totalCount() {
		return this.dashboard?.status.totalMods ?? 0;
	}

	get overlayActive() {
		return this.dashboard?.status.overlayActive ?? false;
	}

	get backupExists() {
		return this.dashboard?.status.backupExists ?? false;
	}

	get recoveryPending() {
		return this.dashboard?.status.recoveryPending ?? false;
	}

	get pendingOperation() {
		return this.dashboard?.status.pendingOperation ?? null;
	}

	get orderedJsonMods() {
		return this.allMods
			.filter((mod) => mod.modKind === 'json_data' && mod.enabled)
			.toSorted((left, right) => left.loadOrder - right.loadOrder);
	}

	get languageMods() {
		return this.allMods.filter((mod) => mod.modKind === 'language');
	}

	get precompiledMods() {
		return this.allMods.filter(
			(mod) => mod.modKind === 'precompiled_overlay' || mod.modKind === 'browser_raw'
		);
	}

	get previewConflictFiles() {
		return this.applyPreview?.files.filter((file) => file.overlapCount > 0) ?? [];
	}

	get previewUnresolvedFiles() {
		return this.applyPreview?.files.filter((file) => !file.resolved) ?? [];
	}

	async ensureLoaded() {
		if (this.dashboard || this.busy.boot) return;
		await this.refreshDashboard();
	}

	clearMessage() {
		this.message = null;
	}

	setError(error: unknown, title: string) {
		const details = this.toMessage(error);
		this.message = { kind: 'error', title, message: details };
		toast.error(title, { description: details });
	}

	toMessage(error: unknown) {
		if (typeof error === 'string') return error;
		if (
			typeof error === 'object' &&
			error &&
			'message' in error &&
			typeof error.message === 'string'
		) {
			return error.message;
		}
		return 'Something went wrong.';
	}

	async refreshDashboard() {
		this.busy.boot = true;
		try {
			this.dashboard = await getDashboard();
			if (!this.selectedPatchModId && this.orderedJsonMods.length > 0) {
				this.selectedPatchModId = this.orderedJsonMods[0].id;
			}
			if (
				this.selectedPatchModId &&
				!this.orderedJsonMods.some((mod) => mod.id === this.selectedPatchModId)
			) {
				this.selectedPatchModId = this.orderedJsonMods[0]?.id ?? null;
			}
			await Promise.all([
				this.refreshPatchSummaries(),
				this.refreshPreview(),
				this.refreshHistory()
			]);
		} catch (error) {
			this.setError(error, 'Could not load the mod manager dashboard');
		} finally {
			this.busy.boot = false;
		}
	}

	async detectInstall() {
		this.busy.detectingGame = true;
		this.clearMessage();
		try {
			const detected = await detectGameInstall();
			if (!detected) {
				this.message = {
					kind: 'info',
					title: 'No auto-detected install',
					message: 'Pick the Crimson Desert.app bundle or the packages directory manually.'
				};
				toast.info('No Crimson Desert install was auto-detected.');
				return null;
			}

			await this.refreshDashboard();
			toast.success('Detected and saved the current Crimson Desert install.');
			return detected;
		} catch (error) {
			this.setError(error, 'Could not auto-detect the game install');
			return null;
		} finally {
			this.busy.detectingGame = false;
		}
	}

	async saveGamePath(path: string) {
		if (!path.trim()) {
			this.message = {
				kind: 'info',
				title: 'Game path required',
				message: 'Choose a Crimson Desert.app bundle or packages directory first.'
			};
			toast.info('Pick a game path first.');
			return null;
		}

		this.busy.settingGame = true;
		this.clearMessage();
		try {
			const install = await setGameInstall(path.trim());
			await this.refreshDashboard();
			toast.success('Saved the game install path.');
			return install;
		} catch (error) {
			this.setError(error, 'Could not save the game install path');
			return null;
		} finally {
			this.busy.settingGame = false;
		}
	}

	async refreshPatchSummaries() {
		if (!this.selectedPatchModId) {
			this.patchSummaries = [];
			return;
		}

		this.busy.patches = true;
		try {
			this.patchSummaries = await getModPatchSummaries(this.selectedPatchModId);
		} catch (error) {
			this.patchSummaries = [];
			this.setError(error, 'Could not load patch toggles');
		} finally {
			this.busy.patches = false;
		}
	}

	async refreshPreview() {
		if (!this.install) {
			this.applyPreview = null;
			return;
		}

		this.busy.previewing = true;
		try {
			this.applyPreview = await getApplyPreview();
		} catch (error) {
			this.applyPreview = null;
			if (this.toMessage(error) !== 'Set the Crimson Desert game path first.') {
				this.setError(error, 'Could not build the apply preview');
			}
		} finally {
			this.busy.previewing = false;
		}
	}

	async refreshHistory() {
		this.busy.history = true;
		try {
			this.historyEntries = await getHistory(40);
		} catch (error) {
			this.historyEntries = [];
			this.setError(error, 'Could not load operation history');
		} finally {
			this.busy.history = false;
		}
	}

	async scanImportSource(path: string) {
		this.busy.scanningMods = true;
		this.clearMessage();
		try {
			this.scanResults = await scanModFolder(path);
			if (this.scanResults.length === 0) {
				this.message = {
					kind: 'info',
					title: 'No mod candidates found',
					message: 'The selected folder or archive does not contain supported mod variants.'
				};
				toast.info('No supported mods were found in that source.');
				return;
			}
			toast.success(`Found ${this.scanResults.length} importable candidate(s).`);
		} catch (error) {
			this.setError(error, 'Could not scan the selected import source');
		} finally {
			this.busy.scanningMods = false;
		}
	}

	async importScanResult(result: ScanResult) {
		this.busy.importing = true;
		this.clearMessage();
		try {
			this.dashboard = await importModVariant(result.path, true);
			this.lastApplyResult = null;
			await Promise.all([
				this.refreshPatchSummaries(),
				this.refreshPreview(),
				this.refreshHistory()
			]);
			toast.success(`Imported ${result.fileName}.`);
		} catch (error) {
			this.setError(error, `Could not import ${result.fileName}`);
		} finally {
			this.busy.importing = false;
		}
	}

	async toggleMod(mod: ModRecord) {
		this.busy.toggling = mod.id;
		this.clearMessage();
		try {
			this.dashboard = await setModEnabled(mod.id, !mod.enabled);
			await Promise.all([
				this.refreshPatchSummaries(),
				this.refreshPreview(),
				this.refreshHistory()
			]);
			toast.success(`${mod.name} ${mod.enabled ? 'disabled' : 'enabled'}.`);
		} catch (error) {
			this.setError(error, `Could not update ${mod.name}`);
		} finally {
			this.busy.toggling = '';
		}
	}

	async moveMod(mod: ModRecord, direction: 'up' | 'down') {
		this.clearMessage();
		try {
			this.dashboard = await moveModInLoadOrder(mod.id, direction);
			await Promise.all([
				this.refreshPatchSummaries(),
				this.refreshPreview(),
				this.refreshHistory()
			]);
			toast.success(`${mod.name} moved ${direction} in JSON load order.`);
		} catch (error) {
			this.setError(error, `Could not move ${mod.name}`);
		}
	}

	async togglePatch(patch: ModPatchSummary) {
		this.clearMessage();
		try {
			this.dashboard = await setPatchEnabled(patch.modId, patch.patchIndex, !patch.enabled);
			await Promise.all([
				this.refreshPatchSummaries(),
				this.refreshPreview(),
				this.refreshHistory()
			]);
			toast.success(`${patch.title} ${patch.enabled ? 'disabled' : 'enabled'}.`);
		} catch (error) {
			this.setError(error, `Could not update ${patch.title}`);
		}
	}

	async chooseLanguage(language: string | null) {
		this.clearMessage();
		try {
			this.dashboard = await setSelectedLanguage(language);
			await Promise.all([this.refreshPreview(), this.refreshHistory()]);
			toast.success(
				language
					? `Selected ${language.toUpperCase()} for language overlays.`
					: 'Cleared the selected language.'
			);
		} catch (error) {
			this.setError(error, 'Could not update the selected language');
		}
	}

	async classifyMod(mod: ModRecord, modKind: ModKind, fallbackKind: ModKind) {
		this.clearMessage();
		if (modKind === 'language' && !this.selectedLanguage) {
			toast.info('Choose the current in-game language first.');
			return;
		}

		const nextKind = modKind === 'language' ? modKind : fallbackKind;
		try {
			this.dashboard = await setModClassification(
				mod.id,
				nextKind,
				modKind === 'language' ? this.selectedLanguage : null
			);
			await Promise.all([this.refreshPreview(), this.refreshHistory()]);
			toast.success(`${mod.name} is now classified as ${nextKind.replace('_', ' ')}.`);
		} catch (error) {
			this.setError(error, `Could not update ${mod.name}`);
		}
	}

	async runApply() {
		this.busy.applying = true;
		this.clearMessage();
		try {
			this.lastApplyResult = await applyMods();
			await this.refreshDashboard();
			toast.success(this.lastApplyResult.message);
			await goto('/apply-logs');
		} catch (error) {
			this.setError(error, 'Could not apply the enabled mods');
		} finally {
			this.busy.applying = false;
		}
	}

	async runRestore() {
		this.busy.restoring = true;
		this.clearMessage();
		try {
			this.dashboard = await restoreVanilla();
			this.lastApplyResult = null;
			this.applyPreview = null;
			await this.refreshHistory();
			toast.success('Restored the game overlay to vanilla.');
		} catch (error) {
			this.setError(error, 'Could not restore the vanilla overlay');
		} finally {
			this.busy.restoring = false;
		}
	}

	async runReset() {
		this.busy.resetting = true;
		this.clearMessage();
		try {
			this.dashboard = await resetActiveMods();
			this.lastApplyResult = null;
			this.applyPreview = null;
			await this.refreshHistory();
			toast.success('Disabled every active mod and restored vanilla files.');
		} catch (error) {
			this.setError(error, 'Could not reset the active mod set');
		} finally {
			this.busy.resetting = false;
		}
	}

	async runFixEverything() {
		this.busy.fixing = true;
		this.clearMessage();
		try {
			this.dashboard = await fixEverything();
			this.lastApplyResult = null;
			this.applyPreview = null;
			await this.refreshHistory();
			toast.success('Reset the manager state and restored vanilla files.');
		} catch (error) {
			this.setError(error, 'Could not run Fix Everything');
		} finally {
			this.busy.fixing = false;
		}
	}

	async runLaunch() {
		this.busy.launching = true;
		this.clearMessage();
		try {
			await launchGame();
			toast.success('Launching Crimson Desert.');
		} catch (error) {
			this.setError(error, 'Could not launch the game');
		} finally {
			this.busy.launching = false;
		}
	}

	async refreshPathcSummary(pathcPath: string, lookup: string) {
		if (!pathcPath && !this.install) {
			this.pathcSummary = null;
			return;
		}

		this.busy.pathc = true;
		try {
			this.pathcSummary = await getPathcSummary(
				pathcPath || null,
				lookup.trim() ? [lookup.trim()] : []
			);
		} catch (error) {
			this.pathcSummary = null;
			if (!this.toMessage(error).includes('Set the Crimson Desert game path first')) {
				this.setError(error, 'Could not inspect PATHC metadata');
			}
		} finally {
			this.busy.pathc = false;
		}
	}

	async runPathcRepack(pathcPath: string, folderPath: string) {
		if (!folderPath.trim()) {
			toast.info('Choose a DDS folder first.');
			return;
		}

		this.clearMessage();
		this.busy.repackingPathc = true;
		try {
			this.pathcResult = await repackPathc(pathcPath || null, folderPath.trim());
			await this.refreshHistory();
			toast.success(`Repacked PATHC with ${this.pathcResult.processedCount} DDS file(s).`);
		} catch (error) {
			this.setError(error, 'Could not repack PATHC');
		} finally {
			this.busy.repackingPathc = false;
		}
	}

	async refreshExtractPreview(virtualPath: string, sourceGroup: string | null) {
		if (!virtualPath.trim()) {
			this.extractPreview = null;
			return;
		}

		this.busy.extracting = true;
		try {
			this.extractPreview = await getVirtualFilePreview(virtualPath.trim(), sourceGroup);
		} catch (error) {
			this.extractPreview = null;
			this.setError(error, 'Could not inspect virtual file');
		} finally {
			this.busy.extracting = false;
		}
	}

	async searchVirtualFiles(query: string, sourceGroup: string | null, limit = 100) {
		this.busy.searchingFiles = true;
		try {
			this.virtualFileMatches = await searchVirtualFiles(query, sourceGroup, limit);
		} catch (error) {
			this.virtualFileMatches = [];
			this.setError(error, 'Could not search virtual files');
		} finally {
			this.busy.searchingFiles = false;
		}
	}

	async extractXmlEntry(virtualPath: string, sourceGroup: string | null, outputDir: string) {
		this.busy.xml = true;
		this.clearMessage();
		try {
			this.xmlPreview = await extractXmlEntry(virtualPath, sourceGroup, outputDir);
			await this.refreshHistory();
			toast.success(`Extracted XML ${this.xmlPreview.virtualPath}.`);
		} catch (error) {
			this.setError(error, 'Could not extract XML entry');
		} finally {
			this.busy.xml = false;
		}
	}

	async repackXmlEntry(
		virtualPath: string,
		sourceGroup: string | null,
		modifiedPath: string,
		outputPath: string | null
	) {
		this.busy.xml = true;
		this.clearMessage();
		try {
			this.xmlRepackResult = await repackXmlEntry(
				virtualPath,
				sourceGroup,
				modifiedPath,
				outputPath
			);
			await this.refreshHistory();
			toast.success(
				this.xmlRepackResult.patchedInPlace
					? 'Patched XML entry in place.'
					: 'Prepared XML repack payload.'
			);
		} catch (error) {
			this.setError(error, 'Could not repack XML entry');
		} finally {
			this.busy.xml = false;
		}
	}

	async runExtractVirtualFile(virtualPath: string, sourceGroup: string | null, outputDir: string) {
		if (!outputDir.trim()) {
			toast.info('Choose an output folder first.');
			return;
		}

		this.busy.extracting = true;
		this.clearMessage();
		try {
			this.extractResult = await extractVirtualFile(
				virtualPath.trim(),
				sourceGroup,
				outputDir.trim()
			);
			await this.refreshHistory();
			toast.success(`Extracted ${this.extractResult.virtualPath}.`);
		} catch (error) {
			this.setError(error, 'Could not extract virtual file');
		} finally {
			this.busy.extracting = false;
		}
	}
}

export const manager = new ManagerState();
