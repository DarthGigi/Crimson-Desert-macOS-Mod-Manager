<script lang="ts">
	import { onMount } from 'svelte';
	import { Package } from '@lucide/svelte';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import * as Empty from '$lib/components/ui/empty';
	import { modKindLabel } from '$lib/manager-helpers';
	import { manager } from '$lib/manager-state.svelte';

	onMount(() => {
		void manager.ensureLoaded();
	});
</script>

<svelte:head><title>Precompiled Mods • Crimson Desert Mod Workbench</title></svelte:head>

<div
	class="mx-auto flex min-h-full w-full max-w-5xl flex-col gap-6 px-4 py-6 sm:px-6 lg:px-8 lg:py-8"
>
	<div class="space-y-2">
		<p class="text-xs font-medium tracking-[0.24em] text-muted-foreground uppercase">
			Precompiled Mods
		</p>
		<h1 class="text-3xl font-semibold tracking-tight">Folder-based precompiled overlays</h1>
		<p class="max-w-3xl text-sm leading-7 text-muted-foreground">
			These mods already ship numeric groups or browser/raw files and install into fresh
			manager-owned groups during apply. Browser/raw imports can also contain raw file types such
			as BNK soundbanks. Use `Use as language` only for translation or
			subtitle-style mods that should apply to one specific in-game language.
		</p>
	</div>
	<Card.Root
		><Card.Header
			><Card.Title class="flex items-center gap-2"
				><Package class="size-5" /> Imported precompiled/browser mods</Card.Title
			><Card.Description
				>Remove old folder-backed imports here, or mark one as a language mod only if it should
				apply to a specific in-game language.</Card.Description
			></Card.Header
		><Card.Content
			>{#if manager.precompiledMods.length === 0}<Empty.Root
					class="min-h-40 border-dashed bg-muted/20 p-8"
					><Empty.Header
						><Empty.Title>No folder-backed mods imported</Empty.Title><Empty.Description
							>Import a precompiled or browser/raw mod source from the Data Mods page.</Empty.Description
						></Empty.Header
					></Empty.Root
				>{:else}<div class="space-y-3">
					{#each manager.precompiledMods as mod (mod.id)}<div
							class="rounded-xl border bg-muted/20 px-4 py-4"
						>
							<div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
								<div>
									<p class="font-medium">{mod.name}</p>
									<p class="text-sm text-muted-foreground">{mod.fileName}</p>
								</div>
								<div class="flex flex-wrap gap-2">
									<Badge variant="outline">{modKindLabel(mod.modKind)}</Badge><Badge
										variant="outline">{mod.targetFiles.length} source groups</Badge
									><Button
										variant="outline"
										size="sm"
										onclick={() => manager.classifyMod(mod, 'language', mod.modKind)}
										>Use as language</Button
									><Button
										variant="destructive"
										size="sm"
										onclick={() => manager.removeMod(mod)}
										>Remove</Button
									>
								</div>
							</div>
						</div>{/each}
				</div>{/if}</Card.Content
		></Card.Root
	>
</div>
