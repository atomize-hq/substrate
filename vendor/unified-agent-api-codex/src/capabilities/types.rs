use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use thiserror::Error;

use super::{BinaryFingerprint, CapabilityCacheKey, CapabilityProbePlan};

/// Snapshot of Codex CLI capabilities derived from probing a specific binary.
///
/// Instances of this type are intended to be cached per binary path so callers can
/// gate optional flags (like `--output-schema`) without repeatedly spawning the CLI.
/// A process-wide `HashMap<CapabilityCacheKey, CodexCapabilities>` (behind a mutex/once)
/// keeps probes cheap; entries should use canonical binary paths where possible and
/// ship a [`BinaryFingerprint`] so we can invalidate stale snapshots when the binary
/// on disk changes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CodexCapabilities {
    /// Canonical path used as the cache key.
    pub cache_key: CapabilityCacheKey,
    /// File metadata used to detect when a cached entry is stale.
    pub fingerprint: Option<BinaryFingerprint>,
    /// Parsed output from `codex --version`; `None` when the command fails.
    pub version: Option<CodexVersionInfo>,
    /// Known feature toggles; fields default to `false` when detection fails.
    pub features: CodexFeatureFlags,
    /// Steps attempted while interrogating the binary (version, features, help).
    pub probe_plan: CapabilityProbePlan,
    /// Timestamp of when the probe finished.
    pub collected_at: SystemTime,
}

/// Parsed version details emitted by `codex --version`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CodexVersionInfo {
    /// Raw stdout from `codex --version` so we do not lose channel/build metadata.
    pub raw: String,
    /// Parsed `major.minor.patch` triplet when the output contains a semantic version.
    pub semantic: Option<(u64, u64, u64)>,
    /// Optional commit hash or build identifier printed by pre-release builds.
    pub commit: Option<String>,
    /// Release channel inferred from the version string suffix (e.g., `-beta`).
    pub channel: CodexReleaseChannel,
}

/// Release channel segments inferred from the Codex version string.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CodexReleaseChannel {
    Stable,
    Beta,
    Nightly,
    /// Fallback for bespoke or vendor-patched builds.
    Custom,
}

impl std::fmt::Display for CodexReleaseChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            CodexReleaseChannel::Stable => "stable",
            CodexReleaseChannel::Beta => "beta",
            CodexReleaseChannel::Nightly => "nightly",
            CodexReleaseChannel::Custom => "custom",
        };
        write!(f, "{label}")
    }
}

/// Release metadata for a specific Codex build channel.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CodexRelease {
    /// Release channel (stable/beta/nightly/custom).
    pub channel: CodexReleaseChannel,
    /// Parsed semantic version for comparison.
    pub version: Version,
}

/// Caller-supplied table of known latest Codex releases.
///
/// The crate intentionally avoids network requests; hosts should populate this
/// with data from their preferred distribution channel (e.g. `npm view
/// @openai/codex version`, `brew info codex --json`, or the GitHub releases
/// API) before requesting an update advisory.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct CodexLatestReleases {
    /// Latest stable release version.
    pub stable: Option<Version>,
    /// Latest beta pre-release when available.
    pub beta: Option<Version>,
    /// Latest nightly build when available.
    pub nightly: Option<Version>,
}

impl CodexLatestReleases {
    /// Returns the most appropriate latest release for the given channel,
    /// falling back to a more stable track when channel-specific data is
    /// missing.
    pub fn select_for_channel(
        &self,
        channel: CodexReleaseChannel,
    ) -> (Option<CodexRelease>, CodexReleaseChannel, bool) {
        if let Some(release) = self.release_for_channel(channel) {
            return (Some(release), channel, false);
        }

        let fallback = self
            .stable
            .as_ref()
            .map(|version| CodexRelease {
                channel: CodexReleaseChannel::Stable,
                version: version.clone(),
            })
            .or_else(|| {
                self.beta.as_ref().map(|version| CodexRelease {
                    channel: CodexReleaseChannel::Beta,
                    version: version.clone(),
                })
            })
            .or_else(|| {
                self.nightly.as_ref().map(|version| CodexRelease {
                    channel: CodexReleaseChannel::Nightly,
                    version: version.clone(),
                })
            });

        let fallback_channel = fallback
            .as_ref()
            .map(|release| release.channel)
            .unwrap_or(channel);
        let fell_back = fallback_channel != channel;
        (fallback, fallback_channel, fell_back)
    }

    fn release_for_channel(&self, channel: CodexReleaseChannel) -> Option<CodexRelease> {
        match channel {
            CodexReleaseChannel::Stable => self.stable.as_ref().map(|version| CodexRelease {
                channel,
                version: version.clone(),
            }),
            CodexReleaseChannel::Beta => self.beta.as_ref().map(|version| CodexRelease {
                channel,
                version: version.clone(),
            }),
            CodexReleaseChannel::Nightly => self.nightly.as_ref().map(|version| CodexRelease {
                channel,
                version: version.clone(),
            }),
            CodexReleaseChannel::Custom => None,
        }
    }
}

/// Update guidance derived from comparing local and latest Codex versions.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CodexUpdateAdvisory {
    /// Local release as parsed from `codex --version`.
    pub local_release: Option<CodexRelease>,
    /// Latest release used for comparison (may be a fallback track).
    pub latest_release: Option<CodexRelease>,
    /// Channel chosen for comparison (local channel when available, otherwise stable).
    pub comparison_channel: CodexReleaseChannel,
    /// High-level outcome to drive host UX.
    pub status: CodexUpdateStatus,
    /// Human-readable hints callers can log or display.
    pub notes: Vec<String>,
}

