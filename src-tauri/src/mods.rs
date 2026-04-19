use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{ModChange, ModKind, ModManifest, ModPatch, ModRecord, ScanResult};
use crate::patcher::{build_file_index, read_pamt_raw, resolve_game_file};
use crate::util::{now_iso_string, sanitize_file_name, unique_id};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PatchTarget {
    pub source_group: String,
    pub game_file: String,
}

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

    let mut precompiled_dirs = Vec::new();
    collect_precompiled_dirs(folder, &mut precompiled_dirs)?;
    precompiled_dirs.sort();

    let mut browser_raw_dirs = Vec::new();
    collect_browser_raw_dirs(folder, &mut browser_raw_dirs)?;
    browser_raw_dirs.sort();

    let indexes = if let Some(packages_dir) = packages_dir {
        let pamt_info = read_pamt_raw(&packages_dir.join("0008").join("0.pamt"))?;
        Some(build_file_index(&pamt_info))
    } else {
        None
    };

    let mut results = Vec::new();
    for path in files {
        let manifest = match load_manifest(&path) {
            Ok(manifest) => manifest,
            Err(AppError::InvalidMod(_)) => continue,
            Err(err) => return Err(err),
        };
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
            mod_kind: ModKind::JsonData,
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

    for dir in precompiled_dirs {
        let record = inspect_precompiled_dir(&dir)?;
        results.push(ScanResult {
            path: dir.display().to_string(),
            mod_kind: ModKind::PrecompiledOverlay,
            file_name: dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            name: record.name,
            description: record.description,
            patch_count: record.patch_count,
            change_count: record.change_count,
            target_files: record.target_files,
            resolvable_files: 0,
            missing_files: Vec::new(),
        });
    }

    for dir in browser_raw_dirs {
        let record = inspect_browser_raw_dir(&dir)?;
        results.push(ScanResult {
            path: dir.display().to_string(),
            mod_kind: ModKind::BrowserRaw,
            file_name: dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            name: record.name,
            description: record.description,
            patch_count: record.patch_count,
            change_count: record.change_count,
            target_files: record.target_files,
            resolvable_files: 0,
            missing_files: Vec::new(),
        });
    }

    Ok(results)
}

pub fn detect_import_kind(path: &Path) -> AppResult<ModKind> {
    if path.is_dir() {
        if inspect_precompiled_dir(path).is_ok() {
            return Ok(ModKind::PrecompiledOverlay);
        }
        if inspect_browser_raw_dir(path).is_ok() {
            return Ok(ModKind::BrowserRaw);
        }
        return Err(AppError::InvalidMod(format!(
            "{} is not a supported folder mod",
            path.display()
        )));
    }

    load_manifest(path)?;
    Ok(ModKind::JsonData)
}

