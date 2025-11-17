use std::{
    collections::{HashMap, HashSet},
    env, fs, io,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;
use serde::Deserialize;
use serde_yaml::Value;

const DEFAULT_PRIORITY: u8 = 50;

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
    /// Load and merge the base manifest plus an optional overlay.
    pub fn load(base: &Path, overlay: Option<&Path>) -> Result<Self> {
        let base_value = read_yaml_value(expand_path(base)?)
            .with_context(|| format!("failed to load manager manifest from {}", base.display()))?;
        let base_manifest: RawManifest =
            serde_yaml::from_value(base_value).context("manager manifest schema is invalid")?;

        let overlay_manifest = if let Some(overlay_path) = overlay {
            let overlay_path = expand_path(overlay_path)?;
            match read_yaml_value_optional(overlay_path.clone())? {
                Some(value) => Some(
                    serde_yaml::from_value(value)
                        .context("overlay manager manifest schema is invalid")?,
                ),
                None => None,
            }
        } else {
            None
        };

        Self::from_raw(base_manifest, overlay_manifest)
    }

    fn from_raw(base: RawManifest, overlay: Option<RawManifest>) -> Result<Self> {
        let mut merged: HashMap<String, RawManagerSpec> = HashMap::new();
        insert_entries(
            &mut merged,
            parse_manager_entries(base.managers)?,
            "base manifest",
        )?;

        if let Some(overlay_manifest) = overlay {
            if overlay_manifest.version != base.version {
                bail!(
                    "overlay manifest version {} does not match base {}",
                    overlay_manifest.version,
                    base.version
                );
            }

            insert_entries(
                &mut merged,
                parse_manager_entries(overlay_manifest.managers)?,
                "overlay manifest",
            )?;
        }

        let mut managers = Vec::with_capacity(merged.len());
        for (name, spec) in merged {
            managers.push(ManagerSpec::from_raw(name, spec)?);
        }

        managers.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| a.name.cmp(&b.name))
        });

        Ok(Self {
            version: base.version,
            managers,
        })
    }

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
    fn from_raw(name: String, spec: RawManagerSpec) -> Result<Self> {
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
    fn from_raw(raw: RawDetectSpec) -> Self {
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
    fn from_raw(raw: RawInitSpec) -> Self {
        Self {
            shell: raw.shell,
            powershell: raw.powershell,
        }
    }

    fn keep_only(&mut self, platform: Platform) {
        match platform {
            Platform::Windows => self.shell = None,
            Platform::Linux | Platform::MacOs => self.powershell = None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct GuestSpec {
    pub detect_cmd: Option<String>,
    pub install: Option<InstallSpec>,
}

impl GuestSpec {
    fn from_raw(detect: Option<RawGuestDetect>, install: Option<RawInstallSpec>) -> Self {
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
    fn from_raw(raw: RawInstallSpec) -> Option<Self> {
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
    fn new(pattern: String) -> Result<Self> {
        let regex = Regex::new(&pattern)?;
        Ok(Self { pattern, regex })
    }
}

#[derive(Debug, Deserialize)]
struct RawManifest {
    version: u32,
    #[serde(default)]
    managers: Value,
}

#[derive(Debug, Deserialize)]
struct RawManagerEntry {
    name: String,
    #[serde(flatten)]
    spec: RawManagerSpec,
}

#[derive(Debug, Default, Deserialize)]
struct RawManagerSpec {
    #[serde(default)]
    priority: Option<u8>,
    #[serde(default)]
    detect: RawDetectSpec,
    #[serde(default)]
    init: RawInitSpec,
    #[serde(default)]
    errors: Vec<String>,
    #[serde(default)]
    repair_hint: Option<String>,
    #[serde(default)]
    guest_detect: Option<RawGuestDetect>,
    #[serde(default)]
    guest_install: Option<RawInstallSpec>,
}

impl RawManagerSpec {
    fn merge(self, overlay: RawManagerSpec) -> RawManagerSpec {
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
struct RawDetectSpec {
    #[serde(default)]
    files: Vec<String>,
    #[serde(default)]
    commands: Vec<String>,
    #[serde(default)]
    env: HashMap<String, String>,
    script: Option<String>,
}

impl RawDetectSpec {
    fn merge(self, overlay: RawDetectSpec) -> RawDetectSpec {
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
struct RawInitSpec {
    shell: Option<String>,
    powershell: Option<String>,
}

impl RawInitSpec {
    fn merge(self, overlay: RawInitSpec) -> RawInitSpec {
        RawInitSpec {
            shell: overlay.shell.or(self.shell),
            powershell: overlay.powershell.or(self.powershell),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct RawGuestDetect {
    command: Option<String>,
}

impl RawGuestDetect {
    fn merge(self, overlay: RawGuestDetect) -> RawGuestDetect {
        RawGuestDetect {
            command: overlay.command.or(self.command),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct RawInstallSpec {
    apt: Option<String>,
    custom: Option<String>,
}

impl RawInstallSpec {
    fn merge(self, overlay: RawInstallSpec) -> RawInstallSpec {
        RawInstallSpec {
            apt: overlay.apt.or(self.apt),
            custom: overlay.custom.or(self.custom),
        }
    }
}

fn parse_manager_entries(value: Value) -> Result<Vec<(String, RawManagerSpec)>> {
    match value {
        Value::Null => Ok(Vec::new()),
        Value::Sequence(entries) => entries
            .into_iter()
            .map(|entry| {
                let raw: RawManagerEntry =
                    serde_yaml::from_value(entry).context("manager entry must include a name")?;
                Ok((raw.name, raw.spec))
            })
            .collect(),
        Value::Mapping(map) => map
            .into_iter()
            .map(|(key, value)| {
                let key = key
                    .as_str()
                    .ok_or_else(|| anyhow!("manager names must be strings"))?
                    .to_string();
                let spec: RawManagerSpec = serde_yaml::from_value(value)
                    .with_context(|| format!("manager `{}` is invalid", key))?;
                Ok((key, spec))
            })
            .collect(),
        other => bail!(
            "`managers` must be a mapping or sequence, found {:?}",
            other
        ),
    }
}

fn read_yaml_value(path: PathBuf) -> Result<Value> {
    let data = fs::read_to_string(&path)
        .with_context(|| format!("failed to read manifest at {}", path.display()))?;
    serde_yaml::from_str(&data)
        .with_context(|| format!("failed to parse manifest at {}", path.display()))
}

fn read_yaml_value_optional(path: PathBuf) -> Result<Option<Value>> {
    match fs::read_to_string(&path) {
        Ok(contents) => {
            let value = serde_yaml::from_str(&contents)
                .with_context(|| format!("failed to parse overlay at {}", path.display()))?;
            Ok(Some(value))
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(anyhow!(err))
            .with_context(|| format!("failed to read overlay at {}", path.display())),
    }
}

fn expand_path(path: &Path) -> Result<PathBuf> {
    let raw = path
        .to_str()
        .ok_or_else(|| anyhow!("manifest path contains invalid UTF-8"))?;
    Ok(PathBuf::from(expand_env_and_tilde(raw)))
}

fn expand_env_and_tilde(raw: &str) -> String {
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

fn insert_entries(
    target: &mut HashMap<String, RawManagerSpec>,
    entries: Vec<(String, RawManagerSpec)>,
    origin: &str,
) -> Result<()> {
    let mut seen = HashSet::new();
    for (name, spec) in entries {
        if !seen.insert(name.clone()) {
            bail!("duplicate manager entry `{}` in {}", name, origin);
        }
        if let Some(existing) = target.remove(&name) {
            target.insert(name, existing.merge(spec));
        } else {
            target.insert(name, spec);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs, path::Path};
    use tempfile::tempdir;

    #[test]
    fn loads_manifest_with_overlay_and_sorting() {
        let dir = tempdir().unwrap();
        let base_path = dir.path().join("base.yaml");
        let overlay_path = dir.path().join("overlay.yaml");

        let base = r#"
version: 1
managers:
  - name: nvm
    priority: 20
    detect:
      files:
        - "$SUBSTRATE_TEST_HOME/.nvm/nvm.sh"
    init:
      shell: "echo source nvm"
    errors:
      - "nvm: .*"
  - name: pyenv
    priority: 5
    detect:
      commands: ["pyenv --version"]
    init:
      shell: "echo pyenv"
    errors: []
"#;

        let overlay = r#"
version: 1
managers:
  nvm:
    priority: 1
  asdf:
    detect:
      files: ["~/custom/.asdf"]
    init:
      shell: "echo asdf"
    errors:
      - "asdf: .*"
"#;

        fs::write(&base_path, base).unwrap();
        fs::write(&overlay_path, overlay).unwrap();
        env::set_var(
            "SUBSTRATE_TEST_HOME",
            dir.path().to_string_lossy().to_string(),
        );

        let manifest = ManagerManifest::load(&base_path, Some(&overlay_path)).unwrap();
        assert_eq!(manifest.version, 1);
        assert_eq!(manifest.managers.len(), 3);
        assert_eq!(manifest.managers[0].name, "nvm");
        assert_eq!(manifest.managers[0].priority, 1);
        assert_eq!(manifest.managers[1].name, "pyenv");
        assert_eq!(manifest.managers[2].name, "asdf");

        assert!(manifest.managers[0].detect.files[0]
            .display()
            .to_string()
            .contains(dir.path().to_str().unwrap()));

        env::remove_var("SUBSTRATE_TEST_HOME");
    }

    #[test]
    fn expands_env_and_tilde_in_detect_fields() {
        let dir = tempdir().unwrap();
        let base_path = dir.path().join("base.yaml");
        fs::write(
            &base_path,
            r#"
version: 1
managers:
  envy:
    detect:
      files:
        - "~/.substrate/envy.sh"
        - "$MANAGER_TEST_HOME/scripts/envy.sh"
      env:
        ENVY_ROOT: "~/.substrate/envy"
        MANAGER_HOME: "$MANAGER_TEST_HOME/manager"
    init: {}
    errors: []
"#,
        )
        .unwrap();

        env::set_var("MANAGER_TEST_HOME", dir.path());
        let manifest = ManagerManifest::load(&base_path, None).unwrap();
        env::remove_var("MANAGER_TEST_HOME");

        let detect = &manifest.managers[0].detect;
        let home = dirs::home_dir().expect("home directory must be set for tests");
        assert_eq!(detect.files[0], home.join(".substrate/envy.sh"));
        assert_eq!(detect.files[1], dir.path().join("scripts/envy.sh"));
        let expected_env = format!("{}/manager", dir.path().display());
        assert_eq!(
            detect.env.get("MANAGER_HOME").map(String::as_str),
            Some(expected_env.as_str())
        );
        assert!(detect.env.get("ENVY_ROOT").is_some());
        assert!(detect
            .env
            .get("ENVY_ROOT")
            .unwrap()
            .contains(".substrate/envy"));
    }

    #[test]
    fn overlay_merges_detect_env_and_install_spec() {
        let dir = tempdir().unwrap();
        let base_path = dir.path().join("base.yaml");
        let overlay_path = dir.path().join("overlay.yaml");
        fs::write(
            &base_path,
            r#"
version: 1
managers:
  direnv:
    detect:
      files: ["~/.direnv"]
      commands: ["direnv version"]
      env:
        DIRENV_DIR: "/base/path"
    init: {}
    errors: []
    guest_detect:
      command: "direnv version"
    guest_install:
      apt: "sudo apt install direnv"
"#,
        )
        .unwrap();
        fs::write(
            &overlay_path,
            r#"
version: 1
managers:
  direnv:
    detect:
      commands: ["direnv --help"]
      env:
        DIRENV_HOME: "/overlay/home"
    guest_install:
      custom: "run-me.sh"
"#,
        )
        .unwrap();

        let manifest = ManagerManifest::load(&base_path, Some(&overlay_path)).unwrap();
        let direnv = manifest
            .managers
            .iter()
            .find(|spec| spec.name == "direnv")
            .unwrap();

        assert_eq!(direnv.detect.commands, vec!["direnv --help"]);
        let home = dirs::home_dir().expect("home directory must be set for tests");
        assert_eq!(direnv.detect.files[0], home.join(".direnv"));
        assert_eq!(
            direnv.detect.env.get("DIRENV_DIR").map(String::as_str),
            Some("/base/path")
        );
        assert_eq!(
            direnv.detect.env.get("DIRENV_HOME").map(String::as_str),
            Some("/overlay/home")
        );
        let install = direnv.guest.install.as_ref().expect("install spec");
        assert_eq!(install.apt.as_deref(), Some("sudo apt install direnv"));
        assert_eq!(install.custom.as_deref(), Some("run-me.sh"));
    }

    #[test]
    fn overlay_version_mismatch_returns_error() {
        let dir = tempdir().unwrap();
        let base_path = dir.path().join("base.yaml");
        let overlay_path = dir.path().join("overlay.yaml");
        fs::write(
            &base_path,
            r#"
version: 1
managers:
  base:
    detect: {}
    init: {}
    errors: []
"#,
        )
        .unwrap();
        fs::write(
            &overlay_path,
            r#"
version: 2
managers:
  base:
    detect: {}
    init: {}
    errors: []
"#,
        )
        .unwrap();

        let err = ManagerManifest::load(&base_path, Some(&overlay_path)).unwrap_err();
        assert!(err
            .to_string()
            .contains("overlay manifest version 2 does not match base 1"));
    }

    #[test]
    fn missing_overlay_is_ignored() {
        let dir = tempdir().unwrap();
        let base_path = dir.path().join("base.yaml");
        fs::write(
            &base_path,
            r#"
version: 1
managers:
  - name: only
    detect:
      files: ["~/file"]
    init:
      shell: "echo"
    errors: []
"#,
        )
        .unwrap();

        let missing = dir.path().join("missing.yaml");
        let manifest = ManagerManifest::load(&base_path, Some(&missing)).unwrap();
        assert_eq!(manifest.managers.len(), 1);
    }

    #[test]
    fn duplicate_names_return_error() {
        let dir = tempdir().unwrap();
        let base_path = dir.path().join("base.yaml");
        fs::write(
            &base_path,
            r#"
version: 1
managers:
  - name: dup
    detect: {}
    init: {}
    errors: []
  - name: dup
    detect: {}
    init: {}
    errors: []
"#,
        )
        .unwrap();

        let err = ManagerManifest::load(&base_path, None).unwrap_err();
        assert!(err.to_string().contains("duplicate manager"));
    }

    #[test]
    fn invalid_regex_is_reported() {
        let dir = tempdir().unwrap();
        let base_path = dir.path().join("base.yaml");
        fs::write(
            &base_path,
            r#"
version: 1
managers:
  broken:
    detect: {}
    init: {}
    errors:
      - "[unclosed"
"#,
        )
        .unwrap();

        let err = ManagerManifest::load(&base_path, None).unwrap_err();
        assert!(err.to_string().contains("invalid regex"));
    }

    #[test]
    fn resolve_for_platform_trims_init_snippets() {
        let dir = tempdir().unwrap();
        let base_path = dir.path().join("base.yaml");
        fs::write(
            &base_path,
            r#"
version: 1
managers:
  shellish:
    detect: {}
    init:
      shell: "echo shell"
      powershell: "Write-Host shell"
    errors: []
"#,
        )
        .unwrap();

        let manifest = ManagerManifest::load(&base_path, None).unwrap();
        let windows = manifest.resolve_for_platform(Platform::Windows);
        assert!(windows[0].init.shell.is_none());
        assert!(windows[0].init.powershell.is_some());

        let linux = manifest.resolve_for_platform(Platform::Linux);
        assert!(linux[0].init.shell.is_some());
        assert!(linux[0].init.powershell.is_none());
    }

    #[test]
    fn tier2_managers_include_complete_metadata() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("tier2.yaml");
        let tier2_root = dir.path().join("tier2-home");
        fs::create_dir_all(&tier2_root).unwrap();
        env::set_var("TIER2_HOME", &tier2_root);

        let manifest_body = format!(
            r#"
version: 1
managers:
  - name: mise
    priority: 9
    detect:
      commands: ["mise --version"]
      env:
        MISE_DATA_DIR: "$TIER2_HOME/state/mise"
      script: |
        test -x "$MISE_DATA_DIR/bin/mise"
    init:
      shell: |
        export MISE_DATA_DIR="${{MISE_DATA_DIR:-$HOME/.local/share/mise}}"
        eval "$(mise activate bash)"
    errors:
      - "mise: command not found"
    repair_hint: |
      eval "$(mise activate bash)"
    guest_detect:
      command: "mise --version"
    guest_install:
      apt: "sudo apt install mise"
  - name: rtx
    priority: 12
    detect:
      files:
        - "~/.local/share/rtx/bin/rtx"
      commands: ["rtx --version"]
    init:
      shell: |
        eval "$(rtx activate bash)"
    errors:
      - "rtx: command not found"
    repair_hint: |
      eval "$(rtx activate bash)"
    guest_detect:
      command: "rtx --version"
    guest_install:
      custom: "curl https://rtx.pub/install.sh | sh"
  - name: rbenv
    priority: 14
    detect:
      files:
        - "{tier2}/.rbenv/bin/rbenv"
      env:
        RBENV_ROOT: "{tier2}/.rbenv"
    init:
      shell: |
        export RBENV_ROOT="${{RBENV_ROOT:-$HOME/.rbenv}}"
        eval "$(rbenv init - bash)"
    errors:
      - "rbenv: command not found"
    repair_hint: |
      eval "$(rbenv init - bash)"
    guest_detect:
      command: "rbenv --version"
    guest_install:
      apt: "sudo apt install rbenv"
  - name: sdkman
    priority: 18
    detect:
      files:
        - "~/.sdkman/bin/sdkman-init.sh"
      env:
        SDKMAN_DIR: "~/.sdkman"
      script: |
        test -d "$SDKMAN_DIR/candidates"
    init:
      shell: |
        export SDKMAN_DIR="${{SDKMAN_DIR:-$HOME/.sdkman}}"
        source "$SDKMAN_DIR/bin/sdkman-init.sh"
    errors:
      - "sdk: command not found"
    repair_hint: |
      source "$SDKMAN_DIR/bin/sdkman-init.sh"
    guest_detect:
      command: "sdk version"
    guest_install:
      custom: "curl -s https://get.sdkman.io | bash"
  - name: bun
    priority: 30
    detect:
      commands: ["bun --version"]
      files:
        - "~/.bun/bin/bun"
    init:
      shell: |
        export BUN_INSTALL="${{BUN_INSTALL:-$HOME/.bun}}"
        export PATH="$BUN_INSTALL/bin:$PATH"
    errors:
      - "bun: command not found"
    repair_hint: |
      curl https://bun.sh/install | bash
    guest_detect:
      command: "bun --version"
    guest_install:
      custom: "curl https://bun.sh/install | bash"
  - name: volta
    priority: 22
    detect:
      env:
        VOLTA_HOME: "$TIER2_HOME/.volta"
      script: |
        test -x "$VOLTA_HOME/bin/volta"
    init:
      shell: |
        export VOLTA_HOME="${{VOLTA_HOME:-$HOME/.volta}}"
        export PATH="$VOLTA_HOME/bin:$PATH"
    errors:
      - "volta: command not found"
    repair_hint: |
      export VOLTA_HOME="${{VOLTA_HOME:-$HOME/.volta}}"
    guest_detect:
      command: "volta --version"
    guest_install:
      apt: "sudo apt install volta"
  - name: goenv
    priority: 35
    detect:
      files:
        - "{tier2}/.goenv/bin/goenv"
      env:
        GOENV_ROOT: "{tier2}/.goenv"
    init:
      shell: |
        export GOENV_ROOT="${{GOENV_ROOT:-$HOME/.goenv}}"
        eval "$(goenv init -)"
    errors:
      - "goenv: command not found"
    repair_hint: |
      eval "$(goenv init -)"
    guest_detect:
      command: "goenv --version"
    guest_install:
      apt: "sudo apt install goenv"
      custom: "brew install goenv"
  - name: asdf-node
    priority: 40
    detect:
      commands:
        - "asdf current nodejs"
    init:
      shell: |
        asdf exec node --version >/dev/null 2>&1
    errors:
      - "asdf: .*"
    repair_hint: |
      asdf plugin add nodejs
    guest_detect:
      command: "asdf current nodejs"
    guest_install:
      custom: "asdf plugin add nodejs && asdf install nodejs latest"
"#,
            tier2 = tier2_root.display()
        );
        fs::write(&manifest_path, manifest_body).unwrap();

        let manifest = ManagerManifest::load(&manifest_path, None).unwrap();
        env::remove_var("TIER2_HOME");

        let expected = [
            "mise",
            "rtx",
            "rbenv",
            "sdkman",
            "bun",
            "volta",
            "goenv",
            "asdf-node",
        ];
        for name in expected {
            assert!(
                manifest.managers.iter().any(|spec| spec.name == name),
                "expected {name} entry in manifest"
            );
        }

        let find = |name: &str| -> &ManagerSpec {
            manifest
                .managers
                .iter()
                .find(|spec| spec.name == name)
                .unwrap_or_else(|| panic!("missing {name} spec"))
        };

        let tier2_mise_dir = tier2_root.join("state").join("mise");
        let mise = find("mise");
        let mise_data = mise
            .detect
            .env
            .get("MISE_DATA_DIR")
            .expect("MISE_DATA_DIR env");
        assert_eq!(Path::new(mise_data), tier2_mise_dir);
        assert!(
            mise.repair_hint
                .as_deref()
                .map(|hint| hint.contains("mise activate bash"))
                .unwrap_or(false),
            "expected mise repair hint to mention activation",
        );
        assert!(mise
            .detect
            .script
            .as_deref()
            .unwrap()
            .contains("$MISE_DATA_DIR/bin/mise"));
        assert_eq!(mise.guest.detect_cmd.as_deref(), Some("mise --version"));
        let mise_install = mise.guest.install.as_ref().expect("mise install spec");
        assert_eq!(mise_install.apt.as_deref(), Some("sudo apt install mise"));

        let home = dirs::home_dir().expect("home directory");
        let rtx = find("rtx");
        assert_eq!(rtx.detect.files[0], home.join(".local/share/rtx/bin/rtx"));
        let rtx_install = rtx.guest.install.as_ref().expect("rtx install spec");
        assert_eq!(
            rtx_install.custom.as_deref(),
            Some("curl https://rtx.pub/install.sh | sh")
        );

        let rbenv = find("rbenv");
        assert_eq!(rbenv.detect.files[0], tier2_root.join(".rbenv/bin/rbenv"));
        let expected_rbenv_root = format!("{}/.rbenv", tier2_root.display());
        assert_eq!(
            rbenv.detect.env.get("RBENV_ROOT").map(String::as_str),
            Some(expected_rbenv_root.as_str())
        );
        assert_eq!(rbenv.guest.detect_cmd.as_deref(), Some("rbenv --version"));

        let sdkman = find("sdkman");
        let sdkman_dir = sdkman.detect.env.get("SDKMAN_DIR").expect("SDKMAN_DIR env");
        assert_eq!(Path::new(sdkman_dir), home.join(".sdkman"));
        assert!(sdkman
            .detect
            .script
            .as_deref()
            .unwrap()
            .contains("candidates"));

        let bun = find("bun");
        assert_eq!(bun.detect.files[0], home.join(".bun/bin/bun"));
        assert_eq!(bun.errors[0].pattern, "bun: command not found".to_string());

        let volta = find("volta");
        let volta_home = volta.detect.env.get("VOLTA_HOME").expect("VOLTA_HOME env");
        assert_eq!(Path::new(volta_home), tier2_root.join(".volta"));

        let goenv = find("goenv");
        let install = goenv.guest.install.as_ref().expect("goenv install");
        assert_eq!(install.apt.as_deref(), Some("sudo apt install goenv"));
        assert_eq!(install.custom.as_deref(), Some("brew install goenv"));

        let asdf_node = find("asdf-node");
        assert_eq!(
            asdf_node.guest.detect_cmd.as_deref(),
            Some("asdf current nodejs")
        );
        assert!(asdf_node.repair_hint.as_ref().unwrap().contains("plugin"));
    }
}
