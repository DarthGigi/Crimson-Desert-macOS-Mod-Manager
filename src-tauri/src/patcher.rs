use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use lz4_flex::block;

use crate::error::{AppError, AppResult};
use crate::models::{ApplyFileResult, ApplyPreview, ApplyPreviewFile, ApplyResult, ManagedGroupRecord, ModChange, ModKind, ModRecord};
use crate::mods::{load_enabled_manifests, merged_changes};
use crate::util::now_iso_string;

const PA_MAGIC: u32 = 0x2145_E233;
const MASK: u32 = 0xFFFF_FFFF;
const PAZ_ALIGNMENT: usize = 16;
const PAMT_UNKNOWN: u32 = 0x610E_0232;
const PAPGT_LANG_ALL: u16 = 0x3FFF;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PazInfo {
    pub index: u32,
    pub crc: u32,
    pub file_size: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HashEntry {
    pub folder_hash: u32,
    pub name_offset: u32,
    pub file_start_index: u32,
    pub file_count: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FileRecord {
    pub name_offset: u32,
    pub paz_offset: u32,
    pub comp_size: u32,
    pub decomp_size: u32,
    pub paz_index: u16,
    pub flags: u16,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PamtInfo {
    pub raw: Vec<u8>,
    pub paz_count: u32,
    pub paz_infos: Vec<PazInfo>,
    pub dir_block_offset: usize,
    pub dir_block_size: usize,
    pub fn_block_offset: usize,
    pub fn_block_size: usize,
    pub fn_data: Vec<u8>,
    pub hash_entries: Vec<HashEntry>,
    pub file_records_offset: usize,
    pub file_records: Vec<FileRecord>,
}

#[derive(Debug, Clone)]
pub struct ResolvedGameFile {
    pub full_path: String,
    pub dir_path: String,
    pub filename: String,
    pub record: FileRecord,
}

#[derive(Debug, Clone)]
pub struct OverlayFile {
    pub dir_path: String,
    pub filename: String,
    pub comp_size: usize,
    pub decomp_size: usize,
    pub paz_offset: usize,
    pub flags: u16,
}

pub fn pa_checksum(data: &[u8]) -> u32 {
    if data.is_empty() {
        return 0;
    }

    let length = data.len() as u32;
    let mut a = length.wrapping_sub(PA_MAGIC) & MASK;
    let mut b = a;
    let mut c = a;

    let mut offset = 0usize;
    let mut remaining = data.len();

    while remaining > 12 {
        a = a.wrapping_add(read_u32_le(data, offset));
        b = b.wrapping_add(read_u32_le(data, offset + 4));
        c = c.wrapping_add(read_u32_le(data, offset + 8));

        a = a.wrapping_sub(c) ^ rotl(c, 4);
        c = c.wrapping_add(b);
        b = b.wrapping_sub(a) ^ rotl(a, 6);
        a = a.wrapping_add(c);
        c = c.wrapping_sub(b) ^ rotl(b, 8);
        b = b.wrapping_add(a);
        a = a.wrapping_sub(c) ^ rotl(c, 16);
        c = c.wrapping_add(b);
        b = b.wrapping_sub(a) ^ rotl(a, 19);
        a = a.wrapping_add(c);
        c = c.wrapping_sub(b) ^ rotl(b, 4);
        b = b.wrapping_add(a);

        offset += 12;
        remaining -= 12;
    }

    if remaining >= 12 {
        c = c.wrapping_add((data[offset + 11] as u32) << 24);
    }
    if remaining >= 11 {
        c = c.wrapping_add((data[offset + 10] as u32) << 16);
    }
    if remaining >= 10 {
        c = c.wrapping_add((data[offset + 9] as u32) << 8);
    }
    if remaining >= 9 {
        c = c.wrapping_add(data[offset + 8] as u32);
    }
    if remaining >= 8 {
        b = b.wrapping_add((data[offset + 7] as u32) << 24);
    }
    if remaining >= 7 {
        b = b.wrapping_add((data[offset + 6] as u32) << 16);
    }
    if remaining >= 6 {
        b = b.wrapping_add((data[offset + 5] as u32) << 8);
    }
    if remaining >= 5 {
        b = b.wrapping_add(data[offset + 4] as u32);
    }
    if remaining >= 4 {
        a = a.wrapping_add((data[offset + 3] as u32) << 24);
    }
    if remaining >= 3 {
        a = a.wrapping_add((data[offset + 2] as u32) << 16);
    }
    if remaining >= 2 {
        a = a.wrapping_add((data[offset + 1] as u32) << 8);
    }
    if remaining >= 1 {
        a = a.wrapping_add(data[offset] as u32);
    }

    let v82 = (b ^ c).wrapping_sub(rotl(b, 14));
    let v83 = (a ^ v82).wrapping_sub(rotl(v82, 11));
    let v84 = (v83 ^ b).wrapping_sub(rotr(v83, 7));
    let v85 = (v84 ^ v82).wrapping_sub(rotl(v84, 16));
    let v86 = rotl(v85, 4);
    let t = (v83 ^ v85).wrapping_sub(v86);
    let v87 = (t ^ v84).wrapping_sub(rotl(t, 14));

    (v87 ^ v85).wrapping_sub(rotr(v87, 8))
}

pub fn read_pamt_raw(path: &Path) -> AppResult<PamtInfo> {
    let data = fs::read(path)?;
    let paz_count = read_u32_le(&data, 4);

    let mut paz_infos = Vec::new();
    let mut offset = 12usize;
    for _ in 0..paz_count {
        paz_infos.push(PazInfo {
            index: read_u32_le(&data, offset),
            crc: read_u32_le(&data, offset + 4),
            file_size: read_u32_le(&data, offset + 8),
        });
        offset += 12;
    }

    let dir_block_offset = offset;
    let dir_block_size = read_u32_le(&data, offset) as usize;
    offset += 4 + dir_block_size;

    let fn_block_offset = offset;
    let fn_block_size = read_u32_le(&data, offset) as usize;
    let fn_data = data[offset + 4..offset + 4 + fn_block_size].to_vec();
    offset += 4 + fn_block_size;

    let hash_count = read_u32_le(&data, offset) as usize;
    offset += 4;
    let mut hash_entries = Vec::new();
    for _ in 0..hash_count {
        hash_entries.push(HashEntry {
            folder_hash: read_u32_le(&data, offset),
            name_offset: read_u32_le(&data, offset + 4),
            file_start_index: read_u32_le(&data, offset + 8),
            file_count: read_u32_le(&data, offset + 12),
        });
        offset += 16;
    }

    let file_records_offset = offset;
    let file_count = read_u32_le(&data, offset) as usize;
    offset += 4;
    let mut file_records = Vec::new();
    for _ in 0..file_count {
        file_records.push(FileRecord {
            name_offset: read_u32_le(&data, offset),
            paz_offset: read_u32_le(&data, offset + 4),
            comp_size: read_u32_le(&data, offset + 8),
            decomp_size: read_u32_le(&data, offset + 12),
            paz_index: read_u16_le(&data, offset + 16),
            flags: read_u16_le(&data, offset + 18),
        });
        offset += 20;
    }

    Ok(PamtInfo {
        raw: data,
        paz_count,
        paz_infos,
        dir_block_offset,
        dir_block_size,
        fn_block_offset,
        fn_block_size,
        fn_data,
        hash_entries,
        file_records_offset,
        file_records,
    })
}

pub fn resolve_filename(fn_data: &[u8], name_offset: u32) -> String {
    resolve_path_block(fn_data, name_offset)
}

pub fn resolve_dirname(dir_data: &[u8], name_offset: u32) -> String {
    resolve_path_block(dir_data, name_offset)
}

pub fn build_file_index(
    pamt_info: &PamtInfo,
) -> (
    BTreeMap<String, ResolvedGameFile>,
    BTreeMap<String, ResolvedGameFile>,
) {
    let dir_data = &pamt_info.raw
        [pamt_info.dir_block_offset + 4..pamt_info.dir_block_offset + 4 + pamt_info.dir_block_size];
    let mut simplified = BTreeMap::new();
    let mut full = BTreeMap::new();

    for hash_entry in &pamt_info.hash_entries {
        let dir_path = resolve_dirname(dir_data, hash_entry.name_offset);
        for index in hash_entry.file_start_index as usize
            ..(hash_entry.file_start_index + hash_entry.file_count) as usize
        {
            let record = pamt_info.file_records[index].clone();
            let filename = resolve_filename(&pamt_info.fn_data, record.name_offset);
            let full_path = if dir_path.is_empty() {
                filename.clone()
            } else {
                format!("{dir_path}/{filename}")
            };

            let info = ResolvedGameFile {
                full_path: full_path.clone(),
                dir_path: dir_path.clone(),
                filename: filename.clone(),
                record,
            };

            full.insert(full_path.clone(), info.clone());

            let simplified_path = dir_path
                .split('/')
                .next()
                .filter(|segment| !segment.is_empty())
                .map(|segment| format!("{segment}/{filename}"))
                .unwrap_or(filename);
            simplified.entry(simplified_path).or_insert(info);
        }
    }

    (simplified, full)
}

pub fn resolve_game_file<'a>(
    game_file: &str,
    simplified: &'a BTreeMap<String, ResolvedGameFile>,
    full: &'a BTreeMap<String, ResolvedGameFile>,
) -> Option<&'a ResolvedGameFile> {
    full.get(game_file).or_else(|| simplified.get(game_file))
}

pub fn apply_mods(
    game_dir: &Path,
    records: &[ModRecord],
    managed_groups: &[ManagedGroupRecord],
    selected_language: Option<&str>,
    disabled_patches: &BTreeMap<String, BTreeSet<usize>>,
) -> AppResult<ApplyResult> {
    let enabled_manifests = load_enabled_manifests(records, selected_language, disabled_patches)?;
    let enabled_precompiled: Vec<ModRecord> = records
        .iter()
        .filter(|record| {
            let is_dir_backed = Path::new(&record.library_path).is_dir();
            record.enabled
                && match record.mod_kind {
                    ModKind::PrecompiledOverlay | ModKind::BrowserRaw => is_dir_backed,
                    ModKind::Language => is_dir_backed && record.language.as_deref() == selected_language,
                    ModKind::JsonData => false,
                }
        })
        .cloned()
        .collect();

    if enabled_manifests.is_empty() && enabled_precompiled.is_empty() {
        return Err(AppError::Patch(
            "No enabled mods are available. Import a mod and enable it before applying."
                .to_string(),
        ));
    }

    let merged = merged_changes(&enabled_manifests);

    let papgt_path = game_dir.join("meta").join("0.papgt");
    let papgt_backup = game_dir.join("meta").join("0.papgt.bak");
    restore_base_papgt(&papgt_path, &papgt_backup)?;
    cleanup_managed_groups(game_dir, managed_groups)?;

    let mut next_group = next_dynamic_group_number(game_dir)?;
    let mut created_groups = Vec::new();

    let mut paz_buffer = Vec::new();
    let mut overlay_files = Vec::new();
    let mut file_results = Vec::new();
    let mut json_pamt_size = 0usize;

    let mut file_index_cache = BTreeMap::new();

    for (target, changes) in merged {
        let (simple_index, full_index) = load_group_indexes(game_dir, &target.source_group, &mut file_index_cache)?;
        let Some(info) = resolve_game_file(&target.game_file, simple_index, full_index) else {
            file_results.push(ApplyFileResult {
                game_file: target.game_file,
                source_group: target.source_group,
                source_paz_index: 0,
                applied_changes: 0,
                skipped_changes: 0,
                overlap_count: 0,
                reason: Some("Target file not found in the declared source group".to_string()),
            });
            continue;
        };

        let record = &info.record;
        let src_paz = game_dir
            .join(&target.source_group)
            .join(format!("{}.paz", record.paz_index));
        let compressed = read_paz_slice(
            &src_paz,
            record.paz_offset as usize,
            record.comp_size as usize,
        )?;
        let mut buffer = block::decompress(&compressed, record.decomp_size as usize)?;

        let mut applied = 0usize;
        let mut skipped = 0usize;
        let overlap_count = count_overlaps(&changes);
        for (mod_name, change) in changes {
            match apply_change(&mut buffer, &mod_name, &change) {
                Ok(true) => applied += 1,
                Ok(false) => skipped += 1,
                Err(err) => return Err(err),
            }
        }

        let recompressed = block::compress(&buffer);
        let paz_offset = paz_buffer.len();
        paz_buffer.extend_from_slice(&recompressed);
        while paz_buffer.len() % PAZ_ALIGNMENT != 0 {
            paz_buffer.push(0);
        }

        overlay_files.push(OverlayFile {
            dir_path: info.dir_path.clone(),
            filename: info.filename.clone(),
            comp_size: recompressed.len(),
            decomp_size: record.decomp_size as usize,
            paz_offset,
            flags: 0x0002,
        });

        file_results.push(ApplyFileResult {
            game_file: info.full_path.clone(),
            source_group: target.source_group,
            source_paz_index: record.paz_index,
            applied_changes: applied,
            skipped_changes: skipped,
            overlap_count,
            reason: None,
        });
    }

    if overlay_files.is_empty() && enabled_precompiled.is_empty() {
        return Err(AppError::Patch(
			"No target files could be patched. Check mod compatibility with the current game build."
				.to_string(),
		));
    }

    if !overlay_files.is_empty() {
        let group_name = format!("{:04}", next_group);
        next_group += 1;
        let overlay_dir = game_dir.join(&group_name);
        fs::create_dir_all(&overlay_dir)?;
        let paz_path = overlay_dir.join("0.paz");
        fs::write(&paz_path, &paz_buffer)?;
        let paz_crc = pa_checksum(&paz_buffer);

        let mut pamt_data = build_multi_pamt(&overlay_files, paz_buffer.len());
        json_pamt_size = pamt_data.len();
        update_pamt_paz_crc(&mut pamt_data, paz_crc);
        let pamt_crc = read_u32_le(&pamt_data, 0);
        let pamt_path = overlay_dir.join("0.pamt");
        fs::write(&pamt_path, &pamt_data)?;

        let new_papgt = build_papgt_with_mod(&papgt_path, &group_name, pamt_crc)?;
        fs::write(&papgt_path, &new_papgt)?;
        created_groups.push(group_name);
    }

    for record in &enabled_precompiled {
        let installed = match record.mod_kind {
            ModKind::PrecompiledOverlay => {
                install_precompiled_mod(game_dir, &papgt_path, record, &mut next_group)?
            }
            ModKind::BrowserRaw => {
                install_browser_raw_mod(game_dir, &papgt_path, record, &mut next_group)?
            }
            ModKind::Language => {
                if Path::new(&record.library_path).join("files").is_dir() {
                    install_browser_raw_mod(game_dir, &papgt_path, record, &mut next_group)?
                } else {
                    install_precompiled_mod(game_dir, &papgt_path, record, &mut next_group)?
                }
            }
            ModKind::JsonData => Vec::new(),
        };
        created_groups.extend(installed);
    }

    Ok(ApplyResult {
        mod_count: enabled_manifests.len() + enabled_precompiled.len(),
        target_file_count: file_results.len(),
        overlay_file_count: overlay_files.len() + created_groups.len().saturating_sub(overlay_files.is_empty() as usize),
        paz_size: paz_buffer.len(),
        pamt_size: json_pamt_size,
        created_groups: created_groups.clone(),
        files: file_results,
        message: format!(
            "Installed {} enabled mod(s) into {} manager-owned group(s).",
            enabled_manifests.len() + enabled_precompiled.len(),
            created_groups.len(),
        ),
    })
}

pub fn preview_apply(
    game_dir: &Path,
    records: &[ModRecord],
    selected_language: Option<&str>,
    disabled_patches: &BTreeMap<String, BTreeSet<usize>>,
) -> AppResult<ApplyPreview> {
    let enabled_manifests = load_enabled_manifests(records, selected_language, disabled_patches)?;
    let enabled_dir_backed: Vec<ModRecord> = records
        .iter()
        .filter(|record| {
            let is_dir_backed = Path::new(&record.library_path).is_dir();
            record.enabled
                && match record.mod_kind {
                    ModKind::PrecompiledOverlay | ModKind::BrowserRaw => is_dir_backed,
                    ModKind::Language => is_dir_backed && record.language.as_deref() == selected_language,
                    ModKind::JsonData => false,
                }
        })
        .cloned()
        .collect();

    let merged = merged_changes(&enabled_manifests);
    let mut file_index_cache = BTreeMap::new();
    let mut files = Vec::new();

    for (target, changes) in merged {
        let source_mods: Vec<String> = changes
            .iter()
            .map(|(mod_name, _)| mod_name.split('#').next().unwrap_or(mod_name).to_string())
            .collect();
        let overlap_count = count_overlaps(&changes);

        let preview_file = match load_group_indexes(game_dir, &target.source_group, &mut file_index_cache) {
            Ok((simple_index, full_index)) => {
                match resolve_game_file(&target.game_file, simple_index, full_index) {
                    Some(info) => ApplyPreviewFile {
                        game_file: info.full_path.clone(),
                        source_group: target.source_group,
                        source_paz_index: Some(info.record.paz_index),
                        change_count: changes.len(),
                        overlap_count,
                        source_mods,
                        resolved: true,
                        reason: None,
                    },
                    None => ApplyPreviewFile {
                        game_file: target.game_file,
                        source_group: target.source_group,
                        source_paz_index: None,
                        change_count: changes.len(),
                        overlap_count,
                        source_mods,
                        resolved: false,
                        reason: Some("Target file not found in the declared source group".to_string()),
                    },
                }
            }
            Err(err) => ApplyPreviewFile {
                game_file: target.game_file,
                source_group: target.source_group,
                source_paz_index: None,
                change_count: changes.len(),
                overlap_count,
                source_mods,
                resolved: false,
                reason: Some(err.to_string()),
            },
        };

        files.push(preview_file);
    }

    files.sort_by(|left, right| {
        left.source_group
            .cmp(&right.source_group)
            .then(left.game_file.cmp(&right.game_file))
    });

    let precompiled_mod_count = enabled_dir_backed
        .iter()
        .filter(|record| record.mod_kind == ModKind::PrecompiledOverlay)
        .count();
    let browser_raw_mod_count = enabled_dir_backed
        .iter()
        .filter(|record| record.mod_kind == ModKind::BrowserRaw)
        .count();
    let estimated_group_count = usize::from(!files.is_empty()) + enabled_dir_backed.len();

    Ok(ApplyPreview {
        mod_count: enabled_manifests.len() + enabled_dir_backed.len(),
        json_mod_count: enabled_manifests.len(),
        precompiled_mod_count,
        browser_raw_mod_count,
        target_file_count: files.len(),
        estimated_group_count,
        selected_language: selected_language.map(str::to_string),
        files,
    })
}

fn load_group_indexes<'a>(
    game_dir: &Path,
    source_group: &str,
    cache: &'a mut BTreeMap<
        String,
        (
            BTreeMap<String, ResolvedGameFile>,
            BTreeMap<String, ResolvedGameFile>,
        ),
    >,
) -> AppResult<&'a (BTreeMap<String, ResolvedGameFile>, BTreeMap<String, ResolvedGameFile>)> {
    if !cache.contains_key(source_group) {
        let pamt_info = read_pamt_raw(&game_dir.join(source_group).join("0.pamt"))?;
        cache.insert(source_group.to_string(), build_file_index(&pamt_info));
    }

    cache.get(source_group).ok_or_else(|| {
        AppError::NotFound(format!("Could not load source group indexes for {source_group}"))
    })
}

