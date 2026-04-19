<script lang="ts">
	import { onMount } from 'svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import { toast } from 'svelte-sonner';
	import {
		AlertCircle,
		Archive,
		ArrowDownUp,
		CheckCircle2,
		FolderSearch,
		Gamepad2,
		Globe2,
		HardDriveDownload,
		Image,
		Info,
		Layers3,
		Package,
		RefreshCcw,
		ShieldAlert,
		Sparkles,
		Wrench
	} from '@lucide/svelte';
	import {
		applyMods,
		detectGameInstall,
		getApplyPreview,
		getDashboard,
		getModPatchSummaries,
		getPathcSummary,
		getVirtualFilePreview,
		fixEverything,
		importModVariant,
		launchGame,
		moveModInLoadOrder,
		resetActiveMods,
		repackPathc,
		restoreVanilla,
		scanModFolder,
		setModClassification,
		setGameInstall,
		setPatchEnabled,
		setSelectedLanguage,
		setModEnabled,
		extractVirtualFile,
		type ApplyResult,
		type ApplyPreview,
		type DashboardData,
		type ExtractPreview,
		type ExtractResult,
		type ModKind,
		type ModPatchSummary,
		type ModRecord,
		type PathcRepackResult,
		type PathcSummary,
		type ScanResult
	} from '$lib/desktop-api';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as Table from '$lib/components/ui/table';
	import * as Alert from '$lib/components/ui/alert';
	import * as ScrollArea from '$lib/components/ui/scroll-area';
	import * as Empty from '$lib/components/ui/empty';
	import * as Collapsible from '$lib/components/ui/collapsible';
	import * as AlertDialog from '$lib/components/ui/alert-dialog';
	import { Separator } from '$lib/components/ui/separator';

	type Filter = 'all' | 'enabled' | 'disabled';
	type Message = {
		kind: 'error' | 'success' | 'info';
		title: string;
		message: string;
	};

	let dashboard = $state<DashboardData | null>(null);
	let applyPreview = $state<ApplyPreview | null>(null);
	let pathcSummary = $state<PathcSummary | null>(null);
	let pathcResult = $state<PathcRepackResult | null>(null);
	let extractPreview = $state<ExtractPreview | null>(null);
	let extractResult = $state<ExtractResult | null>(null);
	let selectedPatchModId = $state<string | null>(null);
	let patchSummaries = $state<ModPatchSummary[]>([]);
	let scanResults = $state<ScanResult[]>([]);
	let lastApplyResult = $state<ApplyResult | null>(null);
	let gamePathInput = $state('');
	let pathcPathInput = $state('');
	let pathcLookupInput = $state('/ui/texture/cd_itemslot_00.dds');
	let pathcFolderInput = $state('');
	let extractVirtualPathInput = $state('character/player/playeractiongraph_main.xml');
	let extractSourceGroupInput = $state('0008');
	let extractOutputDirInput = $state('');
	let scanFolderPath = $state('');
	let search = $state('');
	let filter = $state<Filter>('all');
	let message = $state<Message | null>(null);
	let resetDialogOpen = $state(false);
	let scanDetailsOpen = $state<Record<string, boolean>>({});
	let busy = $state({
		boot: true,
		detectingGame: false,
		settingGame: false,
		scanningMods: false,
		importing: false,
		applying: false,
		previewing: false,
		patches: false,
		pathc: false,
		repackingPathc: false,
		extracting: false,
		restoring: false,
		resetting: false,
		fixing: false,
		launching: false,
		toggling: ''
	});

	const allMods = $derived(dashboard?.available ?? []);
	const install = $derived(dashboard?.status.gameInstall ?? null);
	const overlayActive = $derived(dashboard?.status.overlayActive ?? false);
	const backupExists = $derived(dashboard?.status.backupExists ?? false);
	const recoveryPending = $derived(dashboard?.status.recoveryPending ?? false);
	const pendingOperation = $derived(dashboard?.status.pendingOperation ?? null);
	const enabledCount = $derived(dashboard?.status.enabledMods ?? 0);
	const totalCount = $derived(dashboard?.status.totalMods ?? 0);
	const disabledCount = $derived(dashboard?.status.disabledMods ?? 0);
	const filteredMods = $derived.by(() => {
		const query = search.trim().toLowerCase();

		return allMods.filter((mod) => {
			const matchesFilter =
				filter === 'all' || (filter === 'enabled' ? mod.enabled : !mod.enabled);
			const haystack = [mod.name, mod.fileName, mod.description ?? '', mod.targetFiles.join(' ')]
				.join(' ')
				.toLowerCase();
			const matchesSearch = !query || haystack.includes(query);
			return matchesFilter && matchesSearch;
		});
	});
	const selectedLanguage = $derived(dashboard?.status.selectedLanguage ?? null);
	const languageMods = $derived(allMods.filter((mod) => mod.modKind === 'language'));
	const precompiledMods = $derived(allMods.filter((mod) => mod.modKind === 'precompiled_overlay' || mod.modKind === 'browser_raw'));
	const orderedJsonMods = $derived(
		allMods.filter((mod) => mod.modKind === 'json_data' && mod.enabled).toSorted((left, right) => left.loadOrder - right.loadOrder)
	);
	const languageOptions = ['eng', 'jpn', 'kor', 'rus', 'tur', 'spa_es', 'spa_mx', 'fre', 'ger', 'ita', 'pol', 'por_br', 'zho_tw', 'zho_cn'];

	onMount(() => {
		void refreshDashboard();
	});

	async function refreshDashboard() {
		busy.boot = true;
		try {
			dashboard = await getDashboard();
			gamePathInput = dashboard.status.gameInstall?.packagesPath ?? gamePathInput;
			pathcPathInput = dashboard.status.gameInstall?.packagesPath
				? `${dashboard.status.gameInstall.packagesPath}/meta/0.pathc`
				: pathcPathInput;
			if (!selectedPatchModId && orderedJsonMods.length > 0) {
				selectedPatchModId = orderedJsonMods[0].id;
			}
			if (selectedPatchModId && !orderedJsonMods.some((mod) => mod.id === selectedPatchModId)) {
				selectedPatchModId = orderedJsonMods[0]?.id ?? null;
			}
			await refreshPatchSummaries();
			await refreshPreview();
			await refreshPathcSummary();
		} catch (error) {
			setError(error, 'Could not load the mod manager dashboard');
		} finally {
			busy.boot = false;
		}
	}

	async function refreshPathcSummary() {
		if (!pathcPathInput && !dashboard?.status.gameInstall) {
			pathcSummary = null;
			return;
		}

		busy.pathc = true;
		try {
			pathcSummary = await getPathcSummary(pathcPathInput || null, pathcLookupInput.trim() ? [pathcLookupInput.trim()] : []);
		} catch (error) {
			pathcSummary = null;
			if (!toMessage(error).includes('Set the Crimson Desert game path first')) {
				setError(error, 'Could not inspect PATHC metadata');
			}
		} finally {
			busy.pathc = false;
		}
	}

	async function refreshPatchSummaries() {
		if (!selectedPatchModId) {
			patchSummaries = [];
			return;
		}

		busy.patches = true;
		try {
			patchSummaries = await getModPatchSummaries(selectedPatchModId);
		} catch (error) {
			patchSummaries = [];
			setError(error, 'Could not load patch toggles');
		} finally {
			busy.patches = false;
		}
	}

	async function refreshPreview() {
		if (!dashboard?.status.gameInstall && !gamePathInput) {
			applyPreview = null;
			return;
		}

		busy.previewing = true;
		try {
			applyPreview = await getApplyPreview();
		} catch (error) {
			applyPreview = null;
			if (toMessage(error) !== 'Set the Crimson Desert game path first.') {
				setError(error, 'Could not build the apply preview');
			}
		} finally {
			busy.previewing = false;
		}
	}

	async function chooseGamePath() {
		const selected = await open({
			multiple: false,
			directory: true,
			defaultPath: gamePathInput || '/Applications',
			title: 'Choose Crimson Desert.app or packages directory'
		});

		if (typeof selected === 'string') {
			gamePathInput = selected;
		}
	}

	async function detectInstall() {
		busy.detectingGame = true;
		clearMessage();
		try {
			const detected = await detectGameInstall();
			if (!detected) {
				message = {
					kind: 'info',
					title: 'No auto-detected install',
					message: 'Pick the Crimson Desert.app bundle or the packages directory manually.'
				};
				toast.info('No Crimson Desert install was auto-detected.');
				return;
			}

			gamePathInput = detected.packagesPath;
			await refreshDashboard();
			toast.success('Detected and saved the current Crimson Desert install.');
		} catch (error) {
			setError(error, 'Could not auto-detect the game install');
		} finally {
			busy.detectingGame = false;
		}
	}

	async function saveGamePath() {
		if (!gamePathInput.trim()) {
			message = {
				kind: 'info',
				title: 'Game path required',
				message: 'Choose a Crimson Desert.app bundle or packages directory first.'
			};
			toast.info('Pick a game path first.');
			return;
		}

		busy.settingGame = true;
		clearMessage();
		try {
			const savedInstall = await setGameInstall(gamePathInput.trim());
			gamePathInput = savedInstall.packagesPath;
			await refreshDashboard();
			toast.success('Saved the game install path.');
		} catch (error) {
			setError(error, 'Could not save the game install path');
		} finally {
			busy.settingGame = false;
		}
	}

	async function chooseModFolder() {
		const selected = await open({
			multiple: false,
			directory: true,
			defaultPath: scanFolderPath || undefined,
			title: 'Choose a folder containing JSON or .modpatch files'
		});

		if (typeof selected === 'string') {
			scanFolderPath = selected;
			await scanFolder(selected);
		}
	}

	async function chooseModZip() {
		const selected = await open({
			multiple: false,
			directory: false,
			filters: [{ name: 'ZIP archives', extensions: ['zip'] }],
			defaultPath: scanFolderPath || undefined,
			title: 'Choose a ZIP archive containing Crimson Desert mods'
		});

		if (typeof selected === 'string') {
			scanFolderPath = selected;
			await scanFolder(selected);
		}
	}

	async function choosePathcFile() {
		const selected = await open({
			multiple: false,
			directory: false,
			filters: [{ name: 'PATHC files', extensions: ['pathc'] }],
			defaultPath: pathcPathInput || gamePathInput || undefined,
			title: 'Choose a .pathc file'
		});

		if (typeof selected === 'string') {
			pathcPathInput = selected;
			await refreshPathcSummary();
		}
	}

	async function choosePathcFolder() {
		const selected = await open({
			multiple: false,
			directory: true,
			defaultPath: pathcFolderInput || scanFolderPath || undefined,
			title: 'Choose a folder containing DDS files'
		});

		if (typeof selected === 'string') {
			pathcFolderInput = selected;
		}
	}

	async function scanFolder(path: string) {
		busy.scanningMods = true;
		clearMessage();
		try {
			scanResults = await scanModFolder(path);
			if (scanResults.length === 0) {
				message = {
					kind: 'info',
					title: 'No modpatches found',
					message: 'The selected folder does not contain valid .json or .modpatch variants.'
				};
				toast.info('No valid modpatch files were found in that folder.');
				return;
			}

			toast.success(`Found ${scanResults.length} importable mod variant(s).`);
		} catch (error) {
			setError(error, 'Could not scan the selected folder');
		} finally {
			busy.scanningMods = false;
		}
	}

	async function importScanResult(result: ScanResult) {
		busy.importing = true;
		clearMessage();
		try {
			dashboard = await importModVariant(result.path, true);
			lastApplyResult = null;
			toast.success(`Imported ${result.fileName}.`);
		} catch (error) {
			setError(error, `Could not import ${result.fileName}`);
		} finally {
			busy.importing = false;
		}
	}

	async function toggleMod(mod: ModRecord) {
		busy.toggling = mod.id;
		clearMessage();
		try {
			dashboard = await setModEnabled(mod.id, !mod.enabled);
			toast.success(`${mod.name} ${mod.enabled ? 'disabled' : 'enabled'}.`);
		} catch (error) {
			setError(error, `Could not update ${mod.name}`);
		} finally {
			busy.toggling = '';
		}
	}

	async function runApply() {
		busy.applying = true;
		clearMessage();
		try {
			lastApplyResult = await applyMods();
			await refreshDashboard();
			toast.success(lastApplyResult.message);
		} catch (error) {
			setError(error, 'Could not apply the enabled mods');
		} finally {
			busy.applying = false;
		}
	}

	async function runRestore() {
		busy.restoring = true;
		clearMessage();
		try {
			dashboard = await restoreVanilla();
			lastApplyResult = null;
			applyPreview = null;
			toast.success('Restored the game overlay to vanilla.');
		} catch (error) {
			setError(error, 'Could not restore the vanilla overlay');
		} finally {
			busy.restoring = false;
		}
	}

	async function runReset() {
		busy.resetting = true;
		clearMessage();
		try {
			dashboard = await resetActiveMods();
			lastApplyResult = null;
			applyPreview = null;
			resetDialogOpen = false;
			toast.success('Disabled every active mod and restored vanilla files.');
		} catch (error) {
			setError(error, 'Could not reset the active mod set');
		} finally {
			busy.resetting = false;
		}
	}

	async function runLaunch() {
		busy.launching = true;
		clearMessage();
		try {
			await launchGame();
			toast.success('Launching Crimson Desert.');
		} catch (error) {
			setError(error, 'Could not launch the game');
		} finally {
			busy.launching = false;
		}
	}

	async function runFixEverything() {
		busy.fixing = true;
		clearMessage();
		try {
			dashboard = await fixEverything();
			lastApplyResult = null;
			applyPreview = null;
			toast.success('Reset the manager state and restored vanilla files.');
		} catch (error) {
			setError(error, 'Could not run Fix Everything');
		} finally {
			busy.fixing = false;
		}
	}

	async function runPathcRepack() {
		if (!pathcFolderInput.trim()) {
			toast.info('Choose a DDS folder first.');
			return;
		}

		clearMessage();
		busy.repackingPathc = true;
		try {
			pathcResult = await repackPathc(pathcPathInput || null, pathcFolderInput.trim());
			await refreshPathcSummary();
			toast.success(`Repacked PATHC with ${pathcResult.processedCount} DDS file(s).`);
		} catch (error) {
			setError(error, 'Could not repack PATHC');
		} finally {
			busy.repackingPathc = false;
		}
	}

	async function chooseExtractOutputDir() {
		const selected = await open({
			multiple: false,
			directory: true,
			defaultPath: extractOutputDirInput || undefined,
			title: 'Choose an output folder for extracted files'
		});

		if (typeof selected === 'string') {
			extractOutputDirInput = selected;
		}
	}

	async function refreshExtractPreview() {
		if (!extractVirtualPathInput.trim()) {
			extractPreview = null;
			return;
		}

		busy.extracting = true;
		try {
			extractPreview = await getVirtualFilePreview(extractVirtualPathInput.trim(), extractSourceGroupInput.trim() || null);
		} catch (error) {
			extractPreview = null;
			setError(error, 'Could not inspect virtual file');
		} finally {
			busy.extracting = false;
		}
	}

	async function runExtractVirtualFile() {
		if (!extractOutputDirInput.trim()) {
			toast.info('Choose an output folder first.');
			return;
		}

		busy.extracting = true;
		clearMessage();
		try {
			extractResult = await extractVirtualFile(
				extractVirtualPathInput.trim(),
				extractSourceGroupInput.trim() || null,
				extractOutputDirInput.trim()
			);
			await refreshExtractPreview();
			toast.success(`Extracted ${extractResult.virtualPath}.`);
		} catch (error) {
			setError(error, 'Could not extract virtual file');
		} finally {
			busy.extracting = false;
		}
	}

	async function chooseLanguage(language: string | null) {
		clearMessage();
		try {
			dashboard = await setSelectedLanguage(language);
			toast.success(language ? `Selected ${language.toUpperCase()} for language overlays.` : 'Cleared the selected language.');
		} catch (error) {
			setError(error, 'Could not update the selected language');
		}
	}

	async function classifyMod(mod: ModRecord, modKind: ModKind) {
		clearMessage();
		if (modKind === 'language' && !selectedLanguage) {
			toast.info('Choose the current in-game language first.');
			return;
		}

		try {
			dashboard = await setModClassification(mod.id, modKind, modKind === 'language' ? selectedLanguage : null);
			toast.success(`${mod.name} is now classified as ${modKind.replace('_', ' ')}.`);
		} catch (error) {
			setError(error, `Could not update ${mod.name}`);
		}
	}

	async function moveMod(mod: ModRecord, direction: 'up' | 'down') {
		clearMessage();
		try {
			dashboard = await moveModInLoadOrder(mod.id, direction);
			await refreshPatchSummaries();
			await refreshPreview();
			toast.success(`${mod.name} moved ${direction} in JSON load order.`);
		} catch (error) {
			setError(error, `Could not move ${mod.name}`);
		}
	}

	async function togglePatch(patch: ModPatchSummary) {
		clearMessage();
		try {
			dashboard = await setPatchEnabled(patch.modId, patch.patchIndex, !patch.enabled);
			await refreshPatchSummaries();
			await refreshPreview();
			toast.success(`${patch.title} ${patch.enabled ? 'disabled' : 'enabled'}.`);
		} catch (error) {
			setError(error, `Could not update ${patch.title}`);
		}
	}

	function modKindLabel(modKind: ModKind) {
		if (modKind === 'json_data') return 'JSON';
		if (modKind === 'precompiled_overlay') return 'Precompiled';
		if (modKind === 'browser_raw') return 'Browser/Raw';
		return 'Language';
	}

	function fallbackKindForLanguageMod(mod: ModRecord): ModKind {
		if (mod.libraryPath.endsWith('/files') || mod.libraryPath.includes('/files/')) {
			return 'browser_raw';
		}
		return mod.targetFiles.every((target) => /^\d{4}$/.test(target)) ? 'precompiled_overlay' : 'json_data';
	}

	function clearMessage() {
		message = null;
	}

	function setError(error: unknown, title: string) {
		const details = toMessage(error);
		message = { kind: 'error', title, message: details };
		toast.error(title, { description: details });
	}

	function toMessage(error: unknown) {
		if (typeof error === 'string') return error;
		if (typeof error === 'object' && error && 'message' in error && typeof error.message === 'string') {
			return error.message;
		}
		return 'Something went wrong.';
	}

	function formatTimestamp(value: string) {
		const seconds = Number(value);
		if (!Number.isFinite(seconds) || seconds <= 0) {
			return 'Unknown';
		}

		return new Date(seconds * 1000).toLocaleString();
	}

	function modActionLabel(mod: ModRecord) {
		if (busy.toggling === mod.id) return 'Saving...';
		return mod.enabled ? 'Disable' : 'Enable';
	}
