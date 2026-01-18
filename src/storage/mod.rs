use std::path::PathBuf;

use tokio::fs;

use crate::engine::models::LocalState;
use crate::env;

const LOCAL_STATE_FILE: &str = "version.txt";

#[derive(Clone)]
pub struct StorageManager {
    base_dir: PathBuf,
}

impl StorageManager {
    pub fn new() -> Self {
        let base_dir = env::default_app_dir();
        // Best-effort directory creation; failures are surfaced on write.
        let _ = env::ensure_base_dirs();
        Self { base_dir }
    }

    pub async fn read_local_state(&self) -> Option<LocalState> {
        let path = self.base_dir.join(LOCAL_STATE_FILE);
        fs::read(&path).await.ok().and_then(|bytes| {
            let version = String::from_utf8_lossy(&bytes).trim().to_owned();
            (!version.is_empty()).then_some(LocalState { version })
        })
    }

    pub async fn write_local_state(&self, state: &LocalState) -> Result<(), String> {
        let path = self.base_dir.join(LOCAL_STATE_FILE);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("unable to create state dir: {e}"))?;
        }
        fs::write(&path, state.version.as_bytes())
            .await
            .map_err(|e| format!("unable to persist version: {e}"))
    }

    #[allow(dead_code)]
    pub fn cache_path(&self, filename: &str) -> PathBuf {
        env::cache_dir().join(filename)
    }

    #[allow(dead_code)]
    pub fn game_dir(&self) -> PathBuf {
        env::game_latest_dir()
    }

    #[allow(dead_code)]
    pub fn mods_dir(&self) -> PathBuf {
        env::mods_dir()
    }

    #[allow(dead_code)]
    pub fn logs_dir(&self) -> PathBuf {
        env::logs_dir()
    }

    #[allow(dead_code)]
    pub fn crash_dir(&self) -> PathBuf {
        env::crashes_dir()
    }

    pub async fn uninstall_game(&self) -> Result<(), String> {
        let release_dir = self.base_dir.join("release");
        if fs::metadata(&release_dir).await.is_ok() {
            fs::remove_dir_all(&release_dir)
                .await
                .map_err(|e| format!("failed to remove game files: {e}"))?;
        }

        let jre_dir = env::jre_dir();
        if fs::metadata(&jre_dir).await.is_ok() {
            fs::remove_dir_all(&jre_dir)
                .await
                .map_err(|e| format!("failed to remove bundled JRE: {e}"))?;
        }

        let cache_dir = env::cache_dir();
        if fs::metadata(&cache_dir).await.is_ok() {
            fs::remove_dir_all(&cache_dir)
                .await
                .map_err(|e| format!("failed to remove cache: {e}"))?;
        }

        let butler_dir = env::butler_dir();
        if fs::metadata(&butler_dir).await.is_ok() {
            fs::remove_dir_all(&butler_dir)
                .await
                .map_err(|e| format!("failed to remove butler files: {e}"))?;
        }

        let user_data_dir = self.base_dir.join("UserData");
        if fs::metadata(&user_data_dir).await.is_ok() {
            fs::remove_dir_all(&user_data_dir)
                .await
                .map_err(|e| format!("failed to remove user data: {e}"))?;
        }

        let version_file = self.base_dir.join(LOCAL_STATE_FILE);
        if fs::metadata(&version_file).await.is_ok() {
            fs::remove_file(&version_file)
                .await
                .map_err(|e| format!("failed to clear saved version: {e}"))?;
        }

        Ok(())
    }
}
