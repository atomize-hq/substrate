use super::*;

use std::collections::{BTreeMap, HashMap};
use std::env;
use std::ffi::OsString;

struct RestoreEnvVar {
    key: &'static str,
    original: Option<OsString>,
}

impl RestoreEnvVar {
    fn capture(key: &'static str) -> Self {
        Self {
            key,
            original: env::var_os(key),
        }
    }

    fn set(&self, value: impl Into<OsString>) {
        env::set_var(self.key, value.into());
    }

    fn remove(&self) {
        env::remove_var(self.key);
    }
}

impl Drop for RestoreEnvVar {
    fn drop(&mut self) {
        match self.original.take() {
            Some(value) => env::set_var(self.key, value),
            None => env::remove_var(self.key),
        }
    }
}

fn parse_env_snapshot(snapshot: &str) -> HashMap<String, String> {
    snapshot
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
        .collect()
}

#[cfg(unix)]
#[tokio::test]
async fn stream_resume_env_overrides_apply_after_wrapper_injection_and_do_not_mutate_parent() {
    let _guard = env_guard_async().await;

    let codex_home_restore = RestoreEnvVar::capture("CODEX_HOME");
    let rust_log_restore = RestoreEnvVar::capture("RUST_LOG");
    codex_home_restore.set("parent-home");
    rust_log_restore.remove();

    let temp = tempfile::tempdir().unwrap();
    let script_path = write_fake_codex(
        temp.path(),
        r#"#!/usr/bin/env bash
set -euo pipefail

out=""
for ((i=1; i<=$#; i++)); do
  arg="${!i}"
  if [[ "$arg" == "--output-last-message" ]]; then
    j=$((i+1))
    out="${!j}"
    break
  fi
done

if [[ -z "${out}" ]]; then
  echo "missing --output-last-message" >&2
  exit 2
fi

mkdir -p "$(dirname "$out")"
{
  echo "CODEX_HOME=${CODEX_HOME:-missing}"
  echo "RUST_LOG=${RUST_LOG:-missing}"
  echo "FOO=${FOO:-missing}"
} > "$out"
exit 0
"#,
    );

    let injected_home = temp.path().join("injected-home");
    let override_home = temp.path().join("override-home");

    let client = CodexClient::builder()
        .binary(&script_path)
        .codex_home(&injected_home)
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let mut overrides = BTreeMap::new();
    overrides.insert(
        "CODEX_HOME".to_string(),
        override_home.to_string_lossy().to_string(),
    );
    overrides.insert("RUST_LOG".to_string(), "trace".to_string());
    overrides.insert("FOO".to_string(), "bar".to_string());

    let ExecStreamControl { completion, .. } = client
        .stream_resume_with_env_overrides_control(ResumeRequest::last().prompt("hello"), &overrides)
        .await
        .unwrap();

    let completion = completion.await.unwrap();
    let snapshot = completion.last_message.expect("fake codex wrote snapshot");
    let parsed = parse_env_snapshot(&snapshot);

    assert_eq!(
        parsed.get("CODEX_HOME"),
        Some(&override_home.to_string_lossy().to_string())
    );
    assert_eq!(parsed.get("RUST_LOG"), Some(&"trace".to_string()));
    assert_eq!(parsed.get("FOO"), Some(&"bar".to_string()));

    assert_eq!(
        env::var_os("CODEX_HOME"),
        Some(OsString::from("parent-home"))
    );
    assert_eq!(env::var_os("RUST_LOG"), None);
}
