use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

use crate::error::{AppError, AppResult};
use crate::models::ModRecord;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AsiPluginInfo {
	pub name: String,
	pub enabled: bool,
	pub path: String,
	pub ini_files: Vec<String>,
	pub hook_targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalFileInfo {
	pub name: String,
	pub path: String,
	pub kind: String,
}

pub fn install_asi_mod(record: &ModRecord, target_dir: &Path) -> AppResult<Vec<String>> {
	fs::create_dir_all(target_dir)?;
	let mut installed = Vec::new();
	if Path::new(&record.library_path).is_file() {
		let source = Path::new(&record.library_path);
		let dest = target_dir.join(source.file_name().unwrap_or_default());
		fs::copy(source, &dest)?;
		installed.push(dest.file_name().unwrap_or_default().to_string_lossy().to_string());
		return Ok(installed);
	}

	for entry in walk_files(Path::new(&record.library_path))? {
		let ext = entry
			.extension()
			.and_then(|value| value.to_str())
			.map(|value| value.to_ascii_lowercase())
			.unwrap_or_default();
		let name = entry.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
		if !(ext == "asi" || ext == "ini" || ext == "dll") {
			continue;
		}
		let dest = target_dir.join(entry.file_name().unwrap_or_default());
		fs::copy(&entry, &dest)?;
		installed.push(dest.file_name().unwrap_or_default().to_string_lossy().to_string());
		if name.ends_with(".dll") && !["winmm.dll", "version.dll", "dinput8.dll", "dsound.dll"].contains(&name.as_str()) {
			continue;
		}
	}

	Ok(installed)
}

pub fn scan_asi_plugins(target_dir: &Path) -> AppResult<Vec<AsiPluginInfo>> {
	if !target_dir.is_dir() {
		return Ok(Vec::new());
	}

	let mut plugins = Vec::new();
	for entry in fs::read_dir(target_dir)? {
		let entry = entry?;
		let path = entry.path();
		if !path.is_file() {
			continue;
		}
		let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
		let lower = file_name.to_lowercase();
		if !(lower.ends_with(".asi") || lower.ends_with(".asi.disabled")) {
			continue;
		}
		let enabled = lower.ends_with(".asi");
		let base_name = if enabled {
			path.file_stem().unwrap_or_default().to_string_lossy().to_string()
		} else {
			file_name.trim_end_matches(".disabled").trim_end_matches(".asi").to_string()
		};
		let ini_files = fs::read_dir(target_dir)?
			.filter_map(Result::ok)
			.filter(|candidate| {
				candidate
					.path()
					.extension()
					.and_then(|value| value.to_str())
					.is_some_and(|ext| ext.eq_ignore_ascii_case("ini"))
					&& candidate.file_name().to_string_lossy().to_lowercase().starts_with(&base_name.to_lowercase())
			})
			.map(|candidate| candidate.file_name().to_string_lossy().to_string())
			.collect();

		plugins.push(AsiPluginInfo {
			name: base_name,
			enabled,
			path: path.display().to_string(),
			ini_files,
			hook_targets: Vec::new(),
		});
	}
	plugins.sort_by(|left, right| left.name.cmp(&right.name));
	Ok(plugins)
}

pub fn set_asi_enabled(target_dir: &Path, plugin_name: &str, enabled: bool) -> AppResult<()> {
	fs::create_dir_all(target_dir)?;
	let enabled_path = target_dir.join(format!("{plugin_name}.asi"));
	let disabled_path = target_dir.join(format!("{plugin_name}.asi.disabled"));
	if enabled {
		if disabled_path.exists() {
			fs::rename(disabled_path, enabled_path)?;
		}
	} else if enabled_path.exists() {
		fs::rename(enabled_path, disabled_path)?;
	}
	Ok(())
}

pub fn remove_asi_plugin(target_dir: &Path, plugin_name: &str) -> AppResult<()> {
	let enabled_path = target_dir.join(format!("{plugin_name}.asi"));
	let disabled_path = target_dir.join(format!("{plugin_name}.asi.disabled"));
	if enabled_path.exists() {
		fs::remove_file(enabled_path)?;
	}
	if disabled_path.exists() {
		fs::remove_file(disabled_path)?;
	}
	for entry in fs::read_dir(target_dir)? {
		let entry = entry?;
		let path = entry.path();
		if path
			.extension()
			.and_then(|value| value.to_str())
			.is_some_and(|ext| ext.eq_ignore_ascii_case("ini"))
			&& entry.file_name().to_string_lossy().to_lowercase().starts_with(&plugin_name.to_lowercase())
		{
			fs::remove_file(path)?;
		}
	}
	Ok(())
}

pub fn install_simple_external_mod(record: &ModRecord, target_dir: &Path, allowed_exts: &[&str]) -> AppResult<Vec<String>> {
	fs::create_dir_all(target_dir)?;
	let mut installed = Vec::new();
	for entry in walk_files(Path::new(&record.library_path))? {
		let ext = entry
			.extension()
			.and_then(|value| value.to_str())
			.map(|value| value.to_ascii_lowercase())
			.unwrap_or_default();
		if !allowed_exts.iter().any(|candidate| ext.eq_ignore_ascii_case(candidate)) {
			continue;
		}
		let dest = target_dir.join(entry.file_name().unwrap_or_default());
		fs::copy(&entry, &dest)?;
		installed.push(dest.file_name().unwrap_or_default().to_string_lossy().to_string());
	}
	Ok(installed)
}

pub fn scan_simple_external_files(target_dir: &Path, allowed_exts: &[&str], label: &str) -> AppResult<Vec<ExternalFileInfo>> {
	if !target_dir.is_dir() {
		return Ok(Vec::new());
	}
	let mut files = Vec::new();
	for entry in fs::read_dir(target_dir)? {
		let entry = entry?;
		let path = entry.path();
		if !path.is_file() {
			continue;
		}
		let ext = path
			.extension()
			.and_then(|value| value.to_str())
			.map(|value| value.to_ascii_lowercase())
			.unwrap_or_default();
		if !allowed_exts.iter().any(|candidate| ext.eq_ignore_ascii_case(candidate)) {
			continue;
		}
		files.push(ExternalFileInfo {
			name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
			path: path.display().to_string(),
			kind: label.to_string(),
		});
	}
	files.sort_by(|left, right| left.name.cmp(&right.name));
	Ok(files)
}

pub fn remove_simple_external_file(target_dir: &Path, name: &str) -> AppResult<()> {
	let path = target_dir.join(name);
	if path.exists() {
		fs::remove_file(path)?;
	}
	Ok(())
}

pub fn first_matching_file(root: &Path, allowed_exts: &[&str]) -> AppResult<Option<PathBuf>> {
	for entry in walk_files(root)? {
		let ext = entry
			.extension()
			.and_then(|value| value.to_str())
			.map(|value| value.to_ascii_lowercase())
			.unwrap_or_default();
		if allowed_exts.iter().any(|candidate| ext.eq_ignore_ascii_case(candidate)) {
			return Ok(Some(entry));
		}
	}
	Ok(None)
}

pub fn apply_binary_patch(
    record: &ModRecord,
    target_file: &Path,
    output_file: &Path,
    bundled_xdelta3_path: Option<&Path>,
) -> AppResult<()> {
	let patch_file = first_matching_file(Path::new(&record.library_path), &["bsdiff", "xdelta"])?
		.ok_or_else(|| AppError::NotFound("No .bsdiff or .xdelta file found in the imported mod".to_string()))?;
	let ext = lower_extension(&patch_file);

	if let Some(parent) = output_file.parent() {
		fs::create_dir_all(parent)?;
	}

	let status = if ext == "bsdiff" {
		Command::new("bspatch")
			.arg(target_file)
			.arg(output_file)
			.arg(&patch_file)
			.status()?
	} else {
        let tool = bundled_xdelta3_path
            .filter(|path| path.is_file())
            .map(PathBuf::from)
            .or_else(|| {
                ["/opt/homebrew/bin/xdelta3", "/usr/local/bin/xdelta3", "xdelta3"]
			.into_iter()
			.find(|tool| {
				if tool.contains('/') {
					Path::new(tool).is_file()
				} else {
					Command::new(tool).arg("-V").output().is_ok()
				}
            })
			.map(PathBuf::from)
			})
			.ok_or_else(|| AppError::Other("xdelta3 is not available on this system".to_string()))?;
        Command::new(tool)
			.arg("-d")
			.arg("-s")
			.arg(target_file)
			.arg(&patch_file)
			.arg(output_file)
			.status()?
	};

	if !status.success() {
		return Err(AppError::Other(format!(
			"Failed to apply binary patch {}",
			patch_file.display()
		)));
	}

	Ok(())
}

pub fn run_script_installer(record: &ModRecord, working_dir: &Path) -> AppResult<Vec<String>> {
	let script = first_matching_file(Path::new(&record.library_path), &["sh", "command", "py", "bat"])?
		.ok_or_else(|| AppError::NotFound("No supported script file found in the imported mod".to_string()))?;
	let ext = lower_extension(&script);

	let status = match ext.as_str() {
		"sh" | "command" => Command::new("/bin/sh").arg(&script).current_dir(working_dir).status()?,
		"py" => Command::new("python3").arg(&script).current_dir(working_dir).status()?,
		"bat" => {
			return Err(AppError::Other(
				"Windows .bat installers are not runnable natively on macOS".to_string(),
			))
		}
		_ => return Err(AppError::Other("Unsupported script extension".to_string())),
	};

	if !status.success() {
		return Err(AppError::Other(format!(
			"Script installer failed: {}",
			script.display()
		)));
	}

	Ok(vec![script.file_name().unwrap_or_default().to_string_lossy().to_string()])
}

fn walk_files(root: &Path) -> AppResult<Vec<PathBuf>> {
	let mut files = Vec::new();
	walk_files_inner(root, &mut files)?;
	Ok(files)
}

fn lower_extension(path: &Path) -> String {
	path.extension()
		.and_then(|value| value.to_str())
		.map(|value| value.to_ascii_lowercase())
		.unwrap_or_default()
}

fn walk_files_inner(root: &Path, output: &mut Vec<PathBuf>) -> AppResult<()> {
	for entry in fs::read_dir(root)? {
		let entry = entry?;
		let path = entry.path();
		if path.is_dir() {
			walk_files_inner(&path, output)?;
		} else if path.is_file() {
			output.push(path);
		}
	}
	Ok(())
}