pub fn import_mod(
    app_data_dir: &Path,
    connection: &rusqlite::Connection,
    source_path: &Path,
    enable: bool,
    mod_kind: ModKind,
    language: Option<String>,
) -> AppResult<ModRecord> {
    let mod_id = unique_id("mod");
    let file_name = source_path
        .file_name()
        .ok_or_else(|| AppError::InvalidMod("Missing file name".to_string()))?
        .to_string_lossy()
        .to_string();
    let stored_name = format!("{}_{}", mod_id, sanitize_file_name(&file_name));
    let library_path = app_data_dir.join("mods").join("library").join(stored_name);

    let (name, description, patch_count, change_count, target_files) = match mod_kind {
        ModKind::JsonData | ModKind::Language => {
            let manifest = load_manifest(source_path)?;
            let description = manifest.description.clone();
            let patch_count = manifest.patches.len();
            let change_count = manifest
                .patches
                .iter()
                .map(|patch| patch.changes.len())
                .sum();
            let target_files = target_files(&manifest);
            fs::copy(source_path, &library_path)?;

            (
                manifest.name,
                description,
                patch_count,
                change_count,
                target_files,
            )
        }
        ModKind::PrecompiledOverlay => {
            let record = inspect_precompiled_dir(source_path)?;
            copy_dir_all(source_path, &library_path)?;
            (
                record.name,
                record.description,
                record.patch_count,
                record.change_count,
                record.target_files,
            )
        }
        ModKind::BrowserRaw => {
            let record = inspect_browser_raw_dir(source_path)?;
            copy_dir_all(source_path, &library_path)?;
            (
                record.name,
                record.description,
                record.patch_count,
                record.change_count,
                record.target_files,
            )
        }
    };

    let now = now_iso_string();
    let load_order = db::next_load_order(connection)?;
    let record = ModRecord {
        id: mod_id,
        mod_kind,
        name,
        description,
        file_name,
        source_path: Some(source_path.display().to_string()),
        library_path: library_path.display().to_string(),
        enabled: enable,
        load_order,
        language,
        install_group: None,
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
    let value: Value = serde_json::from_str(raw.trim_start_matches('\u{feff}'))?;
    let patches_value = value
        .get("patches")
        .cloned()
        .ok_or_else(|| AppError::InvalidMod(format!("{} is not a patch manifest", path.display())))?;
    let patches: Vec<ModPatch> = serde_json::from_value(patches_value)?;

    if patches.is_empty() {
        return Err(AppError::InvalidMod(format!(
            "{} does not contain any patches",
            path.display()
        )));
    }

    let modinfo = value.get("modinfo");
    let name = first_string(&[
        value.get("name"),
        value.get("title"),
        modinfo.and_then(|value| value.get("name")),
        modinfo.and_then(|value| value.get("title")),
    ])
    .unwrap_or_else(|| {
        path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });
    let description = first_string(&[
        value.get("description"),
        modinfo.and_then(|value| value.get("description")),
    ]);

    Ok(ModManifest {
        name,
        description,
        patches,
    })
}

pub fn load_enabled_manifests(
    records: &[ModRecord],
    selected_language: Option<&str>,
) -> AppResult<Vec<(ModRecord, ModManifest)>> {
    let mut mods = Vec::new();
    for record in records
        .iter()
        .filter(|record| {
            record.enabled
                && match record.mod_kind {
                    ModKind::JsonData => true,
                    ModKind::Language => {
                        record.language.as_deref() == selected_language
                            && Path::new(&record.library_path).is_file()
                    }
                    ModKind::PrecompiledOverlay | ModKind::BrowserRaw => false,
                }
        })
    {
        let manifest = load_manifest(Path::new(&record.library_path))?;
        mods.push((record.clone(), manifest));
    }
    mods.sort_by_key(|(record, _)| record.load_order);
    Ok(mods)
}

pub fn target_files(manifest: &ModManifest) -> Vec<String> {
    let mut files = BTreeSet::new();
    for patch in &manifest.patches {
        files.insert(patch.game_file.clone());
    }
    files.into_iter().collect()
}

fn first_string(values: &[Option<&Value>]) -> Option<String> {
    values
        .iter()
        .flatten()
        .find_map(|value| value.as_str().map(str::to_string))
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

fn collect_precompiled_dirs(root: &Path, output: &mut Vec<PathBuf>) -> AppResult<()> {
    if is_precompiled_dir(root) {
        output.push(root.to_path_buf());
        return Ok(());
    }

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_precompiled_dirs(&path, output)?;
        }
    }

    Ok(())
}

fn collect_browser_raw_dirs(root: &Path, output: &mut Vec<PathBuf>) -> AppResult<()> {
    if is_browser_raw_dir(root) {
        output.push(root.to_path_buf());
        return Ok(());
    }

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_browser_raw_dirs(&path, output)?;
        }
    }

    Ok(())
}

fn is_precompiled_dir(root: &Path) -> bool {
    let Ok(entries) = fs::read_dir(root) else {
        return false;
    };

    entries.flatten().any(|entry| {
        let path = entry.path();
        path.is_dir()
            && path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.len() == 4 && name.chars().all(|ch| ch.is_ascii_digit()))
            && path.join("0.pamt").is_file()
            && path.join("0.paz").is_file()
    })
}

fn is_browser_raw_dir(root: &Path) -> bool {
    let Some(files_dir) = browser_files_dir(root) else {
        return false;
    };

    let Ok(entries) = fs::read_dir(&files_dir) else {
        return false;
    };

    entries.flatten().any(|entry| {
        let path = entry.path();
        path.is_dir()
            && path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.len() == 4 && name.chars().all(|ch| ch.is_ascii_digit()))
            && contains_any_files(&path)
    })
}

