use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use log::{debug, error, info, warn};
use tokio::sync::mpsc;

use crate::diagnostics::Diagnostics;
use crate::engine::models::LocalState;
use crate::engine::state::{AppState, UserAction};
use crate::env;
use crate::jre::JreManager;
use crate::mods::ModService;
use crate::process::ProcessLauncher;
use crate::pwr;
use crate::storage::StorageManager;

pub mod models;
pub mod state;

pub struct LauncherEngine {
    pub state: AppState,
    storage: StorageManager,
    process: ProcessLauncher,
    mods: ModService,
    jre: JreManager,
    cancel_flag: Arc<AtomicBool>,
}

impl LauncherEngine {
    pub fn new(
        storage: StorageManager,
        process: ProcessLauncher,
        cancel_flag: Arc<AtomicBool>,
    ) -> Self {
        let mods = ModService::new(storage.mods_dir());
        let jre = JreManager::default();
        Self {
            state: AppState::Initialising,
            storage,
            process,
            mods,
            jre,
            cancel_flag,
        }
    }

    pub fn mods_service(&self) -> ModService {
        self.mods.clone()
    }

    pub async fn load_local_state(&mut self, updates: &mpsc::UnboundedSender<AppState>) {
        info!("load_local_state: checking cached install");
        let local_state = self.storage.read_local_state().await;
        let state = match local_state {
            Some(local) if self.client_path().exists() => AppState::ReadyToPlay {
                version: local.version,
            },
            _ => AppState::Idle,
        };
        self.state = state.clone();
        let _ = updates.send(state);
    }

    pub async fn bootstrap(
        &mut self,
        requested_version: Option<u32>,
        updates: &mpsc::UnboundedSender<AppState>,
    ) {
        self.reset_cancel_flag();
        updates.send(AppState::CheckingForUpdates).ok();
        info!("bootstrap: starting update check");
        if let Err(err) = self.ensure_jre_ready(updates).await {
            let err_state = AppState::Error(err);
            self.state = err_state.clone();
            let _ = updates.send(err_state);
            error!(
                "bootstrap: failed to ensure JRE ready: {}",
                self.error_summary()
            );
            return;
        }
        if self.cancel_requested() {
            let err_state = AppState::Error("Download cancelled".into());
            self.state = err_state.clone();
            let _ = updates.send(err_state);
            warn!("bootstrap: cancelled after JRE step");
            return;
        }
        match self.try_prepare_game(requested_version, updates).await {
            Ok(version) => {
                let ready = AppState::ReadyToPlay { version };
                self.state = ready.clone();
                updates.send(ready).ok();
                info!("bootstrap: game ready (version {})", self.state_version());
            }
            Err(err) => {
                let err_state = AppState::Error(err);
                self.state = err_state.clone();
                updates.send(err_state).ok();
                error!(
                    "bootstrap: failed to prepare game: {}",
                    self.error_summary()
                );
            }
        }
    }

