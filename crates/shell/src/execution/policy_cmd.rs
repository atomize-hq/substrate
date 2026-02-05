use crate::execution::cli::{
    Cli, ConfigResetArgs, PolicyAction, PolicyCmd, PolicyCurrentAction, PolicyGlobalAction,
    PolicyGlobalCmd, PolicyInitArgs, PolicySetArgs, PolicyShowArgs, PolicyWorkspaceAction,
    PolicyWorkspaceCmd,
};
use crate::execution::config_model;
use crate::execution::config_model::ConfigUpdate;
use crate::execution::policy_model::PolicyPatch;
use crate::execution::{policy_model, workspace};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use substrate_broker::{Policy, PolicyExplainV1, WorldFsIsolation};
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

const POLICY_WORLD_FS_FULL_STASH_FILENAME: &str = "policy.full.stash.yaml";

const WORLD_FS_FULL_STASH_HEADER: &str = r#"# Substrate policy stash (world_fs full-only fields).
# This file is written by `substrate policy ...` when switching to world_fs.isolation=workspace.
# It preserves full-isolation-only settings (read/write/discover/enforcement) so they can be
# restored when switching back to world_fs.isolation=full.
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
    let mut notes: Vec<String> = Vec::new();

    let updates = config_model::parse_updates(&args.updates)?;
    let mut changed = policy_model::apply_updates_to_policy_patch(&mut patch, &updates)?;
    let stash_path = policy_patch_stash_path(&path);
    let old_stash_raw = fs::read_to_string(&stash_path).ok();
    changed |=
        maybe_apply_world_fs_isolation_transition(&mut patch, &updates, &stash_path, &mut notes)?;

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
            rollback_policy_files(
                &path,
                old_raw.as_deref(),
                &stash_path,
                old_stash_raw.as_deref(),
            );
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
            rollback_policy_files(
                &path,
                old_raw.as_deref(),
                &stash_path,
                old_stash_raw.as_deref(),
            );
            crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
            return Err(config_model::user_error(err.to_string()));
        }
    };
    for note in notes {
        eprintln!("{note}");
    }
    print_policy(&effective, args.json)?;
    Ok(())
}

fn run_global_reset(args: &ConfigResetArgs) -> Result<()> {
    let path = policy_model::global_policy_path()?;
    let old_raw = fs::read_to_string(&path).ok();
    let (mut patch, existed) = policy_model::read_global_policy_patch_or_empty()
        .with_context(|| format!("failed to load global policy patch at {}", path.display()))?;
    let changed = policy_model::reset_policy_patch_keys(&mut patch, &args.keys)?;
    let stash_path = policy_patch_stash_path(&path);
    let old_stash_raw = fs::read_to_string(&stash_path).ok();

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
            rollback_policy_files(
                &path,
                old_raw.as_deref(),
                &stash_path,
                old_stash_raw.as_deref(),
            );
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
            rollback_policy_files(
                &path,
                old_raw.as_deref(),
                &stash_path,
                old_stash_raw.as_deref(),
            );
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
    let mut notes: Vec<String> = Vec::new();

    let updates = config_model::parse_updates(&args.updates)?;
    let mut changed = policy_model::apply_updates_to_policy_patch(&mut patch, &updates)?;
    let stash_path = policy_patch_stash_path(&path);
    let old_stash_raw = fs::read_to_string(&stash_path).ok();
    changed |=
        maybe_apply_world_fs_isolation_transition(&mut patch, &updates, &stash_path, &mut notes)?;
    if changed {
        if let Err(err) = write_atomic_patch_yaml(&path, "", Some(&header), &patch)
            .with_context(|| format!("failed to write {}", path.display()))
        {
            rollback_policy_files(&path, Some(&old_raw), &stash_path, old_stash_raw.as_deref());
            crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
            return Err(err);
        }
        crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
    }

    let (effective, _) = match substrate_broker::resolve_effective_policy_with_explain(&cwd, false)
    {
        Ok(v) => v,
        Err(err) => {
            rollback_policy_files(&path, Some(&old_raw), &stash_path, old_stash_raw.as_deref());
            crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
            return Err(config_model::user_error(err.to_string()));
        }
    };
    for note in notes {
        eprintln!("{note}");
    }
    print_policy(&effective, args.json)?;
    Ok(())
}

