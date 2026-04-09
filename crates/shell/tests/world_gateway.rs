#![cfg(unix)]

#[path = "common.rs"]
mod common;

use clap::Parser;
use common::substrate_shell_driver;
use predicates::prelude::*;
use serde_json::json;
use substrate_shell::execution::{Cli, SubCommands, WorldAction, WorldGatewayAction};
use tempfile::TempDir;

#[path = "support/socket.rs"]
mod socket;

use socket::{AgentSocket, SocketResponse};

fn parse_world_gateway_status_json() -> Cli {
    Cli::try_parse_from(["substrate", "world", "gateway", "status", "--json"])
        .expect("gateway status --json should parse")
}

fn assert_gateway_unavailable(args: &[&str], expected_fragment: &str) {
    let mut cmd = substrate_shell_driver();
    cmd.args(args)
        .assert()
        .code(4)
        .stderr(predicate::str::contains(expected_fragment));
}

fn assert_gateway_unavailable_json(args: &[&str]) {
    let mut cmd = substrate_shell_driver();
    cmd.args(args)
        .assert()
        .code(4)
        .stdout("{\"status\":\"unavailable\"}\n")
        .stderr(predicate::str::is_empty());
}

fn gateway_socket_fixture() -> (TempDir, AgentSocket, std::path::PathBuf) {
    let temp = tempfile::tempdir().expect("gateway socket tempdir");
    let socket_path = temp.path().join("agent.sock");
    let socket = AgentSocket::start(
        &socket_path,
        SocketResponse::GatewayLifecycle {
            status: json!({
                "status": "available",
                "client_wiring": {
                    "openai_base_url": "http://gateway.test/openai",
                    "anthropic_base_url": "http://gateway.test/anthropic"
                }
            }),
            sync: json!({
                "status": "available",
                "client_wiring": {
                    "openai_base_url": "http://gateway.test/openai",
                    "anthropic_base_url": "http://gateway.test/anthropic"
                }
            }),
            restart: json!({
                "status": "available",
                "client_wiring": {
                    "openai_base_url": "http://gateway.test/openai",
                    "anthropic_base_url": "http://gateway.test/anthropic"
                }
            }),
        },
    );

    (temp, socket, socket_path)
}

fn gateway_unavailable_socket_fixture() -> (TempDir, AgentSocket, std::path::PathBuf) {
    let temp = tempfile::tempdir().expect("gateway unavailable socket tempdir");
    let socket_path = temp.path().join("agent.sock");
    let socket = AgentSocket::start(
        &socket_path,
        SocketResponse::GatewayLifecycle {
            status: json!({
                "status": "unavailable"
            }),
            sync: json!({
                "status": "unavailable"
            }),
            restart: json!({
                "status": "unavailable"
            }),
        },
    );

    (temp, socket, socket_path)
}

#[test]
fn world_gateway_status_accepts_json_leaf() {
    let cli = parse_world_gateway_status_json();

    match cli.sub {
        Some(SubCommands::World(world)) => match world.action {
            WorldAction::Gateway(gateway) => match gateway.action {
                WorldGatewayAction::Status(args) => {
                    assert!(args.json, "status --json leaf should set the json flag");
                }
                _ => panic!("expected world gateway status action"),
            },
            _ => panic!("expected world gateway command family"),
        },
        _ => panic!("expected world subcommand"),
    }
}

#[test]
fn world_gateway_rejects_archived_command_ordering() {
    for args in [
        ["substrate", "world", "status", "gateway"],
        ["substrate", "world", "sync", "gateway"],
        ["substrate", "world", "restart", "gateway"],
    ] {
        assert!(
            Cli::try_parse_from(args).is_err(),
            "archived ordering should not parse: {args:?}"
        );
    }
}

#[test]
fn world_gateway_absent_state_is_explicit_for_status_sync_and_restart() {
    assert_gateway_unavailable(
        &["world", "gateway", "status"],
        "substrate world gateway status: unavailable (required gateway/world component unavailable)",
    );
    assert_gateway_unavailable_json(&["world", "gateway", "status", "--json"]);
    assert_gateway_unavailable(
        &["world", "gateway", "sync"],
        "substrate world gateway sync: unavailable (required gateway/world component unavailable)",
    );
    assert_gateway_unavailable(
        &["world", "gateway", "restart"],
        "substrate world gateway restart: unavailable (required gateway/world component unavailable)",
    );
}

#[test]
fn world_gateway_status_json_uses_typed_runtime_contract() {
    let (_temp, _socket, socket_path) = gateway_socket_fixture();

    let mut cmd = substrate_shell_driver();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args(["world", "gateway", "status", "--json"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains(
            "{\"status\":\"available\",\"client_wiring\":{\"openai_base_url\":\"http://gateway.test/openai\",\"anthropic_base_url\":\"http://gateway.test/anthropic\"}}",
        ))
        .stderr(predicate::str::is_empty());
}

#[test]
fn world_gateway_status_json_preserves_unavailable_shape_from_runtime() {
    let (_temp, _socket, socket_path) = gateway_unavailable_socket_fixture();

    let mut cmd = substrate_shell_driver();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args(["world", "gateway", "status", "--json"])
        .assert()
        .code(4)
        .stdout("{\"status\":\"unavailable\"}\n")
        .stderr(predicate::str::is_empty());
}

#[test]
fn world_gateway_disabled_state_skips_typed_runtime_bootstrap() {
    let (_temp, _socket, socket_path) = gateway_socket_fixture();

    let mut cmd = substrate_shell_driver();
    cmd.env("SUBSTRATE_WORLD_ENABLED", "0")
        .env("SUBSTRATE_WORLD", "disabled")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args(["world", "gateway", "status", "--json"])
        .assert()
        .code(4)
        .stdout("{\"status\":\"unavailable\"}\n")
        .stderr(predicate::str::is_empty());
}

#[test]
fn world_gateway_missing_socket_is_classified_as_absent_state() {
    let temp = tempfile::tempdir().expect("missing socket tempdir");
    let missing_socket_path = temp.path().join("missing.sock");

    let mut cmd = substrate_shell_driver();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket_path)
        .args(["world", "gateway", "status"])
        .assert()
        .code(4)
        .stderr(predicate::str::contains(
            "substrate world gateway status: unavailable (required gateway/world component unavailable)",
        ));
}

#[test]
fn world_gateway_http_failures_bubble_as_errors() {
    let temp = tempfile::tempdir().expect("gateway http error tempdir");
    let socket_path = temp.path().join("agent.sock");
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::GatewayLifecycleHttpError {
            status: 500,
            body: "{\"error\":\"internal\"}".to_string(),
        },
    );

    let mut cmd = substrate_shell_driver();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args(["world", "gateway", "status"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("substrate world gateway:"));
}