fn count_overlaps(changes: &[(String, ModChange)]) -> usize {
    let mut counts = BTreeMap::<usize, usize>::new();
    for (_, change) in changes {
        *counts.entry(change.offset).or_default() += 1;
    }

    counts.values().filter(|count| **count > 1).count()
}

pub fn restore_vanilla(game_dir: &Path, managed_groups: &[ManagedGroupRecord]) -> AppResult<()> {
    let papgt_path = game_dir.join("meta").join("0.papgt");
    let papgt_backup = game_dir.join("meta").join("0.papgt.bak");

    restore_base_papgt(&papgt_path, &papgt_backup)?;
    cleanup_managed_groups(game_dir, managed_groups)?;

    Ok(())
}

pub fn managed_group_records(group_names: &[String], purpose: &str) -> Vec<ManagedGroupRecord> {
    group_names
        .iter()
        .map(|group_name| ManagedGroupRecord {
            group_name: group_name.clone(),
            purpose: purpose.to_string(),
            source_mod_id: None,
            created_at: now_iso_string(),
        })
        .collect()
}

fn restore_base_papgt(papgt_path: &Path, papgt_backup: &Path) -> AppResult<()> {
    if !papgt_backup.exists() {
        fs::copy(papgt_path, papgt_backup)?;
    } else {
        fs::copy(papgt_backup, papgt_path)?;
    }

    Ok(())
}

