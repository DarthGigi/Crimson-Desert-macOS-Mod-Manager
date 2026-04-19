mod db;
mod error;
mod game;
mod models;
mod mods;
mod pathc;
mod patcher;
mod util;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use db::{
    clear_managed_groups, clear_patch_toggles, connect, get_setting, insert_history, list_history, list_managed_groups, list_mods,
    list_disabled_patch_indexes, move_mod_in_load_order, replace_managed_groups, set_patch_enabled,
    set_setting, update_mod_classification, update_mod_enabled,
};
use error::{AppError, ErrorPayload};
use game::{
    detect_packages_dir, inspect_game_install, launch_game, resolve_to_packages_dir, LaunchResult,
};
use models::{ApplyPreview, ApplyResult, DashboardData, ExtractPreview, ExtractResult, GameInstallInfo, HistoryEntry, ModKind, ModPatchSummary, ModRecord, PathcRepackResult, PathcSummary, ScanResult, StatusSummary, VirtualFileMatch, XmlPreview, XmlRepackResult};
use tauri::{AppHandle, Manager, State};

const SETTINGS_GAME_PATH: &str = "game_packages_path";
const SETTINGS_GAME_LANGUAGE: &str = "selected_game_language";

struct AppState {
    app_data_dir: PathBuf,
    operation_lock: Mutex<()>,
}

impl AppState {
    fn connection(&self) -> Result<rusqlite::Connection, AppError> {
        connect(&self.app_data_dir)
    }

    fn operation_marker_path(&self) -> PathBuf {
        self.app_data_dir.join("operation-in-progress.json")
    }

    fn import_cache_dir(&self) -> PathBuf {
        self.app_data_dir.join("mods").join("import-cache")
    }
}

fn app_state(app: &AppHandle) -> Result<AppState, AppError> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|err| AppError::Other(format!("Could not resolve app data dir: {err}")))?;
    Ok(AppState {
        app_data_dir,
        operation_lock: Mutex::new(()),
    })
}

fn saved_game_path(connection: &rusqlite::Connection) -> Result<Option<PathBuf>, AppError> {
    Ok(get_setting(connection, SETTINGS_GAME_PATH)?.map(PathBuf::from))
}

fn current_game_install(
    connection: &rusqlite::Connection,
) -> Result<Option<GameInstallInfo>, AppError> {
    if let Some(path) = saved_game_path(connection)? {
        return Ok(Some(inspect_game_install(&path, false)));
    }
    Ok(None)
}

fn build_dashboard(connection: &rusqlite::Connection, app_data_dir: &Path) -> Result<DashboardData, AppError> {
    let mods = list_mods(connection)?;
    let managed_groups = list_managed_groups(connection)?;
    let selected_language = get_setting(connection, SETTINGS_GAME_LANGUAGE)?
        .filter(|value| !value.is_empty());
    let enabled: Vec<ModRecord> = mods
        .iter()
        .filter(|record| record.enabled)
        .cloned()
        .collect();
    let disabled: Vec<ModRecord> = mods
        .iter()
        .filter(|record| !record.enabled)
        .cloned()
        .collect();
    let game_install = current_game_install(connection)?;
    let overlay_active = game_install.as_ref().is_some_and(|install| {
        managed_groups.iter().any(|group| {
            Path::new(&install.packages_path)
                .join(&group.group_name)
                .is_dir()
        })
    });
    let backup_exists = game_install.as_ref().is_some_and(|install| {
        Path::new(&install.packages_path)
            .join("meta")
            .join("0.papgt.bak")
            .is_file()
    });
    let recovery_marker = read_operation_marker(app_data_dir)?;

    Ok(DashboardData {
        status: StatusSummary {
            game_install,
            selected_language,
            recovery_pending: recovery_marker.is_some(),
            pending_operation: recovery_marker,
            overlay_active,
            backup_exists,
            total_mods: mods.len(),
            enabled_mods: enabled.len(),
            disabled_mods: disabled.len(),
        },
        available: mods,
        enabled,
        disabled,
    })
}

