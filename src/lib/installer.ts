import { invoke } from '@tauri-apps/api/tauri';
function blobToDataURL(blob: Blob): Promise<string> {
	return new Promise((resolve, _) => {
		const reader = new FileReader();
		reader.onloadend = () => resolve(reader.result as string);
		reader.readAsDataURL(blob);
	});
}

export async function install_mrpack(
	url: string,
	pack_id: string,
	icon: Blob | undefined,
	pack_name: string,
    profile_dir: string | undefined,
	extra_metadata: unknown
): Promise<void> {
	await invoke('install_mrpack', {
		url: url,
		packId: pack_id,
		icon: icon != undefined ? await blobToDataURL(icon) : undefined,
		packName: pack_name,
        profileDir: profile_dir,
		extraMetadata: extra_metadata
	});
}

export async function get_installed_metadata(profile_dir: string | undefined): Promise<unknown> {
	return await invoke('get_installed_metadata', {
        profileDir: profile_dir
	});
}