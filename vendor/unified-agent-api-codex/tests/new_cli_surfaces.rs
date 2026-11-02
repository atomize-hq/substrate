#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use codex::{
    AppServerCodegenRequest, AppServerProxyRequest, AppServerRequest, CodexClient,
    DebugAppServerSendMessageV2Request, DebugModelsRequest, DebugPromptInputRequest,
    ExecServerRequest, FeaturesDisableRequest, FeaturesEnableRequest, PluginCommandRequest,
    PluginMarketplaceAddRequest, PluginMarketplaceCommandRequest, PluginMarketplaceRemoveRequest,
    PluginMarketplaceUpgradeRequest, SandboxCommandRequest, SandboxPlatform, UpdateCommandRequest,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Invocation {
    argv: Vec<String>,
}

#[tokio::test]
async fn features_enable_disable_spawn_expected_subcommands(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let log_path = temp.path().join("invocations.jsonl");
    let fake_codex = write_fake_codex(&log_path)?;

    let client = CodexClient::builder()
        .binary(&fake_codex)
        .mirror_stdout(false)
        .quiet(true)
        .build();

    client
        .features_enable(FeaturesEnableRequest::new("unified_exec"))
        .await?;
    client
        .features_disable(FeaturesDisableRequest::new("unified_exec"))
        .await?;

    let invocations = read_invocations(&log_path)?;
    assert!(
        invocations
            .iter()
            .any(|inv| inv.argv == ["features", "enable", "unified_exec"]),
        "missing features enable invocation: {:?}",
        invocations
            .iter()
            .map(|inv| inv.argv.as_slice())
            .collect::<Vec<_>>()
    );
    assert!(
        invocations
            .iter()
            .any(|inv| inv.argv == ["features", "disable", "unified_exec"]),
        "missing features disable invocation: {:?}",
        invocations
            .iter()
            .map(|inv| inv.argv.as_slice())
            .collect::<Vec<_>>()
    );

    Ok(())
}

