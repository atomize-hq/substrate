use crate::context::{world_features_enabled, ShimContext};
use crate::logger::{format_timestamp, get_shim_fingerprint, write_log_entry};
use anyhow::Error;
use serde_json::json;
use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use substrate_common::manager_manifest::{ManagerManifest, ManagerSpec, Platform, RegexPattern};
use transport_api_types::{InstallBootstrapContextCarrierV1, PlatformBootstrapMappingV1};
use world_api::FsDiff;

pub(crate) fn log_spawn_failure(
    ctx: &ShimContext,
    real_binary: &Path,
    timestamp: SystemTime,
    error: &Error,
) {
    if let Some(log_path) = &ctx.log_file {
        let spawn_error = error.downcast_ref::<std::io::Error>();
        let mut error_entry = json!({
            "ts": format_timestamp(timestamp),
            "command": ctx.command_name,
            "resolved_path": real_binary.display().to_string(),
            "error": "spawn_failed",
            "depth": ctx.depth,
            "session_id": ctx.session_id,
            "shim_fingerprint": get_shim_fingerprint()
        });

        if let Some(io_err) = spawn_error {
            error_entry["spawn_error_kind"] = json!(format!("{:?}", io_err.kind()));
            if let Some(errno) = io_err.raw_os_error() {
                error_entry["spawn_errno"] = json!(errno);
            }
        }

        let _ = write_log_entry(log_path, &error_entry);
    }
}

pub(crate) fn collect_world_telemetry(
    span_id: &str,
    world_id: &str,
    host_carrier: &InstallBootstrapContextCarrierV1,
    platform_bootstrap_mapping: Option<&PlatformBootstrapMappingV1>,
    project_path: Option<&Path>,
) -> (Vec<String>, Option<FsDiff>) {
    #[cfg(any(target_os = "macos", windows))]
    if platform_bootstrap_mapping.is_none() {
        eprintln!(
            "Warning: Failed to collect authenticated world telemetry: platform bootstrap mapping unavailable"
        );
        return (vec![], None);
    }

    let backend = match world_backend_factory::factory_with_platform_bootstrap(
        Some(host_carrier),
        platform_bootstrap_mapping,
        project_path,
    ) {
        Ok(backend) => backend,
        Err(err) => {
            eprintln!("Warning: Failed to build world telemetry backend: {err}");
            return (vec![], None);
        }
    };

    let handle = world_api::WorldHandle {
        id: world_id.to_string(),
        shared_binding: None,
    };

    let fs_diff = match backend.fs_diff(&handle, span_id) {
        Ok(diff) => Some(diff),
        Err(err) => {
            eprintln!("Warning: Failed to collect fs_diff: {err}");
            None
        }
    };

    (vec![], fs_diff)
}

pub(crate) struct ManagerHintEngine {
    rules: Vec<ManagerHintRule>,
    emitted: HashSet<String>,
}

impl ManagerHintEngine {
    pub(crate) fn new(host_carrier: &InstallBootstrapContextCarrierV1) -> Option<Self> {
        if hints_disabled() {
            return None;
        }

        let force_hints = env::var("SUBSTRATE_SHIM_HINTS").is_ok();
        if !force_hints && !world_features_enabled() {
            return None;
        }

        let (base, overlay) = manifest_paths(host_carrier)?;
        let manifest = ManagerManifest::load(&base, overlay.as_deref()).ok()?;
        let specs = manifest.resolve_for_platform(current_platform());

        let mut rules = Vec::new();
        for spec in specs {
            if let Some(rule) = ManagerHintRule::from_spec(&spec) {
                rules.push(rule);
            }
        }

        if rules.is_empty() {
            None
        } else {
            Some(Self {
                rules,
                emitted: HashSet::new(),
            })
        }
    }

    pub(crate) fn is_active(&self) -> bool {
        !self.rules.is_empty()
    }

    pub(crate) fn evaluate(&mut self, stderr: &[u8]) -> Option<HintMatch> {
        let stderr_text = String::from_utf8_lossy(stderr);
        for rule in &self.rules {
            if self.emitted.contains(&rule.key) {
                continue;
            }
            if let Some(pattern) = rule.matches(&stderr_text) {
                self.emitted.insert(rule.key.clone());
                return Some(HintMatch {
                    manager_name: rule.name.clone(),
                    hint: rule.hint.clone(),
                    pattern,
                });
            }
        }
        None
    }
}

