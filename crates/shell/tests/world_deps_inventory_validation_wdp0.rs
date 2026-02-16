#![cfg(unix)]

mod support;

use support::{substrate_command_for_home, ShellEnvFixture};

use predicates::str::contains;
use std::fs;
use std::path::{Path, PathBuf};

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

fn global_deps_dir(fixture: &ShellEnvFixture) -> PathBuf {
    fixture.home().join(".substrate/deps")
}

#[test]
fn test_wrapper_name_must_be_listed_in_entrypoints() {
    let fixture = ShellEnvFixture::new();
    let path = global_deps_dir(&fixture).join("packages/badwrap.yaml");
    write_file(
        &path,
        concat!(
            "version: 1\n",
            "name: badwrap\n",
            "runnable: true\n",
            "entrypoints:\n",
            "  - ok\n",
            "wrappers:\n",
            "  - name: badwrap\n",
            "    kind: sh_env_exec\n",
            "    exec: ok\n",
            "    env:\n",
            "      FOO: bar\n",
            "install:\n",
            "  method: apt\n",
            "  apt:\n",
            "    - name: badwrap\n",
        ),
    );

    substrate_command_for_home(&fixture)
        .args(["world", "deps", "global", "list", "available"])
        .assert()
        .code(2)
        .stderr(contains("invalid package schema in"))
        .stderr(contains("badwrap.yaml"))
        .stderr(contains(
            "wrappers[].name 'badwrap' must be listed in entrypoints[]",
        ));
}

#[test]
fn test_sh_env_exec_requires_env() {
    let fixture = ShellEnvFixture::new();
    let path = global_deps_dir(&fixture).join("packages/noenv.yaml");
    write_file(
        &path,
        concat!(
            "version: 1\n",
            "name: noenv\n",
            "runnable: true\n",
            "entrypoints:\n",
            "  - noenv\n",
            "wrappers:\n",
            "  - name: noenv\n",
            "    kind: sh_env_exec\n",
            "    exec: noenv\n",
            "install:\n",
            "  method: apt\n",
            "  apt:\n",
            "    - name: noenv\n",
        ),
    );

    substrate_command_for_home(&fixture)
        .args(["world", "deps", "global", "list", "available"])
        .assert()
        .code(2)
        .stderr(contains("invalid package schema in"))
        .stderr(contains("noenv.yaml"))
        .stderr(contains(
            "wrappers[].kind=sh_env_exec requires non-empty env",
        ));
}
