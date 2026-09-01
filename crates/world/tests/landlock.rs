use std::fs;
use std::process::{Command, ExitStatus};
use world::landlock::{
    apply_filesystem_policy, apply_write_only_allowlist, detect_support, LandlockFilesystemPolicy,
};

const EXACT_FILE_CHILD_ENV: &str = "SUBSTRATE_LANDLOCK_EXACT_FILE_CHILD";
const EXACT_FILE_ROOT_ENV: &str = "SUBSTRATE_LANDLOCK_EXACT_FILE_ROOT";

fn run_exact_file_child(scenario: &str, root: &std::path::Path) -> ExitStatus {
    Command::new(std::env::current_exe().expect("current test executable"))
        .arg("--exact")
        .arg("landlock_exact_file_child")
        .arg("--nocapture")
        .env(EXACT_FILE_CHILD_ENV, scenario)
        .env(EXACT_FILE_ROOT_ENV, root)
        .status()
        .expect("run exact-file Landlock child")
}

#[test]
fn landlock_detect_support_is_consistent() {
    let support = detect_support();
    if support.supported {
        assert!(
            support.abi.is_some(),
            "supported=true should include an ABI version"
        );
        assert!(
            support.reason.is_none(),
            "supported=true should not include a reason"
        );
    } else {
        assert!(
            support.abi.is_none(),
            "supported=false should not include an ABI version"
        );
    }
}

#[test]
fn landlock_empty_policy_is_noop() {
    let policy = LandlockFilesystemPolicy {
        exec_paths: Vec::new(),
        discover_paths: Vec::new(),
        read_paths: Vec::new(),
        write_paths: Vec::new(),
    };

    let report = apply_filesystem_policy(&policy);

    assert!(!report.attempted);
    assert!(!report.applied);
    assert_eq!(report.rules_added, 0);

    if report.support.supported {
        assert_eq!(
            report.reason.as_deref(),
            Some("landlock policy was empty; skipping")
        );
    }
}

#[cfg(target_os = "linux")]
#[test]
fn landlock_exact_file_child() {
    let Ok(scenario) = std::env::var(EXACT_FILE_CHILD_ENV) else {
        return;
    };
    let root = std::path::PathBuf::from(
        std::env::var_os(EXACT_FILE_ROOT_ENV).expect("exact-file test root"),
    );
    let allowed = root.join("allowed.txt");
    let sibling = root.join("sibling.txt");

    let (exec_paths, discover_paths, read_paths, write_paths) = match scenario.as_str() {
        "read" => (
            Vec::new(),
            vec![allowed.display().to_string()],
            vec![allowed.display().to_string()],
            Vec::new(),
        ),
        "write" => (
            Vec::new(),
            vec![allowed.display().to_string()],
            vec![allowed.display().to_string()],
            vec![allowed.display().to_string()],
        ),
        "write_only" => (
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![allowed.display().to_string()],
        ),
        "execute" => {
            let shell = fs::canonicalize("/bin/sh").expect("canonical shell interpreter");
            let mut runtime_paths = vec![shell.display().to_string()];
            runtime_paths.extend(
                ["/lib", "/lib64", "/usr/lib", "/etc/ld.so.cache"]
                    .into_iter()
                    .filter_map(|path| fs::canonicalize(path).ok())
                    .map(|path| path.display().to_string()),
            );
            runtime_paths.sort();
            runtime_paths.dedup();
            let mut exec_paths = vec![allowed.display().to_string(), shell.display().to_string()];
            exec_paths.sort();
            exec_paths.dedup();
            let mut read_paths = runtime_paths;
            read_paths.push(allowed.display().to_string());
            (exec_paths, read_paths.clone(), read_paths, Vec::new())
        }
        other => panic!("unknown exact-file Landlock scenario {other}"),
    };
    let report = if scenario == "write_only" {
        apply_write_only_allowlist(&write_paths)
    } else {
        apply_filesystem_policy(&LandlockFilesystemPolicy {
            exec_paths,
            discover_paths,
            read_paths,
            write_paths,
        })
    };
    if !report.support.supported {
        std::process::exit(77);
    }
    assert!(report.applied, "exact-file rule failed: {report:?}");
    if scenario != "execute" {
        assert_eq!(fs::read_to_string(&allowed).unwrap(), "allowed\n");
    }
    if scenario != "write_only" {
        assert!(fs::read_to_string(&sibling).is_err());
    }

    if matches!(scenario.as_str(), "write" | "write_only") {
        fs::write(&allowed, "updated\n").expect("write exact allowed file");
        assert!(fs::write(&sibling, "forbidden\n").is_err());
    } else if scenario == "execute" {
        assert!(Command::new(&allowed).status().unwrap().success());
        assert!(Command::new(&sibling).status().is_err());
    }
}

#[cfg(target_os = "linux")]
#[test]
fn landlock_exact_readable_file_does_not_grant_sibling() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("allowed.txt"), "allowed\n").unwrap();
    fs::write(root.path().join("sibling.txt"), "sibling\n").unwrap();

    let status = run_exact_file_child("read", root.path());
    if status.code() == Some(77) {
        eprintln!("Landlock unavailable; exact-file enforcement test skipped");
        return;
    }
    assert!(
        status.success(),
        "exact readable-file child failed: {status}"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn landlock_exact_writable_file_does_not_grant_sibling() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("allowed.txt"), "allowed\n").unwrap();
    fs::write(root.path().join("sibling.txt"), "sibling\n").unwrap();

    let status = run_exact_file_child("write", root.path());
    if status.code() == Some(77) {
        eprintln!("Landlock unavailable; exact-file enforcement test skipped");
        return;
    }
    assert!(
        status.success(),
        "exact writable-file child failed: {status}"
    );
    assert_eq!(
        fs::read_to_string(root.path().join("allowed.txt")).unwrap(),
        "updated\n"
    );
    assert_eq!(
        fs::read_to_string(root.path().join("sibling.txt")).unwrap(),
        "sibling\n"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn landlock_write_only_exact_file_does_not_grant_sibling_write() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("allowed.txt"), "allowed\n").unwrap();
    fs::write(root.path().join("sibling.txt"), "sibling\n").unwrap();

    let status = run_exact_file_child("write_only", root.path());
    if status.code() == Some(77) {
        eprintln!("Landlock unavailable; exact-file enforcement test skipped");
        return;
    }
    assert!(
        status.success(),
        "write-only exact-file child failed: {status}"
    );
    assert_eq!(
        fs::read_to_string(root.path().join("allowed.txt")).unwrap(),
        "updated\n"
    );
    assert_eq!(
        fs::read_to_string(root.path().join("sibling.txt")).unwrap(),
        "sibling\n"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn landlock_exact_executable_file_does_not_grant_sibling() {
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir().unwrap();
    for name in ["allowed.txt", "sibling.txt"] {
        let path = root.path().join(name);
        fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    }

    let status = run_exact_file_child("execute", root.path());
    if status.code() == Some(77) {
        eprintln!("Landlock unavailable; exact-file enforcement test skipped");
        return;
    }
    assert!(
        status.success(),
        "exact executable-file child failed: {status}"
    );
}
