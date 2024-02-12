<script lang="ts">
	import {
		get_installed_metadata,
		install_mrpack,
		show_profile_dir_selector
	} from '$lib/installer';
	import { get_project, list_versions, type Version } from '$lib/modrinth';
  	import { trans } from '$lib/i18n';
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
					installProgress = trans('progress.clean_old');
					break;
				case 'load_pack':
					installProgress = trans('progress.load_pack');
					currentStep = 1;
					break;
				case 'download_files':
					installProgress = trans('progress.download_files');
					totalMods = payload[2] as number;
					break;
				case 'download_file':
					installProgress = trans('progress.download_file', {
						file: payload[3],
						idx: (payload[2] as number) + 1,
						total: totalMods
					});
					currentStep = (payload[2] as number) + 2;
					break;
				case 'extract_overrides':
					installProgress = trans('progress.extract_overrides');
					currentStep = totalMods + 2;
					break;
				case 'install_loader':
					installProgress = trans('progress.install_loader');
					currentStep = totalMods + 3;
					break;
				case 'add_profile':
					installProgress = trans('progress.add_profile');
					currentStep = totalMods + 4;
					break;
			}
		}
	});
	function confirmUnload(ev: BeforeUnloadEvent) {
		ev.preventDefault();
		return (ev.returnValue = trans("ui.confirm-exit"));
	}
	async function installPack() {
		addEventListener('beforeunload', confirmUnload);
		const unlisten = await appWindow.onCloseRequested(async (ev) => {
			const confirmed = await confirm(
				trans("ui.confirm-exit")
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
		}
	}
	let versions: Version[] | undefined = undefined;
	let selected: string;
	let isolateProfile = false;
	list_versions(PROJECT_ID).then((result) => {
		const featured_versions = result.filter((e) => e.featured);
		const release_versions = featured_versions.filter((e) => e.version_type == 'release');
		versions = featured_versions;
		selected = release_versions[0].id;
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
						<option>{trans('ui.loading-versions')}</option>
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
					title={trans('ui.version-tooltip')}
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
					>{@html trans('ui.isolate-profile')}</label
				>
			</div>
			{#if isolateProfile}
				<div class="flex flex-row gap-2 items-center justify-center">
					<input
						type="text"
						bind:value={profileDirectory}
						id="profile-directory"
						placeholder={trans('ui.profile-dir-placeholder')}
						class="input-box"
					/>
					<button class="fill-text" aria-label={trans('ui.profile-dir-browse-label')} on:click={browseProfileDirectory}>
						{@html FolderIcon}
					</button>
				</div>
			{/if}
			<button
				class="rounded-full bg-blue text-base disabled:bg-surface0 py-2 px-4 focus:outline-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue disabled:text-overlay0"
				on:click={installPack}
				disabled={versions == undefined}>{trans('ui.install-button')}</button
			>
		{:else if state == 'installing'}
			<div class="text-center text-lg">{trans('ui.installing')}</div>
			<progress class="progress" value={currentStep / totalSteps} />
			<div class="text-ellipsis whitespace-nowrap overflow-hidden">
				{installProgress}
			</div>
		{:else if state == 'postInstall'}
			<div class="text-center text-lg">{trans('ui.installed')}</div>
		{:else if state == 'error'}
			<div class="text-center text-lg text-red">
				{@html trans('ui.install-error', { errorMessage })}
			</div>
		{:else}
			<div>
				{@html trans('ui.downgrade-msg')}
			</div>
			<div class="flex flex-row gap-2 items-center justify-center">
				<input
					class="checkbox"
					type="checkbox"
					bind:checked={confirmDowngrade}
					id="confirm-downgrade"
				/>
				<label for="confirm-downgrade">{trans('ui.confirm-downgrade')}</label>
			</div>
			<button
				class="rounded-full bg-blue text-base disabled:bg-surface0 py-2 px-4 focus:outline-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue disabled:text-overlay0"
				on:click={() => (state = 'preInstall')}>{trans('ui.downgrade-cancel')}</button
			>
			<button
				class="rounded-full bg-red text-base disabled:bg-surface0 py-2 px-4 focus:outline-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue disabled:text-overlay0"
				on:click={installPack}
				disabled={!confirmDowngrade}>{trans('ui.downgrade-continue')}</button
			>
		{/if}
	</div>
</div>
