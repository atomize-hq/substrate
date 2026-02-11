use crate::execution::cli::{
    Cli, SyncConflictPolicyArg, SyncDirectionArg, WorkspaceAction, WorkspaceCheckpointArgs,
    WorkspaceCmd, WorkspaceInitArgs, WorkspacePathArgs, WorkspaceRollbackArgs, WorkspaceSyncArgs,
};
use crate::execution::config_model::{CliConfigOverrides, SyncConflictPolicy, SyncDirection};
use crate::execution::settings::{apply_world_root_env, resolve_world_root};
use crate::execution::workspace;
use anyhow::{anyhow, Context, Result};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use substrate_common::WorldRootMode;
use tempfile::NamedTempFile;

const DEFAULT_WORKSPACE_PATCH_YAML: &str = r#"# Substrate config patch (sparse overrides; scope=workspace).
# - This file is a YAML mapping of workspace-scoped config overrides.
# - Omitted keys inherit from global config + defaults.
# - View the effective merged config with: `substrate config current show --explain`
{}
"#;

const DEFAULT_POLICY_PATCH_YAML: &str = r#"# Substrate policy patch (sparse overrides; scope=workspace).
# - This file is a YAML mapping of workspace-scoped policy overrides.
# - Omitted keys inherit from global policy + defaults.
# - View the effective merged policy with: `substrate policy current show --explain`
{}
"#;

pub(crate) fn handle_workspace_command(cmd: &WorkspaceCmd, cli: &Cli) -> i32 {
    let result: Result<i32> = match &cmd.action {
        WorkspaceAction::Init(args) => run_workspace_init(args).map(|_| 0),
        WorkspaceAction::Disable(args) => run_workspace_disable(args).map(|_| 0),
        WorkspaceAction::Enable(args) => run_workspace_enable(args).map(|_| 0),
        WorkspaceAction::Sync(args) => run_workspace_sync(args, cli),
        WorkspaceAction::Checkpoint(args) => run_workspace_checkpoint(args),
        WorkspaceAction::Rollback(args) => run_workspace_rollback(args),
    };

    match result {
        Ok(code) => code,
        Err(err) if err.is::<ActionableError>() => {
            eprintln!("{:#}", err);
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

fn run_workspace_init(args: &WorkspaceInitArgs) -> Result<()> {
    let target = args
        .path
        .clone()
        .unwrap_or_else(|| PathBuf::from("."))
        .canonicalize()
        .with_context(|| "invalid PATH for workspace init")
        .map_err(|err| actionable(err.to_string()))?;

    if !target.is_dir() {
        return Err(actionable(format!(
            "substrate: invalid PATH for workspace init: {} (not a directory)",
            target.display()
        )));
    }

    ensure_not_nested_workspace(&target)?;

    fs::create_dir_all(target.join(workspace::SUBSTRATE_DIR_NAME))
        .with_context(|| format!("failed to create {}", target.display()))?;
    fs::create_dir_all(workspace::internal_git_dir(&target)).with_context(|| {
        format!(
            "failed to create internal git dir under {}",
            target.display()
        )
    })?;

    let mut wrote_any = false;

    let workspace_yaml = workspace::workspace_marker_path(&target);
    if !workspace_yaml.exists() {
        write_atomic_bytes(&workspace_yaml, DEFAULT_WORKSPACE_PATCH_YAML.as_bytes())
            .with_context(|| format!("failed to write {}", workspace_yaml.display()))?;
        wrote_any = true;
    }

    let policy_yaml = workspace::workspace_policy_path(&target);
    if !policy_yaml.exists() {
        write_atomic_bytes(&policy_yaml, DEFAULT_POLICY_PATCH_YAML.as_bytes())
            .with_context(|| format!("failed to write {}", policy_yaml.display()))?;
        wrote_any = true;
    }

    if args.examples {
        ensure_example_files(&target)?;
    }

    ensure_gitignore_rules(&target).context("failed to update .gitignore")?;

    if args.force {
        println!(
            "substrate: workspace initialized at {} (--force; repaired missing entries only)",
            target.display()
        );
    } else {
        println!("substrate: workspace initialized at {}", target.display());
    }

    if wrote_any {
        crate::execution::config_model::invalidate_config_cache();
        crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
    }

    Ok(())
}

fn run_workspace_disable(args: &WorkspacePathArgs) -> Result<()> {
    let start = resolve_search_root(args, "workspace disable")?;
    let workspace_root = require_workspace_root_any(&start, "workspace disable")?;
    let marker = workspace::workspace_disabled_marker_path(&workspace_root);

    if !marker.exists() {
        write_atomic_bytes(&marker, b"")
            .with_context(|| format!("failed to write {}", marker.display()))?;
        crate::execution::config_model::invalidate_config_cache();
        crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();
    }

    println!(
        "substrate: workspace disabled at {}",
        workspace_root.display()
    );
    Ok(())
}

fn run_workspace_enable(args: &WorkspacePathArgs) -> Result<()> {
    let start = resolve_search_root(args, "workspace enable")?;
    let workspace_root = require_workspace_root_any(&start, "workspace enable")?;
    let marker = workspace::workspace_disabled_marker_path(&workspace_root);

    match fs::remove_file(&marker) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(anyhow!("failed to remove {}: {err}", marker.display())),
    }
    crate::execution::config_model::invalidate_config_cache();
    crate::execution::policy_snapshot::invalidate_policy_snapshot_cache();

    println!(
        "substrate: workspace enabled at {}",
        workspace_root.display()
    );
    Ok(())
}

fn resolve_search_root(args: &WorkspacePathArgs, verb: &str) -> Result<PathBuf> {
    let target = args
        .path
        .clone()
        .unwrap_or_else(|| PathBuf::from("."))
        .canonicalize()
        .with_context(|| format!("invalid PATH for {verb}"))
        .map_err(|err| actionable(err.to_string()))?;

    if target.is_dir() {
        return Ok(target);
    }
    if target.is_file() {
        if let Some(parent) = target.parent() {
            return Ok(parent.to_path_buf());
        }
    }

    Err(actionable(format!(
        "substrate: invalid PATH for {verb}: {}",
        target.display()
    )))
}

fn require_workspace_root_any(start: &Path, verb: &str) -> Result<PathBuf> {
    workspace::find_workspace_root_any(start).ok_or_else(|| {
        actionable(format!(
            "substrate: not in a workspace for {verb}; run `substrate workspace init`"
        ))
    })
}

fn require_workspace_root(start: &Path, verb: &str) -> Result<PathBuf> {
    workspace::find_workspace_root(start).ok_or_else(|| {
        actionable(format!(
            "substrate: not in a workspace for {verb}; run `substrate workspace init`"
        ))
    })
}

fn ensure_not_nested_workspace(target: &Path) -> Result<()> {
    let mut ancestors = target.ancestors();
    let _self_dir = ancestors.next();
    for parent in ancestors {
        let marker = workspace::workspace_marker_path(parent);
        if marker.is_file() {
            return Err(actionable(format!(
                "substrate: refusing to create a nested workspace at {}\nFound parent workspace marker at {}\n",
                target.display(),
                marker.display()
            )));
        }
    }
    Ok(())
}

fn sync_direction_as_str(direction: SyncDirection) -> &'static str {
    match direction {
        SyncDirection::FromWorld => "from_world",
        SyncDirection::FromHost => "from_host",
        SyncDirection::Both => "both",
    }
}

fn sync_conflict_policy_as_str(policy: SyncConflictPolicy) -> &'static str {
    match policy {
        SyncConflictPolicy::PreferHost => "prefer_host",
        SyncConflictPolicy::PreferWorld => "prefer_world",
        SyncConflictPolicy::Abort => "abort",
    }
}

