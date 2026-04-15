use assert_cmd::Command;
use clap as _;
use predicates::prelude::*;
use substrate_lift as _;

#[test]
fn top_level_help_lists_required_commands() {
    Command::cargo_bin("lift")
        .expect("lift binary should be buildable for tests")
        .arg("--help")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("score")
                .and(predicate::str::contains("estimate"))
                .and(predicate::str::contains("analyze"))
                .and(predicate::str::contains("validate"))
                .and(predicate::str::contains("print-schema"))
                .and(predicate::str::contains("print-model")),
        );
}

#[test]
fn representative_nested_help_invocations_exit_zero() {
    let cases = [
        (["score", "--help"].as_slice(), "vector"),
        (["estimate", "--help"].as_slice(), "path"),
        (["analyze", "--help"].as_slice(), "symbol"),
        (["validate", "--help"].as_slice(), "config"),
        (["print-schema", "--help"].as_slice(), "Usage"),
        (["print-model", "--help"].as_slice(), "Usage"),
    ];

    for (args, expected) in cases {
        Command::cargo_bin("lift")
            .expect("lift binary should be buildable for tests")
            .args(args)
            .assert()
            .success()
            .stdout(predicate::str::contains(expected));
    }
}
