<script lang="ts">
	import { onMount } from 'svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import { ArrowDownUp, FolderSearch, HardDriveDownload, Info } from '@lucide/svelte';
	import * as Alert from '$lib/components/ui/alert';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import * as Collapsible from '$lib/components/ui/collapsible';
	import * as Empty from '$lib/components/ui/empty';
	import { Separator } from '$lib/components/ui/separator';
	import * as ScrollArea from '$lib/components/ui/scroll-area';
	import { manager } from '$lib/manager-state.svelte';
	import type { ScanResult } from '$lib/desktop-api';

	let importSourcePath = $state('');
	let scanDetailsOpen = $state<Record<string, boolean>>({});
	let variantGroupsOpen = $state<Record<string, boolean>>({});

	type ScanVariantGroup = {
		key: string;
		label: string;
		description: string;
		results: ScanResult[];
		isGrouped: boolean;
	};

	const groupedScanResults = $derived.by(() => buildVariantGroups(manager.scanResults));

	onMount(() => {
		void manager.ensureLoaded();
	});

	async function chooseFolder() {
		const selected = await open({
			multiple: false,
			directory: true,
			defaultPath: importSourcePath || undefined,
			title: 'Choose a mod folder'
		});
		if (typeof selected === 'string') {
			importSourcePath = selected;
			await manager.scanImportSource(selected);
		}
	}

	async function chooseZip() {
		const selected = await open({
			multiple: false,
			directory: false,
			filters: [{ name: 'Mod archives', extensions: ['zip', '7z', 'rar'] }],
			defaultPath: importSourcePath || undefined,
			title: 'Choose a ZIP, 7Z, or RAR archive'
		});
		if (typeof selected === 'string') {
			importSourcePath = selected;
			await manager.scanImportSource(selected);
		}
	}

	function buildVariantGroups(results: ScanResult[]): ScanVariantGroup[] {
		const buckets = new Map<string, ScanResult[]>();
		for (const result of results) {
			const key = `${parentDirectory(result.path)}::${result.modKind}`;
			const bucket = buckets.get(key) ?? [];
			bucket.push(result);
			buckets.set(key, bucket);
		}

		const groups: ScanVariantGroup[] = [];
		for (const [key, bucket] of buckets) {
			bucket.sort((left, right) => left.name.localeCompare(right.name));
			if (bucket.length === 1) {
				groups.push({
					key: `${key}::single`,
					label: bucket[0].name,
					description: bucket[0].description ?? '',
					results: bucket,
					isGrouped: false
				});
				continue;
			}

			const label = commonVariantLabel(bucket);
			if (!label) {
				for (const result of bucket) {
					groups.push({
						key: `${key}::${result.path}`,
						label: result.name,
						description: result.description ?? '',
						results: [result],
						isGrouped: false
					});
				}
				continue;
			}

			groups.push({
				key,
				label,
				description: `${bucket.length} options in this family`,
				results: bucket,
				isGrouped: true
			});
		}

		return groups.toSorted((left, right) => left.label.localeCompare(right.label));
	}

	function parentDirectory(path: string) {
		const normalized = path.replaceAll('\\', '/');
		const slash = normalized.lastIndexOf('/');
		return slash === -1 ? normalized : normalized.slice(0, slash);
	}

	function commonVariantLabel(results: ScanResult[]) {
		const tokenized = results.map((result) =>
			result.name
				.toLowerCase()
				.replaceAll(/[_-]+/g, ' ')
				.replaceAll(/\b(v|ver|version)\s*\d+(?:\.\d+)*\b/g, '')
				.replaceAll(/\b\d+(?:x|%)\b/g, '')
				.trim()
				.split(/\s+/)
				.filter(Boolean)
		);

		if (tokenized.length === 0) return null;
		const prefix: string[] = [];
		for (let index = 0; index < tokenized[0].length; index += 1) {
			const token = tokenized[0][index];
			if (tokenized.every((tokens) => tokens[index] === token)) {
				prefix.push(token);
			} else {
				break;
			}
		}

		if (prefix.length === 0) return null;
		return prefix.map((token) => token.charAt(0).toUpperCase() + token.slice(1)).join(' ');
	}
</script>

<svelte:head><title>Data Mods • Crimson Desert Mod Workbench</title></svelte:head>

<div
	class="mx-auto flex min-h-full w-full max-w-5xl flex-col gap-6 px-4 py-6 sm:px-6 lg:px-8 lg:py-8"
