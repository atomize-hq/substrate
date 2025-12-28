use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use substrate_common::WorldRootMode;

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq, Eq)]
#[value(rename_all = "kebab-case")]
pub enum AnchorModeArg {
    Workspace,
    FollowCwd,
    Custom,
}

impl From<AnchorModeArg> for WorldRootMode {
    fn from(value: AnchorModeArg) -> Self {
        match value {
            AnchorModeArg::Workspace => WorldRootMode::Project,
            AnchorModeArg::FollowCwd => WorldRootMode::FollowCwd,
            AnchorModeArg::Custom => WorldRootMode::Custom,
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "substrate")]
#[command(version, about = "Substrate shell wrapper with comprehensive tracing", long_about = None)]
pub struct Cli {
    /// Execute a single command
    #[arg(
        short = 'c',
        long = "command",
        value_name = "CMD",
        conflicts_with = "script"
    )]
    pub command: Option<String>,

    /// Execute a script file
    #[arg(
        short = 'f',
        long = "file",
        value_name = "SCRIPT",
        conflicts_with = "command"
    )]
    pub script: Option<PathBuf>,

    /// Enable CI mode with strict error handling
    #[arg(long = "ci")]
    pub ci_mode: bool,

    /// Continue execution after errors (overrides CI mode behavior)
    #[arg(long = "no-exit-on-error")]
    pub no_exit_on_error: bool,

    /// Use PTY for full terminal emulation in interactive mode (Unix only)
    #[cfg_attr(not(unix), arg(hide = true))]
    #[arg(long = "pty")]
    pub use_pty: bool,

    /// Specify shell to use (defaults to $SHELL or /bin/bash)
    #[arg(long = "shell", value_name = "PATH")]
    pub shell: Option<String>,

    /// Output version information as JSON
    #[arg(long = "version-json", conflicts_with_all = &["command", "script"])]
    pub version_json: bool,

    /// Show shim deployment status
    #[arg(long = "shim-status", conflicts_with_all = &["command", "script", "shim_deploy", "shim_remove"])]
    pub shim_status: bool,

    /// Show shim deployment status as JSON (CI-friendly)
    #[arg(long = "shim-status-json", conflicts_with_all = &["command", "script", "shim_deploy", "shim_remove"])]
    pub shim_status_json: bool,

    /// Skip shim deployment check
    #[arg(long = "shim-skip")]
    pub shim_skip: bool,

    /// Force deployment of command shims
    #[arg(long = "shim-deploy", conflicts_with_all = &["command", "script", "shim_remove", "shim_status"])]
    pub shim_deploy: bool,

    /// Remove all deployed shims
    #[arg(long = "shim-remove", conflicts_with_all = &["command", "script", "shim_deploy", "shim_status"])]
    pub shim_remove: bool,

    /// Force the async REPL loop (default)
    #[arg(long = "async-repl", conflicts_with_all = &["command", "script", "legacy_repl"])]
    pub async_repl: bool,

    /// Use the legacy synchronous REPL implementation
    #[arg(long = "legacy-repl", conflicts_with_all = &["command", "script", "async_repl"])]
    pub legacy_repl: bool,

    /// Show trace information for a span ID
    #[arg(long = "trace", value_name = "SPAN_ID", conflicts_with_all = &["command", "script", "shim_deploy", "shim_status", "shim_remove", "replay"])]
    pub trace: Option<String>,

    /// Replay a traced command by span ID (agent-first on Linux with a single agent-to-local fallback warning; use --no-world or SUBSTRATE_REPLAY_USE_WORLD=disabled for host-only)
    #[arg(long = "replay", value_name = "SPAN_ID", conflicts_with_all = &["command", "script", "shim_deploy", "shim_status", "shim_remove", "trace"])]
    pub replay: Option<String>,

    /// Verbose replay diagnostics (command/cwd/mode, world toggles, capability warnings, world strategy + scopes)
    #[arg(long = "replay-verbose", requires = "replay")]
    pub replay_verbose: bool,

    /// Flip the recorded execution origin (host/world) before applying other world toggles
    #[arg(long = "flip-world", visible_alias = "flip", requires = "replay")]
    pub flip_world: bool,

    /// Keep the shell anchored to the resolved root
    #[arg(long = "caged", action = ArgAction::SetTrue, conflicts_with = "uncaged")]
    pub caged: bool,

    /// Allow leaving the resolved root anchor
    #[arg(long = "uncaged", action = ArgAction::SetTrue, conflicts_with = "caged")]
    pub uncaged: bool,

    /// Control how the anchor root is selected (workspace, follow-cwd, or custom)
    #[arg(long = "anchor-mode", value_name = "MODE")]
    pub anchor_mode: Option<AnchorModeArg>,

    /// Explicit anchor path (used when --anchor-mode=custom)
    #[arg(long = "anchor-path", value_name = "PATH")]
    pub anchor_path: Option<PathBuf>,

    /// Force world isolation for this run (overrides disabled install/config/env)
    #[arg(
        long = "world",
        action = ArgAction::SetTrue,
        conflicts_with = "no_world",
        global = true
    )]
    pub world: bool,

    /// Disable world isolation (host pass-through)
    #[arg(
        long = "no-world",
        action = ArgAction::SetTrue,
        conflicts_with = "world",
        global = true
    )]
    pub no_world: bool,

    /// Graph commands (ingest/status/what-changed)
    #[command(subcommand)]
    pub sub: Option<SubCommands>,
}

