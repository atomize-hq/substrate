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

fn assert_global_list_available_fails(
    fixture: &ShellEnvFixture,
    path: &Path,
    contents: &str,
    expected_stderr: &str,
) {
    write_file(path, contents);

    substrate_command_for_home(fixture)
        .args(["world", "deps", "global", "list", "available"])
        .assert()
        .code(2)
        .stderr(contains("invalid package schema in"))
        .stderr(contains(expected_stderr));
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

#[test]
fn test_pacman_install_requires_non_empty_package_list() {
    let fixture = ShellEnvFixture::new();
    let path = global_deps_dir(&fixture).join("packages/pacman-empty.yaml");
    assert_global_list_available_fails(
        &fixture,
        &path,
        concat!(
            "version: 1\n",
            "name: pacman-empty\n",
            "runnable: false\n",
            "install:\n",
            "  method: pacman\n",
            "  pacman: []\n",
        ),
        "install.method=pacman requires a non-empty install.pacman list",
    );
}

#[test]
fn test_pacman_install_rejects_install_apt() {
    let fixture = ShellEnvFixture::new();
    let path = global_deps_dir(&fixture).join("packages/pacman-apt.yaml");
    assert_global_list_available_fails(
        &fixture,
        &path,
        concat!(
            "version: 1\n",
            "name: pacman-apt\n",
            "runnable: false\n",
            "install:\n",
            "  method: pacman\n",
            "  pacman:\n",
            "    - pacman-tool\n",
            "  apt:\n",
            "    - name: apt-tool\n",
        ),
        "install.method=pacman must not define install.apt",
    );
}

#[test]
fn test_pacman_install_rejects_script_path() {
    let fixture = ShellEnvFixture::new();
    let script_path = global_deps_dir(&fixture).join("packages/pacman-script-path.yaml");
    assert_global_list_available_fails(
        &fixture,
        &script_path,
        concat!(
            "version: 1\n",
            "name: pacman-script-path\n",
            "runnable: false\n",
            "install:\n",
            "  method: pacman\n",
            "  pacman:\n",
            "    - pacman-tool\n",
            "  script_path: ../scripts/pacman.sh\n",
        ),
        "install.method=pacman must not define install.script or install.script_path",
    );
}

#[test]
fn test_pacman_install_rejects_manual_instructions() {
    let fixture = ShellEnvFixture::new();
    let manual_path = global_deps_dir(&fixture).join("packages/pacman-manual.yaml");
    assert_global_list_available_fails(
        &fixture,
        &manual_path,
        concat!(
            "version: 1\n",
            "name: pacman-manual\n",
            "runnable: false\n",
            "install:\n",
            "  method: pacman\n",
            "  pacman:\n",
            "    - pacman-tool\n",
            "  manual_instructions: |\n",
            "    do not use\n",
        ),
        "install.method=pacman must not define install.manual_instructions",
    );
}

#[test]
fn test_pacman_install_rejects_runnable_true() {
    let fixture = ShellEnvFixture::new();
    let runnable_path = global_deps_dir(&fixture).join("packages/pacman-runnable.yaml");
    assert_global_list_available_fails(
        &fixture,
        &runnable_path,
        concat!(
            "version: 1\n",
            "name: pacman-runnable\n",
            "runnable: true\n",
            "entrypoints:\n",
            "  - pacman-runnable\n",
            "install:\n",
            "  method: pacman\n",
            "  pacman:\n",
            "    - pacman-tool\n",
        ),
        "install.method=pacman packages must not be runnable",
    );
}

#[test]
fn test_pacman_install_rejects_entrypoints() {
    let fixture = ShellEnvFixture::new();
    let entrypoints_path = global_deps_dir(&fixture).join("packages/pacman-entrypoints.yaml");
    assert_global_list_available_fails(
        &fixture,
        &entrypoints_path,
        concat!(
            "version: 1\n",
            "name: pacman-entrypoints\n",
            "runnable: false\n",
            "entrypoints:\n",
            "  - pacman-entrypoints\n",
            "install:\n",
            "  method: pacman\n",
            "  pacman:\n",
            "    - pacman-tool\n",
        ),
        "install.method=pacman packages must not define entrypoints",
    );
}

#[test]
fn test_pacman_install_rejects_wrappers() {
    let fixture = ShellEnvFixture::new();
    let wrappers_path = global_deps_dir(&fixture).join("packages/pacman-wrappers.yaml");
    assert_global_list_available_fails(
        &fixture,
        &wrappers_path,
        concat!(
            "version: 1\n",
            "name: pacman-wrappers\n",
            "runnable: false\n",
            "wrappers:\n",
            "  - name: pacman-wrappers\n",
            "    kind: sh_env_exec\n",
            "    exec: pacman-tool\n",
            "    env:\n",
            "      FOO: bar\n",
            "install:\n",
            "  method: pacman\n",
            "  pacman:\n",
            "    - pacman-tool\n",
        ),
        "install.method=pacman packages must not define wrappers",
    );
}

#[test]
fn test_pacman_install_rejects_probe() {
    let fixture = ShellEnvFixture::new();
    let probe_path = global_deps_dir(&fixture).join("packages/pacman-probe.yaml");
    assert_global_list_available_fails(
        &fixture,
        &probe_path,
        concat!(
            "version: 1\n",
            "name: pacman-probe\n",
            "runnable: false\n",
            "install:\n",
            "  method: pacman\n",
            "  pacman:\n",
            "    - pacman-tool\n",
            "probe:\n",
            "  command: \"pacman-tool --version\"\n",
        ),
        "install.method=pacman packages must not define probe",
    );
}
