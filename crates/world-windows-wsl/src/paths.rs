use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::Path;
use substrate_common::FsDiff;

#[cfg(test)]
use std::path::PathBuf;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_wsl_path_resolves_relative_paths() {
        let project = PathBuf::from("C:\\projects\\substrate");
        let relative = PathBuf::from("src/main.rs");
        let absolute = PathBuf::from("D:\\workspace\\logs\\shim.txt");

        let relative_result = to_wsl_path(&project, &relative).expect("relative path resolves");
        let absolute_result = to_wsl_path(&project, &absolute).expect("absolute path resolves");

        assert_eq!(
            relative_result,
            "/mnt/c/projects/substrate/src/main.rs".to_string()
        );
        assert_eq!(
            absolute_result,
            "/mnt/d/workspace/logs/shim.txt".to_string()
        );
    }

    #[test]
    fn to_windows_display_path_handles_drives_and_unc() {
        let drive_path = PathBuf::from("/mnt/c/repo/new.txt");
        let unc_path = PathBuf::from("/mnt/unc/server/share/log.txt");

        let drive_display =
            to_windows_display_path(&drive_path).expect("drive letter should convert");
        let unc_display = to_windows_display_path(&unc_path).expect("unc should convert");

        assert_eq!(drive_display, "C:\\repo\\new.txt");
        assert_eq!(unc_display, "\\\\server\\share\\log.txt");
    }

    #[test]
    fn normalize_diff_populates_display_map() {
        let mut diff = FsDiff {
            writes: vec![PathBuf::from("/mnt/c/repo/create.txt")],
            mods: vec![PathBuf::from("/mnt/c/repo/change.txt")],
            deletes: vec![PathBuf::from("/mnt/c/repo/remove.txt")],
            ..Default::default()
        };

        normalize_diff(&mut diff);
        let display = diff.display_path.as_ref().expect("display mapping");

        assert_eq!(
            display.get("/mnt/c/repo/create.txt"),
            Some(&"C:\\repo\\create.txt".to_string())
        );
        assert_eq!(
            display.get("/mnt/c/repo/change.txt"),
            Some(&"C:\\repo\\change.txt".to_string())
        );
        assert_eq!(
            display.get("/mnt/c/repo/remove.txt"),
            Some(&"C:\\repo\\remove.txt".to_string())
        );
    }
}
