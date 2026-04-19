mod db;
mod error;
mod game;
mod models;
mod mods;
mod patcher;
mod util;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use db::{
    clear_managed_groups, connect, get_setting, insert_history, list_managed_groups, list_mods,
    list_disabled_patch_indexes, move_mod_in_load_order, replace_managed_groups, set_patch_enabled,
    set_setting, update_mod_classification, update_mod_enabled,
};
use error::{AppError, ErrorPayload};
use game::{
    detect_packages_dir, inspect_game_install, launch_game, resolve_to_packages_dir, LaunchResult,
};
use models::{ApplyPreview, ApplyResult, DashboardData, GameInstallInfo, ModKind, ModPatchSummary, ModRecord, ScanResult, StatusSummary};
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

fn build_dashboard(connection: &rusqlite::Connection) -> Result<DashboardData, AppError> {
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

    Ok(DashboardData {
        status: StatusSummary {
            game_install,
            selected_language,
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

#[tauri::command]
fn get_dashboard(state: State<'_, AppState>) -> Result<DashboardData, ErrorPayload> {
    let connection = state.connection().map_err(ErrorPayload::from)?;
    build_dashboard(&connection).map_err(ErrorPayload::from)
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
    mods::scan_mod_folder(Path::new(&folder_path), packages_dir.as_deref())
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
    build_dashboard(&connection).map_err(ErrorPayload::from)
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
    build_dashboard(&connection).map_err(ErrorPayload::from)
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
    build_dashboard(&connection).map_err(ErrorPayload::from)
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
    build_dashboard(&connection).map_err(ErrorPayload::from)
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
    build_dashboard(&connection).map_err(ErrorPayload::from)
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
    build_dashboard(&connection).map_err(ErrorPayload::from)
}

#[tauri::command]
fn apply_mods_command(state: State<'_, AppState>) -> Result<ApplyResult, ErrorPayload> {
    let _guard = state.operation_lock.lock().map_err(|_| ErrorPayload {
        message: "Operation lock poisoned".to_string(),
    })?;
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
    build_dashboard(&connection).map_err(ErrorPayload::from)
}

#[tauri::command]
fn reset_active_mods_command(state: State<'_, AppState>) -> Result<DashboardData, ErrorPayload> {
    let _guard = state.operation_lock.lock().map_err(|_| ErrorPayload {
        message: "Operation lock poisoned".to_string(),
    })?;
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
    build_dashboard(&connection).map_err(ErrorPayload::from)
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
