<script lang="ts">
	import { onMount } from 'svelte';
	import { AlertCircle, Info, Sparkles } from '@lucide/svelte';
	import * as Alert from '$lib/components/ui/alert';
	import { Badge } from '$lib/components/ui/badge';
	import * as Card from '$lib/components/ui/card';
	import { manager } from '$lib/manager-state.svelte';

	onMount(() => {
		void manager.ensureLoaded();
	});
</script>

<svelte:head>
	<title>Overview • Crimson Desert Mod Workbench</title>
</svelte:head>

<div
	class="mx-auto flex min-h-full w-full max-w-5xl flex-col gap-6 px-4 py-6 sm:px-6 lg:px-8 lg:py-8"
>
	<div class="space-y-2">
		<p class="text-xs font-medium tracking-[0.24em] text-muted-foreground uppercase">Overview</p>
		<h1 class="text-3xl font-semibold tracking-tight sm:text-4xl">Crimson Desert mod workbench</h1>
		<p class="max-w-3xl text-sm leading-7 text-muted-foreground sm:text-base">
			Use the sidebar to move between real tool pages for JSON mods, language overlays, precompiled
			mods, apply logs, recovery, and advanced PATHC/extraction workflows.
		</p>
	</div>

	{#if manager.message}
		<Alert.Root variant={manager.message.kind === 'error' ? 'destructive' : 'default'}>
			<AlertCircle class="size-4" />
			<Alert.Title>{manager.message.title}</Alert.Title>
			<Alert.Description>{manager.message.message}</Alert.Description>
		</Alert.Root>
	{/if}

	{#if manager.recoveryPending}
		<Alert.Root variant="destructive">
			<AlertCircle class="size-4" />
			<Alert.Title>Recovery recommended</Alert.Title>
			<Alert.Description>
				The last operation may have been interrupted{#if manager.pendingOperation}
					during `{manager.pendingOperation}`{/if}. Open `Tools` and run `Fix Everything` to restore
				a clean state.
			</Alert.Description>
		</Alert.Root>
	{/if}

	<div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
		<Card.Root
			><Card.Content class="pt-6"
				><p class="text-xs tracking-[0.18em] text-muted-foreground uppercase">Library</p>
				<p class="mt-2 text-3xl font-semibold">{manager.totalCount}</p></Card.Content
			></Card.Root
		>
		<Card.Root
			><Card.Content class="pt-6"
				><p class="text-xs tracking-[0.18em] text-muted-foreground uppercase">Enabled</p>
				<p class="mt-2 text-3xl font-semibold">{manager.enabledCount}</p></Card.Content
			></Card.Root
		>
		<Card.Root
			><Card.Content class="pt-6"
				><p class="text-xs tracking-[0.18em] text-muted-foreground uppercase">Disabled</p>
				<p class="mt-2 text-3xl font-semibold">{manager.disabledCount}</p></Card.Content
			></Card.Root
		>
		<Card.Root
			><Card.Content class="pt-6"
				><p class="text-xs tracking-[0.18em] text-muted-foreground uppercase">Overlay</p>
				<p class="mt-2 text-3xl font-semibold">
					{manager.overlayActive ? 'Active' : 'Vanilla'}
				</p></Card.Content
			></Card.Root
		>
	</div>

	<Card.Root>
		<Card.Header>
			<Card.Title class="flex items-center gap-2"
				><Sparkles class="size-5" /> Current state</Card.Title
			>
			<Card.Description>Quick summary of the active install and previewed mod set.</Card.Description
			>
		</Card.Header>
		<Card.Content class="space-y-4">
			<div class="flex flex-wrap gap-2">
				<Badge variant="outline">{manager.install?.packagesPath ?? 'No game path saved'}</Badge>
				{#if manager.selectedLanguage}<Badge>{manager.selectedLanguage.toUpperCase()}</Badge>{/if}
				{#if manager.backupExists}<Badge variant="outline">Backup present</Badge>{/if}
				{#if manager.previewConflictFiles.length > 0}<Badge
						>{manager.previewConflictFiles.length} conflicts</Badge
					>{/if}
				{#if manager.previewUnresolvedFiles.length > 0}<Badge variant="secondary"
						>{manager.previewUnresolvedFiles.length} unresolved</Badge
					>{/if}
			</div>

			<div class="grid gap-4 lg:grid-cols-2">
				<Card.Root class="border-dashed"
					><Card.Content class="pt-6"
						><p class="text-sm font-medium">Next best places to work</p>
						<ul class="mt-3 space-y-2 text-sm text-muted-foreground">
							<li>Data Mods: JSON load order, import candidates, patch toggles</li>
							<li>Apply & Logs: preview conflicts, unresolved targets, activity history</li>
							<li>Tools: game path setup, restore, recovery</li>
							<li>Advanced: PATHC DDS indexing and virtual file extraction</li>
						</ul></Card.Content
					></Card.Root
				>
				<Card.Root class="border-dashed"
					><Card.Content class="pt-6"
						><p class="text-sm font-medium">Current preview</p>
						{#if manager.applyPreview}<div class="mt-3 flex flex-wrap gap-2">
								<Badge variant="outline">{manager.applyPreview.modCount} active mods</Badge><Badge
									variant="outline">{manager.applyPreview.targetFileCount} JSON targets</Badge
								><Badge variant="outline">{manager.applyPreview.estimatedGroupCount} groups</Badge>
							</div>{:else}<p class="mt-3 text-sm text-muted-foreground">
								No preview loaded yet.
							</p>{/if}</Card.Content
					></Card.Root
				>
			</div>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title class="flex items-center gap-2"><Info class="size-5" /> Workspace map</Card.Title>
			<Card.Description
				>The sidebar now maps directly to pages instead of a single scroll target.</Card.Description
			>
		</Card.Header>
		<Card.Content class="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
			<div class="rounded-xl border bg-muted/20 p-4 text-sm">
				<p class="font-medium">Data Mods</p>
				<p class="mt-1 text-muted-foreground">
					Import sources, order JSON mods, toggle patch groups.
				</p>
			</div>
			<div class="rounded-xl border bg-muted/20 p-4 text-sm">
				<p class="font-medium">Apply & Logs</p>
				<p class="mt-1 text-muted-foreground">
					Preview conflicts, inspect results, and review history.
				</p>
			</div>
			<div class="rounded-xl border bg-muted/20 p-4 text-sm">
				<p class="font-medium">Tools</p>
				<p class="mt-1 text-muted-foreground">
					Game path setup, recovery, launch and restore actions.
				</p>
			</div>
			<div class="rounded-xl border bg-muted/20 p-4 text-sm">
				<p class="font-medium">Advanced</p>
				<p class="mt-1 text-muted-foreground">PATHC workflow and archive file extraction.</p>
			</div>
		</Card.Content>
	</Card.Root>
</div>
