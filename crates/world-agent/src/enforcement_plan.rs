use agent_api_types::{PolicySnapshotV3, WorldFsDenyEnforcementV3};
use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::{Deserialize, Serialize};

pub(crate) const WORLD_FS_ENFORCEMENT_PLAN_B64_ENV: &str =
    "SUBSTRATE_WORLD_FS_ENFORCEMENT_PLAN_B64";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EnforcementPlanModeV1 {
    Strict,
    BestEffort,
}

impl From<WorldFsDenyEnforcementV3> for EnforcementPlanModeV1 {
    fn from(value: WorldFsDenyEnforcementV3) -> Self {
        match value {
            WorldFsDenyEnforcementV3::Strict => Self::Strict,
            WorldFsDenyEnforcementV3::PreferStrict | WorldFsDenyEnforcementV3::Weak => {
                Self::BestEffort
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EnforcementPlanV1 {
    pub(crate) version: u32,
    pub(crate) enforcement: EnforcementPlanModeV1,
    pub(crate) read_deny: Vec<String>,
    pub(crate) discover_deny: Vec<String>,
    pub(crate) write_deny: Vec<String>,
}

pub(crate) fn maybe_encode_from_snapshot(snapshot: &PolicySnapshotV3) -> Result<Option<String>> {
    let canonical = snapshot
        .canonicalize()
        .map_err(|err| anyhow!("invalid PolicySnapshotV3: {err}"))?;

    let read_deny = canonical
        .world_fs
        .read
        .as_ref()
        .map(|d| d.deny_list.clone())
        .unwrap_or_default();
    let discover_deny = canonical
        .world_fs
        .discover
        .as_ref()
        .map(|d| d.deny_list.clone())
        .unwrap_or_else(|| read_deny.clone());
    let write_deny = canonical.world_fs.write.deny_list.clone();

    let any_deny = !read_deny.is_empty() || !discover_deny.is_empty() || !write_deny.is_empty();
    if !any_deny {
        return Ok(None);
    }

    let enforcement = canonical
        .world_fs
        .deny_enforcement
        .ok_or_else(|| anyhow!("world_fs.deny_enforcement missing for deny_list configuration"))?;

    let plan = EnforcementPlanV1 {
        version: 1,
        enforcement: enforcement.into(),
        read_deny,
        discover_deny,
        write_deny,
    };
    validate_plan(&plan).context("validate enforcement plan")?;

    let json_bytes = serde_json::to_vec(&plan).context("serialize enforcement plan JSON")?;
    Ok(Some(BASE64.encode(json_bytes)))
}

pub(crate) fn read_from_env_and_validate() -> Result<Option<EnforcementPlanV1>> {
    let raw = match std::env::var(WORLD_FS_ENFORCEMENT_PLAN_B64_ENV) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(anyhow!(
            "env {WORLD_FS_ENFORCEMENT_PLAN_B64_ENV} was set but empty"
        ));
    }

    let bytes = BASE64
        .decode(trimmed.as_bytes())
        .with_context(|| format!("base64 decode {WORLD_FS_ENFORCEMENT_PLAN_B64_ENV}"))?;
    let json = std::str::from_utf8(&bytes)
        .with_context(|| format!("decoded {WORLD_FS_ENFORCEMENT_PLAN_B64_ENV} was not UTF-8"))?;
    let plan: EnforcementPlanV1 =
        serde_json::from_str(json).context("parse enforcement plan JSON")?;
    validate_plan(&plan).context("validate enforcement plan")?;
    Ok(Some(plan))
}

fn validate_plan(plan: &EnforcementPlanV1) -> Result<()> {
    if plan.version != 1 {
        anyhow::bail!(
            "unsupported enforcement plan version: {} (expected 1)",
            plan.version
        );
    }

    validate_deny_list("read_deny", &plan.read_deny)?;
    validate_deny_list("discover_deny", &plan.discover_deny)?;
    validate_deny_list("write_deny", &plan.write_deny)?;

    Ok(())
}

fn validate_deny_list(prefix: &str, patterns: &[String]) -> Result<()> {
    for raw in patterns {
        let normalized = normalize_project_pattern(raw)
            .with_context(|| format!("{prefix}: invalid deny pattern"))?;
        if contains_unsupported_deny_metacharacters(&normalized) {
            anyhow::bail!(
                "{prefix}: deny pattern contains unsupported glob metacharacters ('?' or character classes)"
            );
        }
        validate_deny_wildcards(&normalized)
            .with_context(|| format!("{prefix}: invalid wildcard run"))?;
    }
    Ok(())
}

fn normalize_project_pattern(raw: &str) -> Result<String> {
    let mut pattern = raw.trim();
    if pattern.is_empty() {
        anyhow::bail!("pattern must be non-empty");
    }
    if pattern.starts_with('/') {
        anyhow::bail!("absolute paths are not allowed");
    }

    while let Some(stripped) = pattern.strip_prefix("./") {
        pattern = stripped;
    }

    let mut normalized = pattern.trim_end_matches('/').to_string();
    if normalized.is_empty() {
        normalized = ".".to_string();
    }

    if normalized.split('/').any(|segment| segment == "..") {
        anyhow::bail!("path segments must not be '..'");
    }

    Ok(normalized)
}

fn contains_unsupported_deny_metacharacters(value: &str) -> bool {
    value.contains('?') || value.contains('[') || value.contains(']')
}

fn validate_deny_wildcards(pattern: &str) -> Result<()> {
    let mut run = 0usize;
    for ch in pattern.chars() {
        if ch == '*' {
            run += 1;
            continue;
        }
        if run > 0 && run != 1 && run != 2 {
            anyhow::bail!("wildcard runs must be '*' or '**' (no '***' or longer)");
        }
        run = 0;
    }
    if run > 0 && run != 1 && run != 2 {
        anyhow::bail!("wildcard runs must be '*' or '**' (no '***' or longer)");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_api_types::{
        PolicySnapshotV3, PolicySnapshotWorldFsDimensionV3, PolicySnapshotWorldFsFailClosedV3,
        PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3, WorldFsDenyEnforcementV3,
    };
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn minimal_snapshot_full_read_only() -> PolicySnapshotV3 {
        PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: Vec::new(),
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: false,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: true },
                deny_enforcement: None,
                caged_required: false,
                discover: None,
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec!["src".to_string()],
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: false,
                    allow_list: vec!["src".to_string()],
                    deny_list: Vec::new(),
                },
            },
        }
    }

