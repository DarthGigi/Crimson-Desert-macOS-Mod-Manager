use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModKind {
    JsonData,
    PrecompiledOverlay,
    BrowserRaw,
    Language,
}

impl ModKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::JsonData => "json_data",
            Self::PrecompiledOverlay => "precompiled_overlay",
            Self::BrowserRaw => "browser_raw",
            Self::Language => "language",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "json_data" => Some(Self::JsonData),
            "precompiled_overlay" => Some(Self::PrecompiledOverlay),
            "browser_raw" => Some(Self::BrowserRaw),
            "language" => Some(Self::Language),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModChange {
    pub offset: usize,
    pub original: String,
    pub patched: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub entry: Option<String>,
    #[serde(default, alias = "rel_offset")]
    pub rel_offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModPatch {
    #[serde(alias = "game_file")]
    pub game_file: String,
    #[serde(default, alias = "source_group")]
    pub source_group: Option<String>,
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
    pub mod_kind: ModKind,
    pub name: String,
    pub description: Option<String>,
    pub file_name: String,
    pub source_path: Option<String>,
    pub library_path: String,
    pub enabled: bool,
    pub load_order: i64,
    pub language: Option<String>,
    pub install_group: Option<String>,
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
    pub mod_kind: ModKind,
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
    pub source_group: String,
    pub source_paz_index: u16,
    pub applied_changes: usize,
    pub skipped_changes: usize,
    pub overlap_count: usize,
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
    pub created_groups: Vec<String>,
    pub files: Vec<ApplyFileResult>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyPreviewFile {
    pub game_file: String,
    pub source_group: String,
    pub source_paz_index: Option<u16>,
    pub change_count: usize,
    pub overlap_count: usize,
    pub source_mods: Vec<String>,
    pub resolved: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyPreview {
    pub mod_count: usize,
    pub json_mod_count: usize,
    pub precompiled_mod_count: usize,
    pub browser_raw_mod_count: usize,
    pub target_file_count: usize,
    pub estimated_group_count: usize,
    pub selected_language: Option<String>,
    pub files: Vec<ApplyPreviewFile>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModPatchSummary {
    pub mod_id: String,
    pub patch_index: usize,
    pub title: String,
    pub source_group: String,
    pub game_file: String,
    pub change_count: usize,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ManagedGroupRecord {
    pub group_name: String,
    pub purpose: String,
    pub source_mod_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusSummary {
    pub game_install: Option<GameInstallInfo>,
    pub selected_language: Option<String>,
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
