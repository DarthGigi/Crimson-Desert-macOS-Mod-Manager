use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::{AppError, AppResult};
use crate::models::{HistoryEntry, ManagedGroupRecord, ModKind, ModRecord};
use crate::util::{bool_to_int, int_to_bool, now_iso_string};

pub const DATABASE_NAME: &str = "app.db";

pub fn database_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(DATABASE_NAME)
}

pub fn ensure_app_dirs(app_data_dir: &Path) -> AppResult<()> {
    fs::create_dir_all(app_data_dir)?;
    fs::create_dir_all(app_data_dir.join("mods").join("library"))?;
    fs::create_dir_all(app_data_dir.join("mods").join("import-cache"))?;
    Ok(())
}

pub fn connect(app_data_dir: &Path) -> AppResult<Connection> {
    ensure_app_dirs(app_data_dir)?;
    let connection = Connection::open(database_path(app_data_dir))?;
    connection.execute_batch(
        "PRAGMA journal_mode = WAL;
		 PRAGMA foreign_keys = ON;

		 CREATE TABLE IF NOT EXISTS settings (
		   key TEXT PRIMARY KEY,
		   value TEXT NOT NULL
		 );

		 CREATE TABLE IF NOT EXISTS mods (
		   id TEXT PRIMARY KEY,
		   mod_kind TEXT NOT NULL DEFAULT 'json_data',
		   name TEXT NOT NULL,
		   description TEXT,
		   file_name TEXT NOT NULL,
		   source_path TEXT,
		   library_path TEXT NOT NULL UNIQUE,
		   load_order INTEGER NOT NULL DEFAULT 0,
		   language TEXT,
		   install_group TEXT,
		   patch_count INTEGER NOT NULL,
		   change_count INTEGER NOT NULL,
		   target_files_json TEXT NOT NULL,
		   enabled INTEGER NOT NULL DEFAULT 0,
		   imported_at TEXT NOT NULL,
		   updated_at TEXT NOT NULL
		 );

		 CREATE TABLE IF NOT EXISTS history (
		   id INTEGER PRIMARY KEY AUTOINCREMENT,
		   action TEXT NOT NULL,
		   status TEXT NOT NULL,
		   message TEXT NOT NULL,
		   details_json TEXT,
		   created_at TEXT NOT NULL
		 );

         CREATE TABLE IF NOT EXISTS managed_groups (
		   group_name TEXT PRIMARY KEY,
		   purpose TEXT NOT NULL,
		   source_mod_id TEXT,
		   created_at TEXT NOT NULL
		 );

		 CREATE TABLE IF NOT EXISTS patch_toggles (
		   mod_id TEXT NOT NULL,
		   patch_index INTEGER NOT NULL,
		   enabled INTEGER NOT NULL,
		   updated_at TEXT NOT NULL,
		   PRIMARY KEY (mod_id, patch_index)
		 );",
    )?;

    ensure_column(
        &connection,
        "mods",
        "mod_kind",
        "TEXT NOT NULL DEFAULT 'json_data'",
    )?;
    ensure_column(
        &connection,
        "mods",
        "load_order",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column(&connection, "mods", "language", "TEXT")?;
    ensure_column(&connection, "mods", "install_group", "TEXT")?;

    Ok(connection)
}

fn ensure_column(
    connection: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> AppResult<()> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
    let columns = statement.query_map([], |row| row.get::<_, String>(1))?;

    for existing in columns {
        if existing? == column {
            return Ok(());
        }
    }

    connection.execute(
        &format!("ALTER TABLE {table} ADD COLUMN {column} {definition}"),
        [],
    )?;
    Ok(())
}

pub fn set_setting(connection: &Connection, key: &str, value: &str) -> AppResult<()> {
    connection.execute(
        "INSERT INTO settings(key, value) VALUES (?1, ?2)
		 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn get_setting(connection: &Connection, key: &str) -> AppResult<Option<String>> {
    let value = connection
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()?;
    Ok(value)
}

pub fn insert_history(
    connection: &Connection,
    action: &str,
    status: &str,
    message: &str,
    details_json: Option<&str>,
) -> AppResult<()> {
    connection.execute(
        "INSERT INTO history(action, status, message, details_json, created_at)
		 VALUES (?1, ?2, ?3, ?4, ?5)",
        params![action, status, message, details_json, now_iso_string()],
    )?;
    Ok(())
}

pub fn list_history(connection: &Connection, limit: usize) -> AppResult<Vec<HistoryEntry>> {
    let mut statement = connection.prepare(
        "SELECT id, action, status, message, details_json, created_at
         FROM history
         ORDER BY id DESC
         LIMIT ?1",
    )?;

    let rows = statement.query_map(params![limit as i64], |row| {
        Ok(HistoryEntry {
            id: row.get(0)?,
            action: row.get(1)?,
            status: row.get(2)?,
            message: row.get(3)?,
            details_json: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }

    Ok(entries)
}

pub fn upsert_mod(connection: &Connection, record: &ModRecord) -> AppResult<()> {
    let target_files = serde_json::to_string(&record.target_files)?;
    connection.execute(
        "INSERT INTO mods(
		  id, mod_kind, name, description, file_name, source_path, library_path,
		  load_order, language, install_group, patch_count, change_count,
		  target_files_json, enabled, imported_at, updated_at
		) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
		ON CONFLICT(id) DO UPDATE SET
		  mod_kind = excluded.mod_kind,
		  name = excluded.name,
		  description = excluded.description,
		  file_name = excluded.file_name,
		  source_path = excluded.source_path,
		  library_path = excluded.library_path,
		  load_order = excluded.load_order,
		  language = excluded.language,
		  install_group = excluded.install_group,
		  patch_count = excluded.patch_count,
		  change_count = excluded.change_count,
		  target_files_json = excluded.target_files_json,
		  enabled = excluded.enabled,
		  updated_at = excluded.updated_at",
        params![
            record.id,
            record.mod_kind.as_str(),
            record.name,
            record.description,
            record.file_name,
            record.source_path,
            record.library_path,
            record.load_order,
            record.language,
            record.install_group,
            record.patch_count as i64,
            record.change_count as i64,
            target_files,
            bool_to_int(record.enabled),
            record.imported_at,
            record.updated_at,
        ],
    )?;
    Ok(())
}

pub fn next_load_order(connection: &Connection) -> AppResult<i64> {
    let max: Option<i64> = connection
        .query_row("SELECT MAX(load_order) FROM mods", [], |row| row.get(0))
        .optional()?
        .flatten();
    Ok(max.unwrap_or(-1) + 1)
}

pub fn list_managed_groups(connection: &Connection) -> AppResult<Vec<ManagedGroupRecord>> {
    let mut statement = connection.prepare(
        "SELECT group_name, purpose, source_mod_id, created_at
         FROM managed_groups
         ORDER BY group_name ASC",
    )?;

    let rows = statement.query_map([], |row| {
        Ok(ManagedGroupRecord {
            group_name: row.get(0)?,
            purpose: row.get(1)?,
            source_mod_id: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;

    let mut groups = Vec::new();
    for row in rows {
        groups.push(row?);
    }

    Ok(groups)
}

pub fn replace_managed_groups(
    connection: &mut Connection,
    groups: &[ManagedGroupRecord],
) -> AppResult<()> {
    let transaction = connection.transaction()?;
    transaction.execute("DELETE FROM managed_groups", [])?;

    for group in groups {
        transaction.execute(
            "INSERT INTO managed_groups(group_name, purpose, source_mod_id, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                group.group_name,
                group.purpose,
                group.source_mod_id,
                group.created_at,
            ],
        )?;
    }

    transaction.commit()?;
    Ok(())
}

pub fn clear_managed_groups(connection: &Connection) -> AppResult<()> {
    connection.execute("DELETE FROM managed_groups", [])?;
    Ok(())
}

pub fn list_disabled_patch_indexes(
    connection: &Connection,
) -> AppResult<BTreeMap<String, BTreeSet<usize>>> {
    let mut statement = connection.prepare(
        "SELECT mod_id, patch_index
         FROM patch_toggles
         WHERE enabled = 0
         ORDER BY mod_id ASC, patch_index ASC",
    )?;

    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
    })?;

    let mut disabled = BTreeMap::<String, BTreeSet<usize>>::new();
    for row in rows {
        let (mod_id, patch_index) = row?;
        disabled.entry(mod_id).or_default().insert(patch_index);
    }

    Ok(disabled)
}

pub fn set_patch_enabled(
    connection: &Connection,
    mod_id: &str,
    patch_index: usize,
    enabled: bool,
) -> AppResult<()> {
    if enabled {
        connection.execute(
            "DELETE FROM patch_toggles WHERE mod_id = ?1 AND patch_index = ?2",
            params![mod_id, patch_index as i64],
        )?;
    } else {
        connection.execute(
            "INSERT INTO patch_toggles(mod_id, patch_index, enabled, updated_at)
             VALUES (?1, ?2, 0, ?3)
             ON CONFLICT(mod_id, patch_index) DO UPDATE SET
               enabled = 0,
               updated_at = excluded.updated_at",
            params![mod_id, patch_index as i64, now_iso_string()],
        )?;
    }

    Ok(())
}

pub fn clear_patch_toggles(connection: &Connection) -> AppResult<()> {
    connection.execute("DELETE FROM patch_toggles", [])?;
    Ok(())
}

pub fn update_mod_enabled(connection: &Connection, mod_id: &str, enabled: bool) -> AppResult<()> {
    let updated = connection.execute(
        "UPDATE mods SET enabled = ?2, updated_at = ?3 WHERE id = ?1",
        params![mod_id, bool_to_int(enabled), now_iso_string()],
    )?;

    if updated == 0 {
        return Err(AppError::NotFound(format!("No mod found for id {mod_id}")));
    }

    Ok(())
}

pub fn update_mod_classification(
    connection: &Connection,
    mod_id: &str,
    mod_kind: ModKind,
    language: Option<&str>,
) -> AppResult<()> {
    let updated = connection.execute(
        "UPDATE mods
         SET mod_kind = ?2, language = ?3, updated_at = ?4
         WHERE id = ?1",
        params![mod_id, mod_kind.as_str(), language, now_iso_string()],
    )?;

    if updated == 0 {
        return Err(AppError::NotFound(format!("No mod found for id {mod_id}")));
    }

    Ok(())
}

pub fn move_mod_in_load_order(
    connection: &mut Connection,
    mod_id: &str,
    direction: &str,
) -> AppResult<()> {
    let mods = list_mods(connection)?;
    let mut ordered: Vec<ModRecord> = mods
        .into_iter()
        .filter(|record| record.enabled && record.mod_kind == ModKind::JsonData)
        .collect();

    let Some(index) = ordered.iter().position(|record| record.id == mod_id) else {
        return Err(AppError::NotFound(format!(
            "No enabled JSON mod found for load-order move: {mod_id}"
        )));
    };

    let swap_index = match direction {
        "up" if index > 0 => index - 1,
        "down" if index + 1 < ordered.len() => index + 1,
        "up" | "down" => return Ok(()),
        _ => {
            return Err(AppError::Other(format!(
                "Unsupported load-order direction: {direction}"
            )))
        }
    };

    ordered.swap(index, swap_index);

    let transaction = connection.transaction()?;
    for (position, record) in ordered.iter().enumerate() {
        transaction.execute(
            "UPDATE mods SET load_order = ?2, updated_at = ?3 WHERE id = ?1",
            params![record.id, position as i64, now_iso_string()],
        )?;
    }
    transaction.commit()?;

    Ok(())
}

pub fn disable_all_mods(connection: &Connection) -> AppResult<()> {
    connection.execute(
        "UPDATE mods SET enabled = 0, updated_at = ?1 WHERE enabled = 1",
        params![now_iso_string()],
    )?;
    Ok(())
}

pub fn list_mods(connection: &Connection) -> AppResult<Vec<ModRecord>> {
    let mut statement = connection.prepare(
        "SELECT id, mod_kind, name, description, file_name, source_path, library_path,
		        enabled, load_order, language, install_group,
		        patch_count, change_count, target_files_json, imported_at, updated_at
		 FROM mods
		 ORDER BY enabled DESC, load_order ASC, imported_at DESC, file_name ASC",
    )?;

    let rows = statement.query_map([], |row| {
        let target_files_json: String = row.get(13)?;
        let target_files: Vec<String> =
            serde_json::from_str(&target_files_json).map_err(|err| {
                rusqlite::Error::FromSqlConversionFailure(
                    13,
                    rusqlite::types::Type::Text,
                    Box::new(err),
                )
            })?;

        let mod_kind_raw: String = row.get(1)?;
        let mod_kind = ModKind::from_str(&mod_kind_raw).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Text,
                Box::new(AppError::Other(format!("Unknown mod kind: {mod_kind_raw}"))),
            )
        })?;

        Ok(ModRecord {
            id: row.get(0)?,
            mod_kind,
            name: row.get(2)?,
            description: row.get(3)?,
            file_name: row.get(4)?,
            source_path: row.get(5)?,
            library_path: row.get(6)?,
            enabled: int_to_bool(row.get::<_, i64>(7)?),
            load_order: row.get(8)?,
            language: row.get(9)?,
            install_group: row.get(10)?,
            patch_count: row.get::<_, i64>(11)? as usize,
            change_count: row.get::<_, i64>(12)? as usize,
            target_files,
            imported_at: row.get(14)?,
            updated_at: row.get(15)?,
        })
    })?;

    let mut mods = Vec::new();
    for row in rows {
        mods.push(row?);
    }

    Ok(mods)
}
