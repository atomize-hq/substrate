use std::path::{Path, PathBuf};

fn find_repo_root_from(start: &Path) -> Option<PathBuf> {
    for dir in start.ancestors() {
        let candidate = dir.join("crates").join("codex");
        if candidate.is_dir() {
            return Some(dir.to_path_buf());
        }
    }
    None
}

fn repo_root() -> PathBuf {
    if let Ok(current_dir) = std::env::current_dir() {
        if let Some(path) = find_repo_root_from(&current_dir) {
            return path;
        }
    }

    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Some(path) = find_repo_root_from(&crate_dir) {
        return path;
    }

    crate_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or(crate_dir)
}

#[allow(dead_code)]
pub(crate) fn codex_examples_dir() -> PathBuf {
    repo_root().join("crates").join("codex").join("examples")
}
