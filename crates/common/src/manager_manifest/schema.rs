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
    pub(crate) fn from_raw(name: String, spec: RawManagerSpec) -> Result<Self> {
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
            guest: GuestSpec::from_raw(spec.guest_detect, spec.guest_install),
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
    ) -> Self {
        Self {
            detect_cmd: detect.and_then(|spec| spec.command),
            install: install.and_then(InstallSpec::from_raw),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct InstallSpec {
    pub apt: Option<String>,
    pub custom: Option<String>,
}

impl InstallSpec {
    pub(crate) fn from_raw(raw: RawInstallSpec) -> Option<Self> {
        if raw.apt.is_none() && raw.custom.is_none() {
            None
        } else {
            Some(Self {
                apt: raw.apt,
                custom: raw.custom,
            })
        }
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
    pub(crate) apt: Option<String>,
    pub(crate) custom: Option<String>,
}

impl RawInstallSpec {
    pub(crate) fn merge(self, overlay: RawInstallSpec) -> RawInstallSpec {
        RawInstallSpec {
            apt: overlay.apt.or(self.apt),
            custom: overlay.custom.or(self.custom),
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
