#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use codex::{CodexAuthMethod, CodexAuthStatus, CodexClient, CodexLogoutStatus};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct EnvSnapshot {
    #[serde(rename = "CODEX_BINARY")]
    codex_binary: Option<String>,
    #[serde(rename = "CODEX_HOME")]
    codex_home: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Invocation {
    binary: String,
    argv: Vec<String>,
    env: EnvSnapshot,
}

#[tokio::test]
async fn applies_env_overrides_across_spawn_sites() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let log_path = temp.path().join("invocations.jsonl");
    let fake_codex = write_fake_codex(&log_path)?;
    let codex_home = temp.path().join("codex_home");

    let client = CodexClient::builder()
        .binary(&fake_codex)
        .codex_home(&codex_home)
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let prompt = "hello";
    let exec_output = client.send_prompt(prompt).await?;
    assert_eq!(exec_output, "exec-ok");

    let status = client.login_status().await?;
    assert_eq!(status, CodexAuthStatus::LoggedIn(CodexAuthMethod::ChatGpt));

    let logout = client.logout().await?;
    assert_eq!(logout, CodexLogoutStatus::LoggedOut);

    let login_child = client.spawn_login_process()?;
    let login_output = login_child.wait_with_output().await?;
    assert!(login_output.status.success());

    assert!(codex_home.is_dir());
    assert!(codex_home.join("conversations").is_dir());
    assert!(codex_home.join("logs").is_dir());

    let invocations = read_invocations(&log_path)?;
    assert_eq!(invocations.len(), 4);

    let expected_binary = fake_codex.to_string_lossy().to_string();
    let expected_home = codex_home.to_string_lossy().to_string();

    for invocation in &invocations {
        assert_eq!(invocation.binary, expected_binary);
        assert_eq!(
            invocation.env.codex_binary.as_deref(),
            Some(expected_binary.as_str())
        );
        assert_eq!(
            invocation.env.codex_home.as_deref(),
            Some(expected_home.as_str())
        );
    }

    let exec_invocation = find_invocation(&invocations, |inv| {
        inv.argv.first().map(|arg| arg == "exec").unwrap_or(false)
    });
    assert!(
        exec_invocation.argv.contains(&prompt.to_string()),
        "prompt missing from exec args: {:?}",
        exec_invocation.argv
    );

    let login_status_invocation = find_invocation(&invocations, |inv| {
        inv.argv.len() >= 2 && inv.argv[0] == "login" && inv.argv[1] == "status"
    });
    assert_eq!(login_status_invocation.argv[0], "login");
    assert_eq!(login_status_invocation.argv[1], "status");

    let logout_invocation = find_invocation(&invocations, |inv| inv.argv == ["logout"]);
    assert_eq!(logout_invocation.argv, ["logout"]);

    let login_invocation = find_invocation(&invocations, |inv| inv.argv == ["login"]);
    assert_eq!(login_invocation.argv, ["login"]);

    Ok(())
}

fn write_fake_codex(log_path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let script_path = log_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("fake_codex.sh");
    let script = format!(
        r#"#!/usr/bin/env bash
set -euo pipefail

LOG_PATH="{log}"

python3 - "$LOG_PATH" "$0" "$@" <<'PY'
import json
import os
import sys

log_path = sys.argv[1]
binary = sys.argv[2]
argv = sys.argv[3:]

entry = {{
    'binary': binary,
    'argv': argv,
    'env': {{
        'CODEX_BINARY': os.environ.get('CODEX_BINARY'),
        'CODEX_HOME': os.environ.get('CODEX_HOME'),
    }},
}}

with open(log_path, 'a', encoding='utf-8') as handle:
    handle.write(json.dumps(entry))
    handle.write('\n')
PY

if [[ $# -ge 2 && $1 == "login" && $2 == "status" ]]; then
  echo "Logged in using ChatGPT"
elif [[ $# -ge 1 && $1 == "logout" ]]; then
  echo "Successfully logged out"
elif [[ $# -ge 1 && $1 == "login" ]]; then
  echo "Login helper"
elif [[ $# -ge 1 && $1 == "exec" ]]; then
  echo "exec-ok"
else
  echo "unknown command: $@" >&2
  exit 1
fi
"#,
        log = log_path.display()
    );

    fs::write(&script_path, script)?;
    let mut permissions = fs::metadata(&script_path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script_path, permissions)?;
    Ok(script_path)
}

fn read_invocations(log_path: &Path) -> Result<Vec<Invocation>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(log_path)?;
    let mut invocations = Vec::new();
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        invocations.push(serde_json::from_str(line)?);
    }
    Ok(invocations)
}

fn find_invocation<F>(invocations: &[Invocation], predicate: F) -> &Invocation
where
    F: Fn(&Invocation) -> bool,
{
    invocations
        .iter()
        .find(|inv| predicate(inv))
        .unwrap_or_else(|| panic!("missing invocation matching predicate"))
}
