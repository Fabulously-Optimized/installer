<script lang="ts">
	import { download_and_install_mrpack } from '$lib/installer';
	import { get_project, list_versions, type Version } from '$lib/modrinth';
	import { listen } from '@tauri-apps/api/event';
    const PROJECT_ID = "1KVo5zza";
    let totalMods = Infinity;
	listen('install:progress', (event) => {
		console.log(event.payload);
        const payload = event.payload as (string | number)[];
        if (payload[1] == "start") {
            state = 'installing';
            switch(payload[0]) {
                case 'load_pack':
                    installProgress.push("Downloading modpack");
                    break;
                case 'download_files':
                    installProgress.push("Downloading mods");
                    totalMods = payload[2] as number;
                    break;
                case 'download_file':
                    installProgress.push(`Downloading mod ${payload[2] as number + 1}/${totalMods}`);
                    currentStep = payload[2] as number + 1;
                    break;
                case 'extract_overrides':
                    installProgress.push("Extracting configuration files");
                    currentStep = totalMods + 1;
                    break;
                case 'install_loader':
                    installProgress.push("Installing mod loader");
                    currentStep = totalMods + 2;
                    break;
                case 'add_profile':
                    installProgress.push("Creating launcher profile");
                    currentStep = totalMods + 3;
                    break;
            }
            installProgress = installProgress;
        }
	});
	async function installPack() {
        state = 'installing';
        const version = versions?.find(e => e.id == selected);
        const url = version?.files.find(e => e.primary)?.url
        const mc_version = version?.game_versions[0] ?? "";
		download_and_install_mrpack(
			url ?? "",
            `fabulously-optimized-${mc_version}`,
            await (await fetch((await get_project(PROJECT_ID)).icon_url ?? "")).blob(),
            `Fabulously Optimized ${mc_version}`,
            isolateProfile ? `FO-${mc_version}` : undefined
		).then(() => {
            state = 'postInstall';
        }, (err) => {
            state = 'error'
            errorMessage = err;
            console.error(err);
        });
	}
    let versions: Version[] | undefined = undefined;
    let selected: string;
    let isolateProfile: boolean = false;
    list_versions(PROJECT_ID).then(result => {
        versions = result.filter(e => e.featured)
    })
    let state: 'preInstall' | 'installing' | 'postInstall' | 'error' = 'preInstall';
    let installProgress: string[] = [];
    $: totalSteps = totalMods + 4;
    let currentStep = 0;
    let errorMessage: string | undefined = undefined;
</script>
{#if state == 'preInstall'}
<select bind:value={selected} disabled={versions == undefined}>
    {#if versions == undefined}
    <option>Loading versions...</option>
    {:else}
    {#each versions as version}
        <option value={version.id}>{version.name}</option>
    {/each}
    {/if}
</select>
<input type="checkbox" bind:checked={isolateProfile} id="isolate-profile">
<label for="isolate-profile">Create a new .minecraft directory for this version?</label>
<button on:click={installPack} disabled={versions == undefined}>Install!</button>
{:else if state == 'installing'}
Installing... {(currentStep / totalSteps) * 100}%
<progress value={currentStep / totalSteps}></progress>
<ul>
    {#each installProgress as line}
    <li>{line}</li>
    {/each}
</ul>
{:else if state == 'postInstall'}
Fabulously Optimized is installed!
{:else}
An error occurred while installing Fabulously Optimized:
{errorMessage}
{/if}