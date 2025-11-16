use crate::{
    build_agent_client_and_request, commands::world_enable, stream_non_pty_via_agent,
    world_deps_manifest_base_path, WorldDepsAction, WorldDepsCmd, WorldDepsInstallArgs,
    WorldDepsStatusArgs, WorldDepsSyncArgs,
};
use anyhow::{anyhow, bail, Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::{
    collections::HashSet,
    env,
    error::Error as StdError,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
};
use substrate_common::{paths as substrate_paths, WorldDepTool, WorldDepsManifest};
use tokio::runtime::Runtime;

static WORLD_EXEC_FALLBACK: AtomicBool = AtomicBool::new(false);

pub fn run(cmd: &WorldDepsCmd, cli_no_world: bool) -> Result<()> {
    let manifest_paths = ManifestPaths::resolve()?;
    let manifest = WorldDepsManifest::load(&manifest_paths.base, manifest_paths.overlay.as_deref())
        .with_context(|| {
            format!(
                "failed to load world deps manifest from {}",
                manifest_paths.base.display()
            )
        })?;

    let world = WorldState::detect(cli_no_world)?;
    let runner = WorldDepsRunner::new(manifest, manifest_paths, world);

    match &cmd.action {
        WorldDepsAction::Status(args) => runner.run_status(args),
        WorldDepsAction::Install(args) => runner.run_install(args),
        WorldDepsAction::Sync(args) => runner.run_sync(args),
    }
}

struct ManifestPaths {
    base: PathBuf,
    overlay: Option<PathBuf>,
}

impl ManifestPaths {
    fn resolve() -> Result<Self> {
        let base = world_deps_manifest_base_path();
        let overlay = Some(substrate_paths::substrate_home()?.join("world-deps.local.yaml"));
        Ok(Self { base, overlay })
    }

    fn overlay_exists(&self) -> bool {
        self.overlay
            .as_ref()
            .map(|path| path.exists())
            .unwrap_or(false)
    }
}

struct WorldDepsRunner {
    manifest: WorldDepsManifest,
    paths: ManifestPaths,
    world: WorldState,
}

impl WorldDepsRunner {
    fn new(manifest: WorldDepsManifest, paths: ManifestPaths, world: WorldState) -> Self {
        Self {
            manifest,
            paths,
            world,
        }
    }

    fn run_status(&self, args: &WorldDepsStatusArgs) -> Result<()> {
        let tools = self.select_tools(&args.tools)?;
        if tools.is_empty() {
            println!("No tools defined in manifest {}", self.paths.base.display());
            return Ok(());
        }

        println!("Manifest: {}", self.paths.base.display());
        if self.paths.overlay_exists() {
            if let Some(overlay) = &self.paths.overlay {
                println!("Overlay: {}", overlay.display());
            }
        }

        let world_reason = self.world.reason();
        if let Some(message) = &world_reason {
            println!("substrate: warn: world backend disabled ({message})");
        }

        let mut rows = Vec::new();
        for tool in tools {
            let host_detected = detect_host(&tool.host.commands);
            let guest_status = self.probe_guest(tool);

            rows.push(StatusRow {
                tool,
                host: host_detected,
                guest: guest_status,
                provider: tool.install.first().map(|recipe| recipe.provider.as_str()),
            });
        }

        print_status_table(&rows);
        Ok(())
    }

    fn run_install(&self, args: &WorldDepsInstallArgs) -> Result<()> {
        self.world.ensure_enabled()?;
        let tools = self.select_tools(&args.tools)?;
        if tools.is_empty() {
            bail!("no matching tools were found in the manifest");
        }

        for tool in tools {
            self.install_tool(tool, args.verbose, args.dry_run)?;
        }
        Ok(())
    }

    fn run_sync(&self, args: &WorldDepsSyncArgs) -> Result<()> {
        self.world.ensure_enabled()?;
        let tools = self.manifest.tools.iter().collect::<Vec<_>>();
        if tools.is_empty() {
            println!(
                "No tools defined in manifest {}; nothing to sync.",
                self.paths.base.display()
            );
            return Ok(());
        }

        let mut to_install = Vec::new();
        for tool in tools {
            let host_detected = detect_host(&tool.host.commands);
            let guest_status = self.ensure_guest_state(tool)?;
            if guest_status {
                continue;
            }
            if args.all || host_detected {
                to_install.push(tool);
            }
        }

        if to_install.is_empty() {
            println!("All tracked tools are already available inside the guest.");
            return Ok(());
        }

        for tool in to_install {
            self.install_tool(tool, args.verbose, false)?;
        }

        Ok(())
    }

    fn install_tool(&self, tool: &WorldDepTool, verbose: bool, dry_run: bool) -> Result<()> {
        let host_detected = detect_host(&tool.host.commands);
        if !host_detected {
            println!(
                "substrate: warn: `{}` is not detected on the host; continuing with guest install anyway.",
                tool.name
            );
        }

        if self.ensure_guest_state(tool)? {
            println!("{} already available inside the guest.", tool.name);
            return Ok(());
        }

        let recipe = tool.install.first().ok_or_else(|| {
            anyhow!(
                "tool `{}` has no install recipes in {}",
                tool.name,
                self.paths.base.display()
            )
        })?;

        if dry_run {
            println!(
                "[dry-run] `{}` would be installed via {} recipe.",
                tool.name, recipe.provider
            );
            return Ok(());
        }

        println!(
            "Installing `{}` via {} recipe...",
            tool.name, recipe.provider
        );
        run_guest_install(&recipe.script, verbose)?;

        let guest_ready = self.ensure_guest_state(tool)?;
        if guest_ready {
            println!("\u{2713} `{}` installed successfully.", tool.name);
            Ok(())
        } else if WORLD_EXEC_FALLBACK.load(Ordering::SeqCst) {
            println!(
                "substrate: warn: `{}` install finished but guest detection still reports missing. \
                 Verify the overlay recipe or rerun `substrate world deps status` after syncing markers.",
                tool.name
            );
            Ok(())
        } else {
            bail!(
                "`{}` installation finished but the guest command is still unavailable",
                tool.name
            );
        }
    }

    fn select_tools<'a>(&'a self, names: &[String]) -> Result<Vec<&'a WorldDepTool>> {
        if names.is_empty() {
            return Ok(self.manifest.tools.iter().collect());
        }

        let mut seen = HashSet::new();
        let mut tools = Vec::new();
        for name in names {
            if !seen.insert(name.to_ascii_lowercase()) {
                continue;
            }
            let tool = self
                .manifest
                .tool(name)
                .ok_or_else(|| anyhow!("tool `{}` is not defined in the manifest", name))?;
            tools.push(tool);
        }
        Ok(tools)
    }

    fn probe_guest(&self, tool: &WorldDepTool) -> GuestProbe {
        if let Some(reason) = self.world.reason() {
            return GuestProbe::Unavailable(reason);
        }

        match detect_guest(&tool.guest.commands) {
            Ok(result) => GuestProbe::Available(result),
            Err(err) => GuestProbe::Skipped(format!("{:#}", err)),
        }
    }

    fn ensure_guest_state(&self, tool: &WorldDepTool) -> Result<bool> {
        detect_guest(&tool.guest.commands)
            .with_context(|| format!("failed to detect `{}` inside the world backend", tool.name))
    }
}