fn parse_sync_direction(arg: SyncDirectionArg) -> SyncDirection {
    match arg {
        SyncDirectionArg::FromWorld => SyncDirection::FromWorld,
        SyncDirectionArg::FromHost => SyncDirection::FromHost,
        SyncDirectionArg::Both => SyncDirection::Both,
    }
}

fn parse_sync_conflict_policy(arg: SyncConflictPolicyArg) -> SyncConflictPolicy {
    match arg {
        SyncConflictPolicyArg::PreferHost => SyncConflictPolicy::PreferHost,
        SyncConflictPolicyArg::PreferWorld => SyncConflictPolicy::PreferWorld,
        SyncConflictPolicyArg::Abort => SyncConflictPolicy::Abort,
    }
}

fn build_effective_sync_excludes(
    effective_excludes: &[String],
    cli_excludes: &[String],
) -> Result<Vec<String>> {
    let mut excludes: Vec<String> = Vec::new();
    for item in crate::execution::config_model::PROTECTED_EXCLUDES {
        excludes.push(item.to_string());
    }

    for item in effective_excludes {
        if crate::execution::config_model::PROTECTED_EXCLUDES.contains(&item.as_str()) {
            continue;
        }
        excludes.push(item.clone());
    }

    for item in cli_excludes {
        if crate::execution::config_model::PROTECTED_EXCLUDES.contains(&item.as_str()) {
            continue;
        }
        excludes.push(item.clone());
    }

    // Stable, order-preserving de-dupe.
    let mut seen: HashSet<String> = HashSet::new();
    excludes.retain(|item| seen.insert(item.clone()));

    for item in &excludes {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            return Err(actionable("substrate: invalid --exclude pattern: empty"));
        }
        if trimmed.starts_with('/') {
            return Err(actionable(format!(
                "substrate: invalid --exclude pattern (must not start with '/'): {trimmed}"
            )));
        }
    }

    Ok(excludes)
}

