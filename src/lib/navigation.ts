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
	| 'library'
	| 'apply-logs'
	| 'tools'
	| 'advanced';

export type AppSection = {
	id: AppSectionId;
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
				label: 'Overview',
				description: 'Install status, quick actions, and apply summary.',
				icon: Sparkles
			},
			{
				id: 'data-mods',
				label: 'Data Mods',
				description: 'Entry-based JSON data mods and load-order workbench.',
				icon: FolderKanban
			},
			{
				id: 'language-mods',
				label: 'Language Mods',
				description: 'Language-specific overlays and in-game language targeting.',
				icon: Globe2,
				badge: 'Soon'
			},
			{
				id: 'precompiled-mods',
				label: 'Precompiled Mods',
				description: 'Folder mods that ship prebuilt numeric groups and meta files.',
				icon: Package,
				badge: 'Soon'
			}
		]
	},
	{
		label: 'Operations',
		items: [
			{
				id: 'library',
				label: 'Library',
				description: 'Archive-first mod inventory across all supported types.',
				icon: Archive
			},
			{
				id: 'apply-logs',
				label: 'Apply & Logs',
				description: 'Overlay lifecycle, skipped patches, and recent output.',
				icon: Logs
			},
			{
				id: 'tools',
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
				label: 'Advanced',
				description: 'PATHC, raw assets, and format research landing zone.',
				icon: Settings2,
				badge: 'Later'
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
