use super::guest::{
    detect_guest, detect_host, detect_host_bulk, macos_world_deps_unavailable_error,
    run_guest_install, world_exec_fallback_active, HostBulkDetection, WorldBackendUnavailable,
};
use super::models::{
    sanitize_reason, GuestProbe, ManifestLayerInfo, WorldDepGuestStatus, WorldDepStatusEntry,
    WorldDepsManifestInfo, WorldDepsOverlayInfo, WorldDepsStatusReport,
};
use super::state::WorldState;
use crate::{
    WorldDepsAction, WorldDepsCmd, WorldDepsInstallArgs, WorldDepsStatusArgs, WorldDepsSyncArgs,
};
use anyhow::{anyhow, bail, Context, Result};
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use substrate_common::{paths as substrate_paths, WorldDepTool, WorldDepsManifest};
use tracing::warn;

pub(crate) fn status_report_for_health(
    cli_no_world: bool,
    cli_force_world: bool,
    tools: &[String],
) -> Result<WorldDepsStatusReport> {
    let runner = build_runner(cli_no_world, cli_force_world)?;
    runner.status_report(tools, false)
}

pub fn run(cmd: &WorldDepsCmd, cli_no_world: bool, cli_force_world: bool) -> Result<()> {
    let runner = build_runner(cli_no_world, cli_force_world)?;
    match &cmd.action {
        WorldDepsAction::Status(args) => runner.run_status(args),
        WorldDepsAction::Install(args) => runner.run_install(args),
        WorldDepsAction::Sync(args) => runner.run_sync(args),
    }
}

struct ManifestPaths {
    inventory_base: PathBuf,
    inventory_overlay: Option<PathBuf>,
    installed_overlay: PathBuf,
    user_overlay: Option<PathBuf>,
}

impl ManifestPaths {
    fn resolve() -> Result<Self> {
        let substrate_home = substrate_paths::substrate_home()?;
        let inventory_base = crate::execution::manager_manifest_base_path();
        let inventory_overlay = Some(substrate_home.join("manager_hooks.local.yaml"));

        let installed_overlay = crate::execution::world_deps_manifest_base_path();
        let installed_overlay_required = env::var_os("SUBSTRATE_WORLD_DEPS_MANIFEST").is_some();
        if installed_overlay_required && !installed_overlay.exists() {
            bail!(
                "SUBSTRATE_WORLD_DEPS_MANIFEST points to a missing manifest: {}",
                installed_overlay.display()
            );
        }

        let user_overlay = Some(substrate_home.join("world-deps.local.yaml"));

        Ok(Self {
            inventory_base,
            inventory_overlay,
            installed_overlay,
            user_overlay,
        })
    }

    fn inventory_overlay_exists(&self) -> bool {
        self.inventory_overlay
            .as_ref()
            .map(|path| path.exists())
            .unwrap_or(false)
    }

    fn installed_overlay_exists(&self) -> bool {
        self.installed_overlay.exists()
    }

    fn user_overlay_exists(&self) -> bool {
        self.user_overlay
            .as_ref()
            .map(|path| path.exists())
            .unwrap_or(false)
    }

    fn overlays_for_loading(&self) -> Vec<PathBuf> {
        let mut overlays = Vec::new();
        if let Some(path) = &self.inventory_overlay {
            overlays.push(path.clone());
        }
        overlays.push(self.installed_overlay.clone());
        if let Some(path) = &self.user_overlay {
            overlays.push(path.clone());
        }
        overlays
    }
}