fn normalize_workspace_rel_path(raw: &str) -> std::result::Result<String, String> {
    let mut path = raw.trim().replace('\\', "/");
    while let Some(stripped) = path.strip_prefix("./") {
        path = stripped.to_string();
    }
    if path.is_empty() {
        return Err("empty path".to_string());
    }
    if path.starts_with('/') {
        return Err(format!("absolute paths are not allowed: {path}"));
    }
    if path.len() >= 2 && path.as_bytes()[1] == b':' {
        return Err(format!("absolute paths are not allowed: {path}"));
    }
    if path.split('/').any(|segment| segment == "..") {
        return Err(format!("path segments must not be '..': {path}"));
    }
    Ok(path)
}

fn glob_matches(pattern: &str, text: &str) -> bool {
    #[derive(Clone, Copy)]
    enum Token {
        Lit(u8),
        AnyCharNoSlash,
        StarNoSlash,
        StarAny,
    }

    let p = pattern.as_bytes();
    let t = text.as_bytes();

    let mut tokens: Vec<Token> = Vec::new();
    let mut i = 0usize;
    while i < p.len() {
        match p[i] {
            b'?' => {
                tokens.push(Token::AnyCharNoSlash);
                i += 1;
            }
            b'*' => {
                if i + 1 < p.len() && p[i + 1] == b'*' {
                    tokens.push(Token::StarAny);
                    i += 2;
                } else {
                    tokens.push(Token::StarNoSlash);
                    i += 1;
                }
            }
            b => {
                tokens.push(Token::Lit(b));
                i += 1;
            }
        }
    }

    fn match_at(
        tokens: &[Token],
        text: &[u8],
        pi: usize,
        ti: usize,
        memo: &mut std::collections::HashMap<(usize, usize), bool>,
    ) -> bool {
        if let Some(hit) = memo.get(&(pi, ti)) {
            return *hit;
        }

        let out = if pi == tokens.len() {
            ti == text.len()
        } else {
            match tokens[pi] {
                Token::Lit(b) => {
                    ti < text.len() && text[ti] == b && match_at(tokens, text, pi + 1, ti + 1, memo)
                }
                Token::AnyCharNoSlash => {
                    ti < text.len()
                        && text[ti] != b'/'
                        && match_at(tokens, text, pi + 1, ti + 1, memo)
                }
                Token::StarNoSlash => {
                    let mut end = ti;
                    while end < text.len() && text[end] != b'/' {
                        end += 1;
                    }
                    (ti..=end).any(|k| match_at(tokens, text, pi + 1, k, memo))
                }
                Token::StarAny => {
                    (ti..=text.len()).any(|k| match_at(tokens, text, pi + 1, k, memo))
                }
            }
        };

        memo.insert((pi, ti), out);
        out
    }

    let mut memo: std::collections::HashMap<(usize, usize), bool> =
        std::collections::HashMap::new();
    match_at(&tokens, t, 0, 0, &mut memo)
}

fn glob_matches_path(pattern: &str, path: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix("/**") {
        if path == prefix {
            return true;
        }
    }
    glob_matches(pattern, path)
}

