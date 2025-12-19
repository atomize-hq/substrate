use super::guest::{detect_guest, detect_host, run_guest_install, world_exec_fallback_active};
use super::models::{
    GuestProbe, ManifestLayerInfo, WorldDepGuestStatus, WorldDepStatusEntry, WorldDepsManifestInfo,
    WorldDepsOverlayInfo, WorldDepsStatusReport,
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

pub(crate) fn status_report_for_health(
    cli_no_world: bool,
    cli_force_world: bool,
    tools: &[String],
) -> Result<WorldDepsStatusReport> {
    let runner = build_runner(cli_no_world, cli_force_world)?;
    runner.status_report(tools)
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

fn build_runner(cli_no_world: bool, cli_force_world: bool) -> Result<WorldDepsRunner> {
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
        let report = self.status_report(&args.tools)?;
        if args.json {
            println!("{}", serde_json::to_string_pretty(&report)?);
            return Ok(());
        }

        if report.tools.is_empty() {
            println!(
                "No tools defined in inventory {}",
                report.manifest.inventory.base.display()
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

        print_status_table(&report.tools);
        Ok(())
    }

    fn status_report(&self, tool_names: &[String]) -> Result<WorldDepsStatusReport> {
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
        for tool in tools {
            let host_detected = detect_host(&tool.host.commands);
            let guest_status = self.probe_guest(tool);
            let provider = tool.install.first().map(|recipe| recipe.provider.clone());
            entries.push(WorldDepStatusEntry {
                name: tool.name.clone(),
                host_detected,
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
            Err(err) => GuestProbe::Skipped(format!("{:#}", err)),
        }
    }

    fn ensure_guest_state(&self, tool: &WorldDepTool) -> Result<bool> {
        detect_guest(&tool.guest.commands)
            .with_context(|| format!("failed to detect `{}` inside the world backend", tool.name))
    }
}

fn print_status_table(entries: &[WorldDepStatusEntry]) {
    for entry in entries {
        let provider = entry.provider.as_deref().unwrap_or("n/a");
        println!(
            "- {}: host={} guest={} installer={}",
            entry.name,
            if entry.host_detected {
                "present"
            } else {
                "missing"
            },
            entry.guest.label(),
            provider
        );
    }
}