fn prepare_manager_env() {
    let substrate_home = match substrate_paths::substrate_home() {
        Ok(home) => home,
        Err(err) => {
            warn!(
                target = "substrate::shell",
                error = %err,
                "failed to resolve Substrate home for manager init"
            );
            return;
        }
    };

    let manager_init_path = substrate_home.join("manager_init.sh");
    let manager_env_path = substrate_home.join("manager_env.sh");
    let overlay = manager_init_path
        .parent()
        .map(|dir| dir.join("manager_hooks.local.yaml"));
    let manifest_paths = crate::execution::manager_init::ManifestPaths {
        base: crate::execution::manager_manifest_base_path(),
        overlay,
    };
    let init_cfg = crate::execution::manager_init::ManagerInitConfig::from_env(
        crate::execution::current_platform(),
    );

    match crate::execution::manager_init::detect_and_generate(manifest_paths, init_cfg) {
        Ok(result) => {
            if let Err(err) =
                crate::execution::manager_init::write_snippet(&manager_init_path, &result.snippet)
            {
                warn!(
                    target = "substrate::shell",
                    error = %err,
                    "failed to write manager init snippet for world deps"
                );
            } else {
                env::set_var("SUBSTRATE_MANAGER_INIT", &manager_init_path);
            }
        }
        Err(err) => {
            warn!(
                target = "substrate::shell",
                error = %err,
                "manager init generation failed for world deps"
            );
            let placeholder = format!(
                "# Generated by Substrate - do not edit\n# manager init unavailable: {}\n",
                err
            );
            if let Err(write_err) =
                crate::execution::manager_init::write_snippet(&manager_init_path, &placeholder)
            {
                warn!(
                    target = "substrate::shell",
                    error = %write_err,
                    "failed to write placeholder manager init snippet for world deps"
                );
            } else {
                env::set_var("SUBSTRATE_MANAGER_INIT", &manager_init_path);
            }
        }
    }

    if let Err(err) = crate::execution::write_manager_env_script_at(&manager_env_path) {
        warn!(
            target = "substrate::shell",
            error = %err,
            "failed to write manager env script for world deps"
        );
    }
}

