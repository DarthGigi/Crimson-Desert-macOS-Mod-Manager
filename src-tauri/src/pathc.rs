use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};
use crate::models::{PathcLookupResult, PathcRepackResult, PathcSummary};

const HASH_INITVAL: u32 = 0x000C5EDE;

#[derive(Debug, Clone)]
struct PathcHeader {
    unknown0: u32,
    unknown1: u32,
    dds_record_size: u32,
    dds_record_count: u32,
    hash_count: u32,
    collision_path_count: u32,
    collision_blob_size: u32,
}

#[derive(Debug, Clone)]
struct PathcMapEntry {
    selector: u32,
    m1: u32,
    m2: u32,
    m3: u32,
    m4: u32,
}

#[derive(Debug, Clone)]
struct PathcCollisionEntry {
    dds_index: u32,
    m1: u32,
    m2: u32,
    m3: u32,
    m4: u32,
    path: String,
}

#[derive(Debug, Clone)]
struct PathcFile {
    header: PathcHeader,
    dds_records: Vec<Vec<u8>>,
    key_hashes: Vec<u32>,
    map_entries: Vec<PathcMapEntry>,
    collision_entries: Vec<PathcCollisionEntry>,
}

pub fn summarize_pathc(path: &Path, lookups: &[String]) -> AppResult<PathcSummary> {
    let parsed = read_pathc(path)?;

    let direct = parsed
        .map_entries
        .iter()
        .filter(|entry| direct_dds_index(entry.selector).is_some())
        .count();
    let collision = parsed
        .map_entries
        .iter()
        .filter(|entry| collision_range(entry.selector).is_some())
        .count();
    let unknown = parsed.map_entries.len().saturating_sub(direct + collision);

    let lookup_results = lookups
        .iter()
        .map(|lookup| lookup_path(&parsed, lookup))
        .collect::<AppResult<Vec<_>>>()?;

    Ok(PathcSummary {
        path: path.display().to_string(),
        dds_template_count: parsed.dds_records.len(),
        hash_count: parsed.key_hashes.len(),
        collision_path_count: parsed.collision_entries.len(),
        direct_mapping_count: direct,
        collision_mapping_count: collision,
        unknown_mapping_count: unknown,
        lookups: lookup_results,
    })
}

pub fn repack_pathc(pathc_path: &Path, folder_path: &Path) -> AppResult<PathcRepackResult> {
    if !folder_path.is_dir() {
        return Err(AppError::NotFound(format!(
            "DDS folder not found: {}",
            folder_path.display()
        )));
    }

    let mut pathc = read_pathc(pathc_path)?;
    let original_template_count = pathc.dds_records.len();
    let mut processed_count = 0usize;
    let mut updated_count = 0usize;

    for dds_path in collect_dds_files(folder_path)? {
        let relative = dds_path
            .strip_prefix(folder_path)
            .map_err(|err| AppError::Other(format!("Failed to derive DDS relative path: {err}")))?;
        let virtual_path = format!("/{}", relative.to_string_lossy().replace('\\', "/"));
        let dds_data = fs::read(&dds_path)?;
        let dds_record = create_dds_record(&dds_data, pathc.header.dds_record_size as usize)?;
        let metadata = dds_metadata(&dds_data)?;

        let dds_index = match pathc.dds_records.iter().position(|record| *record == dds_record) {
            Some(index) => index as u32,
            None => {
                pathc.dds_records.push(dds_record);
                (pathc.dds_records.len() - 1) as u32
            }
        };

        if update_entry(&mut pathc, &virtual_path, dds_index, (metadata.3, metadata.4, metadata.5, 0)) {
            updated_count += 1;
        }
        processed_count += 1;
    }

    let backup_path = pathc_path.with_extension("pathc.bak");
    if !backup_path.exists() {
        fs::copy(pathc_path, &backup_path)?;
    }

    let serialized = serialize_pathc(&mut pathc)?;
    fs::write(pathc_path, &serialized)?;

    Ok(PathcRepackResult {
        pathc_path: pathc_path.display().to_string(),
        backup_path: backup_path.display().to_string(),
        processed_count,
        updated_count,
        added_template_count: pathc.dds_records.len().saturating_sub(original_template_count),
    })
}

