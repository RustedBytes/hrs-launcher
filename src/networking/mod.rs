use std::path::Path;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use log::warn;
use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::engine::models::{Manifest, ManifestFile};

#[allow(dead_code)]
const MAX_PROBE_VERSION: u32 = 12;
#[allow(dead_code)]
const PATCH_HOST: &str = "https://game-patches.hytale.com";

#[derive(Clone)]
pub struct NetworkClient {
    #[allow(dead_code)]
    client: Client,
}

impl NetworkClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|err| {
                warn!("network client: falling back to default HTTP client configuration ({err})");
                Client::new()
            });
        Self { client }
    }

    /// Find the latest available patch on the Hytale patch server and return a manifest for it.
    #[allow(dead_code)]
    pub async fn fetch_manifest(&self) -> Result<Manifest, String> {
        let (os, arch) = platform_keys();
        let branch = "release";

        let mut found = None;
        for v in (1..=MAX_PROBE_VERSION).rev() {
            let url = format!(
                "{PATCH_HOST}/patches/{}/{}/{}/0/{}.pwr",
                os, arch, branch, v
            );
            if let Some(len) = self.head_content_length(&url).await? {
                found = Some((v, url, len));
                break;
            }
        }

        let (version, url, size) = found.ok_or("no downloadable game versions found")?;

        Ok(Manifest {
            version: version.to_string(),
            files: vec![ManifestFile {
                name: format!("{branch}-{version}.pwr"),
                size_bytes: size,
                checksum: String::new(),
                download_url: url,
            }],
        })
    }

    #[allow(dead_code)]
    async fn head_content_length(&self, url: &str) -> Result<Option<u64>, String> {
        let resp = self
            .client
            .head(url)
            .send()
            .await
            .map_err(|e| format!("HEAD {url} failed: {e}"))?;
        if !resp.status().is_success() {
            return Ok(None);
        }
        Ok(resp.content_length())
    }

    /// Download a file to `dest`, calling `progress` with (downloaded, total, speed_text).
    #[allow(dead_code)]
    pub async fn download_to_path<F>(
        &self,
        url: &str,
        dest: &Path,
        expected_size: Option<u64>,
        mut progress: F,
    ) -> Result<(), String>
    where
        F: FnMut(u64, Option<u64>, &str),
    {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("download request failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("download status error: {e}"))?;

        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("failed to create download dir: {e}"))?;
        }
        let mut file = File::create(dest)
            .await
            .map_err(|e| format!("failed to create file: {e}"))?;

        let total = response.content_length().or(expected_size);
        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut last_tick = Instant::now();
        let mut last_bytes = 0u64;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("stream error: {e}"))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("write error: {e}"))?;
            downloaded += chunk.len() as u64;

            let since = last_tick.elapsed().as_secs_f32();
            if since > 0.2 {
                let speed = (downloaded - last_bytes) as f32 / since;
                let speed_text = format_speed(speed);
                progress(downloaded, total, &speed_text);
                last_tick = Instant::now();
                last_bytes = downloaded;
            }
        }

        // Final callback.
        progress(downloaded, total, "0 B/s");

        file.flush()
            .await
            .map_err(|e| format!("flush error: {e}"))?;

        if let Some(total) = total
            && downloaded < total
        {
            return Err(format!(
                "download incomplete: received {} of {} bytes",
                downloaded, total
            ));
        }

        Ok(())
    }
}

#[allow(dead_code)]
fn platform_keys() -> (&'static str, &'static str) {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
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

#[allow(dead_code)]
fn format_speed(bytes_per_sec: f32) -> String {
    if bytes_per_sec < 1024.0 {
        format!("{bytes_per_sec:.0} B/s")
    } else if bytes_per_sec < 1024.0 * 1024.0 {
        format!("{:.1} KB/s", bytes_per_sec / 1024.0)
    } else {
        format!("{:.1} MB/s", bytes_per_sec / 1024.0 / 1024.0)
    }
}
