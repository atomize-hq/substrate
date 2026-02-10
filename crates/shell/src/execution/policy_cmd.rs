use crate::execution::cli::{
    Cli, ConfigResetArgs, PolicyAction, PolicyCmd, PolicyCurrentAction, PolicyGlobalAction,
    PolicyGlobalCmd, PolicyInitArgs, PolicySetArgs, PolicyShowArgs, PolicyWorkspaceAction,
    PolicyWorkspaceCmd,
};
use crate::execution::config_model;
use crate::execution::policy_model::PolicyPatch;
use crate::execution::{policy_model, workspace};
use anyhow::{anyhow, Context, Result};
use serde::ser::SerializeMap;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use substrate_broker::{Policy, PolicyExplainV1, WorldFsDenyEnforcement};
use tempfile::NamedTempFile;

const DEFAULT_GLOBAL_POLICY_PATCH_HEADER: &str = r#"# Substrate policy patch (sparse overrides; scope=global).
# - This file is a YAML mapping of global-scoped policy overrides.
# - Omitted keys inherit from defaults (and from workspace overrides when applicable).
# - View the effective merged policy with: `substrate policy current show --explain`
"#;

const DEFAULT_WORKSPACE_POLICY_PATCH_HEADER: &str = r#"# Substrate policy patch (sparse overrides; scope=workspace).
# - This file is a YAML mapping of workspace-scoped policy overrides.
# - Omitted keys inherit from global policy + defaults.
# - View the effective merged policy with: `substrate policy current show --explain`
"#;

pub(crate) fn handle_policy_command(cmd: &PolicyCmd, _cli: &Cli) -> i32 {
    let result = match &cmd.action {
        PolicyAction::Current(cmd) => match &cmd.action {
            PolicyCurrentAction::Show(args) => run_current_show(args),
        },
        PolicyAction::Init(args) => run_workspace_init(args),
        PolicyAction::Show(args) => run_current_show(args),
        PolicyAction::Set(args) => run_workspace_set(args),
        PolicyAction::Global(cmd) => run_global(cmd),
        PolicyAction::Workspace(cmd) => run_workspace(cmd),
    };

    match result {
        Ok(()) => 0,
        Err(err) if err.is::<ActionableError>() => {
            eprintln!("{:#}", err);
            2
        }
        Err(err) if config_model::is_user_error(&err) => {
            eprintln!("{err}");
            2
        }
        Err(err) => {
            eprintln!("{:#}", err);
            1
        }
    }
}

#[derive(Debug)]
struct ActionableError(String);

impl std::fmt::Display for ActionableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ActionableError {}

fn actionable(message: impl Into<String>) -> anyhow::Error {
    anyhow::Error::new(ActionableError(message.into()))
}

fn run_global(cmd: &PolicyGlobalCmd) -> Result<()> {
    match &cmd.action {
        PolicyGlobalAction::Init(args) => run_global_init(args),
        PolicyGlobalAction::Show(args) => run_global_show(args),
        PolicyGlobalAction::Set(args) => run_global_set(args),
        PolicyGlobalAction::Reset(args) => run_global_reset(args),
    }
}

fn run_workspace(cmd: &PolicyWorkspaceCmd) -> Result<()> {
    match &cmd.action {
        PolicyWorkspaceAction::Show(args) => run_workspace_show(args),
        PolicyWorkspaceAction::Set(args) => run_workspace_set(args),
        PolicyWorkspaceAction::Reset(args) => run_workspace_reset(args),
    }
}

fn run_global_init(args: &PolicyInitArgs) -> Result<()> {
    let path = policy_model::global_policy_path()?;
    let existed = path.exists();

    if existed && !args.force {
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let _ = policy_model::parse_policy_patch_yaml(&path, &raw)?;
        return Ok(());
    }

    let patch = PolicyPatch::default();
    write_atomic_patch_yaml(&path, DEFAULT_GLOBAL_POLICY_PATCH_HEADER, None, &patch)
        .with_context(|| format!("failed to write {}", path.display()))?;
    crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();

    if existed {
        println!(
            "substrate: overwrote policy patch at {} (--force)",
            path.display()
        );
    } else {
        println!("substrate: wrote global policy patch to {}", path.display());
    }

    Ok(())
}

