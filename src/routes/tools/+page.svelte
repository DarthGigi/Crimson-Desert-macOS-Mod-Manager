<script lang="ts">
	import * as Alert from '$lib/components/ui/alert';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { manager } from '$lib/manager-state.svelte';
	import { AlertCircle, Gamepad2, RefreshCw, Sparkles, Wrench } from '@lucide/svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import { onMount } from 'svelte';

	let gamePathInput = $state('');
	let reportOutputPath = $state('');

	onMount(async () => {
		await manager.ensureLoaded();
		gamePathInput = manager.install?.packagesPath ?? '';
		await Promise.all([manager.refreshIsolationSession(), manager.verifyGameState()]);
	});

	async function chooseGamePath() {
		const selected = await open({
			multiple: false,
			directory: true,
			defaultPath: gamePathInput || '/Users/gigi/Games',
			title: 'Choose Crimson Desert.app or packages directory'
		});
		if (typeof selected === 'string') gamePathInput = selected;
	}

	async function chooseReportOutput() {
		const selected = await open({
			multiple: false,
			directory: false,
			defaultPath: reportOutputPath || undefined,
			title: 'Choose where to save the diagnostic report'
		});
		if (typeof selected === 'string') reportOutputPath = selected;
	}
</script>

<svelte:head><title>Tools • Crimson Desert Mod Workbench</title></svelte:head>

<div
	class="mx-auto flex min-h-full w-full max-w-5xl flex-col gap-6 px-4 py-6 sm:px-6 lg:px-8 lg:py-8"
