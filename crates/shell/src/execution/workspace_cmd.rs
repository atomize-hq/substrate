use crate::execution::cli::{
    Cli, SyncConflictPolicyArg, SyncDirectionArg, WorkspaceAction, WorkspaceCheckpointArgs,
    WorkspaceCmd, WorkspaceInitArgs, WorkspacePathArgs, WorkspaceRollbackArgs, WorkspaceSyncArgs,
};
use crate::execution::config_model::{CliConfigOverrides, SyncConflictPolicy, SyncDirection};
use crate::execution::settings::{apply_world_root_env, resolve_world_root};
use crate::execution::workspace;
use anyhow::{anyhow, Context, Result};
use base64::Engine;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
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
        Err(err) => {
            eprintln!("{:#}", err);
            workspace_cmd_exit_code_for_error(&err)
        }
    }
}

pub(crate) fn workspace_cmd_exit_code_for_error(err: &anyhow::Error) -> i32 {
    if err.is::<ActionableError>() {
        2
    } else {
        1
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

fn capabilities_has_feature(caps: &serde_json::Value, feature: &str) -> bool {
    caps.get("features")
        .and_then(|f| f.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).any(|s| s == feature))
        .unwrap_or(false)
}

#[cfg(unix)]
fn apply_execute_bits(path: &Path, source_mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let meta = fs::metadata(path)
        .with_context(|| format!("failed to read permissions for {}", path.display()))?;
    let mut perms = meta.permissions();
    let current_mode = perms.mode();
    let new_mode = (current_mode & !0o111) | (source_mode & 0o111);
    perms.set_mode(new_mode);
    fs::set_permissions(path, perms)
        .with_context(|| format!("failed to set permissions for {}", path.display()))?;
    Ok(())
}

pub(crate) fn run_workspace_sync_for_auto_sync(
    args: &WorkspaceSyncArgs,
    cli: &Cli,
) -> Result<(i32, Option<String>)> {
    let mut failure_reason: Option<String> = None;
    let exit_code = run_workspace_sync_impl(args, cli, &mut failure_reason)?;
    Ok((exit_code, failure_reason))
}

fn run_workspace_sync(args: &WorkspaceSyncArgs, cli: &Cli) -> Result<i32> {
    let mut ignored_reason: Option<String> = None;
    run_workspace_sync_impl(args, cli, &mut ignored_reason)
}

fn run_workspace_sync_impl(
    args: &WorkspaceSyncArgs,
    cli: &Cli,
    failure_reason: &mut Option<String>,
) -> Result<i32> {
    macro_rules! errln {
        ($($arg:tt)*) => {{
            let line = format!($($arg)*);
            if failure_reason.is_none() && !line.trim().is_empty() {
                *failure_reason = Some(line.clone());
            }
            eprintln!("{line}");
        }};
    }

    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace_root = require_workspace_root(&cwd, "workspace sync")?;

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

    let excludes = build_effective_sync_excludes(&effective.sync.exclude, &args.exclude)?;

    if args.dry_run {
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
    }

    if cli.no_world {
        errln!("workspace sync requires world; remove --no-world");
        return Ok(2);
    }
    if !effective.world.enabled {
        errln!(
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
                errln!(
                    "substrate: workspace sync requires world; run `substrate world enable` then `substrate world doctor`"
                );
                return Ok(3);
            }
        };

    let rt = tokio::runtime::Runtime::new().context("failed to initialize tokio runtime")?;
    let caps = match rt.block_on(async { client.capabilities().await }) {
        Ok(c) => c,
        Err(_err) => {
            errln!(
                "substrate: workspace sync requires world; run `substrate world enable` then `substrate world doctor`"
            );
            return Ok(3);
        }
    };

    let has_pending_diff = capabilities_has_feature(&caps, "pending_diff_v1");

    if !has_pending_diff {
        errln!("substrate: workspace sync pending diff discovery is unsupported by this backend");
        return Ok(4);
    }

    let excludes_non_protected: Vec<&str> = excludes
        .iter()
        .filter(|p| !crate::execution::config_model::PROTECTED_EXCLUDES.contains(&p.as_str()))
        .map(|p| p.as_str())
        .collect();

    let has_pending_diff_reconcile = capabilities_has_feature(&caps, "pending_diff_reconcile_v1");
    if matches!(direction, SyncDirection::FromHost | SyncDirection::Both)
        && !has_pending_diff_reconcile
    {
        errln!(
            "substrate: workspace sync direction {} is unsupported by this backend",
            sync_direction_as_str(direction)
        );
        errln!("  missing feature: pending_diff_reconcile_v1");
        return Ok(4);
    }

    macro_rules! fetch_record {
        () => {{
            match rt.block_on(async { client.pending_diff(request.clone()).await }) {
                Ok(r) => Ok(r),
                Err(err) => {
                    // WS1-spec: when a direction that consults the pending diff record is in effect,
                    // a reachable world backend is required. If pending diff retrieval fails after
                    // capabilities succeeded, prefer treating it as a backend availability issue
                    // (exit 3) unless we can confidently classify it as a non-transport/internal
                    // payload failure.
                    let looks_like_payload_failure =
                        err.chain().any(|cause| cause.is::<serde_json::Error>());
                    if looks_like_payload_failure {
                        errln!("substrate: workspace sync failed to retrieve pending diff");
                        Err(1)
                    } else {
                        errln!(
                            "substrate: workspace sync requires world; run `substrate world enable` then `substrate world doctor`"
                        );
                        Err(3)
                    }
                }
            }
        }};
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum PendingDiffOrigin {
        NonPty,
        Pty,
    }

    impl PendingDiffOrigin {
        fn as_str(self) -> &'static str {
            match self {
                Self::NonPty => "non_pty",
                Self::Pty => "pty",
            }
        }
    }

    #[derive(Clone, Debug)]
    struct RawPendingPath {
        origin: PendingDiffOrigin,
        bucket: &'static str,
        path: String,
    }

    #[derive(Clone, Debug, Default)]
    struct FilteredBucketPaths {
        writes: Vec<String>,
        mods: Vec<String>,
        deletes: Vec<String>,
        excluded_count: usize,
    }

    impl FilteredBucketPaths {
        fn total_paths(&self) -> usize {
            self.writes.len() + self.mods.len() + self.deletes.len()
        }
    }

    #[derive(Clone, Debug)]
    struct PendingDiffState {
        raw_for_decisions: Vec<RawPendingPath>,
        shadowed_paths: Vec<String>,
        non_pty: FilteredBucketPaths,
        pty: Option<FilteredBucketPaths>,
        combined: FilteredBucketPaths,
    }

    macro_rules! build_pending_diff_state {
        ($record:expr) => {{
            (|| -> std::result::Result<PendingDiffState, i32> {
                let record: &agent_api_types::PendingDiffRecordV1 = $record;
                let mut offending_protected: Vec<String> = Vec::new();
                let mut raw_for_decisions: Vec<RawPendingPath> = Vec::new();

                let mut non_pty = FilteredBucketPaths::default();
                let mut pty = record.pty.as_ref().map(|_| FilteredBucketPaths::default());
                let mut combined = FilteredBucketPaths::default();

                let push_filtered =
                    |out: &mut FilteredBucketPaths, bucket: &'static str, path: String| {
                        match bucket {
                            "writes" => out.writes.push(path),
                            "mods" => out.mods.push(path),
                            "deletes" => out.deletes.push(path),
                            _ => {}
                        }
                    };

                let mut ingest_bucket = |origin: PendingDiffOrigin,
                                         bucket: &'static str,
                                         raw: &str,
                                         origin_out: &mut FilteredBucketPaths|
                 -> std::result::Result<(), i32> {
                    let normalized = match normalize_workspace_rel_path(raw) {
                        Ok(p) => p,
                        Err(msg) => {
                            errln!("substrate: workspace sync refused: invalid diff path: {msg}");
                            return Err(5);
                        }
                    };

                    for prot in crate::execution::config_model::PROTECTED_EXCLUDES {
                        if glob_matches_path(prot, &normalized) {
                            offending_protected.push(normalized.clone());
                            break;
                        }
                    }

                    raw_for_decisions.push(RawPendingPath {
                        origin,
                        bucket,
                        path: normalized.clone(),
                    });

                    let excluded = excludes_non_protected
                        .iter()
                        .any(|pat| glob_matches_path(pat, &normalized));
                    if excluded {
                        origin_out.excluded_count += 1;
                        combined.excluded_count += 1;
                        return Ok(());
                    }

                    push_filtered(origin_out, bucket, normalized.clone());
                    push_filtered(&mut combined, bucket, normalized);
                    Ok(())
                };

                for (bucket, items) in [
                    ("writes", &record.non_pty.writes),
                    ("mods", &record.non_pty.mods),
                    ("deletes", &record.non_pty.deletes),
                ] {
                    for raw in items {
                        ingest_bucket(PendingDiffOrigin::NonPty, bucket, raw, &mut non_pty)?;
                    }
                }

                if let Some(ref pty_bucket) = record.pty {
                    if let Some(ref mut out) = pty {
                        for (bucket, items) in [
                            ("writes", &pty_bucket.writes),
                            ("mods", &pty_bucket.mods),
                            ("deletes", &pty_bucket.deletes),
                        ] {
                            for raw in items {
                                ingest_bucket(PendingDiffOrigin::Pty, bucket, raw, out)?;
                            }
                        }
                    }
                }

                offending_protected.sort();
                offending_protected.dedup();
                if !offending_protected.is_empty() {
                    errln!("substrate: workspace sync refused: pending diff contains protected paths");
                    for item in offending_protected {
                        errln!("  - {item}");
                    }
                    return Err(5);
                }

                for out in [&mut non_pty, &mut combined] {
                    out.writes.sort();
                    out.mods.sort();
                    out.deletes.sort();
                    out.writes.dedup();
                    out.mods.dedup();
                    out.deletes.dedup();
                }
                if let Some(ref mut out) = pty {
                    out.writes.sort();
                    out.mods.sort();
                    out.deletes.sort();
                    out.writes.dedup();
                    out.mods.dedup();
                    out.deletes.dedup();
                }

                let mut shadowed_paths: Vec<String> =
                    raw_for_decisions.iter().map(|item| item.path.clone()).collect();
                shadowed_paths.sort();
                shadowed_paths.dedup();

                Ok(PendingDiffState {
                    raw_for_decisions,
                    shadowed_paths,
                    non_pty,
                    pty,
                    combined,
                })
            })()
        }};
    }

    let mut record = match fetch_record!() {
        Ok(r) => r,
        Err(code) => return Ok(code),
    };
    let mut state = match build_pending_diff_state!(&record) {
        Ok(s) => s,
        Err(code) => return Ok(code),
    };

    let mut from_host_conflicts: Vec<String> = Vec::new();
    let baseline_for_conflicts: Option<chrono::DateTime<chrono::Utc>> =
        if matches!(direction, SyncDirection::FromHost | SyncDirection::Both) || !args.dry_run {
            match chrono::DateTime::parse_from_rfc3339(&record.session_started_at) {
                Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
                Err(err) => {
                    errln!(
                        "substrate: workspace sync failed: invalid session_started_at timestamp"
                    );
                    errln!("{err:#}");
                    return Ok(1);
                }
            }
        } else {
            None
        };

    if matches!(direction, SyncDirection::FromHost | SyncDirection::Both) {
        let baseline = baseline_for_conflicts.expect("baseline computed for from_host");
        from_host_conflicts = {
            let mut conflicts: Vec<String> = Vec::new();
            for path in &state.shadowed_paths {
                let host_path = workspace_root.join(path);
                let meta = match fs::symlink_metadata(&host_path) {
                    Ok(m) => m,
                    Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
                    Err(err) => {
                        errln!(
                            "substrate: workspace sync failed: failed to stat {}",
                            host_path.display()
                        );
                        errln!("{err:#}");
                        return Ok(1);
                    }
                };

                let modified = match meta.modified() {
                    Ok(m) => chrono::DateTime::<chrono::Utc>::from(m),
                    Err(err) => {
                        errln!(
                            "substrate: workspace sync failed: failed to read mtime for {}",
                            host_path.display()
                        );
                        errln!("{err:#}");
                        return Ok(1);
                    }
                };

                if modified > baseline {
                    conflicts.push(path.clone());
                }
            }
            conflicts.sort();
            conflicts.dedup();
            conflicts
        };

        if args.dry_run {
            println!("substrate: from_host reconciliation plan");
        } else {
            println!("substrate: from_host reconciliation");
        }
        println!("  total_shadowed_paths: {}", state.shadowed_paths.len());
        println!(
            "  conflicts_detected: {} ({})",
            !from_host_conflicts.is_empty(),
            from_host_conflicts.len()
        );
        println!(
            "  conflict_policy: {}",
            sync_conflict_policy_as_str(conflict_policy)
        );

        if args.verbose {
            println!("  decisions:");
            for path in &state.shadowed_paths {
                let is_conflict = from_host_conflicts.binary_search(path).is_ok();
                let decision = if !is_conflict {
                    "keep"
                } else {
                    match conflict_policy {
                        SyncConflictPolicy::PreferHost => "discard",
                        SyncConflictPolicy::PreferWorld => "keep",
                        SyncConflictPolicy::Abort => "conflict",
                    }
                };
                println!("    - {path} => {decision}");
            }
        }
    }

    if args.dry_run {
        if direction == SyncDirection::FromHost {
            return Ok(0);
        }

        let non_pty_total_paths = state.non_pty.total_paths();
        println!("substrate: pending diff summary (non_pty)");
        if args.verbose {
            println!("  session_started_at: {}", record.session_started_at);
            println!("  diff_id: {}", record.diff_id);
        }
        println!("  total_paths: {non_pty_total_paths}");
        println!("  writes: {}", state.non_pty.writes.len());
        println!("  mods: {}", state.non_pty.mods.len());
        println!("  deletes: {}", state.non_pty.deletes.len());
        if state.non_pty.excluded_count > 0 {
            println!(
                "  excluded_by_patterns: true ({})",
                state.non_pty.excluded_count
            );
        } else {
            println!("  excluded_by_patterns: false");
        }

        if let Some(ref pty) = state.pty {
            let pty_total_paths = pty.total_paths();
            println!("substrate: pending diff summary (pty)");
            println!("  total_paths: {pty_total_paths}");
            println!("  writes: {}", pty.writes.len());
            println!("  mods: {}", pty.mods.len());
            println!("  deletes: {}", pty.deletes.len());
            if pty.excluded_count > 0 {
                println!("  excluded_by_patterns: true ({})", pty.excluded_count);
            } else {
                println!("  excluded_by_patterns: false");
            }
        } else {
            println!("substrate: PTY pending diffs unsupported by this backend");
        }

        let combined_total_paths = {
            let mut set: HashSet<&str> = HashSet::new();
            for item in state
                .combined
                .writes
                .iter()
                .chain(state.combined.mods.iter())
                .chain(state.combined.deletes.iter())
            {
                set.insert(item.as_str());
            }
            set.len()
        };
        let pty_total_paths = state.pty.as_ref().map(|p| p.total_paths()).unwrap_or(0);
        println!("substrate: pending diff summary (combined)");
        println!("  total_paths: {combined_total_paths}");
        println!("  non_pty_total_paths: {non_pty_total_paths}");
        println!("  pty_total_paths: {pty_total_paths}");

        return Ok(0);
    }

    if matches!(direction, SyncDirection::FromHost | SyncDirection::Both) {
        if conflict_policy == SyncConflictPolicy::Abort && !from_host_conflicts.is_empty() {
            errln!("substrate: workspace sync refused: conflicts detected (policy=abort)");
            for item in &from_host_conflicts {
                errln!("  - {item}");
            }
            return Ok(5);
        }

        if conflict_policy == SyncConflictPolicy::PreferHost && !from_host_conflicts.is_empty() {
            let reconcile_request = agent_api_types::PendingDiffReconcileRequestV1 {
                profile: request.profile.clone(),
                cwd: request.cwd.clone(),
                env: request.env.clone(),
                agent_id: request.agent_id.clone(),
                policy_snapshot: request.policy_snapshot.clone(),
                diff_id: record.diff_id.clone(),
                discard_paths: from_host_conflicts.clone(),
            };

            let resp = match rt
                .block_on(async { client.pending_diff_reconcile(reconcile_request).await })
            {
                Ok(r) => r,
                Err(err) => {
                    errln!("substrate: workspace sync failed: from_host reconciliation failed");
                    errln!("{err:#}");
                    return Ok(1);
                }
            };

            if !resp.reconciled {
                errln!("substrate: workspace sync failed: from_host reconciliation did not apply");
                errln!("  reason: diff_id mismatch (concurrent changes detected)");
                return Ok(1);
            }
        }

        if direction == SyncDirection::FromHost {
            return Ok(0);
        }

        // direction=both: re-fetch pending diff after reconciliation to apply the updated snapshot.
        record = match fetch_record!() {
            Ok(r) => r,
            Err(code) => return Ok(code),
        };
        state = match build_pending_diff_state!(&record) {
            Ok(s) => s,
            Err(code) => return Ok(code),
        };
    }

    let raw_paths_for_decisions = state.raw_for_decisions.clone();
    let out_writes = state.combined.writes;
    let out_mods = state.combined.mods;
    let out_deletes = state.combined.deletes;
    let excluded_count = state.combined.excluded_count;

    let total_paths = out_writes.len() + out_mods.len() + out_deletes.len();

    // WS5: apply pending diffs (direction=from_world, including non-PTY + PTY when present).
    if total_paths > 10000 {
        errln!("substrate: workspace sync refused: size guard exceeded");
        errln!("  max_paths: 10000");
        errln!("  observed_paths: {total_paths}");
        return Ok(5);
    }

    let has_pending_diff_clear = capabilities_has_feature(&caps, "pending_diff_clear_v1");
    let has_world_fs_read = capabilities_has_feature(&caps, "world_fs_read_v1");
    if !has_pending_diff_clear || !has_world_fs_read {
        errln!("substrate: workspace sync apply is unsupported by this backend");
        if !has_pending_diff_clear {
            errln!("  missing feature: pending_diff_clear_v1");
        }
        if !has_world_fs_read {
            errln!("  missing feature: world_fs_read_v1");
        }
        return Ok(4);
    }

    let meta_paths: Vec<String> = out_writes.iter().chain(out_mods.iter()).cloned().collect();

    let base_request = agent_api_types::WorldFsReadRequestV1 {
        profile: request.profile.clone(),
        cwd: request.cwd.clone(),
        env: request.env.clone(),
        agent_id: request.agent_id.clone(),
        policy_snapshot: request.policy_snapshot.clone(),
        path: String::new(),
        include_contents: false,
    };

    let world_meta: HashMap<String, agent_api_types::WorldFsReadResponseV1> =
        match rt.block_on(async {
            let mut out: HashMap<String, agent_api_types::WorldFsReadResponseV1> = HashMap::new();
            for path in &meta_paths {
                let req = agent_api_types::WorldFsReadRequestV1 {
                    path: path.clone(),
                    include_contents: false,
                    ..base_request.clone()
                };
                let resp = client.world_fs_read(req).await?;
                out.insert(path.clone(), resp);
            }
            Ok::<_, anyhow::Error>(out)
        }) {
            Ok(map) => map,
            Err(err) => {
                errln!("substrate: workspace sync failed: failed to read world file metadata");
                errln!("{err:#}");
                return Ok(1);
            }
        };

    let mut observed_bytes_to_copy: u64 = 0;
    for path in &meta_paths {
        let Some(meta) = world_meta.get(path) else {
            errln!("substrate: workspace sync failed: missing world metadata for {path}");
            return Ok(1);
        };

        match meta.entry_type {
            agent_api_types::WorldFsEntryTypeV1::RegularFile => {
                let Some(size) = meta.size else {
                    errln!("substrate: workspace sync failed: missing size for {path}");
                    return Ok(1);
                };
                observed_bytes_to_copy = observed_bytes_to_copy.saturating_add(size);
            }
            agent_api_types::WorldFsEntryTypeV1::Directory => {}
            _ => {
                errln!("substrate: workspace sync refused: unsupported file type in apply set");
                errln!("  path: {path}");
                errln!("  file_type: {:?}", meta.entry_type);
                return Ok(5);
            }
        }
    }

    if observed_bytes_to_copy > 104_857_600 {
        errln!("substrate: workspace sync refused: size guard exceeded");
        errln!("  max_bytes_to_copy: 104857600");
        errln!("  observed_bytes_to_copy: {observed_bytes_to_copy}");
        return Ok(5);
    }

    let baseline = match chrono::DateTime::parse_from_rfc3339(&record.session_started_at) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(err) => {
            errln!("substrate: workspace sync failed: invalid session_started_at timestamp");
            errln!("{err:#}");
            return Ok(1);
        }
    };

    let mut conflicts: Vec<String> = Vec::new();
    for path in out_writes
        .iter()
        .chain(out_mods.iter())
        .chain(out_deletes.iter())
    {
        let host_path = workspace_root.join(path);
        let meta = match fs::symlink_metadata(&host_path) {
            Ok(m) => m,
            Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
            Err(err) => {
                errln!(
                    "substrate: workspace sync failed: failed to stat {}",
                    host_path.display()
                );
                errln!("{err:#}");
                return Ok(1);
            }
        };

        let modified = match meta.modified() {
            Ok(m) => chrono::DateTime::<chrono::Utc>::from(m),
            Err(err) => {
                errln!(
                    "substrate: workspace sync failed: failed to read mtime for {}",
                    host_path.display()
                );
                errln!("{err:#}");
                return Ok(1);
            }
        };

        if modified > baseline {
            conflicts.push(path.clone());
        }
    }
    conflicts.sort();
    conflicts.dedup();

    if conflict_policy == SyncConflictPolicy::Abort && !conflicts.is_empty() {
        errln!("substrate: workspace sync refused: conflicts detected (policy=abort)");
        for item in &conflicts {
            errln!("  - {item}");
        }
        return Ok(5);
    }

    let conflicts_set: HashSet<String> = conflicts.iter().cloned().collect();
    let skipped_by_conflict: usize = if conflict_policy == SyncConflictPolicy::PreferHost {
        conflicts_set.len()
    } else {
        0
    };

    let mut apply_writes = out_writes;
    let mut apply_mods = out_mods;
    let mut apply_deletes = out_deletes;

    if conflict_policy == SyncConflictPolicy::PreferHost && !conflicts_set.is_empty() {
        apply_writes.retain(|p| !conflicts_set.contains(p));
        apply_mods.retain(|p| !conflicts_set.contains(p));
        apply_deletes.retain(|p| !conflicts_set.contains(p));
    }

    let mut deletes_sorted = apply_deletes.clone();
    deletes_sorted.sort_by(|a, b| {
        let depth_a = a.split('/').count();
        let depth_b = b.split('/').count();
        depth_b.cmp(&depth_a).then_with(|| b.cmp(a))
    });

    let mut writes_mods_sorted: Vec<(bool, String)> = Vec::new();
    for p in &apply_writes {
        writes_mods_sorted.push((true, p.clone()));
    }
    for p in &apply_mods {
        writes_mods_sorted.push((false, p.clone()));
    }
    writes_mods_sorted.sort_by(|a, b| a.1.cmp(&b.1));

    for path in deletes_sorted {
        let host_path = workspace_root.join(&path);
        let meta = match fs::symlink_metadata(&host_path) {
            Ok(m) => m,
            Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
            Err(err) => {
                errln!(
                    "substrate: workspace sync failed: failed to stat {}",
                    host_path.display()
                );
                errln!("{err:#}");
                return Ok(1);
            }
        };
        let ft = meta.file_type();
        let result = if ft.is_dir() && !ft.is_symlink() {
            fs::remove_dir_all(&host_path)
        } else {
            fs::remove_file(&host_path)
        };
        if let Err(err) = result {
            errln!(
                "substrate: workspace sync failed: failed to delete {}",
                host_path.display()
            );
            errln!("{err:#}");
            return Ok(1);
        }
    }

    for (_is_write, path) in &writes_mods_sorted {
        let meta_req = agent_api_types::WorldFsReadRequestV1 {
            path: path.clone(),
            include_contents: false,
            ..base_request.clone()
        };

        let meta = match rt.block_on(async { client.world_fs_read(meta_req).await }) {
            Ok(m) => m,
            Err(err) => {
                errln!(
                    "substrate: workspace sync failed: failed to read world metadata for {path}"
                );
                errln!("{err:#}");
                return Ok(1);
            }
        };

        let host_path = workspace_root.join(path);
        match meta.entry_type {
            agent_api_types::WorldFsEntryTypeV1::Directory => {
                if let Err(err) = fs::create_dir_all(&host_path) {
                    errln!(
                        "substrate: workspace sync failed: failed to create directory {}",
                        host_path.display()
                    );
                    errln!("{err:#}");
                    return Ok(1);
                }
                #[cfg(unix)]
                if let Some(mode) = meta.mode {
                    if let Err(err) = apply_execute_bits(&host_path, mode) {
                        errln!("{err:#}");
                        return Ok(1);
                    }
                }
            }
            agent_api_types::WorldFsEntryTypeV1::RegularFile => {
                let read_req = agent_api_types::WorldFsReadRequestV1 {
                    path: path.clone(),
                    include_contents: true,
                    ..base_request.clone()
                };
                let file = match rt.block_on(async { client.world_fs_read(read_req).await }) {
                    Ok(f) => f,
                    Err(err) => {
                        errln!(
                            "substrate: workspace sync failed: failed to read world file for {path}"
                        );
                        errln!("{err:#}");
                        return Ok(1);
                    }
                };
                let contents_b64 = file.contents_b64.unwrap_or_default();
                let bytes = match base64::engine::general_purpose::STANDARD.decode(contents_b64) {
                    Ok(b) => b,
                    Err(err) => {
                        errln!("substrate: workspace sync failed: invalid base64 for {path}");
                        errln!("{err:#}");
                        return Ok(1);
                    }
                };

                if let Err(err) = write_atomic_bytes(&host_path, &bytes) {
                    errln!(
                        "substrate: workspace sync failed: failed to write {}",
                        host_path.display()
                    );
                    errln!("{err:#}");
                    return Ok(1);
                }

                #[cfg(unix)]
                if let Some(mode) = file.mode {
                    if let Err(err) = apply_execute_bits(&host_path, mode) {
                        errln!("{err:#}");
                        return Ok(1);
                    }
                }
            }
            _ => {
                errln!("substrate: workspace sync refused: unsupported file type in apply set");
                errln!("  path: {path}");
                errln!("  file_type: {:?}", meta.entry_type);
                return Ok(5);
            }
        }
    }

    let clear_request = agent_api_types::PendingDiffClearRequestV1 {
        profile: request.profile.clone(),
        cwd: request.cwd.clone(),
        env: request.env.clone(),
        agent_id: request.agent_id.clone(),
        policy_snapshot: request.policy_snapshot.clone(),
        diff_id: record.diff_id.clone(),
    };

    let cleared = match rt.block_on(async { client.pending_diff_clear(clear_request).await }) {
        Ok(resp) => resp.cleared,
        Err(err) => {
            errln!("substrate: workspace sync: applied but pending diffs were not cleared");
            errln!("{err:#}");
            return Ok(1);
        }
    };
    if !cleared {
        errln!("substrate: workspace sync: applied but pending diffs were not cleared");
        errln!("  reason: diff_id mismatch (concurrent changes detected)");
        return Ok(1);
    }

    println!("substrate: workspace sync applied");
    if args.verbose {
        println!("  session_started_at: {}", record.session_started_at);
        println!("  diff_id: {}", record.diff_id);
    }
    println!("  writes_applied: {}", apply_writes.len());
    println!("  mods_applied: {}", apply_mods.len());
    println!("  deletes_applied: {}", apply_deletes.len());
    println!("  skipped_by_exclude: {excluded_count}");
    if conflict_policy == SyncConflictPolicy::PreferHost {
        println!("  skipped_by_conflict: {skipped_by_conflict}");
    }

    if args.verbose {
        let mut decisions: Vec<(String, &'static str, &'static str, &'static str)> = Vec::new();
        for item in raw_paths_for_decisions {
            let origin = item.origin.as_str();
            let bucket = item.bucket;
            let path = item.path;
            let excluded = excludes_non_protected
                .iter()
                .any(|pat| glob_matches_path(pat, &path));
            let decision = if excluded {
                "skip_exclude"
            } else if conflict_policy == SyncConflictPolicy::PreferHost
                && conflicts_set.contains(&path)
            {
                "skip_conflict"
            } else {
                "apply"
            };
            decisions.push((path, origin, bucket, decision));
        }
        decisions.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.cmp(b.1))
                .then_with(|| a.2.cmp(b.2))
        });
        println!("  decisions:");
        for (path, origin, bucket, decision) in decisions {
            println!("    - {origin}:{bucket} {path} => {decision}");
        }
    }

    Ok(0)
}