fn run_global_show(args: &PolicyShowArgs) -> Result<()> {
    if args.explain {
        return Err(config_model::user_error(
            "--explain is only supported for `substrate policy current show`",
        ));
    }

    let (patch, _) = policy_model::read_global_policy_patch_or_empty()?;
    if patch.is_empty() {
        eprintln!("substrate: note: global policy patch is empty (no overrides); run 'substrate policy current show --explain' to view the effective policy for this directory");
    }
    print_patch(&patch, args.json)?;
    Ok(())
}

fn run_global_set(args: &PolicySetArgs) -> Result<()> {
    let path = policy_model::global_policy_path()?;
    let old_raw = fs::read_to_string(&path).ok();
    let (mut patch, existed) = policy_model::read_global_policy_patch_or_empty()
        .with_context(|| format!("failed to load global policy patch at {}", path.display()))?;

    let updates = config_model::parse_updates(&args.updates)?;
    let changed = policy_model::apply_updates_to_policy_patch(&mut patch, &updates)?;

    if changed || !existed {
        let header = if existed {
            Some(read_comment_header_prefix(&path)?)
        } else {
            None
        };
        if let Err(err) = write_atomic_patch_yaml(
            &path,
            DEFAULT_GLOBAL_POLICY_PATCH_HEADER,
            header.as_deref(),
            &patch,
        )
        .with_context(|| format!("failed to write {}", path.display()))
        {
            rollback_policy_file(&path, old_raw.as_deref());
            crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
            return Err(err);
        }
        crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let (effective, _) = match substrate_broker::resolve_effective_policy_with_explain(&cwd, false)
    {
        Ok(v) => v,
        Err(err) => {
            rollback_policy_file(&path, old_raw.as_deref());
            crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
            return Err(config_model::user_error(err.to_string()));
        }
    };
    print_policy(&effective, args.json)?;
    Ok(())
}

fn run_global_reset(args: &ConfigResetArgs) -> Result<()> {
    let path = policy_model::global_policy_path()?;
    let old_raw = fs::read_to_string(&path).ok();
    let (mut patch, existed) = policy_model::read_global_policy_patch_or_empty()
        .with_context(|| format!("failed to load global policy patch at {}", path.display()))?;
    let changed = policy_model::reset_policy_patch_keys(&mut patch, &args.keys)?;

    if changed || !existed {
        let header = if existed {
            Some(read_comment_header_prefix(&path)?)
        } else {
            None
        };
        if let Err(err) = write_atomic_patch_yaml(
            &path,
            DEFAULT_GLOBAL_POLICY_PATCH_HEADER,
            header.as_deref(),
            &patch,
        )
        .with_context(|| format!("failed to write {}", path.display()))
        {
            rollback_policy_file(&path, old_raw.as_deref());
            crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
            return Err(err);
        }
        crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let (effective, _) = match substrate_broker::resolve_effective_policy_with_explain(&cwd, false)
    {
        Ok(v) => v,
        Err(err) => {
            rollback_policy_file(&path, old_raw.as_deref());
            crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
            return Err(config_model::user_error(err.to_string()));
        }
    };
    print_policy(&effective, false)?;
    Ok(())
}

fn run_workspace_init(args: &PolicyInitArgs) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace_root = require_workspace(&cwd)?;
    let path = policy_model::workspace_policy_path(&workspace_root);
    let existed = path.exists();

    if existed && !args.force {
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let _ = policy_model::parse_policy_patch_yaml(&path, &raw)?;
        return Ok(());
    }

    let patch = PolicyPatch::default();
    write_atomic_patch_yaml(
        &path,
        DEFAULT_WORKSPACE_POLICY_PATCH_HEADER,
        (if existed {
            Some(read_comment_header_prefix(&path)?)
        } else {
            None
        })
        .as_deref(),
        &patch,
    )
    .with_context(|| format!("failed to write {}", path.display()))?;
    crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();

    if existed {
        println!(
            "substrate: overwrote workspace policy patch at {} (--force)",
            path.display()
        );
    } else {
        println!(
            "substrate: wrote workspace policy patch to {}",
            path.display()
        );
    }

    Ok(())
}

fn run_workspace_show(args: &PolicyShowArgs) -> Result<()> {
    if args.explain {
        return Err(config_model::user_error(
            "--explain is only supported for `substrate policy current show`",
        ));
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace_root = require_workspace(&cwd)?;
    let path = policy_model::workspace_policy_path(&workspace_root);

    let raw = fs::read_to_string(&path).with_context(|| {
        format!(
            "failed to read {}; run `substrate workspace init --force` to repair the workspace",
            path.display()
        )
    })?;
    let patch = policy_model::parse_policy_patch_yaml(&path, &raw)?;
    if patch.is_empty() {
        eprintln!("substrate: note: workspace policy patch is empty (no overrides); run 'substrate policy current show --explain' to view the effective policy for this directory");
    }
    print_patch(&patch, args.json)?;
    Ok(())
}

fn run_current_show(args: &PolicyShowArgs) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if !args.explain {
        eprintln!(
            "substrate: note: showing effective merged policy; use --explain to view per-key sources"
        );
    }
    let (policy, explain) =
        substrate_broker::resolve_effective_policy_with_explain(&cwd, args.explain)
            .map_err(|err| config_model::user_error(err.to_string()))?;
    if let Some(explain) = explain {
        print_explain(&explain)?;
    }
    print_policy(&policy, args.json)?;
    Ok(())
}

fn run_workspace_set(args: &PolicySetArgs) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace_root = require_workspace(&cwd)?;
    let path = policy_model::workspace_policy_path(&workspace_root);

    let raw = fs::read_to_string(&path).with_context(|| {
        format!(
            "failed to read {}; run `substrate workspace init --force` to repair the workspace",
            path.display()
        )
    })?;
    let header = read_comment_header_prefix_from_raw(&raw);
    let old_raw = raw.clone();
    let mut patch = policy_model::parse_policy_patch_yaml(&path, &raw)?;

    let updates = config_model::parse_updates(&args.updates)?;
    let changed = policy_model::apply_updates_to_policy_patch(&mut patch, &updates)?;
    if changed {
        if let Err(err) = write_atomic_patch_yaml(&path, "", Some(&header), &patch)
            .with_context(|| format!("failed to write {}", path.display()))
        {
            rollback_policy_file(&path, Some(&old_raw));
            crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
            return Err(err);
        }
        crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
    }

    let (effective, _) = match substrate_broker::resolve_effective_policy_with_explain(&cwd, false)
    {
        Ok(v) => v,
        Err(err) => {
            rollback_policy_file(&path, Some(&old_raw));
            crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
            return Err(config_model::user_error(err.to_string()));
        }
    };
    print_policy(&effective, args.json)?;
    Ok(())
}

