<script lang="ts">
	import { onMount } from 'svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import { Archive } from '@lucide/svelte';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import * as Empty from '$lib/components/ui/empty';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { manager } from '$lib/manager-state.svelte';
	import { modKindLabel } from '$lib/manager-helpers';

	let binaryTargetPath = $state('');
	let binaryOutputPath = $state('');
	let scriptWorkingDir = $state('');

	onMount(async () => {
		await manager.ensureLoaded();
		await manager.refreshBnkFiles();
	});

	async function chooseBinaryTarget() {
		const selected = await open({
			multiple: false,
			directory: false,
			defaultPath: binaryTargetPath || undefined,
			title: 'Choose the source file to patch'
		});
		if (typeof selected === 'string') binaryTargetPath = selected;
	}

	async function chooseBinaryOutput() {
		const selected = await open({
			multiple: false,
			directory: false,
			defaultPath: binaryOutputPath || undefined,
			title: 'Choose the output file for the patched result'
		});
		if (typeof selected === 'string') binaryOutputPath = selected;
	}

	async function chooseScriptWorkingDir() {
		const selected = await open({
			multiple: false,
			directory: true,
			defaultPath: scriptWorkingDir || undefined,
			title: 'Choose a working directory for the script installer'
		});
		if (typeof selected === 'string') scriptWorkingDir = selected;
	}
</script>

<svelte:head>
	<title>External Mods • Crimson Desert Mod Workbench</title>
</svelte:head>

<div
	class="mx-auto flex min-h-full w-full max-w-5xl flex-col gap-6 px-4 py-6 sm:px-6 lg:px-8 lg:py-8"