fn run_workspace_checkpoint(args: &WorkspaceCheckpointArgs) -> Result<i32> {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace_root = require_workspace_root(&cwd, "workspace checkpoint")?;

    // Per internal-git-spec.md, the directory is created by `workspace init` only.
    let internal_git_dir = workspace::internal_git_dir(&workspace_root);
    if !internal_git_dir.is_dir() {
        eprintln!(
            "substrate: workspace checkpoint requires internal git dir; run `substrate workspace init --force`"
        );
        return Ok(2);
    }

    match Command::new("git").arg("--version").output() {
        Ok(output) if output.status.success() => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            eprintln!("substrate: workspace checkpoint requires git; install git and retry");
            return Ok(3);
        }
        Ok(_) | Err(_) => {
            return Err(anyhow!(
                "failed to validate git dependency for workspace checkpoint"
            ));
        }
    }

    ensure_internal_git_initialized(&workspace_root, &internal_git_dir)
        .context("failed to initialize internal git repo")?;

    internal_git_add_all(&workspace_root, &internal_git_dir)
        .context("failed to stage workspace snapshot")?;

    let has_staged_changes = internal_git_has_staged_changes(&workspace_root, &internal_git_dir)
        .context("failed to detect staged changes")?;
    if !has_staged_changes {
        println!("no-op");
        return Ok(0);
    }

    let checkpoint_id = internal_git_next_checkpoint_id(&workspace_root, &internal_git_dir)
        .context("failed to allocate checkpoint id (cp/<YYYYMMDDTHHMMSSZ>)")?;

    let commit_message = match args.message.as_deref() {
        Some(raw) => {
            if raw.contains('\n') || raw.contains('\r') {
                eprintln!("substrate: invalid --message (must be single-line text)");
                return Ok(2);
            }
            format!("checkpoint: {checkpoint_id} {raw}")
        }
        None => format!("checkpoint: {checkpoint_id}"),
    };

    internal_git_commit(&workspace_root, &internal_git_dir, &commit_message)
        .context("failed to create internal git commit")?;
    internal_git_tag(&workspace_root, &internal_git_dir, &checkpoint_id)
        .context("failed to tag internal git checkpoint")?;

    if args.verbose {
        if let Ok(Some(head)) = internal_git_rev_parse_head(&workspace_root, &internal_git_dir) {
            eprintln!("substrate: workspace checkpoint created");
            eprintln!("  checkpoint_id: {checkpoint_id}");
            eprintln!("  commit: {head}");
        }
    }

    // Stable stdout contract: print the created checkpoint id.
    println!("{checkpoint_id}");
    Ok(0)
}