fn run_workspace_reset(args: &ConfigResetArgs) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace_root = require_workspace(&cwd)?;
    let path = policy_model::workspace_policy_path(&workspace_root);

    let raw = fs::read_to_string(&path).with_context(|| {
        format!(
            "failed to read {}; run `substrate workspace init --force` to repair the workspace",
            path.display()
        )
    })?;
    let header = read_comment_header_prefix_from_raw(&raw);
    let old_raw = raw.clone();
    let mut patch = policy_model::parse_policy_patch_yaml(&path, &raw)?;

    let changed = policy_model::reset_policy_patch_keys(&mut patch, &args.keys)?;
    if changed {
        if let Err(err) = write_atomic_patch_yaml(&path, "", Some(&header), &patch)
            .with_context(|| format!("failed to write {}", path.display()))
        {
            rollback_policy_file(&path, Some(&old_raw));
            crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
            return Err(err);
        }
        crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
    }

    let (effective, _) = match substrate_broker::resolve_effective_policy_with_explain(&cwd, false)
    {
        Ok(v) => v,
        Err(err) => {
            rollback_policy_file(&path, Some(&old_raw));
            crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
            return Err(config_model::user_error(err.to_string()));
        }
    };
    print_policy(&effective, false)?;
    Ok(())
}

