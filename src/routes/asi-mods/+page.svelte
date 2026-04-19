<script lang="ts">
	import { onMount } from 'svelte';
	import { Package } from '@lucide/svelte';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import * as Empty from '$lib/components/ui/empty';
	import * as ScrollArea from '$lib/components/ui/scroll-area';
	import { manager } from '$lib/manager-state.svelte';

	onMount(async () => {
		await manager.ensureLoaded();
		await manager.refreshAsiPlugins();
	});
</script>

<svelte:head>
	<title>ASI Mods • Crimson Desert Mod Workbench</title>
</svelte:head>

<div class="mx-auto flex min-h-full w-full max-w-5xl flex-col gap-6 px-4 py-6 sm:px-6 lg:px-8 lg:py-8">
	<div class="space-y-2">
		<p class="text-xs font-medium tracking-[0.24em] text-muted-foreground uppercase">ASI Mods</p>
		<h1 class="text-3xl font-semibold tracking-tight">ASI plugins and companion files</h1>
		<p class="max-w-3xl text-sm leading-7 text-muted-foreground">
			Import `.asi` mods from the library, copy them into the managed external-mod directory,
			and enable/disable/remove installed plugins. On macOS, these are managed as compatibility-style
			external files rather than overlay content.
		</p>
	</div>

	<Card.Root>
		<Card.Header>
			<Card.Title class="flex items-center gap-2"><Package class="size-5" /> Imported ASI mods</Card.Title>
			<Card.Description>These are imported ASI sources stored in the manager library.</Card.Description>
		</Card.Header>
		<Card.Content>
			{#if manager.asiMods.length === 0}
				<Empty.Root class="min-h-40 border-dashed bg-muted/20 p-8">
					<Empty.Header>
						<Empty.Title>No ASI mods imported</Empty.Title>
						<Empty.Description>
							Import a folder, archive, or direct `.asi` mod source from the Data Mods page.
						</Empty.Description>
					</Empty.Header>
				</Empty.Root>
			{:else}
				<div class="space-y-3">
					{#each manager.asiMods as mod (mod.id)}
						<div class="rounded-xl border bg-muted/20 px-4 py-4">
							<div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
								<div>
									<p class="font-medium">{mod.name}</p>
									<p class="text-sm text-muted-foreground">{mod.fileName}</p>
								</div>
								<div class="flex flex-wrap gap-2">
									<Badge variant="outline">ASI</Badge>
									<Button size="sm" disabled={manager.busy.asi} onclick={() => manager.installAsiMod(mod)}>Install</Button>
									<Button variant="destructive" size="sm" onclick={() => manager.removeMod(mod)}>Remove import</Button>
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
			<div class="flex items-center justify-between gap-3">
				<div>
					<Card.Title class="flex items-center gap-2"><Package class="size-5" /> Installed ASI plugins</Card.Title>
					<Card.Description>Scanned from the managed external-mod directory for this install.</Card.Description>
				</div>
				<Button variant="outline" size="sm" disabled={manager.busy.asi} onclick={() => manager.refreshAsiPlugins()}>
					{manager.busy.asi ? 'Refreshing...' : 'Refresh'}
				</Button>
			</div>
		</Card.Header>
		<Card.Content>
			<ScrollArea.Root class="h-72 rounded-xl border">
				<div class="divide-y">
					{#each manager.asiPlugins as plugin (plugin.name)}
						<div class="space-y-3 px-4 py-3">
							<div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
								<div>
									<p class="font-medium">{plugin.name}</p>
									<p class="text-xs text-muted-foreground break-all">{plugin.path}</p>
								</div>
								<div class="flex flex-wrap gap-2">
									<Badge variant={plugin.enabled ? 'outline' : 'secondary'}>{plugin.enabled ? 'Enabled' : 'Disabled'}</Badge>
									<Button variant="outline" size="sm" onclick={() => manager.setAsiEnabled(plugin.name, !plugin.enabled)}>
										{plugin.enabled ? 'Disable' : 'Enable'}
									</Button>
									<Button variant="destructive" size="sm" onclick={() => manager.removeAsiPlugin(plugin.name)}>Remove</Button>
								</div>
							</div>
							{#if plugin.iniFiles.length > 0}
								<div class="flex flex-wrap gap-2">
									{#each plugin.iniFiles as ini (ini)}
										<Badge variant="outline">{ini}</Badge>
									{/each}
								</div>
							{/if}
						</div>
					{:else}
						<div class="px-4 py-8 text-sm text-muted-foreground">No installed ASI plugins detected yet.</div>
					{/each}
				</div>
			</ScrollArea.Root>
		</Card.Content>
	</Card.Root>
</div>
