use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use futures_util::future::join_all;
use log::{debug, info, warn};
use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::env;
use crate::util::{cancel_requested, format_speed, progress_percent};

pub mod butler;

const PATCH_HOST: &str = "https://game-patches.hytale.com";

#[derive(Clone, Debug, Default)]
pub struct VersionCheckResult {
    pub latest_version: u32,
    pub available_versions: Vec<u32>,
    pub success_url: Option<String>,
    pub checked_urls: Vec<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ProgressUpdate {
    pub stage: &'static str,
    pub progress: f32,
    pub message: String,
    pub current_file: Option<String>,
    pub speed: Option<String>,
    #[allow(dead_code)]
    pub downloaded: Option<u64>,
    #[allow(dead_code)]
    pub total: Option<u64>,
}

pub type ProgressCallback<'a> = Option<&'a mut (dyn FnMut(ProgressUpdate) + Send)>;

fn emit_progress(cb: &mut ProgressCallback<'_>, update: ProgressUpdate) {
    if let Some(callback) = cb.as_deref_mut() {
        callback(update);
    }
}

pub async fn find_latest_version_with_details(version_type: &str) -> VersionCheckResult {
    let (os, arch) = platform_keys();
    if os == "unknown" {
        warn!("version probe: unsupported operating system");
        return VersionCheckResult {
            error: Some("unsupported operating system".into()),
            ..Default::default()
        };
    }

    let api_version_type = normalize_version_type(version_type);
    let start_version = if api_version_type == "pre-release" {
        30
    } else {
        20
    };

    let client = match Client::builder().timeout(Duration::from_secs(10)).build() {
        Ok(client) => client,
        Err(err) => {
            let message = format!("failed to build HTTP client: {err}");
            warn!("version probe: {message}");
            return VersionCheckResult {
                error: Some(message),
                ..Default::default()
            };
        }
    };

    let mut checks = Vec::new();
    for version in 1..=start_version {
        let url = format!(
            "{PATCH_HOST}/patches/{}/{}/{}/0/{}.pwr",
            os, arch, api_version_type, version
        );
        let c = client.clone();
        checks.push(async move {
            match c.head(&url).send().await {
                Ok(resp) => (version, url, resp.status().is_success(), None),
                Err(err) => (version, url, false, Some(err.to_string())),
            }
        });
    }

    let mut result = VersionCheckResult::default();
    let mut had_request_errors = false;
    for (version, url, exists, request_error) in join_all(checks).await {
        result.checked_urls.push(url.clone());
        if let Some(err) = request_error {
            had_request_errors = true;
            warn!("version probe failed for {}: {}", url, err);
        }
        if exists && version > result.latest_version {
            result.latest_version = version;
            result.success_url = Some(url);
        }
        if exists {
            result.available_versions.push(version);
        }
    }
    debug!(
        "version probe: latest={} success_url={:?}",
        result.latest_version, result.success_url
    );

    if !result.available_versions.is_empty() {
        result.available_versions.sort_unstable();
        result.available_versions.dedup();
        result.available_versions.sort_unstable_by(|a, b| b.cmp(a));
    }

    if result.latest_version == 0 && result.error.is_none() {
        result.error = Some(if had_request_errors {
            "unable to reach update server".into()
        } else {
            "no game versions found for this platform".into()
        });
    }

    result
}

pub async fn download_pwr(
    version_type: &str,
    from_version: u32,
    to_version: u32,
    cancel: Option<Arc<AtomicBool>>,
    mut progress: ProgressCallback<'_>,
) -> Result<PathBuf, String> {
    if cancel_requested(&cancel) {
        warn!("download_pwr: cancelled before start");
        return Err("Download cancelled".into());
    }
    let (os, arch) = platform_keys();
    if os == "unknown" {
        return Err("unsupported operating system".into());
    }
    let api_version_type = normalize_version_type(version_type);

    let client = Client::builder()
        .timeout(Duration::from_secs(30 * 60))
        .build()
        .map_err(|e| format!("failed to build HTTP client: {e}"))?;

    // Prefer incremental patch when possible, otherwise fall back to full package.
    let url = format!(
        "{PATCH_HOST}/patches/{}/{}/{}/{}/{}.pwr",
        os, arch, api_version_type, from_version, to_version
    );

    let url = if from_version == 0 || !head_available(&client, &url).await? {
        format!(
            "{PATCH_HOST}/patches/{}/{}/{}/0/{}.pwr",
            os, arch, api_version_type, to_version
        )
    } else {
        url
    };

    let expected_size = content_length(&client, &url).await.unwrap_or(0);

    let cache_dir = env::cache_dir();
    fs::create_dir_all(&cache_dir).map_err(|e| format!("failed to create cache directory: {e}"))?;

    let dest = cache_dir.join(format!("{}.pwr", to_version));
    debug!(
        "download_pwr: target={} expected_size={:?}",
        dest.display(),
        expected_size
    );
    if let Ok(info) = fs::metadata(&dest) {
        if expected_size > 0 && info.len() == expected_size {
            info!("download_pwr: cache hit for version {}", to_version);
            return Ok(dest);
        }
        if expected_size == 0 && info.len() > 1_024 * 1_024 * 1_024 {
            info!(
                "download_pwr: cache hit (size heuristic) for version {}",
                to_version
            );
            return Ok(dest);
        }
        let _ = fs::remove_file(&dest);
    }

    if cancel_requested(&cancel) {
        warn!("download_pwr: cancelled before HTTP request");
        return Err("Download cancelled".into());
    }
    emit_progress(
        &mut progress,
        ProgressUpdate {
            stage: "download",
            progress: 0.0,
            message: "Downloading Hytale...".into(),
            current_file: dest.file_name().map(|n| n.to_string_lossy().into()),
            speed: None,
            downloaded: None,
            total: expected_size.checked_into(),
        },
    );

    let request = client
        .get(&url)
        .header(
            reqwest::header::USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
        .header(reqwest::header::ACCEPT, "*/*")
        .header(reqwest::header::ACCEPT_LANGUAGE, "en-US,en;q=0.9");

    let response = request
        .send()
        .await
        .map_err(|e| format!("failed to download patch: {e}"))?
        .error_for_status()
        .map_err(|e| format!("patch not available: {e}"))?;
    if cancel_requested(&cancel) {
        let _ = fs::remove_file(&dest);
        return Err("Download cancelled".into());
    }

    let total = response.content_length().or(Some(expected_size));
    let mut stream = response.bytes_stream();
    let mut file = File::create(&dest)
        .await
        .map_err(|e| format!("failed to create patch file: {e}"))?;

    let mut downloaded: u64 = 0;
    let mut last_tick = Instant::now();
    let mut last_bytes = 0u64;

    while let Some(chunk) = stream.next().await {
        if cancel_requested(&cancel) {
            let _ = fs::remove_file(&dest);
            return Err("Download cancelled".into());
        }
        let chunk = chunk.map_err(|e| format!("stream error: {e}"))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("write error: {e}"))?;
        downloaded += chunk.len() as u64;

        let elapsed = last_tick.elapsed().as_secs_f32();
        if elapsed > 0.2 {
            let speed = (downloaded - last_bytes) as f32 / elapsed;
            emit_progress(
                &mut progress,
                ProgressUpdate {
                    stage: "download",
                    progress: progress_percent(downloaded, total),
                    message: "Downloading game patch...".into(),
                    current_file: dest.file_name().map(|n| n.to_string_lossy().into()),
                    speed: Some(format_speed(speed)),
                    downloaded: Some(downloaded),
                    total,
                },
            );
            last_tick = Instant::now();
            last_bytes = downloaded;
            debug!(
                "download_pwr: downloaded {} bytes of {:?} ({:.1}%)",
                downloaded,
                total,
                progress_percent(downloaded, total)
            );
        }
    }

    emit_progress(
        &mut progress,
        ProgressUpdate {
            stage: "download",
            progress: 100.0,
            message: "Download complete".into(),
            current_file: dest.file_name().map(|n| n.to_string_lossy().into()),
            speed: Some("0 B/s".into()),
            downloaded: Some(downloaded),
            total,
        },
    );

    if let Some(total) = total
        && downloaded < total
    {
        let _ = fs::remove_file(&dest);
        return Err(format!(
            "download incomplete: got {} of {} bytes",
            downloaded, total
        ));
    }

    info!("download_pwr: completed {}", dest.display());
    Ok(dest)
}

pub async fn apply_pwr(pwr_file: &Path, mut progress: ProgressCallback<'_>) -> Result<(), String> {
    let game_dir = env::game_latest_dir();
    let staging_dir = game_dir.join("staging-temp");
    let client_path = game_client_path(&game_dir);

    if client_path.exists() {
        emit_progress(
            &mut progress,
            ProgressUpdate {
                stage: "install",
                progress: 100.0,
                message: "Game already installed".into(),
                current_file: None,
                speed: None,
                downloaded: None,
                total: None,
            },
        );
        return Ok(());
    }

    let butler_path = butler::install_butler(None).await?;

    emit_progress(
        &mut progress,
        ProgressUpdate {
            stage: "install",
            progress: 0.0,
            message: "Preparing installation...".into(),
            current_file: None,
            speed: None,
            downloaded: None,
            total: None,
        },
    );

    clean_staging_directory(&game_dir)?;
    fs::create_dir_all(&game_dir).map_err(|e| format!("failed to create game directory: {e}"))?;
    fs::create_dir_all(&staging_dir)
        .map_err(|e| format!("failed to create staging directory: {e}"))?;

    emit_progress(
        &mut progress,
        ProgressUpdate {
            stage: "install",
            progress: 5.0,
            message: "Applying game patch...".into(),
            current_file: None,
            speed: None,
            downloaded: None,
            total: None,
        },
    );

    let mut cmd = std::process::Command::new(&butler_path);
    cmd.arg("apply").arg("--staging-dir").arg(&staging_dir);
    if cfg!(target_os = "windows") {
        cmd.arg("--save-interval=60");
    }
    cmd.arg(pwr_file).arg(&game_dir);
    info!(
        "apply_pwr: running butler for {} into {}",
        pwr_file.display(),
        game_dir.display()
    );

    let output = cmd
        .output()
        .map_err(|e| format!("failed to run butler: {e}"))?;
    if !output.status.success() {
        clean_staging_directory(&game_dir).ok();
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "butler apply failed: {}",
            if stderr.trim().is_empty() {
                stdout.trim().to_owned()
            } else {
                stderr.trim().to_owned()
            }
        ));
    }

    clean_staging_directory(&game_dir)?;
    let _ = fs::remove_file(pwr_file);

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&client_path, fs::Permissions::from_mode(0o755));
    }

    emit_progress(
        &mut progress,
        ProgressUpdate {
            stage: "install",
            progress: 100.0,
            message: "Hytale installed successfully".into(),
            current_file: None,
            speed: None,
            downloaded: None,
            total: None,
        },
    );

    info!("apply_pwr: install completed");
    Ok(())
}

