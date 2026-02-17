#![cfg(unix)]

mod common;

use common::{binary_path, ensure_substrate_built, shared_tmpdir, temp_dir};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

struct DepsScaffoldFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    cwd: PathBuf,
}

impl DepsScaffoldFixture {
    fn new() -> Self {
        let temp = temp_dir("substrate-wdh3-scaffold-");
        let home = temp.path().join("home");
        let substrate_home = temp.path().join("substrate-home");
        let cwd = temp.path().join("cwd");
        fs::create_dir_all(&home).expect("create HOME");
        fs::create_dir_all(&substrate_home).expect("create SUBSTRATE_HOME");
        fs::create_dir_all(&cwd).expect("create cwd");
        Self {
            _temp: temp,
            home,
            substrate_home,
            cwd,
        }
    }

    fn bootstrap(&self) -> Output {
        let host_path = std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_string());
        Command::new(binary_path())
            .env_clear()
            .env("PATH", host_path)
            .env("TMPDIR", shared_tmpdir())
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("SUBSTRATE_HOME", &self.substrate_home)
            .current_dir(&self.cwd)
            .arg("--no-world")
            .args(["config", "current", "show", "--json"])
            .output()
            .expect("run bootstrap command")
    }
}

fn deps_dir(substrate_home: &Path) -> PathBuf {
    substrate_home.join("deps")
}

fn required_scaffold_dirs(substrate_home: &Path) -> Vec<PathBuf> {
    let deps = deps_dir(substrate_home);
    vec![
        deps.join("packages"),
        deps.join("bundles"),
        deps.join("scripts"),
    ]
}

fn required_scaffold_files(substrate_home: &Path) -> Vec<PathBuf> {
    let deps = deps_dir(substrate_home);
    vec![
        deps.join("README.md"),
        deps.join("packages").join("example-manual.yaml"),
        deps.join("packages").join("example-script.yaml"),
        deps.join("packages").join("example-apt.yaml"),
        deps.join("bundles").join("example-bundle.yaml"),
        deps.join("scripts").join("example-install.sh"),
    ]
}

fn assert_scaffold_present(substrate_home: &Path) {
    let deps = deps_dir(substrate_home);
    assert!(
        deps.is_dir(),
        "expected deps scaffold dir at {}",
        deps.display()
    );
    for dir in required_scaffold_dirs(substrate_home) {
        assert!(dir.is_dir(), "expected scaffold dir at {}", dir.display());
    }
    for file in required_scaffold_files(substrate_home) {
        assert!(
            file.is_file(),
            "expected scaffold file at {}",
            file.display()
        );
    }
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

#[test]
fn deps_scaffold_is_idempotent_wdh3() {
    ensure_substrate_built();
    let fixture = DepsScaffoldFixture::new();

    assert!(
        !deps_dir(&fixture.substrate_home).exists(),
        "precondition: deps scaffold should not exist yet"
    );

    let first = fixture.bootstrap();
    assert!(
        first.status.success(),
        "bootstrap should succeed (first run): {first:?}"
    );
    assert_scaffold_present(&fixture.substrate_home);

    let second = fixture.bootstrap();
    assert!(
        second.status.success(),
        "bootstrap should succeed (second run): {second:?}"
    );
    assert_scaffold_present(&fixture.substrate_home);
}

#[test]
fn deps_scaffold_does_not_overwrite_existing_files_wdh3() {
    ensure_substrate_built();
    let fixture = DepsScaffoldFixture::new();

    let readme = deps_dir(&fixture.substrate_home).join("README.md");
    let manual = deps_dir(&fixture.substrate_home)
        .join("packages")
        .join("example-manual.yaml");

    write_file(&readme, "user README\n");
    write_file(&manual, "user example\n");

    let output = fixture.bootstrap();
    assert!(
        output.status.success(),
        "bootstrap should succeed: {output:?}"
    );
    assert_scaffold_present(&fixture.substrate_home);

    let readme_after = fs::read_to_string(&readme).expect("read seeded README");
    assert_eq!(
        readme_after, "user README\n",
        "bootstrap must not overwrite existing README.md"
    );
    let manual_after = fs::read_to_string(&manual).expect("read seeded manual example");
    assert_eq!(
        manual_after, "user example\n",
        "bootstrap must not overwrite existing example-manual.yaml"
    );
}
