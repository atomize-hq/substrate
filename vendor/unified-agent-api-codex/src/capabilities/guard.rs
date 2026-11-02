use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use tracing::warn;

use super::{CapabilityCachePolicy, CodexCapabilities, CodexFeatureFlags, CodexVersionInfo};

/// Result of applying a TTL/backoff window to a capability snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilityTtlDecision {
    /// True when the snapshot is outside the TTL window and callers should re-run probes.
    pub should_probe: bool,
    /// Recommended cache policy for the next probe (`Refresh` when fingerprints exist, `Bypass` when metadata is missing).
    pub policy: CapabilityCachePolicy,
}

/// Decides whether a cached capability snapshot should be refreshed based on `collected_at`.
///
/// Callers can use this to apply a TTL/backoff in environments where filesystem metadata is
/// missing or unreliable (e.g., FUSE/overlay filesystems) and when binaries are hot-swapped
/// without changing fingerprints. When the TTL has not elapsed, reuse the provided snapshot;
/// when expired, force a probe with [`CapabilityCachePolicy::Refresh`] (fingerprints present)
/// or [`CapabilityCachePolicy::Bypass`] (metadata missing).
///
/// Recommended defaults: start with a 5 minute TTL when fingerprints exist and prefer
/// `Refresh` for hot-swaps that reuse the same path; when metadata is missing, expect `Bypass`
/// and back off further (e.g., stretch the TTL toward 10-15 minutes) to avoid tight probe loops.
pub fn capability_cache_ttl_decision(
    snapshot: Option<&CodexCapabilities>,
    ttl: Duration,
    now: SystemTime,
) -> CapabilityTtlDecision {
    let default_policy = CapabilityCachePolicy::PreferCache;
    let Some(snapshot) = snapshot else {
        return CapabilityTtlDecision {
            should_probe: true,
            policy: default_policy,
        };
    };

    let expired = now
        .duration_since(snapshot.collected_at)
        .map(|elapsed| elapsed >= ttl)
        .unwrap_or(true);

    if !expired {
        return CapabilityTtlDecision {
            should_probe: false,
            policy: default_policy,
        };
    }

    let policy = if snapshot.fingerprint.is_some() {
        CapabilityCachePolicy::Refresh
    } else {
        CapabilityCachePolicy::Bypass
    };

    CapabilityTtlDecision {
        should_probe: true,
        policy,
    }
}

/// High-level view of whether a specific feature can be used safely.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilitySupport {
    Supported,
    Unsupported,
    Unknown,
}

impl CapabilitySupport {
    /// True when it is safe to enable the guarded feature or flag.
    pub const fn is_supported(self) -> bool {
        matches!(self, CapabilitySupport::Supported)
    }

    /// True when support could not be confirmed due to missing probes.
    pub const fn is_unknown(self) -> bool {
        matches!(self, CapabilitySupport::Unknown)
    }
}

/// Feature/flag tokens that can be guarded based on probed capabilities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityFeature {
    OutputSchema,
    AddDir,
    McpLogin,
    FeaturesList,
}

impl CapabilityFeature {
    fn label(self) -> &'static str {
        match self {
            CapabilityFeature::OutputSchema => "--output-schema",
            CapabilityFeature::AddDir => "codex add-dir",
            CapabilityFeature::McpLogin => "codex login --mcp",
            CapabilityFeature::FeaturesList => "codex features list",
        }
    }
}

/// Result of gating a Codex feature/flag against probed capabilities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityGuard {
    /// Feature being checked.
    pub feature: CapabilityFeature,
    /// Whether the feature is safe to enable.
    pub support: CapabilitySupport,
    /// Notes explaining how the guard was derived.
    pub notes: Vec<String>,
}

impl CapabilityGuard {
    fn supported(feature: CapabilityFeature, note: impl Into<String>) -> Self {
        CapabilityGuard {
            feature,
            support: CapabilitySupport::Supported,
            notes: vec![note.into()],
        }
    }

    fn unsupported(feature: CapabilityFeature, note: impl Into<String>) -> Self {
        CapabilityGuard {
            feature,
            support: CapabilitySupport::Unsupported,
            notes: vec![note.into()],
        }
    }

    fn unknown(feature: CapabilityFeature, notes: Vec<String>) -> Self {
        CapabilityGuard {
            feature,
            support: CapabilitySupport::Unknown,
            notes,
        }
    }

    /// Convenience wrapper for `support.is_supported()`.
    pub const fn is_supported(&self) -> bool {
        self.support.is_supported()
    }

