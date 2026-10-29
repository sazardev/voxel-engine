use bevy::log::LogPlugin;
use bevy::prelude::*;

const PLAYER_LOG_FILTER: &str = "warn,game=info";
const DEV_LOG_FILTER: &str = "info,game=debug,wgpu=warn,naga=warn";

/// How this build was compiled: for players, or with developer tools (`--features dev`).
/// Decided at compile time so player builds contain no dev code; never branch gameplay on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub enum RunMode {
    Player,
    Dev,
}

impl RunMode {
    #[must_use]
    pub const fn current() -> Self {
        if cfg!(feature = "dev") {
            Self::Dev
        } else {
            Self::Player
        }
    }

    /// Logging is the only thing that differs by mode before the app starts.
    #[must_use]
    pub fn log_plugin(self) -> LogPlugin {
        let filter = match self {
            Self::Player => PLAYER_LOG_FILTER,
            Self::Dev => DEV_LOG_FILTER,
        };
        LogPlugin {
            filter: filter.to_owned(),
            ..default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_logs_are_quieter_than_dev_logs() {
        assert!(RunMode::Player.log_plugin().filter.starts_with("warn"));
        assert!(RunMode::Dev.log_plugin().filter.contains("game=debug"));
    }
}