fn inspect_precompiled_dir(root: &Path) -> AppResult<ModRecord> {
    if !is_precompiled_dir(root) {
        return Err(AppError::InvalidMod(format!(
            "{} is not a precompiled overlay directory",
            root.display()
        )));
    }

    let name = read_precompiled_name(root).unwrap_or_else(|| {
        root.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });
    let description = read_precompiled_description(root);
    let target_files = list_precompiled_groups(root)?;

    Ok(ModRecord {
        id: String::new(),
        mod_kind: ModKind::PrecompiledOverlay,
        name,
        description,
        file_name: root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        source_path: Some(root.display().to_string()),
        library_path: root.display().to_string(),
        enabled: false,
        load_order: 0,
        language: None,
        install_group: None,
        patch_count: target_files.len(),
        change_count: 0,
        target_files,
        imported_at: String::new(),
        updated_at: String::new(),
    })
}

fn inspect_browser_raw_dir(root: &Path) -> AppResult<ModRecord> {
    if !is_browser_raw_dir(root) {
        return Err(AppError::InvalidMod(format!(
            "{} is not a browser/raw folder mod",
            root.display()
        )));
    }

    let files_dir = browser_files_dir(root).ok_or_else(|| {
        AppError::InvalidMod(format!("{} does not declare a valid files dir", root.display()))
    })?;
    let name = read_precompiled_name(root).unwrap_or_else(|| {
        root.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });
    let description = read_precompiled_description(root);
    let (target_files, change_count) = list_browser_raw_groups(&files_dir)?;

    Ok(ModRecord {
        id: String::new(),
        mod_kind: ModKind::BrowserRaw,
        name,
        description,
        file_name: root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        source_path: Some(root.display().to_string()),
        library_path: root.display().to_string(),
        enabled: false,
        load_order: 0,
        language: None,
        install_group: None,
        patch_count: target_files.len(),
        change_count,
        target_files,
        imported_at: String::new(),
        updated_at: String::new(),
    })
}

fn read_precompiled_name(root: &Path) -> Option<String> {
    for candidate in ["modinfo.json", "manifest.json", "mod.json"] {
        let path = root.join(candidate);
        if !path.is_file() {
            continue;
        }

        let raw = fs::read_to_string(path).ok()?;
        let value: Value = serde_json::from_str(&raw).ok()?;
        let modinfo = value.get("modinfo");

        if let Some(name) = first_string(&[
            value.get("name"),
            value.get("title"),
            value.get("id"),
            modinfo.and_then(|value| value.get("name")),
            modinfo.and_then(|value| value.get("title")),
        ]) {
            return Some(name);
        }
    }

    None
}

fn browser_files_dir(root: &Path) -> Option<PathBuf> {
    for candidate in ["manifest.json", "modinfo.json", "mod.json"] {
        let path = root.join(candidate);
        if !path.is_file() {
            continue;
        }

        let raw = fs::read_to_string(&path).ok()?;
        let value: Value = serde_json::from_str(&raw).ok()?;
        let files_dir = value
            .get("files_dir")
            .and_then(|value| value.as_str())
            .unwrap_or("files");
        let candidate_dir = root.join(files_dir);
        if candidate_dir.is_dir() {
            return Some(candidate_dir);
        }
    }

    let default = root.join("files");
    default.is_dir().then_some(default)
}

fn read_precompiled_description(root: &Path) -> Option<String> {
    for candidate in ["modinfo.json", "manifest.json", "mod.json"] {
        let path = root.join(candidate);
        if !path.is_file() {
            continue;
        }

        let raw = fs::read_to_string(path).ok()?;
        let value: Value = serde_json::from_str(&raw).ok()?;
        let modinfo = value.get("modinfo");

        if let Some(description) = first_string(&[
            value.get("description"),
            modinfo.and_then(|value| value.get("description")),
        ]) {
            return Some(description);
        }
    }

    None
}

fn list_precompiled_groups(root: &Path) -> AppResult<Vec<String>> {
    let mut groups = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };

        if name.len() == 4
            && name.chars().all(|ch| ch.is_ascii_digit())
            && path.join("0.pamt").is_file()
            && path.join("0.paz").is_file()
        {
            groups.push(name.to_string());
        }
    }
    groups.sort();
    Ok(groups)
}