fn read_operation_marker(app_data_dir: &Path) -> Result<Option<String>, AppError> {
    let marker_path = app_data_dir.join("operation-in-progress.json");
    if !marker_path.is_file() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(marker_path)?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    Ok(value
        .get("operation")
        .and_then(|value| value.as_str())
        .map(str::to_string))
}

fn set_operation_marker(state: &AppState, operation: &str) -> Result<(), AppError> {
    std::fs::write(
        state.operation_marker_path(),
        serde_json::to_vec(&serde_json::json!({ "operation": operation }))?,
    )?;
    Ok(())
}

fn clear_operation_marker(state: &AppState) -> Result<(), AppError> {
    let marker = state.operation_marker_path();
    if marker.exists() {
        std::fs::remove_file(marker)?;
    }
    Ok(())
}

struct OperationMarkerGuard<'a> {
    state: &'a AppState,
    active: bool,
}

impl<'a> OperationMarkerGuard<'a> {
    fn new(state: &'a AppState, operation: &str) -> Result<Self, AppError> {
        set_operation_marker(state, operation)?;
        Ok(Self { state, active: true })
    }

    fn clear(&mut self) -> Result<(), AppError> {
        if self.active {
            clear_operation_marker(self.state)?;
            self.active = false;
        }
        Ok(())
    }
}

impl Drop for OperationMarkerGuard<'_> {
    fn drop(&mut self) {
        if self.active {
            let _ = clear_operation_marker(self.state);
        }
    }
}

#[tauri::command]
fn get_dashboard(state: State<'_, AppState>) -> Result<DashboardData, ErrorPayload> {
    let connection = state.connection().map_err(ErrorPayload::from)?;
    build_dashboard(&connection, &state.app_data_dir).map_err(ErrorPayload::from)
}

#[tauri::command]
fn detect_game_install_command(
    state: State<'_, AppState>,
) -> Result<Option<GameInstallInfo>, ErrorPayload> {
    let connection = state.connection().map_err(ErrorPayload::from)?;
    if let Some(path) = detect_packages_dir() {
        set_setting(&connection, SETTINGS_GAME_PATH, &path.display().to_string())
            .map_err(ErrorPayload::from)?;
        return Ok(Some(inspect_game_install(&path, true)));
    }
    Ok(None)
}

#[tauri::command]
fn set_game_install_command(
    path: String,
    state: State<'_, AppState>,
) -> Result<GameInstallInfo, ErrorPayload> {
    let connection = state.connection().map_err(ErrorPayload::from)?;
    let packages_dir = resolve_to_packages_dir(&path).map_err(ErrorPayload::from)?;
    set_setting(
        &connection,
        SETTINGS_GAME_PATH,
        &packages_dir.display().to_string(),
    )
    .map_err(ErrorPayload::from)?;
    Ok(inspect_game_install(&packages_dir, false))
}

#[tauri::command]
fn scan_mod_folder_command(
    folder_path: String,
    state: State<'_, AppState>,
) -> Result<Vec<ScanResult>, ErrorPayload> {
    let connection = state.connection().map_err(ErrorPayload::from)?;
    let packages_dir = saved_game_path(&connection).map_err(ErrorPayload::from)?;
    mods::scan_import_source(Path::new(&folder_path), packages_dir.as_deref(), &state.app_data_dir)
        .map_err(ErrorPayload::from)
}

#[tauri::command]
fn import_mod_variant_command(
    path: String,
    enable: bool,
    state: State<'_, AppState>,
) -> Result<DashboardData, ErrorPayload> {
    let connection = state.connection().map_err(ErrorPayload::from)?;
    let mod_kind = mods::detect_import_kind(Path::new(&path)).map_err(ErrorPayload::from)?;
    let record = mods::import_mod(
        &state.app_data_dir,
        &connection,
        Path::new(&path),
        enable,
        mod_kind,
        None,
    )
    .map_err(ErrorPayload::from)?;
    insert_history(
        &connection,
        "import",
        "ok",
        &format!("Imported {}", record.file_name),
        None,
    )
    .map_err(ErrorPayload::from)?;
    build_dashboard(&connection, &state.app_data_dir).map_err(ErrorPayload::from)
}

