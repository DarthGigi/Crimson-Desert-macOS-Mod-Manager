use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{ModManifest, ModRecord, ScanResult};
use crate::patcher::{build_file_index, read_pamt_raw, resolve_game_file};
use crate::util::{now_iso_string, sanitize_file_name, unique_id};

pub fn scan_mod_folder(folder: &Path, packages_dir: Option<&Path>) -> AppResult<Vec<ScanResult>> {
    if !folder.is_dir() {
        return Err(AppError::NotFound(format!(
            "Folder not found: {}",
            folder.display()
        )));
    }

    let mut files = Vec::new();
    collect_mod_files(folder, &mut files)?;
    files.sort();

    let indexes = if let Some(packages_dir) = packages_dir {
        let pamt_info = read_pamt_raw(&packages_dir.join("0008").join("0.pamt"))?;
        Some(build_file_index(&pamt_info))
    } else {
        None
    };

    let mut results = Vec::new();
    for path in files {
        let manifest = load_manifest(&path)?;
        let target_files = target_files(&manifest);
        let (resolvable_files, missing_files) =
            if let Some((ref simple_index, ref full_index)) = indexes {
                let mut resolved = 0;
                let mut missing = Vec::new();
                for game_file in &target_files {
                    if resolve_game_file(game_file, simple_index, full_index).is_some() {
                        resolved += 1;
                    } else {
                        missing.push(game_file.clone());
                    }
                }
                (resolved, missing)
            } else {
                (0, Vec::new())
            };

        results.push(ScanResult {
            path: path.display().to_string(),
            file_name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            name: manifest.name,
            description: manifest.description,
            patch_count: manifest.patches.len(),
            change_count: manifest
                .patches
                .iter()
                .map(|patch| patch.changes.len())
                .sum(),
            target_files,
            resolvable_files,
            missing_files,
        });
    }

    Ok(results)
}

pub fn import_mod(
    app_data_dir: &Path,
    connection: &rusqlite::Connection,
    source_path: &Path,
    enable: bool,
) -> AppResult<ModRecord> {
    let manifest = load_manifest(source_path)?;
    let description = manifest.description.clone();
    let patch_count = manifest.patches.len();
    let change_count = manifest
        .patches
        .iter()
        .map(|patch| patch.changes.len())
        .sum();
    let target_files = target_files(&manifest);
    let mod_id = unique_id("mod");
    let file_name = source_path
        .file_name()
        .ok_or_else(|| AppError::InvalidMod("Missing file name".to_string()))?
        .to_string_lossy()
        .to_string();
    let stored_name = format!("{}_{}", mod_id, sanitize_file_name(&file_name));
    let library_path = app_data_dir.join("mods").join("library").join(stored_name);
    fs::copy(source_path, &library_path)?;

    let now = now_iso_string();
    let record = ModRecord {
        id: mod_id,
        name: manifest.name,
        description,
        file_name,
        source_path: Some(source_path.display().to_string()),
        library_path: library_path.display().to_string(),
        enabled: enable,
        patch_count,
        change_count,
        target_files,
        imported_at: now.clone(),
        updated_at: now,
    };

    db::upsert_mod(connection, &record)?;
    Ok(record)
}

pub fn load_manifest(path: &Path) -> AppResult<ModManifest> {
    let raw = fs::read_to_string(path)?;
    let manifest: ModManifest = serde_json::from_str(raw.trim_start_matches('\u{feff}'))?;

    if manifest.patches.is_empty() {
        return Err(AppError::InvalidMod(format!(
            "{} does not contain any patches",
            path.display()
        )));
    }

    Ok(manifest)
}

pub fn load_enabled_manifests(records: &[ModRecord]) -> AppResult<Vec<(ModRecord, ModManifest)>> {
    let mut mods = Vec::new();
    for record in records.iter().filter(|record| record.enabled) {
        let manifest = load_manifest(Path::new(&record.library_path))?;
        mods.push((record.clone(), manifest));
    }
    Ok(mods)
}

pub fn target_files(manifest: &ModManifest) -> Vec<String> {
    let mut files = BTreeSet::new();
    for patch in &manifest.patches {
        files.insert(patch.game_file.clone());
    }
    files.into_iter().collect()
}

fn collect_mod_files(root: &Path, output: &mut Vec<PathBuf>) -> AppResult<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_mod_files(&path, output)?;
            continue;
        }

        let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
            continue;
        };

        if !(extension.eq_ignore_ascii_case("json") || extension.eq_ignore_ascii_case("modpatch")) {
            continue;
        }

        if path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.starts_with('_'))
        {
            continue;
        }

        output.push(path);
    }

    Ok(())
}

pub fn merged_changes(
    enabled_mods: &[(ModRecord, ModManifest)],
) -> BTreeMap<String, Vec<(String, crate::models::ModChange)>> {
    let mut merged = BTreeMap::new();
    for (record, manifest) in enabled_mods {
        for patch in &manifest.patches {
            let entry = merged
                .entry(patch.game_file.clone())
                .or_insert_with(Vec::new);
            for change in &patch.changes {
                entry.push((record.name.clone(), change.clone()));
            }
        }
    }
    merged
}
