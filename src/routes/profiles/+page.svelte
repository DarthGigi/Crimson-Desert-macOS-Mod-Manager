<script lang="ts">
	import { onMount } from 'svelte';
	import { Sparkles } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as Empty from '$lib/components/ui/empty';
	import { manager } from '$lib/manager-state.svelte';

	let name = $state('');
	let description = $state('');

	onMount(async () => {
		await manager.ensureLoaded();
		await manager.refreshProfiles();
	});
</script>

<svelte:head>
	<title>Profiles • Crimson Desert Mod Workbench</title>
</svelte:head>

<div
	class="mx-auto flex min-h-full w-full max-w-5xl flex-col gap-6 px-4 py-6 sm:px-6 lg:px-8 lg:py-8"
>
	<div class="space-y-2">
		<p class="text-xs font-medium tracking-[0.24em] text-muted-foreground uppercase">Profiles</p>
		<h1 class="text-3xl font-semibold tracking-tight">Saved mod sets</h1>
		<p class="max-w-3xl text-sm leading-7 text-muted-foreground">
			Save the current enabled mod set as a reusable profile, then reapply it later with one click.
		</p>
	</div>

	<Card.Root>
		<Card.Header>
			<Card.Title class="flex items-center gap-2"
				><Sparkles class="size-5" /> Create profile</Card.Title
			>
			<Card.Description>Profiles capture the currently enabled mods.</Card.Description>
		</Card.Header>
		<Card.Content class="space-y-4">
			<div class="space-y-2">
				<Label for="profile-name">Name</Label>
				<Input id="profile-name" bind:value={name} placeholder="My farming setup" />
			</div>
			<div class="space-y-2">
				<Label for="profile-description">Description</Label>
				<Input
					id="profile-description"
					bind:value={description}
					placeholder="Optional note about this mod set"
				/>
			</div>
			<Button onclick={() => manager.createProfile(name, description || null)}
				>Create from current enabled mods</Button
			>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Saved profiles</Card.Title>
			<Card.Description
				>Apply a profile to replace the current enabled set, or resave it from current state.</Card.Description
			>
		</Card.Header>
		<Card.Content>
			{#if manager.profiles.length === 0}
				<Empty.Root class="min-h-40 border-dashed bg-muted/20 p-8">
					<Empty.Header>
						<Empty.Title>No profiles yet</Empty.Title>
						<Empty.Description
							>Create your first profile from the currently enabled mods.</Empty.Description
						>
					</Empty.Header>
				</Empty.Root>
			{:else}
				<div class="space-y-3">
					{#each manager.profiles as profile (profile.id)}
						<div class="rounded-xl border bg-muted/20 px-4 py-4">
							<div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
								<div>
									<p class="font-medium">{profile.name}</p>
									{#if profile.description}<p class="text-sm text-muted-foreground">
											{profile.description}
										</p>{/if}
								</div>
								<div class="flex flex-wrap gap-2">
									<Button size="sm" onclick={() => manager.applyProfile(profile.id)}>Apply</Button>
									<Button
										variant="outline"
										size="sm"
										onclick={() => manager.saveProfile(profile.id)}>Resave</Button
									>
									<Button
										variant="destructive"
										size="sm"
										onclick={() => manager.deleteProfile(profile.id)}>Delete</Button
									>
								</div>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</Card.Content>
	</Card.Root>
</div>