fn run_workspace_rollback(args: &WorkspaceRollbackArgs) -> Result<i32> {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace_root = require_workspace_root(&cwd, "workspace rollback")?;

    let raw_target = match args.target.as_deref() {
        Some(t) => t.trim(),
        None => {
            eprintln!(
                "substrate: workspace rollback requires a target (`last` or cp/<YYYYMMDDTHHMMSSZ>)"
            );
            return Ok(2);
        }
    };
    if raw_target.is_empty() {
        eprintln!(
            "substrate: workspace rollback requires a target (`last` or cp/<YYYYMMDDTHHMMSSZ>)"
        );
        return Ok(2);
    }
    if raw_target.chars().any(|c| c.is_whitespace()) {
        eprintln!("substrate: invalid rollback target (must not contain whitespace): {raw_target}");
        return Ok(2);
    }

    // Per internal-git-spec.md, the directory is created by `workspace init` only.
    let internal_git_dir = workspace::internal_git_dir(&workspace_root);
    if !internal_git_dir.is_dir() {
        eprintln!(
            "substrate: workspace rollback requires internal git dir; run `substrate workspace init --force`"
        );
        return Ok(2);
    }

    match Command::new("git").arg("--version").output() {
        Ok(output) if output.status.success() => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            eprintln!("substrate: workspace rollback requires git; install git and retry");
            return Ok(3);
        }
        Ok(_) | Err(_) => {
            return Err(anyhow!(
                "failed to validate git dependency for workspace rollback"
            ));
        }
    }

    let user_git_marker = workspace_root.join(".git");
    if user_git_marker.exists() {
        let mut cmd = Command::new("git");
        cmd.current_dir(&workspace_root);
        cmd.arg("status").arg("--porcelain");
        let rendered = format!("{cmd:?}");
        let output = cmd
            .output()
            .with_context(|| format!("failed to run git command: {rendered}"))?;
        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "git status failed (exit={}): stdout={} stderr={}",
                output.status,
                stdout.trim(),
                stderr.trim()
            ));
        }
        let dirty = !String::from_utf8_lossy(&output.stdout).trim().is_empty();
        if dirty && !args.force {
            eprintln!(
                "substrate: workspace rollback refused: workspace is dirty; re-run with --force"
            );
            return Ok(5);
        }
    }

    ensure_internal_git_initialized(&workspace_root, &internal_git_dir)
        .context("failed to initialize internal git repo")?;

    let checkpoint_id = if raw_target == "last" {
        match internal_git_last_checkpoint_id(&workspace_root, &internal_git_dir)
            .context("failed to resolve rollback target `last`")?
        {
            Some(id) => id,
            None => {
                eprintln!("substrate: workspace rollback failed: no checkpoints exist");
                return Ok(2);
            }
        }
    } else {
        if !checkpoint_id_looks_valid(raw_target) {
            eprintln!("substrate: invalid rollback target checkpoint id: {raw_target}");
            return Ok(2);
        }
        raw_target.to_string()
    };

    let target_commit = match internal_git_rev_parse_tag_commit(
        &workspace_root,
        &internal_git_dir,
        &checkpoint_id,
    )
    .context("failed to resolve rollback target commit")?
    {
        Some(commit) => commit,
        None => {
            eprintln!("substrate: workspace rollback failed: invalid target checkpoint id: {checkpoint_id}");
            return Ok(2);
        }
    };

    let snapshot_files =
        internal_git_ls_tree_files(&workspace_root, &internal_git_dir, &target_commit)
            .context("failed to enumerate snapshot files for rollback target")?;
    let snapshot_required_paths = snapshot_required_paths(&snapshot_files);

    let extra_paths = workspace_paths_not_in_snapshot(&workspace_root, &snapshot_required_paths)
        .context("failed to evaluate rollback safety rails")?;
    if !extra_paths.is_empty() && !args.force {
        eprintln!("substrate: workspace rollback refused: would delete non-checkpointed paths; re-run with --force");
        print_path_list_truncated("  paths", &extra_paths);
        return Ok(5);
    }

    internal_git_checkout_main(&workspace_root, &internal_git_dir)
        .context("failed to checkout internal branch main for rollback")?;
    internal_git_reset_hard(&workspace_root, &internal_git_dir, &target_commit)
        .context("failed to restore workspace to rollback target")?;

    if args.force && !extra_paths.is_empty() {
        delete_workspace_paths(&workspace_root, &extra_paths)
            .context("failed to delete non-checkpointed paths during rollback")?;
    }

    if args.verbose {
        eprintln!("substrate: workspace rollback applied");
        eprintln!("  checkpoint_id: {checkpoint_id}");
        eprintln!("  commit: {target_commit}");
        if args.force {
            eprintln!("  deleted_non_checkpointed_paths: {}", extra_paths.len());
        }
    }

    Ok(0)
}