#[derive(Subcommand, Debug)]
pub enum SubCommands {
    Graph(GraphCmd),
    World(WorldCmd),
    Config(ConfigCmd),
    Policy(PolicyCmd),
    Workspace(WorkspaceCmd),
    Shim(ShimCmd),
    Health(HealthCmd),
}

#[derive(Args, Debug)]
pub struct GraphCmd {
    #[command(subcommand)]
    pub action: GraphAction,
}

#[derive(Subcommand, Debug)]
pub enum GraphAction {
    Ingest {
        file: std::path::PathBuf,
    },
    Status,
    WhatChanged {
        span_id: String,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
}

#[derive(Args, Debug)]
pub struct WorldCmd {
    #[command(subcommand)]
    pub action: WorldAction,
}

#[derive(Args, Debug)]
pub struct ConfigCmd {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Args, Debug)]
pub struct PolicyCmd {
    #[command(subcommand)]
    pub action: PolicyAction,
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Print the effective config for the current workspace (YAML by default, JSON with --json)
    Show(ConfigShowArgs),
    /// Update the workspace config via dotted updates (key=value, key+=value, key-=value)
    Set(ConfigSetArgs),
    /// Global config commands ($SUBSTRATE_HOME/config.yaml)
    Global(ConfigGlobalCmd),
}

#[derive(Args, Debug)]
pub struct ConfigGlobalCmd {
    #[command(subcommand)]
    pub action: ConfigGlobalAction,
}

#[derive(Subcommand, Debug)]
pub enum ConfigGlobalAction {
    /// Initialize $SUBSTRATE_HOME/config.yaml (creates if missing; overwrites with --force)
    Init(ConfigInitArgs),
    /// Print $SUBSTRATE_HOME/config.yaml if present; otherwise print built-in defaults
    Show(ConfigShowArgs),
    /// Update $SUBSTRATE_HOME/config.yaml via dotted updates (key=value, key+=value, key-=value)
    Set(ConfigSetArgs),
}

