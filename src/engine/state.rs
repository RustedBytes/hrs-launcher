// The central source of truth for your UI.
#[derive(Clone, Debug)]
pub enum AppState {
    Idle,
    Initialising,
    CheckingForUpdates,
    Downloading {
        file: String,
        progress: f32,
        speed: String,
    },
    Uninstalling,
    ReadyToPlay {
        version: String,
    },
    DiagnosticsRunning,
    DiagnosticsReady {
        report: String,
    },
    Playing,
    Error(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthMode {
    Offline,
    Online,
}

impl AuthMode {
    pub fn arg_value(self) -> &'static str {
        match self {
            AuthMode::Offline => "offline",
            AuthMode::Online => "online",
        }
    }
}

// Actions triggered by the user from the UI layer.
#[derive(Clone, Debug)]
pub enum UserAction {
    ClickPlay {
        player_name: String,
        auth_mode: AuthMode,
    },
    ClickCancelDownload,
    CheckForUpdates {
        target_version: Option<u32>,
    },
    DownloadMod {
        mod_id: i32,
    },
    RunDiagnostics,
    UninstallGame,
    DownloadGame {
        target_version: Option<u32>,
    },
    OpenGameFolder,
}
