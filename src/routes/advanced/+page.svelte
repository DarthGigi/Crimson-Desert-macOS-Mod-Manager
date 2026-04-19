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
	let virtualSearchInput = $state('playeractiongraph');
	let extractVirtualPathInput = $state('character/player/playeractiongraph_main.xml');
	let extractSourceGroupInput = $state('0008');
	let extractOutputDirInput = $state('');
	let xmlVirtualPathInput = $state('technique/lightpreset.xml');
	let xmlSourceGroupInput = $state('0008');
	let xmlOutputDirInput = $state('');
	let xmlModifiedPathInput = $state('');
	let xmlPayloadOutputPath = $state('');

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

		if (typeof selected === 'string') {
			pathcFolderInput = selected;
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

	async function chooseXmlOutputDir() {
		const selected = await open({
			multiple: false,
			directory: true,
			defaultPath: xmlOutputDirInput || undefined,
			title: 'Choose an output folder for extracted XML'
		});
		if (typeof selected === 'string') {
			xmlOutputDirInput = selected;
		}
	}

	async function chooseXmlModifiedFile() {
		const selected = await open({
			multiple: false,
			directory: false,
			filters: [{ name: 'XML files', extensions: ['xml'] }],
			defaultPath: xmlModifiedPathInput || undefined,
			title: 'Choose a modified XML file'
		});
		if (typeof selected === 'string') {
			xmlModifiedPathInput = selected;
		}
	}

	async function chooseXmlPayloadOutput() {
		const selected = await open({
			multiple: false,
			directory: false,
			defaultPath: xmlPayloadOutputPath || undefined,
			title: 'Choose an output file for the encrypted/compressed payload'
		});
		if (typeof selected === 'string') {
			xmlPayloadOutputPath = selected;
		}
	}
</script>

<svelte:head>
	<title>Advanced • Crimson Desert Mod Workbench</title>
</svelte:head>

<div
	class="mx-auto flex min-h-full w-full max-w-5xl flex-col gap-6 px-4 py-6 sm:px-6 lg:px-8 lg:py-8"
>
	<div class="space-y-2">
		<p class="text-xs font-medium tracking-[0.24em] text-muted-foreground uppercase">Advanced</p>
		<h1 class="text-3xl font-semibold tracking-tight">PATHC and extraction workflows</h1>
		<p class="max-w-3xl text-sm leading-7 text-muted-foreground">
			Inspect `0.pathc`, repack DDS folders with backup protection, search virtual archive paths,
			and preview or extract decompressed files from the game archives.
		</p>
	</div>

	<Card.Root>
		<Card.Header>
			<Card.Title class="flex items-center gap-2"
				><Image class="size-5" /> PATHC texture index</Card.Title
			>
			<Card.Description>
				Inspect `0.pathc`, verify virtual-path lookups, and repack a DDS folder into the PATHC index
				with a `.bak` safety backup.
			</Card.Description>
		</Card.Header>
		<Card.Content class="space-y-4">
			<div class="space-y-2">
				<Label for="pathc-path">PATHC file</Label>
				<div class="flex flex-wrap gap-2">
					<Input id="pathc-path" bind:value={pathcPathInput} placeholder=".../meta/0.pathc" />
					<Button variant="outline" onclick={choosePathcFile}>Browse</Button>
					<Button
						variant="outline"
						disabled={manager.busy.pathc}
						onclick={() => manager.refreshPathcSummary(pathcPathInput, pathcLookupInput)}
					>
						{manager.busy.pathc ? 'Refreshing...' : 'Refresh'}
					</Button>
				</div>
			</div>

			<div class="space-y-2">
				<Label for="pathc-lookup">Lookup virtual path</Label>
				<div class="flex flex-wrap gap-2">
					<Input
						id="pathc-lookup"
						bind:value={pathcLookupInput}
						placeholder="/ui/texture/example.dds"
					/>
					<Button
						variant="outline"
						disabled={manager.busy.pathc}
						onclick={() => manager.refreshPathcSummary(pathcPathInput, pathcLookupInput)}
					>
						Lookup
					</Button>
				</div>
			</div>

			{#if manager.pathcSummary}
				<div class="grid gap-3 sm:grid-cols-3">
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

				{#if manager.pathcSummary.lookups.length > 0}
					<ScrollArea.Root class="h-44 rounded-xl border">
						<div class="divide-y">
							{#each manager.pathcSummary.lookups as lookup (lookup.virtualPath)}
								<div class="space-y-2 px-4 py-3">
									<div class="flex flex-wrap items-center gap-2">
										<p class="text-sm font-medium break-all">{lookup.virtualPath}</p>
										<Badge variant={lookup.found ? 'outline' : 'secondary'}
											>{lookup.found ? 'Found' : 'Missing'}</Badge
										>
									</div>
									<p class="text-xs text-muted-foreground">
										Hash: 0x{lookup.keyHash.toString(16).toUpperCase()}
									</p>
									{#if lookup.found}
										<p class="text-xs text-muted-foreground">
											DDS {lookup.directDdsIndex} / {lookup.width}x{lookup.height} / mip {lookup.mipCount}
											/ {lookup.formatLabel}
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
					<Input
						id="pathc-folder"
						bind:value={pathcFolderInput}
						placeholder="Folder containing DDS files"
					/>
					<Button variant="outline" onclick={choosePathcFolder}>Choose folder</Button>
					<Button
						disabled={manager.busy.repackingPathc}
						onclick={() => manager.runPathcRepack(pathcPathInput, pathcFolderInput)}
					>
						{manager.busy.repackingPathc ? 'Repacking...' : 'Repack PATHC'}
					</Button>
				</div>
			</div>

			{#if manager.pathcResult}
				<Alert.Root>
					<Info class="size-4" />
					<Alert.Title>Last PATHC repack</Alert.Title>
					<Alert.Description>
						Processed {manager.pathcResult.processedCount} DDS file(s), updated {manager.pathcResult
							.updatedCount} hash entries, added {manager.pathcResult.addedTemplateCount} new DDS templates.
						Backup: {manager.pathcResult.backupPath}
					</Alert.Description>
				</Alert.Root>
			{/if}
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title class="flex items-center gap-2"
				><Archive class="size-5" /> Virtual file extraction</Card.Title
			>
			<Card.Description>
				Preview a virtual file in the game archives and extract its decompressed bytes into a chosen
				output folder.
			</Card.Description>
		</Card.Header>
		<Card.Content class="space-y-4">
			<div class="grid gap-4 sm:grid-cols-2">
				<div class="space-y-2 sm:col-span-2">
					<Label for="extract-virtual-path">Virtual path</Label>
					<Input
						id="extract-virtual-path"
						bind:value={extractVirtualPathInput}
						placeholder="character/player/playeractiongraph_main.xml"
					/>
				</div>
				<div class="space-y-2">
					<Label for="extract-source-group">Preferred source group</Label>
					<Input
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
						/>
						<Button variant="outline" onclick={chooseExtractOutputDir}>Browse</Button>
					</div>
				</div>
			</div>

			<div class="flex flex-wrap gap-2">
				<Button
					variant="outline"
					disabled={manager.busy.extracting}
					onclick={() =>
						manager.refreshExtractPreview(extractVirtualPathInput, extractSourceGroupInput || null)}
				>
					{manager.busy.extracting ? 'Inspecting...' : 'Inspect file'}
				</Button>
				<Button
					disabled={manager.busy.extracting}
					onclick={() =>
						manager.runExtractVirtualFile(
							extractVirtualPathInput,
							extractSourceGroupInput || null,
							extractOutputDirInput
						)}
				>
					{manager.busy.extracting ? 'Extracting...' : 'Extract file'}
				</Button>
			</div>

			{#if manager.extractPreview}
				<div class="rounded-xl border bg-muted/20 p-4 text-sm">
					<div class="flex flex-wrap items-center gap-2">
						<p class="font-medium break-all">{manager.extractPreview.virtualPath}</p>
						<Badge variant={manager.extractPreview.resolved ? 'outline' : 'secondary'}
							>{manager.extractPreview.resolved ? 'Resolved' : 'Missing'}</Badge
						>
					</div>
					<p class="mt-2 text-xs text-muted-foreground">
						Source group: {manager.extractPreview.sourceGroup}
					</p>
					{#if manager.extractPreview.resolved}
						<p class="mt-1 text-xs text-muted-foreground">
							{manager.extractPreview.resolvedGameFile} / PAZ {manager.extractPreview
								.sourcePazIndex} / {manager.extractPreview.compressedSize} compressed / {manager
								.extractPreview.decompressedSize} decompressed / flags {manager.extractPreview
								.flags}
						</p>
					{:else if manager.extractPreview.reason}
						<p class="mt-1 text-xs text-destructive">{manager.extractPreview.reason}</p>
					{/if}
				</div>
			{/if}

			{#if manager.extractResult}
				<Alert.Root>
					<Info class="size-4" />
					<Alert.Title>Last extraction</Alert.Title>
					<Alert.Description>
						Extracted {manager.extractResult.virtualPath} from {manager.extractResult.sourceGroup} to
						{manager.extractResult.outputPath} ({manager.extractResult.decompressedSize} bytes).
					</Alert.Description>
				</Alert.Root>
			{/if}
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title class="flex items-center gap-2"
				><Info class="size-5" /> XML decrypt and repack</Card.Title
			>
			<Card.Description>
				Extract a real XML entry with filename-derived ChaCha20 decryption and prepare an
				encrypted/compressed replacement payload. In-place patching only happens when the resulting
				payload exactly matches the original compressed size.
			</Card.Description>
		</Card.Header>
		<Card.Content class="space-y-4">
			<div class="grid gap-4 sm:grid-cols-2">
				<div class="space-y-2 sm:col-span-2">
					<Label for="xml-virtual-path">XML virtual path</Label>
					<Input
						id="xml-virtual-path"
						bind:value={xmlVirtualPathInput}
						placeholder="technique/lightpreset.xml"
					/>
				</div>
				<div class="space-y-2">
					<Label for="xml-source-group">Source group</Label>
					<Input id="xml-source-group" bind:value={xmlSourceGroupInput} placeholder="0008" />
				</div>
				<div class="space-y-2">
					<Label for="xml-output-dir">XML output folder</Label>
					<div class="flex flex-wrap gap-2">
						<Input
							id="xml-output-dir"
							bind:value={xmlOutputDirInput}
							placeholder="Choose an output folder"
						/>
						<Button variant="outline" onclick={chooseXmlOutputDir}>Browse</Button>
					</div>
				</div>
			</div>

			<div class="flex flex-wrap gap-2">
				<Button
					variant="outline"
					disabled={manager.busy.xml}
					onclick={() =>
						manager.extractXmlEntry(
							xmlVirtualPathInput,
							xmlSourceGroupInput || null,
							xmlOutputDirInput
						)}
				>
					{manager.busy.xml ? 'Processing...' : 'Extract XML'}
				</Button>
			</div>

			{#if manager.xmlPreview}
				<div class="rounded-xl border bg-muted/20 p-4 text-sm">
					<p class="font-medium break-all">{manager.xmlPreview.virtualPath}</p>
					<p class="mt-2 text-xs text-muted-foreground">
						{manager.xmlPreview.sourceGroup} / PAZ {manager.xmlPreview.sourcePazIndex} / {manager
							.xmlPreview.compressed
							? 'Compressed'
							: 'Stored'} / {manager.xmlPreview.encrypted ? 'Encrypted' : 'Plain'}
					</p>
					<p class="mt-1 text-xs text-muted-foreground">
						{manager.xmlPreview.compressedSize} compressed / {manager.xmlPreview.decompressedSize} decompressed
					</p>
					<p class="mt-1 text-xs break-all">Extracted to {manager.xmlPreview.extractedPath}</p>
				</div>
			{/if}

			<div class="grid gap-4 sm:grid-cols-2">
				<div class="space-y-2 sm:col-span-2">
					<Label for="xml-modified">Modified XML file</Label>
					<div class="flex flex-wrap gap-2">
						<Input
							id="xml-modified"
							bind:value={xmlModifiedPathInput}
							placeholder="Choose a modified XML file"
						/>
						<Button variant="outline" onclick={chooseXmlModifiedFile}>Browse</Button>
					</div>
				</div>
				<div class="space-y-2 sm:col-span-2">
					<Label for="xml-payload-output">Optional payload output file</Label>
					<div class="flex flex-wrap gap-2">
						<Input
							id="xml-payload-output"
							bind:value={xmlPayloadOutputPath}
							placeholder="Write encrypted/compressed payload to file instead of patching in place"
						/>
						<Button variant="outline" onclick={chooseXmlPayloadOutput}>Browse</Button>
					</div>
				</div>
			</div>

			<div class="flex flex-wrap gap-2">
				<Button
					disabled={manager.busy.xml}
					onclick={() =>
						manager.repackXmlEntry(
							xmlVirtualPathInput,
							xmlSourceGroupInput || null,
							xmlModifiedPathInput,
							xmlPayloadOutputPath || null
						)}
				>
					{manager.busy.xml ? 'Processing...' : 'Repack XML'}
				</Button>
			</div>

			{#if manager.xmlRepackResult}
				<Alert.Root>
					<Info class="size-4" />
					<Alert.Title>Last XML repack</Alert.Title>
					<Alert.Description>
						Target size {manager.xmlRepackResult.targetCompSize} bytes, new payload {manager
							.xmlRepackResult.newCompSize} bytes. {#if manager.xmlRepackResult.patchedInPlace}Patched
							in place.{:else if manager.xmlRepackResult.outputPath}Wrote payload to {manager
								.xmlRepackResult.outputPath}.{:else if !manager.xmlRepackResult.exactFit}Payload
							size does not match the original entry, so no in-place patch was made.{/if}
					</Alert.Description>
				</Alert.Root>
			{/if}
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title class="flex items-center gap-2"><Info class="size-5" /> Archive search</Card.Title
			>
			<Card.Description>
				Search virtual files by partial path and push a match directly into the extraction workflow.
			</Card.Description>
		</Card.Header>
		<Card.Content class="space-y-4">
			<div class="grid gap-4 sm:grid-cols-2">
				<div class="space-y-2 sm:col-span-2">
					<Label for="virtual-search">Search virtual paths</Label>
					<Input
						id="virtual-search"
						bind:value={virtualSearchInput}
						placeholder="ui/texture or playeractiongraph"
					/>
				</div>
				<div class="space-y-2">
					<Label for="search-source-group">Source group filter</Label>
					<Input
						id="search-source-group"
						bind:value={extractSourceGroupInput}
						placeholder="0008 or leave blank"
					/>
				</div>
				<div class="flex items-end">
					<Button
						variant="outline"
						disabled={manager.busy.searchingFiles}
						onclick={() =>
							manager.searchVirtualFiles(virtualSearchInput, extractSourceGroupInput || null, 100)}
					>
						{manager.busy.searchingFiles ? 'Searching...' : 'Search archives'}
					</Button>
				</div>
			</div>

			<ScrollArea.Root class="h-64 rounded-xl border">
				<div class="divide-y">
					{#each manager.virtualFileMatches as match (match.sourceGroup + match.virtualPath)}
						<button
							class="w-full space-y-1 px-4 py-3 text-left transition hover:bg-muted/40"
							onclick={async () => {
								extractVirtualPathInput = match.virtualPath;
								extractSourceGroupInput = match.sourceGroup;
								await manager.refreshExtractPreview(
									extractVirtualPathInput,
									extractSourceGroupInput
								);
							}}
						>
							<div class="flex flex-wrap items-center justify-between gap-3">
								<p class="text-sm font-medium break-all">{match.virtualPath}</p>
								<Badge variant="outline">{match.sourceGroup}</Badge>
							</div>
							<p class="text-xs text-muted-foreground">
								PAZ {match.sourcePazIndex} / {match.compressedSize} compressed / {match.decompressedSize}
								decompressed / flags {match.flags}
							</p>
						</button>
					{:else}
						<div class="px-4 py-8 text-sm text-muted-foreground">
							No virtual file matches loaded yet.
						</div>
					{/each}
				</div>
			</ScrollArea.Root>
		</Card.Content>
	</Card.Root>
</div>
