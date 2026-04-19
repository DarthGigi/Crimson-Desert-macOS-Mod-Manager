<script lang="ts">
	import { onMount } from 'svelte';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import * as Empty from '$lib/components/ui/empty';
	import { Input } from '$lib/components/ui/input';
	import * as ScrollArea from '$lib/components/ui/scroll-area';
	import * as Table from '$lib/components/ui/table';
	import { fallbackKindForLanguageMod, formatTimestamp, modKindLabel } from '$lib/manager-helpers';
	import { manager } from '$lib/manager-state.svelte';

	let search = $state('');
	let filter = $state<'all' | 'enabled' | 'disabled'>('all');
	const filteredMods = $derived.by(() => {
		const query = search.trim().toLowerCase();
		return manager.allMods.filter((mod) => {
			const matchesFilter = filter === 'all' || (filter === 'enabled' ? mod.enabled : !mod.enabled);
			const haystack = [mod.name, mod.fileName, mod.description ?? '', mod.targetFiles.join(' ')]
				.join(' ')
				.toLowerCase();
			return matchesFilter && (!query || haystack.includes(query));
		});
	});

	onMount(() => {
		void manager.ensureLoaded();
	});
</script>

<svelte:head><title>Library • Crimson Desert Mod Workbench</title></svelte:head>

<div
	class="mx-auto flex min-h-full w-full max-w-6xl flex-col gap-6 px-4 py-6 sm:px-6 lg:px-8 lg:py-8"
>
	<div class="space-y-2">
		<p class="text-xs font-medium tracking-[0.24em] text-muted-foreground uppercase">Library</p>
		<h1 class="text-3xl font-semibold tracking-tight">Archive-first mod inventory</h1>
		<p class="max-w-3xl text-sm leading-7 text-muted-foreground">
			Browse all imported mods, search by file or target path, and classify mods into the language
			lane when needed.
		</p>
	</div>
	<div class="space-y-2">
		<Input bind:value={search} placeholder="Search names, files, and targets" />
		<div class="flex flex-wrap gap-2">
			<Button
				variant={filter === 'all' ? 'default' : 'outline'}
				size="sm"
				onclick={() => (filter = 'all')}>All</Button
			><Button
				variant={filter === 'enabled' ? 'default' : 'outline'}
				size="sm"
				onclick={() => (filter = 'enabled')}>Enabled</Button
			><Button
				variant={filter === 'disabled' ? 'default' : 'outline'}
				size="sm"
				onclick={() => (filter = 'disabled')}>Archived</Button
			>
		</div>
	</div>
	<Card.Root
		><Card.Content class="pt-6"
			><ScrollArea.Root class="h-[38rem] rounded-xl border"
				>{#if filteredMods.length === 0}<Empty.Root class="border-0"
						><Empty.Header
							><Empty.Title>No matching mods</Empty.Title><Empty.Description
								>Import mods from Data Mods or change the current filter.</Empty.Description
							></Empty.Header
						></Empty.Root
					>{:else}<Table.Root
						><Table.Header
							><Table.Row
								><Table.Head>Mod</Table.Head><Table.Head>Targets</Table.Head><Table.Head
									>Imported</Table.Head
								><Table.Head class="text-right">State</Table.Head></Table.Row
							></Table.Header
						><Table.Body
							>{#each filteredMods as mod (mod.id)}<Table.Row
									><Table.Cell class="align-top"
										><div class="space-y-2">
											<div class="flex flex-wrap items-center gap-2">
												<p class="font-medium">{mod.name}</p>
												<Badge variant="outline">{modKindLabel(mod.modKind)}</Badge><Badge
													variant={mod.enabled ? 'default' : 'secondary'}
													>{mod.enabled ? 'Enabled' : 'Archived'}</Badge
												>{#if mod.language}<Badge>{mod.language.toUpperCase()}</Badge>{/if}
											</div>
											<p class="text-sm text-muted-foreground">{mod.fileName}</p>
											{#if mod.description}<p
													class="max-w-xl text-sm leading-6 text-muted-foreground"
												>
													{mod.description}
												</p>{/if}
										</div></Table.Cell
									><Table.Cell class="align-top"
										><div class="flex max-w-md flex-wrap gap-2 pt-1">
											{#each mod.targetFiles.slice(0, 3) as target (target)}<Badge variant="outline"
													>{target}</Badge
												>{/each}{#if mod.targetFiles.length > 3}<Badge variant="secondary"
													>+{mod.targetFiles.length - 3} more</Badge
												>{/if}
										</div></Table.Cell
									><Table.Cell class="align-top text-sm text-muted-foreground"
										>{formatTimestamp(mod.importedAt)}</Table.Cell
									><Table.Cell class="text-right align-top"
										><div class="flex justify-end gap-2">
											<Button
												variant={mod.enabled ? 'outline' : 'default'}
												size="sm"
												disabled={manager.busy.toggling === mod.id}
												onclick={() => manager.toggleMod(mod)}
												>{mod.enabled ? 'Disable' : 'Enable'}</Button
											>{#if mod.modKind === 'language'}<Button
													variant="outline"
													size="sm"
													onclick={() =>
														manager.classifyMod(
															mod,
															fallbackKindForLanguageMod(mod),
															fallbackKindForLanguageMod(mod)
														)}>Unset language</Button
												>{:else}<Button
													variant="outline"
													size="sm"
													onclick={() =>
														manager.classifyMod(mod, 'language', fallbackKindForLanguageMod(mod))}
													>Language</Button
												>{/if}
										</div></Table.Cell
									></Table.Row
								>{/each}</Table.Body
						></Table.Root
					>{/if}</ScrollArea.Root
			></Card.Content
		></Card.Root
	>
</div>