fn read_pathc(path: &Path) -> AppResult<PathcFile> {
    let raw = fs::read(path)?;
    if raw.len() < 0x1C {
        return Err(AppError::Other(format!("{} is too small to be a valid .pathc file", path.display())));
    }

    let header = PathcHeader {
        unknown0: read_u32(&raw, 0)?,
        unknown1: read_u32(&raw, 4)?,
        dds_record_size: read_u32(&raw, 8)?,
        dds_record_count: read_u32(&raw, 12)?,
        hash_count: read_u32(&raw, 16)?,
        collision_path_count: read_u32(&raw, 20)?,
        collision_blob_size: read_u32(&raw, 24)?,
    };

    let dds_table_off = 0x1Cusize;
    let dds_table_size = header.dds_record_size as usize * header.dds_record_count as usize;
    let hash_table_off = dds_table_off + dds_table_size;
    let hash_table_size = header.hash_count as usize * 4;
    let map_table_off = hash_table_off + hash_table_size;
    let map_table_size = header.hash_count as usize * 20;
    let collision_table_off = map_table_off + map_table_size;
    let collision_table_size = header.collision_path_count as usize * 24;
    let collision_blob_off = collision_table_off + collision_table_size;
    let collision_blob_end = collision_blob_off + header.collision_blob_size as usize;
    if collision_blob_end > raw.len() {
        return Err(AppError::Other("PATHC collision blob extends beyond file size".to_string()));
    }

    let mut dds_records = Vec::new();
    for index in 0..header.dds_record_count as usize {
        let start = dds_table_off + index * header.dds_record_size as usize;
        let end = start + header.dds_record_size as usize;
        if end > raw.len() {
            return Err(AppError::Other("DDS template table is truncated".to_string()));
        }
        dds_records.push(raw[start..end].to_vec());
    }

    let mut key_hashes = Vec::new();
    for index in 0..header.hash_count as usize {
        key_hashes.push(read_u32(&raw, hash_table_off + index * 4)?);
    }

    let mut map_entries = Vec::new();
    for index in 0..header.hash_count as usize {
        let offset = map_table_off + index * 20;
        map_entries.push(PathcMapEntry {
            selector: read_u32(&raw, offset)?,
            m1: read_u32(&raw, offset + 4)?,
            m2: read_u32(&raw, offset + 8)?,
            m3: read_u32(&raw, offset + 12)?,
            m4: read_u32(&raw, offset + 16)?,
        });
    }

    let collision_blob = &raw[collision_blob_off..collision_blob_end];
    let mut collision_entries = Vec::new();
    for index in 0..header.collision_path_count as usize {
        let offset = collision_table_off + index * 24;
        let path_offset = read_u32(&raw, offset)?;
        collision_entries.push(PathcCollisionEntry {
            dds_index: read_u32(&raw, offset + 4)?,
            m1: read_u32(&raw, offset + 8)?,
            m2: read_u32(&raw, offset + 12)?,
            m3: read_u32(&raw, offset + 16)?,
            m4: read_u32(&raw, offset + 20)?,
            path: read_c_string(collision_blob, path_offset as usize),
        });
    }

    Ok(PathcFile {
        header,
        dds_records,
        key_hashes,
        map_entries,
        collision_entries,
    })
}

fn serialize_pathc(pathc: &mut PathcFile) -> AppResult<Vec<u8>> {
    let mut collision_blob = Vec::new();
    let mut collision_rows = Vec::new();
    for entry in &pathc.collision_entries {
        let path_offset = collision_blob.len() as u32;
        collision_blob.extend_from_slice(entry.path.as_bytes());
        collision_blob.push(0);
        collision_rows.extend_from_slice(&path_offset.to_le_bytes());
        collision_rows.extend_from_slice(&entry.dds_index.to_le_bytes());
        collision_rows.extend_from_slice(&entry.m1.to_le_bytes());
        collision_rows.extend_from_slice(&entry.m2.to_le_bytes());
        collision_rows.extend_from_slice(&entry.m3.to_le_bytes());
        collision_rows.extend_from_slice(&entry.m4.to_le_bytes());
    }

    pathc.header.dds_record_count = pathc.dds_records.len() as u32;
    pathc.header.hash_count = pathc.key_hashes.len() as u32;
    pathc.header.collision_path_count = pathc.collision_entries.len() as u32;
    pathc.header.collision_blob_size = collision_blob.len() as u32;

    let mut output = Vec::new();
    output.extend_from_slice(&pathc.header.unknown0.to_le_bytes());
    output.extend_from_slice(&pathc.header.unknown1.to_le_bytes());
    output.extend_from_slice(&pathc.header.dds_record_size.to_le_bytes());
    output.extend_from_slice(&pathc.header.dds_record_count.to_le_bytes());
    output.extend_from_slice(&pathc.header.hash_count.to_le_bytes());
    output.extend_from_slice(&pathc.header.collision_path_count.to_le_bytes());
    output.extend_from_slice(&pathc.header.collision_blob_size.to_le_bytes());

    for record in &pathc.dds_records {
        output.extend_from_slice(record);
    }
    for key_hash in &pathc.key_hashes {
        output.extend_from_slice(&key_hash.to_le_bytes());
    }
    for entry in &pathc.map_entries {
        output.extend_from_slice(&entry.selector.to_le_bytes());
        output.extend_from_slice(&entry.m1.to_le_bytes());
        output.extend_from_slice(&entry.m2.to_le_bytes());
        output.extend_from_slice(&entry.m3.to_le_bytes());
        output.extend_from_slice(&entry.m4.to_le_bytes());
    }
    output.extend_from_slice(&collision_rows);
    output.extend_from_slice(&collision_blob);

    Ok(output)
}

