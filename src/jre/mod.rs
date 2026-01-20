use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::env;
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use log::{debug, info, warn};
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tar::Archive;
use tokio::fs as async_fs;
use tokio::io::AsyncWriteExt;
use zip::read::ZipArchive;

const JRE_CONFIG_URL: &str =
    "https://raw.githubusercontent.com/RustedBytes/hrs-launcher/main/jre.json";
const LOCAL_JRE_CONFIG: &str = "jre.json";
const JRE_VERSION: &str = "25";
const EMBEDDED_JRE_CONFIG: &str = include_str!("../../jre.json");
const CANCELLED: &str = "Download cancelled";

#[derive(Debug, Clone, Deserialize)]
struct JrePlatform {
    url: String,
    #[serde(default)]
    sha256: String,
}

#[derive(Debug, Clone, Deserialize)]
struct JreConfig {
    #[serde(rename = "download_url")]
    download_url: HashMap<String, HashMap<String, JrePlatform>>,
}

#[derive(Clone, Copy, Debug)]
enum ArchiveKind {
    TarGz,
    Zip,
}

#[derive(Debug)]
pub struct JreManager {
    cache_dir: PathBuf,
    jre_dir: PathBuf,
    client: Client,
}

impl JreManager {
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        let base = base_dir.as_ref();
        let cache_dir = base.join("cache");
        let jre_dir = base.join("jre");
        Self {
            cache_dir,
            jre_dir,
            client: Client::new(),
        }
    }

    pub fn default() -> Self {
        Self::new(env::default_app_dir())
    }

    pub async fn ensure_jre(&self, cancel_flag: Option<&AtomicBool>) -> Result<PathBuf, String> {
        info!("jre: ensuring runtime");
        check_cancel(cancel_flag)?;
        let java_path = self.java_path();
        if java_path.exists() {
            debug!("jre: runtime already present at {}", java_path.display());
            return Ok(java_path);
        }
        if self.jre_dir.exists() {
            self.normalize_layout()?;
            if java_path.exists() {
                debug!("jre: runtime found after layout normalization");
                return Ok(java_path);
            }
        }

        check_cancel(cancel_flag)?;
        fs::create_dir_all(&self.jre_dir).map_err(|e| format!("unable to create JRE dir: {e}"))?;
        fs::create_dir_all(&self.cache_dir)
            .map_err(|e| format!("unable to create cache dir: {e}"))?;

        let config = self
            .fetch_remote_config()
            .await
            .or_else(|_| self.load_local_config())?;
        check_cancel(cancel_flag)?;
        let target = self
            .pick_platform_target(&config)
            .unwrap_or_else(|| self.adoptium_fallback());
        info!("jre: selected target {}", target.url);

        let archive_path = self
            .cache_dir
            .join(format!("jre{}", target.archive.extension()));
        let expected_checksum = target
            .checksum
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned);
        let mut needs_download = !archive_path.exists();
        if !needs_download
            && let Some(expected) = expected_checksum.as_deref()
            && self.verify_sha256(&archive_path, expected).is_err()
        {
            let _ = fs::remove_file(&archive_path);
            needs_download = true;
        }
        if needs_download {
            info!("jre: downloading archive to {}", archive_path.display());
            self.download(&target.url, &archive_path, cancel_flag)
                .await
                .map_err(|e| {
                    if e == CANCELLED {
                        e
                    } else {
                        format!("failed to download JRE: {e}")
                    }
                })?;
        }
        check_cancel(cancel_flag)?;
        if let Some(expected) = expected_checksum.as_deref() {
            self.verify_sha256(&archive_path, expected)?;
        }

        check_cancel(cancel_flag)?;
        self.extract_archive(&archive_path, target.archive)?;
        check_cancel(cancel_flag)?;
        self.normalize_layout()?;

        info!("jre: ready at {}", java_path.display());
        Ok(java_path)
    }

    fn java_path(&self) -> PathBuf {
        let bin = if cfg!(target_os = "windows") {
            Path::new("bin").join("java.exe")
        } else {
            Path::new("bin").join("java")
        };
        self.jre_dir.join(bin)
    }

    async fn fetch_remote_config(&self) -> Result<JreConfig, String> {
        let resp = self
            .client
            .get(JRE_CONFIG_URL)
            .send()
            .await
            .map_err(|e| format!("config request failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("config request bad status: {e}"))?;
        let text = resp
            .text()
            .await
            .map_err(|e| format!("config body error: {e}"))?;
        serde_json::from_str(&text).map_err(|e| format!("config parse error: {e}"))
    }

    fn load_local_config(&self) -> Result<JreConfig, String> {
        warn!("jre: falling back to bundled config {}", LOCAL_JRE_CONFIG);
        let contents = match fs::read_to_string(LOCAL_JRE_CONFIG) {
            Ok(contents) => contents,
            Err(err) => {
                warn!(
                    "jre: failed to read local {}, using embedded copy: {err}",
                    LOCAL_JRE_CONFIG
                );
                EMBEDDED_JRE_CONFIG.to_string()
            }
        };
        serde_json::from_str(&contents).map_err(|e| format!("jre.json parse error: {e}"))
    }

    fn pick_platform_target(&self, config: &JreConfig) -> Option<DownloadTarget> {
        let (os_key, arch_key, default_archive) = platform_keys();
        let arch_map = config.download_url.get(os_key)?;
        let platform = arch_map.get(arch_key)?;
        Some(DownloadTarget {
            url: platform.url.clone(),
            checksum: if platform.sha256.trim().is_empty() {
                None
            } else {
                Some(platform.sha256.clone())
            },
            archive: guess_archive_kind(&platform.url).unwrap_or(default_archive),
        })
    }

    fn adoptium_fallback(&self) -> DownloadTarget {
        let (os_key, arch_key, archive) = adoptium_platform();
        let url = format!(
            "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jre/hotspot/normal/eclipse?project=jdk",
            JRE_VERSION, os_key, arch_key
        );
        warn!("jre: using adoptium fallback for {} {}", os_key, arch_key);
        DownloadTarget {
            url,
            checksum: None,
            archive,
        }
    }

    async fn download(
        &self,
        url: &str,
        dest: &Path,
        cancel_flag: Option<&AtomicBool>,
    ) -> Result<(), String> {
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("download request failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("download status error: {e}"))?;
        if let Some(parent) = dest.parent() {
            async_fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("failed to create download dir: {e}"))?;
        }

        let mut file = async_fs::File::create(dest)
            .await
            .map_err(|e| format!("failed to create archive file: {e}"))?;
        let mut stream = resp.bytes_stream();
        while let Some(chunk_res) = stream.next().await {
            if is_cancelled(cancel_flag) {
                let _ = async_fs::remove_file(dest).await;
                return Err(CANCELLED.into());
            }
            let chunk = chunk_res.map_err(|e| format!("download read error: {e}"))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("failed to write archive: {e}"))?;
        }
        if is_cancelled(cancel_flag) {
            let _ = async_fs::remove_file(dest).await;
            return Err(CANCELLED.into());
        }
        file.flush()
            .await
            .map_err(|e| format!("failed to flush archive: {e}"))?;
        Ok(())
    }

    fn verify_sha256(&self, path: &Path, expected: &str) -> Result<(), String> {
        let mut file = fs::File::open(path).map_err(|e| format!("checksum open error: {e}"))?;
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 8192];
        loop {
            let read = file
                .read(&mut buf)
                .map_err(|e| format!("checksum read error: {e}"))?;
            if read == 0 {
                break;
            }
            hasher.update(&buf[..read]);
        }
        let actual = format!("{:x}", hasher.finalize());
        if actual != expected.to_lowercase() {
            return Err(format!(
                "checksum mismatch: expected {expected}, got {actual}"
            ));
        }
        Ok(())
    }

    fn extract_archive(&self, archive_path: &Path, kind: ArchiveKind) -> Result<(), String> {
        info!("jre: extracting {} as {:?}", archive_path.display(), kind);
        match kind {
            ArchiveKind::TarGz => self.extract_targz(archive_path),
            ArchiveKind::Zip => self.extract_zip(archive_path),
        }
    }

    fn extract_targz(&self, archive_path: &Path) -> Result<(), String> {
        let file = fs::File::open(archive_path).map_err(|e| format!("tar.gz open error: {e}"))?;
        let dec = GzDecoder::new(file);
        let mut archive = Archive::new(dec);
        archive
            .unpack(&self.jre_dir)
            .map_err(|e| format!("tar.gz extract error: {e}"))
    }

    fn extract_zip(&self, archive_path: &Path) -> Result<(), String> {
        let file = fs::File::open(archive_path).map_err(|e| format!("zip open error: {e}"))?;
        let mut archive = ZipArchive::new(file).map_err(|e| format!("zip parse error: {e}"))?;
        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| format!("zip entry error: {e}"))?;
            let out_path = self.jre_dir.join(entry.mangled_name());
            if entry.name().ends_with('/') {
                fs::create_dir_all(&out_path).map_err(|e| format!("zip dir create error: {e}"))?;
                continue;
            }
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("zip parent dir error: {e}"))?;
            }
            let mut out_file =
                fs::File::create(&out_path).map_err(|e| format!("zip create file error: {e}"))?;
            io::copy(&mut entry, &mut out_file).map_err(|e| format!("zip write error: {e}"))?;
        }
        Ok(())
    }

    fn normalize_layout(&self) -> Result<(), String> {
        debug!("jre: normalizing layout in {}", self.jre_dir.display());
        let mut entries =
            fs::read_dir(&self.jre_dir).map_err(|e| format!("read jre dir error: {e}"))?;
        let first = match entries.next() {
            Some(Ok(entry)) => entry,
            _ => return Ok(()),
        };
        if entries.next().is_some() {
            return Ok(()); // already flat enough
        }

        if !first.file_type().map_err(|e| e.to_string())?.is_dir() {
            return Ok(());
        }

        #[cfg_attr(not(target_os = "macos"), allow(unused_mut))]
        let mut subdir = self.jre_dir.join(first.file_name());
        #[cfg(target_os = "macos")]
        {
            let mac_home = subdir.join("Contents").join("Home");
            if mac_home.exists() {
                subdir = mac_home;
            }
        }

        let sub_entries = fs::read_dir(&subdir).map_err(|e| format!("read subdir error: {e}"))?;
        for entry in sub_entries {
            let entry = entry.map_err(|e| format!("subdir entry error: {e}"))?;
            let from = entry.path();
            let to = self.jre_dir.join(entry.file_name());
            match fs::rename(&from, &to) {
                Ok(_) => {}
                Err(_) => {
                    // Fallback to copy if rename crosses devices.
                    match entry.file_type() {
                        Ok(ft) if ft.is_dir() => copy_dir(&from, &to)?,
                        _ => {
                            fs::copy(&from, &to).map_err(|e| format!("copy file error: {e}"))?;
                        }
                    }
                    // Best-effort cleanup old path if rename failed.
                    let _ = fs::remove_file(&from);
                }
            }
        }

        let _ = fs::remove_dir_all(subdir);
        Ok(())
    }
}

