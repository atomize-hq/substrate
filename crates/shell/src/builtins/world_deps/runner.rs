use super::guest::{
    detect_guest, detect_host, detect_host_bulk, run_guest_install, world_exec_fallback_active,
    WorldBackendUnavailable,
};
use super::models::{
    sanitize_reason, GuestProbe, ManifestLayerInfo, WorldDepGuestState, WorldDepGuestStatus,
    WorldDepStatusEntry, WorldDepsManifestInfo, WorldDepsOverlayInfo, WorldDepsSelectionInfo,
    WorldDepsSelectionScope, WorldDepsStatusReport,
};
use super::selection::{
    add_tools_to_selection_file, resolve_active_selection, resolve_selection_target,
    write_empty_selection_file, ActiveSelection, SelectionConfigError,
};
use super::state::WorldState;
use crate::{
    WorldDepsAction, WorldDepsCmd, WorldDepsInitArgs, WorldDepsInstallArgs, WorldDepsSelectArgs,
    WorldDepsStatusArgs, WorldDepsSyncArgs,
};
use anyhow::{anyhow, bail, Context, Result};
use std::collections::HashSet;
use std::env;
use std::path::{Component, Path, PathBuf};
use substrate_common::{paths as substrate_paths, InstallClass, WorldDepTool, WorldDepsManifest};
use tracing::warn;

pub(crate) fn status_report_for_health(
    cli_no_world: bool,
    cli_force_world: bool,
    tools: &[String],
) -> Result<WorldDepsStatusReport> {
    let runner = build_runner(cli_no_world, cli_force_world)?;
    runner.status_report(tools, false)
}

pub fn run(cmd: &WorldDepsCmd, cli_no_world: bool, cli_force_world: bool) -> i32 {
    let result = (|| -> Result<()> {
        let runner = build_runner(cli_no_world, cli_force_world)?;
        match &cmd.action {
            WorldDepsAction::Status(args) => runner.run_status(args),
            WorldDepsAction::Install(args) => runner.run_install(args),
            WorldDepsAction::Sync(args) => runner.run_sync(args),
            WorldDepsAction::Init(args) => runner.run_init(args),
            WorldDepsAction::Select(args) => runner.run_select(args),
        }
    })();

    match result {
        Ok(()) => 0,
        Err(err) => {
            let code = world_deps_exit_code(&err);
            if code == 5 {
                eprintln!(
                    "substrate: world deps blocked by hardening/cage: required writes to `/var/lib/substrate/world-deps` are not permitted.\nHint: ensure `/var/lib/substrate/world-deps` is bind-mounted read-write inside the world and retry (see `docs/project_management/_archived/p0-agent-hub-isolation-hardening/I2-spec.md` and `docs/project_management/_archived/p0-agent-hub-isolation-hardening/I3-spec.md`)."
                );
                eprintln!("Underlying error: {:#}", err);
            } else if code == 3 {
                if let Some(reason) = world_backend_unavailable_reason(&err) {
                    let header = if cfg!(target_os = "macos") {
                        "substrate: world backend unavailable for world deps on macOS; run `substrate world doctor --json` to inspect backend status, then retry."
                    } else {
                        "substrate: world backend unavailable for world deps; run `substrate world doctor --json` to inspect backend status, then retry."
                    };
                    eprintln!("{header}\nUnderlying error: {reason}");
                } else {
                    eprintln!("{:#}", err);
                }
            } else {
                eprintln!("{:#}", err);
            }
            code
        }
    }
}

fn world_backend_unavailable_reason(err: &anyhow::Error) -> Option<String> {
    err.chain()
        .find_map(|cause| cause.downcast_ref::<WorldBackendUnavailable>())
        .map(|e| e.reason().to_string())
}

fn world_deps_exit_code(err: &anyhow::Error) -> i32 {
    if err.is::<SelectionConfigError>() {
        return 2;
    }
    if err.is::<WorldDepsUnmetPrerequisiteError>() {
        return 4;
    }
    if err.is::<WorldDepsBackendRequiredError>() {
        return 3;
    }
    if err
        .chain()
        .any(|cause| cause.downcast_ref::<WorldBackendUnavailable>().is_some())
    {
        return 3;
    }
    if looks_like_world_deps_hardening_violation(err) {
        return 5;
    }
    1
}

