import {
	Archive,
	FolderKanban,
	Gamepad2,
	Globe2,
	HardDriveDownload,
	Logs,
	Package,
	Settings2,
	Sparkles,
	Wrench
} from '@lucide/svelte';

export type AppSectionId =
	| 'overview'
	| 'data-mods'
	| 'language-mods'
	| 'precompiled-mods'
	| 'external-mods'
	| 'asi-mods'
	| 'library'
	| 'profiles'
	| 'apply-logs'
	| 'tools'
	| 'advanced';

export type AppSection = {
	id: AppSectionId;
	href: string;
	label: string;
	description: string;
	icon: any;
	badge?: string;
};

export type AppNavGroup = {
	label: string;
	items: AppSection[];
};

export const appNavGroups: AppNavGroup[] = [
	{
		label: 'Workspace',
		items: [
			{
				id: 'overview',
				href: '/',
				label: 'Overview',
				description: 'Install status, quick actions, and apply summary.',
				icon: Sparkles
			},
			{
				id: 'data-mods',
				href: '/data-mods',
				label: 'Data Mods',
				description: 'Entry-based JSON data mods and load-order workbench.',
				icon: FolderKanban
			},
			{
				id: 'language-mods',
				href: '/language-mods',
				label: 'Language Mods',
				description: 'Language-specific overlays and in-game language targeting.',
				icon: Globe2
			},
			{
				id: 'precompiled-mods',
				href: '/precompiled-mods',
				label: 'Precompiled Mods',
				description: 'Folder mods that ship prebuilt numeric groups and meta files.',
				icon: Package
			},
			{
				id: 'external-mods',
				href: '/external-mods',
				label: 'External Mods',
				description: 'ASI, BNK, binary patches, and script installers.',
				icon: Archive
			},
			{
				id: 'asi-mods',
				href: '/asi-mods',
				label: 'ASI Mods',
				description: 'Import and manage ASI plugins and companion files.',
				icon: Package
			}
		]
	},
	{
		label: 'Operations',
		items: [
			{
				id: 'library',
				href: '/library',
				label: 'Library',
				description: 'Archive-first mod inventory across all supported types.',
				icon: Archive
			},
			{
				id: 'profiles',
				href: '/profiles',
				label: 'Profiles',
				description: 'Save and restore curated sets of enabled mods.',
				icon: Sparkles
			},
			{
				id: 'apply-logs',
				href: '/apply-logs',
				label: 'Apply & Logs',
				description: 'Overlay lifecycle, skipped patches, and recent output.',
				icon: Logs
			},
			{
				id: 'tools',
				href: '/tools',
				label: 'Tools',
				description: 'Game path, restore, backup guidance, and launcher helpers.',
				icon: Wrench
			}
		]
	},
	{
		label: 'Research',
		items: [
			{
				id: 'advanced',
				href: '/advanced',
				label: 'Advanced',
				description: 'PATHC, raw assets, and format research landing zone.',
				icon: Settings2
			}
		]
	}
];

export const appSidebarMeta = {
	title: 'Crimson Desert',
	subtitle: 'Mod Workbench',
	description: 'JSON, precompiled, and language overlays for the macOS build.',
	icon: Gamepad2,
	importIcon: HardDriveDownload
};