    /// Convenience wrapper for `support.is_unknown()`.
    pub const fn is_unknown(&self) -> bool {
        self.support.is_unknown()
    }
}

/// Description of how we interrogate the CLI to populate a [`CodexCapabilities`] snapshot.
///
/// Probes should prefer an explicit feature list when available, fall back to parsing
/// `codex --help` flags, and finally rely on coarse version heuristics. Each attempted
/// step is recorded so hosts can trace why a particular flag was enabled or skipped.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct CapabilityProbePlan {
    /// Steps attempted in order; consumers should push entries as probes run.
    pub steps: Vec<CapabilityProbeStep>,
}

/// Command-level probes used to infer feature support.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CapabilityProbeStep {
    /// Invoke `codex --version` to capture version/build metadata.
    VersionFlag,
    /// Prefer `codex features list --json` when supported for structured output.
    FeaturesListJson,
    /// Fallback to `codex features list` when only plain text is available.
    FeaturesListText,
    /// Parse `codex --help` to spot known flags (e.g., `--output-schema`, `add-dir`, `login --mcp`) when the features list is missing.
    HelpFallback,
    /// Caller-supplied capability overrides were applied to the snapshot.
    ManualOverride,
}

impl CodexCapabilities {
    /// Guards whether `--output-schema` should be passed to `codex exec`.
    pub fn guard_output_schema(&self) -> CapabilityGuard {
        self.guard_feature(CapabilityFeature::OutputSchema)
    }

    /// Guards whether `codex add-dir` can be invoked safely.
    pub fn guard_add_dir(&self) -> CapabilityGuard {
        self.guard_feature(CapabilityFeature::AddDir)
    }

    /// Guards whether `codex login --mcp` is available.
    pub fn guard_mcp_login(&self) -> CapabilityGuard {
        self.guard_feature(CapabilityFeature::McpLogin)
    }

    /// Guards whether `codex features list` is supported by the probed binary.
    pub fn guard_features_list(&self) -> CapabilityGuard {
        self.guard_feature(CapabilityFeature::FeaturesList)
    }

    /// Returns a guard describing if a feature/flag is supported by the probed binary.
    ///
    /// The guard treats missing `features list` support as `Unknown` so hosts can
    /// degrade gracefully on older binaries instead of passing unsupported flags.
    pub fn guard_feature(&self, feature: CapabilityFeature) -> CapabilityGuard {
        guard_feature_support(feature, &self.features, self.version.as_ref())
    }
}

fn guard_feature_support(
    feature: CapabilityFeature,
    flags: &CodexFeatureFlags,
    version: Option<&CodexVersionInfo>,
) -> CapabilityGuard {
    let supported = match feature {
        CapabilityFeature::OutputSchema => flags.supports_output_schema,
        CapabilityFeature::AddDir => flags.supports_add_dir,
        CapabilityFeature::McpLogin => flags.supports_mcp_login,
        CapabilityFeature::FeaturesList => flags.supports_features_list,
    };

    if supported {
        return CapabilityGuard::supported(
            feature,
            format!("Support for {} reported by Codex probe.", feature.label()),
        );
    }

    if feature == CapabilityFeature::FeaturesList {
        let mut notes = vec![format!(
            "Support for {} could not be confirmed; feature list probes failed or were unavailable.",
            feature.label()
        )];
        if version.is_none() {
            notes.push(
                "Version was unavailable; assuming compatibility with older Codex builds."
                    .to_string(),
            );
        }
        return CapabilityGuard::unknown(feature, notes);
    }

    if flags.supports_features_list {
        return CapabilityGuard::unsupported(
            feature,
            format!(
                "`{}` did not advertise {}; skipping related flag to stay compatible.",
                CapabilityFeature::FeaturesList.label(),
                feature.label()
            ),
        );
    }

    let mut notes = vec![format!(
        "Support for {} is unknown because {} is unavailable; disable the flag for compatibility.",
        feature.label(),
        CapabilityFeature::FeaturesList.label()
    )];
    if version.is_none() {
        notes.push(
            "Version could not be parsed; treating feature support conservatively to avoid CLI errors."
                .to_string(),
        );
    }

    CapabilityGuard::unknown(feature, notes)
}

pub(crate) fn guard_is_supported(guard: &CapabilityGuard) -> bool {
    matches!(guard.support, CapabilitySupport::Supported)
}

pub(crate) fn log_guard_skip(guard: &CapabilityGuard) {
    warn!(
        feature = guard.feature.label(),
        support = ?guard.support,
        notes = ?guard.notes,
        "Skipping requested Codex capability because support was not confirmed"
    );
}