    #[test]
    fn encode_returns_none_when_no_deny_lists() {
        let snapshot = minimal_snapshot_full_read_only();
        snapshot.validate().expect("snapshot validates");
        let encoded = maybe_encode_from_snapshot(&snapshot).expect("encode succeeds");
        assert!(encoded.is_none(), "expected no plan when deny lists empty");
    }

    #[test]
    fn encode_defaults_discover_deny_to_read_deny() {
        let mut snapshot = minimal_snapshot_full_read_only();
        snapshot.world_fs.deny_enforcement = Some(WorldFsDenyEnforcementV3::Strict);
        snapshot
            .world_fs
            .read
            .as_mut()
            .expect("read present")
            .deny_list = vec!["**/*.pem".to_string()];
        snapshot.validate().expect("snapshot validates");

        let encoded = maybe_encode_from_snapshot(&snapshot)
            .expect("encode succeeds")
            .expect("plan should be present");
        let bytes = BASE64.decode(encoded.as_bytes()).expect("valid base64");
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).expect("valid JSON enforcement plan");

        assert_eq!(value["version"], 1);
        assert_eq!(value["enforcement"], "strict");
        assert_eq!(value["read_deny"], serde_json::json!(["**/*.pem"]));
        assert_eq!(value["discover_deny"], serde_json::json!(["**/*.pem"]));
        assert_eq!(value["write_deny"], serde_json::json!([]));
    }

    #[test]
    fn encode_does_not_default_discover_deny_when_discover_present() {
        let mut snapshot = minimal_snapshot_full_read_only();
        snapshot.world_fs.deny_enforcement = Some(WorldFsDenyEnforcementV3::Strict);
        snapshot
            .world_fs
            .read
            .as_mut()
            .expect("read present")
            .deny_list = vec!["secrets/secret.txt".to_string()];
        snapshot.world_fs.discover = Some(PolicySnapshotWorldFsDimensionV3 {
            allow_list: vec![".".to_string()],
            deny_list: vec![],
        });
        snapshot.validate().expect("snapshot validates");

        let encoded = maybe_encode_from_snapshot(&snapshot)
            .expect("encode succeeds")
            .expect("plan should be present");
        let bytes = BASE64.decode(encoded.as_bytes()).expect("valid base64");
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).expect("valid JSON enforcement plan");

        assert_eq!(
            value["read_deny"],
            serde_json::json!(["secrets/secret.txt"])
        );
        assert_eq!(value["discover_deny"], serde_json::json!([]));
    }

    #[test]
    fn read_from_env_rejects_unknown_fields() {
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();

        let plan = serde_json::json!({
          "version": 1,
          "enforcement": "strict",
          "read_deny": ["**/*.pem"],
          "discover_deny": ["**/*.pem"],
          "write_deny": [],
          "extra": "nope"
        });
        let encoded = BASE64.encode(serde_json::to_vec(&plan).unwrap());
        std::env::set_var(WORLD_FS_ENFORCEMENT_PLAN_B64_ENV, encoded);
        let err = read_from_env_and_validate()
            .expect_err("unknown fields must be rejected (deny_unknown_fields)");
        std::env::remove_var(WORLD_FS_ENFORCEMENT_PLAN_B64_ENV);

        let msg = format!("{err:#}");
        assert!(
            msg.contains("unknown field") || msg.contains("unknown variant"),
            "unexpected error: {msg}"
        );
    }
}