#[tauri::command]
fn set_mod_enabled_command(
    mod_id: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<DashboardData, ErrorPayload> {
    let connection = state.connection().map_err(ErrorPayload::from)?;
    update_mod_enabled(&connection, &mod_id, enabled).map_err(ErrorPayload::from)?;
    insert_history(
        &connection,
        if enabled { "enable" } else { "disable" },
        "ok",
        &format!(
            "{} mod {mod_id}",
            if enabled { "Enabled" } else { "Disabled" }
        ),
        None,
    )
    .map_err(ErrorPayload::from)?;
    build_dashboard(&connection, &state.app_data_dir).map_err(ErrorPayload::from)
}

#[tauri::command]
fn set_selected_language_command(
    language: Option<String>,
    state: State<'_, AppState>,
) -> Result<DashboardData, ErrorPayload> {
    let connection = state.connection().map_err(ErrorPayload::from)?;
    set_setting(
        &connection,
        SETTINGS_GAME_LANGUAGE,
        language.as_deref().unwrap_or(""),
    )
    .map_err(ErrorPayload::from)?;
    build_dashboard(&connection, &state.app_data_dir).map_err(ErrorPayload::from)
}

#[tauri::command]
fn set_mod_classification_command(
    mod_id: String,
    mod_kind: ModKind,
    language: Option<String>,
    state: State<'_, AppState>,
) -> Result<DashboardData, ErrorPayload> {
    let connection = state.connection().map_err(ErrorPayload::from)?;
    let mods = list_mods(&connection).map_err(ErrorPayload::from)?;
    let existing = mods
        .iter()
        .find(|record| record.id == mod_id)
        .ok_or_else(|| ErrorPayload {
            message: format!("No mod found for id {mod_id}"),
        })?;
    let resolved_kind = if mod_kind == ModKind::PrecompiledOverlay && Path::new(&existing.library_path).is_dir() {
        mods::detect_import_kind(Path::new(&existing.library_path)).unwrap_or(mod_kind)
    } else {
        mod_kind
    };
    let normalized_language = if mod_kind == ModKind::Language {
        language.filter(|value| !value.trim().is_empty())
    } else {
        None
    };
    update_mod_classification(&connection, &mod_id, resolved_kind, normalized_language.as_deref())
        .map_err(ErrorPayload::from)?;
    insert_history(
        &connection,
        "classify",
        "ok",
        &format!("Updated classification for mod {mod_id}"),
        None,
    )
    .map_err(ErrorPayload::from)?;
    build_dashboard(&connection, &state.app_data_dir).map_err(ErrorPayload::from)
}

#[tauri::command]
fn move_mod_in_load_order_command(
    mod_id: String,
    direction: String,
    state: State<'_, AppState>,
) -> Result<DashboardData, ErrorPayload> {
    let mut connection = state.connection().map_err(ErrorPayload::from)?;
    move_mod_in_load_order(&mut connection, &mod_id, &direction).map_err(ErrorPayload::from)?;
    insert_history(
        &connection,
        "reorder",
        "ok",
        &format!("Moved mod {mod_id} {direction} in JSON load order"),
        None,
    )
    .map_err(ErrorPayload::from)?;
    build_dashboard(&connection, &state.app_data_dir).map_err(ErrorPayload::from)
}

#[tauri::command]
fn get_mod_patch_summaries_command(
    mod_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ModPatchSummary>, ErrorPayload> {
    let connection = state.connection().map_err(ErrorPayload::from)?;
    let mods = list_mods(&connection).map_err(ErrorPayload::from)?;
    let record = mods
        .iter()
        .find(|record| record.id == mod_id)
        .ok_or_else(|| ErrorPayload {
            message: format!("No mod found for id {mod_id}"),
        })?;
    let manifest = mods::load_manifest(Path::new(&record.library_path)).map_err(ErrorPayload::from)?;
    let disabled = list_disabled_patch_indexes(&connection).map_err(ErrorPayload::from)?;
    Ok(mods::patch_summaries(&mod_id, &manifest, disabled.get(&mod_id)))
}

#[tauri::command]
fn set_patch_enabled_command(
    mod_id: String,
    patch_index: usize,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<DashboardData, ErrorPayload> {
    let connection = state.connection().map_err(ErrorPayload::from)?;
    set_patch_enabled(&connection, &mod_id, patch_index, enabled).map_err(ErrorPayload::from)?;
    insert_history(
        &connection,
        "toggle_patch",
        "ok",
        &format!("{} patch {patch_index} for mod {mod_id}", if enabled { "Enabled" } else { "Disabled" }),
        None,
    )
    .map_err(ErrorPayload::from)?;
    build_dashboard(&connection, &state.app_data_dir).map_err(ErrorPayload::from)
}

#[tauri::command]
fn apply_mods_command(state: State<'_, AppState>) -> Result<ApplyResult, ErrorPayload> {
    let _guard = state.operation_lock.lock().map_err(|_| ErrorPayload {
        message: "Operation lock poisoned".to_string(),
    })?;
    let mut marker = OperationMarkerGuard::new(&state, "apply").map_err(ErrorPayload::from)?;
    let mut connection = state.connection().map_err(ErrorPayload::from)?;
    let packages_dir = saved_game_path(&connection)
        .map_err(ErrorPayload::from)?
        .ok_or_else(|| ErrorPayload {
            message: "Set the Crimson Desert game path first.".to_string(),
        })?;
    let mods = list_mods(&connection).map_err(ErrorPayload::from)?;
    let managed_groups = list_managed_groups(&connection).map_err(ErrorPayload::from)?;
    let disabled_patches = list_disabled_patch_indexes(&connection).map_err(ErrorPayload::from)?;
    let selected_language = get_setting(&connection, SETTINGS_GAME_LANGUAGE)
        .map_err(ErrorPayload::from)?
        .filter(|value| !value.is_empty());
    let result = patcher::apply_mods(
        &packages_dir,
        &mods,
        &managed_groups,
        selected_language.as_deref(),
        &disabled_patches,
    )
        .map_err(ErrorPayload::from)?;
    let replacement_groups = patcher::managed_group_records(&result.created_groups, "json_overlay");
    replace_managed_groups(&mut connection, &replacement_groups).map_err(ErrorPayload::from)?;
    insert_history(&connection, "apply", "ok", &result.message, None)
        .map_err(ErrorPayload::from)?;
    marker.clear().map_err(ErrorPayload::from)?;
    Ok(result)
}

#[tauri::command]
fn get_apply_preview_command(state: State<'_, AppState>) -> Result<ApplyPreview, ErrorPayload> {
    let connection = state.connection().map_err(ErrorPayload::from)?;
    let packages_dir = saved_game_path(&connection)
        .map_err(ErrorPayload::from)?
        .ok_or_else(|| ErrorPayload {
            message: "Set the Crimson Desert game path first.".to_string(),
        })?;
    let mods = list_mods(&connection).map_err(ErrorPayload::from)?;
    let disabled_patches = list_disabled_patch_indexes(&connection).map_err(ErrorPayload::from)?;
    let selected_language = get_setting(&connection, SETTINGS_GAME_LANGUAGE)
        .map_err(ErrorPayload::from)?
        .filter(|value| !value.is_empty());
    patcher::preview_apply(&packages_dir, &mods, selected_language.as_deref(), &disabled_patches)
        .map_err(ErrorPayload::from)
}

#[tauri::command]
fn restore_vanilla_command(state: State<'_, AppState>) -> Result<DashboardData, ErrorPayload> {
    let _guard = state.operation_lock.lock().map_err(|_| ErrorPayload {
        message: "Operation lock poisoned".to_string(),
    })?;
    let mut marker = OperationMarkerGuard::new(&state, "restore").map_err(ErrorPayload::from)?;
    let connection = state.connection().map_err(ErrorPayload::from)?;
    let packages_dir = saved_game_path(&connection)
        .map_err(ErrorPayload::from)?
        .ok_or_else(|| ErrorPayload {
            message: "Set the Crimson Desert game path first.".to_string(),
        })?;
    let managed_groups = list_managed_groups(&connection).map_err(ErrorPayload::from)?;
    patcher::restore_vanilla(&packages_dir, &managed_groups).map_err(ErrorPayload::from)?;
    clear_managed_groups(&connection).map_err(ErrorPayload::from)?;
    insert_history(
        &connection,
        "restore",
        "ok",
        "Restored the game to vanilla",
        None,
    )
    .map_err(ErrorPayload::from)?;
    marker.clear().map_err(ErrorPayload::from)?;
    build_dashboard(&connection, &state.app_data_dir).map_err(ErrorPayload::from)
}

#[tauri::command]
fn reset_active_mods_command(state: State<'_, AppState>) -> Result<DashboardData, ErrorPayload> {
    let _guard = state.operation_lock.lock().map_err(|_| ErrorPayload {
        message: "Operation lock poisoned".to_string(),
    })?;
    let mut marker = OperationMarkerGuard::new(&state, "reset").map_err(ErrorPayload::from)?;
    let connection = state.connection().map_err(ErrorPayload::from)?;
    if let Some(packages_dir) = saved_game_path(&connection).map_err(ErrorPayload::from)? {
        let managed_groups = list_managed_groups(&connection).map_err(ErrorPayload::from)?;
        patcher::restore_vanilla(&packages_dir, &managed_groups).map_err(ErrorPayload::from)?;
    }
    clear_managed_groups(&connection).map_err(ErrorPayload::from)?;
    db::disable_all_mods(&connection).map_err(ErrorPayload::from)?;
    insert_history(
        &connection,
        "reset",
        "ok",
        "Restored the game to vanilla and disabled all active mods",
        None,
    )
    .map_err(ErrorPayload::from)?;
    marker.clear().map_err(ErrorPayload::from)?;
    build_dashboard(&connection, &state.app_data_dir).map_err(ErrorPayload::from)
}

#[tauri::command]
fn launch_game_command(state: State<'_, AppState>) -> Result<LaunchResult, ErrorPayload> {
    let connection = state.connection().map_err(ErrorPayload::from)?;
    let packages_dir = saved_game_path(&connection)
        .map_err(ErrorPayload::from)?
        .ok_or_else(|| ErrorPayload {
            message: "Set the Crimson Desert game path first.".to_string(),
        })?;
    launch_game(&packages_dir).map_err(ErrorPayload::from)?;
    Ok(LaunchResult { launched: true })
}

#[tauri::command]
fn get_pathc_summary_command(
    path: Option<String>,
    lookups: Vec<String>,
    state: State<'_, AppState>,
) -> Result<PathcSummary, ErrorPayload> {
    let connection = state.connection().map_err(ErrorPayload::from)?;
    let resolved_path = if let Some(path) = path.filter(|value| !value.trim().is_empty()) {
        PathBuf::from(path)
    } else {
        let packages_dir = saved_game_path(&connection)
            .map_err(ErrorPayload::from)?
            .ok_or_else(|| ErrorPayload {
                message: "Set the Crimson Desert game path first or choose a .pathc file.".to_string(),
            })?;
        packages_dir.join("meta").join("0.pathc")
    };

    pathc::summarize_pathc(&resolved_path, &lookups).map_err(ErrorPayload::from)
}

#[tauri::command]
fn repack_pathc_command(
    path: Option<String>,
    folder_path: String,
    state: State<'_, AppState>,
) -> Result<PathcRepackResult, ErrorPayload> {
    let _guard = state.operation_lock.lock().map_err(|_| ErrorPayload {
        message: "Operation lock poisoned".to_string(),
    })?;
    let mut marker = OperationMarkerGuard::new(&state, "pathc_repack").map_err(ErrorPayload::from)?;
    let connection = state.connection().map_err(ErrorPayload::from)?;
    let resolved_path = if let Some(path) = path.filter(|value| !value.trim().is_empty()) {
        PathBuf::from(path)
    } else {
        let packages_dir = saved_game_path(&connection)
            .map_err(ErrorPayload::from)?
            .ok_or_else(|| ErrorPayload {
                message: "Set the Crimson Desert game path first or choose a .pathc file.".to_string(),
            })?;
        packages_dir.join("meta").join("0.pathc")
    };

    let result = pathc::repack_pathc(&resolved_path, Path::new(&folder_path)).map_err(ErrorPayload::from)?;
    insert_history(
        &connection,
        "pathc_repack",
        "ok",
        &format!("Repacked PATHC with {} DDS files", result.processed_count),
        None,
    )
    .map_err(ErrorPayload::from)?;
    marker.clear().map_err(ErrorPayload::from)?;
    Ok(result)
}

#[tauri::command]
fn fix_everything_command(state: State<'_, AppState>) -> Result<DashboardData, ErrorPayload> {
    let _guard = state.operation_lock.lock().map_err(|_| ErrorPayload {
        message: "Operation lock poisoned".to_string(),
    })?;
    let mut marker = OperationMarkerGuard::new(&state, "fix_everything").map_err(ErrorPayload::from)?;
    let connection = state.connection().map_err(ErrorPayload::from)?;

    if let Some(packages_dir) = saved_game_path(&connection).map_err(ErrorPayload::from)? {
        let managed_groups = list_managed_groups(&connection).map_err(ErrorPayload::from)?;
        patcher::restore_vanilla(&packages_dir, &managed_groups).map_err(ErrorPayload::from)?;
    }

    clear_managed_groups(&connection).map_err(ErrorPayload::from)?;
    clear_patch_toggles(&connection).map_err(ErrorPayload::from)?;
    db::disable_all_mods(&connection).map_err(ErrorPayload::from)?;
    let import_cache = state.import_cache_dir();
    if import_cache.is_dir() {
        std::fs::remove_dir_all(&import_cache).map_err(AppError::from).map_err(ErrorPayload::from)?;
        std::fs::create_dir_all(&import_cache).map_err(AppError::from).map_err(ErrorPayload::from)?;
    }

    insert_history(
        &connection,
        "fix_everything",
        "ok",
        "Restored vanilla state, cleared managed groups, disabled mods, and reset patch toggles",
        None,
    )
    .map_err(ErrorPayload::from)?;
    marker.clear().map_err(ErrorPayload::from)?;
    build_dashboard(&connection, &state.app_data_dir).map_err(ErrorPayload::from)
}

#[tauri::command]
fn get_virtual_file_preview_command(
    virtual_path: String,
    source_group: Option<String>,
    state: State<'_, AppState>,
) -> Result<ExtractPreview, ErrorPayload> {
    let connection = state.connection().map_err(ErrorPayload::from)?;
    let packages_dir = saved_game_path(&connection)
        .map_err(ErrorPayload::from)?
        .ok_or_else(|| ErrorPayload {
            message: "Set the Crimson Desert game path first.".to_string(),
        })?;
    patcher::preview_virtual_file(&packages_dir, &virtual_path, source_group.as_deref())
        .map_err(ErrorPayload::from)
}

#[tauri::command]
fn extract_virtual_file_command(
    virtual_path: String,
    source_group: Option<String>,
    output_dir: String,
    state: State<'_, AppState>,
) -> Result<ExtractResult, ErrorPayload> {
    let _guard = state.operation_lock.lock().map_err(|_| ErrorPayload {
        message: "Operation lock poisoned".to_string(),
    })?;
    let mut marker = OperationMarkerGuard::new(&state, "extract_virtual_file").map_err(ErrorPayload::from)?;
    let connection = state.connection().map_err(ErrorPayload::from)?;
    let packages_dir = saved_game_path(&connection)
        .map_err(ErrorPayload::from)?
        .ok_or_else(|| ErrorPayload {
            message: "Set the Crimson Desert game path first.".to_string(),
        })?;
    let result = patcher::extract_virtual_file(&packages_dir, &virtual_path, source_group.as_deref(), Path::new(&output_dir))
        .map_err(ErrorPayload::from)?;
    insert_history(
        &connection,
        "extract_virtual_file",
        "ok",
        &format!("Extracted {} to {}", result.virtual_path, result.output_path),
        None,
    )
    .map_err(ErrorPayload::from)?;
    marker.clear().map_err(ErrorPayload::from)?;
    Ok(result)
}

#[tauri::command]
fn search_virtual_files_command(
    query: String,
    source_group: Option<String>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<VirtualFileMatch>, ErrorPayload> {
    let connection = state.connection().map_err(ErrorPayload::from)?;
    let packages_dir = saved_game_path(&connection)
        .map_err(ErrorPayload::from)?
        .ok_or_else(|| ErrorPayload {
            message: "Set the Crimson Desert game path first.".to_string(),
        })?;
    patcher::search_virtual_files(&packages_dir, &query, source_group.as_deref(), limit.unwrap_or(100))
        .map_err(ErrorPayload::from)
}

#[tauri::command]
fn extract_xml_entry_command(
    virtual_path: String,
    source_group: Option<String>,
    output_dir: String,
    state: State<'_, AppState>,
) -> Result<XmlPreview, ErrorPayload> {
    let _guard = state.operation_lock.lock().map_err(|_| ErrorPayload {
        message: "Operation lock poisoned".to_string(),
    })?;
    let mut marker = OperationMarkerGuard::new(&state, "extract_xml_entry").map_err(ErrorPayload::from)?;
    let connection = state.connection().map_err(ErrorPayload::from)?;
    let packages_dir = saved_game_path(&connection)
        .map_err(ErrorPayload::from)?
        .ok_or_else(|| ErrorPayload {
            message: "Set the Crimson Desert game path first.".to_string(),
        })?;
    let result = patcher::extract_xml_entry(&packages_dir, &virtual_path, source_group.as_deref(), Path::new(&output_dir))
        .map_err(ErrorPayload::from)?;
    insert_history(
        &connection,
        "extract_xml_entry",
        "ok",
        &format!("Extracted XML {} to {}", result.virtual_path, result.extracted_path),
        None,
    )
    .map_err(ErrorPayload::from)?;
    marker.clear().map_err(ErrorPayload::from)?;
    Ok(result)
}

#[tauri::command]
fn repack_xml_entry_command(
    virtual_path: String,
    source_group: Option<String>,
    modified_path: String,
    output_path: Option<String>,
    state: State<'_, AppState>,
) -> Result<XmlRepackResult, ErrorPayload> {
    let _guard = state.operation_lock.lock().map_err(|_| ErrorPayload {
        message: "Operation lock poisoned".to_string(),
    })?;
    let mut marker = OperationMarkerGuard::new(&state, "repack_xml_entry").map_err(ErrorPayload::from)?;
    let connection = state.connection().map_err(ErrorPayload::from)?;
    let packages_dir = saved_game_path(&connection)
        .map_err(ErrorPayload::from)?
        .ok_or_else(|| ErrorPayload {
            message: "Set the Crimson Desert game path first.".to_string(),
        })?;
    let result = patcher::repack_xml_entry(
        &packages_dir,
        &virtual_path,
        source_group.as_deref(),
        Path::new(&modified_path),
        output_path.as_deref().map(Path::new),
    )
    .map_err(ErrorPayload::from)?;
    insert_history(
        &connection,
        "repack_xml_entry",
        "ok",
        &format!("Repacked XML {} ({} -> {} bytes)", result.virtual_path, result.target_comp_size, result.new_comp_size),
        None,
    )
    .map_err(ErrorPayload::from)?;
    marker.clear().map_err(ErrorPayload::from)?;
    Ok(result)
}

#[tauri::command]
fn get_history_command(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<HistoryEntry>, ErrorPayload> {
    let connection = state.connection().map_err(ErrorPayload::from)?;
    list_history(&connection, limit.unwrap_or(50)).map_err(ErrorPayload::from)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = app_state(&app.handle())
                .map_err(|err| -> Box<dyn std::error::Error> { Box::new(err) })?;
            state.connection()?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_dashboard,
            detect_game_install_command,
            set_game_install_command,
            scan_mod_folder_command,
            import_mod_variant_command,
            set_mod_enabled_command,
            set_selected_language_command,
            set_mod_classification_command,
            move_mod_in_load_order_command,
            get_mod_patch_summaries_command,
            set_patch_enabled_command,
            get_apply_preview_command,
            apply_mods_command,
            restore_vanilla_command,
            reset_active_mods_command,
            launch_game_command,
            get_pathc_summary_command,
            repack_pathc_command,
            fix_everything_command,
            get_virtual_file_preview_command,
            extract_virtual_file_command,
            get_history_command,
            search_virtual_files_command,
            extract_xml_entry_command,
            repack_xml_entry_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
