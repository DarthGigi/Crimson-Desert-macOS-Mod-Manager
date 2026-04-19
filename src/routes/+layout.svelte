<script lang="ts">
	import { onMount } from 'svelte';
	import { MoonStar, Sun } from '@lucide/svelte';
	import { ModeWatcher, mode, toggleMode } from 'mode-watcher';
	import './layout.css';
	import AppSidebar from '$lib/components/app-sidebar.svelte';
	import { Button } from '$lib/components/ui/button';
	import { manager } from '$lib/manager-state.svelte';
	import * as Sidebar from '$lib/components/ui/sidebar';
	import { Toaster } from '$lib/components/ui/sonner';

	const { children } = $props();

	onMount(() => {
		void manager.ensureLoaded();
	});
</script>

<ModeWatcher defaultMode="dark" />

<Sidebar.Provider>
	<AppSidebar />
	<Sidebar.Inset>
		<header
			class="sticky top-0 z-20 flex h-14 items-center gap-3 border-b bg-background/80 px-4 backdrop-blur sm:px-6"
		>
			<Sidebar.Trigger />
			<div class="min-w-0">
				<p class="text-sm font-medium">Crimson Desert Mod Workbench</p>
				<p class="text-xs text-muted-foreground">
					Overlay-safe manager for JSON, precompiled, and language mods.
				</p>
			</div>
			<div class="ml-auto">
				<Button variant="outline" size="icon-sm" onclick={toggleMode} aria-label="Toggle theme" title={mode.current === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}>
					{#if mode.current === 'dark'}
						<Sun class="size-4" />
					{:else}
						<MoonStar class="size-4" />
					{/if}
				</Button>
			</div>
		</header>
		<div class="min-h-[calc(100svh-3.5rem)]">{@render children()}</div>
	</Sidebar.Inset>
</Sidebar.Provider>
<Toaster richColors position="top-right" />
