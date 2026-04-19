use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

use crate::error::{AppError, AppResult};
use crate::models::GameInstallInfo;

pub const SOURCE_GROUP: &str = "0008";

fn is_valid_packages_dir(path: &Path) -> bool {
    path.join("meta").join("0.papgt").is_file() && path.join(SOURCE_GROUP).join("0.pamt").is_file()
}

pub fn resolve_to_packages_dir(raw: &str) -> AppResult<PathBuf> {
    let path = PathBuf::from(raw).expand_home();
    let path =
        fs::canonicalize(&path).map_err(|_| AppError::InvalidGameInstall(raw.to_string()))?;

    if path.is_dir() && path.extension().and_then(|ext| ext.to_str()) == Some("app") {
        let inner = path.join("Contents").join("Resources").join("packages");
        if is_valid_packages_dir(&inner) {
            return Ok(inner);
        }
    }

    if is_valid_packages_dir(&path) {
        return Ok(path);
    }

    Err(AppError::InvalidGameInstall(raw.to_string()))
}

pub fn detect_packages_dir() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let candidates = [
        "/Applications/Crimson Desert.app".to_string(),
        format!("{home}/Applications/Crimson Desert.app"),
        format!(
			"{home}/Library/Application Support/Steam/steamapps/common/Crimson Desert/Crimson Desert.app"
		),
        format!("{home}/Library/Application Support/Steam/steamapps/common/Crimson Desert"),
    ];

    for candidate in candidates {
        if let Ok(path) = resolve_to_packages_dir(&candidate) {
            return Some(path);
        }
    }

    None
}

pub fn packages_to_app_path(packages_path: &Path) -> Option<PathBuf> {
    let app_path = packages_path.parent()?.parent()?.parent()?;
    if app_path.is_dir() && app_path.extension().and_then(|ext| ext.to_str()) == Some("app") {
        Some(app_path.to_path_buf())
    } else {
        None
    }
}

pub fn inspect_game_install(packages_path: &Path, detected: bool) -> GameInstallInfo {
    let meta_exists = packages_path.join("meta").join("0.papgt").is_file();
    let pamt_exists = packages_path.join(SOURCE_GROUP).join("0.pamt").is_file();
    let writable = fs::metadata(packages_path)
        .map(|metadata| !metadata.permissions().readonly())
        .unwrap_or(false);

    GameInstallInfo {
        packages_path: packages_path.display().to_string(),
        app_path: packages_to_app_path(packages_path).map(|path| path.display().to_string()),
        meta_exists,
        pamt_exists,
        writable,
        detected,
    }
}

pub fn launch_game(packages_path: &Path) -> AppResult<()> {
    let app_path = packages_to_app_path(packages_path).ok_or_else(|| {
        AppError::NotFound("Could not derive the .app bundle from the packages path".to_string())
    })?;

    let status = Command::new("open").arg(&app_path).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(AppError::Other(format!(
            "Failed to launch game: open exited with status {status}"
        )))
    }
}

trait ExpandHome {
    fn expand_home(self) -> PathBuf;
}

impl ExpandHome for PathBuf {
    fn expand_home(self) -> PathBuf {
        let raw = self.to_string_lossy();
        if !raw.starts_with('~') {
            return self;
        }

        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(raw.replacen('~', &home, 1));
        }

        self
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchResult {
    pub launched: bool,
}
