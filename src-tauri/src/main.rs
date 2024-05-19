#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{
    io::{Cursor, Read},
    mem::ManuallyDrop,
    path::{Component, Path, PathBuf},
};

use anyhow::{anyhow, Context};
use mrpack::PackDependency;
use reqwest::StatusCode;
use sha2::Digest;
use tauri::Manager;
use tokio::{fs::File, io::AsyncWriteExt};

mod config;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            install_mrpack,
            get_installed_metadata,
            show_profile_dir_selector,
            is_launcher_installed
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

struct SelfCleanupFile<'a>(ManuallyDrop<File>, &'a Path);

impl<'a> SelfCleanupFile<'a> {
    async fn new(path: &'a Path) -> tokio::io::Result<Self> {
        Ok(Self(ManuallyDrop::new(File::create(path).await?), path))
    }

    fn finalize(mut self) {
        unsafe { ManuallyDrop::drop(&mut self.0) };
        std::mem::forget(self);
    }
}

impl Drop for SelfCleanupFile<'_> {
    fn drop(&mut self) {
        unsafe { ManuallyDrop::drop(&mut self.0) };
        _ = std::fs::remove_file(self.1);
    }
}

mod mrpack;

#[tauri::command]
async fn show_profile_dir_selector() -> Option<PathBuf> {
    let (send, recv) = tokio::sync::oneshot::channel();
    let mut builder = tauri::api::dialog::FileDialogBuilder::new();
    if let Ok(path) = get_launcher_path().await {
        builder = builder.set_directory(path)
    }
    builder.pick_folder(|folder| {
        let _ = send.send(folder);
    });
    recv.await.ok().flatten()
}

#[tauri::command]
async fn is_launcher_installed() -> bool {
    if let Ok(path) = get_launcher_path()
        .await
        .map(|path| path.join("launcher_profiles.json"))
    {
        tokio::fs::try_exists(path).await.unwrap_or(false)
    } else {
        false
    }
}

async fn get_launcher_path() -> anyhow::Result<PathBuf> {
    if let Ok(path) = std::env::var("PAIGALDAJA_LAUNCHER_PATH") {
        return Ok(PathBuf::from(path));
    }
    #[cfg(target_os = "windows")]
    {
        let path = tauri::api::path::data_dir()
            .ok_or(anyhow!("Could not determine APPDATA directory!"))?;
        Ok(path.join(".minecraft"))
    }
    #[cfg(target_os = "macos")]
    {
        let path = tauri::api::path::local_data_dir()
            .ok_or(anyhow!("Could not determine local data directory!"))?;
        Ok(path.join("minecraft"))
    }
    #[cfg(target_os = "linux")]
    {
        let path =
            tauri::api::path::home_dir().ok_or(anyhow!("Could not determine home directory!"))?;
        // check for flatpak
        Ok(
            if tokio::fs::try_exists(path.join(".var/app/com.mojang.Minecraft/.minecraft"))
                .await
                .unwrap_or(false)
            {
                path.join(".var/app/com.mojang.Minecraft/.minecraft")
            } else {
                path.join(".minecraft")
            },
        )
    }
}

async fn install_fabriclike(
    app_handle: &tauri::AppHandle,
    client: &reqwest::Client,
    profile_url: String,
    profile_name: &str,
) -> anyhow::Result<()> {
    let mut profile_json = client
        .get(profile_url)
        .header(
            "User-Agent",
            format!(
                "Paigaldaja/{} (+https://github.com/Fabulously-Optimized/vanilla-installer-rust)",
                app_handle.package_info().version
            ),
        )
        .send()
        .await?;
    if profile_json.status() != StatusCode::OK {
        return Err(anyhow!("Metadata server did not respond with 200"));
    }
    let versions_dir = get_launcher_path().await?.join("versions");
    let profile_dir = versions_dir.join(profile_name);
    let profile_json_path = profile_dir.join(format!("{}.json", &profile_name));
    let profile_jar_path = profile_dir.join(format!("{}.jar", &profile_name));
    if !profile_dir.is_dir() {
        tokio::fs::create_dir_all(&profile_dir).await?;
    }
    let mut profile_json_file = SelfCleanupFile::new(&profile_json_path).await?;
    while let Some(chunk) = profile_json.chunk().await? {
        profile_json_file.0.write_all(&chunk).await?;
    }
    profile_json_file.finalize();
    if profile_jar_path.is_file() {
        tokio::fs::remove_file(&profile_jar_path).await?;
    }
    Ok(())
}