>
	<div class="space-y-2">
		<p class="text-xs font-medium tracking-[0.24em] text-muted-foreground uppercase">Data Mods</p>
		<h1 class="text-3xl font-semibold tracking-tight">JSON mod workflow</h1>
		<p class="max-w-3xl text-sm leading-7 text-muted-foreground">
			Import JSON patch mods, reorder them, and toggle individual patch groups before preview and
			apply.
		</p>
	</div>

	<Card.Root>
		<Card.Header
			><Card.Title class="flex items-center gap-2"
				><HardDriveDownload class="size-5" /> Import source</Card.Title
			><Card.Description
				>Scan a folder or ZIP/7Z/RAR archive and import one candidate at a time.</Card.Description
			></Card.Header
		>
		<Card.Content class="space-y-4">
			<div class="flex flex-wrap gap-2">
				<Button disabled={manager.busy.scanningMods} onclick={chooseFolder}
					><FolderSearch class="size-4" />
					{manager.busy.scanningMods ? 'Scanning...' : 'Choose folder'}</Button
				>
				<Button variant="outline" disabled={manager.busy.scanningMods} onclick={chooseZip}
					><HardDriveDownload class="size-4" /> Choose archive</Button
				>
				{#if manager.scanResults.length > 1}
					<Button
						variant="outline"
						disabled={manager.busy.importing}
						onclick={() => manager.importScanResults(manager.scanResults)}
						>Import all scanned mods</Button
					>
				{/if}
			</div>
			{#if importSourcePath}<p class="text-sm break-all text-muted-foreground">
					{importSourcePath}
				</p>{/if}
			{#if manager.scanResults.length === 0}
				<Empty.Root class="min-h-40 border-dashed bg-muted/20 p-8"
					><Empty.Header
						><Empty.Title>No scanned candidates</Empty.Title><Empty.Description
							>Pick a folder or ZIP/7Z/RAR archive to preview importable JSON, precompiled, and
							browser/raw mods.</Empty.Description
						></Empty.Header
					></Empty.Root
				>
			{:else}
				<ScrollArea.Root class="h-96 rounded-xl border"
					><div class="space-y-3 p-3">
						{#each groupedScanResults as group (group.key)}
							<Collapsible.Root
								open={Boolean(variantGroupsOpen[group.key])}
								class="rounded-xl border bg-muted/20 px-4 py-4"
							>
								<div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
									<div class="space-y-2">
										<p class="font-medium">{group.label}</p>
										<p class="text-sm text-muted-foreground">
											{group.isGrouped ? group.description : group.results[0].fileName}
										</p>
										<div class="flex flex-wrap gap-2">
											<Badge variant="secondary">{group.results[0].modKind}</Badge>
											<Badge variant="outline"
												>{group.results.length} option{group.results.length === 1 ? '' : 's'}</Badge
											>
											<Badge variant="outline"
												>{group.results.reduce((sum, result) => sum + result.changeCount, 0)} total changes</Badge
											>
										</div>
									</div>
									<div class="flex gap-2">
										{#if group.results.length === 1}
											<Button
												size="sm"
												disabled={manager.busy.importing}
												onclick={() => manager.importScanResult(group.results[0])}
											>
												Import
											</Button>
										{/if}
										{#if group.results.length > 1}
											<Button
												variant="outline"
												size="sm"
												disabled={manager.busy.importing}
												onclick={() => manager.importScanResults(group.results)}>Import all</Button
											>
										{/if}
										<Button
											variant="outline"
											size="sm"
											onclick={() => (variantGroupsOpen[group.key] = !variantGroupsOpen[group.key])}
										>
											{variantGroupsOpen[group.key]
												? 'Hide options'
												: group.results.length > 1
													? 'Show options'
													: 'Details'}
										</Button>
									</div>
								</div>

								<Collapsible.Content class="pt-4">
									<Separator class="mb-4" />
									<div class="space-y-3">
										{#each group.results as result (result.path)}
											<div class="rounded-lg border bg-background/60 px-4 py-4">
												<div
													class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between"
												>
													<div class="space-y-2">
														<p class="font-medium">{result.name}</p>
														<p class="text-sm text-muted-foreground">{result.fileName}</p>
														<div class="flex flex-wrap gap-2">
															<Badge variant="outline">{result.patchCount} groups</Badge>
															<Badge variant="outline">{result.changeCount} changes</Badge>
														</div>
													</div>
													<div class="flex gap-2">
														<Button
															variant="outline"
															size="sm"
															onclick={() =>
																(scanDetailsOpen[result.path] = !scanDetailsOpen[result.path])}
														>
															{scanDetailsOpen[result.path] ? 'Hide details' : 'Details'}
														</Button>
														<Button
															size="sm"
															disabled={manager.busy.importing}
															onclick={() => manager.importScanResult(result)}>Import</Button
														>
													</div>
												</div>

												{#if scanDetailsOpen[result.path]}
													<div class="mt-4">
														<Separator class="mb-4" />
														{#if result.description}
															<p class="text-sm leading-6 text-muted-foreground">
																{result.description}
															</p>
														{/if}
														<div class="mt-4 flex flex-wrap gap-2">
															{#each result.targetFiles as target (target)}
																<Badge variant="outline">{target}</Badge>
															{/each}
														</div>
													</div>
												{/if}
											</div>
										{/each}
									</div>
								</Collapsible.Content>
							</Collapsible.Root>
						{/each}
					</div></ScrollArea.Root
				>
			{/if}
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header
			><Card.Title class="flex items-center gap-2"
				><ArrowDownUp class="size-5" /> JSON load order</Card.Title
			><Card.Description
				>Lower items win when multiple JSON mods target the same entry or byte range.</Card.Description
			></Card.Header
		>
		<Card.Content>
			{#if manager.orderedJsonMods.length === 0}
				<Alert.Root
					><Info class="size-4" /><Alert.Title>No enabled JSON mods</Alert.Title><Alert.Description
						>Enable one or more JSON mods to establish apply order.</Alert.Description
					></Alert.Root
				>
			{:else}
				<div class="space-y-3">
					{#each manager.orderedJsonMods as mod, index (mod.id)}<div
							class="flex flex-col gap-3 rounded-xl border bg-muted/20 px-4 py-4 sm:flex-row sm:items-center sm:justify-between"
						>
							<div>
								<p class="font-medium">{index + 1}. {mod.name}</p>
								<p class="text-sm text-muted-foreground">{mod.fileName}</p>
							</div>
							<div class="flex gap-2">
								<Button
									variant="outline"
									size="sm"
									disabled={index === 0}
									onclick={() => manager.moveMod(mod, 'up')}>Up</Button
								><Button
									variant="outline"
									size="sm"
									disabled={index === manager.orderedJsonMods.length - 1}
									onclick={() => manager.moveMod(mod, 'down')}>Down</Button
								>
							</div>
						</div>{/each}
				</div>
			{/if}
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header
			><Card.Title class="flex items-center gap-2"><Info class="size-5" /> Patch toggles</Card.Title
			><Card.Description
				>Enable or disable individual JSON patch groups before preview and apply.</Card.Description
			></Card.Header
		>
		<Card.Content class="space-y-4">
			{#if manager.orderedJsonMods.length === 0}
				<Alert.Root
					><Info class="size-4" /><Alert.Title>No JSON patch groups available</Alert.Title
					><Alert.Description
						>Enable at least one JSON mod to manage patch groups.</Alert.Description
					></Alert.Root
				>
			{:else}
				<div class="flex flex-wrap gap-2">
					{#each manager.orderedJsonMods as mod (mod.id)}<Button
							variant={manager.selectedPatchModId === mod.id ? 'default' : 'outline'}
							size="sm"
							onclick={async () => {
								manager.selectedPatchModId = mod.id;
								await manager.refreshPatchSummaries();
							}}>{mod.name}</Button
						>{/each}
				</div>
				<ScrollArea.Root class="h-72 rounded-xl border"
					><div class="divide-y">
						{#each manager.patchSummaries as patch (patch.modId + ':' + patch.patchIndex)}<div
								class="flex flex-col gap-3 px-4 py-3 sm:flex-row sm:items-center sm:justify-between"
							>
								<div>
									<p class="text-sm font-medium">{patch.title}</p>
									<p class="text-xs text-muted-foreground">
										{patch.sourceGroup} / {patch.gameFile}
									</p>
									<p class="text-xs text-muted-foreground">{patch.changeCount} byte changes</p>
								</div>
								<Button
									variant={patch.enabled ? 'outline' : 'secondary'}
									size="sm"
									disabled={manager.busy.patches}
									onclick={() => manager.togglePatch(patch)}
									>{patch.enabled ? 'Disable' : 'Enable'}</Button
								>
							</div>{:else}<div class="px-4 py-8 text-sm text-muted-foreground">
								No patch groups found for the selected mod.
							</div>{/each}
					</div></ScrollArea.Root
				>
			{/if}
		</Card.Content>
	</Card.Root>
</div>
