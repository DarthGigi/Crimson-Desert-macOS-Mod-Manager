<script lang="ts">
	import { onMount } from 'svelte';
	import { Globe2 } from '@lucide/svelte';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import * as Empty from '$lib/components/ui/empty';
	import { manager } from '$lib/manager-state.svelte';

	const languageOptions = [
		'eng',
		'jpn',
		'kor',
		'rus',
		'tur',
		'spa_es',
		'spa_mx',
		'fre',
		'ger',
		'ita',
		'pol',
		'por_br',
		'zho_tw',
		'zho_cn'
	];

	onMount(() => {
		void manager.ensureLoaded();
	});
</script>

<svelte:head><title>Language Mods • Crimson Desert Mod Workbench</title></svelte:head>

<div
	class="mx-auto flex min-h-full w-full max-w-5xl flex-col gap-6 px-4 py-6 sm:px-6 lg:px-8 lg:py-8"
>
	<div class="space-y-2">
		<p class="text-xs font-medium tracking-[0.24em] text-muted-foreground uppercase">
			Language Mods
		</p>
		<h1 class="text-3xl font-semibold tracking-tight">Language-targeted overlays</h1>
		<p class="max-w-3xl text-sm leading-7 text-muted-foreground">
			Choose the language you use in the game. Then mark only translation/localization mods as
			`language mods`. Those mods will only install when their saved language matches the one you
			select here.
		</p>
	</div>
	<Card.Root
		><Card.Header
			><Card.Title class="flex items-center gap-2"
				><Globe2 class="size-5" /> Selected language</Card.Title
			><Card.Description
				>This is for localization-style mods only. Normal gameplay or texture mods should stay in
				their regular categories.</Card.Description
			></Card.Header
		><Card.Content class="space-y-4"
			><div class="flex flex-wrap gap-2">
				<Button
					variant={manager.selectedLanguage === null ? 'default' : 'outline'}
					size="sm"
					onclick={() => manager.chooseLanguage(null)}>None</Button
				>{#each languageOptions as language (language)}<Button
						variant={manager.selectedLanguage === language ? 'default' : 'outline'}
						size="sm"
						onclick={() => manager.chooseLanguage(language)}>{language.toUpperCase()}</Button
					>{/each}
			</div>
			<p class="text-sm text-muted-foreground">
				Current selection: <span class="font-medium text-foreground"
					>{manager.selectedLanguage?.toUpperCase() ?? 'Not set'}</span
				>
			</p></Card.Content
		></Card.Root
	>
	<Card.Root
		><Card.Content class="pt-6"
			><div class="rounded-xl border bg-muted/20 p-4 text-sm leading-6 text-muted-foreground">
				<p class="font-medium text-foreground">When should you use this?</p>
				<p class="mt-2">
					Use the language lane for subtitle, text, font, and translation mods that should only
					activate for one in-game language.
				</p>
				<p class="mt-2">
					Do not use it for regular gameplay changes like stamina, item balance, textures, or UI
					mods unless the mod author specifically says it is language-specific.
				</p>
			</div></Card.Content
		></Card.Root
	>
	<Card.Root
		><Card.Header
			><Card.Title>Language-classified mods</Card.Title><Card.Description
				>These mods will only install when their target language matches the current selection.</Card.Description
			></Card.Header
		><Card.Content
			>{#if manager.languageMods.length === 0}<Empty.Root
					class="min-h-40 border-dashed bg-muted/20 p-8"
					><Empty.Header
						><Empty.Title>No language mods classified</Empty.Title><Empty.Description
							>Classify imported mods as language mods from the Library or Precompiled Mods pages.</Empty.Description
						></Empty.Header
					></Empty.Root
				>{:else}<div class="space-y-3">
					{#each manager.languageMods as mod (mod.id)}<div
							class="rounded-xl border bg-muted/20 px-4 py-4"
						>
							<div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
								<div>
									<p class="font-medium">{mod.name}</p>
									<p class="text-sm text-muted-foreground">{mod.fileName}</p>
								</div>
								<div class="flex flex-wrap gap-2">
									<Badge>{mod.language?.toUpperCase() ?? 'Unassigned'}</Badge><Badge
										variant="outline">{mod.modKind}</Badge
									><Button
										variant="outline"
										size="sm"
										onclick={() => manager.classifyMod(mod, 'json_data', 'json_data')}
										>Unset language</Button
									><Button variant="destructive" size="sm" onclick={() => manager.removeMod(mod)}
										>Remove</Button
									>
								</div>
							</div>
						</div>{/each}
				</div>{/if}</Card.Content
		></Card.Root
	>
</div>