#[derive(Args, Debug)]
pub struct ConfigInitArgs {
    /// Overwrite the config even if it already exists
    #[arg(long)]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct ConfigShowArgs {
    /// Emit JSON instead of YAML
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct ConfigSetArgs {
    /// Emit JSON summary instead of human output
    #[arg(long)]
    pub json: bool,
    /// One or more dotted updates (key=value, key+=value, key-=value)
    #[arg(value_name = "UPDATE", required = true)]
    pub updates: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum PolicyAction {
    /// Initialize <workspace_root>/.substrate/policy.yaml (creates if missing; overwrites with --force)
    Init(PolicyInitArgs),
    /// Print the effective policy for the current workspace (YAML by default, JSON with --json)
    Show(PolicyShowArgs),
    /// Update <workspace_root>/.substrate/policy.yaml via dotted updates (key=value, key+=value, key-=value)
    Set(PolicySetArgs),
    /// Global policy commands ($SUBSTRATE_HOME/policy.yaml)
    Global(PolicyGlobalCmd),
}

#[derive(Args, Debug)]
pub struct PolicyGlobalCmd {
    #[command(subcommand)]
    pub action: PolicyGlobalAction,
}

#[derive(Subcommand, Debug)]
pub enum PolicyGlobalAction {
    /// Initialize $SUBSTRATE_HOME/policy.yaml (creates if missing; overwrites with --force)
    Init(PolicyInitArgs),
    /// Print $SUBSTRATE_HOME/policy.yaml if present; otherwise print built-in defaults
    Show(PolicyShowArgs),
    /// Update $SUBSTRATE_HOME/policy.yaml via dotted updates (key=value, key+=value, key-=value)
    Set(PolicySetArgs),
}

#[derive(Args, Debug)]
pub struct PolicyInitArgs {
    /// Overwrite the policy even if it already exists
    #[arg(long)]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct PolicyShowArgs {
    /// Emit JSON instead of YAML
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct PolicySetArgs {
    /// Emit JSON instead of human output
    #[arg(long)]
    pub json: bool,
    /// One or more dotted updates (key=value, key+=value, key-=value)
    #[arg(value_name = "UPDATE", required = true)]
    pub updates: Vec<String>,
}

#[derive(Args, Debug)]
pub struct WorkspaceCmd {
    #[command(subcommand)]
    pub action: WorkspaceAction,
}

#[derive(Subcommand, Debug)]
pub enum WorkspaceAction {
    /// Initialize a workspace at PATH (defaults to .)
    Init(WorkspaceInitArgs),
}

#[derive(Args, Debug)]
pub struct WorkspaceInitArgs {
    /// Workspace root path (defaults to .)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
    /// Repair missing directories/files without overwriting existing files
    #[arg(long)]
    pub force: bool,
}

#[derive(Subcommand, Debug)]
pub enum WorldAction {
    Doctor {
        /// Output machine-readable JSON for CI
        #[arg(long)]
        json: bool,
    },
    Enable(WorldEnableArgs),
    Deps(WorldDepsCmd),
    Cleanup(WorldCleanupArgs),
    /// Run end-to-end world_fs enforcement verification (read_only + full cage).
    Verify(WorldVerifyArgs),
}

#[derive(Args, Debug, Clone)]
pub struct WorldEnableArgs {
    /// Installation prefix to upgrade (defaults to ~/.substrate or SUBSTRATE_HOME)
    #[arg(long = "prefix", value_name = "PATH")]
    pub prefix: Option<PathBuf>,
    /// Provisioning profile label passed to the helper script
    #[arg(long = "profile", value_name = "NAME", default_value = "release")]
    pub profile: String,
    /// Show provisioning actions without executing them
    #[arg(long = "dry-run")]
    pub dry_run: bool,
    /// Stream helper output to stdout/stderr in addition to the log file
    #[arg(long = "verbose")]
    pub verbose: bool,
    /// Re-run provisioning even if metadata reports the world is already enabled
    #[arg(long = "force")]
    pub force: bool,
    /// Seconds to wait for the world socket/doctor health checks
    #[arg(long = "timeout", value_name = "SECONDS", default_value_t = 120)]
    pub timeout: u64,
}

#[derive(Args, Debug)]
pub struct WorldCleanupArgs {
    /// Attempt to delete detected namespaces/nft tables/cgroups
    #[arg(long, help = "Apply cleanup actions instead of just reporting")]
    pub purge: bool,
}

#[derive(Args, Debug, Clone)]
pub struct WorldVerifyArgs {
    /// Emit a structured JSON report (stable fields for CI)
    #[arg(long)]
    pub json: bool,

    /// Treat skipped checks (unsupported/prereq-missing) as failures
    #[arg(long)]
    pub strict: bool,

    /// Directory to write logs/artifacts under (defaults to the OS temp directory)
    #[arg(long = "root", value_name = "PATH")]
    pub root: Option<PathBuf>,

    /// Keep temporary projects instead of deleting them at exit
    #[arg(long = "keep-temp")]
    pub keep_temp: bool,
}

#[derive(Args, Debug)]
pub struct WorldDepsCmd {
    #[command(subcommand)]
    pub action: WorldDepsAction,
}

#[derive(Subcommand, Debug)]
pub enum WorldDepsAction {
    Status(WorldDepsStatusArgs),
    Install(WorldDepsInstallArgs),
    Sync(WorldDepsSyncArgs),
}

#[derive(Args, Debug, Clone)]
pub struct WorldDepsStatusArgs {
    /// Specific tools to inspect (defaults to host-present tools; use --all to include host-missing)
    #[arg(value_name = "TOOL")]
    pub tools: Vec<String>,
    /// Include every manifest entry, even when missing on the host
    #[arg(long = "all")]
    pub all: bool,
    /// Emit JSON summary for automation
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct WorldDepsInstallArgs {
    /// Tool names to install inside the guest
    #[arg(value_name = "TOOL", required = true)]
    pub tools: Vec<String>,
    /// Show planned actions without executing them
    #[arg(long = "dry-run")]
    pub dry_run: bool,
    /// Stream guest logs while running installers
    #[arg(long = "verbose")]
    pub verbose: bool,
}

#[derive(Args, Debug, Clone)]
pub struct WorldDepsSyncArgs {
    /// Install every missing tool in the manifest (even if not detected on the host)
    #[arg(long = "all")]
    pub all: bool,
    /// Stream guest logs while running installers
    #[arg(long = "verbose")]
    pub verbose: bool,
}

#[derive(Args, Debug)]
pub struct ShimCmd {
    #[command(subcommand)]
    pub action: ShimAction,
}

#[derive(Args, Debug)]
pub struct HealthCmd {
    /// Output machine-readable JSON summary
    #[arg(long)]
    pub json: bool,
}

#[derive(Subcommand, Debug)]
pub enum ShimAction {
    Doctor {
        /// Output machine-readable JSON instead of the text report
        #[arg(long)]
        json: bool,
    },
    Repair {
        /// Manager name as defined in the manifest
        #[arg(long = "manager", value_name = "NAME")]
        manager: String,
        /// Apply the repair snippet without prompting
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },
}
