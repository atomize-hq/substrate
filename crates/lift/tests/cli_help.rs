#![allow(unused_crate_dependencies)]

use assert_cmd::Command;
use gix as _;
use globset as _;
use jsonschema as _;
use predicates::prelude::*;
use serde as _;
use serde_jcs as _;
use serde_json as _;
use sha2 as _;
use std::process;
use substrate_lift as _;
use thiserror as _;
use toml as _;
use walkdir as _;

fn lift_command() -> Command {
    let binary = assert_cmd::cargo::cargo_bin("lift");
    Command::from_std(process::Command::new(binary))
}

#[test]
fn top_level_help_lists_required_commands() {
    lift_command().arg("--help").assert().success().stdout(
        predicate::str::contains("score")
            .and(predicate::str::contains("impact"))
            .and(predicate::str::contains("policy"))
            .and(predicate::str::contains("contract"))
            .and(predicate::str::contains("context"))
            .and(predicate::str::contains("index"))
            .and(predicate::str::contains("query"))
            .and(predicate::str::contains("rewrite"))
            .and(predicate::str::contains("estimate").not())
            .and(predicate::str::contains("analyze").not())
            .and(predicate::str::contains("explain").not())
            .and(predicate::str::contains("validate").not())
            .and(predicate::str::contains("print-schema").not())
            .and(predicate::str::contains("print-model").not()),
    );
}

#[test]
fn representative_nested_help_invocations_exit_zero() {
    let cases = [
        (["score", "--help"].as_slice(), "Usage: lift score"),
        (["impact", "--help"].as_slice(), "Usage: lift impact"),
        (["policy", "--help"].as_slice(), "Usage: lift policy"),
        (["contract", "--help"].as_slice(), "Usage: lift contract"),
        (["context", "--help"].as_slice(), "Usage: lift context"),
        (["index", "--help"].as_slice(), "Usage: lift index"),
        (["query", "--help"].as_slice(), "Usage: lift query"),
        (["rewrite", "--help"].as_slice(), "Usage: lift rewrite"),
    ];

    for (args, expected) in cases {
        lift_command()
            .args(args)
            .assert()
            .success()
            .stdout(predicate::str::contains(expected));
    }
}
