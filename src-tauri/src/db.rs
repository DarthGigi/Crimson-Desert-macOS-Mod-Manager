use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::{AppError, AppResult};
use crate::models::ModRecord;
use crate::util::{bool_to_int, int_to_bool, now_iso_string};

pub const DATABASE_NAME: &str = "app.db";

pub fn database_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(DATABASE_NAME)
}

pub fn ensure_app_dirs(app_data_dir: &Path) -> AppResult<()> {
    fs::create_dir_all(app_data_dir)?;
    fs::create_dir_all(app_data_dir.join("mods").join("library"))?;
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
		   name TEXT NOT NULL,
		   description TEXT,
		   file_name TEXT NOT NULL,
		   source_path TEXT,
		   library_path TEXT NOT NULL UNIQUE,
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
		 );",
    )?;
    Ok(connection)
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

pub fn upsert_mod(connection: &Connection, record: &ModRecord) -> AppResult<()> {
    let target_files = serde_json::to_string(&record.target_files)?;
    connection.execute(
        "INSERT INTO mods(
		  id, name, description, file_name, source_path, library_path,
		  patch_count, change_count, target_files_json, enabled, imported_at, updated_at
		) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
		ON CONFLICT(id) DO UPDATE SET
		  name = excluded.name,
		  description = excluded.description,
		  file_name = excluded.file_name,
		  source_path = excluded.source_path,
		  library_path = excluded.library_path,
		  patch_count = excluded.patch_count,
		  change_count = excluded.change_count,
		  target_files_json = excluded.target_files_json,
		  enabled = excluded.enabled,
		  updated_at = excluded.updated_at",
        params![
            record.id,
            record.name,
            record.description,
            record.file_name,
            record.source_path,
            record.library_path,
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

pub fn disable_all_mods(connection: &Connection) -> AppResult<()> {
    connection.execute(
        "UPDATE mods SET enabled = 0, updated_at = ?1 WHERE enabled = 1",
        params![now_iso_string()],
    )?;
    Ok(())
}

pub fn list_mods(connection: &Connection) -> AppResult<Vec<ModRecord>> {
    let mut statement = connection.prepare(
        "SELECT id, name, description, file_name, source_path, library_path, enabled,
		        patch_count, change_count, target_files_json, imported_at, updated_at
		 FROM mods
		 ORDER BY imported_at DESC, file_name ASC",
    )?;

    let rows = statement.query_map([], |row| {
        let target_files_json: String = row.get(9)?;
        let target_files: Vec<String> =
            serde_json::from_str(&target_files_json).map_err(|err| {
                rusqlite::Error::FromSqlConversionFailure(
                    9,
                    rusqlite::types::Type::Text,
                    Box::new(err),
                )
            })?;

        Ok(ModRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            file_name: row.get(3)?,
            source_path: row.get(4)?,
            library_path: row.get(5)?,
            enabled: int_to_bool(row.get::<_, i64>(6)?),
            patch_count: row.get::<_, i64>(7)? as usize,
            change_count: row.get::<_, i64>(8)? as usize,
            target_files,
            imported_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    })?;

    let mut mods = Vec::new();
    for row in rows {
        mods.push(row?);
    }

    Ok(mods)
}
