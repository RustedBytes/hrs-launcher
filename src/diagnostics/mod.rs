#![allow(dead_code)]

use std::fs;
use std::net::ToSocketAddrs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use log::{debug, info, warn};
use reqwest::Client;
use reqwest::Url;
use serde::Serialize;
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::env as app_env;
use std::env::consts as os_consts;
use std::fmt::Write;

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticReport {
    pub platform: PlatformInfo,
    pub connectivity: ConnectivityInfo,
    pub game_status: GameStatusInfo,
    pub dependencies: DependenciesInfo,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub launcher_version: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ConnectivityInfo {
    pub hytale_patches: bool,
    pub github: bool,
    pub itch_io: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct GameStatusInfo {
    pub installed: bool,
    pub version: Option<String>,
    pub client_exists: bool,
    pub online_fix_applied: bool,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct DependenciesInfo {
    pub java_installed: bool,
    pub java_path: Option<String>,
    pub butler_installed: bool,
    pub butler_path: Option<String>,
}

pub struct Diagnostics {
    client: Client,
    launcher_version: String,
}

impl Diagnostics {
    pub fn new(launcher_version: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("HytaleLauncherDiagnostics/0.1")
            .build()
            .expect("reqwest client for diagnostics");
        Self {
            client,
            launcher_version: launcher_version.into(),
        }
    }

    pub async fn run(&self) -> DiagnosticReport {
        DiagnosticReport {
            platform: self.platform_info(),
            connectivity: self.check_connectivity().await,
            game_status: self.check_game_status(),
            dependencies: self.check_dependencies(),
            timestamp: format_timestamp(SystemTime::now()),
        }
    }

    pub fn save_report(&self, report: &DiagnosticReport) -> Result<PathBuf, String> {
        info!("diagnostics: saving report");
        let logs = app_env::logs_dir();
        fs::create_dir_all(&logs).map_err(|e| format!("unable to create logs dir: {e}"))?;

        let filename = format!(
            "diagnostic_{}.txt",
            report
                .timestamp
                .replace(':', "-")
                .replace(' ', "_")
                .replace('.', "-")
        );
        let path = logs.join(filename);
        fs::write(&path, format_report(report))
            .map_err(|e| format!("failed to write report: {e}"))?;
        info!("diagnostics: report written to {}", path.display());
        Ok(path)
    }

    fn platform_info(&self) -> PlatformInfo {
        debug!("diagnostics: collecting platform info");
        PlatformInfo {
            os: os_consts::OS.into(),
            arch: os_consts::ARCH.into(),
            launcher_version: self.launcher_version.clone(),
        }
    }

    async fn check_connectivity(&self) -> ConnectivityInfo {
        info!("diagnostics: checking connectivity");
        let mut info = ConnectivityInfo {
            hytale_patches: self.endpoint_ok("https://game-patches.hytale.com").await,
            github: self.endpoint_ok("https://api.github.com").await,
            itch_io: self.endpoint_ok("https://broth.itch.zone").await,
            ..Default::default()
        };

        // DNS probe
        if ("game-patches.hytale.com", 443)
            .to_socket_addrs()
            .is_ok_and(|mut iter| iter.next().is_some())
        {
            // ok
        } else {
            info.error = Some("DNS resolution failed for game-patches.hytale.com".into());
            warn!("diagnostics: DNS resolution failed for game-patches.hytale.com");
        }

        info
    }

    fn check_game_status(&self) -> GameStatusInfo {
        info!("diagnostics: checking game status");
        let mut status = GameStatusInfo::default();
        let game_dir = app_env::game_latest_dir();

        let client_path = if cfg!(target_os = "windows") {
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
        };

        status.client_exists = client_path.exists();
        status.installed = status.client_exists || game_dir.exists();

        let version_file = app_env::default_app_dir().join("version.txt");
        if let Ok(bytes) = fs::read(&version_file) {
            let version = String::from_utf8_lossy(&bytes).trim().to_owned();
            if !version.is_empty() {
                status.version = Some(version);
            }
        }

        status.online_fix_applied = if cfg!(target_os = "windows") {
            game_dir.join("Server").join("start-server.bat").exists()
        } else {
            true
        };

        debug!(
            "diagnostics: game installed={} version={:?} client_exists={}",
            status.installed, status.version, status.client_exists
        );
        status
    }

    fn check_dependencies(&self) -> DependenciesInfo {
        info!("diagnostics: checking dependencies");
        let mut deps = DependenciesInfo::default();

        let java_bin = if cfg!(target_os = "windows") {
            app_env::jre_dir().join("bin").join("java.exe")
        } else {
            app_env::jre_dir().join("bin").join("java")
        };
        if java_bin.exists() {
            deps.java_installed = true;
            deps.java_path = Some(java_bin.display().to_string());
        }

        let butler_bin = if cfg!(target_os = "windows") {
            app_env::butler_dir().join("butler.exe")
        } else {
            app_env::butler_dir().join("butler")
        };
        if butler_bin.exists() {
            deps.butler_installed = true;
            deps.butler_path = Some(butler_bin.display().to_string());
        }

        debug!(
            "diagnostics: java_installed={} butler_installed={}",
            deps.java_installed, deps.butler_installed
        );
        deps
    }

    async fn endpoint_ok(&self, url: &str) -> bool {
        let http_ok = self.http_probe(url).await;
        if http_ok {
            return true;
        }

        // If HTTP failed (e.g., HEAD disabled), fall back to a TCP reachability probe.
        if let Some((host, port)) = self.host_and_port(url) {
            return self.tcp_probe(&host, port).await;
        }

        false
    }

    async fn http_probe(&self, url: &str) -> bool {
        debug!("diagnostics: HTTP probe {}", url);
        let head_ok = self
            .client
            .head(url)
            .header("Accept", "*/*")
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .is_ok();
        if head_ok {
            debug!("diagnostics: {} HEAD ok", url);
            return true;
        }

        let ok = self
            .client
            .get(url)
            .header("Accept", "*/*")
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .is_ok();
        if ok {
            debug!("diagnostics: {} GET ok", url);
        } else {
            warn!("diagnostics: {} HTTP probe failed", url);
        }
        ok
    }

    fn host_and_port(&self, url: &str) -> Option<(String, u16)> {
        let parsed = Url::parse(url).ok()?;
        let host = parsed.host_str()?.to_owned();
        let port = parsed.port_or_known_default()?;
        Some((host, port))
    }

    async fn tcp_probe(&self, host: &str, port: u16) -> bool {
        let target = format!("{host}:{port}");
        let connect = TcpStream::connect(target);
        let ok = timeout(Duration::from_secs(5), connect).await.is_ok();
        if ok {
            debug!("diagnostics: TCP probe {host}:{port} ok");
        } else {
            warn!("diagnostics: TCP probe {host}:{port} failed");
        }
        ok
    }
}

pub fn format_report(report: &DiagnosticReport) -> String {
    let mut output = String::new();

    let yes_no = |value| if value { "yes" } else { "no" };
    let status = |value| if value { "OK" } else { "FAILED" };
    let fallback = |value: &Option<String>, placeholder: &str| {
        value
            .as_deref()
            .map(str::to_owned)
            .unwrap_or_else(|| placeholder.to_owned())
    };

    // Capture a quick summary line and a short note of anything that failed.
    let mut connectivity_issues = Vec::new();
    if !report.connectivity.hytale_patches {
        connectivity_issues.push("Hytale patches server");
    }
    if !report.connectivity.github {
        connectivity_issues.push("GitHub API");
    }
    if !report.connectivity.itch_io {
        connectivity_issues.push("itch.io (Butler)");
    }
    if let Some(err) = &report.connectivity.error {
        connectivity_issues.push(err.as_str());
    }

    let connectivity_note = if connectivity_issues.is_empty() {
        "All endpoints reachable".into()
    } else {
        format!("Issues: {}", connectivity_issues.join(", "))
    };

    let _ = writeln!(&mut output, "hrs-launcher Diagnostic Report");
    let _ = writeln!(&mut output, "Generated: {}", report.timestamp);
    let _ = writeln!(
        &mut output,
        "Summary: connectivity={} | installed={} | java={} | butler={}",
        status(connectivity_issues.is_empty()),
        yes_no(report.game_status.installed),
        yes_no(report.dependencies.java_installed),
        yes_no(report.dependencies.butler_installed),
    );

    let _ = writeln!(&mut output, "\n=== PLATFORM ===");
    let _ = writeln!(&mut output, "OS: {}", report.platform.os);
    let _ = writeln!(&mut output, "Arch: {}", report.platform.arch);
    let _ = writeln!(
        &mut output,
        "Launcher Version: {}",
        report.platform.launcher_version
    );

    let _ = writeln!(&mut output, "\n=== CONNECTIVITY ===");
    let _ = writeln!(
        &mut output,
        "Hytale Patches Server: {}",
        status(report.connectivity.hytale_patches)
    );
    let _ = writeln!(
        &mut output,
        "GitHub API: {}",
        status(report.connectivity.github)
    );
    let _ = writeln!(
        &mut output,
        "itch.io (Butler): {}",
        status(report.connectivity.itch_io)
    );
    let _ = writeln!(
        &mut output,
        "Notes: {}",
        fallback(&report.connectivity.error, &connectivity_note)
    );

    let _ = writeln!(&mut output, "\n=== GAME STATUS ===");
    let _ = writeln!(
        &mut output,
        "Installed: {}",
        yes_no(report.game_status.installed)
    );
    let _ = writeln!(
        &mut output,
        "Version: {}",
        report
            .game_status
            .version
            .clone()
            .unwrap_or_else(|| "unknown".into())
    );
    let _ = writeln!(
        &mut output,
        "Client Exists: {}",
        yes_no(report.game_status.client_exists)
    );
    let _ = writeln!(
        &mut output,
        "Online Fix Applied: {}",
        yes_no(report.game_status.online_fix_applied)
    );

    let _ = writeln!(&mut output, "\n=== DEPENDENCIES ===");
    let _ = writeln!(
        &mut output,
        "Java Installed: {}",
        yes_no(report.dependencies.java_installed)
    );
    let _ = writeln!(
        &mut output,
        "Java Path: {}",
        fallback(&report.dependencies.java_path, "-")
    );
    let _ = writeln!(
        &mut output,
        "Butler Installed: {}",
        yes_no(report.dependencies.butler_installed)
    );
    let _ = writeln!(
        &mut output,
        "Butler Path: {}",
        fallback(&report.dependencies.butler_path, "-")
    );

    output
}

fn format_timestamp(time: SystemTime) -> String {
    let dt: chrono::DateTime<chrono::Utc> = time.into();
    dt.to_rfc3339()
}