fn cleanup_managed_groups(game_dir: &Path, managed_groups: &[ManagedGroupRecord]) -> AppResult<()> {
    for group in managed_groups {
        let group_dir = game_dir.join(&group.group_name);
        if group_dir.is_dir() {
            fs::remove_dir_all(group_dir)?;
        }
    }

    Ok(())
}

fn next_dynamic_group_number(game_dir: &Path) -> AppResult<u16> {
    let mut max_group = 35u16;

    for entry in fs::read_dir(game_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };

        if name.len() != 4 || !name.chars().all(|ch| ch.is_ascii_digit()) {
            continue;
        }

        let value = name.parse::<u16>().map_err(|_| {
            AppError::Other(format!("Failed to parse numeric group folder: {name}"))
        })?;
        max_group = max_group.max(value);
    }

    Ok(max_group + 1)
}

fn install_precompiled_mod(
    game_dir: &Path,
    papgt_path: &Path,
    record: &ModRecord,
    next_group: &mut u16,
) -> AppResult<Vec<String>> {
    let source_root = Path::new(&record.library_path);
    let mut source_groups = Vec::new();

    for entry in fs::read_dir(source_root)? {
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
            source_groups.push(path);
        }
    }

    source_groups.sort();
    if source_groups.is_empty() {
        return Err(AppError::InvalidMod(format!(
            "{} does not contain any precompiled numeric groups",
            record.library_path
        )));
    }

    let mut created = Vec::new();
    for source_group in source_groups {
        let target_group = format!("{:04}", *next_group);
        *next_group += 1;

        let target_dir = game_dir.join(&target_group);
        copy_dir_all(&source_group, &target_dir)?;

        let pamt_bytes = fs::read(target_dir.join("0.pamt"))?;
        let pamt_crc = read_u32_le(&pamt_bytes, 0);
        let new_papgt = build_papgt_with_mod(papgt_path, &target_group, pamt_crc)?;
        fs::write(papgt_path, &new_papgt)?;
        created.push(target_group);
    }

    Ok(created)
}

