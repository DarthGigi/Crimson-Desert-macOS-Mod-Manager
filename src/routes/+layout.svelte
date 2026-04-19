<script lang="ts">
	import { onMount } from 'svelte';
	import './layout.css';
	import AppSidebar from '$lib/components/app-sidebar.svelte';
	import { manager } from '$lib/manager-state.svelte';
	import * as Sidebar from '$lib/components/ui/sidebar';
	import { Toaster } from '$lib/components/ui/sonner';

	const { children } = $props();

	onMount(() => {
		void manager.ensureLoaded();
	});
</script>

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
		</header>
		<div class="min-h-[calc(100svh-3.5rem)]">{@render children()}</div>
	</Sidebar.Inset>
</Sidebar.Provider>
<Toaster richColors position="top-right" />
