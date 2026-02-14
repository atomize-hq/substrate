use crate::execution::config_model;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HostPlatform {
    Linux,
    MacOs,
    Windows,
}

impl HostPlatform {
    pub(crate) fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOs
        } else if cfg!(windows) {
            Self::Windows
        } else {
            Self::Linux
        }
    }

    fn matches(self, platforms: &[HostPlatform]) -> bool {
        platforms.contains(&self)
    }
}

impl std::fmt::Display for HostPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            HostPlatform::Linux => "linux",
            HostPlatform::MacOs => "macos",
            HostPlatform::Windows => "windows",
        })
    }
}

fn parse_platform(raw: &str) -> Option<HostPlatform> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "linux" => Some(HostPlatform::Linux),
        "macos" => Some(HostPlatform::MacOs),
        "windows" => Some(HostPlatform::Windows),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum InstallMethodV1 {
    Apt,
    Script,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct AptSpecV1 {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct InstallDefV1 {
    pub method: InstallMethodV1,
    #[serde(default)]
    pub apt: Vec<AptSpecV1>,
    #[serde(default)]
    pub script: Option<String>,
    #[serde(default)]
    pub script_path: Option<String>,
    #[serde(default)]
    pub manual_instructions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProbeDefV1 {
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub(crate) enum WrapperKindV1 {
    BashFunction {
        bash_source: String,
        function: String,
    },
    BashSourceExec {
        bash_source: String,
        exec: String,
    },
    ShEnvExec {
        exec: String,
        #[serde(default)]
        env: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct WrapperDefV1 {
    pub name: String,
    #[serde(flatten)]
    pub kind: WrapperKindV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct PackageDefV1 {
    pub version: u32,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub runnable: bool,
    #[serde(default)]
    pub entrypoints: Vec<String>,
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub wrappers: Vec<WrapperDefV1>,
    pub install: InstallDefV1,
    #[serde(default)]
    pub probe: Option<ProbeDefV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleDefV1 {
    pub version: u32,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub platforms: Vec<String>,
    pub packages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub(crate) enum InventoryItemDefV1 {
    Package(PackageDefV1),
    Bundle(BundleDefV1),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct InventoryViewV1 {
    pub packages: BTreeMap<String, PackageDefV1>,
    pub bundles: BTreeMap<String, BundleDefV1>,
}

impl InventoryViewV1 {
    pub(crate) fn is_empty(&self) -> bool {
        self.packages.is_empty() && self.bundles.is_empty()
    }

    pub(crate) fn get(&self, name: &str) -> Option<InventoryItemDefV1> {
        if let Some(pkg) = self.packages.get(name) {
            return Some(InventoryItemDefV1::Package(pkg.clone()));
        }
        if let Some(bundle) = self.bundles.get(name) {
            return Some(InventoryItemDefV1::Bundle(bundle.clone()));
        }
        None
    }

    pub(crate) fn validate_no_collisions(&self) -> Result<()> {
        for name in self.packages.keys() {
            if self.bundles.contains_key(name) {
                return Err(config_model::user_error(format!(
                    "invalid deps inventory: name collision: '{name}' exists in both packages and bundles"
                )));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct InventoryListItemSummaryV1 {
    pub kind: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runnable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<InstallMethodV1>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub entrypoints: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub platforms: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

pub(crate) fn builtin_inventory_v1(platform: HostPlatform) -> InventoryViewV1 {
    let mut view = InventoryViewV1::default();
    for pkg in builtin_packages_v1() {
        if is_visible_on_platform(&pkg.platforms, platform).unwrap_or(true) {
            view.packages.insert(pkg.name.clone(), pkg);
        }
    }
    for bundle in builtin_bundles_v1() {
        if is_visible_on_platform(&bundle.platforms, platform).unwrap_or(true) {
            view.bundles.insert(bundle.name.clone(), bundle);
        }
    }
    view
}

fn builtin_packages_v1() -> Vec<PackageDefV1> {
    vec![
        PackageDefV1 {
            version: 1,
            name: "bun".to_string(),
            description: Some("Bun runtime (script install into world-deps prefix).".to_string()),
            runnable: true,
            entrypoints: vec!["bun".to_string()],
            platforms: Vec::new(),
            wrappers: Vec::new(),
            install: InstallDefV1 {
                method: InstallMethodV1::Script,
                apt: Vec::new(),
                script: None,
                script_path: Some("../scripts/bun.sh".to_string()),
                manual_instructions: None,
            },
            probe: Some(ProbeDefV1 {
                command: "bun --version".to_string(),
            }),
        },
        PackageDefV1 {
            version: 1,
            name: "node".to_string(),
            description: Some("Node.js runtime via apt.".to_string()),
            runnable: true,
            entrypoints: vec!["node".to_string()],
            platforms: Vec::new(),
            wrappers: Vec::new(),
            install: InstallDefV1 {
                method: InstallMethodV1::Apt,
                apt: vec![AptSpecV1 {
                    name: "nodejs".to_string(),
                    version: None,
                }],
                script: None,
                script_path: None,
                manual_instructions: None,
            },
            probe: Some(ProbeDefV1 {
                command: "node --version".to_string(),
            }),
        },
        PackageDefV1 {
            version: 1,
            name: "npm".to_string(),
            description: Some("npm CLI via apt.".to_string()),
            runnable: true,
            entrypoints: vec!["npm".to_string(), "npx".to_string()],
            platforms: Vec::new(),
            wrappers: Vec::new(),
            install: InstallDefV1 {
                method: InstallMethodV1::Apt,
                apt: vec![AptSpecV1 {
                    name: "npm".to_string(),
                    version: None,
                }],
                script: None,
                script_path: None,
                manual_instructions: None,
            },
            probe: Some(ProbeDefV1 {
                command: "npm --version && npx --version".to_string(),
            }),
        },
    ]
}

fn builtin_bundles_v1() -> Vec<BundleDefV1> {
    vec![BundleDefV1 {
        version: 1,
        name: "node-runtime".to_string(),
        description: Some("Node.js + npm bundle.".to_string()),
        platforms: Vec::new(),
        packages: vec!["node".to_string(), "npm".to_string()],
    }]
}

pub(crate) fn load_inventory_dir_v1(dir: &Path, platform: HostPlatform) -> Result<InventoryViewV1> {
    let packages_dir = dir.join("packages");
    let bundles_dir = dir.join("bundles");

    let mut view = InventoryViewV1::default();
    if packages_dir.is_dir() {
        let pkgs = load_package_dir_v1(&packages_dir, platform)
            .with_context(|| format!("failed to read {}", packages_dir.display()))?;
        view.packages.extend(pkgs);
    }
    if bundles_dir.is_dir() {
        let bundles = load_bundle_dir_v1(&bundles_dir, platform)
            .with_context(|| format!("failed to read {}", bundles_dir.display()))?;
        view.bundles.extend(bundles);
    }
    view.validate_no_collisions()?;
    Ok(view)
}

fn load_package_dir_v1(
    dir: &Path,
    platform: HostPlatform,
) -> Result<BTreeMap<String, PackageDefV1>> {
    let mut out = BTreeMap::new();
    for entry in fs::read_dir(dir).with_context(|| format!("read_dir {}", dir.display()))? {
        let entry = entry.context("read_dir entry")?;
        let path = entry.path();
        if !is_yaml_file(&path) {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or_else(|| {
                config_model::user_error(format!("invalid package filename: {}", path.display()))
            })?
            .to_string();

        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let parsed: PackageDefV1 = serde_yaml::from_str(&raw).map_err(|err| {
            config_model::user_error(format!(
                "invalid YAML in {}: {}",
                path.display(),
                err.to_string().trim()
            ))
        })?;
        validate_package_v1(&path, &stem, &parsed, platform)?;
        if is_visible_on_platform(&parsed.platforms, platform)? {
            out.insert(parsed.name.clone(), parsed);
        }
    }
    Ok(out)
}

fn load_bundle_dir_v1(dir: &Path, platform: HostPlatform) -> Result<BTreeMap<String, BundleDefV1>> {
    let mut out = BTreeMap::new();
    for entry in fs::read_dir(dir).with_context(|| format!("read_dir {}", dir.display()))? {
        let entry = entry.context("read_dir entry")?;
        let path = entry.path();
        if !is_yaml_file(&path) {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or_else(|| {
                config_model::user_error(format!("invalid bundle filename: {}", path.display()))
            })?
            .to_string();
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let parsed: BundleDefV1 = serde_yaml::from_str(&raw).map_err(|err| {
            config_model::user_error(format!(
                "invalid YAML in {}: {}",
                path.display(),
                err.to_string().trim()
            ))
        })?;
        validate_bundle_v1(&path, &stem, &parsed, platform)?;
        if is_visible_on_platform(&parsed.platforms, platform)? {
            out.insert(parsed.name.clone(), parsed);
        }
    }
    Ok(out)
}

fn is_yaml_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(OsStr::to_str),
        Some("yaml") | Some("yml")
    )
}

fn validate_package_v1(
    path: &Path,
    expected_name: &str,
    pkg: &PackageDefV1,
    platform: HostPlatform,
) -> Result<()> {
    if pkg.version != 1 {
        return Err(config_model::user_error(format!(
            "invalid package schema in {}: version must be 1 (got {})",
            path.display(),
            pkg.version
        )));
    }
    if pkg.name.trim().is_empty() {
        return Err(config_model::user_error(format!(
            "invalid package schema in {}: name must be a non-empty string",
            path.display()
        )));
    }
    if pkg.name != expected_name {
        return Err(config_model::user_error(format!(
            "invalid package schema in {}: name '{}' must match filename '{}.yaml'",
            path.display(),
            pkg.name,
            expected_name
        )));
    }
    if pkg.runnable && pkg.entrypoints.is_empty() {
        return Err(config_model::user_error(format!(
            "invalid package schema in {}: runnable=true requires a non-empty entrypoints list",
            path.display()
        )));
    }
    for entrypoint in &pkg.entrypoints {
        if entrypoint.trim().is_empty() {
            return Err(config_model::user_error(format!(
                "invalid package schema in {}: entrypoints must be non-empty strings",
                path.display()
            )));
        }
    }
    match pkg.install.method {
        InstallMethodV1::Apt => {
            if pkg.install.apt.is_empty() {
                return Err(config_model::user_error(format!(
                    "invalid package schema in {}: install.method=apt requires a non-empty install.apt list",
                    path.display()
                )));
            }
            for spec in &pkg.install.apt {
                if spec.name.trim().is_empty() {
                    return Err(config_model::user_error(format!(
                        "invalid package schema in {}: install.apt[].name must be non-empty",
                        path.display()
                    )));
                }
            }
        }
        InstallMethodV1::Script => {
            if pkg.install.script.is_none() && pkg.install.script_path.is_none() {
                return Err(config_model::user_error(format!(
                    "invalid package schema in {}: install.method=script requires install.script or install.script_path",
                    path.display()
                )));
            }
        }
        InstallMethodV1::Manual => {
            if pkg
                .install
                .manual_instructions
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
            {
                return Err(config_model::user_error(format!(
                    "invalid package schema in {}: install.method=manual requires non-empty install.manual_instructions",
                    path.display()
                )));
            }
        }
    }
    for wrapper in &pkg.wrappers {
        if wrapper.name.trim().is_empty() {
            return Err(config_model::user_error(format!(
                "invalid package schema in {}: wrappers[].name must be non-empty",
                path.display()
            )));
        }
        match &wrapper.kind {
            WrapperKindV1::BashFunction {
                bash_source,
                function,
            } => {
                if bash_source.trim().is_empty() || function.trim().is_empty() {
                    return Err(config_model::user_error(format!(
                        "invalid package schema in {}: wrappers[].kind=bash_function requires non-empty bash_source and function",
                        path.display()
                    )));
                }
            }
            WrapperKindV1::BashSourceExec { bash_source, exec } => {
                if bash_source.trim().is_empty() || exec.trim().is_empty() {
                    return Err(config_model::user_error(format!(
                        "invalid package schema in {}: wrappers[].kind=bash_source_exec requires non-empty bash_source and exec",
                        path.display()
                    )));
                }
            }
            WrapperKindV1::ShEnvExec { exec, .. } => {
                if exec.trim().is_empty() {
                    return Err(config_model::user_error(format!(
                        "invalid package schema in {}: wrappers[].kind=sh_env_exec requires non-empty exec",
                        path.display()
                    )));
                }
            }
        }
    }

    let _ = is_visible_on_platform(&pkg.platforms, platform)?;
    Ok(())
}

fn validate_bundle_v1(
    path: &Path,
    expected_name: &str,
    bundle: &BundleDefV1,
    platform: HostPlatform,
) -> Result<()> {
    if bundle.version != 1 {
        return Err(config_model::user_error(format!(
            "invalid bundle schema in {}: version must be 1 (got {})",
            path.display(),
            bundle.version
        )));
    }
    if bundle.name.trim().is_empty() {
        return Err(config_model::user_error(format!(
            "invalid bundle schema in {}: name must be a non-empty string",
            path.display()
        )));
    }
    if bundle.name != expected_name {
        return Err(config_model::user_error(format!(
            "invalid bundle schema in {}: name '{}' must match filename '{}.yaml'",
            path.display(),
            bundle.name,
            expected_name
        )));
    }
    if bundle.packages.is_empty() || bundle.packages.iter().any(|p| p.trim().is_empty()) {
        return Err(config_model::user_error(format!(
            "invalid bundle schema in {}: packages must be a non-empty list of non-empty strings",
            path.display()
        )));
    }
    let _ = is_visible_on_platform(&bundle.platforms, platform)?;
    Ok(())
}

fn is_visible_on_platform(platforms: &[String], platform: HostPlatform) -> Result<bool> {
    if platforms.is_empty() {
        return Ok(true);
    }
    let mut parsed = Vec::with_capacity(platforms.len());
    for raw in platforms {
        let Some(p) = parse_platform(raw) else {
            return Err(config_model::user_error(format!(
                "invalid platforms entry '{raw}'; expected one of: linux, macos, windows"
            )));
        };
        parsed.push(p);
    }
    Ok(platform.matches(&parsed))
}

pub(crate) fn merge_inventory_layer_v1(into: &mut InventoryViewV1, layer: InventoryViewV1) {
    for (name, pkg) in layer.packages {
        into.packages.insert(name, pkg);
    }
    for (name, bundle) in layer.bundles {
        into.bundles.insert(name, bundle);
    }
}

pub(crate) fn summarize_inventory_v1(view: &InventoryViewV1) -> Vec<InventoryListItemSummaryV1> {
    let mut out = Vec::new();
    for pkg in view.packages.values() {
        out.push(InventoryListItemSummaryV1 {
            kind: "package".to_string(),
            name: pkg.name.clone(),
            runnable: Some(pkg.runnable),
            method: Some(pkg.install.method.clone()),
            entrypoints: pkg.entrypoints.clone(),
            platforms: pkg.platforms.clone(),
            description: pkg.description.clone(),
        });
    }
    for bundle in view.bundles.values() {
        out.push(InventoryListItemSummaryV1 {
            kind: "bundle".to_string(),
            name: bundle.name.clone(),
            runnable: None,
            method: None,
            entrypoints: Vec::new(),
            platforms: bundle.platforms.clone(),
            description: bundle.description.clone(),
        });
    }
    out.sort_by(|a, b| (a.kind.as_str(), a.name.as_str()).cmp(&(b.kind.as_str(), b.name.as_str())));
    out
}

pub(crate) fn find_workspace_inventory_chain(cwd: &Path, stop_at: Option<&Path>) -> Vec<PathBuf> {
    let mut layers = Vec::new();
    for dir in cwd.ancestors() {
        let candidate = dir.join(".substrate").join("deps");
        if candidate.is_dir() {
            layers.push(candidate);
        }
        if stop_at.is_some_and(|stop| stop == dir) {
            break;
        }
    }
    layers.reverse();
    layers
}
