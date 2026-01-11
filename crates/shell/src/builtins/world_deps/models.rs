use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct WorldDepsStatusReport {
    #[serde(default)]
    pub selection: WorldDepsSelectionInfo,
    pub manifest: WorldDepsManifestInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_disabled_reason: Option<String>,
    pub tools: Vec<WorldDepStatusEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub(crate) struct WorldDepsSelectionInfo {
    pub configured: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_scope: Option<WorldDepsSelectionScope>,
    pub shadowed_paths: Vec<PathBuf>,
    pub selected: Vec<String>,
    pub ignored_due_to_all: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorldDepsSelectionScope {
    Workspace,
    Global,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct WorldDepsManifestInfo {
    pub inventory: ManifestLayerInfo,
    pub overlays: WorldDepsOverlayInfo,
    pub layers: Vec<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ManifestLayerInfo {
    pub base: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlay: Option<PathBuf>,
    pub overlay_exists: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct WorldDepsOverlayInfo {
    pub installed: PathBuf,
    pub installed_exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<PathBuf>,
    pub user_exists: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct WorldDepStatusEntry {
    pub name: String,
    #[serde(default)]
    pub selected: bool,
    pub host_detected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_reason: Option<String>,
    pub guest: WorldDepGuestStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct WorldDepGuestStatus {
    pub status: WorldDepGuestState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorldDepGuestState {
    Present,
    Missing,
    Skipped,
    Unavailable,
}

pub(crate) enum GuestProbe {
    Available(bool),
    Skipped(String),
    Unavailable(String),
}

impl WorldDepGuestStatus {
    pub(crate) fn from_probe(probe: GuestProbe) -> Self {
        match probe {
            GuestProbe::Available(true) => Self {
                status: WorldDepGuestState::Present,
                reason: None,
            },
            GuestProbe::Available(false) => Self {
                status: WorldDepGuestState::Missing,
                reason: None,
            },
            GuestProbe::Skipped(reason) => Self {
                status: WorldDepGuestState::Skipped,
                reason: Some(sanitize_reason(&reason)),
            },
            GuestProbe::Unavailable(reason) => Self {
                status: WorldDepGuestState::Unavailable,
                reason: Some(sanitize_reason(&reason)),
            },
        }
    }

    pub(crate) fn label(&self) -> String {
        match self.status {
            WorldDepGuestState::Present => "present".to_string(),
            WorldDepGuestState::Missing => "missing".to_string(),
            WorldDepGuestState::Skipped => {
                format!("n/a ({})", self.reason.as_deref().unwrap_or("skipped"))
            }
            WorldDepGuestState::Unavailable => format!(
                "missing ({})",
                self.reason.as_deref().unwrap_or("backend unavailable")
            ),
        }
    }
}

pub(crate) fn sanitize_reason(reason: &str) -> String {
    reason
        .replace('\n', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