fn install_browser_raw_mod(
    game_dir: &Path,
    papgt_path: &Path,
    record: &ModRecord,
    next_group: &mut u16,
) -> AppResult<Vec<String>> {
    let source_root = Path::new(&record.library_path);
    let files_dir = resolve_browser_files_dir(source_root)?;
    let mut source_groups = Vec::new();

    for entry in fs::read_dir(&files_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };

        if name.len() == 4 && name.chars().all(|ch| ch.is_ascii_digit()) {
            source_groups.push(path);
        }
    }

    source_groups.sort();
    if source_groups.is_empty() {
        return Err(AppError::InvalidMod(format!(
            "{} does not contain any browser/raw source groups",
            record.library_path
        )));
    }

    let mut created = Vec::new();
    for source_group in source_groups {
        let overlay_files = collect_loose_overlay_files(&source_group)?;
        if overlay_files.is_empty() {
            continue;
        }

        let target_group = format!("{:04}", *next_group);
        *next_group += 1;
        let target_dir = game_dir.join(&target_group);
        fs::create_dir_all(&target_dir)?;

        let (paz_bytes, pamt_bytes) = build_loose_overlay_archive(&overlay_files);
        fs::write(target_dir.join("0.paz"), &paz_bytes)?;
        fs::write(target_dir.join("0.pamt"), &pamt_bytes)?;

        let pamt_crc = read_u32_le(&pamt_bytes, 0);
        let new_papgt = build_papgt_with_mod(papgt_path, &target_group, pamt_crc)?;
        fs::write(papgt_path, &new_papgt)?;
        created.push(target_group);
    }

    Ok(created)
}

