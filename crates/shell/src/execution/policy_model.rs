use crate::execution::config_model::{self, ConfigUpdate, UpdateOp};
use crate::execution::value_parse::parse_bool_flag;
use crate::execution::workspace;
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use substrate_broker::{Policy, WorldFsIsolation};
use substrate_common::WorldFsMode;

#[derive(Debug, Clone)]
pub(crate) enum PolicySource {
    Workspace,
    Global,
    Default,
}

pub(crate) fn global_policy_path() -> Result<PathBuf> {
    substrate_common::paths::policy_file()
}

pub(crate) fn workspace_policy_path(workspace_root: &Path) -> PathBuf {
    workspace::workspace_policy_path(workspace_root)
}

pub(crate) fn load_global_policy_or_defaults() -> Result<(Policy, bool)> {
    let path = global_policy_path()?;
    match fs::read_to_string(&path) {
        Ok(raw) => Ok((parse_policy_yaml(&path, &raw)?, true)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok((Policy::default(), false)),
        Err(err) => Err(anyhow!("failed to read {}: {err}", path.display())),
    }
}

pub(crate) fn load_effective_policy(cwd: &Path) -> Result<(Policy, PolicySource)> {
    if let Some(workspace_root) = workspace::find_workspace_root(cwd) {
        let path = workspace_policy_path(&workspace_root);
        if path.exists() {
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let policy = parse_policy_yaml(&path, &raw)?;
            return Ok((policy, PolicySource::Workspace));
        }
    }

    let global_path = global_policy_path()?;
    if global_path.exists() {
        let raw = fs::read_to_string(&global_path)
            .with_context(|| format!("failed to read {}", global_path.display()))?;
        let policy = parse_policy_yaml(&global_path, &raw)?;
        return Ok((policy, PolicySource::Global));
    }

    Ok((Policy::default(), PolicySource::Default))
}

pub(crate) fn parse_policy_yaml(path: &Path, raw: &str) -> Result<Policy> {
    let parsed: Policy = serde_yaml::from_str(raw).map_err(|err| {
        config_model::user_error(format!(
            "invalid YAML in {}: {}",
            path.display(),
            err.to_string().trim()
        ))
    })?;
    validate_policy(&parsed)?;
    Ok(parsed)
}

pub(crate) fn apply_updates(policy: &mut Policy, updates: &[ConfigUpdate]) -> Result<bool> {
    let mut changed = false;
    for update in updates {
        changed |= apply_update(policy, update)?;
    }
    validate_policy(policy)?;
    Ok(changed)
}

fn validate_policy(policy: &Policy) -> Result<()> {
    if policy.world_fs_mode == WorldFsMode::ReadOnly && !policy.world_fs_require_world {
        return Err(config_model::user_error(
            "world_fs.mode=read_only requires world_fs.require_world=true",
        ));
    }
    if policy.world_fs_isolation == WorldFsIsolation::Full && !policy.world_fs_require_world {
        return Err(config_model::user_error(
            "world_fs.isolation=full requires world_fs.require_world=true",
        ));
    }
    Ok(())
}

fn apply_update(policy: &mut Policy, update: &ConfigUpdate) -> Result<bool> {
    match update.key.as_str() {
        "id" => apply_string(&mut policy.id, &update.op, &update.value),
        "name" => apply_string(&mut policy.name, &update.op, &update.value),
        "world_fs.mode" => {
            apply_enum_world_fs_mode(&mut policy.world_fs_mode, &update.op, &update.value)
        }
        "world_fs.isolation" => {
            apply_enum_world_fs_isolation(&mut policy.world_fs_isolation, &update.op, &update.value)
        }
        "world_fs.require_world" => apply_bool(
            &mut policy.world_fs_require_world,
            &update.op,
            &update.value,
        ),
        "world_fs.read_allowlist" => apply_string_list(&mut policy.fs_read, update),
        "world_fs.write_allowlist" => apply_string_list(&mut policy.fs_write, update),
        "net_allowed" => apply_string_list(&mut policy.net_allowed, update),
        "cmd_allowed" => apply_string_list(&mut policy.cmd_allowed, update),
        "cmd_denied" => apply_string_list(&mut policy.cmd_denied, update),
        "cmd_isolated" => apply_string_list(&mut policy.cmd_isolated, update),
        "require_approval" => apply_bool(&mut policy.require_approval, &update.op, &update.value),
        "allow_shell_operators" => {
            apply_bool(&mut policy.allow_shell_operators, &update.op, &update.value)
        }
        "limits.max_memory_mb" => {
            apply_u64_opt(&mut policy.limits.max_memory_mb, &update.op, &update.value)
        }
        "limits.max_cpu_percent" => apply_u32_opt(
            &mut policy.limits.max_cpu_percent,
            &update.op,
            &update.value,
        ),
        "limits.max_runtime_ms" => {
            apply_u64_opt(&mut policy.limits.max_runtime_ms, &update.op, &update.value)
        }
        "limits.max_egress_bytes" => apply_u64_opt(
            &mut policy.limits.max_egress_bytes,
            &update.op,
            &update.value,
        ),
        "metadata" => apply_metadata(&mut policy.metadata, &update.op, &update.value),
        other => Err(config_model::user_error(format!(
            "unsupported policy key '{}'",
            other
        ))),
    }
}

fn apply_string(target: &mut String, op: &UpdateOp, raw: &str) -> Result<bool> {
    match op {
        UpdateOp::Set => {
            if target == raw {
                return Ok(false);
            }
            *target = raw.to_string();
            Ok(true)
        }
        UpdateOp::Append | UpdateOp::Remove => Err(config_model::user_error(
            "operator += and -= are only valid for list keys",
        )),
    }
}

fn apply_bool(target: &mut bool, op: &UpdateOp, raw: &str) -> Result<bool> {
    let UpdateOp::Set = op else {
        return Err(config_model::user_error(
            "operator += and -= are only valid for list keys",
        ));
    };
    let next = parse_bool_flag(raw).ok_or_else(|| {
        config_model::user_error(format!("invalid boolean value '{}'", raw.trim()))
    })?;
    if *target == next {
        return Ok(false);
    }
    *target = next;
    Ok(true)
}

fn apply_u64_opt(target: &mut Option<u64>, op: &UpdateOp, raw: &str) -> Result<bool> {
    let UpdateOp::Set = op else {
        return Err(config_model::user_error(
            "operator += and -= are only valid for list keys",
        ));
    };
    let trimmed = raw.trim();
    let next = trimmed.parse::<u64>().map_err(|_| {
        config_model::user_error(format!(
            "invalid integer value '{}' (expected base-10)",
            trimmed
        ))
    })?;
    let next = Some(next);
    if *target == next {
        return Ok(false);
    }
    *target = next;
    Ok(true)
}

fn apply_u32_opt(target: &mut Option<u32>, op: &UpdateOp, raw: &str) -> Result<bool> {
    let UpdateOp::Set = op else {
        return Err(config_model::user_error(
            "operator += and -= are only valid for list keys",
        ));
    };
    let trimmed = raw.trim();
    let next = trimmed.parse::<u32>().map_err(|_| {
        config_model::user_error(format!(
            "invalid integer value '{}' (expected base-10)",
            trimmed
        ))
    })?;
    let next = Some(next);
    if *target == next {
        return Ok(false);
    }
    *target = next;
    Ok(true)
}

fn apply_enum_world_fs_mode(target: &mut WorldFsMode, op: &UpdateOp, raw: &str) -> Result<bool> {
    let UpdateOp::Set = op else {
        return Err(config_model::user_error(
            "operator += and -= are only valid for list keys",
        ));
    };
    let normalized = raw.trim().to_ascii_lowercase();
    let next = match normalized.as_str() {
        "writable" => WorldFsMode::Writable,
        "read_only" => WorldFsMode::ReadOnly,
        _ => {
            return Err(config_model::user_error(format!(
                "invalid world_fs.mode '{}' (expected writable or read_only)",
                raw.trim()
            )));
        }
    };
    if *target == next {
        return Ok(false);
    }
    *target = next;
    Ok(true)
}

fn apply_enum_world_fs_isolation(
    target: &mut WorldFsIsolation,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    let UpdateOp::Set = op else {
        return Err(config_model::user_error(
            "operator += and -= are only valid for list keys",
        ));
    };
    let next = match raw.trim().to_ascii_lowercase().as_str() {
        "workspace" | "project" => WorldFsIsolation::Workspace,
        "full" => WorldFsIsolation::Full,
        _ => {
            return Err(config_model::user_error(format!(
                "invalid world_fs.isolation '{}' (expected workspace or full)",
                raw.trim()
            )));
        }
    };
    if *target == next {
        return Ok(false);
    }
    *target = next;
    Ok(true)
}

fn apply_string_list(target: &mut Vec<String>, update: &ConfigUpdate) -> Result<bool> {
    match update.op {
        UpdateOp::Set => {
            let parsed = parse_yaml_string_list(&update.value)?;
            if *target == parsed {
                return Ok(false);
            }
            *target = parsed;
            Ok(true)
        }
        UpdateOp::Append => {
            target.push(update.value.clone());
            Ok(true)
        }
        UpdateOp::Remove => {
            let before = target.len();
            target.retain(|item| item != &update.value);
            Ok(before != target.len())
        }
    }
}

fn parse_yaml_string_list(raw: &str) -> Result<Vec<String>> {
    let parsed: Vec<String> = serde_yaml::from_str(raw).map_err(|err| {
        config_model::user_error(format!(
            "invalid YAML list literal '{}': {}",
            raw.trim(),
            err.to_string().trim()
        ))
    })?;
    Ok(parsed)
}

fn apply_metadata(target: &mut HashMap<String, String>, op: &UpdateOp, raw: &str) -> Result<bool> {
    match op {
        UpdateOp::Set => {
            let parsed: HashMap<String, String> = serde_yaml::from_str(raw).map_err(|err| {
                config_model::user_error(format!(
                    "invalid YAML mapping literal for metadata '{}': {}",
                    raw.trim(),
                    err.to_string().trim()
                ))
            })?;
            if *target == parsed {
                return Ok(false);
            }
            *target = parsed;
            Ok(true)
        }
        UpdateOp::Append | UpdateOp::Remove => Err(config_model::user_error(
            "metadata+= and metadata-= are not allowed (use metadata=... to replace the full mapping)",
        )),
    }
}
