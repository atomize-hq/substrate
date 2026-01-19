use crate::execution::cli::{
    AnchorModeArg, Cli, ConfigAction, ConfigCmd, ConfigCurrentAction, ConfigGlobalAction,
    ConfigGlobalCmd, ConfigInitArgs, ConfigResetArgs, ConfigSetArgs, ConfigShowArgs,
    ConfigWorkspaceAction,
};
use crate::execution::config_model::{
    self, CliConfigOverrides, ConfigExplainV1, SubstrateConfig, SubstrateConfigPatch,
};
use crate::execution::write_env_sh;
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

const DEFAULT_GLOBAL_PATCH_HEADER: &str = r#"# Substrate config patch (sparse overrides; scope=global).
# - This file is a YAML mapping of global-scoped config overrides.
# - Omitted keys inherit from defaults (and from workspace overrides when applicable).
# - View the effective merged config with: `substrate config current show --explain`
"#;

pub(crate) fn handle_config_command(cmd: &ConfigCmd, cli: &Cli) -> i32 {
    let result = match &cmd.action {
        ConfigAction::Current(cmd) => match &cmd.action {
            ConfigCurrentAction::Show(args) => run_current_show(args, cli),
        },
        ConfigAction::Show(args) => run_current_show(args, cli),
        ConfigAction::Set(args) => run_workspace_set(args, cli),
        ConfigAction::Global(cmd) => run_global(cmd),
        ConfigAction::Workspace(cmd) => match &cmd.action {
            ConfigWorkspaceAction::Show(args) => run_workspace_show(args),
            ConfigWorkspaceAction::Set(args) => run_workspace_set(args, cli),
            ConfigWorkspaceAction::Reset(args) => run_workspace_reset(args, cli),
        },
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

fn run_global(cmd: &ConfigGlobalCmd) -> Result<()> {
    match &cmd.action {
        ConfigGlobalAction::Init(args) => run_global_init(args),
        ConfigGlobalAction::Show(args) => run_global_show(args),
        ConfigGlobalAction::Set(args) => run_global_set(args),
        ConfigGlobalAction::Reset(args) => run_global_reset(args),
    }
}

fn run_global_init(args: &ConfigInitArgs) -> Result<()> {
    let path = config_model::global_config_path()?;
    let existed = path.exists();

    if existed && !args.force {
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let _ = config_model::parse_config_patch_yaml(&path, &raw)?;
        return Ok(());
    }

    let patch = SubstrateConfigPatch::default();
    write_atomic_patch_yaml(&path, DEFAULT_GLOBAL_PATCH_HEADER, None, &patch)
        .with_context(|| format!("failed to write {}", path.display()))?;
    config_model::invalidate_config_cache();
    let cfg = SubstrateConfig::default();
    write_env_sh(&cfg).context("failed to write env.sh")?;
    if existed {
        println!(
            "substrate: overwrote config at {} (--force)",
            path.display()
        );
    } else {
        println!("substrate: wrote default config to {}", path.display());
    }
    Ok(())
}

fn run_global_show(args: &ConfigShowArgs) -> Result<()> {
    if args.explain {
        return Err(config_model::user_error(
            "--explain is only supported for `substrate config current show`",
        ));
    }
    let (patch, _) = config_model::read_global_config_patch_or_empty()?;
    if patch.is_empty() {
        eprintln!("substrate: note: global config patch is empty (no overrides); run 'substrate config current show --explain' to view the effective config for this directory");
    }
    print_patch(&patch, args.json)?;
    Ok(())
}

fn run_global_set(args: &ConfigSetArgs) -> Result<()> {
    let path = config_model::global_config_path()?;
    let (mut patch, existed) = config_model::read_global_config_patch_or_empty()
        .with_context(|| format!("failed to load global config patch at {}", path.display()))?;

    let updates = config_model::parse_updates(&args.updates)?;
    let changed = config_model::apply_updates_to_patch(&mut patch, &updates)?;

    if changed || !existed {
        let header = if existed {
            Some(read_comment_header_prefix(&path)?)
        } else {
            None
        };
        write_atomic_patch_yaml(
            &path,
            DEFAULT_GLOBAL_PATCH_HEADER,
            header.as_deref(),
            &patch,
        )
        .with_context(|| format!("failed to write {}", path.display()))?;
        config_model::invalidate_config_cache();
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let (cfg, _) = config_model::resolve_effective_config_with_explain(
        &cwd,
        &CliConfigOverrides::default(),
        false,
    )?;
    write_env_sh(&cfg).context("failed to write env.sh")?;
    print_config(&cfg, args.json)?;
    Ok(())
}

fn run_global_reset(args: &ConfigResetArgs) -> Result<()> {
    let path = config_model::global_config_path()?;
    let (mut patch, existed) = config_model::read_global_config_patch_or_empty()
        .with_context(|| format!("failed to load global config patch at {}", path.display()))?;
    let changed = if args.keys.is_empty() {
        let was_empty = patch.is_empty();
        patch = SubstrateConfigPatch::default();
        !was_empty
    } else {
        config_model::reset_patch_keys(&mut patch, &args.keys)?
    };

    if changed || !existed {
        let header = if existed {
            Some(read_comment_header_prefix(&path)?)
        } else {
            None
        };
        write_atomic_patch_yaml(
            &path,
            DEFAULT_GLOBAL_PATCH_HEADER,
            header.as_deref(),
            &patch,
        )
        .with_context(|| format!("failed to write {}", path.display()))?;
        config_model::invalidate_config_cache();
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let (cfg, _) = config_model::resolve_effective_config_with_explain(
        &cwd,
        &CliConfigOverrides::default(),
        false,
    )?;
    write_env_sh(&cfg).context("failed to write env.sh")?;
    print_config(&cfg, false)?;
    Ok(())
}

fn run_current_show(args: &ConfigShowArgs, cli: &Cli) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let overrides = cli_overrides(cli);
    eprintln!(
        "substrate: note: showing effective merged config; use --explain to view per-key sources"
    );
    let (cfg, explain) =
        config_model::resolve_effective_config_with_explain(&cwd, &overrides, args.explain)?;
    if let Some(explain) = explain {
        print_explain(&explain)?;
    }
    print_config(&cfg, args.json)?;
    Ok(())
}

fn run_workspace_show(args: &ConfigShowArgs) -> Result<()> {
    if args.explain {
        return Err(config_model::user_error(
            "--explain is only supported for `substrate config current show`",
        ));
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace_root = require_workspace(&cwd)?;
    let path = crate::execution::workspace::workspace_marker_path(&workspace_root);
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let patch = config_model::parse_config_patch_yaml(&path, &raw)?;
    if patch.is_empty() {
        eprintln!("substrate: note: workspace config patch is empty (no overrides); run 'substrate config current show --explain' to view the effective config for this directory");
    }
    print_patch(&patch, args.json)?;
    Ok(())
}

fn run_workspace_set(args: &ConfigSetArgs, cli: &Cli) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace_root = require_workspace(&cwd)?;

    let path = crate::execution::workspace::workspace_marker_path(&workspace_root);
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let header = read_comment_header_prefix_from_raw(&raw);
    let mut patch = config_model::parse_config_patch_yaml(&path, &raw)?;

    let updates = config_model::parse_updates(&args.updates)?;
    let changed = config_model::apply_updates_to_patch(&mut patch, &updates)?;
    if changed {
        write_atomic_patch_yaml(&path, "", Some(&header), &patch)
            .with_context(|| format!("failed to write {}", path.display()))?;
        config_model::invalidate_config_cache();
    }

    let overrides = cli_overrides(cli);
    let (effective, _) =
        config_model::resolve_effective_config_with_explain(&cwd, &overrides, false)?;
    print_config(&effective, args.json)?;
    Ok(())
}

fn run_workspace_reset(args: &ConfigResetArgs, cli: &Cli) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace_root = require_workspace(&cwd)?;
    let path = crate::execution::workspace::workspace_marker_path(&workspace_root);
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let header = read_comment_header_prefix_from_raw(&raw);
    let mut patch = config_model::parse_config_patch_yaml(&path, &raw)?;

    let changed = if args.keys.is_empty() {
        let was_empty = patch.is_empty();
        patch = SubstrateConfigPatch::default();
        !was_empty
    } else {
        config_model::reset_patch_keys(&mut patch, &args.keys)?
    };
    if changed {
        write_atomic_patch_yaml(&path, "", Some(&header), &patch)
            .with_context(|| format!("failed to write {}", path.display()))?;
        config_model::invalidate_config_cache();
    }

    let overrides = cli_overrides(cli);
    let (effective, _) =
        config_model::resolve_effective_config_with_explain(&cwd, &overrides, false)?;
    print_config(&effective, false)?;
    Ok(())
}

fn require_workspace(cwd: &Path) -> Result<PathBuf> {
    crate::execution::workspace::find_workspace_root(cwd).ok_or_else(|| {
        actionable("substrate: not in a workspace; run `substrate workspace init`".to_string())
    })
}

fn print_config(cfg: &SubstrateConfig, json: bool) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string(cfg).context("failed to serialize JSON")?
        );
        return Ok(());
    }
    println!(
        "{}",
        serde_yaml::to_string(cfg).context("failed to serialize YAML")?
    );
    Ok(())
}