#[tokio::test]
async fn debug_app_server_send_message_v2_spawns_expected_subcommand(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let log_path = temp.path().join("invocations.jsonl");
    let fake_codex = write_fake_codex(&log_path)?;

    let client = CodexClient::builder()
        .binary(&fake_codex)
        .mirror_stdout(false)
        .quiet(true)
        .build();

    client
        .debug_app_server_send_message_v2(DebugAppServerSendMessageV2Request::new("hello"))
        .await?;

    let invocations = read_invocations(&log_path)?;
    assert!(
        invocations
            .iter()
            .any(|inv| inv.argv == ["debug", "app-server", "send-message-v2", "hello"]),
        "missing debug send-message-v2 invocation: {:?}",
        invocations
            .iter()
            .map(|inv| inv.argv.as_slice())
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
async fn app_server_codegen_experimental_emits_flag() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let log_path = temp.path().join("invocations.jsonl");
    let fake_codex = write_fake_codex(&log_path)?;

    let client = CodexClient::builder()
        .binary(&fake_codex)
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let out_dir = temp.path().join("app-server-schema");
    client
        .generate_app_server_bindings(
            AppServerCodegenRequest::json_schema(&out_dir).experimental(true),
        )
        .await?;

    let invocations = read_invocations(&log_path)?;
    let invocation = invocations
        .iter()
        .find(|inv| inv.argv.first().map(|v| v.as_str()) == Some("app-server"))
        .expect("expected an app-server invocation");

    assert!(
        invocation.argv.iter().any(|arg| arg == "--experimental"),
        "--experimental missing from argv: {:?}",
        invocation.argv
    );

    Ok(())
}

#[tokio::test]
async fn new_0125_surfaces_spawn_expected_subcommands() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let log_path = temp.path().join("invocations.jsonl");
    let fake_codex = write_fake_codex(&log_path)?;

    let client = CodexClient::builder()
        .binary(&fake_codex)
        .mirror_stdout(false)
        .quiet(true)
        .build();

    client
        .debug_models(DebugModelsRequest::new().bundled(true))
        .await?;
    client
        .debug_prompt_input(
            DebugPromptInputRequest::new()
                .image(temp.path().join("one.png"))
                .image(temp.path().join("two.png"))
                .prompt("hello"),
        )
        .await?;
    client.plugin(PluginCommandRequest::new()).await?;
    client
        .plugin_marketplace(PluginMarketplaceCommandRequest::new())
        .await?;
    client
        .plugin_marketplace_add(
            PluginMarketplaceAddRequest::new("owner/repo")
                .source_ref("main")
                .sparse_path("marketplaces/core"),
        )
        .await?;
    client
        .plugin_marketplace_remove(PluginMarketplaceRemoveRequest::new("primary"))
        .await?;
    client
        .plugin_marketplace_upgrade(
            PluginMarketplaceUpgradeRequest::new().marketplace_name("primary"),
        )
        .await?;

    let mut app_server_proxy = client.start_app_server_proxy(
        AppServerProxyRequest::new().socket_path(temp.path().join("app-server.sock")),
    )?;
    let app_server_proxy_status = app_server_proxy.wait().await?;
    assert!(app_server_proxy_status.success());

    let mut app_server = client.start_app_server(
        AppServerRequest::new()
            .listen("127.0.0.1:9090")
            .ws_audience("aud")
            .ws_auth("shared-secret")
            .ws_issuer("issuer")
            .ws_max_clock_skew_seconds(15)
            .ws_shared_secret_file(temp.path().join("shared.secret"))
            .ws_token_file(temp.path().join("token.jwt"))
            .ws_token_sha256("abc123"),
    )?;
    let app_server_status = app_server.wait().await?;
    assert!(app_server_status.success());

    let mut exec_server = client.start_exec_server(ExecServerRequest::new().listen("stdio"))?;
    let exec_server_status = exec_server.wait().await?;
    assert!(exec_server_status.success());

    let invocations = read_invocations(&log_path)?;
    let argv_sets: Vec<_> = invocations
        .iter()
        .map(|inv| inv.argv.as_slice())
        .collect::<Vec<_>>();

    assert!(
        invocations
            .iter()
            .any(|inv| inv.argv == ["debug", "models", "--bundled"]),
        "missing debug models invocation: {:?}",
        argv_sets
    );
    assert!(
        invocations.iter().any(|inv| {
            inv.argv.first().map(|value| value.as_str()) == Some("debug")
                && inv.argv.get(1).map(|value| value.as_str()) == Some("prompt-input")
                && inv.argv.iter().any(|value| value == "--image")
                && inv.argv.last().map(|value| value.as_str()) == Some("hello")
        }),
        "missing debug prompt-input invocation: {:?}",
        argv_sets
    );
    assert!(
        invocations.iter().any(|inv| inv.argv == ["plugin"]),
        "missing plugin invocation: {:?}",
        argv_sets
    );
    assert!(
        invocations
            .iter()
            .any(|inv| inv.argv == ["plugin", "marketplace"]),
        "missing plugin marketplace invocation: {:?}",
        argv_sets
    );
    assert!(
        invocations.iter().any(|inv| {
            inv.argv
                == [
                    "plugin",
                    "marketplace",
                    "add",
                    "--ref",
                    "main",
                    "--sparse",
                    "marketplaces/core",
                    "owner/repo",
                ]
        }),
        "missing plugin marketplace add invocation: {:?}",
        argv_sets
    );
    assert!(
        invocations
            .iter()
            .any(|inv| { inv.argv == ["plugin", "marketplace", "remove", "primary"] }),
        "missing plugin marketplace remove invocation: {:?}",
        argv_sets
    );
    assert!(
        invocations
            .iter()
            .any(|inv| { inv.argv == ["plugin", "marketplace", "upgrade", "primary"] }),
        "missing plugin marketplace upgrade invocation: {:?}",
        argv_sets
    );
    assert!(
        invocations.iter().any(|inv| {
            inv.argv.first().map(|value| value.as_str()) == Some("app-server")
                && inv.argv.get(1).map(|value| value.as_str()) == Some("proxy")
                && inv.argv.iter().any(|value| value == "--sock")
        }),
        "missing app-server proxy invocation: {:?}",
        argv_sets
    );
    assert!(
        invocations.iter().any(|inv| {
            inv.argv.first().map(|value| value.as_str()) == Some("app-server")
                && inv.argv.get(1).map(|value| value.as_str()) != Some("proxy")
                && inv.argv.iter().any(|value| value == "--listen")
                && inv.argv.iter().any(|value| value == "--ws-audience")
                && inv.argv.iter().any(|value| value == "--ws-auth")
                && inv.argv.iter().any(|value| value == "--ws-issuer")
                && inv
                    .argv
                    .iter()
                    .any(|value| value == "--ws-max-clock-skew-seconds")
                && inv
                    .argv
                    .iter()
                    .any(|value| value == "--ws-shared-secret-file")
                && inv.argv.iter().any(|value| value == "--ws-token-file")
                && inv.argv.iter().any(|value| value == "--ws-token-sha256")
        }),
        "missing app-server invocation: {:?}",
        argv_sets
    );
    assert!(
        invocations.iter().any(|inv| {
            inv.argv.first().map(|value| value.as_str()) == Some("exec-server")
                && inv.argv.iter().any(|value| value == "--listen")
        }),
        "missing exec-server listen invocation: {:?}",
        argv_sets
    );

    Ok(())
}

