use std::path::Path;

use anyhow::{anyhow, Context, Result};

use crate::manager_manifest::{InstallSpec, ManagerManifest, ManagerSpec};

/// Wrapper around the shared manager manifest that exposes world-deps specific
/// helpers (host detection commands + guest install recipes).
#[derive(Clone, Debug)]
pub struct WorldDepsManifest {
    pub version: u32,
    pub tools: Vec<WorldDepTool>,
}

impl WorldDepsManifest {
    pub fn load(base: &Path, overlay: Option<&Path>) -> Result<Self> {
        let manifest = ManagerManifest::load(base, overlay)
            .with_context(|| format!("failed to load manager manifest from {}", base.display()))?;
        let tools = manifest
            .managers
            .iter()
            .map(WorldDepTool::from_manager)
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            version: manifest.version,
            tools,
        })
    }

    pub fn tool(&self, name: &str) -> Option<&WorldDepTool> {
        self.tools
            .iter()
            .find(|tool| tool.name.eq_ignore_ascii_case(name))
    }
}

#[derive(Clone, Debug)]
pub struct WorldDepTool {
    pub name: String,
    pub host: WorldDepDetectSpec,
    pub guest: WorldDepDetectSpec,
    pub install: Vec<WorldDepInstallRecipe>,
}

impl WorldDepTool {
    fn from_manager(spec: &ManagerSpec) -> Result<Self> {
        let host_commands = build_host_commands(&spec.name, &spec.detect.commands);
        let guest_commands = build_guest_commands(&spec.name, spec.guest.detect_cmd.as_deref());
        let install = build_install_recipes(&spec.guest.install, &spec.name)?;

        Ok(Self {
            name: spec.name.clone(),
            host: WorldDepDetectSpec {
                commands: host_commands,
            },
            guest: WorldDepDetectSpec {
                commands: guest_commands,
            },
            install,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct WorldDepDetectSpec {
    pub commands: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct WorldDepInstallRecipe {
    pub provider: String,
    pub script: String,
}

impl WorldDepInstallRecipe {
    pub fn provider_label(&self) -> &str {
        &self.provider
    }
}

fn build_host_commands(name: &str, commands: &[String]) -> Vec<String> {
    if commands.is_empty() {
        vec![format!("command -v {}", name)]
    } else {
        commands.to_vec()
    }
}

fn build_guest_commands(name: &str, detect_cmd: Option<&str>) -> Vec<String> {
    if let Some(cmd) = detect_cmd {
        vec![cmd.to_string()]
    } else {
        vec![format!("command -v {}", name)]
    }
}

fn build_install_recipes(
    spec: &Option<InstallSpec>,
    tool: &str,
) -> Result<Vec<WorldDepInstallRecipe>> {
    let mut recipes = Vec::new();
    if let Some(install) = spec {
        if let Some(apt) = &install.apt {
            let script = apt.trim();
            if script.is_empty() {
                return Err(anyhow!(
                    "tool `{}` declares an apt recipe without commands",
                    tool
                ));
            }
            recipes.push(WorldDepInstallRecipe {
                provider: "apt".to_string(),
                script: script.to_string(),
            });
        }
        if let Some(custom) = &install.custom {
            let script = custom.trim();
            if script.is_empty() {
                return Err(anyhow!(
                    "tool `{}` declares a custom recipe without commands",
                    tool
                ));
            }
            recipes.push(WorldDepInstallRecipe {
                provider: "custom".to_string(),
                script: script.to_string(),
            });
        }
    }

    Ok(recipes)
}