pub(crate) struct ManagerHintRule {
    name: String,
    key: String,
    hint: String,
    patterns: Vec<RegexPattern>,
}

impl ManagerHintRule {
    fn from_spec(spec: &ManagerSpec) -> Option<Self> {
        let hint = spec.repair_hint.as_ref()?.trim();
        if hint.is_empty() || spec.errors.is_empty() {
            return None;
        }

        Some(Self {
            name: spec.name.clone(),
            key: spec.name.to_ascii_lowercase(),
            hint: hint.to_string(),
            patterns: spec.errors.clone(),
        })
    }

    fn matches(&self, stderr: &str) -> Option<String> {
        for pattern in &self.patterns {
            if pattern.regex.is_match(stderr) {
                return Some(pattern.pattern.clone());
            }
        }
        None
    }
}

pub(crate) struct HintMatch {
    pub(crate) manager_name: String,
    pub(crate) hint: String,
    pub(crate) pattern: String,
}

pub(crate) fn hint_payload(match_info: &HintMatch) -> serde_json::Value {
    json!({
        "name": match_info.manager_name,
        "hint": match_info.hint,
        "pattern": match_info.pattern,
        "ts": format_timestamp(SystemTime::now())
    })
}

fn hints_disabled() -> bool {
    match env::var("SUBSTRATE_SHIM_HINTS") {
        Ok(value) => disabled_flag(&value),
        Err(_) => false,
    }
}

fn disabled_flag(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "0" | "false" | "off" | "disabled"
    )
}

fn manifest_paths(
    host_carrier: &InstallBootstrapContextCarrierV1,
) -> Option<(PathBuf, Option<PathBuf>)> {
    host_carrier.validate().ok()?;
    let base = repo_manifest_path(host_carrier);
    if !base.exists() {
        return None;
    }
    Some((base, manifest_overlay_path(host_carrier)))
}

fn manifest_overlay_path(host_carrier: &InstallBootstrapContextCarrierV1) -> Option<PathBuf> {
    host_carrier.validate().ok()?;
    Some(PathBuf::from(&host_carrier.context.selected_host_prefix).join("manager_hooks.local.yaml"))
}

fn repo_manifest_path(host_carrier: &InstallBootstrapContextCarrierV1) -> PathBuf {
    PathBuf::from(&host_carrier.context.selected_host_prefix).join("manager_hooks.yaml")
}