fn build_runner(cli_no_world: bool, cli_force_world: bool) -> Result<WorldDepsRunner> {
    prepare_manager_env();
    let manifest_paths = ManifestPaths::resolve()?;
    let overlays = manifest_paths.overlays_for_loading();
    let manifest = WorldDepsManifest::load_layered(
        crate::execution::current_platform(),
        &manifest_paths.inventory_base,
        &overlays,
    )
    .with_context(|| {
        format!(
            "failed to load world deps inventory from {}",
            manifest_paths.inventory_base.display()
        )
    })?;
    let world = WorldState::detect(cli_no_world, cli_force_world)?;
    Ok(WorldDepsRunner::new(manifest, manifest_paths, world))
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

    fn manifest_info(&self) -> WorldDepsManifestInfo {
        let layers = {
            let mut layers = Vec::new();
            layers.push(self.paths.inventory_base.clone());
            if let Some(path) = &self.paths.inventory_overlay {
                layers.push(path.clone());
            }
            layers.push(self.paths.installed_overlay.clone());
            if let Some(path) = &self.paths.user_overlay {
                layers.push(path.clone());
            }
            layers
        };

        WorldDepsManifestInfo {
            inventory: ManifestLayerInfo {
                base: self.paths.inventory_base.clone(),
                overlay: self.paths.inventory_overlay.clone(),
                overlay_exists: self.paths.inventory_overlay_exists(),
            },
            overlays: WorldDepsOverlayInfo {
                installed: self.paths.installed_overlay.clone(),
                installed_exists: self.paths.installed_overlay_exists(),
                user: self.paths.user_overlay.clone(),
                user_exists: self.paths.user_overlay_exists(),
            },
            layers,
        }
    }

    fn run_status(&self, args: &WorldDepsStatusArgs) -> Result<()> {
        let report = self.status_report(&args.tools, args.all)?;
        if args.json {
            println!("{}", serde_json::to_string_pretty(&report)?);
            return Ok(());
        }

        if self.world.cli_disabled() {
            if let Some(message) = &report.world_disabled_reason {
                println!("substrate: warn: world deps status skipped ({message})");
            } else {
                println!("substrate: warn: world deps status skipped");
            }
            return Ok(());
        }

        if self.manifest.tools.is_empty() {
            println!(
                "No tools defined in inventory {}",
                self.paths.inventory_base.display()
            );
            return Ok(());
        }

        println!("Inventory: {}", report.manifest.inventory.base.display());
        if let Some(overlay) = &report.manifest.inventory.overlay {
            let status = if report.manifest.inventory.overlay_exists {
                "present"
            } else {
                "missing"
            };
            println!("Inventory overlay: {} ({status})", overlay.display());
        }
        let installed_status = if report.manifest.overlays.installed_exists {
            "present"
        } else {
            "missing"
        };
        println!(
            "Installed overlay: {} ({installed_status})",
            report.manifest.overlays.installed.display()
        );
        if let Some(user) = &report.manifest.overlays.user {
            let status = if report.manifest.overlays.user_exists {
                "present"
            } else {
                "missing"
            };
            println!("User overlay: {} ({status})", user.display());
        }

        if let Some(message) = &report.world_disabled_reason {
            println!("substrate: warn: world backend disabled ({message})");
        }

        if report.tools.is_empty() && args.tools.is_empty() && !args.all {
            let host_probe = self
                .manifest
                .tools
                .first()
                .map(|tool| detect_host(&tool.host.commands));
            if let Some(reason) = host_probe.and_then(|probe| probe.reason) {
                println!(
                    "No host-present tools detected (host detection skipped or degraded: {}).",
                    sanitize_reason(&reason)
                );
            } else {
                println!("No host-present tools detected.");
            }
            println!("Re-run `substrate world deps status --all` to include host-missing tools.");
            return Ok(());
        }

        print_status_table(&report.tools);
        Ok(())
    }

    fn status_report(
        &self,
        tool_names: &[String],
        include_all: bool,
    ) -> Result<WorldDepsStatusReport> {
        if self.world.cli_disabled() {
            return Ok(WorldDepsStatusReport {
                manifest: self.manifest_info(),
                world_disabled_reason: self.world.reason(),
                tools: Vec::new(),
            });
        }

        let tools = self.select_tools(tool_names)?;
        if tools.is_empty() {
            return Ok(WorldDepsStatusReport {
                manifest: self.manifest_info(),
                world_disabled_reason: self.world.reason(),
                tools: Vec::new(),
            });
        }

        let mut entries = Vec::with_capacity(tools.len());
        let filter_host_present = tool_names.is_empty() && !include_all;
        let host_bulk: Option<HostBulkDetection> = filter_host_present.then(|| {
            detect_host_bulk(
                &tools
                    .iter()
                    .map(|tool| tool.host.commands.clone())
                    .collect::<Vec<_>>(),
            )
        });
        for (idx, tool) in tools.into_iter().enumerate() {
            let (host_detected, host_reason) = if let Some(summary) = host_bulk.as_ref() {
                let detected = summary.detected.get(idx).copied().unwrap_or(false);
                let reason = (!detected)
                    .then(|| summary.degraded_reason.as_deref().map(sanitize_reason))
                    .flatten();
                (detected, reason)
            } else {
                let host_probe = detect_host(&tool.host.commands);
                (
                    host_probe.detected,
                    host_probe.reason.map(|reason| sanitize_reason(&reason)),
                )
            };
            if filter_host_present && !host_detected {
                continue;
            }
            let guest_status = self.probe_guest(tool);
            let provider = tool.install.first().map(|recipe| recipe.provider.clone());
            entries.push(WorldDepStatusEntry {
                name: tool.name.clone(),
                host_detected,
                host_reason,
                provider,
                guest: WorldDepGuestStatus::from_probe(guest_status),
            });
        }

        Ok(WorldDepsStatusReport {
            manifest: self.manifest_info(),
            world_disabled_reason: self.world.reason(),
            tools: entries,
        })
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
                "No tools defined in inventory {}; nothing to sync.",
                self.paths.inventory_base.display()
            );
            return Ok(());
        }

        let mut to_install = Vec::new();
        let mut host_detection_reason: Option<String> = None;
        let mut missing_in_guest = false;
        let mut skipped_on_host = Vec::new();
        for tool in tools {
            let host_probe = detect_host(&tool.host.commands);
            if host_detection_reason.is_none() {
                host_detection_reason = host_probe.reason.clone();
            }
            let host_detected = host_probe.detected;
            let guest_status = self.ensure_guest_state(tool)?;
            if guest_status {
                continue;
            }
            missing_in_guest = true;
            if args.all || host_detected {
                to_install.push(tool);
            } else {
                skipped_on_host.push(tool.name.clone());
            }
        }

        let host_detection_reason = host_detection_reason.map(|reason| sanitize_reason(&reason));

        if to_install.is_empty() {
            if missing_in_guest && !args.all {
                if let Some(reason) = host_detection_reason.as_deref() {
                    println!(
                        "No tools were synced because host detection was skipped or degraded ({}).",
                        reason
                    );
                    return Ok(());
                }
                if !skipped_on_host.is_empty() {
                    println!(
                        "No tools were synced because these tools were not detected on the host: {}.",
                        skipped_on_host.join(", ")
                    );
                    println!(
                        "Install them on the host first, or rerun `substrate world deps sync --all` to force guest installs."
                    );
                    return Ok(());
                }
            }
            println!("All tracked tools are already available inside the guest.");
            return Ok(());
        }

        for tool in to_install {
            self.install_tool(tool, args.verbose, false)?;
        }

        Ok(())
    }

    fn install_tool(&self, tool: &WorldDepTool, verbose: bool, dry_run: bool) -> Result<()> {
        let host_probe = detect_host(&tool.host.commands);
        let host_detected = host_probe.detected;
        if !host_detected {
            let reason = host_probe
                .reason
                .as_ref()
                .map(|value| sanitize_reason(value));
            if let Some(reason) = reason {
                println!(
                    "substrate: warn: `{}` is not detected on the host ({}); continuing with guest install anyway.",
                    tool.name, reason
                );
            } else {
                println!(
                    "substrate: warn: `{}` is not detected on the host; continuing with guest install anyway.",
                    tool.name
                );
            }
        }

        if self.ensure_guest_state(tool)? {
            println!("{} already available inside the guest.", tool.name);
            return Ok(());
        }

        let recipe = tool.install.first().ok_or_else(|| {
            anyhow!(
                "tool `{}` has no install recipes in {}",
                tool.name,
                self.paths.inventory_base.display()
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
        } else if world_exec_fallback_active() {
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
            Err(err) => {
                if let Some(unavailable) = err.downcast_ref::<WorldBackendUnavailable>() {
                    GuestProbe::Unavailable(format!(
                        "backend unavailable: {}",
                        unavailable.reason()
                    ))
                } else {
                    GuestProbe::Skipped(format!("{:#}", err))
                }
            }
        }
    }

    fn ensure_guest_state(&self, tool: &WorldDepTool) -> Result<bool> {
        detect_guest(&tool.guest.commands)
            .map_err(map_guest_unavailable_error)
            .with_context(|| format!("failed to detect `{}` inside the world backend", tool.name))
    }
}

fn map_guest_unavailable_error(err: anyhow::Error) -> anyhow::Error {
    if cfg!(target_os = "macos") {
        if let Some(unavailable) = err.downcast_ref::<WorldBackendUnavailable>() {
            return macos_world_deps_unavailable_error(unavailable.reason());
        }
    }
    err
}

fn print_status_table(entries: &[WorldDepStatusEntry]) {
    for entry in entries {
        let provider = entry.provider.as_deref().unwrap_or("n/a");
        let host_label = if entry.host_detected {
            "present".to_string()
        } else if let Some(reason) = entry.host_reason.as_deref() {
            format!("missing ({reason})")
        } else {
            "missing".to_string()
        };
        println!(
            "- {}: host={} guest={} installer={}",
            entry.name,
            host_label,
            entry.guest.label(),
            provider
        );
    }
}