fn resolve_browser_files_dir(root: &Path) -> AppResult<std::path::PathBuf> {
    for candidate in ["manifest.json", "modinfo.json", "mod.json"] {
        let path = root.join(candidate);
        if !path.is_file() {
            continue;
        }

        let raw = fs::read_to_string(&path)?;
        let value: serde_json::Value = serde_json::from_str(&raw)?;
        let files_dir = value
            .get("files_dir")
            .and_then(|value| value.as_str())
            .unwrap_or("files");
        let candidate_dir = root.join(files_dir);
        if candidate_dir.is_dir() {
            return Ok(candidate_dir);
        }
    }

    let default = root.join("files");
    if default.is_dir() {
        return Ok(default);
    }

    Err(AppError::InvalidMod(format!(
        "{} does not contain a valid browser/raw files directory",
        root.display()
    )))
}

#[derive(Debug, Clone)]
struct OverlayFileBytes {
    dir_path: String,
    filename: String,
    bytes: Vec<u8>,
    flags: u16,
}

fn collect_loose_overlay_files(root: &Path) -> AppResult<Vec<OverlayFileBytes>> {
    let mut files = Vec::new();
    collect_loose_overlay_files_inner(root, root, &mut files)?;
    files.sort_by(|left, right| {
        left.dir_path
            .cmp(&right.dir_path)
            .then(left.filename.cmp(&right.filename))
    });
    Ok(files)
}

