<script lang="ts">
	import { get_installed_metadata, install_mrpack } from '$lib/installer';
	import { get_project, list_versions, type Version } from '$lib/modrinth';
	import { listen } from '@tauri-apps/api/event';
    import semver from 'semver';
    const PROJECT_ID = "1KVo5zza";
    let totalMods = Infinity;
	listen('install:progress', (event) => {
		console.log(event.payload);
        const payload = event.payload as (string | number)[];
        if (payload[1] == "start") {
            state = 'installing';
            switch(payload[0]) {
                case 'clean_old':
                    installProgress.push("Cleaning up old files");
                    break;
                case 'load_pack':
                    installProgress.push("Downloading modpack");
                    currentStep = 1
                    break;
                case 'download_files':
                    installProgress.push("Downloading mods");
                    totalMods = payload[2] as number;
                    break;
                case 'download_file':
                    installProgress.push(`Downloading mod ${payload[2] as number + 1}/${totalMods}`);
                    currentStep = payload[2] as number + 2;
                    break;
                case 'extract_overrides':
                    installProgress.push("Extracting configuration files");
                    currentStep = totalMods + 2;
                    break;
                case 'install_loader':
                    installProgress.push("Installing mod loader");
                    currentStep = totalMods + 3;
                    break;
                case 'add_profile':
                    installProgress.push("Creating launcher profile");
                    currentStep = totalMods + 4;
                    break;
            }
            installProgress = installProgress;
        }
	});
	async function installPack() {
        try {
            const version = versions?.find(e => e.id == selected)!;
            const url = version?.files.find(e => e.primary)?.url!
            const mc_version = version?.game_versions[0];
            const profile_dir = isolateProfile ? `fabulously-optimized-${mc_version}` : undefined
            if (state != 'confirmDowngrade') {
                const installed_metadata = await get_installed_metadata(profile_dir);
                if (typeof installed_metadata == "object" && installed_metadata != null) {
                    if ("mc_version" in installed_metadata && typeof installed_metadata.mc_version == "string") {
                        const installed_mc_version = installed_metadata.mc_version;
                        if (semver.lt(mc_version, installed_mc_version)) {
                            state = 'confirmDowngrade'
                            confirmDowngrade = false
                            return
                        }
                    }
                }
            }
            state = 'installing';
            const project = await get_project(PROJECT_ID);
            const icon = await fetch(project.icon_url!);
            await install_mrpack(
                url,
                isolateProfile ? `fabulously-optimized-${mc_version}` : 'fabulously-optimized',
                await icon.blob(),
                isolateProfile ? `Fabulously Optimized ${mc_version}` : "Fabulously Optimized",
                profile_dir,
                {
                    fo_version: {
                        id: version.id,
                        version_number: version.version_number
                    },
                    mc_version: mc_version
                }
            )
            state = 'postInstall';
        } catch (e) {
            state = 'error'
            errorMessage = String(e);
            console.error(e);
        }
	}
    let versions: Version[] | undefined = undefined;
    let selected: string;
    let isolateProfile: boolean = false;
    list_versions(PROJECT_ID).then(result => {
        versions = result.filter(e => e.featured)
        selected = versions[0].id
    })
    let state: 'preInstall' | 'installing' | 'postInstall' | 'error' | 'confirmDowngrade' = 'preInstall';
    let installProgress: string[] = [];
    $: totalSteps = totalMods + 4;
    let currentStep = 0;
    let errorMessage: string | undefined = undefined;
    let confirmDowngrade = false;
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
{:else if state == 'error'}
An error occurred while installing Fabulously Optimized:
{errorMessage}
{:else}
Really downgrade version?
<input type="checkbox" bind:checked={confirmDowngrade} id="confirm-downgrade">
<label for="confirm-downgrade">I understand that downgrades can cause issues, and I want to downgrade the version.</label>
<button on:click={() => state = 'preInstall'}>Back</button>
<button on:click={installPack} disabled={!confirmDowngrade}>Continue</button>
{/if}