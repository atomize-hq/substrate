#![cfg(unix)]

#[path = "common.rs"]
mod common;

use clap::Parser;
use common::substrate_shell_driver;
use predicates::prelude::*;
use substrate_shell::execution::{Cli, SubCommands, WorldAction, WorldGatewayAction};

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
        "substrate world gateway status: unavailable",
    );
    assert_gateway_unavailable(
        &["world", "gateway", "status", "--json"],
        "substrate world gateway status --json: unavailable",
    );
    assert_gateway_unavailable(
        &["world", "gateway", "sync"],
        "substrate world gateway sync: unavailable",
    );
    assert_gateway_unavailable(
        &["world", "gateway", "restart"],
        "substrate world gateway restart: unavailable",
    );
}