#[allow(dead_code)]
pub fn get_local_version() -> Option<u32> {
    let version_file = env::default_app_dir().join("version.txt");
    let data = fs::read_to_string(version_file).ok()?;
    data.trim().parse::<u32>().ok()
}

pub fn save_local_version(version: u32) -> Result<(), String> {
    env::ensure_base_dirs().map_err(|e| format!("failed to prepare directories: {e}"))?;
    let version_file = env::default_app_dir().join("version.txt");
    fs::write(&version_file, version.to_string())
        .map_err(|e| format!("failed to save version: {e}"))
}

async fn head_available(client: &Client, url: &str) -> Result<bool, String> {
    let exists = match client.head(url).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    };
    Ok(exists)
}

async fn content_length(client: &Client, url: &str) -> Option<u64> {
    match client.head(url).send().await {
        Ok(resp) if resp.status().is_success() => resp.content_length(),
        _ => None,
    }
}

fn clean_staging_directory(game_dir: &Path) -> Result<(), String> {
    let staging = game_dir.join("staging-temp");
    if staging.exists() {
        fs::remove_dir_all(&staging)
            .or_else(|_| {
                #[cfg(target_os = "windows")]
                {
                    for entry in walkdir::WalkDir::new(&staging).into_iter().flatten() {
                        if entry.file_type().is_file() {
                            let _ = fs::remove_file(entry.path());
                        }
                    }
                }
                fs::remove_dir_all(&staging)
            })
            .map_err(|e| format!("failed to clean staging dir: {e}"))?;
    }

    if game_dir.exists() {
        for entry in fs::read_dir(game_dir).map_err(|e| format!("failed to read game dir: {e}"))? {
            let entry = entry.map_err(|e| format!("failed to read dir entry: {e}"))?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.ends_with(".tmp") || name.starts_with("sf-") {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    Ok(())
}

fn platform_keys() -> (&'static str, &'static str) {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "amd64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        std::env::consts::ARCH
    };

    (os, arch)
}

fn normalize_version_type(value: &str) -> String {
    if value.eq_ignore_ascii_case("prerelease") || value.eq_ignore_ascii_case("pre-release") {
        "pre-release".into()
    } else {
        value.to_owned()
    }
}

fn game_client_path(game_dir: &Path) -> PathBuf {
    if cfg!(target_os = "windows") {
        game_dir.join("Client").join("HytaleClient.exe")
    } else if cfg!(target_os = "macos") {
        game_dir
            .join("Client")
            .join("Hytale.app")
            .join("Contents")
            .join("MacOS")
            .join("HytaleClient")
    } else {
        game_dir.join("Client").join("HytaleClient")
    }
}

trait CheckedInto<T> {
    fn checked_into(self) -> Option<T>;
}

impl CheckedInto<u64> for u64 {
    fn checked_into(self) -> Option<u64> {
        Some(self)
    }
}