fn lookup_path(pathc: &PathcFile, virtual_path: &str) -> AppResult<PathcLookupResult> {
    let normalized = normalize_path(virtual_path);
    let key_hash = hashlittle(normalized.to_lowercase().as_bytes(), HASH_INITVAL);
    let index = pathc.key_hashes.binary_search(&key_hash).ok();

    let mut result = PathcLookupResult {
        virtual_path: normalized.clone(),
        key_hash,
        found: false,
        direct_dds_index: None,
        width: None,
        height: None,
        mip_count: None,
        format_label: None,
        m1: None,
        m2: None,
        m3: None,
        m4: None,
    };

    let Some(index) = index else {
        return Ok(result);
    };
    let entry = &pathc.map_entries[index];
    result.found = true;
    result.m1 = Some(entry.m1);
    result.m2 = Some(entry.m2);
    result.m3 = Some(entry.m3);
    result.m4 = Some(entry.m4);

    if let Some(dds_index) = direct_dds_index(entry.selector) {
        result.direct_dds_index = Some(dds_index as usize);
        if let Some(record) = pathc.dds_records.get(dds_index as usize) {
            let metadata = dds_record_metadata(record)?;
            result.width = Some(metadata.0);
            result.height = Some(metadata.1);
            result.mip_count = Some(metadata.2);
            result.format_label = Some(metadata.3);
        }
        return Ok(result);
    }

    if let Some((start, end)) = collision_range(entry.selector) {
        for candidate in &pathc.collision_entries[start as usize..end as usize] {
            if normalize_path(&candidate.path).to_lowercase() == normalized.to_lowercase() {
                result.direct_dds_index = Some(candidate.dds_index as usize);
                result.m1 = Some(candidate.m1);
                result.m2 = Some(candidate.m2);
                result.m3 = Some(candidate.m3);
                result.m4 = Some(candidate.m4);
                if let Some(record) = pathc.dds_records.get(candidate.dds_index as usize) {
                    let metadata = dds_record_metadata(record)?;
                    result.width = Some(metadata.0);
                    result.height = Some(metadata.1);
                    result.mip_count = Some(metadata.2);
                    result.format_label = Some(metadata.3);
                }
                break;
            }
        }
    }

    Ok(result)
}

fn update_entry(pathc: &mut PathcFile, virtual_path: &str, dds_index: u32, m: (u32, u32, u32, u32)) -> bool {
    let target_hash = hashlittle(normalize_path(virtual_path).to_lowercase().as_bytes(), HASH_INITVAL);
    let idx = match pathc.key_hashes.binary_search(&target_hash) {
        Ok(index) => index,
        Err(index) => {
            pathc.key_hashes.insert(index, target_hash);
            pathc.map_entries.insert(
                index,
                PathcMapEntry {
                    selector: 0xFFFF_0000 | (dds_index & 0xFFFF),
                    m1: m.0,
                    m2: m.1,
                    m3: m.2,
                    m4: m.3,
                },
            );
            return false;
        }
    };

    pathc.map_entries[idx].selector = 0xFFFF_0000 | (dds_index & 0xFFFF);
    pathc.map_entries[idx].m1 = m.0;
    pathc.map_entries[idx].m2 = m.1;
    pathc.map_entries[idx].m3 = m.2;
    pathc.map_entries[idx].m4 = m.3;
    true
}

fn collect_dds_files(root: &Path) -> AppResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_dds_files_inner(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_dds_files_inner(root: &Path, output: &mut Vec<PathBuf>) -> AppResult<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_dds_files_inner(&path, output)?;
        } else if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("dds"))
        {
            output.push(path);
        }
    }
    Ok(())
}