fn list_browser_raw_groups(root: &Path) -> AppResult<(Vec<String>, usize)> {
    let mut groups = Vec::new();
    let mut total_files = 0usize;

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };

        if name.len() == 4 && name.chars().all(|ch| ch.is_ascii_digit()) {
            let count = count_files_recursively(&path)?;
            if count > 0 {
                groups.push(name.to_string());
                total_files += count;
            }
        }
    }

    groups.sort();
    Ok((groups, total_files))
}

fn contains_any_files(root: &Path) -> bool {
    count_files_recursively(root).unwrap_or(0) > 0
}

fn count_files_recursively(root: &Path) -> AppResult<usize> {
    let mut count = 0usize;
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            count += count_files_recursively(&path)?;
        } else if path.is_file() {
            count += 1;
        }
    }

    Ok(count)
}

fn copy_dir_all(source: &Path, destination: &Path) -> AppResult<()> {
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_path = entry.path();
        let target_path = destination.join(entry.file_name());
        if entry_path.is_dir() {
            copy_dir_all(&entry_path, &target_path)?;
        } else {
            fs::copy(&entry_path, &target_path)?;
        }
    }

    Ok(())
}

pub fn merged_changes(
    enabled_mods: &[(ModRecord, ModManifest)],
) -> BTreeMap<PatchTarget, Vec<(String, ModChange)>> {
    let mut merged = BTreeMap::new();
    for (record, manifest) in enabled_mods {
        for patch in &manifest.patches {
            let target = PatchTarget {
                source_group: patch
                    .source_group
                    .clone()
                    .unwrap_or_else(|| "0008".to_string()),
                game_file: patch.game_file.clone(),
            };
            let entry = merged
                .entry(target)
                .or_insert_with(Vec::new);
            for change in &patch.changes {
                entry.push((format!("{}#{}", record.name, record.load_order), change.clone()));
            }
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOWNLOADED_MODS_DIR: &str = "/Users/gigi/Downloads/CD Mods";

    #[test]
    fn detects_real_downloaded_mod_kinds() {
        let json_mod = Path::new(DOWNLOADED_MODS_DIR)
            .join("stamina_json_v1.02.00")
            .join("stamina_v1.02.00_infinite.json");
        let precompiled_mod = Path::new(DOWNLOADED_MODS_DIR).join("item_price_display");
        let browser_raw_mod = Path::new(DOWNLOADED_MODS_DIR).join("Better_Inventory_UI_Compatible");

        assert_eq!(detect_import_kind(&json_mod).unwrap(), ModKind::JsonData);
        assert_eq!(detect_import_kind(&precompiled_mod).unwrap(), ModKind::PrecompiledOverlay);
        assert_eq!(detect_import_kind(&browser_raw_mod).unwrap(), ModKind::BrowserRaw);
    }

    #[test]
    fn scans_downloaded_mods_root_and_finds_all_supported_kinds() {
        let results = scan_mod_folder(Path::new(DOWNLOADED_MODS_DIR), None).unwrap();

        assert!(results.iter().any(|result| {
            result.mod_kind == ModKind::JsonData && result.file_name == "stamina_v1.02.00_infinite.json"
        }));
        assert!(results.iter().any(|result| {
            result.mod_kind == ModKind::PrecompiledOverlay && result.file_name == "item_price_display"
        }));
        assert!(results.iter().any(|result| {
            result.mod_kind == ModKind::BrowserRaw && result.file_name == "Better_Inventory_UI_Compatible"
        }));
    }

    #[test]
    fn browser_raw_manifest_uses_files_dir() {
        let browser_raw_mod = Path::new(DOWNLOADED_MODS_DIR).join("Better_Inventory_UI_Compatible");
        let record = inspect_browser_raw_dir(&browser_raw_mod).unwrap();

        assert_eq!(record.mod_kind, ModKind::BrowserRaw);
        assert_eq!(record.name, "Better Inventory UI compatible with BTM");
        assert!(record.target_files.iter().any(|group| group == "0012"));
        assert!(record.change_count > 0);
    }
}
