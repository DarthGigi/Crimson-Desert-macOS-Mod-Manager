<script lang="ts">
	import { onMount } from 'svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import { AlertCircle, Gamepad2, Wrench } from '@lucide/svelte';
	import * as Alert from '$lib/components/ui/alert';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { manager } from '$lib/manager-state.svelte';

	let gamePathInput = $state('');

	onMount(async () => {
		await manager.ensureLoaded();
		gamePathInput = manager.install?.packagesPath ?? '';
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
</div>
