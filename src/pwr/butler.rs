use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use log::warn;
use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use zip::read::ZipArchive;

use crate::env;
use crate::util::{format_speed, progress_percent};

use super::{ProgressCallback, ProgressUpdate};

const BROTH_URL: &str = "https://broth.itch.zone/butler/{os}-{arch}/LATEST/archive/default";

/// Ensure the Butler binary is available, downloading and extracting if needed.
pub async fn install_butler(mut progress: ProgressCallback<'_>) -> Result<PathBuf, String> {
    let dir = env::butler_dir();
    let path = butler_path(&dir);

    if path.exists() {
        emit_progress(
            &mut progress,
            ProgressUpdate {
                stage: "butler",
                progress: 100.0,
                message: "Butler ready".into(),
                current_file: None,
                speed: None,
                downloaded: None,
                total: None,
            },
        );
        return Ok(path);
    }

    fs::create_dir_all(&dir).map_err(|e| format!("failed to create butler directory: {e}"))?;

    let (os, arch) = butler_platform_keys();
    let url = BROTH_URL.replace("{os}", os).replace("{arch}", arch);
    let cache_path = env::cache_dir().join("butler.zip");

    // Retry once on a bad ZIP to recover from truncated downloads.
    for attempt in 0..2 {
        download_with_progress(&url, &cache_path, &mut progress).await?;
        match extract_zip(&cache_path, &dir) {
            Ok(_) => break,
            Err(err) if attempt == 0 => {
                warn!(
                    "install_butler: zip extract failed ({}); redownloading once",
                    err
                );
                let _ = fs::remove_file(&cache_path);
                let _ = fs::remove_dir_all(&dir);
                fs::create_dir_all(&dir)
                    .map_err(|e| format!("failed to recreate butler directory: {e}"))?;
            }
            Err(err) => return Err(err),
        }
    }

    let _ = fs::remove_file(&cache_path);

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o755));
    }

    emit_progress(
        &mut progress,
        ProgressUpdate {
            stage: "butler",
            progress: 100.0,
            message: "Butler installed".into(),
            current_file: None,
            speed: None,
            downloaded: None,
            total: None,
        },
    );

    Ok(path)
}

fn butler_path(dir: &Path) -> PathBuf {
    if cfg!(target_os = "windows") {
        dir.join("butler.exe")
    } else {
        dir.join("butler")
    }
}

fn butler_platform_keys() -> (&'static str, &'static str) {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    };

    // Butler does not publish darwin-arm64; force amd64 on macOS (Rosetta).
    let arch = if cfg!(target_os = "macos") || cfg!(target_arch = "x86_64") {
        "amd64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        std::env::consts::ARCH
    };

    (os, arch)
}

async fn download_with_progress(
    url: &str,
    dest: &Path,
    progress: &mut ProgressCallback<'_>,
) -> Result<(), String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10 * 60))
        .build()
        .map_err(|e| format!("failed to build HTTP client: {e}"))?;

    emit_progress(
        progress,
        ProgressUpdate {
            stage: "butler",
            progress: 0.0,
            message: "Downloading Butler...".into(),
            current_file: dest.file_name().map(|n| n.to_string_lossy().into()),
            speed: None,
            downloaded: None,
            total: None,
        },
    );

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed to create cache dir: {e}"))?;
    }

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("failed to download butler: {e}"))?
        .error_for_status()
        .map_err(|e| format!("butler download status error: {e}"))?;

    let total = response.content_length();
    let mut stream = response.bytes_stream();
    let mut file = File::create(dest)
        .await
        .map_err(|e| format!("failed to create cache file: {e}"))?;

    let mut downloaded: u64 = 0;
    let mut last_tick = Instant::now();
    let mut last_bytes = 0u64;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("stream error: {e}"))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("write error: {e}"))?;
        downloaded += chunk.len() as u64;

        let elapsed = last_tick.elapsed().as_secs_f32();
        if elapsed > 0.2 {
            let speed = (downloaded - last_bytes) as f32 / elapsed;
            emit_progress(
                progress,
                ProgressUpdate {
                    stage: "butler",
                    progress: progress_percent(downloaded, total),
                    message: "Downloading Butler...".into(),
                    current_file: dest.file_name().map(|n| n.to_string_lossy().into()),
                    speed: Some(format_speed(speed)),
                    downloaded: Some(downloaded),
                    total,
                },
            );
            last_tick = Instant::now();
            last_bytes = downloaded;
        }
    }

    Ok(())
}

fn extract_zip(archive_path: &Path, dest: &Path) -> Result<(), String> {
    let file = fs::File::open(archive_path).map_err(|e| format!("zip open error: {e}"))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("zip parse error: {e}"))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("zip entry error: {e}"))?;
        let out_path = dest.join(entry.mangled_name());
        if entry.name().ends_with('/') {
            fs::create_dir_all(&out_path).map_err(|e| format!("zip dir create error: {e}"))?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("zip parent dir error: {e}"))?;
        }
        let mut out_file =
            fs::File::create(&out_path).map_err(|e| format!("zip create file error: {e}"))?;
        std::io::copy(&mut entry, &mut out_file).map_err(|e| format!("zip write error: {e}"))?;
    }

    Ok(())
}

fn emit_progress(cb: &mut ProgressCallback<'_>, update: ProgressUpdate) {
    if let Some(callback) = cb.as_deref_mut() {
        callback(update);
    }
}