fn collect_loose_overlay_files_inner(
    root: &Path,
    current: &Path,
    output: &mut Vec<OverlayFileBytes>,
) -> AppResult<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_loose_overlay_files_inner(root, &path, output)?;
            continue;
        }

        let relative = path.strip_prefix(root).map_err(|err| {
            AppError::Other(format!("Failed to derive browser/raw relative path: {err}"))
        })?;
        let filename = relative
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let dir_path = relative
            .parent()
            .map(|parent| parent.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        output.push(OverlayFileBytes {
            dir_path,
            filename,
            bytes: fs::read(&path)?,
            flags: 0,
        });
    }

    Ok(())
}

fn build_loose_overlay_archive(files: &[OverlayFileBytes]) -> (Vec<u8>, Vec<u8>) {
    let mut paz_buffer = Vec::new();
    let mut overlay_files = Vec::new();

    for file in files {
        let paz_offset = paz_buffer.len();
        paz_buffer.extend_from_slice(&file.bytes);
        while paz_buffer.len() % PAZ_ALIGNMENT != 0 {
            paz_buffer.push(0);
        }

        overlay_files.push(OverlayFile {
            dir_path: file.dir_path.clone(),
            filename: file.filename.clone(),
            comp_size: file.bytes.len(),
            decomp_size: file.bytes.len(),
            paz_offset,
            flags: file.flags,
        });
    }

    let paz_crc = pa_checksum(&paz_buffer);
    let mut pamt_data = build_multi_pamt(&overlay_files, paz_buffer.len());
    update_pamt_paz_crc(&mut pamt_data, paz_crc);
    (paz_buffer, pamt_data)
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

fn apply_change(buffer: &mut [u8], mod_name: &str, change: &ModChange) -> AppResult<bool> {
    let original = decode_hex(&change.original).map_err(|err| {
        AppError::Patch(format!(
            "{mod_name}: invalid original hex at offset {} ({err})",
            change.offset
        ))
    })?;
    let patched = decode_hex(&change.patched).map_err(|err| {
        AppError::Patch(format!(
            "{mod_name}: invalid patched hex at offset {} ({err})",
            change.offset
        ))
    })?;

    if original.len() != patched.len() {
        return Ok(false);
    }

    let end = change.offset.saturating_add(original.len());
    if end > buffer.len() {
        return Ok(false);
    }

    if &buffer[change.offset..end] == original.as_slice() {
        buffer[change.offset..end].copy_from_slice(&patched);
        Ok(true)
    } else {
        Ok(false)
    }
}

fn read_paz_slice(path: &Path, offset: usize, size: usize) -> AppResult<Vec<u8>> {
    let data = fs::read(path)?;
    let end = offset.saturating_add(size);
    if end > data.len() {
        return Err(AppError::Patch(format!(
            "PAZ slice out of range: {} [{offset}..{end}]",
            path.display()
        )));
    }
    Ok(data[offset..end].to_vec())
}

fn build_multi_pamt(files: &[OverlayFile], paz_data_len: usize) -> Vec<u8> {
    let mut dir_block = Vec::new();
    let mut segment_offsets = BTreeMap::new();

    let mut unique_dirs: Vec<&str> = files.iter().map(|file| file.dir_path.as_str()).collect();
    unique_dirs.sort();
    unique_dirs.dedup();

    for dir_path in unique_dirs {
        if dir_path.is_empty() {
            continue;
        }

        let parts: Vec<&str> = dir_path.split('/').collect();
        for index in 0..parts.len() {
            let partial = parts[..=index].join("/");
            if segment_offsets.contains_key(&partial) {
                continue;
            }

            let offset = dir_block.len() as u32;
            segment_offsets.insert(partial.clone(), offset);

            let (parent, name) = if index == 0 {
                (0xFFFF_FFFF, parts[index].to_string())
            } else {
                let parent_path = parts[..index].join("/");
                (
                    *segment_offsets.get(&parent_path).unwrap(),
                    format!("/{}", parts[index]),
                )
            };

            dir_block.extend_from_slice(&parent.to_le_bytes());
            dir_block.push(name.len() as u8);
            dir_block.extend_from_slice(name.as_bytes());
        }
    }

    let mut files_by_dir = BTreeMap::<String, Vec<&OverlayFile>>::new();
    for file in files {
        files_by_dir
            .entry(file.dir_path.clone())
            .or_default()
            .push(file);
    }

    let mut fn_block = Vec::new();
    let mut hash_entries = Vec::new();
    let mut file_records = Vec::new();
    let mut file_index = 0u32;

    for (dir_path, dir_files) in files_by_dir {
        let dir_hash = pa_checksum(dir_path.as_bytes());
        let dir_name_offset = if dir_path.is_empty() {
            0xFFFF_FFFF
        } else {
            *segment_offsets.get(&dir_path).unwrap()
        };

        let file_start = file_index;
        for file in dir_files {
            let fn_offset = fn_block.len() as u32;
            fn_block.extend_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
            fn_block.push(file.filename.len() as u8);
            fn_block.extend_from_slice(file.filename.as_bytes());

            file_records.extend_from_slice(&fn_offset.to_le_bytes());
            file_records.extend_from_slice(&(file.paz_offset as u32).to_le_bytes());
            file_records.extend_from_slice(&(file.comp_size as u32).to_le_bytes());
            file_records.extend_from_slice(&(file.decomp_size as u32).to_le_bytes());
            file_records.extend_from_slice(&0u16.to_le_bytes());
            file_records.extend_from_slice(&file.flags.to_le_bytes());
            file_index += 1;
        }

        hash_entries.extend_from_slice(&dir_hash.to_le_bytes());
        hash_entries.extend_from_slice(&dir_name_offset.to_le_bytes());
        hash_entries.extend_from_slice(&file_start.to_le_bytes());
        hash_entries.extend_from_slice(&(file_index - file_start).to_le_bytes());
    }

    let mut body = Vec::new();
    body.extend_from_slice(&1u32.to_le_bytes());
    body.extend_from_slice(&PAMT_UNKNOWN.to_le_bytes());
    body.extend_from_slice(&0u32.to_le_bytes());
    body.extend_from_slice(&0u32.to_le_bytes());
    body.extend_from_slice(&(paz_data_len as u32).to_le_bytes());
    body.extend_from_slice(&(dir_block.len() as u32).to_le_bytes());
    body.extend_from_slice(&dir_block);
    body.extend_from_slice(&(fn_block.len() as u32).to_le_bytes());
    body.extend_from_slice(&fn_block);
    body.extend_from_slice(&((hash_entries.len() / 16) as u32).to_le_bytes());
    body.extend_from_slice(&hash_entries);
    body.extend_from_slice(&((file_records.len() / 20) as u32).to_le_bytes());
    body.extend_from_slice(&file_records);

    let mut output = Vec::new();
    output.extend_from_slice(&0u32.to_le_bytes());
    output.extend_from_slice(&body);
    let header_crc = pa_checksum(&output[12..]);
    output[0..4].copy_from_slice(&header_crc.to_le_bytes());
    output
}

fn update_pamt_paz_crc(pamt_data: &mut [u8], paz_crc: u32) {
    pamt_data[16..20].copy_from_slice(&paz_crc.to_le_bytes());
    let header_crc = pa_checksum(&pamt_data[12..]);
    pamt_data[0..4].copy_from_slice(&header_crc.to_le_bytes());
}

fn build_papgt_with_mod(path: &Path, mod_dir_name: &str, pamt_crc: u32) -> AppResult<Vec<u8>> {
    let original = fs::read(path)?;
    let group_count = original[8] as usize;
    let string_block_offset = 12 + group_count * 12;
    let string_block = &original[string_block_offset + 4..];

    let mut entries = Vec::new();
    for index in 0..group_count {
        let offset = 12 + index * 12;
        let name_offset = read_u32_le(&original, offset + 4) as usize;
        let end = string_block[name_offset..]
            .iter()
            .position(|byte| *byte == 0)
            .map(|position| name_offset + position)
            .ok_or_else(|| AppError::Patch("Invalid PAPGT string block".to_string()))?;
        let name = String::from_utf8_lossy(&string_block[name_offset..end]).to_string();
        entries.push((
            original[offset],
            read_u16_le(&original, offset + 1),
            original[offset + 3],
            name,
            read_u32_le(&original, offset + 8),
        ));
    }

    if let Some(existing) = entries.iter_mut().find(|entry| entry.3 == mod_dir_name) {
        existing.4 = pamt_crc;
    } else {
        entries.insert(
            0,
            (0, PAPGT_LANG_ALL, 0, mod_dir_name.to_string(), pamt_crc),
        );
    }

    let mut strings = Vec::new();
    let mut name_offsets = Vec::new();
    for entry in &entries {
        name_offsets.push(strings.len() as u32);
        strings.extend_from_slice(entry.3.as_bytes());
        strings.push(0);
    }

    let mut payload = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        payload.push(entry.0);
        payload.extend_from_slice(&entry.1.to_le_bytes());
        payload.push(entry.2);
        payload.extend_from_slice(&name_offsets[index].to_le_bytes());
        payload.extend_from_slice(&entry.4.to_le_bytes());
    }
    payload.extend_from_slice(&(strings.len() as u32).to_le_bytes());
    payload.extend_from_slice(&strings);

    let file_crc = pa_checksum(&payload);
    let mut output = Vec::new();
    output.extend_from_slice(&original[0..4]);
    output.extend_from_slice(&file_crc.to_le_bytes());
    output.push(entries.len() as u8);
    output.extend_from_slice(&original[9..11]);
    output.push(original[11]);
    output.extend_from_slice(&payload);
    Ok(output)
}