fn checkpoint_id_looks_valid(raw: &str) -> bool {
    if !raw.starts_with("cp/") {
        return false;
    }
    let ts = match raw.strip_prefix("cp/") {
        Some(t) => t,
        None => return false,
    };
    chrono::NaiveDateTime::parse_from_str(ts, "%Y%m%dT%H%M%SZ").is_ok()
}

fn internal_git_rev_parse_tag_commit(
    workspace_root: &Path,
    internal_git_dir: &Path,
    tag: &str,
) -> Result<Option<String>> {
    let mut cmd = internal_git_base_cmd(workspace_root, internal_git_dir);
    cmd.arg("rev-parse")
        .arg("--verify")
        .arg("--quiet")
        .arg(format!("refs/tags/{tag}^{{commit}}"));
    let out = match cmd.output() {
        Ok(o) => o,
        Err(err) => return Err(anyhow!("failed to run git rev-parse for tag {tag}: {err}")),
    };
    if !out.status.success() {
        return Ok(None);
    }
    Ok(Some(
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
    ))
}

fn internal_git_ls_tree_files(
    workspace_root: &Path,
    internal_git_dir: &Path,
    commit: &str,
) -> Result<Vec<String>> {
    let mut cmd = internal_git_base_cmd(workspace_root, internal_git_dir);
    cmd.arg("ls-tree").arg("-r").arg("--name-only").arg(commit);
    let out = internal_git_run(cmd)?;
    let mut files: Vec<String> = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let normalized = normalize_workspace_rel_path(trimmed)
            .map_err(|msg| anyhow!("invalid snapshot path from git ls-tree: {msg}"))?;
        files.push(normalized);
    }
    Ok(files)
}