fn require_workspace(cwd: &Path) -> Result<PathBuf> {
    workspace::find_workspace_root(cwd).ok_or_else(|| {
        actionable("substrate: not in a workspace; run `substrate workspace init`".to_string())
    })
}

#[derive(Debug, Clone)]
struct SortedMetadata<'a>(&'a HashMap<String, String>);

impl Serialize for SortedMetadata<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut entries: Vec<_> = self.0.iter().collect();
        entries.sort_by(|(a_key, _), (b_key, _)| a_key.cmp(b_key));

        let mut map = serializer.serialize_map(Some(entries.len()))?;
        for (key, value) in entries {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

#[derive(Debug)]
struct EffectivePolicyDisplayV3<'a> {
    policy: &'a Policy,
    world_fs: WorldFsEffectiveDisplayV3,
    metadata: SortedMetadata<'a>,
}

impl Serialize for EffectivePolicyDisplayV3<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let policy = self.policy;
        let mut map = serializer.serialize_map(Some(11))?;
        map.serialize_entry("id", &policy.id)?;
        map.serialize_entry("name", &policy.name)?;
        map.serialize_entry("world_fs", &self.world_fs)?;
        map.serialize_entry("net_allowed", &policy.net_allowed)?;
        map.serialize_entry("cmd_allowed", &policy.cmd_allowed)?;
        map.serialize_entry("cmd_denied", &policy.cmd_denied)?;
        map.serialize_entry("cmd_isolated", &policy.cmd_isolated)?;
        map.serialize_entry("require_approval", &policy.require_approval)?;
        map.serialize_entry("allow_shell_operators", &policy.allow_shell_operators)?;
        map.serialize_entry("limits", &policy.limits)?;
        map.serialize_entry("metadata", &self.metadata)?;
        map.end()
    }
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct WorldFsEffectiveDisplayV3 {
    host_visible: bool,
    fail_closed: WorldFsFailClosedEffectiveDisplayV3,
    #[serde(skip_serializing_if = "Option::is_none")]
    deny_enforcement: Option<WorldFsDenyEnforcement>,
    caged_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    discover: Option<WorldFsDimensionEffectiveDisplayV3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    read: Option<WorldFsDimensionEffectiveDisplayV3>,
    write: WorldFsWriteEffectiveDisplayV3,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct WorldFsFailClosedEffectiveDisplayV3 {
    routing: bool,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct WorldFsDimensionEffectiveDisplayV3 {
    allow_list: Vec<String>,
    deny_list: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct WorldFsWriteEffectiveDisplayV3 {
    enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    allow_list: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deny_list: Option<Vec<String>>,
}

fn display_policy_v3(policy: &Policy) -> Result<EffectivePolicyDisplayV3<'_>> {
    let (discover, read, write_allow_list, write_deny_list) = if policy.world_fs_host_visible {
        (None, None, None, None)
    } else {
        let read = policy
            .world_fs_read
            .as_ref()
            .context("effective policy was missing world_fs.read in full isolation")?;
        let discover = policy
            .world_fs_discover
            .as_ref()
            .or(policy.world_fs_read.as_ref())
            .context("effective policy was missing world_fs.discover in full isolation")?;

        let read = Some(WorldFsDimensionEffectiveDisplayV3 {
            allow_list: read.allow_list.clone(),
            deny_list: read.deny_list.clone(),
        });
        let discover = Some(WorldFsDimensionEffectiveDisplayV3 {
            allow_list: discover.allow_list.clone(),
            deny_list: discover.deny_list.clone(),
        });

        if policy.world_fs_write_enabled {
            let write = policy
                .world_fs_write
                .as_ref()
                .context("effective policy was missing world_fs.write in full isolation")?;
            (
                discover,
                read,
                Some(write.allow_list.clone()),
                Some(write.deny_list.clone()),
            )
        } else {
            // When writes are disabled, keep the V3 shape stable by rendering empty lists.
            (discover, read, Some(Vec::new()), Some(Vec::new()))
        }
    };

    Ok(EffectivePolicyDisplayV3 {
        policy,
        world_fs: WorldFsEffectiveDisplayV3 {
            host_visible: policy.world_fs_host_visible,
            fail_closed: WorldFsFailClosedEffectiveDisplayV3 {
                routing: policy.world_fs_fail_closed_routing,
            },
            deny_enforcement: policy.world_fs_deny_enforcement,
            caged_required: policy.world_fs_caged_required,
            discover,
            read,
            write: WorldFsWriteEffectiveDisplayV3 {
                enabled: policy.world_fs_write_enabled,
                allow_list: write_allow_list,
                deny_list: write_deny_list,
            },
        },
        metadata: SortedMetadata(&policy.metadata),
    })
}

fn print_policy(policy: &Policy, json: bool) -> Result<()> {
    let display = display_policy_v3(policy)?;
    if json {
        println!(
            "{}",
            serde_json::to_string(&display).context("failed to serialize JSON")?
        );
        return Ok(());
    }
    println!(
        "{}",
        serde_yaml::to_string(&display).context("failed to serialize YAML")?
    );
    Ok(())
}

fn print_patch(patch: &PolicyPatch, json: bool) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string(patch).context("failed to serialize JSON")?
        );
        return Ok(());
    }
    println!(
        "{}",
        serde_yaml::to_string(patch).context("failed to serialize YAML")?
    );
    Ok(())
}

