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
    #[allow(dead_code)]
    Online,
}

impl AuthMode {
    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        match self {
            AuthMode::Offline => "Offline",
            AuthMode::Online => "Online",
        }
    }

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
    CheckForUpdates,
    #[allow(dead_code)]
    DownloadMod {
        mod_id: i32,
    },
    RunDiagnostics,
    UninstallGame,
}