    pub async fn handle_action(
        &mut self,
        action: UserAction,
        updates: &mpsc::UnboundedSender<AppState>,
    ) {
        match action {
            UserAction::CheckForUpdates { target_version } => {
                info!(
                    "action: CheckForUpdates (target={})",
                    target_version
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "latest".into())
                );
                self.bootstrap(target_version, updates).await;
            }
            UserAction::DownloadGame { target_version } => {
                info!(
                    "action: DownloadGame (target={})",
                    target_version
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "latest".into())
                );
                self.bootstrap(target_version, updates).await;
            }
            UserAction::ClickPlay {
                player_name,
                auth_mode,
            } => match self.state.clone() {
                AppState::ReadyToPlay { version } => {
                    info!(
                        "action: ClickPlay for version {} as {}",
                        version, player_name
                    );
                    if let Err(err) = self.ensure_game_unpacked(&version, updates) {
                        let err_state = AppState::Error(err);
                        self.state = err_state.clone();
                        updates.send(err_state).ok();
                        error!("play failed: {}", self.error_summary());
                        return;
                    }
                    
                    // Apply enabled mods before launching the game
                    info!("Applying enabled mods...");
                    if let Err(err) = self.mods.apply_enabled_mods().await {
                        warn!("Failed to apply mods: {}", err);
                        // Continue anyway - mods are optional
                    } else {
                        info!("Mods applied successfully");
                    }
                    
                    updates.send(AppState::Playing).ok();
                    self.state = AppState::Playing;
                    if let Err(err) =
                        self.process
                            .launch(&version, &player_name, auth_mode.arg_value())
                    {
                        let err_state = AppState::Error(err);
                        self.state = err_state.clone();
                        updates.send(err_state).ok();
                        error!("launch failed: {}", self.error_summary());
                    } else {
                        self.state = AppState::Idle;
                        updates.send(AppState::Idle).ok();
                        info!("game launched successfully");
                    }
                }
                AppState::Error(_) => {
                    warn!("action: ClickPlay while in Error; re-running bootstrap");
                    self.bootstrap(None, updates).await;
                }
                _ => {}
            },
            UserAction::ClickCancelDownload => {
                self.cancel_flag.store(true, Ordering::SeqCst);
                warn!("action: ClickCancelDownload");
            }
            UserAction::RunDiagnostics => {
                info!("action: RunDiagnostics");
                updates.send(AppState::DiagnosticsRunning).ok();
                let report = self.run_diagnostics().await;
                let state = AppState::DiagnosticsReady { report };
                self.state = state.clone();
                updates.send(state).ok();
                info!("diagnostics completed");
            }
            UserAction::UninstallGame => {
                info!("action: UninstallGame");
                updates.send(AppState::Uninstalling).ok();
                self.state = AppState::Uninstalling;
                match self.storage.uninstall_game().await {
                    Ok(_) => {
                        self.state = AppState::Idle;
                        updates.send(AppState::Idle).ok();
                        info!("uninstall completed");
                    }
                    Err(err) => {
                        let err_state = AppState::Error(err);
                        self.state = err_state.clone();
                        updates.send(err_state).ok();
                        error!("uninstall failed: {}", self.error_summary());
                    }
                }
            }
            UserAction::DownloadMod { mod_id } => match self.download_mod(mod_id, updates).await {
                Ok(_) => {
                    let next_state = if let Some(local) = self.storage.read_local_state().await {
                        AppState::ReadyToPlay {
                            version: local.version,
                        }
                    } else {
                        AppState::Idle
                    };
                    self.state = next_state.clone();
                    updates.send(next_state).ok();
                    info!("mod {} downloaded", mod_id);
                }
                Err(err) => {
                    let err_state = AppState::Error(err);
                    self.state = err_state.clone();
                    updates.send(err_state).ok();
                    error!("mod {} download failed: {}", mod_id, self.error_summary());
                }
            },
            UserAction::OpenGameFolder => {
                info!("action: OpenGameFolder");
                if let Err(err) = self.open_game_folder() {
                    let err_state = AppState::Error(err);
                    self.state = err_state.clone();
                    updates.send(err_state).ok();
                    error!("open game folder failed: {}", self.error_summary());
                }
            }
        }
    }

    pub fn storage_clone(&self) -> StorageManager {
        self.storage.clone()
    }

    pub async fn available_versions(&self) -> pwr::VersionCheckResult {
        Self::available_versions_with_storage(self.storage.clone()).await
    }

    pub async fn available_versions_with_storage(
        storage: StorageManager,
    ) -> pwr::VersionCheckResult {
        let mut check = pwr::find_latest_version_with_details("release").await;
        if let Some(local) = storage.read_local_state().await
            && let Ok(parsed) = local.version.parse::<u32>()
            && !check.available_versions.contains(&parsed)
        {
            check.available_versions.push(parsed);
            check.available_versions.sort_unstable_by(|a, b| b.cmp(a));
            check.available_versions.dedup();
        }
        check
    }

    pub async fn run_diagnostics(&self) -> String {
        let diag = Diagnostics::new(env!("CARGO_PKG_VERSION")).run().await;
        crate::diagnostics::format_report(&diag)
    }

    async fn try_prepare_game(
        &mut self,
        requested_version: Option<u32>,
        updates: &mpsc::UnboundedSender<AppState>,
    ) -> Result<String, String> {
        if self.cancel_requested() {
            warn!("prepare_game: cancellation requested");
            return Err("Download cancelled".into());
        }

        let local_state = self.storage.read_local_state().await;
        let local_version = local_state
            .as_ref()
            .and_then(|s| s.version.parse::<u32>().ok());
        let client_exists = self.client_path().exists();

        let check = self.available_versions().await;
        let mut known_versions = check.available_versions.clone();
        if let Some(local) = local_version
            && !known_versions.contains(&local)
        {
            known_versions.push(local);
        }
        if !known_versions.is_empty() {
            known_versions.sort_unstable_by(|a, b| b.cmp(a));
            known_versions.dedup();
        }

        if let Some(err) = check.error.clone() {
            if let Some(target) = requested_version {
                if client_exists && local_version == Some(target) {
                    warn!(
                        "prepare_game: version check failed ({}); using cached version {}",
                        err, target
                    );
                    return Ok(target.to_string());
                }
                error!(
                    "prepare_game: version check failed for requested version {}: {}",
                    target, err
                );
                return Err(err);
            }
            if client_exists && local_version.is_some() {
                warn!(
                    "prepare_game: version check failed ({}); using cached version {:?}",
                    err, local_version
                );
                if let Some(local) = local_version {
                    return Ok(local.to_string());
                }
            }
            error!("prepare_game: version check failed: {}", err);
            return Err(err);
        }

        let latest = check.latest_version;
        info!(
            "prepare_game: latest version {}, checked URLs={:?}",
            latest, check.checked_urls
        );

        debug!(
            "prepare_game: local version {:?}, client exists={}, requested={:?}",
            local_state.as_ref().map(|s| s.version.clone()),
            client_exists,
            requested_version
        );

        let target_version = requested_version
            .or_else(|| (latest > 0).then_some(latest))
            .or(local_version)
            .unwrap_or(0);
        if target_version == 0 {
            return Err("No game versions available for this platform".into());
        }
        if let Some(max_known) = known_versions.first()
            && target_version <= *max_known
            && !known_versions.contains(&target_version)
        {
            return Err(format!(
                "Version {} is not available for this platform",
                target_version
            ));
        }

        if client_exists && local_version == Some(target_version) {
            info!("prepare_game: version {} already installed", target_version);
            return Ok(target_version.to_string());
        }

        let mut progress_cb = |update: pwr::ProgressUpdate| {
            let label = update
                .current_file
                .clone()
                .unwrap_or_else(|| update.stage.to_string());
            let speed = update
                .speed
                .clone()
                .unwrap_or_else(|| update.message.clone());
            let state = AppState::Downloading {
                file: label,
                progress: update.progress,
                speed,
            };
            let _ = updates.send(state.clone());
            debug!(
                "download progress: stage={} file={:?} progress={:.1} speed={:?}",
                update.stage, update.current_file, update.progress, update.speed
            );
        };

        let baseline_version = if client_exists { local_version } else { None }.unwrap_or(0);
        let pwr_path = pwr::download_pwr(
            "release",
            baseline_version,
            target_version,
            Some(self.cancel_flag.clone()),
            Some(&mut progress_cb),
        )
        .await?;
        if self.cancel_requested() {
            warn!("prepare_game: cancelled after download");
            return Err("Download cancelled".into());
        }
        pwr::apply_pwr(&pwr_path, Some(&mut progress_cb)).await?;

        let version_str = target_version.to_string();
        self.storage
            .write_local_state(&LocalState {
                version: version_str.clone(),
            })
            .await?;
        let _ = pwr::save_local_version(target_version);

        Ok(version_str)
    }

    async fn ensure_jre_ready(
        &mut self,
        updates: &mpsc::UnboundedSender<AppState>,
    ) -> Result<(), String> {
        let state = AppState::Downloading {
            file: "Java Runtime".into(),
            progress: 0.0,
            speed: "starting".into(),
        };
        let _ = updates.send(state);
        info!("ensure_jre_ready: ensuring runtime");
        self.jre.ensure_jre(Some(self.cancel_flag.as_ref())).await?;
        info!("ensure_jre_ready: runtime available");
        Ok(())
    }

    async fn download_mod(
        &mut self,
        mod_id: i32,
        updates: &mpsc::UnboundedSender<AppState>,
    ) -> Result<(), String> {
        if !self.client_path().exists() {
            return Err("Install the game before installing mods.".into());
        }
        self.reset_cancel_flag();
        let label = format!("mod-{mod_id}");
        let start = AppState::Downloading {
            file: label.clone(),
            progress: 0.0,
            speed: "starting".into(),
        };
        updates.send(start).ok();

        self.mods
            .download_latest(mod_id, Some(self.cancel_flag.clone()), |pct, message| {
                let state = AppState::Downloading {
                    file: label.clone(),
                    progress: pct,
                    speed: message.to_string(),
                };
                let _ = updates.send(state);
                debug!("mod {} progress: {:.1}% ({})", mod_id, pct, message);
            })
            .await
            .map(|_| ())
    }

    fn ensure_game_unpacked(
        &self,
        version: &str,
        _updates: &mpsc::UnboundedSender<AppState>,
    ) -> Result<(), String> {
        let client = self.client_path();
        if client.exists() {
            debug!(
                "ensure_game_unpacked: client exists at {}",
                client.display()
            );
            return Ok(());
        }

        Err(format!(
            "Game version {version} is not installed. Please redownload in the launcher."
        ))
    }

    fn client_path(&self) -> PathBuf {
        let base = self.storage.game_dir();
        if cfg!(target_os = "windows") {
            base.join("Client").join("HytaleClient.exe")
        } else if cfg!(target_os = "macos") {
            base.join("Client")
                .join("Hytale.app")
                .join("Contents")
                .join("MacOS")
                .join("HytaleClient")
        } else {
            base.join("Client").join("HytaleClient")
        }
    }

    fn reset_cancel_flag(&self) {
        self.cancel_flag.store(false, Ordering::SeqCst);
        debug!("cancel flag reset");
    }

    fn cancel_requested(&self) -> bool {
        let value = self.cancel_flag.load(Ordering::SeqCst);
        if value {
            debug!("cancel flag observed set");
        }
        value
    }

    fn error_summary(&self) -> String {
        match &self.state {
            AppState::Error(msg) => msg.clone(),
            _ => "unknown error".into(),
        }
    }

    fn state_version(&self) -> String {
        match &self.state {
            AppState::ReadyToPlay { version } => version.clone(),
            _ => "-".into(),
        }
    }

    fn open_game_folder(&self) -> Result<(), String> {
        let dir = env::game_latest_dir();
        if !dir.exists() {
            return Err("Game folder not found. Download the game first.".into());
        }
        open::that(&dir).map_err(|err| format!("failed to open game folder: {err}"))?;
        Ok(())
    }
}
