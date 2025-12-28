use std::path::{Path, PathBuf};

pub(crate) const SUBSTRATE_DIR_NAME: &str = ".substrate";
pub(crate) const WORKSPACE_MARKER_FILENAME: &str = "workspace.yaml";
pub(crate) const LEGACY_SETTINGS_FILENAME: &str = "settings.yaml";
pub(crate) const WORKSPACE_POLICY_FILENAME: &str = "policy.yaml";

pub(crate) fn workspace_marker_path(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join(SUBSTRATE_DIR_NAME)
        .join(WORKSPACE_MARKER_FILENAME)
}

pub(crate) fn workspace_policy_path(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join(SUBSTRATE_DIR_NAME)
        .join(WORKSPACE_POLICY_FILENAME)
}

pub(crate) fn workspace_legacy_settings_path(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join(SUBSTRATE_DIR_NAME)
        .join(LEGACY_SETTINGS_FILENAME)
}

pub(crate) fn internal_git_dir(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".substrate-git").join("repo.git")
}

pub(crate) fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    let start = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());

    for dir in start.ancestors() {
        let marker = workspace_marker_path(dir);
        if marker.is_file() {
            return Some(dir.to_path_buf());
        }
    }
    None
}
