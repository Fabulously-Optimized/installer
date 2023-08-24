<script lang="ts">
	import {
		get_installed_metadata,
		install_mrpack,
		show_profile_dir_selector
	} from '$lib/installer';
	import { get_project, list_versions, type Version } from '$lib/modrinth';
	import { listen } from '@tauri-apps/api/event';
	import { appWindow } from '@tauri-apps/api/window';
	import { confirm } from '@tauri-apps/api/dialog';
	import { open } from '@tauri-apps/api/shell';
	import semver from 'semver';
	import LightThemeIcon from '@fluentui/svg-icons/icons/weather_sunny_24_regular.svg?raw';
	import DarkThemeIcon from '@fluentui/svg-icons/icons/dark_theme_24_regular.svg?raw';
	import DeviceThemeIcon from '@fluentui/svg-icons/icons/laptop_24_regular.svg?raw';
	import HelpIcon from '@fluentui/svg-icons/icons/question_circle_32_regular.svg?raw';
	import FolderIcon from '@fluentui/svg-icons/icons/folder_32_regular.svg?raw';
	const PROJECT_ID = '1KVo5zza';
	let totalMods = Infinity;
	listen('install:progress', (event) => {
		console.log(event.payload);
		const payload = event.payload as (string | number)[];
		if (payload[1] == 'start') {
			switch (payload[0]) {
				case 'clean_old':
					installProgress = 'Cleaning up old files';
					break;
				case 'load_pack':
					installProgress = 'Downloading modpack';
					currentStep = 1;
					break;
				case 'download_files':
					installProgress = 'Downloading mods';
					totalMods = payload[2] as number;
					break;
				case 'download_file':
					installProgress = `Downloading ${payload[3]} (${
						(payload[2] as number) + 1
					}/${totalMods})`;
					currentStep = (payload[2] as number) + 2;
					break;
				case 'extract_overrides':
					installProgress = 'Extracting configuration files';
					currentStep = totalMods + 2;
					break;
				case 'install_loader':
					installProgress = 'Installing mod loader';
					currentStep = totalMods + 3;
					break;
				case 'add_profile':
					installProgress = 'Creating launcher profile';
					currentStep = totalMods + 4;
					break;
			}
		}
	});
	function confirmUnload(ev: BeforeUnloadEvent) {
		ev.preventDefault();
		return (ev.returnValue = 'Fabulously Optimized is installing. Are you sure you want to exit?');
	}
	async function installPack() {
		addEventListener('beforeunload', confirmUnload);
		const unlisten = await appWindow.onCloseRequested(async (ev) => {
			const confirmed = await confirm(
				'Fabulously Optimized is installing. Are you sure you want to exit?'
			);
			if (!confirmed) {
				// user did not confirm closing the window; let's prevent it
				ev.preventDefault();
			}
		});
		try {
			const version = versions!.find((e) => e.id == selected)!;
			const url = version!.files.find((e) => e.primary)!.url;
			const mc_version = version?.game_versions[0];
			const profile_dir =
				profileDirectory != ''
					? profileDirectory
					: isolateProfile
					? `fabulously-optimized-${mc_version}`
					: undefined;
			if (state != 'confirmDowngrade') {
				const installed_metadata = await get_installed_metadata(profile_dir);
				if (typeof installed_metadata == 'object' && installed_metadata != null) {
					if (
						'mc_version' in installed_metadata &&
						typeof installed_metadata.mc_version == 'string'
					) {
						const installed_mc_version = installed_metadata.mc_version;
						if (semver.lt(mc_version, installed_mc_version)) {
							state = 'confirmDowngrade';
							confirmDowngrade = false;
							removeEventListener('beforeunload', confirmUnload);
							unlisten();
							// TODO: this is a workaround for tauri-apps/tauri#7119
							await appWindow.onCloseRequested(() => {});
							return;
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
				isolateProfile ? `Fabulously Optimized ${mc_version}` : 'Fabulously Optimized',
				profile_dir,
				{
					fo_version: {
						id: version.id,
						version_number: version.version_number
					},
					mc_version: mc_version
				}
			);
			state = 'postInstall';
		} catch (e) {
			state = 'error';
			errorMessage = String(e);
			console.error(e);
		} finally {
			removeEventListener('beforeunload', confirmUnload);
			unlisten();
			// TODO: this is a workaround for tauri-apps/tauri#7119
			await appWindow.onCloseRequested(() => {});
		}
	}
	let versions: Version[] | undefined = undefined;
	let selected: string;
	let isolateProfile = false;
	list_versions(PROJECT_ID).then((result) => {
		const featured_versions = result.filter((e) => e.featured);
		const release_versions = featured_versions.filter((e) => e.version_type == 'release');
		const beta_versions = featured_versions.filter((e) => e.version_type == 'beta');
		const alpha_versions = featured_versions.filter((e) => e.version_type == 'alpha');
		versions = release_versions.concat(beta_versions, alpha_versions);
		selected = versions[0].id;
	});
	let state: 'preInstall' | 'installing' | 'postInstall' | 'error' | 'confirmDowngrade' =
		'preInstall';
	let installProgress = '';
	$: totalSteps = totalMods + 4;
	let currentStep = 0;
	let errorMessage: string | undefined = undefined;
	let confirmDowngrade = false;

	const theme_icon_map = {
		'device-theme': DeviceThemeIcon,
		'light-theme': LightThemeIcon,
		'dark-theme': DarkThemeIcon
	};
	const themes: ('device-theme' | 'light-theme' | 'dark-theme')[] = [
		'device-theme',
		'light-theme',
		'dark-theme'
	];

	let theme: 'device-theme' | 'light-theme' | 'dark-theme' = 'device-theme';
	$: document.body.className = theme;

	function cycle_theme() {
		const idx = themes.indexOf(theme);
		theme = themes[(idx + 1) % 3];
	}

	function openHelp() {
		open('https://fabulously-optimized.gitbook.io/modpack/readme/version-support');
	}

	async function browseProfileDirectory() {
		const result = await show_profile_dir_selector();
		if (result != null) profileDirectory = result;
	}

	let profileDirectory = '';
</script>

<button
	class="absolute top-0 right-0 bg-surface0 rounded-lg m-4 p-2 fill-text shadow-lg"
	on:click={cycle_theme}
>
	{@html theme_icon_map[theme]}
</button>

<div class="flex items-center justify-center w-full h-full bg-base text-text">
	<div class="flex flex-col gap-4 max-w-md">
		{#if state == 'preInstall'}
			<div class="flex flex-row gap-2 items-center justify-center">
				<select class="input-box" bind:value={selected} disabled={versions == undefined}>
					{#if versions == undefined}
						<option>Loading versions...</option>
					{:else}
						{#each versions as version}
							<option value={version.id}>{version.name}</option>
						{/each}
					{/if}
				</select>
				<a
					href="#top"
					on:click={openHelp}
					on:keypress={openHelp}
					tabindex="0"
					class="fill-text"
					title="Vanilla Installer allows easy installation of all supported versions of Fabulously Optimized. For outdated versions, use a different launcher."
				>
					{@html HelpIcon}
				</a>
			</div>
			<div class="flex flex-row gap-2 items-center justify-center">
				<input
					type="checkbox"
					bind:checked={isolateProfile}
					id="isolate-profile"
					class="checkbox"
				/>
				<label for="isolate-profile"
					>Use a subfolder inside the <code class="inline-code">.minecraft</code> folder</label
				>
			</div>
			{#if isolateProfile}
				<div class="flex flex-row gap-2 items-center justify-center">
					<input
						type="text"
						bind:value={profileDirectory}
						id="profile-directory"
						placeholder="(default .minecraft folder path)"
						class="input-box"
					/>
					<button class="fill-text" aria-label="Browse folders" on:click={browseProfileDirectory}>
						{@html FolderIcon}
					</button>
				</div>
			{/if}
			<button
				class="rounded-full bg-blue text-base disabled:bg-surface0 py-2 px-4 focus:outline-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue disabled:text-overlay0"
				on:click={installPack}
				disabled={versions == undefined}>Install!</button
			>
		{:else if state == 'installing'}
			<div class="text-center text-lg">Installing...</div>
			<progress class="progress" value={currentStep / totalSteps} />
			<div class="text-ellipsis whitespace-nowrap overflow-hidden">
				{installProgress}
			</div>
		{:else if state == 'postInstall'}
			<div class="text-center text-lg">Fabulously Optimized is installed!</div>
		{:else if state == 'error'}
			<div class="text-center text-lg text-red">
				An error occurred while installing Fabulously Optimized: {errorMessage}
			</div>
		{:else}
			<div>
				You are attempting to downgrade the Minecraft version. This is <span
					class="inline font-semibold">NOT SUPPORTED</span
				>
				by Mojang or Fabulously Optimized and it may cause world corruption or crashes. <br />
				If you want to do this safely, you should backup <code class="inline-code">mods</code>,
				<code class="inline-code">config</code>
				and <code class="inline-code">saves</code> folders to a different location and delete them
				from your .minecraft folder.<br />
				To skip this warning after backing up the folders, delete
				<code class="inline-code">paigaldaja_meta.json</code> from your .minecraft folder.
			</div>
			<div class="flex flex-row gap-2 items-center justify-center">
				<input
					class="checkbox"
					type="checkbox"
					bind:checked={confirmDowngrade}
					id="confirm-downgrade"
				/>
				<label for="confirm-downgrade">Yes, I want to downgrade FO.</label>
			</div>
			<button
				class="rounded-full bg-blue text-base disabled:bg-surface0 py-2 px-4 focus:outline-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue disabled:text-overlay0"
				on:click={() => (state = 'preInstall')}>Back</button
			>
			<button
				class="rounded-full bg-red text-base disabled:bg-surface0 py-2 px-4 focus:outline-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue disabled:text-overlay0"
				on:click={installPack}
				disabled={!confirmDowngrade}>Continue</button
			>
		{/if}
	</div>
</div>
