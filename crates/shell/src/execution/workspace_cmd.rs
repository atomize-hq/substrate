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

fn run_workspace_sync(args: &WorkspaceSyncArgs, cli: &Cli) -> Result<i32> {
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

    let record = match rt.block_on(async { client.pending_diff(request.clone()).await }) {
        Ok(r) => r,
        Err(err) => {
            // WS1-spec: when `--direction from_world` is in effect, a reachable world backend is
            // required. If pending diff retrieval fails after capabilities succeeded, prefer
            // treating it as a backend availability issue (exit 3) unless we can confidently
            // classify it as a non-transport/internal payload failure.
            let looks_like_payload_failure =
                err.chain().any(|cause| cause.is::<serde_json::Error>());
            if looks_like_payload_failure {
                eprintln!("substrate: workspace sync failed to retrieve pending diff");
                return Ok(1);
            }

            eprintln!(
                "substrate: workspace sync requires world; run `substrate world enable` then `substrate world doctor`"
            );
            return Ok(3);
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

    let raw_paths_for_decisions = raw_paths.clone();

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

    if args.dry_run {
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
        return Ok(0);
    }

    // WS2: apply pending diffs (direction=from_world, non-PTY only).
    if total_paths > 10000 {
        eprintln!("substrate: workspace sync refused: size guard exceeded");
        eprintln!("  max_paths: 10000");
        eprintln!("  observed_paths: {total_paths}");
        return Ok(5);
    }

    let has_pending_diff_clear = capabilities_has_feature(&caps, "pending_diff_clear_v1");
    let has_world_fs_read = capabilities_has_feature(&caps, "world_fs_read_v1");
    if !has_pending_diff_clear || !has_world_fs_read {
        eprintln!("substrate: workspace sync apply is unsupported by this backend");
        if !has_pending_diff_clear {
            eprintln!("  missing feature: pending_diff_clear_v1");
        }
        if !has_world_fs_read {
            eprintln!("  missing feature: world_fs_read_v1");
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
                eprintln!("substrate: workspace sync failed: failed to read world file metadata");
                eprintln!("{err:#}");
                return Ok(1);
            }
        };

    let mut observed_bytes_to_copy: u64 = 0;
    for path in &meta_paths {
        let Some(meta) = world_meta.get(path) else {
            eprintln!("substrate: workspace sync failed: missing world metadata for {path}");
            return Ok(1);
        };

        match meta.entry_type {
            agent_api_types::WorldFsEntryTypeV1::RegularFile => {
                let Some(size) = meta.size else {
                    eprintln!("substrate: workspace sync failed: missing size for {path}");
                    return Ok(1);
                };
                observed_bytes_to_copy = observed_bytes_to_copy.saturating_add(size);
            }
            agent_api_types::WorldFsEntryTypeV1::Directory => {}
            _ => {
                eprintln!("substrate: workspace sync refused: unsupported file type in apply set");
                eprintln!("  path: {path}");
                eprintln!("  file_type: {:?}", meta.entry_type);
                return Ok(5);
            }
        }
    }

    if observed_bytes_to_copy > 104_857_600 {
        eprintln!("substrate: workspace sync refused: size guard exceeded");
        eprintln!("  max_bytes_to_copy: 104857600");
        eprintln!("  observed_bytes_to_copy: {observed_bytes_to_copy}");
        return Ok(5);
    }

    let baseline = match chrono::DateTime::parse_from_rfc3339(&record.session_started_at) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(err) => {
            eprintln!("substrate: workspace sync failed: invalid session_started_at timestamp");
            eprintln!("{err:#}");
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
                eprintln!(
                    "substrate: workspace sync failed: failed to stat {}",
                    host_path.display()
                );
                eprintln!("{err:#}");
                return Ok(1);
            }
        };

        let modified = match meta.modified() {
            Ok(m) => chrono::DateTime::<chrono::Utc>::from(m),
            Err(err) => {
                eprintln!(
                    "substrate: workspace sync failed: failed to read mtime for {}",
                    host_path.display()
                );
                eprintln!("{err:#}");
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
        eprintln!("substrate: workspace sync refused: conflicts detected (policy=abort)");
        for item in &conflicts {
            eprintln!("  - {item}");
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
                eprintln!(
                    "substrate: workspace sync failed: failed to stat {}",
                    host_path.display()
                );
                eprintln!("{err:#}");
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
            eprintln!(
                "substrate: workspace sync failed: failed to delete {}",
                host_path.display()
            );
            eprintln!("{err:#}");
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
                eprintln!(
                    "substrate: workspace sync failed: failed to read world metadata for {path}"
                );
                eprintln!("{err:#}");
                return Ok(1);
            }
        };

        let host_path = workspace_root.join(path);
        match meta.entry_type {
            agent_api_types::WorldFsEntryTypeV1::Directory => {
                if let Err(err) = fs::create_dir_all(&host_path) {
                    eprintln!(
                        "substrate: workspace sync failed: failed to create directory {}",
                        host_path.display()
                    );
                    eprintln!("{err:#}");
                    return Ok(1);
                }
                #[cfg(unix)]
                if let Some(mode) = meta.mode {
                    if let Err(err) = apply_execute_bits(&host_path, mode) {
                        eprintln!("{err:#}");
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
                        eprintln!("substrate: workspace sync failed: failed to read world file for {path}");
                        eprintln!("{err:#}");
                        return Ok(1);
                    }
                };
                let contents_b64 = file.contents_b64.unwrap_or_default();
                let bytes = match base64::engine::general_purpose::STANDARD.decode(contents_b64) {
                    Ok(b) => b,
                    Err(err) => {
                        eprintln!("substrate: workspace sync failed: invalid base64 for {path}");
                        eprintln!("{err:#}");
                        return Ok(1);
                    }
                };

                if let Err(err) = write_atomic_bytes(&host_path, &bytes) {
                    eprintln!(
                        "substrate: workspace sync failed: failed to write {}",
                        host_path.display()
                    );
                    eprintln!("{err:#}");
                    return Ok(1);
                }

                #[cfg(unix)]
                if let Some(mode) = file.mode {
                    if let Err(err) = apply_execute_bits(&host_path, mode) {
                        eprintln!("{err:#}");
                        return Ok(1);
                    }
                }
            }
            _ => {
                eprintln!("substrate: workspace sync refused: unsupported file type in apply set");
                eprintln!("  path: {path}");
                eprintln!("  file_type: {:?}", meta.entry_type);
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
            eprintln!("substrate: workspace sync: applied but pending diffs were not cleared");
            eprintln!("{err:#}");
            return Ok(1);
        }
    };
    if !cleared {
        eprintln!("substrate: workspace sync: applied but pending diffs were not cleared");
        eprintln!("  reason: diff_id mismatch (concurrent changes detected)");
        return Ok(1);
    }

    println!("substrate: workspace sync applied (non_pty)");
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
        let mut decisions: Vec<(String, &'static str, &'static str)> = Vec::new();
        for (bucket, path) in raw_paths_for_decisions {
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
            decisions.push((path, bucket, decision));
        }
        decisions.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1)));
        println!("  decisions:");
        for (path, bucket, decision) in decisions {
            println!("    - {bucket} {path} => {decision}");
        }
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
