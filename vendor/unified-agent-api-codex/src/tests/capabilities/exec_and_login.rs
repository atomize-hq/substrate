use super::*;

#[tokio::test]
async fn exec_applies_guarded_flags_when_supported() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let temp = tempfile::tempdir().unwrap();
    let log_path = temp.path().join("exec.log");
    let script = format!(
        r#"#!/bin/bash
log="{log}"
if [[ "$1" == "--version" ]]; then
  echo "codex 1.2.3"
elif [[ "$1" == "features" && "$2" == "list" && "$3" == "--json" ]]; then
  echo '{{"features":["output_schema","add_dir","mcp_login"]}}'
elif [[ "$1" == "features" && "$2" == "list" ]]; then
  echo "output_schema add_dir login --mcp"
elif [[ "$1" == "--help" ]]; then
  echo "Usage: codex --output-schema add-dir login --mcp"
elif [[ "$1" == "exec" ]]; then
  echo "$@" >> "$log"
  echo "ok"
fi
"#,
        log = log_path.display()
    );
    let binary = write_fake_codex(temp.path(), &script);
    let client = CodexClient::builder()
        .binary(&binary)
        .timeout(Duration::from_secs(5))
        .add_dir("src")
        .output_schema(true)
        .quiet(true)
        .mirror_stdout(false)
        .build();

    let response = client.send_prompt("hello").await.unwrap();
    assert_eq!(response.trim(), "ok");

    let logged = std_fs::read_to_string(&log_path).unwrap();
    assert!(logged.contains("--add-dir"));
    assert!(logged.contains("src"));
    assert!(logged.contains("--output-schema"));
}

#[tokio::test]
async fn exec_skips_guarded_flags_when_unknown() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let temp = tempfile::tempdir().unwrap();
    let log_path = temp.path().join("exec.log");
    let script = format!(
        r#"#!/bin/bash
log="{log}"
if [[ "$1" == "--version" ]]; then
  echo "codex 0.9.0"
elif [[ "$1" == "features" && "$2" == "list" && "$3" == "--json" ]]; then
  echo "feature list unavailable" >&2
  exit 1
elif [[ "$1" == "features" && "$2" == "list" ]]; then
  echo "feature list unavailable" >&2
  exit 1
elif [[ "$1" == "--help" ]]; then
  echo "Usage: codex exec"
elif [[ "$1" == "exec" ]]; then
  echo "$@" >> "$log"
  echo "ok"
fi
"#,
        log = log_path.display()
    );
    let binary = write_fake_codex(temp.path(), &script);
    let client = CodexClient::builder()
        .binary(&binary)
        .timeout(Duration::from_secs(5))
        .add_dir("src")
        .output_schema(true)
        .quiet(true)
        .mirror_stdout(false)
        .build();

    let response = client.send_prompt("hello").await.unwrap();
    assert_eq!(response.trim(), "ok");

    let logged = std_fs::read_to_string(&log_path).unwrap();
    assert!(!logged.contains("--add-dir"));
    assert!(!logged.contains("--output-schema"));
}

#[tokio::test]
async fn mcp_login_skips_when_unsupported() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let temp = tempfile::tempdir().unwrap();
    let log_path = temp.path().join("login.log");
    let script = format!(
        r#"#!/bin/bash
log="{log}"
if [[ "$1" == "--version" ]]; then
  echo "codex 1.0.0"
elif [[ "$1" == "features" && "$2" == "list" && "$3" == "--json" ]]; then
  echo '{{"features":["output_schema","add_dir"]}}'
elif [[ "$1" == "features" && "$2" == "list" ]]; then
  echo "output_schema add-dir"
elif [[ "$1" == "--help" ]]; then
  echo "Usage: codex exec"
elif [[ "$1" == "login" ]]; then
  echo "$@" >> "$log"
  echo "login invoked"
fi
"#,
        log = log_path.display()
    );
    let binary = write_fake_codex(temp.path(), &script);
    let client = CodexClient::builder()
        .binary(&binary)
        .timeout(Duration::from_secs(5))
        .build();

    let login = client.spawn_mcp_login_process().await.unwrap();
    assert!(login.is_none());
    assert!(!log_path.exists());
}