</script>

<svelte:head>
	<title>Crimson Desert Mod Manager</title>
</svelte:head>

<AlertDialog.Root bind:open={resetDialogOpen}>
	<div class="mx-auto flex min-h-full w-full max-w-5xl flex-col gap-6 px-4 py-6 sm:px-6 lg:px-8 lg:py-8">
		<section id="overview" class="scroll-mt-24 space-y-4">
			<div class="space-y-2">
				<p class="text-muted-foreground text-xs font-medium uppercase tracking-[0.24em]">Overview</p>
				<h1 class="text-3xl font-semibold tracking-tight sm:text-4xl">Mod workbench for the macOS build</h1>
				<p class="text-muted-foreground max-w-3xl text-sm leading-7 sm:text-base">
					The shell now reflects the real tool direction: sidebar-driven navigation, one-column workflow, and dedicated lanes for data mods,
					precompiled overlays, language targeting, and advanced format tooling.
				</p>
			</div>

			<div class="flex flex-wrap gap-2">
				<Badge variant="outline">{totalCount} in library</Badge>
				<Badge>{enabledCount} enabled</Badge>
				<Badge variant="secondary">{disabledCount} archived</Badge>
				<Badge variant={overlayActive ? 'default' : 'outline'}>{overlayActive ? 'Overlay active' : 'Vanilla'}</Badge>
				<Badge variant={backupExists ? 'outline' : 'secondary'}>{backupExists ? 'Backup present' : 'No backup yet'}</Badge>
			</div>

		{#if message}
			<Alert.Root variant={message.kind === 'error' ? 'destructive' : 'default'}>
				<AlertCircle class="size-4" />
				<Alert.Title>{message.title}</Alert.Title>
				<Alert.Description>{message.message}</Alert.Description>
			</Alert.Root>
		{/if}

		{#if recoveryPending}
			<Alert.Root variant="destructive">
				<AlertCircle class="size-4" />
				<Alert.Title>Recovery recommended</Alert.Title>
				<Alert.Description>
					The last operation may have been interrupted{#if pendingOperation} during `{pendingOperation}`{/if}. Run `Fix Everything` in Tools to restore a clean manager state.
				</Alert.Description>
			</Alert.Root>
		{/if}

			<Card.Root>
				<Card.Header>
					<Card.Title class="flex items-center gap-2"><Sparkles class="size-5" /> Current direction</Card.Title>
					<Card.Description>
						The backend already handles JSON import and overlay application. The next implementation passes will add dynamic numeric group
						management, load order, precompiled overlays, and language-targeted installs.
					</Card.Description>
				</Card.Header>
			</Card.Root>
		</section>

		<section id="data-mods" class="scroll-mt-24 space-y-4">
			<div class="space-y-2">
				<p class="text-muted-foreground text-xs font-medium uppercase tracking-[0.24em]">Data Mods</p>
				<h2 class="text-2xl font-semibold tracking-tight">JSON mod workflow</h2>
				<p class="text-muted-foreground max-w-3xl text-sm leading-7">
					Entry-based JSON mods live here. The current pass supports scanning, importing, enabling, and applying them; load order and overlap
					intelligence are the next backend milestone.
				</p>
			</div>

			<Card.Root>
				<Card.Header>
					<Card.Title class="flex items-center gap-2"><HardDriveDownload class="size-5" /> Import from folder</Card.Title>
					<Card.Description>Scan a folder or ZIP archive for JSON, precompiled, and browser/raw variants, then import exactly one candidate at a time.</Card.Description>
				</Card.Header>
				<Card.Content class="space-y-4">
					<div class="flex flex-wrap gap-2">
						<Button class="w-full sm:w-auto" disabled={busy.scanningMods} onclick={chooseModFolder}>
							<FolderSearch class="size-4" />
							{busy.scanningMods ? 'Scanning folder...' : 'Choose mod folder'}
						</Button>
						<Button class="w-full sm:w-auto" variant="outline" disabled={busy.scanningMods} onclick={chooseModZip}>
							<Archive class="size-4" />
							{busy.scanningMods ? 'Scanning archive...' : 'Choose ZIP archive'}
						</Button>
					</div>

					{#if scanFolderPath}
						<p class="text-muted-foreground text-sm break-all">{scanFolderPath}</p>
					{/if}

					{#if scanResults.length === 0}
						<Empty.Root class="min-h-44 border-dashed bg-muted/20 p-8">
							<Empty.Header>
								<Empty.Title>No scanned variants yet</Empty.Title>
								<Empty.Description>Choose a folder to preview compatible JSON mod variants.</Empty.Description>
							</Empty.Header>
						</Empty.Root>
					{:else}
						<ScrollArea.Root class="h-96 rounded-xl border">
							<div class="space-y-3 p-3">
								{#each scanResults as result (result.path)}
									<Collapsible.Root open={Boolean(scanDetailsOpen[result.path])} class="rounded-xl border bg-muted/20 px-4 py-4">
										<div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
											<div class="space-y-2">
												<p class="font-medium">{result.name}</p>
												<p class="text-muted-foreground text-sm">{result.fileName}</p>
												<div class="flex flex-wrap gap-2">
													<Badge variant="secondary">{modKindLabel(result.modKind)}</Badge>
													<Badge variant="outline">{result.patchCount} patch groups</Badge>
													<Badge variant="outline">{result.changeCount} byte changes</Badge>
													<Badge variant={result.missingFiles.length === 0 ? 'default' : 'secondary'}>
														{result.resolvableFiles}/{result.targetFiles.length} resolvable
													</Badge>
												</div>
											</div>
											<div class="flex gap-2">
												<Button variant="outline" size="sm" onclick={() => (scanDetailsOpen[result.path] = !scanDetailsOpen[result.path])}>
													{scanDetailsOpen[result.path] ? 'Hide details' : 'Details'}
												</Button>
												<Button size="sm" disabled={busy.importing} onclick={() => importScanResult(result)}>Import</Button>
											</div>
										</div>

										<Collapsible.Content class="pt-4">
											<Separator class="mb-4" />
											{#if result.description}
												<p class="text-muted-foreground text-sm leading-6">{result.description}</p>
											{/if}
											<div class="mt-4 flex flex-wrap gap-2">
												{#each result.targetFiles as target (target)}
													<Badge variant="outline">{target}</Badge>
												{/each}
											</div>
											{#if result.missingFiles.length > 0}
												<Alert.Root variant="destructive" class="mt-4">
													<ShieldAlert class="size-4" />
													<Alert.Title>Missing target files</Alert.Title>
													<Alert.Description>{result.missingFiles.join(', ')}</Alert.Description>
												</Alert.Root>
											{/if}
										</Collapsible.Content>
									</Collapsible.Root>
								{/each}
							</div>
						</ScrollArea.Root>
					{/if}
				</Card.Content>
			</Card.Root>

			<Card.Root>
				<Card.Header>
					<Card.Title class="flex items-center gap-2"><ArrowDownUp class="size-5" /> Load order and overlap</Card.Title>
					<Card.Description>
						Enabled JSON mods are merged in the order shown below. Lower items win when they target the same entry or byte range.
					</Card.Description>
				</Card.Header>
				<Card.Content>
					{#if orderedJsonMods.length === 0}
						<Alert.Root>
							<Info class="size-4" />
							<Alert.Title>No enabled JSON mods</Alert.Title>
							<Alert.Description>Enable one or more JSON mods to establish an apply order.</Alert.Description>
						</Alert.Root>
					{:else}
						<div class="space-y-3">
							{#each orderedJsonMods as mod, index (mod.id)}
								<div class="flex flex-col gap-3 rounded-xl border bg-muted/20 px-4 py-4 sm:flex-row sm:items-center sm:justify-between">
									<div>
										<p class="font-medium">{index + 1}. {mod.name}</p>
										<p class="text-muted-foreground text-sm">{mod.fileName}</p>
									</div>
									<div class="flex gap-2">
										<Button variant="outline" size="sm" disabled={index === 0} onclick={() => moveMod(mod, 'up')}>Up</Button>
										<Button variant="outline" size="sm" disabled={index === orderedJsonMods.length - 1} onclick={() => moveMod(mod, 'down')}>Down</Button>
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</Card.Content>
			</Card.Root>

			<Card.Root>
				<Card.Header>
					<Card.Title class="flex items-center gap-2"><Info class="size-5" /> Patch toggles</Card.Title>
					<Card.Description>Enable or disable individual JSON patch groups before preview and apply.</Card.Description>
				</Card.Header>
				<Card.Content class="space-y-4">
					{#if orderedJsonMods.length === 0}
						<Alert.Root>
							<Info class="size-4" />
							<Alert.Title>No JSON patch groups available</Alert.Title>
							<Alert.Description>Enable at least one JSON mod to manage individual patch groups.</Alert.Description>
						</Alert.Root>
					{:else}
						<div class="flex flex-wrap gap-2">
							{#each orderedJsonMods as mod (mod.id)}
								<Button variant={selectedPatchModId === mod.id ? 'default' : 'outline'} size="sm" onclick={async () => { selectedPatchModId = mod.id; await refreshPatchSummaries(); }}>
									{mod.name}
								</Button>
							{/each}
						</div>

						<ScrollArea.Root class="h-72 rounded-xl border">
							<div class="divide-y">
								{#each patchSummaries as patch (patch.modId + ':' + patch.patchIndex)}
									<div class="flex flex-col gap-3 px-4 py-3 sm:flex-row sm:items-center sm:justify-between">
										<div>
											<p class="text-sm font-medium">{patch.title}</p>
											<p class="text-muted-foreground text-xs">{patch.sourceGroup} / {patch.gameFile}</p>
											<p class="text-muted-foreground text-xs">{patch.changeCount} byte changes</p>
										</div>
										<Button variant={patch.enabled ? 'outline' : 'secondary'} size="sm" disabled={busy.patches} onclick={() => togglePatch(patch)}>
											{patch.enabled ? 'Disable' : 'Enable'}
										</Button>
									</div>
								{:else}
									<div class="px-4 py-8 text-sm text-muted-foreground">No patch groups found for the selected mod.</div>
								{/each}
							</div>
						</ScrollArea.Root>
					{/if}
				</Card.Content>
			</Card.Root>
		</section>

		<section id="language-mods" class="scroll-mt-24 space-y-4">
			<div class="space-y-2">
				<p class="text-muted-foreground text-xs font-medium uppercase tracking-[0.24em]">Language Mods</p>
				<h2 class="text-2xl font-semibold tracking-tight">Language-targeted overlays</h2>
			</div>
			<Card.Root>
				<Card.Header>
					<Card.Title class="flex items-center gap-2"><Globe2 class="size-5" /> Planned language lane</Card.Title>
					<Card.Description>
						Language mods are now tied to a selected in-game language. Only language-classified mods that match this selection are installed on apply.
					</Card.Description>
				</Card.Header>
				<Card.Content class="space-y-4">
					<div class="flex flex-wrap gap-2">
						<Button variant={selectedLanguage === null ? 'default' : 'outline'} size="sm" onclick={() => chooseLanguage(null)}>None</Button>
						{#each languageOptions as language (language)}
							<Button variant={selectedLanguage === language ? 'default' : 'outline'} size="sm" onclick={() => chooseLanguage(language)}>
								{language.toUpperCase()}
							</Button>
						{/each}
					</div>
					<p class="text-muted-foreground text-sm">Selected language: <span class="text-foreground font-medium">{selectedLanguage?.toUpperCase() ?? 'Not set'}</span></p>

					{#if languageMods.length === 0}
						<Empty.Root class="min-h-36 border-dashed bg-muted/20 p-8">
							<Empty.Header>
								<Empty.Title>No language mods yet</Empty.Title>
								<Empty.Description>Classify imported JSON or precompiled mods as language mods once you know the target language.</Empty.Description>
							</Empty.Header>
						</Empty.Root>
					{:else}
						<div class="space-y-3">
							{#each languageMods as mod (mod.id)}
								<div class="rounded-xl border bg-muted/20 px-4 py-4">
									<div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
										<div>
											<p class="font-medium">{mod.name}</p>
											<p class="text-muted-foreground text-sm">{mod.fileName}</p>
										</div>
										<div class="flex flex-wrap gap-2">
											<Badge>{mod.language?.toUpperCase() ?? 'Unassigned'}</Badge>
											<Badge variant="outline">{modKindLabel(mod.modKind)}</Badge>
										</div>
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</Card.Content>
			</Card.Root>
		</section>

		<section id="precompiled-mods" class="scroll-mt-24 space-y-4">
			<div class="space-y-2">
				<p class="text-muted-foreground text-xs font-medium uppercase tracking-[0.24em]">Precompiled Mods</p>
				<h2 class="text-2xl font-semibold tracking-tight">Folder-based precompiled overlays</h2>
			</div>
			<Card.Root>
				<Card.Header>
					<Card.Title class="flex items-center gap-2"><Package class="size-5" /> Planned precompiled support</Card.Title>
					<Card.Description>
						Precompiled overlays with numeric group folders now import and install into fresh manager-owned groups during apply.
					</Card.Description>
				</Card.Header>
				<Card.Content>
					{#if precompiledMods.length === 0}
						<Empty.Root class="min-h-32 border-dashed bg-muted/20 p-8">
							<Empty.Header>
								<Empty.Title>No precompiled overlays imported</Empty.Title>
								<Empty.Description>Imported numeric-group mods will appear here once scanned from folder packages like `item_price_display`.</Empty.Description>
							</Empty.Header>
						</Empty.Root>
					{:else}
						<div class="space-y-3">
							{#each precompiledMods as mod (mod.id)}
								<div class="rounded-xl border bg-muted/20 px-4 py-4">
									<div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
										<div>
											<p class="font-medium">{mod.name}</p>
											<p class="text-muted-foreground text-sm">{mod.fileName}</p>
										</div>
										<div class="flex flex-wrap gap-2">
											<Badge variant="outline">{mod.targetFiles.length} source groups</Badge>
											<Button variant="outline" size="sm" onclick={() => classifyMod(mod, 'language')}>Use as language</Button>
										</div>
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</Card.Content>
			</Card.Root>
		</section>

		<section id="library" class="scroll-mt-24 space-y-4">
			<div class="space-y-2">
				<p class="text-muted-foreground text-xs font-medium uppercase tracking-[0.24em]">Library</p>
				<h2 class="text-2xl font-semibold tracking-tight">Archive-first mod inventory</h2>
			</div>

			<div class="space-y-2">
				<Label for="mod-search">Search library</Label>
				<Input id="mod-search" bind:value={search} placeholder="Search names, files, and target paths" />
			</div>

			<div class="flex flex-wrap gap-2">
				<Button variant={filter === 'all' ? 'default' : 'outline'} size="sm" onclick={() => (filter = 'all')}>All</Button>
				<Button variant={filter === 'enabled' ? 'default' : 'outline'} size="sm" onclick={() => (filter = 'enabled')}>Enabled</Button>
				<Button variant={filter === 'disabled' ? 'default' : 'outline'} size="sm" onclick={() => (filter = 'disabled')}>Archived</Button>
			</div>

			<Card.Root>
				<Card.Content class="pt-6">
					<ScrollArea.Root class="h-[34rem] rounded-xl border">
						{#if filteredMods.length === 0}
							<Empty.Root class="border-0">
								<Empty.Header>
									<Empty.Title>No matching mods</Empty.Title>
									<Empty.Description>Import a folder or change the current filter to populate the library.</Empty.Description>
								</Empty.Header>
							</Empty.Root>
						{:else}
							<Table.Root>
								<Table.Header>
									<Table.Row>
										<Table.Head>Mod</Table.Head>
										<Table.Head>Targets</Table.Head>
										<Table.Head>Imported</Table.Head>
										<Table.Head class="text-right">State</Table.Head>
									</Table.Row>
								</Table.Header>
								<Table.Body>
									{#each filteredMods as mod (mod.id)}
										<Table.Row>
											<Table.Cell class="align-top">
												<div class="space-y-2">
													<div class="flex flex-wrap items-center gap-2">
														<p class="font-medium">{mod.name}</p>
														<Badge variant="outline">{modKindLabel(mod.modKind)}</Badge>
														<Badge variant={mod.enabled ? 'default' : 'secondary'}>{mod.enabled ? 'Enabled' : 'Archived'}</Badge>
														{#if mod.language}
															<Badge>{mod.language.toUpperCase()}</Badge>
														{/if}
													</div>
													<p class="text-muted-foreground text-sm">{mod.fileName}</p>
													{#if mod.description}
														<p class="text-muted-foreground max-w-xl text-sm leading-6">{mod.description}</p>
													{/if}
												</div>
											</Table.Cell>
											<Table.Cell class="align-top">
												<div class="flex max-w-md flex-wrap gap-2 pt-1">
													{#each mod.targetFiles.slice(0, 3) as target (target)}
														<Badge variant="outline">{target}</Badge>
													{/each}
													{#if mod.targetFiles.length > 3}
														<Badge variant="secondary">+{mod.targetFiles.length - 3} more</Badge>
													{/if}
												</div>
											</Table.Cell>
											<Table.Cell class="text-muted-foreground align-top text-sm">{formatTimestamp(mod.importedAt)}</Table.Cell>
											<Table.Cell class="align-top text-right">
												<div class="flex justify-end gap-2">
													<Button variant={mod.enabled ? 'outline' : 'default'} size="sm" disabled={busy.toggling === mod.id} onclick={() => toggleMod(mod)}>
														{modActionLabel(mod)}
													</Button>
													{#if mod.modKind === 'language'}
														<Button variant="outline" size="sm" onclick={() => classifyMod(mod, fallbackKindForLanguageMod(mod))}>Unset language</Button>
													{:else}
														<Button variant="outline" size="sm" onclick={() => classifyMod(mod, 'language')}>Language</Button>
													{/if}
												</div>
											</Table.Cell>
										</Table.Row>
									{/each}
								</Table.Body>
							</Table.Root>
						{/if}
					</ScrollArea.Root>
				</Card.Content>
			</Card.Root>
		</section>

		<section id="apply-logs" class="scroll-mt-24 space-y-4">
			<div class="space-y-2">
				<p class="text-muted-foreground text-xs font-medium uppercase tracking-[0.24em]">Apply & Logs</p>
				<h2 class="text-2xl font-semibold tracking-tight">Overlay lifecycle</h2>
			</div>

			<div class="flex flex-wrap gap-2">
				<Button variant="outline" disabled={busy.previewing} onclick={refreshPreview}>{busy.previewing ? 'Refreshing preview...' : 'Refresh preview'}</Button>
				<Button disabled={!install || enabledCount === 0 || busy.applying} onclick={runApply}><Sparkles class="size-4" /> Apply enabled mods</Button>
				<Button variant="outline" disabled={!install || busy.restoring} onclick={runRestore}><RefreshCcw class="size-4" /> Restore vanilla overlay</Button>
				<Button variant="destructive" disabled={busy.resetting} onclick={() => (resetDialogOpen = true)}>Reset active mods</Button>
			</div>

			<Card.Root>
				<Card.Header>
					<Card.Title class="flex items-center gap-2"><Info class="size-5" /> Apply preview</Card.Title>
					<Card.Description>Preview the active mod set before creating fresh manager-owned overlay groups.</Card.Description>
				</Card.Header>
				<Card.Content class="space-y-4">
					{#if !applyPreview}
						<Alert.Root>
							<Info class="size-4" />
							<Alert.Title>No preview available</Alert.Title>
							<Alert.Description>Save a valid game path and enable at least one mod to build an apply preview.</Alert.Description>
						</Alert.Root>
					{:else}
						<div class="flex flex-wrap gap-2">
							<Badge variant="outline">{applyPreview.modCount} active mods</Badge>
							<Badge variant="outline">{applyPreview.targetFileCount} JSON targets</Badge>
							<Badge variant="outline">{applyPreview.estimatedGroupCount} estimated groups</Badge>
							{#if applyPreview.selectedLanguage}
								<Badge>{applyPreview.selectedLanguage.toUpperCase()}</Badge>
							{/if}
						</div>

						<div class="grid gap-3 sm:grid-cols-3">
							<div class="rounded-xl border bg-muted/20 p-4">
								<p class="text-muted-foreground text-xs uppercase tracking-[0.18em]">JSON mods</p>
								<p class="mt-2 text-2xl font-semibold">{applyPreview.jsonModCount}</p>
							</div>
							<div class="rounded-xl border bg-muted/20 p-4">
								<p class="text-muted-foreground text-xs uppercase tracking-[0.18em]">Precompiled</p>
								<p class="mt-2 text-2xl font-semibold">{applyPreview.precompiledModCount}</p>
							</div>
							<div class="rounded-xl border bg-muted/20 p-4">
								<p class="text-muted-foreground text-xs uppercase tracking-[0.18em]">Browser/raw</p>
								<p class="mt-2 text-2xl font-semibold">{applyPreview.browserRawModCount}</p>
							</div>
						</div>

						<ScrollArea.Root class="h-72 rounded-xl border">
							<div class="divide-y">
								{#each applyPreview.files as file (file.sourceGroup + ':' + file.gameFile)}
									<div class="space-y-2 px-4 py-3">
										<div class="flex flex-wrap items-center justify-between gap-3">
											<div>
												<p class="text-sm font-medium break-all">{file.gameFile}</p>
												<p class="text-muted-foreground text-xs">{file.sourceGroup}{#if file.sourcePazIndex !== null} / PAZ {file.sourcePazIndex}{/if}</p>
											</div>
											<div class="flex flex-wrap gap-2">
												<Badge variant={file.resolved ? 'outline' : 'secondary'}>{file.resolved ? 'Resolved' : 'Unresolved'}</Badge>
												<Badge variant="outline">{file.changeCount} changes</Badge>
												{#if file.overlapCount > 0}
													<Badge>{file.overlapCount} overlaps</Badge>
												{/if}
											</div>
										</div>
										<p class="text-muted-foreground text-xs">Mods: {file.sourceMods.join(', ')}</p>
										{#if file.reason}
											<p class="text-destructive text-xs">{file.reason}</p>
										{/if}
									</div>
								{/each}
							</div>
						</ScrollArea.Root>
					{/if}
				</Card.Content>
			</Card.Root>

			<Card.Root>
				<Card.Header>
					<Card.Title class="flex items-center gap-2"><Layers3 class="size-5" /> Current overlay state</Card.Title>
				</Card.Header>
				<Card.Content class="space-y-2 text-sm text-muted-foreground">
					<p>Overlay: {overlayActive ? '0036 active' : 'vanilla'}</p>
					<p>Backup: {backupExists ? '0.papgt.bak present' : 'not created yet'}</p>
					<p>Enabled mods queued: {enabledCount}</p>
					<p>Writable install: {install?.writable ? 'yes' : 'unknown / no'}</p>
				</Card.Content>
			</Card.Root>

			{#if lastApplyResult}
				<Card.Root>
					<Card.Header>
						<Card.Title class="flex items-center gap-2"><CheckCircle2 class="size-5" /> Last apply result</Card.Title>
						<Card.Description>{lastApplyResult.message}</Card.Description>
					</Card.Header>
					<Card.Content class="space-y-4">
						<div class="flex flex-wrap gap-2">
							<Badge variant="outline">{lastApplyResult.modCount} mods</Badge>
							<Badge variant="outline">{lastApplyResult.overlayFileCount} files</Badge>
							<Badge variant="outline">{lastApplyResult.pazSize} byte PAZ</Badge>
						</div>
						<ScrollArea.Root class="h-60 rounded-xl border">
							<div class="divide-y">
								{#each lastApplyResult.files as file (file.gameFile)}
									<div class="space-y-1 px-4 py-3">
										<div class="flex items-start justify-between gap-3">
											<p class="text-sm font-medium break-all">{file.gameFile}</p>
											<Badge variant="secondary">PAZ {file.sourcePazIndex}</Badge>
										</div>
										<p class="text-muted-foreground text-xs">
											Applied {file.appliedChanges}, skipped {file.skippedChanges}
											{#if file.reason}
												- {file.reason}
											{/if}
										</p>
									</div>
								{/each}
							</div>
						</ScrollArea.Root>
					</Card.Content>
				</Card.Root>
			{/if}
		</section>

		<section id="tools" class="scroll-mt-24 space-y-4">
			<div class="space-y-2">
				<p class="text-muted-foreground text-xs font-medium uppercase tracking-[0.24em]">Tools</p>
				<h2 class="text-2xl font-semibold tracking-tight">Game path, restore, and launcher</h2>
			</div>

			<Card.Root>
				<Card.Header>
					<Card.Title class="flex items-center gap-2"><Gamepad2 class="size-5" /> Game install</Card.Title>
					<Card.Description>Saved target for apply, restore, and launch actions.</Card.Description>
				</Card.Header>
				<Card.Content class="space-y-4">
					<div class="space-y-2">
						<Label for="game-path">Game path</Label>
						<Input id="game-path" bind:value={gamePathInput} placeholder="Crimson Desert.app or packages path" />
					</div>
					<div class="flex flex-wrap gap-2">
						<Button variant="outline" onclick={chooseGamePath}>Browse</Button>
						<Button variant="outline" disabled={busy.detectingGame} onclick={detectInstall}>Detect</Button>
						<Button disabled={busy.settingGame} onclick={saveGamePath}>Save Path</Button>
						<Button variant="outline" disabled={!install || busy.launching} onclick={runLaunch}><Wrench class="size-4" /> Start game</Button>
						<Button variant="destructive" disabled={busy.fixing} onclick={runFixEverything}>{busy.fixing ? 'Fixing...' : 'Fix Everything'}</Button>
					</div>
					<p class="text-muted-foreground text-sm break-all">{install?.packagesPath ?? 'Not configured yet'}</p>
					{#if recoveryPending}
						<p class="text-destructive text-xs">Pending recovery marker{#if pendingOperation}: {pendingOperation}{/if}</p>
					{/if}
				</Card.Content>
			</Card.Root>
		</section>

		<section id="advanced" class="scroll-mt-24 space-y-4 pb-8">
			<div class="space-y-2">
				<p class="text-muted-foreground text-xs font-medium uppercase tracking-[0.24em]">Advanced</p>
				<h2 class="text-2xl font-semibold tracking-tight">PATHC and DDS workflow</h2>
			</div>
			<Card.Root>
				<Card.Header>
					<Card.Title class="flex items-center gap-2"><Image class="size-5" /> PATHC texture index</Card.Title>
					<Card.Description>
						Inspect `0.pathc`, verify virtual-path lookups, and repack a DDS folder into the PATHC index with a `.bak` safety backup.
					</Card.Description>
				</Card.Header>
				<Card.Content class="space-y-4">
					<div class="space-y-2">
						<Label for="pathc-path">PATHC file</Label>
						<div class="flex flex-wrap gap-2">
							<Input id="pathc-path" bind:value={pathcPathInput} placeholder=".../meta/0.pathc" />
							<Button variant="outline" onclick={choosePathcFile}>Browse</Button>
							<Button variant="outline" disabled={busy.pathc} onclick={refreshPathcSummary}>{busy.pathc ? 'Refreshing...' : 'Refresh'}</Button>
						</div>
					</div>

					<div class="space-y-2">
						<Label for="pathc-lookup">Lookup virtual path</Label>
						<div class="flex flex-wrap gap-2">
							<Input id="pathc-lookup" bind:value={pathcLookupInput} placeholder="/ui/texture/example.dds" />
							<Button variant="outline" disabled={busy.pathc} onclick={refreshPathcSummary}>Lookup</Button>
						</div>
					</div>

					{#if pathcSummary}
						<div class="grid gap-3 sm:grid-cols-3">
							<div class="rounded-xl border bg-muted/20 p-4">
								<p class="text-muted-foreground text-xs uppercase tracking-[0.18em]">DDS templates</p>
								<p class="mt-2 text-2xl font-semibold">{pathcSummary.ddsTemplateCount}</p>
							</div>
							<div class="rounded-xl border bg-muted/20 p-4">
								<p class="text-muted-foreground text-xs uppercase tracking-[0.18em]">Hashes</p>
								<p class="mt-2 text-2xl font-semibold">{pathcSummary.hashCount}</p>
							</div>
							<div class="rounded-xl border bg-muted/20 p-4">
								<p class="text-muted-foreground text-xs uppercase tracking-[0.18em]">Collisions</p>
								<p class="mt-2 text-2xl font-semibold">{pathcSummary.collisionPathCount}</p>
							</div>
						</div>

						{#if pathcSummary.lookups.length > 0}
							<ScrollArea.Root class="h-44 rounded-xl border">
								<div class="divide-y">
									{#each pathcSummary.lookups as lookup (lookup.virtualPath)}
										<div class="space-y-2 px-4 py-3">
											<div class="flex flex-wrap items-center gap-2">
												<p class="text-sm font-medium break-all">{lookup.virtualPath}</p>
												<Badge variant={lookup.found ? 'outline' : 'secondary'}>{lookup.found ? 'Found' : 'Missing'}</Badge>
											</div>
											<p class="text-muted-foreground text-xs">Hash: 0x{lookup.keyHash.toString(16).toUpperCase()}</p>
											{#if lookup.found}
												<p class="text-muted-foreground text-xs">
													DDS {lookup.directDdsIndex} / {lookup.width}x{lookup.height} / mip {lookup.mipCount} / {lookup.formatLabel}
												</p>
											{/if}
										</div>
									{/each}
								</div>
							</ScrollArea.Root>
						{/if}
					{/if}

					<div class="space-y-2">
						<Label for="pathc-folder">DDS source folder</Label>
						<div class="flex flex-wrap gap-2">
							<Input id="pathc-folder" bind:value={pathcFolderInput} placeholder="Folder containing DDS files" />
							<Button variant="outline" onclick={choosePathcFolder}>Choose folder</Button>
							<Button disabled={busy.repackingPathc} onclick={runPathcRepack}>{busy.repackingPathc ? 'Repacking...' : 'Repack PATHC'}</Button>
						</div>
					</div>

					{#if pathcResult}
						<Alert.Root>
							<Info class="size-4" />
							<Alert.Title>Last PATHC repack</Alert.Title>
							<Alert.Description>
								Processed {pathcResult.processedCount} DDS file(s), updated {pathcResult.updatedCount} hash entries, added {pathcResult.addedTemplateCount} new DDS templates. Backup: {pathcResult.backupPath}
							</Alert.Description>
						</Alert.Root>
					{/if}
				</Card.Content>
			</Card.Root>

			<Card.Root>
				<Card.Header>
					<Card.Title class="flex items-center gap-2"><Archive class="size-5" /> Virtual file extraction</Card.Title>
					<Card.Description>
						Preview a virtual file in the game archives and extract its decompressed bytes into a chosen output folder.
					</Card.Description>
				</Card.Header>
				<Card.Content class="space-y-4">
					<div class="grid gap-4 sm:grid-cols-2">
						<div class="space-y-2 sm:col-span-2">
							<Label for="extract-virtual-path">Virtual path</Label>
							<Input id="extract-virtual-path" bind:value={extractVirtualPathInput} placeholder="character/player/playeractiongraph_main.xml" />
						</div>
						<div class="space-y-2">
							<Label for="extract-source-group">Preferred source group</Label>
							<Input id="extract-source-group" bind:value={extractSourceGroupInput} placeholder="0008" />
						</div>
						<div class="space-y-2">
							<Label for="extract-output">Output folder</Label>
							<div class="flex flex-wrap gap-2">
								<Input id="extract-output" bind:value={extractOutputDirInput} placeholder="Choose an output folder" />
								<Button variant="outline" onclick={chooseExtractOutputDir}>Browse</Button>
							</div>
						</div>
					</div>

					<div class="flex flex-wrap gap-2">
						<Button variant="outline" disabled={busy.extracting} onclick={refreshExtractPreview}>{busy.extracting ? 'Inspecting...' : 'Inspect file'}</Button>
						<Button disabled={busy.extracting} onclick={runExtractVirtualFile}>{busy.extracting ? 'Extracting...' : 'Extract file'}</Button>
					</div>

					{#if extractPreview}
						<div class="rounded-xl border bg-muted/20 p-4 text-sm">
							<div class="flex flex-wrap items-center gap-2">
								<p class="font-medium break-all">{extractPreview.virtualPath}</p>
								<Badge variant={extractPreview.resolved ? 'outline' : 'secondary'}>{extractPreview.resolved ? 'Resolved' : 'Missing'}</Badge>
							</div>
							<p class="text-muted-foreground mt-2 text-xs">Source group: {extractPreview.sourceGroup}</p>
							{#if extractPreview.resolved}
								<p class="text-muted-foreground mt-1 text-xs">
									{extractPreview.resolvedGameFile} / PAZ {extractPreview.sourcePazIndex} / {extractPreview.compressedSize} compressed / {extractPreview.decompressedSize} decompressed / flags {extractPreview.flags}
								</p>
							{:else if extractPreview.reason}
								<p class="text-destructive mt-1 text-xs">{extractPreview.reason}</p>
							{/if}
						</div>
					{/if}

					{#if extractResult}
						<Alert.Root>
							<Info class="size-4" />
							<Alert.Title>Last extraction</Alert.Title>
							<Alert.Description>
								Extracted {extractResult.virtualPath} from {extractResult.sourceGroup} to {extractResult.outputPath} ({extractResult.decompressedSize} bytes).
							</Alert.Description>
						</Alert.Root>
					{/if}
				</Card.Content>
			</Card.Root>
		</section>
	</div>

	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>Reset active mods?</AlertDialog.Title>
			<AlertDialog.Description>
				This restores the game to vanilla and disables every active mod in the library. Archived imports remain available for later re-enabling.
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
			<AlertDialog.Action disabled={busy.resetting} onclick={runReset}>Reset active mods</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