>
	<div class="space-y-2">
		<p class="text-xs font-medium tracking-[0.24em] text-muted-foreground uppercase">Tools</p>
		<h1 class="text-3xl font-semibold tracking-tight">Game path, recovery, and launcher</h1>
		<p class="max-w-3xl text-sm leading-7 text-muted-foreground">
			Configure the macOS game install, launch the app bundle, and restore or fully reset the
			manager state when needed.
		</p>
	</div>
	<div class="grid gap-4 md:grid-cols-3">
		<Card.Root
			><Card.Content class="pt-6"
				><p class="text-sm font-medium">Normal recovery</p>
				<p class="mt-2 text-sm text-muted-foreground">
					Use `Restore vanilla overlay` or `Reset active mods` after a failed apply or when
					switching setups.
				</p></Card.Content
			></Card.Root
		>
		<Card.Root
			><Card.Content class="pt-6"
				><p class="text-sm font-medium">Full reset</p>
				<p class="mt-2 text-sm text-muted-foreground">
					Use `Fix Everything` only when the manager state looks broken or an operation was
					interrupted.
				</p></Card.Content
			></Card.Root
		>
		<Card.Root
			><Card.Content class="pt-6"
				><p class="text-sm font-medium">Crash troubleshooting</p>
				<p class="mt-2 text-sm text-muted-foreground">
					Use problem-mod isolation to narrow a crash down to a smaller test set before removing
					mods manually.
				</p></Card.Content
			></Card.Root
		>
	</div>
	<Card.Root
		><Card.Header
			><Card.Title class="flex items-center gap-2"
				><RefreshCw class="size-5" /> App updates</Card.Title
			><Card.Description
				>Check GitHub releases for a newer version of the app and install it automatically when
				available.</Card.Description
			></Card.Header
		><Card.Content class="space-y-4"
			><div class="flex flex-wrap gap-2">
				<Button
					variant="outline"
					disabled={manager.busy.updater}
					onclick={() => manager.checkForUpdates()}
					>{manager.busy.updater ? 'Checking...' : 'Check for updates'}</Button
				>{#if manager.updateInfo?.available}<Button
						disabled={manager.busy.updater}
						onclick={() => manager.installUpdate()}
						>Download and install {manager.updateInfo.version}</Button
					>{/if}
			</div>
			{#if manager.updateInfo}<div class="rounded-xl border bg-muted/20 p-4 text-sm">
					<p class="font-medium">Current version: {manager.updateInfo.currentVersion}</p>
					{#if manager.updateInfo.available}<p class="mt-2 text-muted-foreground">
							Update {manager.updateInfo.version} is available.
						</p>
						{#if manager.updateInfo.body}<p
								class="mt-2 text-sm whitespace-pre-wrap text-muted-foreground"
							>
								{manager.updateInfo.body}
							</p>{/if}{:else}<p class="mt-2 text-muted-foreground">
							No newer version is currently available.
						</p>{/if}
				</div>{/if}</Card.Content
		></Card.Root
	>
	{#if manager.recoveryPending}<Alert.Root variant="destructive"
			><AlertCircle class="size-4" /><Alert.Title>Recovery recommended</Alert.Title
			><Alert.Description
				>The last operation may have been interrupted{#if manager.pendingOperation}
					during `{manager.pendingOperation}`{/if}. Run `Fix Everything` to restore a clean state.</Alert.Description
			></Alert.Root
		>{/if}
	<Card.Root
		><Card.Header
			><Card.Title class="flex items-center gap-2"
				><Gamepad2 class="size-5" /> Game install</Card.Title
			><Card.Description
				>Saved target for apply, restore, extraction, and PATHC tools.</Card.Description
			></Card.Header
		><Card.Content class="space-y-4"
			><div class="space-y-2">
				<Label for="game-path">Game path</Label><Input
					id="game-path"
					bind:value={gamePathInput}
					placeholder="Crimson Desert.app or packages path"
				/>
			</div>
			<div class="flex flex-wrap gap-2">
				<Button variant="outline" onclick={chooseGamePath}>Browse</Button><Button
					variant="outline"
					disabled={manager.busy.detectingGame}
					onclick={async () => {
						const detected = await manager.detectInstall();
						gamePathInput = detected?.packagesPath ?? gamePathInput;
					}}>Detect</Button
				><Button
					disabled={manager.busy.settingGame}
					onclick={async () => {
						const saved = await manager.saveGamePath(gamePathInput);
						gamePathInput = saved?.packagesPath ?? gamePathInput;
					}}>Save Path</Button
				><Button
					variant="outline"
					disabled={!manager.install || manager.busy.launching}
					onclick={() => manager.runLaunch()}><Wrench class="size-4" /> Start game</Button
				>
			</div>
			<p class="text-sm break-all text-muted-foreground">
				{manager.install?.packagesPath ?? 'Not configured yet'}
			</p></Card.Content
		></Card.Root
	>
	<Card.Root
		><Card.Header
			><Card.Title>Recovery tools</Card.Title><Card.Description
				>Use restore/reset for normal cleanup, or `Fix Everything` for a full manager reset.</Card.Description
			></Card.Header
		><Card.Content class="space-y-4"
			><div class="flex flex-wrap gap-2">
				<Button
					variant="outline"
					disabled={!manager.install || manager.busy.restoring}
					onclick={() => manager.runRestore()}>Restore vanilla overlay</Button
				><Button
					variant="outline"
					disabled={manager.busy.resetting}
					onclick={() => manager.runReset()}>Reset active mods</Button
				><Button
					variant="destructive"
					disabled={manager.busy.fixing}
					onclick={() => manager.runFixEverything()}
					>{manager.busy.fixing ? 'Fixing...' : 'Fix Everything'}</Button
				>
			</div>
			<p class="text-sm text-muted-foreground">
				Fix Everything restores vanilla state, clears manager-owned groups, disables mods, clears
				patch toggles, and resets import cache.
			</p></Card.Content
		></Card.Root
	>
	<Card.Root
		><Card.Header
			><Card.Title class="flex items-center gap-2"
				><Sparkles class="size-5" /> Problem-mod isolation</Card.Title
			><Card.Description
				>Use a guided binary-search workflow to narrow down which currently enabled mod is causing a
				crash or bad behavior.</Card.Description
			></Card.Header
		><Card.Content class="space-y-4"
			>{#if !manager.isolationSession}<div class="flex flex-wrap gap-2">
					<Button onclick={() => manager.startProblemModIsolation()}>Start isolation</Button>
				</div>
				<p class="text-sm text-muted-foreground">
					1. Enable the mods you want to test. 2. Start isolation. 3. Launch the game and try to
					reproduce the problem. 4. Report whether the current test set crashed or stayed stable.
				</p>{:else}<div class="rounded-xl border bg-muted/20 p-4 text-sm">
					<p class="font-medium">Round {manager.isolationSession.rounds}</p>
					<p class="mt-2 text-muted-foreground">
						Testing {manager.isolationSession.currentTestSet.length} mod(s) out of {manager
							.isolationSession.suspects.length} suspect(s).
					</p>
					<div class="mt-3 flex flex-wrap gap-2">
						{#each manager.isolationCurrentNames as name (name)}<Badge variant="outline"
								>{name}</Badge
							>{/each}
					</div>
					{#if manager.isolationResolvedName}<p class="mt-3 text-destructive">
							Likely culprit: {manager.isolationResolvedName}
						</p>{:else}<p class="mt-3 text-sm text-muted-foreground">
							Remaining suspects: {manager.isolationSuspectNames.join(', ')}
						</p>{/if}
				</div>
				<div class="flex flex-wrap gap-2">
					<Button variant="destructive" onclick={() => manager.reportProblemModIsolation(true)}
						>This set crashed</Button
					><Button variant="outline" onclick={() => manager.reportProblemModIsolation(false)}
						>This set is stable</Button
					><Button variant="outline" onclick={() => manager.clearProblemModIsolation()}
						>Clear session</Button
					>
				</div>{/if}</Card.Content
		></Card.Root
	>
	<Card.Root
		><Card.Header
			><Card.Title>Verify game state</Card.Title><Card.Description
				>Quick health summary for the current install, overlay state, backup, and recovery markers.</Card.Description
			></Card.Header
		><Card.Content class="space-y-4"
			><div class="flex flex-wrap gap-2">
				<Button variant="outline" onclick={() => manager.verifyGameState()}
					>Refresh verification</Button
				>
			</div>
			{#if manager.gameStateReport}<div class="grid gap-3 sm:grid-cols-3">
					<div class="rounded-xl border bg-muted/20 p-4">
						<p class="text-xs tracking-[0.18em] text-muted-foreground uppercase">Metadata</p>
						<p class="mt-2 text-sm">
							{manager.gameStateReport.metaExists ? '0.papgt present' : 'Missing 0.papgt'}
						</p>
					</div>
					<div class="rounded-xl border bg-muted/20 p-4">
						<p class="text-xs tracking-[0.18em] text-muted-foreground uppercase">Base archive</p>
						<p class="mt-2 text-sm">
							{manager.gameStateReport.pamtExists ? '0008/0.pamt present' : 'Missing 0008/0.pamt'}
						</p>
					</div>
					<div class="rounded-xl border bg-muted/20 p-4">
						<p class="text-xs tracking-[0.18em] text-muted-foreground uppercase">Recovery</p>
						<p class="mt-2 text-sm">
							{manager.gameStateReport.recoveryPending
								? 'Pending recovery marker'
								: 'No pending recovery marker'}
						</p>
					</div>
				</div>
				<div class="flex flex-wrap gap-2">
					<Badge variant="outline">{manager.gameStateReport.enabledModCount} enabled</Badge><Badge
						variant="outline">{manager.gameStateReport.disabledModCount} disabled</Badge
					><Badge variant="outline"
						>{manager.gameStateReport.managedGroupCount} managed groups</Badge
					>{#if manager.gameStateReport.backupExists}<Badge variant="outline">Backup present</Badge
						>{/if}
				</div>{/if}</Card.Content
		></Card.Root
	>
	<Card.Root
		><Card.Header
			><Card.Title>Diagnostic report</Card.Title><Card.Description
				>Export a JSON report containing dashboard state, history, and any active isolation session.</Card.Description
			></Card.Header
		><Card.Content class="space-y-4"
			><div class="space-y-2">
				<Label for="report-output">Report output path</Label>
				<div class="flex flex-wrap gap-2">
					<Input
						id="report-output"
						bind:value={reportOutputPath}
						placeholder="Choose where to save the diagnostic report"
					/><Button variant="outline" onclick={chooseReportOutput}>Browse</Button><Button
						disabled={!reportOutputPath.trim()}
						onclick={() => manager.exportDiagnosticReport(reportOutputPath)}>Export report</Button
					>
				</div>
			</div>
			<p class="text-sm text-muted-foreground">
				The exported report includes dashboard state, recent history, and any active isolation
				session.
			</p></Card.Content
		></Card.Root
	>
</div>