fn snapshot_required_paths(snapshot_files: &[String]) -> HashSet<String> {
    let mut required: HashSet<String> = HashSet::new();
    for file in snapshot_files {
        required.insert(file.clone());
        let parts: Vec<&str> = file.split('/').collect();
        if parts.len() <= 1 {
            continue;
        }
        let mut prefix = String::new();
        for part in &parts[..parts.len() - 1] {
            if prefix.is_empty() {
                prefix.push_str(part);
            } else {
                prefix.push('/');
                prefix.push_str(part);
            }
            required.insert(prefix.clone());
        }
    }
    required
}

fn is_protected_workspace_path(path: &str) -> bool {
    for prot in crate::execution::config_model::PROTECTED_EXCLUDES {
        if glob_matches_path(prot, path) {
            return true;
        }
    }
    false
}

fn workspace_paths_not_in_snapshot(
    workspace_root: &Path,
    snapshot_required_paths: &HashSet<String>,
) -> Result<Vec<String>> {
    fn walk(
        workspace_root: &Path,
        dir: &Path,
        snapshot_required_paths: &HashSet<String>,
        out: &mut Vec<String>,
    ) -> Result<()> {
        let entries = fs::read_dir(dir)
            .with_context(|| format!("failed to read directory {}", dir.display()))?;
        for entry in entries {
            let entry = entry
                .with_context(|| format!("failed to read directory entry in {}", dir.display()))?;
            let path = entry.path();
            let rel = match path.strip_prefix(workspace_root) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let rel_str = rel.to_string_lossy();
            let normalized = normalize_workspace_rel_path(&rel_str)
                .map_err(|msg| anyhow!("failed to normalize workspace path {rel_str}: {msg}"))?;

            if is_protected_workspace_path(&normalized) {
                continue;
            }

            if !snapshot_required_paths.contains(&normalized) {
                out.push(normalized.clone());
            }

            let meta = fs::symlink_metadata(&path)
                .with_context(|| format!("failed to stat {}", path.display()))?;
            let ft = meta.file_type();
            if ft.is_dir() && !ft.is_symlink() {
                walk(workspace_root, &path, snapshot_required_paths, out)?;
            }
        }
        Ok(())
    }

    let mut out: Vec<String> = Vec::new();
    walk(
        workspace_root,
        workspace_root,
        snapshot_required_paths,
        &mut out,
    )?;
    out.sort();
    out.dedup();
    Ok(out)
}

