use std::{
    env as std_env,
    process::{Command, Stdio},
};

use crate::env;
use log::{debug, info, warn};
use sysinfo::{System, SystemExt};

#[derive(Clone, Default)]
pub struct ProcessLauncher;

impl ProcessLauncher {
    pub fn new() -> Self {
        Self
    }

    pub fn launch(&self, version: &str, player_name: &str, auth_mode: &str) -> Result<(), String> {
        let base_dir = env::default_app_dir();
        let version_dir = env::game_version_dir(version);
        let game_dir = if version_dir.exists() {
            version_dir
        } else {
            env::game_latest_dir()
        };

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

        if !client_path.exists() {
            warn!("launch: client not found at {}", client_path.display());
            return Err(format!(
                "game client not found at {}",
                client_path.display()
            ));
        }

        let user_dir = base_dir.join("UserData");
        std::fs::create_dir_all(&user_dir)
            .map_err(|e| format!("failed to ensure user data dir: {e}"))?;

        let jre_path = if cfg!(target_os = "windows") {
            env::jre_dir().join("bin").join("java.exe")
        } else {
            env::jre_dir().join("bin").join("java")
        };
        if !jre_path.exists() {
            warn!("launch: Java runtime missing at {}", jre_path.display());
            return Err(format!("Java runtime not found at {}", jre_path.display()));
        }

        info!(
            "launch: starting version {} for player {} using auth {}",
            version, player_name, auth_mode
        );
        debug!(
            "launch: game_dir={} jre_path={} user_dir={}",
            game_dir.display(),
            jre_path.display(),
            user_dir.display()
        );

        let java_env = compute_java_options()
            .map(|opts| merge_java_options(std_env::var("JDK_JAVA_OPTIONS").ok(), &opts));

        let mut cmd = if cfg!(target_os = "macos") {
            let app_bundle = game_dir.join("Client").join("Hytale.app");
            let mut command = Command::new("open");
            command
                .arg(app_bundle)
                .arg("--args")
                .arg("--app-dir")
                .arg(&game_dir)
                .arg("--user-dir")
                .arg(&user_dir)
                .arg("--java-exec")
                .arg(&jre_path)
                .arg("--auth-mode")
                .arg(auth_mode)
                .arg("--uuid")
                .arg("00000000-1337-1337-1337-000000000000")
                .arg("--name")
                .arg(player_name);
            command
        } else {
            let mut command = Command::new(&client_path);
            command
                .arg("--app-dir")
                .arg(&game_dir)
                .arg("--user-dir")
                .arg(&user_dir)
                .arg("--java-exec")
                .arg(&jre_path)
                .arg("--auth-mode")
                .arg(auth_mode)
                .arg("--uuid")
                .arg("00000000-1337-1337-1337-000000000000")
                .arg("--name")
                .arg(player_name);

            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                // CREATE_NO_WINDOW | DETACHED_PROCESS
                command.creation_flags(0x08000000 | 0x00000008);
            }

            #[cfg(target_os = "linux")]
            {
                let client_dir = game_dir.join("Client");
                let ld = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
                let new_ld = format!("{}:{}", client_dir.display(), ld);
                command.env("LD_LIBRARY_PATH", new_ld);
            }

            command
        };

        cmd.current_dir(&base_dir);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        if let Some(merged_opts) = java_env {
            debug!("launch: JDK_JAVA_OPTIONS={}", merged_opts);
            cmd.env("JDK_JAVA_OPTIONS", merged_opts);
        } else {
            debug!("launch: skipping JDK_JAVA_OPTIONS; unable to determine system resources");
        }

        cmd.spawn()
            .map_err(|e| format!("failed to start game process: {e}"))?;
        info!("launch: process started");
        Ok(())
    }
}

fn compute_java_options() -> Option<String> {
    // Derive JVM tuning flags from available system resources.
    let mut system = System::new();
    system.refresh_memory();

    let total_kib = system.total_memory();
    let available_kib = system.available_memory();
    if total_kib == 0 || available_kib == 0 {
        return None;
    }

    let total_bytes = total_kib.saturating_mul(1024);
    let available_bytes = available_kib.saturating_mul(1024);
    if total_bytes == 0 {
        return None;
    }

    let max_ram_percent =
        ((available_bytes as f64 / total_bytes as f64) * 100.0 - 10.0).clamp(40.0, 80.0);
    let initial_ram_percent = (max_ram_percent * 0.6).clamp(25.0, 60.0);
    let cpu_count = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1);

    Some(format!(
        "-XX:+UseStringDeduplication -XX:ActiveProcessorCount={} \
         -XX:MaxRAMPercentage={:.1} -XX:InitialRAMPercentage={:.1}",
        cpu_count, max_ram_percent, initial_ram_percent
    ))
}

fn merge_java_options(existing: Option<String>, computed: &str) -> String {
    let mut merged = existing.unwrap_or_default().trim().to_string();
    let skip_max = merged.contains("MaxRAMPercentage");
    let skip_initial = merged.contains("InitialRAMPercentage");
    let skip_cpu = merged.contains("ActiveProcessorCount");
    let skip_dedupe = merged.contains("UseStringDeduplication");
    let skip_gc = merged.contains("Use") && merged.contains("GC");

    for token in computed.split_whitespace() {
        let include = match token {
            opt if opt.contains("Use") && opt.contains("GC") => !skip_gc,
            opt if opt.contains("MaxRAMPercentage") => !skip_max,
            opt if opt.contains("InitialRAMPercentage") => !skip_initial,
            opt if opt.contains("ActiveProcessorCount") => !skip_cpu,
            opt if opt.contains("UseStringDeduplication") => !skip_dedupe,
            _ => true,
        };
        if include {
            if !merged.is_empty() {
                merged.push(' ');
            }
            merged.push_str(token);
        }
    }

    merged
}
