use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use substrate_common::FsDiff;

pub fn to_wsl_path(project_path: &Path, path: &Path) -> Result<String> {
    if path.is_relative() {
        let joined = project_path.join(path);
        return to_wsl_path(project_path, joined.as_path());
    }

    let raw = path
        .to_str()
        .ok_or_else(|| anyhow!("path is not valid UTF-8: {}", path.display()))?;
    let normalized = raw.replace('\\', "/");
    if let Some((drive, rest)) = normalized.split_once(':') {
        let rest = rest.trim_start_matches('/');
        Ok(format!("/mnt/{}/{}", drive.to_lowercase(), rest))
    } else {
        Ok(normalized)
    }
}

pub fn to_windows_display_path(path: &Path) -> Option<String> {
    let raw = path.to_str()?;
    let stripped = raw.strip_prefix("/mnt/")?;
    if let Some((prefix, rest)) = stripped.split_once('/') {
        if prefix.eq_ignore_ascii_case("unc") {
            if rest.is_empty() {
                return None;
            }
            let sep = std::path::MAIN_SEPARATOR.to_string();
            let converted = rest.replace('/', sep.as_str());
            return Some(format!("\\{}", converted));
        }

        if prefix.len() == 1 {
            let drive = prefix.chars().next()?.to_ascii_uppercase();
            let sep = std::path::MAIN_SEPARATOR.to_string();
            let converted = rest.replace('/', sep.as_str());
            if converted.is_empty() {
                return Some(format!("{drive}:{sep}", sep = std::path::MAIN_SEPARATOR));
            }
            return Some(format!(
                "{drive}:{sep}{converted}",
                sep = std::path::MAIN_SEPARATOR,
                converted = converted
            ));
        }
    } else if stripped.len() == 1 {
        let drive = stripped.chars().next()?.to_ascii_uppercase();
        return Some(format!("{drive}:{sep}", sep = std::path::MAIN_SEPARATOR));
    }

    None
}

pub fn normalize_diff(diff: &mut FsDiff) {
    let mut display = HashMap::new();
    for path in diff
        .writes
        .iter()
        .chain(diff.mods.iter())
        .chain(diff.deletes.iter())
    {
        if let Some(display_path) = to_windows_display_path(path) {
            display.insert(path.to_string_lossy().to_string(), display_path);
        }
    }

    diff.display_path = if display.is_empty() {
        None
    } else {
        Some(display)
    };
}
