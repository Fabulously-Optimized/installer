import { invoke } from '@tauri-apps/api/tauri';
function blobToDataURL(blob: Blob): Promise<string> {
	return new Promise((resolve, _) => {
		const reader = new FileReader();
		reader.onloadend = () => resolve(reader.result as string);
		reader.readAsDataURL(blob);
	});
}

export async function download_and_install_mrpack(
	url: string,
	pack_id: string,
	icon: Blob,
	pack_name: string
): Promise<void> {
	invoke('download_and_install_mrpack', {
		url: url,
		packId: pack_id,
		icon: await blobToDataURL(icon),
		packName: pack_name
	});
}
