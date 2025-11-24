use crate::context::{world_features_enabled, ShimContext};
use crate::logger::{format_timestamp, get_shim_fingerprint, write_log_entry};
use anyhow::Error;
use serde_json::json;
use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use substrate_common::{
    manager_manifest::{ManagerManifest, ManagerSpec, Platform, RegexPattern},
    paths,
};
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

pub(crate) fn collect_world_telemetry(span_id: &str) -> (Vec<String>, Option<FsDiff>) {
    let world_id = match env::var("SUBSTRATE_WORLD_ID") {
        Ok(id) => id,
        Err(_) => {
            return (vec![], None);
        }
    };

    if let Ok(backend) = world_backend_factory::factory() {
        let handle = world_api::WorldHandle {
            id: world_id.clone(),
        };

        let fs_diff = match backend.fs_diff(&handle, span_id) {
            Ok(diff) => Some(diff),
            Err(e) => {
                eprintln!("Warning: Failed to collect fs_diff: {}", e);
                None
            }
        };

        let scopes_used = vec![];
        (scopes_used, fs_diff)
    } else {
        (vec![], None)
    }
}

pub(crate) struct ManagerHintEngine {
    rules: Vec<ManagerHintRule>,
    emitted: HashSet<String>,
}

impl ManagerHintEngine {
    pub(crate) fn new() -> Option<Self> {
        if hints_disabled() {
            return None;
        }

        let force_hints = env::var("SUBSTRATE_SHIM_HINTS").is_ok();
        if !force_hints && !world_features_enabled() {
            return None;
        }

        let (base, overlay) = manifest_paths()?;
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

fn manifest_paths() -> Option<(PathBuf, Option<PathBuf>)> {
    if let Ok(override_path) = env::var("SUBSTRATE_MANAGER_MANIFEST") {
        return Some((PathBuf::from(override_path), manifest_overlay_path()));
    }

    if let Ok(home) = paths::substrate_home() {
        let base = home.join("manager_hooks.yaml");
        if base.exists() {
            return Some((base, Some(home.join("manager_hooks.local.yaml"))));
        }
    }

    let fallback = repo_manifest_path();
    if fallback.exists() {
        Some((fallback, manifest_overlay_path()))
    } else {
        None
    }
}

fn manifest_overlay_path() -> Option<PathBuf> {
    paths::substrate_home()
        .ok()
        .map(|home| home.join("manager_hooks.local.yaml"))
}

fn repo_manifest_path() -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .and_then(|dir| dir.parent())
        .map(|root| root.join("config").join("manager_hooks.yaml"))
        .unwrap_or_else(|| PathBuf::from("config/manager_hooks.yaml"))
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

    #[test]
    #[serial]
    fn manager_hint_engine_matches_once_and_dedupes() {
        let temp = TempDir::new().unwrap();
        let manifest_path = temp.path().join("manager_hooks.yaml");
        fs::write(
            &manifest_path,
            r#"version: 1
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

        let _manifest_guard = EnvGuard::set(
            "SUBSTRATE_MANAGER_MANIFEST",
            manifest_path.to_string_lossy(),
        );
        let _world_guard = EnvGuard::set("SUBSTRATE_WORLD", "enabled");
        let _hints_guard = EnvGuard::set("SUBSTRATE_SHIM_HINTS", "1");

        let mut engine = ManagerHintEngine::new().expect("hint engine should load");
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
            r#"version: 1
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

        let _manifest_guard = EnvGuard::set(
            "SUBSTRATE_MANAGER_MANIFEST",
            manifest_path.to_string_lossy(),
        );
        let _world_guard = EnvGuard::set("SUBSTRATE_WORLD", "enabled");
        let _hints_guard = EnvGuard::set("SUBSTRATE_SHIM_HINTS", "0");

        assert!(ManagerHintEngine::new().is_none());
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