fn check_cancel(cancel_flag: Option<&AtomicBool>) -> Result<(), String> {
    if is_cancelled(cancel_flag) {
        warn!("jre: cancellation requested");
        return Err(CANCELLED.into());
    }
    Ok(())
}

fn is_cancelled(cancel_flag: Option<&AtomicBool>) -> bool {
    cancel_flag
        .map(|flag| flag.load(Ordering::SeqCst))
        .unwrap_or(false)
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| format!("copy dir create error: {e}"))?;
    for entry in fs::read_dir(from).map_err(|e| format!("copy dir read error: {e}"))? {
        let entry = entry.map_err(|e| format!("copy dir entry error: {e}"))?;
        let src_path = entry.path();
        let dst_path = to.join(entry.file_name());
        if entry
            .file_type()
            .map_err(|e| format!("copy filetype error: {e}"))?
            .is_dir()
        {
            copy_dir(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).map_err(|e| format!("copy file error: {e}"))?;
        }
    }
    Ok(())
}

fn platform_keys() -> (&'static str, &'static str, ArchiveKind) {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        std::env::consts::ARCH
    };

    let archive = if cfg!(target_os = "windows") {
        ArchiveKind::Zip
    } else {
        ArchiveKind::TarGz
    };

    (os, arch, archive)
}

fn adoptium_platform() -> (&'static str, &'static str, ArchiveKind) {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else {
        "linux"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        std::env::consts::ARCH
    };

    let archive = if cfg!(target_os = "windows") {
        ArchiveKind::Zip
    } else {
        ArchiveKind::TarGz
    };

    (os, arch, archive)
}

fn guess_archive_kind(url: &str) -> Option<ArchiveKind> {
    if url.ends_with(".zip") {
        Some(ArchiveKind::Zip)
    } else if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
        Some(ArchiveKind::TarGz)
    } else {
        None
    }
}

struct DownloadTarget {
    url: String,
    checksum: Option<String>,
    archive: ArchiveKind,
}

impl ArchiveKind {
    fn extension(self) -> &'static str {
        match self {
            ArchiveKind::TarGz => ".tar.gz",
            ArchiveKind::Zip => ".zip",
        }
    }
}