fn resolve_path_block(data: &[u8], name_offset: u32) -> String {
    let mut parts = Vec::new();
    let mut current = name_offset;
    let mut depth = 0;

    while current != 0xFFFF_FFFF && depth < 64 {
        let offset = current as usize;
        if offset + 5 > data.len() {
            break;
        }
        let parent = read_u32_le(data, offset);
        let length = data[offset + 4] as usize;
        if offset + 5 + length > data.len() {
            break;
        }
        parts.push(String::from_utf8_lossy(&data[offset + 5..offset + 5 + length]).to_string());
        current = parent;
        depth += 1;
    }

    parts.reverse();
    parts.concat()
}

fn rotl(value: u32, shift: u32) -> u32 {
    value.rotate_left(shift)
}

fn rotr(value: u32, shift: u32) -> u32 {
    value.rotate_right(shift)
}

fn read_u16_le(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn read_u32_le(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

fn decode_hex(input: &str) -> Result<Vec<u8>, String> {
    let trimmed = input.trim();
    if trimmed.len() % 2 != 0 {
        return Err("hex length must be even".to_string());
    }

    let mut output = Vec::with_capacity(trimmed.len() / 2);
    let bytes = trimmed.as_bytes();
    for chunk in bytes.chunks(2) {
        let high = decode_nibble(chunk[0])?;
        let low = decode_nibble(chunk[1])?;
        output.push((high << 4) | low);
    }

    Ok(output)
}

fn decode_nibble(value: u8) -> Result<u8, String> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(format!("invalid hex character '{}'", value as char)),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};
    use std::sync::{LazyLock, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::db;
    use crate::mods;

    use super::*;

    const GAME_PACKAGES_DIR: &str = "/Users/gigi/Games/Crimson Desert.app/Contents/Resources/packages";
    const DOWNLOADED_MODS_DIR: &str = "/Users/gigi/Downloads/CD Mods";
    static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[test]
    fn applies_real_json_mod_in_sandbox() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let sandbox = make_sandbox().unwrap();
        let connection = db::connect(&sandbox.app_data_dir).unwrap();
        let mod_path = Path::new(DOWNLOADED_MODS_DIR)
            .join("stamina_json_v1.02.00")
            .join("stamina_v1.02.00_infinite.json");

        mods::import_mod(
            &sandbox.app_data_dir,
            &connection,
            &mod_path,
            true,
            crate::models::ModKind::JsonData,
            None,
        )
        .unwrap();

        let records = db::list_mods(&connection).unwrap();
        let result = apply_mods(&sandbox.packages_dir, &records, &[], None, &BTreeMap::new()).unwrap();

        assert!(!result.created_groups.is_empty());
        let group_dir = sandbox.packages_dir.join(&result.created_groups[0]);
        assert!(group_dir.join("0.paz").is_file());
        assert!(group_dir.join("0.pamt").is_file());
        assert!(sandbox.packages_dir.join("meta").join("0.papgt").is_file());
        assert!(sandbox.packages_dir.join("meta").join("0.papgt.bak").is_file());
    }

    #[test]
    fn applies_real_browser_raw_mod_in_sandbox() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let sandbox = make_sandbox().unwrap();
        let connection = db::connect(&sandbox.app_data_dir).unwrap();
        let mod_path = Path::new(DOWNLOADED_MODS_DIR).join("Better_Inventory_UI_Compatible");

        mods::import_mod(
            &sandbox.app_data_dir,
            &connection,
            &mod_path,
            true,
            crate::models::ModKind::BrowserRaw,
            None,
        )
        .unwrap();

        let records = db::list_mods(&connection).unwrap();
        let result = apply_mods(&sandbox.packages_dir, &records, &[], None, &BTreeMap::new()).unwrap();

        assert!(!result.created_groups.is_empty());
        let group_dir = sandbox.packages_dir.join(&result.created_groups[0]);
        assert!(group_dir.join("0.paz").is_file());
        assert!(group_dir.join("0.pamt").is_file());
    }

    struct Sandbox {
        root: PathBuf,
        packages_dir: PathBuf,
        app_data_dir: PathBuf,
    }

    impl Drop for Sandbox {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn make_sandbox() -> AppResult<Sandbox> {
        let base = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut root = std::env::temp_dir().join(format!("cdmm_sandbox_{base}"));
        let mut suffix = 0u32;
        while root.exists() {
            suffix += 1;
            root = std::env::temp_dir().join(format!("cdmm_sandbox_{base}_{suffix}"));
        }
        let packages_dir = root.join("packages");
        let app_data_dir = root.join("app_data");
        fs::create_dir_all(&packages_dir)?;
        fs::create_dir_all(&app_data_dir)?;

        let real_packages = Path::new(GAME_PACKAGES_DIR);
        for entry in fs::read_dir(real_packages)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let target = packages_dir.join(&file_name);

            if path.is_dir() {
                if file_name == "meta" {
                    fs::create_dir_all(&target)?;
                    fs::copy(path.join("0.papgt"), target.join("0.papgt"))?;
                } else {
                    symlink(&path, &target)?;
                }
            }
        }

        Ok(Sandbox {
            root,
            packages_dir,
            app_data_dir,
        })
    }
}