fn create_dds_record(data: &[u8], record_size: usize) -> AppResult<Vec<u8>> {
    if data.len() < 4 || &data[..4] != b"DDS " {
        return Err(AppError::Other("Not a valid DDS file".to_string()));
    }

    let mut record = vec![0u8; record_size];
    let to_copy = data.len().min(record_size);
    record[..to_copy].copy_from_slice(&data[..to_copy]);
    Ok(record)
}

fn dds_metadata(data: &[u8]) -> AppResult<(u32, u32, u32, u32, u32, u32, String)> {
    if data.len() < 128 || &data[..4] != b"DDS " {
        return Err(AppError::Other("Invalid DDS file".to_string()));
    }

    let height = read_u32(data, 12)?;
    let width = read_u32(data, 16)?;
    let pitch = read_u32(data, 20)?;
    let mip_count = read_u32(data, 28)?.max(1);
    let fourcc = &data[84..88];
    let (format_label, block_bytes, bits_per_pixel) = if fourcc == b"DX10" && data.len() >= 148 {
        let dxgi = read_u32(data, 128)?;
        (format!("DX10/{dxgi}"), dxgi_block_bytes(dxgi), dxgi_bits_per_pixel(dxgi))
    } else {
        (
            String::from_utf8_lossy(fourcc).trim_end_matches('\0').to_string(),
            fourcc_block_bytes(fourcc),
            if read_u32(data, 80)? & 0x40 != 0 {
                read_u32(data, 88)?
            } else {
                0
            },
        )
    };

    let mut sizes = [0u32; 4];
    let mut current_width = width.max(1);
    let mut current_height = height.max(1);
    for index in 0..4usize.min(mip_count as usize) {
        let size = if let Some(block_bytes) = block_bytes {
            current_width.div_ceil(4).max(1) * current_height.div_ceil(4).max(1) * block_bytes
        } else if bits_per_pixel > 0 {
            ((current_width * bits_per_pixel + 7) / 8) * current_height
        } else if index == 0 && pitch > 0 {
            pitch
        } else {
            0
        };
        sizes[index] = size;
        current_width = (current_width / 2).max(1);
        current_height = (current_height / 2).max(1);
    }

    Ok((width, height, mip_count, sizes[0], sizes[1], sizes[2], format_label))
}

fn dds_record_metadata(record: &[u8]) -> AppResult<(u32, u32, u32, String)> {
    let metadata = dds_metadata(record)?;
    Ok((metadata.0, metadata.1, metadata.2, metadata.6))
}

fn normalize_path(path: &str) -> String {
    let cleaned = path.replace('\\', "/").trim().trim_start_matches('/').trim_end_matches('/').to_string();
    format!("/{cleaned}")
}

fn direct_dds_index(selector: u32) -> Option<u32> {
    let high = (selector >> 16) & 0xFFFF;
    let low = selector & 0xFFFF;
    (high == 0xFFFF).then_some(low)
}

fn collision_range(selector: u32) -> Option<(u16, u16)> {
    let high = ((selector >> 16) & 0xFFFF) as u16;
    let low = (selector & 0xFFFF) as u16;
    if low != 0xFFFF {
        return None;
    }
    let start = high & 0x00FF;
    let end = (high >> 8) & 0x00FF;
    (end >= start).then_some((start, end))
}

fn read_c_string(data: &[u8], offset: usize) -> String {
    if offset >= data.len() {
        return String::new();
    }
    let end = data[offset..]
        .iter()
        .position(|byte| *byte == 0)
        .map(|position| offset + position)
        .unwrap_or(data.len());
    String::from_utf8_lossy(&data[offset..end]).to_string()
}

