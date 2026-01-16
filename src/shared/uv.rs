/// Minimal reimplementation of the bits we used from the `uv-dirs` crate, kept together for clarity.
///
/// Source: Adapted from https://github.com/astral-sh/uv/tree/main/crates/uv-dirs (MIT), pruned to
/// only the helpers needed by libmacchina. The behavior matches uv-dirs v0.0.14 for:
/// - user executable dir resolution (override / XDG / $HOME fallback)
/// - user state dir and legacy state dir (XDG vs native)
pub mod uv_dirs {
    use std::{
        ffi::OsString,
        path::{Path, PathBuf},
    };

    use etcetera::BaseStrategy;

    // Environment variable names used by uv for directory resolution.
    const XDG_BIN_HOME: &str = "XDG_BIN_HOME";
    const XDG_DATA_HOME: &str = "XDG_DATA_HOME";

    /// Returns an appropriate user-level directory for storing executables.
    ///
    /// Order (same as uv-dirs): override var → XDG_BIN_HOME → XDG_DATA_HOME/../bin → $HOME/.local/bin.
    /// Returns `None` if `$HOME` cannot be resolved. Does not
    /// check if the directory exists.
    pub fn user_executable_directory(override_variable: Option<&'static str>) -> Option<PathBuf> {
        override_variable
            .and_then(std::env::var_os)
            .and_then(parse_path)
            .or_else(|| std::env::var_os(XDG_BIN_HOME).and_then(parse_xdg_path))
            .or_else(|| {
                std::env::var_os(XDG_DATA_HOME)
                    .and_then(parse_xdg_path)
                    .map(|path| path.join("../bin"))
            })
            .or_else(|| {
                let home_dir = etcetera::home_dir().ok();
                home_dir.map(|path| path.join(".local").join("bin"))
            })
    }

    /// `$XDG_DATA_HOME/uv` (or platform-native equivalent) per uv-dirs behavior.
    pub fn user_state_dir() -> Option<PathBuf> {
        etcetera::base_strategy::choose_base_strategy()
            .ok()
            .map(|dirs| dirs.data_dir().join("uv"))
    }

    /// Legacy location (`~/Library/Application Support/uv` on macOS; native strategy elsewhere).
    pub fn legacy_user_state_dir() -> Option<PathBuf> {
        etcetera::base_strategy::choose_native_strategy()
            .ok()
            .map(|dirs| dirs.data_dir().join("uv"))
            .map(|dir| if cfg!(windows) { dir.join("data") } else { dir })
    }

    /// Return a [`PathBuf`] from the given [`OsString`], if non-empty.
    ///
    /// Unlike [`parse_xdg_path`], this accepts relative paths (used for uv override vars).
    fn parse_path(path: OsString) -> Option<PathBuf> {
        if path.is_empty() {
            None
        } else {
            Some(PathBuf::from(path))
        }
    }

    /// Return a [`PathBuf`] if the given [`OsString`] is an absolute path, per XDG spec.
    /// Relative paths are treated as invalid.
    fn parse_xdg_path(path: OsString) -> Option<PathBuf> {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            Some(path)
        } else {
            None
        }
    }
}