fn print_patch(patch: &SubstrateConfigPatch, json: bool) -> Result<()> {
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

fn print_explain(explain: &ConfigExplainV1) -> Result<()> {
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

fn write_atomic_patch_yaml(
    path: &Path,
    default_header: &str,
    existing_header: Option<&str>,
    patch: &SubstrateConfigPatch,
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

fn cli_overrides(cli: &Cli) -> CliConfigOverrides {
    let mut overrides = CliConfigOverrides::default();

    if cli.world {
        overrides.world_enabled = Some(true);
    } else if cli.no_world {
        overrides.world_enabled = Some(false);
    }

    if let Some(mode) = cli.anchor_mode {
        overrides.anchor_mode = Some(match mode {
            AnchorModeArg::Workspace => substrate_common::WorldRootMode::Project,
            AnchorModeArg::FollowCwd => substrate_common::WorldRootMode::FollowCwd,
            AnchorModeArg::Custom => substrate_common::WorldRootMode::Custom,
        });
    }

    if let Some(path) = &cli.anchor_path {
        overrides.anchor_path = Some(path.to_string_lossy().to_string());
    }

    if cli.caged {
        overrides.caged = Some(true);
    } else if cli.uncaged {
        overrides.caged = Some(false);
    }

    overrides
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::cli::{Cli as RootCli, SubCommands};
    use clap::Parser;
    use serial_test::serial;
    use std::fs;
    use tempfile::TempDir;

    struct CwdGuard {
        prev: std::path::PathBuf,
    }

    impl CwdGuard {
        fn set(path: &Path) -> Self {
            let prev = std::env::current_dir().unwrap();
            std::env::set_current_dir(path).unwrap();
            Self { prev }
        }
    }

    impl Drop for CwdGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.prev);
        }
    }

    fn write_workspace_yaml(root: &Path, body: &[u8]) -> PathBuf {
        let workspace_yaml = crate::execution::workspace::workspace_marker_path(root);
        fs::create_dir_all(workspace_yaml.parent().unwrap()).unwrap();
        fs::write(&workspace_yaml, body).unwrap();
        workspace_yaml
    }

    fn run_cli(args: &[&str]) -> i32 {
        let cli = RootCli::parse_from(args);
        let Some(SubCommands::Config(cmd)) = &cli.sub else {
            panic!("expected config command");
        };
        handle_config_command(cmd, &cli)
    }

    #[test]
    #[serial]
    fn test_invalid_enum_inventory_mode_is_exit_2_and_no_workspace_writes() {
        let tmp = TempDir::new().unwrap();
        let workspace_root = tmp.path().join("ws");
        fs::create_dir_all(&workspace_root).unwrap();
        let _cwd = CwdGuard::set(&workspace_root);

        let initial = b"# workspace patch\n";
        let workspace_yaml = write_workspace_yaml(&workspace_root, initial);
        let before = fs::read(&workspace_yaml).unwrap();

        let code = run_cli(&[
            "substrate",
            "config",
            "workspace",
            "set",
            "world.deps.inventory_mode=bogus",
        ]);
        assert_eq!(code, 2);

        let after = fs::read(&workspace_yaml).unwrap();
        assert_eq!(after, before);
    }

    #[test]
    #[serial]
    fn test_invalid_enum_builtins_is_exit_2_and_no_workspace_writes() {
        let tmp = TempDir::new().unwrap();
        let workspace_root = tmp.path().join("ws");
        fs::create_dir_all(&workspace_root).unwrap();
        let _cwd = CwdGuard::set(&workspace_root);

        let initial = b"# workspace patch\n";
        let workspace_yaml = write_workspace_yaml(&workspace_root, initial);
        let before = fs::read(&workspace_yaml).unwrap();

        let code = run_cli(&[
            "substrate",
            "config",
            "workspace",
            "set",
            "world.deps.builtins=bogus",
        ]);
        assert_eq!(code, 2);

        let after = fs::read(&workspace_yaml).unwrap();
        assert_eq!(after, before);
    }

    #[test]
    #[serial]
    fn test_unknown_key_is_exit_2_and_no_workspace_writes() {
        let tmp = TempDir::new().unwrap();
        let workspace_root = tmp.path().join("ws");
        fs::create_dir_all(&workspace_root).unwrap();
        let _cwd = CwdGuard::set(&workspace_root);

        let initial = b"# workspace patch\n";
        let workspace_yaml = write_workspace_yaml(&workspace_root, initial);
        let before = fs::read(&workspace_yaml).unwrap();

        let code = run_cli(&["substrate", "config", "workspace", "set", "nope.nope=1"]);
        assert_eq!(code, 2);

        let after = fs::read(&workspace_yaml).unwrap();
        assert_eq!(after, before);
    }
}
