#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{
    io::{Cursor, Read},
    path::PathBuf,
};

use anyhow::anyhow;
use mrpack::PackDependency;
use sha2::Digest;
use tauri::{
    api::http::{ClientBuilder, HttpRequestBuilder, ResponseType},
    http::status::StatusCode,
    Manager,
};
use zip::ZipArchive;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            install_mrpack,
            get_installed_metadata
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

mod mrpack;

async fn get_launcher_path() -> PathBuf {
    if let Ok(path) = std::env::var("PAIGALDAJA_LAUNCHER_PATH") {
        return PathBuf::from(path);
    }
    #[cfg(target_os = "windows")]
    {
        let mut path = PathBuf::from(std::env::var("APPDATA").unwrap());
        path.push(".minecraft");
        path
    }
    #[cfg(target_os = "macos")]
    {
        // Rationale: home_dir is deprecated because of surprising behaviour in Windows
        // This code path only gets compiled on macOS, which is not Windows
        #[allow(deprecated)]
        let mut path = std::env::home_dir().unwrap();
        path.push("Library");
        path.push("Application Support");
        path.push("minecraft");
        path
    }
    #[cfg(target_os = "linux")]
    {
        // Rationale: home_dir is deprecated because of surprising behaviour in Windows
        // This code path only gets compiled on Linux, which is not Windows
        #[allow(deprecated)]
        let path = std::env::home_dir().unwrap();
        // check for flatpak
        if tokio::fs::try_exists(path.join(".var/app/com.mojang.Minecraft/.minecraft")).await.unwrap_or(false) {
            path.join(".var/app/com.mojang.Minecraft/.minecraft")
        } else {
            path.join(".minecraft")
        }
    }
}

async fn install_fabriclike(
    client: &tauri::api::http::Client,
    profile_url: String,
    profile_name: &str,
) -> anyhow::Result<()> {
    let profile_json = client
        .send(HttpRequestBuilder::new("GET", profile_url)?.response_type(ResponseType::Text))
        .await?;
    if profile_json.status() != StatusCode::OK {
        return Err(anyhow!("Metadata server did not respond with 200"));
    }
    let versions_dir = get_launcher_path().await.join("versions");
    let profile_dir = versions_dir.join(&profile_name);
    let profile_json_path = profile_dir.join(format!("{}.json", &profile_name));
    let profile_jar_path = profile_dir.join(format!("{}.jar", &profile_name));
    if !profile_dir.is_dir() {
        tokio::fs::create_dir(&profile_dir).await?;
    }
    tokio::fs::write(
        &profile_json_path,
        profile_json.read().await?.data.as_str().unwrap(),
    )
    .await?;
    if profile_jar_path.is_file() {
        tokio::fs::remove_file(&profile_jar_path).await?;
    }
    // yes, actually
    // we create an empty file
    tokio::fs::write(&profile_jar_path, []).await?;
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
    let meta_path = match profile_dir {
        Some(path) => get_launcher_path().await.join(PathBuf::from(path)),
        None => get_launcher_path().await,
    }
    .join("paigaldaja_meta.json");
    let meta: serde_json::Value =
        serde_json::from_str(&tokio::fs::read_to_string(meta_path).await.ok()?).ok()?;
    if let serde_json::Value::Object(mut map) = meta {
        map.remove("metadata")
    } else {
        None
    }
}

