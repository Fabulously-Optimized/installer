<script lang="ts">
	import { download_and_install_mrpack } from '$lib/installer';
	import { listen } from '@tauri-apps/api/event';
	listen('install:progress', (event) => {
		console.log(event);
	});
	async function installFO() {
		download_and_install_mrpack(
			'https://cdn.modrinth.com/data/1KVo5zza/versions/7fkQgrjp/Fabulously%20Optimized-4.9.4.mrpack',
            'fabulously-optimized-1.19.4',
            await (await fetch("https://cdn-raw.modrinth.com/data/1KVo5zza/d8152911f8fd5d7e9a8c499fe89045af81fe816e.png")).blob(),
            "Fabulously Optimized 1.19.4"
		).catch((err) => {
            console.error(err);
        });
	}
</script>

<h1>Welcome to SvelteKit</h1>
<p>Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation</p>
<button on:click={installFO}>Install FO</button>