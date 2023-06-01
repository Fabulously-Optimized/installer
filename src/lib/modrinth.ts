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
	version_type: "release" | "beta" | "alpha"
}

export interface VersionFile {
	url: string;
	primary: boolean;
}

export async function get_project(id: string): Promise<Project> {
	const resp = await fetch(`https://api.modrinth.com/v2/project/${id}`);
	return await resp.json();
}

export async function list_versions(id: string): Promise<Version[]> {
	const resp = await fetch(`https://api.modrinth.com/v2/project/${id}/version`);
	return await resp.json();
}