fn current_platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::MacOs
    } else if cfg!(windows) {
        Platform::Windows
    } else {
        Platform::Linux
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{ShimContext, TRACE_LOG_VAR};
    use serial_test::serial;
    use std::time::SystemTime;
    use std::{env, fs};
    use tempfile::TempDir;
    use transport_api_types::{InstallBootstrapContextCarrierV1, InstallBootstrapContextV1};

    struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: impl Into<String>) -> Self {
            let previous = env::var(key).ok();
            env::set_var(key, value.into());
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(value) = self.previous.take() {
                env::set_var(self.key, value);
            } else {
                env::remove_var(self.key);
            }
        }
    }

    fn test_host_carrier(prefix: &Path) -> InstallBootstrapContextCarrierV1 {
        #[cfg(unix)]
        {
            InstallBootstrapContextCarrierV1::from_context(
                InstallBootstrapContextV1::new_unix(prefix.to_str().unwrap(), "alice", 1000)
                    .unwrap(),
            )
            .unwrap()
        }

        #[cfg(windows)]
        {
            InstallBootstrapContextCarrierV1::from_context(
                InstallBootstrapContextV1::new_windows(
                    prefix.to_str().unwrap(),
                    r"ACME\Alice",
                    "S-1-5-21-1000",
                )
                .unwrap(),
            )
            .unwrap()
        }
    }

    #[test]
    #[serial]
    fn manager_hint_engine_matches_once_and_dedupes() {
        let temp = TempDir::new().unwrap();
        let manifest_path = temp.path().join("manager_hooks.yaml");
        fs::write(
            &manifest_path,
            r#"version: 2
managers:
  - name: Tool
    detect: {}
    init: {}
    errors:
      - "tool: command not found"
    repair_hint: "install tool"
"#,
        )
        .unwrap();

        let _manifest_guard = EnvGuard::set("SUBSTRATE_MANAGER_MANIFEST", "/tmp/ambient-B.yaml");
        let _world_guard = EnvGuard::set("SUBSTRATE_WORLD", "enabled");
        let _hints_guard = EnvGuard::set("SUBSTRATE_SHIM_HINTS", "1");
        let host_carrier = test_host_carrier(temp.path());

        let mut engine = ManagerHintEngine::new(&host_carrier).expect("hint engine should load");
        assert!(engine.is_active());

        let first = engine
            .evaluate(b"tool: command not found")
            .expect("first match should emit hint");
        assert_eq!(first.pattern, "tool: command not found");
        assert!(engine.evaluate(b"tool: command not found").is_none());
    }

    #[test]
    #[serial]
    fn manager_hint_engine_respects_disable_flag() {
        let temp = TempDir::new().unwrap();
        let manifest_path = temp.path().join("manager_hooks.yaml");
        fs::write(
            &manifest_path,
            r#"version: 2
managers:
  - name: Disabled
    detect: {}
    init: {}
    errors:
      - "disabled: command not found"
    repair_hint: "noop"
"#,
        )
        .unwrap();

        let _manifest_guard = EnvGuard::set("SUBSTRATE_MANAGER_MANIFEST", "/tmp/ambient-B.yaml");
        let _world_guard = EnvGuard::set("SUBSTRATE_WORLD", "enabled");
        let _hints_guard = EnvGuard::set("SUBSTRATE_SHIM_HINTS", "0");
        let host_carrier = test_host_carrier(temp.path());

        assert!(ManagerHintEngine::new(&host_carrier).is_none());
    }

    #[test]
    fn manifest_paths_use_authenticated_prefix_only() {
        let temp = TempDir::new().unwrap();
        let host_carrier = test_host_carrier(temp.path());
        let ambient = temp.path().join("ambient").join("manager_hooks.yaml");
        fs::create_dir_all(ambient.parent().unwrap()).unwrap();
        fs::write(&ambient, "version: 2\nmanagers: []\n").unwrap();
        fs::write(
            temp.path().join("manager_hooks.yaml"),
            "version: 2\nmanagers: []\n",
        )
        .unwrap();
        let _manifest_guard = EnvGuard::set(
            "SUBSTRATE_MANAGER_MANIFEST",
            ambient.to_string_lossy().to_string(),
        );

        let (base, overlay) = manifest_paths(&host_carrier).expect("manifest paths");
        assert_eq!(base, temp.path().join("manager_hooks.yaml"));
        assert_eq!(overlay, Some(temp.path().join("manager_hooks.local.yaml")));
    }

    #[test]
    #[serial]
    fn spawn_failures_emit_log_entries() {
        let temp = TempDir::new().unwrap();
        let trace_path = temp.path().join("trace.jsonl");
        fs::create_dir_all(trace_path.parent().unwrap()).unwrap();

        let _trace_guard = EnvGuard::set(TRACE_LOG_VAR, trace_path.to_string_lossy());
        let ctx = ShimContext {
            command_name: "missing".to_string(),
            shim_dir: temp.path().to_path_buf(),
            search_paths: Vec::new(),
            log_file: Some(trace_path.clone()),
            session_id: "session".to_string(),
            depth: 3,
        };

        let io_err = std::io::Error::from(std::io::ErrorKind::NotFound);
        let errno = io_err.raw_os_error();
        let log_error: anyhow::Error = io_err.into();
        log_spawn_failure(
            &ctx,
            Path::new("/tmp/missing"),
            SystemTime::now(),
            &log_error,
        );

        let contents = fs::read_to_string(&trace_path).expect("trace log should be created");
        let last_line = contents.lines().last().expect("log entry missing");
        let value: serde_json::Value = serde_json::from_str(last_line).expect("valid json log");

        assert_eq!(
            value.get("error").and_then(|v| v.as_str()),
            Some("spawn_failed")
        );
        assert_eq!(
            value.get("command").and_then(|v| v.as_str()),
            Some("missing")
        );
        assert_eq!(
            value.get("spawn_error_kind").and_then(|v| v.as_str()),
            Some("NotFound")
        );
        assert_eq!(
            value.get("spawn_errno").and_then(|v| v.as_i64()),
            errno.map(|code| code as i64)
        );
        assert_eq!(value.get("depth").and_then(|v| v.as_u64()), Some(3));
    }
}
