//! Probe the Codex binary for version/features and gate optional flags.
//!
//! This example runs `codex --version` and `codex features list` (if available) and then
//! demonstrates gating streaming/logging/artifact flags plus MCP/app-server flows. If the binary
//! is missing, it falls back to sample capability data. Set `CODEX_BINARY` to override the binary
//! path.
//!
//! Example:
//! ```bash
//! cargo run -p unified-agent-api-codex --example feature_detection
//! CODEX_BINARY=/opt/codex-nightly cargo run -p unified-agent-api-codex --example feature_detection
//! ```

use std::{
    collections::HashMap,
    env,
    error::Error,
    path::Path,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use serde_json::json;
use tokio::process::Command;
use toml::Value as TomlValue;

const MANIFEST_FALLBACK: &[(&str, &[&str])] = &[
    // Observed to work on 0.61.0 even though `features list` omits them.
    (
        "0.61.0",
        &[
            "json-stream",
            "output-last-message",
            "output-schema",
            "diff",
            "apply",
            "resume",
            "app-server",
            "mcp-server",
        ],
    ),
];

#[derive(Debug, Clone)]
struct Capability {
    version: Option<Version>,
    features: Vec<String>,
    manifest_source: Option<String>,
    forced: Vec<String>,
    advertised_allow: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

static CAPABILITY_CACHE: OnceLock<Mutex<HashMap<PathBuf, Capability>>> = OnceLock::new();

impl Version {
    fn parse(raw: &str) -> Option<Self> {
        let tokens: Vec<&str> = raw.split(|c: char| c.is_whitespace() || c == '-').collect();
        let version_str = tokens.iter().find(|token| {
            token
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        })?;
        let parts: Vec<&str> = version_str.split('.').collect();
        if parts.len() < 2 {
            return None;
        }
        let major = parts.first()?.parse().ok()?;
        let minor = parts.get(1)?.parse().ok()?;
        let patch = parts.get(2).unwrap_or(&"0").parse().ok()?;
        Some(Self {
            major,
            minor,
            patch,
        })
    }

    fn as_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let json_output = take_flag(&mut args, "--json");

    let binary = resolve_binary();
    let (capability, cached) = if binary_exists(&binary) {
        cached_probe(&binary).await
    } else {
        eprintln!(
            "Binary not found at {}. Using sample capability set.",
            binary.display()
        );
        (sample_capability(), false)
    };

    if json_output {
        let report = json!({
            "binary": binary.display().to_string(),
            "cached": cached,
            "version": capability.version.as_ref().map(|v| v.as_string()),
            "features": capability.features,
            "manifest_source": capability.manifest_source,
            "forced": capability.forced,
            "advertised_allow": capability.advertised_allow,
        });
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        if let Some(version) = capability.version.as_ref() {
            println!("Detected Codex version: {}", version.as_string());
        } else {
            println!("Version unknown (could not parse output)");
        }
        if cached {
            println!("Capabilities served from cache for {}", binary.display());
        }
        println!("Features: {}", capability.features.join(", "));
        if let Some(source) = capability.manifest_source.as_ref() {
            println!("Manifest source: {source}");
        }
        if !capability.forced.is_empty() {
            println!("Forced: {}", capability.forced.join(", "));
        }
        if let Some(allow) = capability.advertised_allow.as_ref() {
            println!("Advertised allowlist: {}", allow.join(", "));
        }
        println!(
            "Cache scope: per binary path for this process; refresh probes after upgrading the binary."
        );
    }

    Ok(())
}

async fn cached_probe(binary: &Path) -> (Capability, bool) {
    let cache = CAPABILITY_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(existing) = cache.lock().unwrap().get(binary) {
        return (existing.clone(), true);
    }

    let capability = probe_capabilities(binary).await;
    cache
        .lock()
        .unwrap()
        .insert(binary.to_path_buf(), capability.clone());
    (capability, false)
}

async fn probe_capabilities(binary: &Path) -> Capability {
    let version = run_version(binary)
        .await
        .and_then(|raw| Version::parse(&raw));
    let mut manifest_source = None;
    let mut forced = Vec::new();
    let mut advertised_allow: Option<Vec<String>> = None;
    let mut features = run_features(binary)
        .await
        .unwrap_or_else(|| vec!["json-stream".into(), "output-last-message".into()]);

    // Apply manifest hints when the binary reports a known version.
    if let Some(version) = version.as_ref().map(|v| v.as_string()) {
        if let Some((manifest_features, source)) = manifest_for(&version) {
            manifest_source = Some(source);
            // Manifest acts as the authoritative advertised set for this version.
            features = manifest_features;
        }
    }

    // Environment override: CODEX_FEATURE_FORCE=feature1,feature2
    if let Ok(force) = env::var("CODEX_FEATURE_FORCE") {
        forced = force
            .split(',')
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        if !forced.is_empty() {
            println!("Applying forced feature list from CODEX_FEATURE_FORCE");
            features.extend(forced.clone());
        }
    }

    // Optional allowlist: CODEX_FEATURE_ADVERTISE=feature1,feature2 restricts the advertised set.
    if let Ok(allow) = env::var("CODEX_FEATURE_ADVERTISE") {
        let allow_list: Vec<String> = allow
            .split(',')
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        if !allow_list.is_empty() {
            println!("Restricting advertised features to CODEX_FEATURE_ADVERTISE");
            advertised_allow = Some(allow_list.clone());
            features.retain(|f| {
                let norm = normalize(f);
                allow_list.contains(&norm)
            });
        }
    }

    // Deduplicate by normalized name.
    let mut seen = std::collections::HashSet::new();
    features.retain(|f| seen.insert(normalize(f)));

    Capability {
        version,
        features,
        manifest_source,
        forced,
        advertised_allow,
    }
}

fn sample_capability() -> Capability {
    Capability {
        version: Some(Version {
            major: 1,
            minor: 4,
            patch: 0,
        }),
        features: vec![
            "json-stream".into(),
            "output-last-message".into(),
            "output-schema".into(),
            "diff".into(),
            "apply".into(),
            "resume".into(),
            "app-server".into(),
            "mcp-server".into(),
        ],
        manifest_source: Some("sample".into()),
        forced: Vec::new(),
        advertised_allow: None,
    }
}

async fn run_version(binary: &Path) -> Option<String> {
    Command::new(binary)
        .arg("--version")
        .output()
        .await
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
}

async fn run_features(binary: &Path) -> Option<Vec<String>> {
    let output = Command::new(binary)
        .args(["features", "list"])
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let mut features = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        features.push(normalize(trimmed));
    }
    Some(features)
}

fn normalize(feature: &str) -> String {
    feature
        .split(|c: char| c.is_whitespace() || c == ':' || c == '=')
        .next()
        .unwrap_or(feature)
        .to_ascii_lowercase()
}

fn resolve_binary() -> PathBuf {
    env::var_os("CODEX_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("codex"))
}

fn binary_exists(path: &Path) -> bool {
    if path.is_absolute() || path.components().count() > 1 {
        std::fs::metadata(path).is_ok()
    } else {
        env::var_os("PATH")
            .and_then(|paths| {
                env::split_paths(&paths)
                    .map(|dir| dir.join(path))
                    .find(|candidate| std::fs::metadata(candidate).is_ok())
            })
            .is_some()
    }
}

fn manifest_for(version: &str) -> Option<(Vec<String>, String)> {
    // Env override for a manifest path; default to ./feature_manifest.toml if present.
    if let Some((from_file, path)) = load_manifest_file(version) {
        return Some((from_file, path));
    }

    MANIFEST_FALLBACK
        .iter()
        .find(|(v, _)| *v == version)
        .map(|(_, feats)| {
            (
                feats.iter().map(|f| f.to_string()).collect(),
                "builtin".to_string(),
            )
        })
}

fn load_manifest_file(version: &str) -> Option<(Vec<String>, String)> {
    let path = env::var_os("CODEX_FEATURE_MANIFEST")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("feature_manifest.toml"));

    if !path.exists() {
        return None;
    }

    let contents = std::fs::read_to_string(&path).ok()?;
    let toml = contents.parse::<TomlValue>().ok()?;
    let versions = toml.get("versions")?;
    let Some(table) = versions.as_table() else {
        eprintln!("feature_manifest.toml: expected [versions] table mapping version -> [features]");
        return None;
    };

    let features = table.get(version)?.as_array()?;
    let collected = features
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();
    if collected.is_empty() {
        eprintln!(
            "feature manifest at {} lists {} but has no features",
            path.display(),
            version
        );
        None
    } else {
        Some((collected, path.display().to_string()))
    }
}

fn take_flag(args: &mut Vec<String>, flag: &str) -> bool {
    let before = args.len();
    args.retain(|value| value != flag);
    before != args.len()
}