fn delete_workspace_paths(workspace_root: &Path, paths: &[String]) -> Result<()> {
    let mut sorted = paths.to_vec();
    sorted.sort_by(|a, b| {
        let depth_a = a.split('/').count();
        let depth_b = b.split('/').count();
        depth_b.cmp(&depth_a).then_with(|| b.cmp(a))
    });

    for path in sorted {
        if is_protected_workspace_path(&path) {
            continue;
        }
        let host_path = workspace_root.join(&path);
        let meta = match fs::symlink_metadata(&host_path) {
            Ok(m) => m,
            Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
            Err(err) => return Err(anyhow!("failed to stat {}: {err}", host_path.display())),
        };
        let ft = meta.file_type();
        let result = if ft.is_dir() && !ft.is_symlink() {
            fs::remove_dir_all(&host_path)
        } else {
            fs::remove_file(&host_path)
        };
        match result {
            Ok(()) => {}
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(anyhow!(
                    "failed to delete {} during rollback: {err}",
                    host_path.display()
                ))
            }
        }
    }
    Ok(())
}

fn internal_git_checkout_main(workspace_root: &Path, internal_git_dir: &Path) -> Result<()> {
    let mut cmd = internal_git_base_cmd(workspace_root, internal_git_dir);
    cmd.arg("checkout").arg("-f").arg("main");
    internal_git_run(cmd).map(|_| ())
}