fn read_u32(data: &[u8], offset: usize) -> AppResult<u32> {
    let slice = data.get(offset..offset + 4).ok_or_else(|| {
        AppError::Other(format!("PATHC read out of range at offset {offset}"))
    })?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn fourcc_block_bytes(fourcc: &[u8]) -> Option<u32> {
    match fourcc {
        b"DXT1" | b"ATI1" | b"BC4U" | b"BC4S" => Some(8),
        b"DXT3" | b"DXT5" | b"ATI2" | b"BC5U" | b"BC5S" => Some(16),
        _ => None,
    }
}

fn dxgi_block_bytes(dxgi: u32) -> Option<u32> {
    match dxgi {
        70 | 71 | 72 | 79 | 80 | 81 => Some(8),
        73 | 74 | 75 | 76 | 77 | 78 | 82 | 83 | 84 | 94 | 95 | 96 | 97 | 98 | 99 => Some(16),
        _ => None,
    }
}

fn dxgi_bits_per_pixel(dxgi: u32) -> u32 {
    match dxgi {
        10 => 64,
        24 | 28 => 32,
        61 => 8,
        _ => 0,
    }
}

fn rot32(value: u32, bits: u32) -> u32 {
    value.rotate_left(bits)
}

fn hashlittle(data: &[u8], initval: u32) -> u32 {
    let length = data.len() as u32;
    let mut remaining = data.len();
    let mut a = 0xDEAD_BEEF_u32.wrapping_add(length).wrapping_add(initval);
    let mut b = a;
    let mut c = a;
    let mut offset = 0usize;

    while remaining > 12 {
        a = a.wrapping_add(u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()));
        b = b.wrapping_add(u32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap()));
        c = c.wrapping_add(u32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap()));
        a = a.wrapping_sub(c) ^ rot32(c, 4);
        c = c.wrapping_add(b);
        b = b.wrapping_sub(a) ^ rot32(a, 6);
        a = a.wrapping_add(c);
        c = c.wrapping_sub(b) ^ rot32(b, 8);
        b = b.wrapping_add(a);
        a = a.wrapping_sub(c) ^ rot32(c, 16);
        c = c.wrapping_add(b);
        b = b.wrapping_sub(a) ^ rot32(a, 19);
        a = a.wrapping_add(c);
        c = c.wrapping_sub(b) ^ rot32(b, 4);
        b = b.wrapping_add(a);
        offset += 12;
        remaining -= 12;
    }

    let mut tail = [0u8; 12];
    tail[..remaining].copy_from_slice(&data[offset..offset + remaining]);
    match remaining {
        12 => c = c.wrapping_add(u32::from_le_bytes(tail[8..12].try_into().unwrap())),
        9..=11 => c = c.wrapping_add(u32::from_le_bytes(tail[8..12].try_into().unwrap()) & (0xFFFF_FFFF >> (8 * (12 - remaining)))),
        _ => {}
    }
    match remaining {
        8..=12 => b = b.wrapping_add(u32::from_le_bytes(tail[4..8].try_into().unwrap())),
        5..=7 => b = b.wrapping_add(u32::from_le_bytes(tail[4..8].try_into().unwrap()) & (0xFFFF_FFFF >> (8 * (8 - remaining)))),
        _ => {}
    }
    match remaining {
        4..=12 => a = a.wrapping_add(u32::from_le_bytes(tail[0..4].try_into().unwrap())),
        1..=3 => a = a.wrapping_add(u32::from_le_bytes(tail[0..4].try_into().unwrap()) & (0xFFFF_FFFF >> (8 * (4 - remaining)))),
        0 => return c,
        _ => {}
    }

    c ^= b;
    c = c.wrapping_sub(rot32(b, 14));
    a ^= c;
    a = a.wrapping_sub(rot32(c, 11));
    b ^= a;
    b = b.wrapping_sub(rot32(a, 25));
    c ^= b;
    c = c.wrapping_sub(rot32(b, 16));
    a ^= c;
    a = a.wrapping_sub(rot32(c, 4));
    b ^= a;
    b = b.wrapping_sub(rot32(a, 14));
    c ^= b;
    c = c.wrapping_sub(rot32(b, 24));
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    const GAME_PATHC: &str = "/Users/gigi/Games/Crimson Desert.app/Contents/Resources/packages/meta/0.pathc";
    const DDS_FOLDER: &str = "/Users/gigi/Downloads/CD Mods/Better_Inventory_UI_Compatible/files/0012";

    #[test]
    fn summarizes_real_pathc_file() {
        let summary = summarize_pathc(Path::new(GAME_PATHC), &["/ui/texture/cd_itemslot_00.dds".to_string()]).unwrap();
        assert!(summary.dds_template_count > 0);
        assert!(summary.hash_count > 0);
        assert_eq!(summary.lookups.len(), 1);
    }

    #[test]
    fn repacks_pathc_in_sandbox() {
        let temp_root = std::env::temp_dir().join("cdmm_pathc_test");
        let _ = fs::remove_dir_all(&temp_root);
        fs::create_dir_all(&temp_root).unwrap();
        let sandbox_pathc = temp_root.join("0.pathc");
        fs::copy(GAME_PATHC, &sandbox_pathc).unwrap();

        let result = repack_pathc(&sandbox_pathc, Path::new(DDS_FOLDER)).unwrap();
        assert!(result.processed_count > 0);
        assert!(Path::new(&result.backup_path).is_file());

        let _ = fs::remove_dir_all(&temp_root);
    }
}
