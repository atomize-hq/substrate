use agent_api_types::{PolicySnapshotV2, WorldFsEnforcementV2};
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

impl From<WorldFsEnforcementV2> for EnforcementPlanModeV1 {
    fn from(value: WorldFsEnforcementV2) -> Self {
        match value {
            WorldFsEnforcementV2::Strict => Self::Strict,
            WorldFsEnforcementV2::BestEffort => Self::BestEffort,
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

pub(crate) fn maybe_encode_from_snapshot(snapshot: &PolicySnapshotV2) -> Result<Option<String>> {
    let read_deny = snapshot
        .world_fs
        .read
        .as_ref()
        .map(|d| d.deny_list.clone())
        .unwrap_or_default();
    let discover_deny = snapshot
        .world_fs
        .discover
        .as_ref()
        .map(|d| d.deny_list.clone())
        .unwrap_or_else(|| read_deny.clone());
    let write_deny = snapshot
        .world_fs
        .write
        .as_ref()
        .map(|d| d.deny_list.clone())
        .unwrap_or_default();

    let any_deny = !read_deny.is_empty() || !discover_deny.is_empty() || !write_deny.is_empty();
    if !any_deny {
        return Ok(None);
    }

    let enforcement = snapshot
        .world_fs
        .enforcement
        .ok_or_else(|| anyhow!("world_fs.enforcement missing for deny_list configuration"))?;

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