fn print_explain(explain: &PolicyExplainV1) -> Result<()> {
    eprintln!(
        "{}",
        serde_json::to_string(explain).context("failed to serialize explain JSON")?
    );
    Ok(())
}

fn read_comment_header_prefix(path: &Path) -> Result<String> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(read_comment_header_prefix_from_raw(&raw))
}

fn read_comment_header_prefix_from_raw(raw: &str) -> String {
    let mut out = String::new();
    for line in raw.split_inclusive('\n') {
        let check = line.trim_end_matches('\n');
        let check = check.trim_start();
        if check.is_empty() || check.starts_with('#') {
            out.push_str(line);
            continue;
        }
        break;
    }
    out
}

fn write_atomic_patch_yaml<T: serde::Serialize>(
    path: &Path,
    default_header: &str,
    existing_header: Option<&str>,
    patch: &T,
) -> Result<()> {
    let header = existing_header.unwrap_or(default_header);
    let mut body = serde_yaml::to_string(patch)
        .with_context(|| format!("failed to serialize {}", path.display()))?;
    if let Some(rest) = body.strip_prefix("---\n") {
        body = rest.to_string();
    }

    let mut out = String::new();
    out.push_str(header);
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&body);
    if !out.ends_with('\n') {
        out.push('\n');
    }

    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("path {} has no parent", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    let mut tmp = NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temp file near {}", path.display()))?;
    tmp.write_all(out.as_bytes())
        .with_context(|| format!("failed to write {}", path.display()))?;
    tmp.flush()?;
    tmp.persist(path)
        .map_err(|err| anyhow!("failed to persist {}: {}", path.display(), err.error))?;
    Ok(())
}

fn rollback_policy_file(patch_path: &Path, old_patch_raw: Option<&str>) {
    match old_patch_raw {
        Some(raw) => {
            let _ = fs::write(patch_path, raw);
        }
        None => {
            let _ = fs::remove_file(patch_path);
        }
    }
}