struct StatusRow<'a> {
    tool: &'a WorldDepTool,
    host: bool,
    guest: GuestProbe,
    provider: Option<&'a str>,
}

enum GuestProbe {
    Available(bool),
    Skipped(String),
    Unavailable(String),
}

impl GuestProbe {
    fn label(&self) -> String {
        match self {
            GuestProbe::Available(true) => "present".to_string(),
            GuestProbe::Available(false) => "missing".to_string(),
            GuestProbe::Skipped(reason) => format!("n/a ({})", sanitize_reason(reason)),
            GuestProbe::Unavailable(reason) => {
                format!("missing ({})", sanitize_reason(reason))
            }
        }
    }
}

fn sanitize_reason(reason: &str) -> String {
    reason
        .replace('\n', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn print_status_table(rows: &[StatusRow<'_>]) {
    for row in rows {
        let provider = row.provider.unwrap_or("n/a");
        println!(
            "- {}: host={} guest={} installer={}",
            row.tool.name,
            if row.host { "present" } else { "missing" },
            row.guest.label(),
            provider
        );
    }
}

struct WorldState {
    cli_disabled: bool,
    env_disabled: bool,
    config_disabled: bool,
}

impl WorldState {
    fn detect(cli_no_world: bool) -> Result<Self> {
        let env_disabled = env::var("SUBSTRATE_WORLD")
            .map(|value| value.eq_ignore_ascii_case("disabled"))
            .unwrap_or(false)
            || env::var("SUBSTRATE_WORLD_ENABLED")
                .map(|value| value == "0")
                .unwrap_or(false);

        let install_config = world_enable::load_install_config(&substrate_paths::config_file()?)?;
        Ok(Self {
            cli_disabled: cli_no_world,
            env_disabled,
            config_disabled: !install_config.world_enabled,
        })
    }

    fn is_disabled(&self) -> bool {
        self.cli_disabled || self.env_disabled || self.config_disabled
    }

    fn ensure_enabled(&self) -> Result<()> {
        if self.is_disabled() {
            let reason = self
                .reason()
                .unwrap_or_else(|| "unknown reason".to_string());
            bail!(
                "world backend disabled ({}). Re-run `substrate world enable` or drop --no-world.",
                reason
            );
        }
        Ok(())
    }

    fn reason(&self) -> Option<String> {
        if self.cli_disabled {
            Some("--no-world flag is active".to_string())
        } else if self.env_disabled {
            Some("SUBSTRATE_WORLD=disabled".to_string())
        } else if self.config_disabled {
            Some("install metadata reports world disabled".to_string())
        } else {
            None
        }
    }
}

fn detect_host(commands: &[String]) -> bool {
    for cmd in commands {
        if run_host_command(cmd) {
            return true;
        }
    }
    false
}

fn run_host_command(command: &str) -> bool {
    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.arg("/C").arg(command);
        c
    } else {
        let mut c = Command::new("sh");
        c.arg("-c").arg(command);
        c
    };
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    cmd.status().map(|status| status.success()).unwrap_or(false)
}

fn run_world_command(command: &str) -> Result<agent_api_types::ExecuteResponse> {
    let (client, request, _) = build_agent_client_and_request(command)?;
    let rt = Runtime::new()?;
    let response = rt.block_on(async move { client.execute(request).await })?;
    Ok(response)
}

fn run_host_install(script: &str, verbose: bool) -> Result<()> {
    println!("substrate: warn: world backend unavailable; running installer on the host.");
    let body = build_bash_body(script, true);
    let mut cmd = Command::new("bash");
    cmd.arg("-lc").arg(&body);
    if verbose {
        let status = cmd.status()?;
        if status.success() {
            Ok(())
        } else {
            bail!(
                "installer exited with status {}",
                status.code().unwrap_or(-1)
            );
        }
    } else {
        let output = cmd.output()?;
        if output.status.success() {
            Ok(())
        } else {
            eprintln!("{}", String::from_utf8_lossy(&output.stdout));
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            bail!(
                "installer exited with status {}",
                output.status.code().unwrap_or(-1)
            );
        }
    }
}

fn detect_guest(commands: &[String]) -> Result<bool> {
    for cmd in commands {
        if WORLD_EXEC_FALLBACK.load(Ordering::SeqCst) {
            if run_host_command(cmd) {
                return Ok(true);
            }
            continue;
        }

        let wrapped = wrap_for_bash(cmd, false);
        match run_world_command(&wrapped) {
            Ok(response) => {
                if response.exit == 0 {
                    return Ok(true);
                }
            }
            Err(err) if should_fallback_to_host(&err) => {
                mark_world_exec_unavailable(&err);
                if run_host_command(cmd) {
                    return Ok(true);
                }
            }
            Err(err) => return Err(err),
        }
    }
    Ok(false)
}

fn run_guest_install(script: &str, verbose: bool) -> Result<()> {
    if WORLD_EXEC_FALLBACK.load(Ordering::SeqCst) {
        return run_host_install(script, verbose);
    }

    let command = wrap_for_bash(script, true);
    if verbose {
        match stream_non_pty_via_agent(&command) {
            Ok(outcome) => {
                if outcome.exit_code == 0 {
                    Ok(())
                } else {
                    bail!("installer exited with status {}", outcome.exit_code);
                }
            }
            Err(err) if should_fallback_to_host(&err) => {
                mark_world_exec_unavailable(&err);
                run_host_install(script, verbose)
            }
            Err(err) => Err(err),
        }
    } else {
        match run_world_command(&command) {
            Ok(response) => {
                if response.exit == 0 {
                    Ok(())
                } else {
                    let stdout = BASE64
                        .decode(response.stdout_b64.as_bytes())
                        .unwrap_or_default();
                    let stderr = BASE64
                        .decode(response.stderr_b64.as_bytes())
                        .unwrap_or_default();
                    eprintln!("{}", String::from_utf8_lossy(&stdout));
                    eprintln!("{}", String::from_utf8_lossy(&stderr));
                    bail!("installer exited with status {}", response.exit);
                }
            }
            Err(err) if should_fallback_to_host(&err) => {
                mark_world_exec_unavailable(&err);
                run_host_install(script, verbose)
            }
            Err(err) => Err(err),
        }
    }
}

fn wrap_for_bash(script: &str, strict: bool) -> String {
    let body = build_bash_body(script, strict);
    let escaped = body.replace('\'', "'\"'\"'");
    format!("bash -lc '{}'", escaped)
}

fn build_bash_body(script: &str, strict: bool) -> String {
    let mut body = String::new();
    if strict {
        body.push_str("set -euo pipefail; ");
    }
    body.push_str(script);
    body
}

fn should_fallback_to_host(err: &anyhow::Error) -> bool {
    if WORLD_EXEC_FALLBACK.load(Ordering::SeqCst) {
        return true;
    }
    let mut current: Option<&(dyn StdError + 'static)> = Some(err.as_ref());
    while let Some(err) = current {
        let message = err.to_string();
        if message.contains("world-agent")
            || message.contains("platform world context")
            || message.contains("world backend")
        {
            return true;
        }
        current = err.source();
    }
    false
}

fn mark_world_exec_unavailable(err: &anyhow::Error) {
    let previously = WORLD_EXEC_FALLBACK.swap(true, Ordering::SeqCst);
    if !previously {
        println!(
            "substrate: warn: world backend unavailable for world deps (falling back to host execution): {:#}",
            err
        );
    }
}