fn internal_git_reset_hard(
    workspace_root: &Path,
    internal_git_dir: &Path,
    commit: &str,
) -> Result<()> {
    let mut cmd = internal_git_base_cmd(workspace_root, internal_git_dir);
    cmd.arg("reset").arg("--hard").arg(commit);
    internal_git_run(cmd).map(|_| ())
}

fn print_path_list_truncated(label: &str, paths: &[String]) {
    const MAX: usize = 50;
    eprintln!("{label}:");
    for item in paths.iter().take(MAX) {
        eprintln!("    - {item}");
    }
    if paths.len() > MAX {
        eprintln!("    - ... ({} more)", paths.len().saturating_sub(MAX));
    }
}

fn internal_git_base_cmd(workspace_root: &Path, internal_git_dir: &Path) -> Command {
    let mut cmd = Command::new("git");
    cmd.current_dir(workspace_root);
    cmd.arg("--git-dir")
        .arg(internal_git_dir)
        .arg("--work-tree")
        .arg(workspace_root);

    cmd.env("GIT_TERMINAL_PROMPT", "0");
    cmd.env("GIT_AUTHOR_NAME", "Substrate");
    cmd.env("GIT_AUTHOR_EMAIL", "substrate@localhost");
    cmd.env("GIT_COMMITTER_NAME", "Substrate");
    cmd.env("GIT_COMMITTER_EMAIL", "substrate@localhost");

    cmd
}