fn run_workspace_sync(args: &WorkspaceSyncArgs, cli: &Cli) -> Result<i32> {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let _workspace_root = require_workspace_root(&cwd, "workspace sync")?;

    let cli_world_enabled = if cli.world {
        Some(true)
    } else if cli.no_world {
        Some(false)
    } else {
        None
    };
    let cli_caged = if cli.caged {
        Some(true)
    } else if cli.uncaged {
        Some(false)
    } else {
        None
    };

    let effective = crate::execution::config_model::resolve_effective_config(
        &cwd,
        &CliConfigOverrides {
            world_enabled: cli_world_enabled,
            anchor_mode: cli.anchor_mode.map(WorldRootMode::from),
            anchor_path: cli
                .anchor_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            caged: cli_caged,
        },
    )?;

    let direction = args
        .direction
        .map(parse_sync_direction)
        .unwrap_or(effective.sync.direction);
    let conflict_policy = args
        .conflict_policy
        .map(parse_sync_conflict_policy)
        .unwrap_or(effective.sync.conflict_policy);

    if !args.dry_run {
        match direction {
            SyncDirection::FromWorld => {
                eprintln!("substrate: workspace sync is not implemented until WS2");
            }
            SyncDirection::FromHost | SyncDirection::Both => {
                eprintln!("substrate: workspace sync is not implemented until WS5");
            }
        }
        return Ok(4);
    }

    let excludes = build_effective_sync_excludes(&effective.sync.exclude, &args.exclude)?;

    println!("substrate: workspace sync --dry-run preview (WS1)");
    println!("  auto_sync: {}", effective.sync.auto_sync);
    println!("  direction: {}", sync_direction_as_str(direction));
    println!(
        "  conflict_policy: {}",
        sync_conflict_policy_as_str(conflict_policy)
    );
    println!("  exclude:");
    for item in &excludes {
        println!("    - {item}");
    }

    match direction {
        SyncDirection::FromHost | SyncDirection::Both => {
            eprintln!(
                "substrate: workspace sync direction {} is not implemented until WS5",
                sync_direction_as_str(direction)
            );
            return Ok(4);
        }
        SyncDirection::FromWorld => {}
    }

    if cli.no_world {
        eprintln!("workspace sync requires world; remove --no-world");
        return Ok(2);
    }
    if !effective.world.enabled {
        eprintln!(
            "substrate: workspace sync requires world; run `substrate world enable` then `substrate world doctor`"
        );
        return Ok(3);
    }

    let world_root_settings = resolve_world_root(
        cli.anchor_mode.map(WorldRootMode::from),
        cli.anchor_path.clone(),
        cli_caged,
        &cwd,
    )?;
    apply_world_root_env(&world_root_settings);

    let (client, request, _agent_id) =
        match crate::execution::routing::build_agent_client_and_pending_diff_request() {
            Ok(ok) => ok,
            Err(_err) => {
                eprintln!(
                    "substrate: workspace sync requires world; run `substrate world enable` then `substrate world doctor`"
                );
                return Ok(3);
            }
        };

    let rt = tokio::runtime::Runtime::new().context("failed to initialize tokio runtime")?;
    let caps = match rt.block_on(async { client.capabilities().await }) {
        Ok(c) => c,
        Err(_err) => {
            eprintln!(
                "substrate: workspace sync requires world; run `substrate world enable` then `substrate world doctor`"
            );
            return Ok(3);
        }
    };

    let has_pending_diff = caps
        .get("features")
        .and_then(|f| f.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .any(|s| s == "pending_diff_v1")
        })
        .unwrap_or(false);

    if !has_pending_diff {
        eprintln!(
            "substrate: workspace sync pending diff discovery is unsupported by this backend"
        );
        return Ok(4);
    }

    let record = match rt.block_on(async { client.pending_diff(request).await }) {
        Ok(r) => r,
        Err(_err) => {
            eprintln!("substrate: workspace sync failed to retrieve pending diff");
            return Ok(1);
        }
    };

    let mut offending_protected: Vec<String> = Vec::new();
    let mut raw_paths: Vec<(&'static str, String)> = Vec::new();
    for (bucket, items) in [
        ("writes", &record.non_pty.writes),
        ("mods", &record.non_pty.mods),
        ("deletes", &record.non_pty.deletes),
    ] {
        for raw in items {
            let normalized = match normalize_workspace_rel_path(raw) {
                Ok(p) => p,
                Err(msg) => {
                    eprintln!("substrate: workspace sync refused: invalid diff path: {msg}");
                    return Ok(5);
                }
            };

            for prot in crate::execution::config_model::PROTECTED_EXCLUDES {
                if glob_matches_path(prot, &normalized) {
                    offending_protected.push(normalized.clone());
                    break;
                }
            }
            raw_paths.push((bucket, normalized));
        }
    }

    offending_protected.sort();
    offending_protected.dedup();
    if !offending_protected.is_empty() {
        eprintln!("substrate: workspace sync refused: pending diff contains protected paths");
        for item in offending_protected {
            eprintln!("  - {item}");
        }
        return Ok(5);
    }

    let excludes_non_protected: Vec<&str> = excludes
        .iter()
        .filter(|p| !crate::execution::config_model::PROTECTED_EXCLUDES.contains(&p.as_str()))
        .map(|p| p.as_str())
        .collect();

    let mut out_writes: Vec<String> = Vec::new();
    let mut out_mods: Vec<String> = Vec::new();
    let mut out_deletes: Vec<String> = Vec::new();
    let mut excluded_count: usize = 0;

    for (bucket, path) in raw_paths {
        let excluded = excludes_non_protected
            .iter()
            .any(|pat| glob_matches_path(pat, &path));
        if excluded {
            excluded_count += 1;
            continue;
        }
        match bucket {
            "writes" => out_writes.push(path),
            "mods" => out_mods.push(path),
            "deletes" => out_deletes.push(path),
            _ => {}
        }
    }

    out_writes.sort();
    out_mods.sort();
    out_deletes.sort();
    out_writes.dedup();
    out_mods.dedup();
    out_deletes.dedup();

    let total_paths = out_writes.len() + out_mods.len() + out_deletes.len();

    println!("substrate: pending diff summary (non_pty)");
    if args.verbose {
        println!("  session_started_at: {}", record.session_started_at);
        println!("  diff_id: {}", record.diff_id);
    }
    println!("  total_paths: {total_paths}");
    println!("  writes: {}", out_writes.len());
    println!("  mods: {}", out_mods.len());
    println!("  deletes: {}", out_deletes.len());
    if excluded_count > 0 {
        println!("  excluded_by_patterns: true ({excluded_count})");
    } else {
        println!("  excluded_by_patterns: false");
    }

    Ok(0)
}