#[tokio::test]
async fn new_0129_surfaces_spawn_expected_subcommands() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let log_path = temp.path().join("invocations.jsonl");
    let fake_codex = write_fake_codex(&log_path)?;

    let client = CodexClient::builder()
        .binary(&fake_codex)
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let mut exec_server = client.start_exec_server(
        ExecServerRequest::new()
            .listen("stdio")
            .executor_id("executor-1")
            .name("background-worker"),
    )?;
    let exec_server_status = exec_server.wait().await?;
    assert!(exec_server_status.success());

    let access_token_login = client.spawn_with_access_token_login_process()?;
    let access_token_login_output = access_token_login.wait_with_output().await?;
    assert!(access_token_login_output.status.success());

    let sandbox_linux = client
        .run_sandbox(
            SandboxCommandRequest::new(SandboxPlatform::Linux, ["echo", "linux"])
                .include_managed_config(true)
                .permissions_profile("linux-profile"),
        )
        .await?;
    assert!(sandbox_linux.status.success());

    let sandbox_macos = client
        .run_sandbox(
            SandboxCommandRequest::new(SandboxPlatform::Macos, ["echo", "macos"])
                .include_managed_config(true)
                .permissions_profile("macos-profile"),
        )
        .await?;
    assert!(sandbox_macos.status.success());

    let sandbox_windows = client
        .run_sandbox(
            SandboxCommandRequest::new(SandboxPlatform::Windows, ["echo", "windows"])
                .include_managed_config(true)
                .permissions_profile("windows-profile"),
        )
        .await?;
    assert!(sandbox_windows.status.success());

    let update = client.update(UpdateCommandRequest::new()).await?;
    assert!(update.status.success());

    let invocations = read_invocations(&log_path)?;
    let argv_sets: Vec<_> = invocations
        .iter()
        .map(|inv| inv.argv.as_slice())
        .collect::<Vec<_>>();

    assert!(
        invocations.iter().any(|inv| {
            inv.argv.first().map(|value| value.as_str()) == Some("exec-server")
                && inv.argv.iter().any(|value| value == "--listen")
                && inv.argv.iter().any(|value| value == "--executor-id")
                && inv.argv.iter().any(|value| value == "--name")
        }),
        "missing 0.129 exec-server invocation: {:?}",
        argv_sets
    );
    assert!(
        invocations
            .iter()
            .any(|inv| inv.argv == ["login", "--with-access-token"]),
        "missing access-token login invocation: {:?}",
        argv_sets
    );
    assert!(
        invocations.iter().any(|inv| {
            inv.argv.first().map(|value| value.as_str()) == Some("sandbox")
                && inv.argv.get(1).map(|value| value.as_str()) == Some("linux")
                && inv
                    .argv
                    .iter()
                    .any(|value| value == "--include-managed-config")
                && inv
                    .argv
                    .iter()
                    .any(|value| value == "--permissions-profile")
        }),
        "missing sandbox linux 0.129 invocation: {:?}",
        argv_sets
    );
    assert!(
        invocations.iter().any(|inv| {
            inv.argv.first().map(|value| value.as_str()) == Some("sandbox")
                && inv.argv.get(1).map(|value| value.as_str()) == Some("macos")
                && inv
                    .argv
                    .iter()
                    .any(|value| value == "--include-managed-config")
                && inv
                    .argv
                    .iter()
                    .any(|value| value == "--permissions-profile")
        }),
        "missing sandbox macos 0.129 invocation: {:?}",
        argv_sets
    );
    assert!(
        invocations.iter().any(|inv| {
            inv.argv.first().map(|value| value.as_str()) == Some("sandbox")
                && inv.argv.get(1).map(|value| value.as_str()) == Some("windows")
                && inv
                    .argv
                    .iter()
                    .any(|value| value == "--include-managed-config")
                && inv
                    .argv
                    .iter()
                    .any(|value| value == "--permissions-profile")
        }),
        "missing sandbox windows 0.129 invocation: {:?}",
        argv_sets
    );
    assert!(
        invocations.iter().any(|inv| inv.argv == ["update"]),
        "missing update invocation: {:?}",
        argv_sets
    );

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