fn set_or_create_profile(
    json: &mut serde_json::Value,
    profile_id: &str,
    profile_name: &str,
    profile_icon: Option<&str>,
    profile_version: &str,
    profile_dir: Option<&str>,
) -> Option<()> {
    let profiles: &mut serde_json::Map<String, serde_json::Value> =
        json.as_object_mut()?.get_mut("profiles")?.as_object_mut()?;
    let now = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Iso8601::DEFAULT)
        .ok()?;
    let mut profile = serde_json::json!({
        "name": profile_name,
        "type": "custom",
        "created": now,
        "lastUsed": now,
        "lastVersionId": profile_version,
    });
    if let Some(profile_dir) = profile_dir {
        profile
            .as_object_mut()
            .unwrap()
            .insert("gameDir".to_string(), profile_dir.into());
    }
    if let Some(profile_icon) = profile_icon {
        profile
            .as_object_mut()
            .unwrap()
            .insert("icon".to_string(), profile_icon.into());
    }
    profiles.insert(profile_id.to_string(), profile);
    Some(())
}

#[tauri::command]
async fn get_installed_metadata(profile_dir: Option<String>) -> Option<serde_json::Value> {
    let meta_path = canonicalize_profile_path(&profile_dir)
        .await
        .ok()?
        .join("paigaldaja_meta.json");
    let meta: serde_json::Value =
        serde_json::from_str(&tokio::fs::read_to_string(meta_path).await.ok()?).ok()?;
    if let serde_json::Value::Object(mut map) = meta {
        map.remove("metadata")
    } else {
        None
    }
}

async fn get_installed_files(profile_dir: &Option<String>) -> Option<Vec<String>> {
    let meta_path = canonicalize_profile_path(profile_dir)
        .await
        .ok()?
        .join("paigaldaja_meta.json");
    let meta: serde_json::Value =
        serde_json::from_str(&tokio::fs::read_to_string(meta_path).await.ok()?).ok()?;
    if let serde_json::Value::Object(mut map) = meta {
        serde_json::from_value(map.remove("files")?).ok()
    } else {
        None
    }
}

#[tauri::command]
async fn install_mrpack(
    app_handle: tauri::AppHandle,
    url: String,
    pack_id: String,
    icon: Option<String>,
    pack_name: String,
    profile_dir: Option<String>,
    extra_metadata: serde_json::Value,
) -> Result<(), String> {
    install_mrpack_inner(
        app_handle,
        url,
        pack_id,
        icon,
        pack_name,
        profile_dir,
        extra_metadata,
    )
    .await
    .map_err(|e| format!("{e:#}"))
}

async fn canonicalize_profile_path(profile_dir: &Option<String>) -> anyhow::Result<PathBuf> {
    Ok(if let Some(path) = profile_dir {
        let mut path = PathBuf::from(path);
        if !path.is_absolute() {
            path = get_launcher_path().await?.join(path);
        }
        path
    } else {
        get_launcher_path().await?
    })
}

async fn try_download(
    app_handle: &tauri::AppHandle,
    client: &reqwest::Client,
    url: &str,
    path: &Path,
    expected_hash: &[u8],
    expected_size: usize,
) -> anyhow::Result<()> {
    let mut url = reqwest::Url::parse(url).context("Invalid download URL!")?;
    let host = url
        .host_str()
        .ok_or(anyhow!("Download URL doesn't have a host?"))?;
    if !config::DOWNLOADS_DOMAIN_WHITELIST.contains(&host) {
        return Err(anyhow!("Domain not allowed for download: {}", host));
    }
    if url.scheme() == "http" {
        url.set_scheme("https")
            .map_err(|()| anyhow!("Failed to upgrade HTTP to HTTPS!"))?;
    }
    if url.scheme() != "https" {
        return Err(anyhow!(
            "Weird scheme, possibly malicious: {}",
            url.scheme()
        ));
    }
    let mut resp = client
        .get(url)
        .header(
            "User-Agent",
            format!(
                "Paigaldaja/{} (+https://github.com/Fabulously-Optimized/vanilla-installer-rust)",
                app_handle.package_info().version
            ),
        )
        .send()
        .await?;
    if resp.status() != StatusCode::OK {
        return Err(anyhow!("Status code was not 200, but {}", resp.status()));
    }
    let mut size = 0usize;
    let mut hasher = sha2::Sha512::new();
    let mut file = SelfCleanupFile::new(path).await?;
    while let Some(chunk) = resp.chunk().await? {
        size += chunk.len();
        if size > expected_size {
            return Err(anyhow!(
                "File is bigger than expected: expected {} bytes, aborting on {} bytes",
                expected_size,
                size
            ));
        }
        hasher.update(&chunk);
        file.0.write_all(&chunk).await?;
    }
    if size < expected_size {
        return Err(anyhow!(
            "File is smaller than expected: expected {} bytes, got {} bytes",
            expected_size,
            size
        ));
    }
    let hash: [u8; 64] = hasher.finalize().into();
    if hash != expected_hash {
        return Err(anyhow!(
            "Wrong hash: got {}, expected {}",
            hex::encode(hash),
            hex::encode(expected_hash)
        ));
    }
    file.finalize();
    Ok(())
}

