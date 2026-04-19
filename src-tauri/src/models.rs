use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModChange {
    pub offset: usize,
    pub original: String,
    pub patched: String,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModPatch {
    pub game_file: String,
    pub changes: Vec<ModChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModManifest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub patches: Vec<ModPatch>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInstallInfo {
    pub packages_path: String,
    pub app_path: Option<String>,
    pub meta_exists: bool,
    pub pamt_exists: bool,
    pub writable: bool,
    pub detected: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModRecord {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub file_name: String,
    pub source_path: Option<String>,
    pub library_path: String,
    pub enabled: bool,
    pub patch_count: usize,
    pub change_count: usize,
    pub target_files: Vec<String>,
    pub imported_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub path: String,
    pub file_name: String,
    pub name: String,
    pub description: Option<String>,
    pub patch_count: usize,
    pub change_count: usize,
    pub target_files: Vec<String>,
    pub resolvable_files: usize,
    pub missing_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyFileResult {
    pub game_file: String,
    pub source_paz_index: u16,
    pub applied_changes: usize,
    pub skipped_changes: usize,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResult {
    pub mod_count: usize,
    pub target_file_count: usize,
    pub overlay_file_count: usize,
    pub paz_size: usize,
    pub pamt_size: usize,
    pub files: Vec<ApplyFileResult>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusSummary {
    pub game_install: Option<GameInstallInfo>,
    pub overlay_active: bool,
    pub backup_exists: bool,
    pub total_mods: usize,
    pub enabled_mods: usize,
    pub disabled_mods: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardData {
    pub status: StatusSummary,
    pub available: Vec<ModRecord>,
    pub enabled: Vec<ModRecord>,
    pub disabled: Vec<ModRecord>,
}
