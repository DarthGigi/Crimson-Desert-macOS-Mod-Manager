<script lang="ts">
	import { onMount } from 'svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import { Archive, Image, Info } from '@lucide/svelte';
	import * as Alert from '$lib/components/ui/alert';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as ScrollArea from '$lib/components/ui/scroll-area';
	import { manager } from '$lib/manager-state.svelte';

	let pathcPathInput = $state('');
	let pathcLookupInput = $state('/ui/texture/cd_itemslot_00.dds');
	let pathcFolderInput = $state('');
	let extractVirtualPathInput = $state('character/player/playeractiongraph_main.xml');
	let extractSourceGroupInput = $state('0008');
	let extractOutputDirInput = $state('');

	onMount(async () => {
		await manager.ensureLoaded();
		pathcPathInput = manager.install?.packagesPath
			? `${manager.install.packagesPath}/meta/0.pathc`
			: '';
		await manager.refreshPathcSummary(pathcPathInput, pathcLookupInput);
	});

	async function choosePathcFile() {
		const selected = await open({
			multiple: false,
			directory: false,
			filters: [{ name: 'PATHC files', extensions: ['pathc'] }],
			defaultPath: pathcPathInput || undefined,
			title: 'Choose a .pathc file'
		});
		if (typeof selected === 'string') {
			pathcPathInput = selected;
			await manager.refreshPathcSummary(pathcPathInput, pathcLookupInput);
		}
	}

	async function choosePathcFolder() {
		const selected = await open({
			multiple: false,
			directory: true,
			defaultPath: pathcFolderInput || undefined,
			title: 'Choose a folder containing DDS files'
		});
		if (typeof selected === 'string') pathcFolderInput = selected;
	}

	async function chooseExtractOutputDir() {
		const selected = await open({
			multiple: false,
			directory: true,
			defaultPath: extractOutputDirInput || undefined,
			title: 'Choose an output folder for extracted files'
		});
		if (typeof selected === 'string') extractOutputDirInput = selected;
	}
</script>

<svelte:head><title>Advanced • Crimson Desert Mod Workbench</title></svelte:head>

<div
	class="mx-auto flex min-h-full w-full max-w-5xl flex-col gap-6 px-4 py-6 sm:px-6 lg:px-8 lg:py-8"