python3 - "$LOG_PATH" "$@" <<'PY'
import json
import sys

log_path = sys.argv[1]
argv = sys.argv[2:]

with open(log_path, 'a', encoding='utf-8') as handle:
    handle.write(json.dumps({{'argv': argv}}))
    handle.write('\n')
PY

if [[ $# -ge 2 && $1 == "features" && ( $2 == "enable" || $2 == "disable" ) ]]; then
  echo "features-ok"
  exit 0
fi

if [[ $# -ge 4 && $1 == "debug" && $2 == "app-server" && $3 == "send-message-v2" ]]; then
  echo "debug-ok"
  exit 0
fi

if [[ $# -ge 2 && $1 == "debug" && $2 == "models" ]]; then
  echo "debug-models-ok"
  exit 0
fi

if [[ $# -ge 2 && $1 == "debug" && $2 == "prompt-input" ]]; then
  echo "debug-prompt-input-ok"
  exit 0
fi

if [[ $# -ge 2 && $1 == "app-server" && ( $2 == "generate-ts" || $2 == "generate-json-schema" ) ]]; then
  echo "app-server-ok"
  exit 0
fi

if [[ $# -ge 2 && $1 == "app-server" && $2 == "proxy" ]]; then
  echo "app-server-proxy-ok"
  exit 0
fi

if [[ $# -ge 1 && $1 == "app-server" ]]; then
  echo "app-server-root-ok"
  exit 0
fi

if [[ $# -ge 1 && $1 == "exec-server" ]]; then
  echo "exec-server-ok"
  exit 0
fi

if [[ $# -ge 2 && $1 == "login" && $2 == "--with-access-token" ]]; then
  echo "login-with-access-token-ok"
  exit 0
fi

if [[ $# -ge 2 && $1 == "sandbox" ]]; then
  echo "sandbox-ok"
  exit 0
fi

if [[ $# -ge 1 && $1 == "plugin" ]]; then
  echo "plugin-ok"
  exit 0
fi

if [[ $# -ge 1 && $1 == "update" ]]; then
  echo "update-ok"
  exit 0
fi

echo "unknown command: $@" >&2
exit 1
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
