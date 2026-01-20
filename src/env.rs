use std::env;
use std::fs;
use std::path::PathBuf;

/// Returns the root directory used by the launcher (mirrors hrs-launcher defaults).
pub fn default_app_dir() -> PathBuf {
    let base = match env::consts::OS {
        "windows" => env::var_os("LOCALAPPDATA")
            .or_else(|| env::var_os("APPDATA"))
            .map(PathBuf::from),
        "macos" => env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join("Library").join("Application Support")),
        _ => env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(".local").join("share")),
    }
    .unwrap_or_else(|| PathBuf::from("."));

    base.join("hrs-launcher")
}

pub fn cache_dir() -> PathBuf {
    default_app_dir().join("cache")
}

pub fn logs_dir() -> PathBuf {
    default_app_dir().join("logs")
}

pub fn crashes_dir() -> PathBuf {
    default_app_dir().join("crashes")
}

pub fn jre_dir() -> PathBuf {
    default_app_dir().join("jre")
}

pub fn butler_dir() -> PathBuf {
    default_app_dir().join("butler")
}

pub fn game_latest_dir() -> PathBuf {
    default_app_dir()
        .join("release")
        .join("package")
        .join("game")
        .join("latest")
}

pub fn game_version_dir(version: &str) -> PathBuf {
    default_app_dir()
        .join("release")
        .join("package")
        .join("game")
        .join(version)
}

pub fn mods_dir() -> PathBuf {
    default_app_dir().join("UserData").join("Mods")
}

/// Create the on-disk folder layout expected by the launcher.
pub fn ensure_base_dirs() -> std::io::Result<()> {
    let root = default_app_dir();
    let folders = [
        root.clone(),
        jre_dir(),
        butler_dir(),
        cache_dir(),
        logs_dir(),
        crashes_dir(),
        game_latest_dir(),
        default_app_dir().join("UserData"),
        mods_dir(),
    ];

    for dir in folders {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}
