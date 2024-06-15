import { getVersion } from '@tauri-apps/api/app';

export interface Project {
	icon_url?: string;
}

export interface Version {
	name: string;
	id: string;
	game_versions: string[];
	featured: boolean;
	files: VersionFile[];
	version_number: string;
	version_type: 'release' | 'beta' | 'alpha';
}

export interface VersionFile {
	url: string;
	primary: boolean;
	filename: string;
}

export async function get_project(id: string): Promise<Project> {
	const resp = await fetch(`https://api.modrinth.com/v2/project/${id}`, {
		headers: {
			'User-Agent': `Paigaldaja/${await getVersion()} (+https://github.com/Fabulously-Optimized/vanilla-installer-rust)`
		}
	});
	return await resp.json();
}

export async function list_versions(id: string): Promise<Version[]> {
	const resp = await fetch(`https://api.modrinth.com/v2/project/${id}/version`, {
		headers: {
			'User-Agent': `Paigaldaja/${await getVersion()} (+https://github.com/Fabulously-Optimized/vanilla-installer-rust)`
		}
	});
	return await resp.json();
}
