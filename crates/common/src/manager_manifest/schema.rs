use anyhow::{Context, Result};
use regex::Regex;
use serde::Deserialize;
use serde_yaml::Value;
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

pub const DEFAULT_PRIORITY: u8 = 50;
pub const MANAGER_MANIFEST_VERSION: u32 = 2;

/// Platform-specific view selection when resolving manager manifests.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Platform {
    Linux,
    MacOs,
    Windows,
}

#[derive(Clone, Debug)]
pub struct ManagerManifest {
    pub version: u32,
    pub managers: Vec<ManagerSpec>,
}

impl ManagerManifest {
    /// Return manifest managers filtered for a specific platform.
    pub fn resolve_for_platform(&self, platform: Platform) -> Vec<ManagerSpec> {
        let mut managers = self.managers.clone();
        for manager in &mut managers {
            manager.init.keep_only(platform);
        }
        managers
    }
}

#[derive(Clone, Debug)]
pub struct ManagerSpec {
    pub name: String,
    pub priority: u8,
    pub detect: DetectSpec,
    pub init: InitSpec,
    pub errors: Vec<RegexPattern>,
    pub repair_hint: Option<String>,
    pub guest: GuestSpec,
}

impl ManagerSpec {
    pub(crate) fn from_raw(name: String, spec: RawManagerSpec, manifest_version: u32) -> Result<Self> {
        let detect = DetectSpec::from_raw(spec.detect);
        let init = InitSpec::from_raw(spec.init);
        let errors = spec
            .errors
            .into_iter()
            .map(|pattern| {
                RegexPattern::new(pattern.clone())
                    .with_context(|| format!("manager `{}` has invalid regex `{}`", name, pattern))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            name,
            priority: spec.priority.unwrap_or(DEFAULT_PRIORITY),
            detect,
            init,
            errors,
            repair_hint: spec.repair_hint,
            guest: GuestSpec::from_raw(spec.guest_detect, spec.guest_install, manifest_version)
                .with_context(|| format!("manager `{}` guest spec is invalid", name))?,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct DetectSpec {
    pub files: Vec<PathBuf>,
    pub commands: Vec<String>,
    pub env: HashMap<String, String>,
    pub script: Option<String>,
}

impl DetectSpec {
    pub(crate) fn from_raw(raw: RawDetectSpec) -> Self {
        let files = raw
            .files
            .into_iter()
            .map(|value| PathBuf::from(expand_env_and_tilde(&value)))
            .collect();
        let env = raw
            .env
            .into_iter()
            .map(|(key, value)| (key, expand_env_and_tilde(&value)))
            .collect();

        Self {
            files,
            commands: raw.commands,
            env,
            script: raw.script,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct InitSpec {
    pub shell: Option<String>,
    pub powershell: Option<String>,
}

impl InitSpec {
    pub(crate) fn from_raw(raw: RawInitSpec) -> Self {
        Self {
            shell: raw.shell,
            powershell: raw.powershell,
        }
    }

    pub(crate) fn keep_only(&mut self, platform: Platform) {
        match platform {
            Platform::Windows => {
                // Prefer a PowerShell snippet on Windows when present,
                // but fall back to a cross-platform shell snippet.
                if self.powershell.is_some() {
                    self.shell = None;
                }
            }
            Platform::Linux | Platform::MacOs => {
                // Non-Windows platforms never use PowerShell snippets.
                self.powershell = None;
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct GuestSpec {
    pub detect_cmd: Option<String>,
    pub install: Option<InstallSpec>,
}

impl GuestSpec {
    pub(crate) fn from_raw(
        detect: Option<RawGuestDetect>,
        install: Option<RawInstallSpec>,
        manifest_version: u32,
    ) -> Result<Self> {
        let detect_cmd = detect.and_then(|spec| spec.command);
        let install = InstallSpec::from_raw(install, detect_cmd.as_deref(), manifest_version)?;
        Ok(Self { detect_cmd, install })
    }
}

#[derive(Clone, Debug, Default)]
pub struct InstallSpec {
    pub class: InstallClass,
    pub custom: Option<String>,
    pub system_packages: Option<SystemPackagesSpec>,
    pub manual_instructions: Option<String>,
}

impl InstallSpec {
    pub(crate) fn from_raw(
        raw: Option<RawInstallSpec>,
        guest_detect_cmd: Option<&str>,
        manifest_version: u32,
    ) -> Result<Option<Self>> {
        let Some(raw) = raw else {
            return Ok(None);
        };

        if manifest_version != MANAGER_MANIFEST_VERSION {
            anyhow::bail!(
                "manifest version must be {} (got {})",
                MANAGER_MANIFEST_VERSION,
                manifest_version
            );
        }

        let class = raw.class.ok_or_else(|| {
            anyhow::anyhow!("guest_install.class is required (user_space|system_packages|manual|copy_from_host)")
        })?;

        match class {
            InstallClass::UserSpace => {
                if raw.system_packages.is_some() || raw.manual_instructions.is_some() {
                    anyhow::bail!(
                        "guest_install.class=user_space forbids system_packages/manual_instructions"
                    );
                }
                let custom = raw.custom.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
                if custom.is_none() {
                    anyhow::bail!("guest_install.class=user_space requires custom");
                }
                Ok(Some(Self {
                    class,
                    custom,
                    system_packages: None,
                    manual_instructions: None,
                }))
            }
            InstallClass::SystemPackages => {
                if raw.custom.is_some() || raw.manual_instructions.is_some() {
                    anyhow::bail!(
                        "guest_install.class=system_packages forbids custom/manual_instructions"
                    );
                }
                if guest_detect_cmd.is_none() {
                    anyhow::bail!(
                        "guest_install.class=system_packages requires guest_detect.command"
                    );
                }
                let system_packages = raw.system_packages.ok_or_else(|| {
                    anyhow::anyhow!("guest_install.class=system_packages requires system_packages")
                })?;
                let system_packages = SystemPackagesSpec::from_raw(system_packages)?;
                Ok(Some(Self {
                    class,
                    custom: None,
                    system_packages: Some(system_packages),
                    manual_instructions: None,
                }))
            }
            InstallClass::Manual => {
                if raw.custom.is_some() || raw.system_packages.is_some() {
                    anyhow::bail!(
                        "guest_install.class=manual forbids custom/system_packages"
                    );
                }
                let manual_instructions = raw
                    .manual_instructions
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
                if manual_instructions.is_none() {
                    anyhow::bail!("guest_install.class=manual requires manual_instructions");
                }
                Ok(Some(Self {
                    class,
                    custom: None,
                    system_packages: None,
                    manual_instructions,
                }))
            }
            InstallClass::CopyFromHost => {
                if raw.custom.is_some()
                    || raw.system_packages.is_some()
                    || raw.manual_instructions.is_some()
                {
                    anyhow::bail!(
                        "guest_install.class=copy_from_host forbids custom/system_packages/manual_instructions"
                    );
                }
                Ok(Some(Self {
                    class,
                    custom: None,
                    system_packages: None,
                    manual_instructions: None,
                }))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallClass {
    UserSpace,
    SystemPackages,
    Manual,
    CopyFromHost,
}

#[derive(Clone, Debug)]
pub struct SystemPackagesSpec {
    pub apt: Vec<String>,
}

impl SystemPackagesSpec {
    fn from_raw(raw: RawSystemPackagesSpec) -> Result<Self> {
        let apt = raw.apt;
        if apt.is_empty() {
            anyhow::bail!("guest_install.system_packages.apt must be non-empty");
        }
        Ok(Self { apt })
    }
}

#[derive(Clone, Debug)]
pub struct RegexPattern {
    pub pattern: String,
    pub regex: Regex,
}

impl RegexPattern {
    pub(crate) fn new(pattern: String) -> Result<Self> {
        let regex = Regex::new(&pattern)?;
        Ok(Self { pattern, regex })
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawManifest {
    pub(crate) version: u32,
    #[serde(default)]
    pub(crate) managers: Value,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawManagerEntry {
    pub(crate) name: String,
    #[serde(flatten)]
    pub(crate) spec: RawManagerSpec,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct RawManagerSpec {
    #[serde(default)]
    pub(crate) priority: Option<u8>,
    #[serde(default)]
    pub(crate) detect: RawDetectSpec,
    #[serde(default)]
    pub(crate) init: RawInitSpec,
    #[serde(default)]
    pub(crate) errors: Vec<String>,
    #[serde(default)]
    pub(crate) repair_hint: Option<String>,
    #[serde(default)]
    pub(crate) guest_detect: Option<RawGuestDetect>,
    #[serde(default)]
    pub(crate) guest_install: Option<RawInstallSpec>,
}

impl RawManagerSpec {
    pub(crate) fn merge(self, overlay: RawManagerSpec) -> RawManagerSpec {
        RawManagerSpec {
            priority: overlay.priority.or(self.priority),
            detect: self.detect.merge(overlay.detect),
            init: self.init.merge(overlay.init),
            errors: if overlay.errors.is_empty() {
                self.errors
            } else {
                overlay.errors
            },
            repair_hint: overlay.repair_hint.or(self.repair_hint),
            guest_detect: match (self.guest_detect, overlay.guest_detect) {
                (Some(base), Some(next)) => Some(base.merge(next)),
                (None, Some(next)) => Some(next),
                (Some(base), None) => Some(base),
                (None, None) => None,
            },
            guest_install: match (self.guest_install, overlay.guest_install) {
                (Some(base), Some(next)) => Some(base.merge(next)),
                (None, Some(next)) => Some(next),
                (Some(base), None) => Some(base),
                (None, None) => None,
            },
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct RawDetectSpec {
    #[serde(default)]
    pub(crate) files: Vec<String>,
    #[serde(default)]
    pub(crate) commands: Vec<String>,
    #[serde(default)]
    pub(crate) env: HashMap<String, String>,
    pub(crate) script: Option<String>,
}

impl RawDetectSpec {
    pub(crate) fn merge(self, overlay: RawDetectSpec) -> RawDetectSpec {
        let files = if overlay.files.is_empty() {
            self.files
        } else {
            overlay.files
        };
        let commands = if overlay.commands.is_empty() {
            self.commands
        } else {
            overlay.commands
        };
        let env = if overlay.env.is_empty() {
            self.env
        } else {
            let mut merged = self.env;
            for (key, value) in overlay.env {
                merged.insert(key, value);
            }
            merged
        };

        RawDetectSpec {
            files,
            commands,
            env,
            script: overlay.script.or(self.script),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct RawInitSpec {
    pub(crate) shell: Option<String>,
    pub(crate) powershell: Option<String>,
}

impl RawInitSpec {
    pub(crate) fn merge(self, overlay: RawInitSpec) -> RawInitSpec {
        RawInitSpec {
            shell: overlay.shell.or(self.shell),
            powershell: overlay.powershell.or(self.powershell),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct RawGuestDetect {
    pub(crate) command: Option<String>,
}

impl RawGuestDetect {
    pub(crate) fn merge(self, overlay: RawGuestDetect) -> RawGuestDetect {
        RawGuestDetect {
            command: overlay.command.or(self.command),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct RawInstallSpec {
    #[serde(default)]
    pub(crate) class: Option<InstallClass>,
    pub(crate) custom: Option<String>,
    pub(crate) system_packages: Option<RawSystemPackagesSpec>,
    pub(crate) manual_instructions: Option<String>,
}

impl RawInstallSpec {
    pub(crate) fn merge(self, overlay: RawInstallSpec) -> RawInstallSpec {
        RawInstallSpec {
            class: overlay.class.or(self.class),
            custom: overlay.custom.or(self.custom),
            system_packages: match (self.system_packages, overlay.system_packages) {
                (Some(base), Some(next)) => Some(base.merge(next)),
                (None, Some(next)) => Some(next),
                (Some(base), None) => Some(base),
                (None, None) => None,
            },
            manual_instructions: overlay.manual_instructions.or(self.manual_instructions),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawSystemPackagesSpec {
    #[serde(default)]
    pub(crate) apt: Vec<String>,
}

impl RawSystemPackagesSpec {
    pub(crate) fn merge(self, overlay: RawSystemPackagesSpec) -> RawSystemPackagesSpec {
        RawSystemPackagesSpec {
            apt: if overlay.apt.is_empty() { self.apt } else { overlay.apt },
        }
    }
}

pub(crate) fn expand_path(path: &Path) -> Result<PathBuf> {
    let raw = path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("manifest path contains invalid UTF-8"))?;
    Ok(PathBuf::from(expand_env_and_tilde(raw)))
}

pub(crate) fn expand_env_and_tilde(raw: &str) -> String {
    let tilde_expanded = expand_tilde(raw);
    expand_env_vars(&tilde_expanded)
}

fn expand_tilde(raw: &str) -> String {
    if !raw.starts_with('~') {
        return raw.to_string();
    }

    if let Some(home) = dirs::home_dir() {
        if raw == "~" {
            return home.to_string_lossy().to_string();
        }

        let remainder = &raw[1..];
        if remainder.starts_with('/') || remainder.starts_with('\\') {
            let trimmed = remainder.trim_start_matches(['/', '\\']);
            if trimmed.is_empty() {
                return home.to_string_lossy().to_string();
            }
            return home.join(trimmed).to_string_lossy().to_string();
        }
    }

    raw.to_string()
}

fn expand_env_vars(raw: &str) -> String {
    let mut result = String::new();
    let mut chars = raw.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            if matches!(chars.peek(), Some('{')) {
                chars.next();
                let mut name = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '}' {
                        break;
                    }
                    name.push(c);
                    chars.next();
                }

                if matches!(chars.peek(), Some('}')) {
                    chars.next();
                } else {
                    result.push_str("${");
                    result.push_str(&name);
                    continue;
                }

                if name.is_empty() {
                    continue;
                }

                if let Ok(value) = env::var(&name) {
                    result.push_str(&value);
                } else {
                    result.push_str("${");
                    result.push_str(&name);
                    result.push('}');
                }
                continue;
            }

            let mut name = String::new();
            while let Some(&c) = chars.peek() {
                if c == '_' || c.is_ascii_alphanumeric() {
                    name.push(c);
                    chars.next();
                } else {
                    break;
                }
            }

            if name.is_empty() {
                result.push('$');
                continue;
            }

            if let Ok(value) = env::var(&name) {
                result.push_str(&value);
            } else {
                result.push('$');
                result.push_str(&name);
            }
            continue;
        }

        result.push(ch);
    }

    result
}
