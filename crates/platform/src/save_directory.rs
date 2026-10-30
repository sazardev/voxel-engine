//! Where this device keeps the game's saves: the per-user data directory of each OS.

use std::path::PathBuf;

/// Folder name under the user's data directory.
const GAME_FOLDER: &str = "voxel-engine";

/// The save directory, or `None` where the device has no writable file system (the web) or the
/// user's home cannot be found. The caller then runs without saves.
#[must_use]
pub fn save_directory() -> Option<PathBuf> {
    data_directory().map(|data| data.join(GAME_FOLDER).join("saves"))
}

#[cfg(target_os = "windows")]
fn data_directory() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(PathBuf::from)
}

#[cfg(target_os = "macos")]
fn data_directory() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Application Support"))
}

#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_os = "android"),
    not(target_os = "ios")
))]
fn data_directory() -> Option<PathBuf> {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
}

/// Mobile and the web get their own storage adapters later; until then, no saves.
#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    all(unix, not(target_os = "android"), not(target_os = "ios"))
)))]
fn data_directory() -> Option<PathBuf> {
    None
}
