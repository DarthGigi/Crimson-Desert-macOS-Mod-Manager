<script lang="ts">
	import { onMount } from 'svelte';
	import { CheckCircle2, Info, Layers3, RefreshCcw, Sparkles } from '@lucide/svelte';
	import * as Alert from '$lib/components/ui/alert';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import * as ScrollArea from '$lib/components/ui/scroll-area';
	import { formatTimestamp } from '$lib/manager-helpers';
	import { manager } from '$lib/manager-state.svelte';

	onMount(() => {
		void manager.ensureLoaded();
	});
</script>

<svelte:head><title>Apply & Logs • Crimson Desert Mod Workbench</title></svelte:head>

<div
	class="mx-auto flex min-h-full w-full max-w-5xl flex-col gap-6 px-4 py-6 sm:px-6 lg:px-8 lg:py-8"
>
	<div class="space-y-2">
		<p class="text-xs font-medium tracking-[0.24em] text-muted-foreground uppercase">
			Apply & Logs
		</p>
		<h1 class="text-3xl font-semibold tracking-tight">Overlay lifecycle</h1>
		<p class="max-w-3xl text-sm leading-7 text-muted-foreground">
			Preview conflicts before apply, inspect the last result, and review recent manager actions.
		</p>
	</div>
	<div class="flex flex-wrap gap-2">
		<Button
			variant="outline"
			disabled={manager.busy.previewing}
			onclick={() => manager.refreshPreview()}
			>{manager.busy.previewing ? 'Refreshing preview...' : 'Refresh preview'}</Button
		><Button
			disabled={!manager.install || manager.enabledCount === 0 || manager.busy.applying}
			onclick={() => manager.runApply()}><Sparkles class="size-4" /> Apply enabled mods</Button
		><Button
			variant="outline"
			disabled={!manager.install || manager.busy.restoring}
			onclick={() => manager.runRestore()}
			><RefreshCcw class="size-4" /> Restore vanilla overlay</Button
		><Button
			variant="destructive"
			disabled={manager.busy.resetting}
			onclick={() => manager.runReset()}>Reset active mods</Button
		>
	</div>
	<Card.Root
		><Card.Header
			><Card.Title class="flex items-center gap-2"><Info class="size-5" /> Apply preview</Card.Title
			><Card.Description
				>Preview the active mod set before creating fresh manager-owned overlay groups.</Card.Description
			></Card.Header
		><Card.Content class="space-y-4"
			>{#if !manager.applyPreview}<Alert.Root
					><Info class="size-4" /><Alert.Title>No preview available</Alert.Title><Alert.Description
						>Save a valid game path and enable at least one mod to build an apply preview.</Alert.Description
					></Alert.Root
				>{:else}<div class="flex flex-wrap gap-2">
					<Badge variant="outline">{manager.applyPreview.modCount} active mods</Badge><Badge
						variant="outline">{manager.applyPreview.targetFileCount} JSON targets</Badge
					><Badge variant="outline"
						>{manager.applyPreview.estimatedGroupCount} estimated groups</Badge
					>{#if manager.previewConflictFiles.length > 0}<Badge
							>{manager.previewConflictFiles.length} conflicts</Badge
						>{/if}{#if manager.previewUnresolvedFiles.length > 0}<Badge variant="secondary"
							>{manager.previewUnresolvedFiles.length} unresolved</Badge
						>{/if}{#if manager.applyPreview.selectedLanguage}<Badge
							>{manager.applyPreview.selectedLanguage.toUpperCase()}</Badge
						>{/if}
				</div>
				{#if manager.previewConflictFiles.length > 0 || manager.previewUnresolvedFiles.length > 0}<div
						class="grid gap-3 sm:grid-cols-2"
					>
						{#if manager.previewConflictFiles.length > 0}<div
								class="rounded-xl border bg-muted/20 p-4"
							>
								<p class="text-xs tracking-[0.18em] text-muted-foreground uppercase">
									Conflict files
								</p>
								<div class="mt-3 space-y-2 text-sm">
									{#each manager.previewConflictFiles.slice(0, 5) as file (file.sourceGroup + file.gameFile)}<p
											class="break-all"
										>
											{file.gameFile}
											<span class="text-muted-foreground">({file.overlapCount} overlaps)</span>
										</p>{/each}
								</div>
							</div>{/if}{#if manager.previewUnresolvedFiles.length > 0}<div
								class="rounded-xl border bg-muted/20 p-4"
							>
								<p class="text-xs tracking-[0.18em] text-muted-foreground uppercase">
									Unresolved files
								</p>
								<div class="mt-3 space-y-2 text-sm">
									{#each manager.previewUnresolvedFiles.slice(0, 5) as file (file.sourceGroup + file.gameFile)}<p
											class="break-all"
										>
											{file.gameFile} <span class="text-muted-foreground">({file.reason})</span>
										</p>{/each}
								</div>
							</div>{/if}
					</div>{/if}<ScrollArea.Root class="h-72 rounded-xl border"
					><div class="divide-y">
						{#each manager.applyPreview.files as file (file.sourceGroup + ':' + file.gameFile)}<div
								class="space-y-2 px-4 py-3"
							>
								<div class="flex flex-wrap items-center justify-between gap-3">
									<div>
										<p class="text-sm font-medium break-all">{file.gameFile}</p>
										<p class="text-xs text-muted-foreground">
											{file.sourceGroup}{#if file.sourcePazIndex !== null}
												/ PAZ {file.sourcePazIndex}{/if}
										</p>
									</div>
									<div class="flex flex-wrap gap-2">
										<Badge variant={file.resolved ? 'outline' : 'secondary'}
											>{file.resolved ? 'Resolved' : 'Unresolved'}</Badge
										><Badge variant="outline">{file.changeCount} changes</Badge
										>{#if file.overlapCount > 0}<Badge>{file.overlapCount} overlaps</Badge>{/if}
									</div>
								</div>
								<p class="text-xs text-muted-foreground">Mods: {file.sourceMods.join(', ')}</p>
								{#if file.reason}<p class="text-xs text-destructive">{file.reason}</p>{/if}
							</div>{/each}
					</div></ScrollArea.Root
				>{/if}</Card.Content
		></Card.Root
	>
	<Card.Root
		><Card.Header
			><Card.Title class="flex items-center gap-2"
				><Layers3 class="size-5" /> Current overlay state</Card.Title
			></Card.Header
		><Card.Content class="space-y-2 text-sm text-muted-foreground"
			><p>Overlay: {manager.overlayActive ? 'active' : 'vanilla'}</p>
			<p>Backup: {manager.backupExists ? '0.papgt.bak present' : 'not created yet'}</p>
			<p>Enabled mods queued: {manager.enabledCount}</p>
			<p>Writable install: {manager.install?.writable ? 'yes' : 'unknown / no'}</p></Card.Content
		></Card.Root
	>
	{#if manager.lastApplyResult}<Card.Root
			><Card.Header
				><Card.Title class="flex items-center gap-2"
					><CheckCircle2 class="size-5" /> Last apply result</Card.Title
				><Card.Description>{manager.lastApplyResult.message}</Card.Description></Card.Header
			><Card.Content class="space-y-4"
				><div class="flex flex-wrap gap-2">
					<Badge variant="outline">{manager.lastApplyResult.modCount} mods</Badge><Badge
						variant="outline">{manager.lastApplyResult.overlayFileCount} files</Badge
					><Badge variant="outline">{manager.lastApplyResult.pazSize} byte PAZ</Badge>
				</div>
				<ScrollArea.Root class="h-60 rounded-xl border"
					><div class="divide-y">
						{#each manager.lastApplyResult.files as file (file.gameFile)}<div
								class="space-y-1 px-4 py-3"
							>
								<div class="flex items-start justify-between gap-3">
									<p class="text-sm font-medium break-all">{file.gameFile}</p>
									<Badge variant="secondary">PAZ {file.sourcePazIndex}</Badge>
								</div>
								<p class="text-xs text-muted-foreground">
									Applied {file.appliedChanges}, skipped {file.skippedChanges}{#if file.reason}
										- {file.reason}{/if}
								</p>
							</div>{/each}
					</div></ScrollArea.Root
				></Card.Content
			></Card.Root
		>{/if}
	<Card.Root
		><Card.Header
			><div class="flex items-center justify-between gap-3">
				<div>
					<Card.Title class="flex items-center gap-2"
						><Info class="size-5" /> Activity log</Card.Title
					><Card.Description
						>Recent operations and recovery actions recorded by the app.</Card.Description
					>
				</div>
				<Button
					variant="outline"
					size="sm"
					disabled={manager.busy.history}
					onclick={() => manager.refreshHistory()}
					>{manager.busy.history ? 'Refreshing...' : 'Refresh log'}</Button
				>
			</div></Card.Header
		><Card.Content
			><ScrollArea.Root class="h-64 rounded-xl border"
				><div class="divide-y">
					{#each manager.historyEntries as entry (entry.id)}<div
							class="space-y-1 px-4 py-3 text-sm"
						>
							<div class="flex flex-wrap items-center justify-between gap-3">
								<div class="flex items-center gap-2">
									<Badge variant={entry.status === 'ok' ? 'outline' : 'secondary'}
										>{entry.action}</Badge
									>
									<p class="font-medium">{entry.message}</p>
								</div>
								<p class="text-xs text-muted-foreground">{formatTimestamp(entry.createdAt)}</p>
							</div>
						</div>{:else}<div class="px-4 py-8 text-sm text-muted-foreground">
							No history entries yet.
						</div>{/each}
				</div></ScrollArea.Root
			></Card.Content
		></Card.Root
	>
</div>
