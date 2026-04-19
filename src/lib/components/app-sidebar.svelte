<script lang="ts">
	import { page } from '$app/state';
	import { HardDriveDownload } from '@lucide/svelte';
	import * as Sidebar from '$lib/components/ui/sidebar';
	import { appNavGroups, appSidebarMeta } from '$lib/navigation';

	const currentPath = $derived(page.url.pathname);
</script>

<Sidebar.Root collapsible="icon" variant="inset">
	<Sidebar.Header class="gap-3 border-b px-3 py-4">
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton size="lg" isActive>
					{#snippet child({ props })}
						<a href="/" {...props}>
							<div
								class="flex size-9 items-center justify-center rounded-xl border border-white/10 bg-sidebar-primary text-sidebar-primary-foreground"
							>
								<appSidebarMeta.icon class="size-4" />
							</div>
							<div class="grid flex-1 text-left text-sm leading-tight">
								<span class="truncate font-medium">{appSidebarMeta.title}</span>
								<span class="truncate text-xs text-sidebar-foreground/70"
									>{appSidebarMeta.subtitle}</span
								>
							</div>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
		<p
			class="px-2 text-xs leading-5 text-sidebar-foreground/70 group-data-[collapsible=icon]:hidden"
		>
			{appSidebarMeta.description}
		</p>
	</Sidebar.Header>

	<Sidebar.Content>
		{#each appNavGroups as group (group.label)}
			<Sidebar.Group>
				<Sidebar.GroupLabel>{group.label}</Sidebar.GroupLabel>
				<Sidebar.GroupContent>
					<Sidebar.Menu>
						{#each group.items as item (item.id)}
							<Sidebar.MenuItem>
								<Sidebar.MenuButton
									isActive={currentPath === item.href}
									tooltipContent={item.label}
								>
									{#snippet child({ props })}
										<a href={item.href} {...props}>
											<item.icon />
											<span>{item.label}</span>
										</a>
									{/snippet}
								</Sidebar.MenuButton>
								{#if item.badge}
									<Sidebar.MenuBadge>{item.badge}</Sidebar.MenuBadge>
								{/if}
							</Sidebar.MenuItem>
						{/each}
					</Sidebar.Menu>
				</Sidebar.GroupContent>
			</Sidebar.Group>
		{/each}
	</Sidebar.Content>

	<Sidebar.Footer class="border-t px-3 py-3">
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton tooltipContent="Import folder">
					{#snippet child({ props })}
						<a href="/data-mods" {...props}>
							<HardDriveDownload />
							<span>Import Folder</span>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
			<Sidebar.MenuItem class="group-data-[collapsible=icon]:hidden">
				<div
					class="rounded-xl border border-sidebar-border bg-sidebar-accent/40 p-3 text-xs leading-5 text-sidebar-foreground/75"
				>
					<p class="font-medium text-sidebar-foreground">Current focus</p>
					<p class="mt-1">
						Routed workspace is live. Use the sidebar to move between mod types, apply tools, and
						advanced workflows.
					</p>
				</div>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Footer>
	<Sidebar.Rail />
</Sidebar.Root>