impl CodexUpdateAdvisory {
    /// True when the host should prompt for or attempt an update.
    pub fn is_update_recommended(&self) -> bool {
        matches!(
            self.status,
            CodexUpdateStatus::UpdateRecommended | CodexUpdateStatus::UnknownLocalVersion
        )
    }
}

/// Enum summarizing whether an update is needed based on provided release data.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CodexUpdateStatus {
    /// Local binary matches the latest known release for the comparison channel.
    UpToDate,
    /// A newer release exists for the comparison channel.
    UpdateRecommended,
    /// Local binary appears newer than the provided release table (e.g., dev build).
    LocalNewerThanKnown,
    /// No local version data was available (probe failure).
    UnknownLocalVersion,
    /// Caller did not provide a comparable latest release.
    UnknownLatestVersion,
}

/// Feature gates for Codex CLI flags.
///
/// All fields default to `false` so callers can conservatively avoid passing flags
/// unless probes prove that the binary understands them.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct CodexFeatureFlags {
    /// True when `codex features list` is available.
    pub supports_features_list: bool,
    /// True when `--output-schema` is accepted by `codex exec`.
    pub supports_output_schema: bool,
    /// True when `codex add-dir` is available for recursive prompting.
    pub supports_add_dir: bool,
    /// True when `codex login --mcp` is recognized for MCP integration.
    pub supports_mcp_login: bool,
}

/// Optional overrides for feature detection that can be layered onto probe results.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct CapabilityFeatureOverrides {
    /// Override for `codex features list` support; `None` defers to probes.
    pub supports_features_list: Option<bool>,
    /// Override for `--output-schema` support; `None` defers to probes.
    pub supports_output_schema: Option<bool>,
    /// Override for `codex add-dir` support; `None` defers to probes.
    pub supports_add_dir: Option<bool>,
    /// Override for `codex login --mcp` support; `None` defers to probes.
    pub supports_mcp_login: Option<bool>,
}

impl CapabilityFeatureOverrides {
    /// Returns true when no overrides are set.
    pub fn is_empty(&self) -> bool {
        self.supports_features_list.is_none()
            && self.supports_output_schema.is_none()
            && self.supports_add_dir.is_none()
            && self.supports_mcp_login.is_none()
    }

    /// Builds overrides that mirror every provided feature flag, including false values.
    pub fn from_flags(flags: CodexFeatureFlags) -> Self {
        CapabilityFeatureOverrides {
            supports_features_list: Some(flags.supports_features_list),
            supports_output_schema: Some(flags.supports_output_schema),
            supports_add_dir: Some(flags.supports_add_dir),
            supports_mcp_login: Some(flags.supports_mcp_login),
        }
    }

    /// Builds overrides that only force-enable flags that are true in the input set.
    pub fn enabling(flags: CodexFeatureFlags) -> Self {
        CapabilityFeatureOverrides {
            supports_features_list: flags.supports_features_list.then_some(true),
            supports_output_schema: flags.supports_output_schema.then_some(true),
            supports_add_dir: flags.supports_add_dir.then_some(true),
            supports_mcp_login: flags.supports_mcp_login.then_some(true),
        }
    }
}

/// Caller-supplied capability data that can short-circuit or adjust probing.
/// Manual snapshots override cached/probed data, and feature/version overrides
/// apply on top of whichever snapshot is returned.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct CapabilityOverrides {
    /// Manual snapshot to return instead of probing when present (after applying feature/version overrides).
    pub snapshot: Option<CodexCapabilities>,
    /// Version override applied after probing.
    pub version: Option<CodexVersionInfo>,
    /// Feature-level overrides merged into probed or manual capabilities.
    pub features: CapabilityFeatureOverrides,
}

impl CapabilityOverrides {
    /// Returns true when no override data is present.
    pub fn is_empty(&self) -> bool {
        self.snapshot.is_none() && self.version.is_none() && self.features.is_empty()
    }
}

/// Supported serialization formats for capability snapshots and overrides.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilitySnapshotFormat {
    Json,
    Toml,
}

impl CapabilitySnapshotFormat {
    pub(crate) fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_string_lossy().to_lowercase();
        match ext.as_str() {
            "json" => Some(Self::Json),
            "toml" => Some(Self::Toml),
            _ => None,
        }
    }
}

/// Errors encountered while saving or loading capability snapshots.
#[derive(Debug, Error)]
pub enum CapabilitySnapshotError {
    #[error("failed to read capability snapshot from `{path}`: {source}")]
    ReadSnapshot {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write capability snapshot to `{path}`: {source}")]
    WriteSnapshot {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to decode capability snapshot from JSON: {source}")]
    JsonDecode {
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to encode capability snapshot to JSON: {source}")]
    JsonEncode {
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to decode capability snapshot from TOML: {source}")]
    TomlDecode {
        #[source]
        source: toml::de::Error,
    },
    #[error("failed to encode capability snapshot to TOML: {source}")]
    TomlEncode {
        #[source]
        source: toml::ser::Error,
    },
    #[error("unsupported capability snapshot format for `{path}`; use .json/.toml or supply a format explicitly")]
    UnsupportedFormat { path: PathBuf },
}