>
	<div class="space-y-2">
		<p class="text-xs font-medium tracking-[0.24em] text-muted-foreground uppercase">Advanced</p>
		<h1 class="text-3xl font-semibold tracking-tight">PATHC and extraction workflows</h1>
		<p class="max-w-3xl text-sm leading-7 text-muted-foreground">
			Inspect `0.pathc`, repack DDS folders with backup protection, and preview or extract
			decompressed virtual files from the game archives.
		</p>
	</div>
	<Card.Root
		><Card.Header
			><Card.Title class="flex items-center gap-2"
				><Image class="size-5" /> PATHC texture index</Card.Title
			><Card.Description
				>Inspect `0.pathc`, verify virtual-path lookups, and repack a DDS folder into the PATHC
				index with a `.bak` safety backup.</Card.Description
			></Card.Header
		><Card.Content class="space-y-4"
			><div class="space-y-2">
				<Label for="pathc-path">PATHC file</Label>
				<div class="flex flex-wrap gap-2">
					<Input
						id="pathc-path"
						bind:value={pathcPathInput}
						placeholder=".../meta/0.pathc"
					/><Button variant="outline" onclick={choosePathcFile}>Browse</Button><Button
						variant="outline"
						disabled={manager.busy.pathc}
						onclick={() => manager.refreshPathcSummary(pathcPathInput, pathcLookupInput)}
						>{manager.busy.pathc ? 'Refreshing...' : 'Refresh'}</Button
					>
				</div>
			</div>
			<div class="space-y-2">
				<Label for="pathc-lookup">Lookup virtual path</Label>
				<div class="flex flex-wrap gap-2">
					<Input
						id="pathc-lookup"
						bind:value={pathcLookupInput}
						placeholder="/ui/texture/example.dds"
					/><Button
						variant="outline"
						disabled={manager.busy.pathc}
						onclick={() => manager.refreshPathcSummary(pathcPathInput, pathcLookupInput)}
						>Lookup</Button
					>
				</div>
			</div>
			{#if manager.pathcSummary}<div class="grid gap-3 sm:grid-cols-3">
					<div class="rounded-xl border bg-muted/20 p-4">
						<p class="text-xs tracking-[0.18em] text-muted-foreground uppercase">DDS templates</p>
						<p class="mt-2 text-2xl font-semibold">{manager.pathcSummary.ddsTemplateCount}</p>
					</div>
					<div class="rounded-xl border bg-muted/20 p-4">
						<p class="text-xs tracking-[0.18em] text-muted-foreground uppercase">Hashes</p>
						<p class="mt-2 text-2xl font-semibold">{manager.pathcSummary.hashCount}</p>
					</div>
					<div class="rounded-xl border bg-muted/20 p-4">
						<p class="text-xs tracking-[0.18em] text-muted-foreground uppercase">Collisions</p>
						<p class="mt-2 text-2xl font-semibold">{manager.pathcSummary.collisionPathCount}</p>
					</div>
				</div>
				{#if manager.pathcSummary.lookups.length > 0}<ScrollArea.Root class="h-44 rounded-xl border"
						><div class="divide-y">
							{#each manager.pathcSummary.lookups as lookup (lookup.virtualPath)}<div
									class="space-y-2 px-4 py-3"
								>
									<div class="flex flex-wrap items-center gap-2">
										<p class="text-sm font-medium break-all">{lookup.virtualPath}</p>
										<Badge variant={lookup.found ? 'outline' : 'secondary'}
											>{lookup.found ? 'Found' : 'Missing'}</Badge
										>
									</div>
									<p class="text-xs text-muted-foreground">
										Hash: 0x{lookup.keyHash.toString(16).toUpperCase()}
									</p>
									{#if lookup.found}<p class="text-xs text-muted-foreground">
											DDS {lookup.directDdsIndex} / {lookup.width}x{lookup.height} / mip {lookup.mipCount}
											/ {lookup.formatLabel}
										</p>{/if}
								</div>{/each}
						</div></ScrollArea.Root
					>{/if}{/if}
			<div class="space-y-2">
				<Label for="pathc-folder">DDS source folder</Label>
				<div class="flex flex-wrap gap-2">
					<Input
						id="pathc-folder"
						bind:value={pathcFolderInput}
						placeholder="Folder containing DDS files"
					/><Button variant="outline" onclick={choosePathcFolder}>Choose folder</Button><Button
						disabled={manager.busy.repackingPathc}
						onclick={() => manager.runPathcRepack(pathcPathInput, pathcFolderInput)}
						>{manager.busy.repackingPathc ? 'Repacking...' : 'Repack PATHC'}</Button
					>
				</div>
			</div>
			{#if manager.pathcResult}<Alert.Root
					><Info class="size-4" /><Alert.Title>Last PATHC repack</Alert.Title><Alert.Description
						>Processed {manager.pathcResult.processedCount} DDS file(s), updated {manager
							.pathcResult.updatedCount} hash entries, added {manager.pathcResult
							.addedTemplateCount} new DDS templates. Backup: {manager.pathcResult
							.backupPath}</Alert.Description
					></Alert.Root
				>{/if}</Card.Content
		></Card.Root
	>
	<Card.Root
		><Card.Header
			><Card.Title class="flex items-center gap-2"
				><Archive class="size-5" /> Virtual file extraction</Card.Title
			><Card.Description
				>Preview a virtual file in the game archives and extract its decompressed bytes into a
				chosen output folder.</Card.Description
			></Card.Header
		><Card.Content class="space-y-4"
			><div class="grid gap-4 sm:grid-cols-2">
				<div class="space-y-2 sm:col-span-2">
					<Label for="extract-virtual-path">Virtual path</Label><Input
						id="extract-virtual-path"
						bind:value={extractVirtualPathInput}
						placeholder="character/player/playeractiongraph_main.xml"
					/>
				</div>
				<div class="space-y-2">
					<Label for="extract-source-group">Preferred source group</Label><Input
						id="extract-source-group"
						bind:value={extractSourceGroupInput}
						placeholder="0008"
					/>
				</div>
				<div class="space-y-2">
					<Label for="extract-output">Output folder</Label>
					<div class="flex flex-wrap gap-2">
						<Input
							id="extract-output"
							bind:value={extractOutputDirInput}
							placeholder="Choose an output folder"
						/><Button variant="outline" onclick={chooseExtractOutputDir}>Browse</Button>
					</div>
				</div>
			</div>
			<div class="flex flex-wrap gap-2">
				<Button
					variant="outline"
					disabled={manager.busy.extracting}
					onclick={() =>
						manager.refreshExtractPreview(extractVirtualPathInput, extractSourceGroupInput || null)}
					>{manager.busy.extracting ? 'Inspecting...' : 'Inspect file'}</Button
				><Button
					disabled={manager.busy.extracting}
					onclick={() =>
						manager.runExtractVirtualFile(
							extractVirtualPathInput,
							extractSourceGroupInput || null,
							extractOutputDirInput
						)}>{manager.busy.extracting ? 'Extracting...' : 'Extract file'}</Button
				>
			</div>
			{#if manager.extractPreview}<div class="rounded-xl border bg-muted/20 p-4 text-sm">
					<div class="flex flex-wrap items-center gap-2">
						<p class="font-medium break-all">{manager.extractPreview.virtualPath}</p>
						<Badge variant={manager.extractPreview.resolved ? 'outline' : 'secondary'}
							>{manager.extractPreview.resolved ? 'Resolved' : 'Missing'}</Badge
						>
					</div>
					<p class="mt-2 text-xs text-muted-foreground">
						Source group: {manager.extractPreview.sourceGroup}
					</p>
					{#if manager.extractPreview.resolved}<p class="mt-1 text-xs text-muted-foreground">
							{manager.extractPreview.resolvedGameFile} / PAZ {manager.extractPreview
								.sourcePazIndex} / {manager.extractPreview.compressedSize} compressed / {manager
								.extractPreview.decompressedSize} decompressed / flags {manager.extractPreview
								.flags}
						</p>{:else if manager.extractPreview.reason}<p class="mt-1 text-xs text-destructive">
							{manager.extractPreview.reason}
						</p>{/if}
				</div>{/if}{#if manager.extractResult}<Alert.Root
					><Info class="size-4" /><Alert.Title>Last extraction</Alert.Title><Alert.Description
						>Extracted {manager.extractResult.virtualPath} from {manager.extractResult.sourceGroup} to
						{manager.extractResult.outputPath} ({manager.extractResult.decompressedSize} bytes).</Alert.Description
					></Alert.Root
				>{/if}</Card.Content
		></Card.Root
	>
</div>