>
	<div class="space-y-2">
		<p class="text-xs font-medium tracking-[0.24em] text-muted-foreground uppercase">
			External Mods
		</p>
		<h1 class="text-3xl font-semibold tracking-tight">Non-overlay mod formats</h1>
		<p class="max-w-3xl text-sm leading-7 text-muted-foreground">
			These formats are managed outside the normal JSON/precompiled overlay pipeline. They include
			BNK soundbanks, script-installer mods, and binary patch packages.
		</p>
	</div>
	<div class="grid gap-4 md:grid-cols-3">
		<Card.Root
			><Card.Content class="pt-6"
				><p class="text-sm font-medium">BNK</p>
				<p class="mt-2 text-sm text-muted-foreground">
					Copies imported soundbank files into the managed soundbank folder for this install.
				</p></Card.Content
			></Card.Root
		>
		<Card.Root
			><Card.Content class="pt-6"
				><p class="text-sm font-medium">Binary patches</p>
				<p class="mt-2 text-sm text-muted-foreground">
					Apply `.bsdiff` or `.xdelta` patches to a source file and write the result to a chosen
					output file.
				</p></Card.Content
			></Card.Root
		>
		<Card.Root
			><Card.Content class="pt-6"
				><p class="text-sm font-medium">Script installers</p>
				<p class="mt-2 text-sm text-muted-foreground">
					Stage the script files, then run a macOS-supported installer from a chosen working
					directory.
				</p></Card.Content
			></Card.Root
		>
	</div>

	<Card.Root>
		<Card.Header>
			<Card.Title class="flex items-center gap-2"
				><Archive class="size-5" /> Imported external mods</Card.Title
			>
			<Card.Description>Install or remove imported external formats from here.</Card.Description>
		</Card.Header>
		<Card.Content>
			{#if manager.bnkMods.length === 0 && manager.scriptMods.length === 0 && manager.binaryPatchMods.length === 0}
				<Empty.Root class="min-h-40 border-dashed bg-muted/20 p-8">
					<Empty.Header>
						<Empty.Title>No external mods imported</Empty.Title>
						<Empty.Description
							>Import BNK, script-installer, or binary patch mod sources from the Data Mods page.</Empty.Description
						>
					</Empty.Header>
				</Empty.Root>
			{:else}
				<div class="space-y-3">
					{#each [...manager.bnkMods, ...manager.scriptMods, ...manager.binaryPatchMods] as mod (mod.id)}
						<div class="rounded-xl border bg-muted/20 px-4 py-4">
							<div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
								<div>
									<p class="font-medium">{mod.name}</p>
									<p class="text-sm text-muted-foreground">{mod.fileName}</p>
								</div>
								<div class="flex flex-wrap gap-2">
									<Badge variant="outline">{modKindLabel(mod.modKind)}</Badge>
									{#if mod.modKind === 'bnk'}
										<Button size="sm" onclick={() => manager.installBnkMod(mod)}>Install</Button>
									{:else if mod.modKind === 'script_installer'}
										<Button size="sm" onclick={() => manager.installScriptMod(mod)}
											>Stage files</Button
										>
										<Button
											variant="outline"
											size="sm"
											disabled={!scriptWorkingDir.trim()}
											onclick={() => manager.runScriptInstaller(mod, scriptWorkingDir)}
											>Run script</Button
										>
									{:else if mod.modKind === 'binary_patch'}
										<Button
											size="sm"
											disabled={!binaryTargetPath.trim() || !binaryOutputPath.trim()}
											onclick={() =>
												manager.applyBinaryPatch(mod, binaryTargetPath, binaryOutputPath)}
											>Apply patch</Button
										>
									{/if}
									<Button variant="destructive" size="sm" onclick={() => manager.removeMod(mod)}
										>Remove import</Button
									>
								</div>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Installed BNK files</Card.Title>
			<Card.Description
				>BNK files copied into the managed soundbank target directory for this install.</Card.Description
			>
		</Card.Header>
		<Card.Content>
			{#if manager.bnkFiles.length === 0}
				<div class="text-sm text-muted-foreground">No installed BNK files detected yet.</div>
			{:else}
				<div class="space-y-2">
					{#each manager.bnkFiles as file (file.path)}
						<div
							class="flex flex-col gap-2 rounded-lg border bg-background/60 px-4 py-3 sm:flex-row sm:items-center sm:justify-between"
						>
							<div>
								<p class="font-medium">{file.name}</p>
								<p class="text-xs break-all text-muted-foreground">{file.path}</p>
							</div>
							<Button
								variant="destructive"
								size="sm"
								onclick={() => manager.removeBnkFile(file.name)}>Remove</Button
							>
						</div>
					{/each}
				</div>
			{/if}
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Binary patch workflow</Card.Title>
			<Card.Description
				>Pick a source file and an output file, then apply an imported `.bsdiff` or `.xdelta` patch
				mod.</Card.Description
			>
		</Card.Header>
		<Card.Content class="space-y-4">
			<div class="space-y-2">
				<Label for="binary-target">Source file</Label>
				<div class="flex flex-wrap gap-2">
					<Input
						id="binary-target"
						bind:value={binaryTargetPath}
						placeholder="Choose the file to patch"
					/>
					<Button variant="outline" onclick={chooseBinaryTarget}>Browse</Button>
				</div>
			</div>
			<div class="space-y-2">
				<Label for="binary-output">Patched output file</Label>
				<div class="flex flex-wrap gap-2">
					<Input
						id="binary-output"
						bind:value={binaryOutputPath}
						placeholder="Choose where to write the patched result"
					/>
					<Button variant="outline" onclick={chooseBinaryOutput}>Browse</Button>
				</div>
			</div>
			<p class="text-sm text-muted-foreground">
				Binary patch mods require you to choose the original source file and where the patched
				output should be written.
			</p>
			<p class="text-xs text-muted-foreground">
				Tip: write to a separate output file first so you can inspect the patched result before
				replacing anything manually.
			</p>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Script installer workflow</Card.Title>
			<Card.Description
				>Stage the script files, then run the imported installer with a chosen working directory.</Card.Description
			>
		</Card.Header>
		<Card.Content class="space-y-4">
			<div class="space-y-2">
				<Label for="script-working-dir">Working directory</Label>
				<div class="flex flex-wrap gap-2">
					<Input
						id="script-working-dir"
						bind:value={scriptWorkingDir}
						placeholder="Choose a working directory"
					/>
					<Button variant="outline" onclick={chooseScriptWorkingDir}>Browse</Button>
				</div>
			</div>
			<p class="text-sm text-muted-foreground">
				`.sh`, `.command`, and `.py` installers can be executed here. Windows `.bat` installers will
				be detected but remain unsupported on native macOS.
			</p>
			{#if !scriptWorkingDir.trim()}<p class="text-xs text-muted-foreground">
					Choose a working directory before running a script installer.
				</p>{/if}
		</Card.Content>
	</Card.Root>
</div>