fn run_workspace_checkpoint(_args: &WorkspaceCheckpointArgs) -> Result<i32> {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let _workspace_root = require_workspace_root(&cwd, "workspace checkpoint")?;
    eprintln!("substrate: workspace checkpoint is not implemented until WS6");
    Ok(4)
}

fn run_workspace_rollback(_args: &WorkspaceRollbackArgs) -> Result<i32> {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let _workspace_root = require_workspace_root(&cwd, "workspace rollback")?;
    eprintln!("substrate: workspace rollback is not implemented until WS7");
    Ok(4)
}

fn ensure_gitignore_rules(root: &Path) -> Result<()> {
    let gitignore = root.join(".gitignore");
    let mut existing = match fs::read_to_string(&gitignore) {
        Ok(raw) => raw.lines().map(|l| l.to_string()).collect::<Vec<_>>(),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Vec::new(),
        Err(err) => return Err(anyhow!("failed to read {}: {err}", gitignore.display())),
    };

    let substrate_ignore = ".substrate/";
    let workspace_allow = "!.substrate/workspace.yaml";
    let policy_allow = "!.substrate/policy.yaml";

    if !existing
        .iter()
        .any(|line| line.trim_end() == substrate_ignore)
    {
        existing.push(substrate_ignore.to_string());
    }

    let last_substrate_ignore_idx = existing
        .iter()
        .rposition(|line| line.trim_end() == substrate_ignore)
        .expect("substrate_ignore must exist");

    let has_workspace_allow_after = existing[last_substrate_ignore_idx + 1..]
        .iter()
        .any(|line| line.trim_end() == workspace_allow);
    let has_policy_allow_after = existing[last_substrate_ignore_idx + 1..]
        .iter()
        .any(|line| line.trim_end() == policy_allow);

    if !has_workspace_allow_after {
        existing.push(workspace_allow.to_string());
    }
    if !has_policy_allow_after {
        existing.push(policy_allow.to_string());
    }

    let mut body = existing.join("\n");
    if !body.ends_with('\n') {
        body.push('\n');
    }
    write_atomic_bytes(&gitignore, body.as_bytes())
}

fn ensure_example_files(workspace_root: &Path) -> Result<()> {
    let substrate_dir = workspace_root.join(workspace::SUBSTRATE_DIR_NAME);
    let workspace_example = substrate_dir.join("workspace.example.yaml");
    let policy_example = substrate_dir.join("policy.example.yaml");

    if !workspace_example.exists() {
        write_atomic_bytes(&workspace_example, DEFAULT_WORKSPACE_PATCH_YAML.as_bytes())
            .with_context(|| {
                format!(
                    "failed to write workspace example file {}",
                    workspace_example.display()
                )
            })?;
    }

    if !policy_example.exists() {
        write_atomic_bytes(&policy_example, DEFAULT_POLICY_PATCH_YAML.as_bytes()).with_context(
            || {
                format!(
                    "failed to write policy example file {}",
                    policy_example.display()
                )
            },
        )?;
    }

    Ok(())
}

fn write_atomic_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("path {} has no parent", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    let mut tmp = NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temp file near {}", path.display()))?;
    tmp.write_all(bytes)
        .with_context(|| format!("failed to write {}", path.display()))?;
    tmp.flush()?;
    tmp.persist(path)
        .map_err(|err| anyhow!("failed to persist {}: {}", path.display(), err.error))?;
    Ok(())
}
