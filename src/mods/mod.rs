#![allow(dead_code)]
#![allow(non_snake_case)]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use chrono::Utc;
use futures_util::StreamExt;
use log::warn;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::util::{cancel_requested, format_speed};

const CURSE_FORGE_BASE: &str = "https://api.curseforge.com/v1";
const HYTALE_GAME_ID: u32 = 70216;
// Public key used by hrs-launcher for browsing CurseForge.
const CF_API_KEY: &str = "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModManifest {
    pub mods: Vec<InstalledMod>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledMod {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub download_url: String,
    pub curseforge_id: i32,
    pub file_id: i32,
    pub enabled: bool,
    pub installed_at: String,
    pub updated_at: String,
    pub file_path: String,
    pub icon_url: Option<String>,
    pub downloads: i64,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CurseForgeResponse<T> {
    pub data: T,
    #[allow(dead_code)]
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pagination {
    pub index: u32,
    pub pageSize: u32,
    pub resultCount: u32,
    pub totalCount: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CurseForgeMod {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub summary: String,
    #[serde(default)]
    pub downloadCount: i64,
    #[serde(default)]
    pub dateModified: String,
    #[serde(default)]
    pub logo: Option<ModLogo>,
    #[serde(default)]
    pub categories: Vec<ModCategory>,
    #[serde(default)]
    pub authors: Vec<ModAuthor>,
    #[serde(default)]
    pub latestFiles: Vec<ModFile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModLogo {
    #[serde(default)]
    pub thumbnailUrl: String,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModCategory {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModAuthor {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModFile {
    pub id: i32,
    #[serde(default)]
    pub displayName: String,
    #[serde(default)]
    pub fileName: String,
    #[serde(default)]
    pub fileLength: u64,
    #[serde(default)]
    pub downloadUrl: String,
    #[serde(default)]
    pub fileDate: String,
}

#[derive(Clone)]
pub struct ModService {
    client: Client,
    mods_dir: PathBuf,
}

impl ModService {
    pub fn new(mods_dir: PathBuf) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|err| {
                warn!(
                    "mods: failed to build HTTP client ({}); using default configuration",
                    err
                );
                Client::new()
            });
        Self { client, mods_dir }
    }

    pub async fn search(
        &self,
        query: &str,
        page: u32,
    ) -> Result<CurseForgeResponse<Vec<CurseForgeMod>>, String> {
        let url = format!(
            "{CURSE_FORGE_BASE}/mods/search?gameId={HYTALE_GAME_ID}&searchFilter={query}&pageSize=20&index={}",
            page * 20
        );
        let resp = self
            .client
            .get(&url)
            .header("x-api-key", CF_API_KEY)
            .send()
            .await
            .map_err(|e| format!("mod search failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("mod search status error: {e}"))?;
        resp.json::<CurseForgeResponse<Vec<CurseForgeMod>>>()
            .await
            .map_err(|e| format!("mod search parse error: {e}"))
    }

    pub async fn mod_details(&self, mod_id: i32) -> Result<CurseForgeMod, String> {
        let url = format!("{CURSE_FORGE_BASE}/mods/{mod_id}");
        let resp = self
            .client
            .get(&url)
            .header("x-api-key", CF_API_KEY)
            .send()
            .await
            .map_err(|e| format!("mod details failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("mod details status error: {e}"))?;
        let wrapped: CurseForgeResponse<CurseForgeMod> = resp
            .json()
            .await
            .map_err(|e| format!("mod details parse error: {e}"))?;
        Ok(wrapped.data)
    }

    pub async fn mod_files(&self, mod_id: i32) -> Result<Vec<ModFile>, String> {
        let url = format!("{CURSE_FORGE_BASE}/mods/{mod_id}/files");
        let resp = self
            .client
            .get(&url)
            .header("x-api-key", CF_API_KEY)
            .send()
            .await
            .map_err(|e| format!("mod files failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("mod files status error: {e}"))?;
        let wrapped: CurseForgeResponse<Vec<ModFile>> = resp
            .json()
            .await
            .map_err(|e| format!("mod files parse error: {e}"))?;
        Ok(wrapped.data)
    }

    /// Download the latest available file for the given mod and record it in the manifest.
    pub async fn download_latest<F>(
        &self,
        mod_id: i32,
        cancel: Option<Arc<AtomicBool>>,
        mut progress: F,
    ) -> Result<InstalledMod, String>
    where
        F: FnMut(f32, &str),
    {
        if cancel_requested(&cancel) {
            return Err("Download cancelled".into());
        }
        let details = self.mod_details(mod_id).await?;
        let latest = pick_latest_file(&details).ok_or("no downloadable files for this mod")?;
        if latest.downloadUrl.is_empty() {
            return Err("mod author disabled downloads".into());
        }

        fs::create_dir_all(&self.mods_dir)
            .await
            .map_err(|e| format!("unable to create mods dir: {e}"))?;
        let dest = self.mods_dir.join(&latest.fileName);

        progress(0.0, &format!("Downloading {}...", details.name));
        self.download_file(
            &latest.downloadUrl,
            &dest,
            latest.fileLength,
            cancel.clone(),
            |d, t, speed| {
                let pct = match t {
                    Some(total) if total > 0 => (d as f32 / total as f32) * 100.0,
                    _ => 0.0,
                };
                progress(pct, &format!("Downloading {}... {}", details.name, speed));
            },
        )
        .await?;

        let author = details
            .authors
            .first()
            .map(|a| a.name.clone())
            .unwrap_or_else(|| "Unknown".into());
        let category = details.categories.first().map(|c| c.name.clone());
        let icon = details.logo.as_ref().map(|l| {
            if !l.thumbnailUrl.is_empty() {
                l.thumbnailUrl.clone()
            } else {
                l.url.clone()
            }
        });
        let timestamp = Utc::now().to_rfc3339();

        let installed = InstalledMod {
            id: format!("cf-{}", details.id),
            name: details.name.clone(),
            slug: details.slug.clone(),
            version: latest.displayName.clone(),
            author,
            description: details.summary.clone(),
            download_url: latest.downloadUrl.clone(),
            curseforge_id: details.id,
            file_id: latest.id,
            enabled: true,
            installed_at: timestamp.clone(),
            updated_at: timestamp,
            file_path: dest.display().to_string(),
            icon_url: icon,
            downloads: details.downloadCount,
            category,
        };

        self.upsert_manifest_entry(installed.clone()).await?;
        progress(100.0, &format!("Installed {} successfully", details.name));

        Ok(installed)
    }

    pub async fn installed_mods(&self) -> Result<Vec<InstalledMod>, String> {
        let manifest = self.load_manifest().await?;
        Ok(manifest.mods)
    }

    pub async fn remove_installed(&self, mod_id: &str) -> Result<(), String> {
        let mut manifest = self.load_manifest().await?;
        if let Some(entry) = manifest.mods.iter().find(|m| m.id == mod_id) {
            let path = PathBuf::from(&entry.file_path);
            if path.exists() {
                fs::remove_file(&path)
                    .await
                    .map_err(|e| format!("failed to delete mod file: {e}"))?;
            }
        }
        let initial_len = manifest.mods.len();
        manifest.mods.retain(|m| m.id != mod_id);
        if manifest.mods.len() == initial_len {
            return Err("mod not found in manifest".into());
        }
        self.save_manifest(&manifest).await
    }

    async fn upsert_manifest_entry(&self, mod_entry: InstalledMod) -> Result<(), String> {
        let mut manifest = self.load_manifest().await?;
        if let Some(existing) = manifest.mods.iter_mut().find(|m| m.id == mod_entry.id) {
            *existing = mod_entry;
        } else {
            manifest.mods.push(mod_entry);
        }
        manifest.version = "1.0".into();
        self.save_manifest(&manifest).await
    }

    async fn load_manifest(&self) -> Result<ModManifest, String> {
        let path = self.mods_dir.join("manifest.json");
        let bytes = match fs::read(&path).await {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Ok(ModManifest::default());
            }
            Err(err) => return Err(format!("failed to read mod manifest: {err}")),
        };
        serde_json::from_slice(&bytes).map_err(|e| format!("failed to parse mod manifest: {e}"))
    }

    async fn save_manifest(&self, manifest: &ModManifest) -> Result<(), String> {
        let path = self.mods_dir.join("manifest.json");
        let bytes = serde_json::to_vec_pretty(manifest)
            .map_err(|e| format!("failed to serialize manifest: {e}"))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("failed to create manifest dir: {e}"))?;
        }
        fs::write(&path, &bytes)
            .await
            .map_err(|e| format!("failed to write manifest: {e}"))
    }

    async fn download_file<F>(
        &self,
        url: &str,
        dest: &Path,
        expected_size: u64,
        cancel: Option<Arc<AtomicBool>>,
        mut progress: F,
    ) -> Result<(), String>
    where
        F: FnMut(u64, Option<u64>, String),
    {
        if cancel_requested(&cancel) {
            return Err("Download cancelled".into());
        }
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("mod download failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("mod download status error: {e}"))?;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("failed to create mod dir: {e}"))?;
        }
        let mut file = fs::File::create(dest)
            .await
            .map_err(|e| format!("failed to create mod file: {e}"))?;
        let total = resp.content_length().or(Some(expected_size));
        let mut stream = resp.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut last_tick = Instant::now();
        let mut last_bytes = 0u64;

        while let Some(chunk) = stream.next().await {
            if cancel_requested(&cancel) {
                let _ = fs::remove_file(dest).await;
                return Err("Download cancelled".into());
            }
            let chunk = chunk.map_err(|e| format!("mod stream error: {e}"))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("mod write error: {e}"))?;
            downloaded += chunk.len() as u64;

            if last_tick.elapsed().as_secs_f32() > 0.2 {
                let speed = (downloaded - last_bytes) as f32 / last_tick.elapsed().as_secs_f32();
                let speed_text = format_speed(speed);
                progress(downloaded, total, speed_text);
                last_tick = Instant::now();
                last_bytes = downloaded;
            }
        }

        progress(downloaded, total, "0 B/s".into());
        file.flush()
            .await
            .map_err(|e| format!("mod flush error: {e}"))?;
        if let Some(total) = total
            && downloaded < total
        {
            return Err(format!(
                "mod download incomplete: received {} of {} bytes",
                downloaded, total
            ));
        }
        Ok(())
    }
}

fn pick_latest_file(details: &CurseForgeMod) -> Option<ModFile> {
    details
        .latestFiles
        .iter()
        .max_by_key(|f| &f.fileDate)
        .cloned()
}