#[tokio::test]
async fn mcp_login_runs_when_supported() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let temp = tempfile::tempdir().unwrap();
    let log_path = temp.path().join("login.log");
    let script = format!(
        r#"#!/bin/bash
log="{log}"
if [[ "$1" == "--version" ]]; then
  echo "codex 2.0.0"
elif [[ "$1" == "features" && "$2" == "list" && "$3" == "--json" ]]; then
  echo '{{"features":["output_schema","add_dir"],"mcp_login":true}}'
elif [[ "$1" == "features" && "$2" == "list" ]]; then
  echo "output_schema add_dir login --mcp"
elif [[ "$1" == "--help" ]]; then
  echo "Usage: codex --output-schema add-dir login --mcp"
elif [[ "$1" == "login" ]]; then
  echo "$@" >> "$log"
  echo "login invoked"
fi
"#,
        log = log_path.display()
    );
    let binary = write_fake_codex(temp.path(), &script);
    let client = CodexClient::builder()
        .binary(&binary)
        .timeout(Duration::from_secs(5))
        .build();

    let login = client
        .spawn_mcp_login_process()
        .await
        .unwrap()
        .expect("expected login child");
    let output = login.wait_with_output().await.unwrap();
    assert!(output.status.success());

    let logged = std_fs::read_to_string(&log_path).unwrap();
    assert!(logged.contains("login --mcp"));
}

#[tokio::test]
async fn probe_capabilities_caches_and_invalidates() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let temp = tempfile::tempdir().unwrap();
    let script_v1 = r#"#!/bin/bash
if [[ "$1" == "--version" ]]; then
  echo "codex 1.2.3-beta (commit cafe123)"
elif [[ "$1" == "features" && "$2" == "list" && "$3" == "--json" ]]; then
  echo '{"features":["output_schema","add_dir","mcp_login"]}'
elif [[ "$1" == "features" && "$2" == "list" ]]; then
  echo "output_schema add-dir login --mcp"
elif [[ "$1" == "--help" ]]; then
  echo "Usage: codex --output-schema add-dir login --mcp"
fi
"#;
    let binary = write_fake_codex(temp.path(), script_v1);
    let client = CodexClient::builder()
        .binary(&binary)
        .timeout(Duration::from_secs(5))
        .build();

    let first = client.probe_capabilities().await;
    assert_eq!(
        first.version.as_ref().and_then(|v| v.semantic),
        Some((1, 2, 3))
    );
    assert_eq!(
        first.version.as_ref().map(|v| v.channel),
        Some(CodexReleaseChannel::Beta)
    );
    assert_eq!(
        first.version.as_ref().and_then(|v| v.commit.as_deref()),
        Some("cafe123")
    );
    assert!(first.features.supports_features_list);
    assert!(first.features.supports_output_schema);
    assert!(first.features.supports_add_dir);
    assert!(first.features.supports_mcp_login);

    let cached = client.probe_capabilities().await;
    assert_eq!(cached, first);

    let script_v2 = r#"#!/bin/bash
if [[ "$1" == "--version" ]]; then
  echo "codex 2.0.0 (commit deadbeef)"
elif [[ "$1" == "features" && "$2" == "list" && "$3" == "--json" ]]; then
  echo '{"features":["add_dir"]}'
elif [[ "$1" == "features" && "$2" == "list" ]]; then
  echo "add-dir"
elif [[ "$1" == "--help" ]]; then
  echo "Usage: codex add-dir"
fi
"#;
    std_fs::write(&binary, script_v2).unwrap();
    let mut perms = std_fs::metadata(&binary).unwrap().permissions();
    perms.set_mode(0o755);
    std_fs::set_permissions(&binary, perms).unwrap();

    let refreshed = client.probe_capabilities().await;
    assert_ne!(refreshed.version, first.version);
    assert_eq!(
        refreshed.version.as_ref().and_then(|v| v.semantic),
        Some((2, 0, 0))
    );
    assert!(refreshed.features.supports_features_list);
    assert!(refreshed.features.supports_add_dir);
    assert!(!refreshed.features.supports_output_schema);
    assert!(!refreshed.features.supports_mcp_login);
    clear_capability_cache();
}