fn internal_git_run(mut cmd: Command) -> Result<Output> {
    let rendered = format!("{cmd:?}");
    let output = cmd
        .output()
        .with_context(|| format!("failed to run git command: {rendered}"))?;
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "git command failed (exit={}): stdout={} stderr={}",
            output.status,
            stdout.trim(),
            stderr.trim()
        ));
    }
    Ok(output)
}

fn ensure_internal_git_initialized(workspace_root: &Path, internal_git_dir: &Path) -> Result<()> {
    let head = internal_git_dir.join("HEAD");
    if head.exists() {
        return Ok(());
    }

    let mut cmd = internal_git_base_cmd(workspace_root, internal_git_dir);
    cmd.arg("init").arg("--initial-branch=main");
    internal_git_run(cmd).map(|_| ())
}

fn internal_git_add_all(workspace_root: &Path, internal_git_dir: &Path) -> Result<()> {
    let mut cmd = internal_git_base_cmd(workspace_root, internal_git_dir);
    cmd.arg("add")
        .arg("-A")
        .arg("-f")
        .arg("--")
        .arg(".")
        .arg(":!.git")
        .arg(":!.substrate");
    internal_git_run(cmd).map(|_| ())
}

fn internal_git_has_staged_changes(workspace_root: &Path, internal_git_dir: &Path) -> Result<bool> {
    let mut cmd = internal_git_base_cmd(workspace_root, internal_git_dir);
    cmd.arg("status")
        .arg("--porcelain")
        .arg("--untracked-files=no");
    let out = internal_git_run(cmd)?;
    Ok(!String::from_utf8_lossy(&out.stdout).trim().is_empty())
}

fn internal_git_next_checkpoint_id(
    workspace_root: &Path,
    internal_git_dir: &Path,
) -> Result<String> {
    let mut candidate = format!("cp/{}", chrono::Utc::now().format("%Y%m%dT%H%M%SZ"));

    if let Some(last) = internal_git_last_checkpoint_id(workspace_root, internal_git_dir)? {
        if candidate <= last {
            let last_ts = last
                .strip_prefix("cp/")
                .ok_or_else(|| anyhow!("invalid checkpoint tag: {last}"))?;
            let last_dt = chrono::NaiveDateTime::parse_from_str(last_ts, "%Y%m%dT%H%M%SZ")
                .with_context(|| format!("invalid checkpoint timestamp in tag: {last}"))?;
            let next = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                last_dt + chrono::Duration::seconds(1),
                chrono::Utc,
            );
            candidate = format!("cp/{}", next.format("%Y%m%dT%H%M%SZ"));
        }
    }

    Ok(candidate)
}

fn internal_git_last_checkpoint_id(
    workspace_root: &Path,
    internal_git_dir: &Path,
) -> Result<Option<String>> {
    let mut cmd = internal_git_base_cmd(workspace_root, internal_git_dir);
    cmd.arg("tag").arg("--list").arg("cp/*");
    let out = internal_git_run(cmd)?;
    let mut tags: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    tags.sort();
    Ok(tags.pop())
}

fn internal_git_rev_parse_head(
    workspace_root: &Path,
    internal_git_dir: &Path,
) -> Result<Option<String>> {
    let mut cmd = internal_git_base_cmd(workspace_root, internal_git_dir);
    cmd.arg("rev-parse").arg("HEAD");
    let out = match cmd.output() {
        Ok(o) => o,
        Err(err) => return Err(anyhow!("failed to run git rev-parse HEAD: {err}")),
    };
    if !out.status.success() {
        return Ok(None);
    }
    Ok(Some(
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
    ))
}

fn internal_git_commit(
    workspace_root: &Path,
    internal_git_dir: &Path,
    message: &str,
) -> Result<()> {
    let mut cmd = internal_git_base_cmd(workspace_root, internal_git_dir);
    cmd.arg("-c")
        .arg("commit.gpgsign=false")
        .arg("-c")
        .arg("user.name=Substrate")
        .arg("-c")
        .arg("user.email=substrate@localhost")
        .arg("commit")
        .arg("-m")
        .arg(message);
    internal_git_run(cmd).map(|_| ())
}

fn internal_git_tag(workspace_root: &Path, internal_git_dir: &Path, tag: &str) -> Result<()> {
    let mut cmd = internal_git_base_cmd(workspace_root, internal_git_dir);
    cmd.arg("-c").arg("tag.gpgSign=false").arg("tag").arg(tag);
    internal_git_run(cmd).map(|_| ())
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
