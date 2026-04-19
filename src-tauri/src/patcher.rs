use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use lz4_flex::block;

use crate::error::{AppError, AppResult};
use crate::game::{MOD_GROUP, SOURCE_GROUP};
use crate::models::{ApplyFileResult, ApplyResult, ModChange, ModRecord};
use crate::mods::{load_enabled_manifests, merged_changes};

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

pub fn apply_mods(game_dir: &Path, records: &[ModRecord]) -> AppResult<ApplyResult> {
    let enabled_manifests = load_enabled_manifests(records)?;
    if enabled_manifests.is_empty() {
        return Err(AppError::Patch(
            "No enabled mods are available. Import a mod and enable it before applying."
                .to_string(),
        ));
    }

    let pamt_info = read_pamt_raw(&game_dir.join(SOURCE_GROUP).join("0.pamt"))?;
    let (simple_index, full_index) = build_file_index(&pamt_info);
    let merged = merged_changes(&enabled_manifests);

    let overlay_dir = game_dir.join(MOD_GROUP);
    if overlay_dir.is_dir() {
        fs::remove_dir_all(&overlay_dir)?;
    }

    let mut paz_buffer = Vec::new();
    let mut overlay_files = Vec::new();
    let mut file_results = Vec::new();

    for (game_file, changes) in merged {
        let Some(info) = resolve_game_file(&game_file, &simple_index, &full_index) else {
            file_results.push(ApplyFileResult {
                game_file,
                source_paz_index: 0,
                applied_changes: 0,
                skipped_changes: 0,
                reason: Some("Target file not found in 0008/0.pamt".to_string()),
            });
            continue;
        };

        let record = &info.record;
        let src_paz = game_dir
            .join(SOURCE_GROUP)
            .join(format!("{}.paz", record.paz_index));
        let compressed = read_paz_slice(
            &src_paz,
            record.paz_offset as usize,
            record.comp_size as usize,
        )?;
        let mut buffer = block::decompress(&compressed, record.decomp_size as usize)?;

        let mut applied = 0usize;
        let mut skipped = 0usize;
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
        });

        file_results.push(ApplyFileResult {
            game_file: info.full_path.clone(),
            source_paz_index: record.paz_index,
            applied_changes: applied,
            skipped_changes: skipped,
            reason: None,
        });
    }

    if overlay_files.is_empty() {
        return Err(AppError::Patch(
			"No target files could be patched. Check mod compatibility with the current game build."
				.to_string(),
		));
    }

    fs::create_dir_all(&overlay_dir)?;
    let paz_path = overlay_dir.join("0.paz");
    fs::write(&paz_path, &paz_buffer)?;
    let paz_crc = pa_checksum(&paz_buffer);

    let mut pamt_data = build_multi_pamt(&overlay_files, paz_buffer.len());
    update_pamt_paz_crc(&mut pamt_data, paz_crc);
    let pamt_crc = read_u32_le(&pamt_data, 0);
    let pamt_path = overlay_dir.join("0.pamt");
    fs::write(&pamt_path, &pamt_data)?;

    let papgt_path = game_dir.join("meta").join("0.papgt");
    let papgt_backup = game_dir.join("meta").join("0.papgt.bak");
    if !papgt_backup.exists() {
        fs::copy(&papgt_path, &papgt_backup)?;
    } else {
        fs::copy(&papgt_backup, &papgt_path)?;
    }

    let new_papgt = build_papgt_with_mod(&papgt_path, MOD_GROUP, pamt_crc)?;
    fs::write(&papgt_path, &new_papgt)?;

    Ok(ApplyResult {
        mod_count: enabled_manifests.len(),
        target_file_count: file_results.len(),
        overlay_file_count: overlay_files.len(),
        paz_size: paz_buffer.len(),
        pamt_size: pamt_data.len(),
        files: file_results,
        message: format!(
            "Merged {} enabled mod(s) into {} patched file(s).",
            enabled_manifests.len(),
            overlay_files.len()
        ),
    })
}

pub fn restore_vanilla(game_dir: &Path) -> AppResult<()> {
    let papgt_path = game_dir.join("meta").join("0.papgt");
    let papgt_backup = game_dir.join("meta").join("0.papgt.bak");
    let overlay_dir = game_dir.join(MOD_GROUP);

    if papgt_backup.exists() {
        fs::copy(papgt_backup, papgt_path)?;
    }

    if overlay_dir.is_dir() {
        fs::remove_dir_all(overlay_dir)?;
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
            file_records.extend_from_slice(&0x0002u16.to_le_bytes());
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