fn parse_and_sanitize_path(path: &str) -> Option<&Path> {
    if path.contains('\0') {
        return None;
    }
    let path = Path::new(path);
    let mut depth = 0usize;
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => return None,
            Component::ParentDir => depth = depth.checked_sub(1)?,
            Component::Normal(_) => depth += 1,
            Component::CurDir => (),
        }
    }
    Some(path)
}

async fn install_mrpack_inner(
    app_handle: tauri::AppHandle,
    url: String,
    pack_id: String,
    icon: Option<String>,
    pack_name: String,
    profile_dir: Option<String>,
    extra_metadata: serde_json::Value,
) -> anyhow::Result<()> {
    let profile_base_path = canonicalize_profile_path(&profile_dir)
        .await
        .context("Could not determine profile directory")?;
    let _ = app_handle.emit_all("install:progress", ("clean_old", "start"));
    if let Some(files) = get_installed_files(&profile_dir).await {
        for file in files {
            // ignore Result as cleanup failing shouldn't abort install
            let _ = tokio::fs::remove_file(profile_base_path.join(PathBuf::from(file))).await;
        }
    }
    let _ = app_handle.emit_all("install:progress", ("clean_old", "complete"));
    let _ = app_handle.emit_all("install:progress", ("load_pack", "start"));
    let mut written_files = vec![];
    let mut url = reqwest::Url::parse(&url).context("Invalid modpack URL!")?;
    let host = url
        .host_str()
        .ok_or(anyhow!("Modpack URL doesn't have a host?"))?;
    if !config::PACK_DOMAIN_WHITELIST.contains(&host) {
        return Err(anyhow!("Domain not allowed for pack download: {}", host));
    }
    if url.scheme() == "http" {
        url.set_scheme("https")
            .map_err(|()| anyhow!("Failed to upgrade HTTP to HTTPS!"))?;
    }
    if url.scheme() != "https" {
        return Err(anyhow!(
            "Weird scheme, possibly malicious: {}",
            url.scheme()
        ));
    }
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(
            "User-Agent",
            format!(
                "Paigaldaja/{} (+https://github.com/Fabulously-Optimized/vanilla-installer-rust)",
                app_handle.package_info().version
            ),
        )
        .send()
        .await
        .context("Failed to fetch modpack data")?;
    if response.status() != StatusCode::OK {
        return Err(anyhow!("Server did not respond with 200"));
    }
    let bytes = response
        .bytes()
        .await
        .context("Failed to fetch modpack data")?;
    let mut mrpack =
        zip::ZipArchive::new(Cursor::new(bytes)).context("Failed to parse modpack file")?;
    let index: mrpack::PackIndex = serde_json::from_reader(
        mrpack
            .by_name("modrinth.index.json")
            .context("No modrinth.index.json in mrpack?")?,
    )
    .context("modrinth.index.json is invalid")?;
    if index.format_version != 1 {
        return Err(anyhow!("Unknown format version {}", index.format_version));
    }
    if index.game != "minecraft" {
        return Err(anyhow!("Unknown game {}", index.game));
    }
    let _ = app_handle.emit_all("install:progress", ("load_pack", "complete"));
    let _ = app_handle.emit_all(
        "install:progress",
        ("download_files", "start", index.files.len()),
    );
    for (i, file) in index.files.into_iter().enumerate() {
        let _ = app_handle.emit_all(
            "install:progress",
            ("download_file", "start", i, &file.path),
        );
        let path = parse_and_sanitize_path(&file.path)
            .ok_or(anyhow!("Possibly malicious download path: {}", file.path))?;
        let path = profile_base_path.join(path);
        if let Some(env) = file.env {
            if let Some(&mrpack::SideType::Unsupported) = env.get(&mrpack::EnvType::Client) {
                continue;
            }
        }
        let hash = hex::decode(
            file.hashes
                .get(&mrpack::PackFileHash::Sha512)
                .ok_or(anyhow!(
                    "No SHA512 hash for file {}; This violates spec!",
                    file.path
                ))?,
        )?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(profile_base_path.join(parent)).await?;
        }
        let mut success = false;
        let mut last_err = None;
        for url in file.downloads {
            match try_download(
                &app_handle,
                &client,
                &url,
                &path,
                &hash,
                file.file_size as usize,
            )
            .await
            {
                Ok(()) => {
                    written_files.push(path.to_owned());
                    success = true;
                    break;
                }
                Err(e) => {
                    last_err.replace(e);
                }
            }
        }
        if !success {
            return Err(anyhow!(
                "Download failed for {}: {}",
                file.path,
                last_err.unwrap()
            ));
        }
        let _ = app_handle.emit_all(
            "install:progress",
            ("download_file", "complete", i, &file.path),
        );
    }
    let _ = app_handle.emit_all("install:progress", ("download_files", "complete"));
    let _ = app_handle.emit_all("install:progress", ("extract_overrides", "start"));

    for filename in mrpack
        .file_names()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
    {
        if filename.starts_with("overrides")
            && mrpack.by_name(&format!("client-{filename}")).is_ok()
        {
            continue;
        }
        let mut buf: Vec<u8>;
        let path: PathBuf;
        {
            let mut file = mrpack
                .by_name(&filename)
                .context("Failed to read configuration file; corrupted mrpack?")?;
            if file.is_dir() {
                continue;
            }
            let path_ref = file
                .enclosed_name()
                .ok_or(anyhow!("Possibly malicious config path: {}", file.name()))?;
            path = if let Ok(path) = path_ref
                .strip_prefix("overrides")
                .or_else(|_| path_ref.strip_prefix("client-overrides"))
                .map(Path::to_owned)
            {
                path
            } else {
                continue;
            };
            buf = vec![];
            file.read_to_end(&mut buf)
                .context("Failed to read configuration file; corrupted mrpack?")?;
        }
        let abs_path = profile_base_path.join(&path);
        if let Some(parent) = abs_path.parent() {
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!(
                    "Failed to create directories for configuration file at {}",
                    path.to_string_lossy()
                )
            })?;
        }
        tokio::fs::write(&abs_path, &buf).await.with_context(|| {
            format!(
                "Failed to write configuration file at {}",
                abs_path.to_string_lossy()
            )
        })?;
        written_files.push(path);
    }

    let _ = app_handle.emit_all("install:progress", ("extract_overrides", "complete"));
    let _ = app_handle.emit_all("install:progress", ("install_loader", "start"));
    if index.dependencies.contains_key(&PackDependency::Forge) {
        return Err(anyhow!("Forge is currently unsupported"));
    }
    let mc_version = match index.dependencies.get(&PackDependency::Minecraft) {
        Some(version) => version,
        None => return Err(anyhow!("Modpack does not specify Minecraft version")),
    };
    let mut version_name = mc_version.clone();
    if let Some(fabric_version) = index.dependencies.get(&PackDependency::FabricLoader) {
        let profile_url = format!(
            "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
            mc_version, fabric_version
        );
        version_name = format!("fabric-loader-{}-{}", fabric_version, mc_version);
        install_fabriclike(&app_handle, &client, profile_url, &version_name)
            .await
            .context("Failed to install Fabric")?;
    } else if let Some(quilt_version) = index.dependencies.get(&PackDependency::QuiltLoader) {
        let profile_url = format!(
            "https://meta.quiltmc.org/v3/versions/loader/{}/{}/profile/json",
            mc_version, quilt_version
        );
        version_name = format!("quilt-loader-{}-{}", quilt_version, mc_version);
        install_fabriclike(&app_handle, &client, profile_url, &version_name)
            .await
            .context("Failed to install Quilt")?;
    }
    let _ = app_handle.emit_all("install:progress", ("install_loader", "complete"));
    let _ = app_handle.emit_all("install:progress", ("add_profile", "start"));
    let profiles_path = get_launcher_path()
        .await
        .context("Could not determine profile directory")?
        .join("launcher_profiles.json");
    let mut profiles: serde_json::Value = serde_json::from_str(
        &tokio::fs::read_to_string(&profiles_path)
            .await
            .context("Failed to read launcher profiles")?,
    )
    .context("Failed to parse launcher profiles")?;
    let profile_base_path_string = profile_base_path.to_string_lossy();
    set_or_create_profile(
        &mut profiles,
        &pack_id,
        &pack_name,
        icon.as_deref(),
        &version_name,
        if profile_dir.is_some() {
            Some(&profile_base_path_string)
        } else {
            None
        },
    )
    .ok_or(anyhow!("Could not create launcher profile"))?;
    tokio::fs::write(profiles_path, serde_json::to_string(&profiles)?)
        .await
        .context("Failed to write launcher profiles")?;
    tokio::fs::write(
        profile_base_path.join("paigaldaja_meta.json"),
        serde_json::to_string(&serde_json::json!({
            "files": written_files,
            "metadata": extra_metadata
        }))?,
    )
    .await
    .context("Failed to write installer metadata")?;
    let _ = app_handle.emit_all("install:progress", ("add_profile", "complete"));
    Ok(())
}