fn looks_like_world_deps_hardening_violation(err: &anyhow::Error) -> bool {
    let mut current: Option<&(dyn std::error::Error + 'static)> = Some(err.as_ref());
    while let Some(e) = current {
        let msg = e.to_string();
        if msg.contains("/var/lib/substrate/world-deps")
            && (msg.contains("Permission denied")
                || msg.contains("permission denied")
                || msg.contains("Read-only file system")
                || msg.contains("read-only file system"))
        {
            return true;
        }
        current = e.source();
    }
    false
}

fn diff_paths(target: &Path, base: &Path) -> Option<PathBuf> {
    if target.is_absolute() != base.is_absolute() {
        return None;
    }

    let target_components: Vec<Component<'_>> = target.components().collect();
    let base_components: Vec<Component<'_>> = base.components().collect();

    if matches!(target_components.first(), Some(Component::Prefix(_)))
        && matches!(base_components.first(), Some(Component::Prefix(_)))
        && target_components.first() != base_components.first()
    {
        return None;
    }

    let mut common_len = 0usize;
    while common_len < target_components.len()
        && common_len < base_components.len()
        && target_components[common_len] == base_components[common_len]
    {
        common_len += 1;
    }

    let mut segments: Vec<String> = Vec::new();
    for component in &base_components[common_len..] {
        match component {
            Component::Normal(_) | Component::ParentDir => segments.push("..".to_string()),
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => {}
        }
    }

    for component in &target_components[common_len..] {
        match component {
            Component::Normal(seg) => segments.push(seg.to_string_lossy().to_string()),
            Component::ParentDir => segments.push("..".to_string()),
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => {}
        }
    }

    if segments.is_empty() {
        return Some(PathBuf::from("."));
    }

    // Use forward slashes even on Windows so JSON output is stable across platforms and matches
    // the planning-pack smoke expectations (e.g., `.substrate/world-deps.selection.yaml`).
    Some(PathBuf::from(segments.join("/")))
}

#[derive(Debug)]
struct WorldDepsBackendRequiredError {
    message: String,
}

impl std::fmt::Display for WorldDepsBackendRequiredError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for WorldDepsBackendRequiredError {}

#[derive(Debug)]
struct WorldDepsUnmetPrerequisiteError {
    message: String,
}

impl std::fmt::Display for WorldDepsUnmetPrerequisiteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for WorldDepsUnmetPrerequisiteError {}

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
    let manifest_paths = ManifestPaths::resolve()?;
    let overlays = manifest_paths.overlays_for_loading();
    let manifest = WorldDepsManifest::load_layered(
        crate::execution::current_platform(),
        &manifest_paths.inventory_base,
        &overlays,
    )
    .map_err(|err| {
        let context = format!(
            "failed to load world deps inventory from {}",
            manifest_paths.inventory_base.display()
        );
        anyhow!(SelectionConfigError {
            message: format!(
                "substrate: invalid world-deps manager manifest ({context}): {:#}",
                err
            ),
        })
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

    fn inventory_tool_names(&self) -> HashSet<String> {
        self.manifest
            .tools
            .iter()
            .map(|tool| tool.name.to_ascii_lowercase())
            .collect()
    }

    fn load_active_selection(&self) -> Result<ActiveSelection> {
        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let inventory = self.inventory_tool_names();
        resolve_active_selection(&cwd, &inventory)
    }

    fn ensure_manager_env_ready(&self) {
        if cfg!(windows) {
            return;
        }
        prepare_manager_env();
    }

    fn require_world_backend(&self) -> Result<()> {
        if self.world.is_disabled() {
            let reason = self
                .world
                .reason()
                .unwrap_or_else(|| "unknown reason".to_string());
            return Err(anyhow!(WorldDepsBackendRequiredError {
                message: format!(
                    "substrate: world backend unavailable for world deps ({reason})\nHint: run `substrate world doctor --json` and/or `substrate world enable`, then retry."
                ),
            }));
        }
        Ok(())
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

        if !report.selection.configured {
            print_selection_missing_guidance();
            return Ok(());
        }

        if self.manifest.tools.is_empty() {
            println!(
                "No tools defined in inventory {}",
                self.paths.inventory_base.display()
            );
            return Ok(());
        }

        if let Some(path) = &report.selection.active_path {
            let scope = match report.selection.active_scope {
                Some(WorldDepsSelectionScope::Workspace) => "workspace",
                Some(WorldDepsSelectionScope::Global) => "global",
                None => "unknown",
            };
            println!("Selection: {} ({scope})", path.display());
        }
        if report.selection.ignored_due_to_all {
            println!("Selection ignored due to --all");
        }

        if report.selection.selected.is_empty() && args.tools.is_empty() && !args.all {
            println!("Selection configured but empty; no tools selected.");
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

        for entry in &report.tools {
            if entry.install_class != "manual" {
                continue;
            }
            if entry.guest.status == WorldDepGuestState::Present {
                continue;
            }
            let Some(tool) = self.manifest.tool(&entry.name) else {
                continue;
            };
            let Some(instructions) = tool.manual_instructions.as_deref() else {
                continue;
            };
            println!(
                "\nManual install instructions for `{}`:\n{}",
                entry.name, instructions
            );
        }

        Ok(())
    }

    fn status_report(
        &self,
        tool_names: &[String],
        include_all: bool,
    ) -> Result<WorldDepsStatusReport> {
        let selection = self.load_active_selection()?;
        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let report_active_path = match (selection.active_scope, selection.active_path.as_ref()) {
            (Some(WorldDepsSelectionScope::Workspace), Some(path)) => {
                Some(diff_paths(path, &cwd).unwrap_or_else(|| path.clone()))
            }
            (_, Some(path)) => Some(path.clone()),
            (_, None) => None,
        };
        let selection_info = WorldDepsSelectionInfo {
            configured: selection.configured,
            active_path: report_active_path,
            active_scope: selection.active_scope,
            shadowed_paths: selection.shadowed_paths.clone(),
            selected: selection.selected.clone(),
            ignored_due_to_all: selection.configured && include_all,
        };

        if !selection.configured {
            return Ok(WorldDepsStatusReport {
                selection: selection_info,
                manifest: self.manifest_info(),
                world_disabled_reason: self.world.reason(),
                tools: Vec::new(),
            });
        }

        let selected_set: HashSet<String> = selection.selected.iter().cloned().collect();

        let tools = if tool_names.is_empty() {
            if include_all {
                self.manifest.tools.iter().collect::<Vec<_>>()
            } else {
                self.manifest
                    .tools
                    .iter()
                    .filter(|tool| selected_set.contains(&tool.name.to_ascii_lowercase()))
                    .collect::<Vec<_>>()
            }
        } else {
            self.select_tools(tool_names)?
        };

        if !tools.is_empty() {
            self.ensure_manager_env_ready();
        }

        let mut entries = Vec::with_capacity(tools.len());
        let host_bulk = detect_host_bulk(
            &tools
                .iter()
                .map(|tool| tool.host.commands.clone())
                .collect::<Vec<_>>(),
        );
        for (idx, tool) in tools.into_iter().enumerate() {
            let selected = selected_set.contains(&tool.name.to_ascii_lowercase());
            let detected = host_bulk.detected.get(idx).copied().unwrap_or(false);
            let host_reason = (!detected)
                .then(|| host_bulk.degraded_reason.as_deref().map(sanitize_reason))
                .flatten();

            let guest_status = if include_all || tool_names.is_empty() || selected {
                self.probe_guest_for_status(tool)
            } else {
                GuestProbe::Skipped("not selected".to_string())
            };
            let provider = tool.install.first().map(|recipe| recipe.provider.clone());
            let name = tool.name.to_ascii_lowercase();
            entries.push(WorldDepStatusEntry {
                name,
                selected,
                install_class: install_class_label(tool.install_class).to_string(),
                host_detected: detected,
                host_reason,
                provider,
                guest: WorldDepGuestStatus::from_probe(guest_status),
            });
        }

        Ok(WorldDepsStatusReport {
            selection: selection_info,
            manifest: self.manifest_info(),
            world_disabled_reason: self.world.reason(),
            tools: entries,
        })
    }

    fn run_install(&self, args: &WorldDepsInstallArgs) -> Result<()> {
        let selection = self.load_active_selection()?;
        if !selection.configured {
            print_selection_missing_guidance();
            return Ok(());
        }

        let selected_set: HashSet<String> = selection.selected.iter().cloned().collect();
        if !args.all {
            let mut not_selected = Vec::new();
            for name in &args.tools {
                let normalized = name.trim().to_ascii_lowercase();
                if normalized.is_empty() {
                    continue;
                }
                if !selected_set.contains(&normalized) {
                    not_selected.push(normalized);
                }
            }
            not_selected.sort();
            not_selected.dedup();
            if !not_selected.is_empty() {
                return Err(anyhow!(SelectionConfigError {
                    message: format!(
                        "substrate: tool not selected: {}\nHint: add it to the selection file (e.g. `substrate world deps select {}`) or pass --all to ignore selection.",
                        not_selected.join(", "),
                        not_selected.join(" ")
                    ),
                }));
            }
        } else {
            println!("Selection ignored due to --all");
        }

        self.ensure_manager_env_ready();
        self.require_world_backend()?;
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
        let selection = self.load_active_selection()?;
        if !selection.configured {
            print_selection_missing_guidance();
            return Ok(());
        }

        let selected_set: HashSet<String> = selection.selected.iter().cloned().collect();
        let tools = if args.all {
            println!("Selection ignored due to --all");
            self.manifest.tools.iter().collect::<Vec<_>>()
        } else {
            self.manifest
                .tools
                .iter()
                .filter(|tool| selected_set.contains(&tool.name.to_ascii_lowercase()))
                .collect::<Vec<_>>()
        };

        if tools.is_empty() {
            println!("No tools selected; nothing to do.");
            return Ok(());
        }

        self.ensure_manager_env_ready();
        self.require_world_backend()?;

        let mut to_install = Vec::new();
        let mut blocked = Vec::new();
        for tool in tools {
            if tool.install_class == InstallClass::CopyFromHost {
                blocked.push(format!(
                    "`{}`: blocked (install_class=copy_from_host)\n  copy_from_host is reserved and unsupported in this increment.",
                    tool.name
                ));
                continue;
            }

            let guest_status = self.ensure_guest_state(tool)?;
            if guest_status {
                continue;
            }

            match tool.install_class {
                InstallClass::UserSpace => to_install.push(tool),
                InstallClass::SystemPackages => blocked.push(format!(
                    "`{}`: blocked (install_class=system_packages)\n  Requires OS packages. Run:\n    substrate world deps provision",
                    tool.name
                )),
                InstallClass::Manual => {
                    let mut message = format!(
                        "`{}`: blocked (install_class=manual)\n  Manual install required.",
                        tool.name
                    );
                    if let Some(instructions) = tool.manual_instructions.as_deref() {
                        message.push_str("\n\n");
                        message.push_str(instructions);
                    }
                    blocked.push(message);
                }
                InstallClass::CopyFromHost => {}
            }
        }

        if to_install.is_empty() && blocked.is_empty() {
            println!("All scoped tools are already available inside the guest.");
            return Ok(());
        }

        for tool in to_install {
            self.install_tool(tool, args.verbose, args.dry_run)?;
        }

        if blocked.is_empty() {
            return Ok(());
        }

        Err(anyhow!(WorldDepsUnmetPrerequisiteError {
            message: blocked.join("\n"),
        }))
    }

    fn run_init(&self, args: &WorldDepsInitArgs) -> Result<()> {
        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let (scope, path) = resolve_selection_target(&cwd, args.workspace, args.global)?;
        write_empty_selection_file(&path, args.force)?;
        match scope {
            WorldDepsSelectionScope::Workspace => {
                println!(
                    "substrate: initialized world-deps selection at {} (workspace)",
                    path.display()
                );
            }
            WorldDepsSelectionScope::Global => {
                println!(
                    "substrate: initialized world-deps selection at {} (global)",
                    path.display()
                );
            }
        }
        Ok(())
    }

    fn run_select(&self, args: &WorldDepsSelectArgs) -> Result<()> {
        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let (scope, path) = resolve_selection_target(&cwd, args.workspace, args.global)?;
        let inventory = self.inventory_tool_names();
        let added = add_tools_to_selection_file(&path, &args.tools, &inventory)?;
        let scope_label = match scope {
            WorldDepsSelectionScope::Workspace => "workspace",
            WorldDepsSelectionScope::Global => "global",
        };
        if added.is_empty() {
            println!(
                "substrate: world-deps selection unchanged at {} ({})",
                path.display(),
                scope_label
            );
        } else {
            println!(
                "substrate: updated world-deps selection at {} ({})\nAdded: {}",
                path.display(),
                scope_label,
                added.join(", ")
            );
        }
        Ok(())
    }

    fn install_tool(&self, tool: &WorldDepTool, verbose: bool, dry_run: bool) -> Result<()> {
        if tool.install_class == InstallClass::CopyFromHost {
            return Err(anyhow!(WorldDepsUnmetPrerequisiteError {
                message: format!(
                    "`{}`: blocked (install_class=copy_from_host)\n  copy_from_host is reserved and unsupported in this increment.",
                    tool.name
                ),
            }));
        }

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

        match tool.install_class {
            InstallClass::UserSpace => {}
            InstallClass::SystemPackages => {
                return Err(anyhow!(WorldDepsUnmetPrerequisiteError {
                    message: format!(
                        "`{}`: blocked (install_class=system_packages)\n  Requires OS packages. Run:\n    substrate world deps provision",
                        tool.name
                    ),
                }));
            }
            InstallClass::Manual => {
                let mut message = format!(
                    "`{}`: blocked (install_class=manual)\n  Manual install required.",
                    tool.name
                );
                if let Some(instructions) = tool.manual_instructions.as_deref() {
                    message.push_str("\n\n");
                    message.push_str(instructions);
                }
                return Err(anyhow!(WorldDepsUnmetPrerequisiteError { message }));
            }
            InstallClass::CopyFromHost => {
                return Err(anyhow!(WorldDepsUnmetPrerequisiteError {
                    message: format!(
                        "`{}`: blocked (install_class=copy_from_host)\n  copy_from_host is reserved and unsupported in this increment.",
                        tool.name
                    ),
                }));
            }
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
                .ok_or_else(|| {
                    anyhow!(SelectionConfigError {
                        message: format!(
                            "substrate: unknown tool `{}` (not in inventory)\nHint: run `substrate world deps status --all` after initializing selection to discover available tool names.",
                            name
                        ),
                    })
                })?;
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

    fn probe_guest_for_status(&self, tool: &WorldDepTool) -> GuestProbe {
        let probe = self.probe_guest(tool);
        match probe {
            GuestProbe::Available(true) => probe,
            GuestProbe::Available(false) => match tool.install_class {
                InstallClass::UserSpace => probe,
                InstallClass::SystemPackages => GuestProbe::Skipped(
                    "requires system packages; run `substrate world deps provision`".to_string(),
                ),
                InstallClass::Manual => GuestProbe::Skipped("manual install required".to_string()),
                InstallClass::CopyFromHost => {
                    GuestProbe::Skipped("unsupported in this increment".to_string())
                }
            },
            other => other,
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
        let host_label = if entry.host_detected {
            "present".to_string()
        } else if let Some(reason) = entry.host_reason.as_deref() {
            format!("missing ({reason})")
        } else {
            "missing".to_string()
        };
        println!(
            "- {}: selected={} class={} host={} guest={} installer={}",
            entry.name,
            entry.selected,
            entry.install_class,
            host_label,
            entry.guest.label(),
            provider
        );
    }
}

fn install_class_label(class: InstallClass) -> &'static str {
    match class {
        InstallClass::UserSpace => "user_space",
        InstallClass::SystemPackages => "system_packages",
        InstallClass::Manual => "manual",
        InstallClass::CopyFromHost => "copy_from_host",
    }
}

fn print_selection_missing_guidance() {
    println!("substrate: world deps not configured (selection file missing)");
    println!("Next steps:");
    println!("  - Create a selection file: substrate world deps init --workspace");
    println!("  - Discover available tools: substrate world deps status --all");
}