fn run_workspace_reset(args: &ConfigResetArgs) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace_root = require_workspace(&cwd)?;
    let path = policy_model::workspace_policy_path(&workspace_root);
    let stash_path = policy_patch_stash_path(&path);
    let old_stash_raw = fs::read_to_string(&stash_path).ok();

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
            rollback_policy_files(&path, Some(&old_raw), &stash_path, old_stash_raw.as_deref());
            crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
            return Err(err);
        }
        crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
    }

    let (effective, _) = match substrate_broker::resolve_effective_policy_with_explain(&cwd, false)
    {
        Ok(v) => v,
        Err(err) => {
            rollback_policy_files(&path, Some(&old_raw), &stash_path, old_stash_raw.as_deref());
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

fn print_policy(policy: &Policy, json: bool) -> Result<()> {
    if json {
        let mut value = serde_json::to_value(policy).context("failed to serialize JSON")?;
        if let Some(require_world) = value
            .get("world_fs")
            .and_then(|fs| fs.get("require_world"))
            .and_then(|v| v.as_bool())
        {
            value["world_fs_require_world"] = serde_json::Value::Bool(require_world);
        }
        println!(
            "{}",
            serde_json::to_string(&value).context("failed to serialize JSON")?
        );
        return Ok(());
    }
    println!(
        "{}",
        serde_yaml::to_string(policy).context("failed to serialize YAML")?
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

fn policy_patch_stash_path(patch_path: &Path) -> PathBuf {
    patch_path.with_file_name(POLICY_WORLD_FS_FULL_STASH_FILENAME)
}

fn isolation_update(updates: &[ConfigUpdate]) -> Option<WorldFsIsolation> {
    updates
        .iter()
        .find(|u| u.key == "world_fs.isolation")
        .and_then(|u| WorldFsIsolation::parse(&u.value))
}

fn maybe_apply_world_fs_isolation_transition(
    patch: &mut PolicyPatch,
    updates: &[ConfigUpdate],
    stash_path: &Path,
    notes: &mut Vec<String>,
) -> Result<bool> {
    let Some(isolation) = isolation_update(updates) else {
        return Ok(false);
    };

    match isolation {
        WorldFsIsolation::Workspace => {
            let stash = extract_full_isolation_world_fs_stash(patch);
            if full_isolation_world_fs_stash_is_empty(&stash) {
                return Ok(false);
            }

            write_atomic_patch_yaml(stash_path, WORLD_FS_FULL_STASH_HEADER, None, &stash)
                .with_context(|| format!("failed to write {}", stash_path.display()))?;

            clear_full_isolation_world_fs_fields(patch);
            notes.push(format!(
                "substrate: note: stashed world_fs full-isolation settings to {} (workspace isolation forbids read/write/discover/enforcement keys)",
                stash_path.display()
            ));
            Ok(true)
        }
        WorldFsIsolation::Full => {
            let mut changed = false;
            if stash_path.exists() {
                let raw = fs::read_to_string(stash_path)
                    .with_context(|| format!("failed to read {}", stash_path.display()))?;
                let stash = policy_model::parse_policy_patch_yaml(stash_path, &raw)?;
                if merge_full_isolation_world_fs_stash_into_patch(patch, &stash) {
                    notes.push(format!(
                        "substrate: note: restored world_fs full-isolation settings from {}",
                        stash_path.display()
                    ));
                    changed = true;
                }
            }

            notes.push("substrate: note: world_fs.isolation=full defaults read/write allow_list to '.' (entire project); set world_fs.read.allow_list and world_fs.write.allow_list to restrict access".to_string());
            Ok(changed)
        }
    }
}

fn extract_full_isolation_world_fs_stash(patch: &PolicyPatch) -> PolicyPatch {
    let mut stash = PolicyPatch::default();
    stash.world_fs.enforcement = patch.world_fs.enforcement;
    stash.world_fs.discover = patch.world_fs.discover.clone();
    stash.world_fs.read = patch.world_fs.read.clone();
    stash.world_fs.write = patch.world_fs.write.clone();
    stash
}

fn full_isolation_world_fs_stash_is_empty(stash: &PolicyPatch) -> bool {
    stash.world_fs.enforcement.is_none()
        && dimension_patch_is_empty(&stash.world_fs.discover)
        && dimension_patch_is_empty(&stash.world_fs.read)
        && dimension_patch_is_empty(&stash.world_fs.write)
}

fn dimension_patch_is_empty(patch: &policy_model::WorldFsDimensionPatch) -> bool {
    patch.allow_list.is_none() && patch.deny_list.is_none()
}

fn clear_full_isolation_world_fs_fields(patch: &mut PolicyPatch) {
    patch.world_fs.enforcement = None;
    patch.world_fs.discover = policy_model::WorldFsDimensionPatch::default();
    patch.world_fs.read = policy_model::WorldFsDimensionPatch::default();
    patch.world_fs.write = policy_model::WorldFsDimensionPatch::default();
}

fn merge_full_isolation_world_fs_stash_into_patch(
    patch: &mut PolicyPatch,
    stash: &PolicyPatch,
) -> bool {
    let mut changed = false;

    if patch.world_fs.enforcement.is_none() && stash.world_fs.enforcement.is_some() {
        patch.world_fs.enforcement = stash.world_fs.enforcement;
        changed = true;
    }

    changed |= merge_dimension_patch(&mut patch.world_fs.discover, &stash.world_fs.discover);
    changed |= merge_dimension_patch(&mut patch.world_fs.read, &stash.world_fs.read);
    changed |= merge_dimension_patch(&mut patch.world_fs.write, &stash.world_fs.write);

    changed
}

fn merge_dimension_patch(
    target: &mut policy_model::WorldFsDimensionPatch,
    stash: &policy_model::WorldFsDimensionPatch,
) -> bool {
    let mut changed = false;
    if target.allow_list.is_none() && stash.allow_list.is_some() {
        target.allow_list = stash.allow_list.clone();
        changed = true;
    }
    if target.deny_list.is_none() && stash.deny_list.is_some() {
        target.deny_list = stash.deny_list.clone();
        changed = true;
    }
    changed
}

fn rollback_policy_files(
    patch_path: &Path,
    old_patch_raw: Option<&str>,
    stash_path: &Path,
    old_stash_raw: Option<&str>,
) {
    match old_patch_raw {
        Some(raw) => {
            let _ = fs::write(patch_path, raw);
        }
        None => {
            let _ = fs::remove_file(patch_path);
        }
    }

    match old_stash_raw {
        Some(raw) => {
            let _ = fs::write(stash_path, raw);
        }
        None => {
            let _ = fs::remove_file(stash_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::config_model::UpdateOp;
    use substrate_broker::WorldFsEnforcement;
    use tempfile::tempdir;

    #[test]
    fn world_fs_isolation_workspace_stashes_and_clears_full_only_fields() {
        let dir = tempdir().expect("tempdir");
        let patch_path = dir.path().join("policy.yaml");
        let stash_path = policy_patch_stash_path(&patch_path);

        let mut patch = PolicyPatch::default();
        patch.world_fs.enforcement = Some(WorldFsEnforcement::Strict);
        patch.world_fs.read.allow_list = Some(vec![".".to_string()]);
        patch.world_fs.write.allow_list = Some(vec!["dist".to_string()]);

        let updates = vec![ConfigUpdate {
            key: "world_fs.isolation".to_string(),
            op: UpdateOp::Set,
            value: "workspace".to_string(),
        }];
        let mut notes = Vec::new();
        let changed = maybe_apply_world_fs_isolation_transition(
            &mut patch,
            &updates,
            &stash_path,
            &mut notes,
        )
        .expect("apply isolation transition");
        assert!(changed, "expected stash+clear to report changes");
        assert!(stash_path.exists(), "expected stash file to be written");

        assert!(patch.world_fs.enforcement.is_none());
        assert!(patch.world_fs.read.allow_list.is_none());
        assert!(patch.world_fs.write.allow_list.is_none());

        let stash_raw = fs::read_to_string(&stash_path).expect("read stash");
        let stash = policy_model::parse_policy_patch_yaml(&stash_path, &stash_raw).expect("parse");
        assert_eq!(stash.world_fs.enforcement, Some(WorldFsEnforcement::Strict));
        assert_eq!(
            stash.world_fs.read.allow_list.clone().unwrap(),
            vec![".".to_string()]
        );
        assert_eq!(
            stash.world_fs.write.allow_list.clone().unwrap(),
            vec!["dist".to_string()]
        );
        assert!(
            notes.iter().any(|n| n.contains("stashed world_fs")),
            "expected a stash note"
        );
    }

    #[test]
    fn world_fs_isolation_full_restores_from_stash_when_missing() {
        let dir = tempdir().expect("tempdir");
        let patch_path = dir.path().join("policy.yaml");
        let stash_path = policy_patch_stash_path(&patch_path);

        let mut stash = PolicyPatch::default();
        stash.world_fs.enforcement = Some(WorldFsEnforcement::Strict);
        stash.world_fs.read.allow_list = Some(vec![".".to_string()]);
        stash.world_fs.write.allow_list = Some(vec!["dist".to_string()]);
        write_atomic_patch_yaml(&stash_path, WORLD_FS_FULL_STASH_HEADER, None, &stash)
            .expect("write stash");

        let mut patch = PolicyPatch::default();
        let updates = vec![ConfigUpdate {
            key: "world_fs.isolation".to_string(),
            op: UpdateOp::Set,
            value: "full".to_string(),
        }];
        let mut notes = Vec::new();
        let changed = maybe_apply_world_fs_isolation_transition(
            &mut patch,
            &updates,
            &stash_path,
            &mut notes,
        )
        .expect("apply isolation transition");
        assert!(changed, "expected restore to report changes");

        assert_eq!(patch.world_fs.enforcement, Some(WorldFsEnforcement::Strict));
        assert_eq!(
            patch.world_fs.read.allow_list.clone().unwrap(),
            vec![".".to_string()]
        );
        assert_eq!(
            patch.world_fs.write.allow_list.clone().unwrap(),
            vec!["dist".to_string()]
        );
        assert!(
            notes.iter().any(|n| n.contains("restored world_fs"))
                && notes
                    .iter()
                    .any(|n| n.contains("defaults read/write allow_list")),
            "expected restore + full-default note"
        );
    }
}