async fn get_installed_files(profile_dir: Option<String>) -> Option<Vec<String>> {
    let meta_path = match profile_dir {
        Some(path) => get_launcher_path().await.join(PathBuf::from(path)),
        None => get_launcher_path().await,
    }
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
    .map_err(|e| e.to_string())
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
    let profile_base_path = match profile_dir.clone() {
        Some(path) => get_launcher_path().await.join(PathBuf::from(path)),
        None => get_launcher_path().await,
    };
    let _ = app_handle.emit_all("install:progress", ("clean_old", "start"));
    if let Some(files) = get_installed_files(profile_dir.clone()).await {
        for file in files {
            // ignore Result as cleanup failing shouldn't abort install
            let _ = tokio::fs::remove_file(profile_base_path.join(PathBuf::from(file))).await;
        }
    }
    let _ = app_handle.emit_all("install:progress", ("clean_old", "complete"));
    let _ = app_handle.emit_all("install:progress", ("load_pack", "start"));
    let mut written_files = vec![];
    let client = ClientBuilder::new().build().unwrap();
    let request = HttpRequestBuilder::new("GET", url)?.response_type(ResponseType::Binary);
    let response = client.send(request).await?;
    if response.status() != StatusCode::OK {
        return Err(anyhow!("Server did not respond with 200"));
    }
    let bytes = response.bytes().await?.data;
    let mut mrpack = zip::ZipArchive::new(Cursor::new(bytes))?;
    let index: mrpack::PackIndex = serde_json::from_reader(mrpack.by_name("modrinth.index.json")?)?;
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
        let mut success = false;
        for url in file.downloads {
            if let Ok(resp) = client
                .send(HttpRequestBuilder::new("GET", url)?.response_type(ResponseType::Binary))
                .await
            {
                if resp.status() == StatusCode::OK {
                    if let Ok(blob) = resp.bytes().await {
                        if std::convert::Into::<[u8; 64]>::into(sha2::Sha512::digest(&blob.data))
                            .as_ref()
                            == hash
                        {
                            if let Some(parent) = PathBuf::from(&file.path).parent() {
                                tokio::fs::create_dir_all(profile_base_path.join(parent)).await?;
                            }
                            tokio::fs::write(
                                profile_base_path.join(PathBuf::from(&file.path)),
                                blob.data,
                            )
                            .await?;
                            written_files.push(PathBuf::from(&file.path));
                            success = true;
                            break;
                        }
                    }
                }
            }
        }
        if !success {
            return Err(anyhow!("Download failed for {}", file.path));
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
        // This is overly complex and only used once
        // But is required to work around rust-lang/rust#63768
        fn complex_helper_function<T>(
            archive: &mut ZipArchive<T>,
            filename: &str,
        ) -> anyhow::Result<Option<(PathBuf, Vec<u8>)>>
        where
            T: std::io::Read,
            T: std::io::Seek,
        {
            if filename.starts_with("overrides")
                && archive.by_name(&("client-".to_string() + filename)).is_ok()
            {
                return Ok(None);
            }
            let mut file = archive.by_name(filename)?;
            if file.is_file() {
                if let Ok(path) = file.mangled_name().strip_prefix("overrides") {
                    let mut buf: Vec<u8> = vec![];
                    file.read_to_end(&mut buf)?;
                    return Ok(Some((path.to_owned(), buf)));
                } else if let Ok(path) = file.mangled_name().strip_prefix("client-overrides") {
                    let mut buf: Vec<u8> = vec![];
                    file.read_to_end(&mut buf)?;
                    return Ok(Some((path.to_owned(), buf)));
                }
            }
            Ok(None)
        }
        if let Some((rel_path, buf)) = complex_helper_function(&mut mrpack, &filename)? {
            let path = profile_base_path.join(&rel_path);
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(&path, buf).await?;
            written_files.push(rel_path);
        }
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
        install_fabriclike(&client, profile_url, &version_name).await?;
    } else if let Some(quilt_version) = index.dependencies.get(&PackDependency::QuiltLoader) {
        let profile_url = format!(
            "https://meta.quiltmc.org/v3/versions/loader/{}/{}/profile/json",
            mc_version, quilt_version
        );
        version_name = format!("quilt-loader-{}-{}", quilt_version, mc_version);
        install_fabriclike(&client, profile_url, &version_name).await?;
    }
    let _ = app_handle.emit_all("install:progress", ("install_loader", "complete"));
    let _ = app_handle.emit_all("install:progress", ("add_profile", "start"));
    let profiles_path = get_launcher_path().await.join("launcher_profiles.json");
    let mut profiles: serde_json::Value =
        serde_json::from_str(&tokio::fs::read_to_string(&profiles_path).await?)?;
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
    tokio::fs::write(profiles_path, serde_json::to_string(&profiles)?).await?;
    tokio::fs::write(
        profile_base_path.join("paigaldaja_meta.json"),
        serde_json::to_string(&serde_json::json!({
            "files": written_files,
            "metadata": extra_metadata
        }))?,
    )
    .await?;
    let _ = app_handle.emit_all("install:progress", ("add_profile", "complete"));
    Ok(())
}
