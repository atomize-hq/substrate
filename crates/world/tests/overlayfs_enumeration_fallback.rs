#![cfg(all(unix, target_os = "linux"))]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use tempfile::TempDir;
use world::overlayfs::OverlayFs;

fn overlay_available() -> bool {
    (unsafe { libc::geteuid() == 0 })
        && fs::read_to_string("/proc/filesystems")
            .map(|data| data.contains("overlay"))
            .unwrap_or(false)
}

fn fuse_available() -> bool {
    std::path::Path::new("/dev/fuse").exists() && which::which("fuse-overlayfs").is_ok()
}

fn run_ls_a1(path: &std::path::Path, path_override: Option<&std::path::Path>) -> String {
    let mut cmd = Command::new("ls");
    cmd.arg("-a1").current_dir(path);
    if let Some(prefix) = path_override {
        let prev = std::env::var("PATH").unwrap_or_default();
        cmd.env("PATH", format!("{}:{}", prefix.display(), prev));
    }
    let output = cmd.output().expect("ls -a1");
    assert!(
        output.status.success(),
        "ls -a1 should succeed: status={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn simulated_primary_enumeration_probe_failure_falls_back_to_fuse_mount() {
    if !overlay_available() {
        eprintln!("skipping: overlay support or privileges missing");
        return;
    }
    if !fuse_available() {
        eprintln!("skipping: fuse-overlayfs unavailable (/dev/fuse or binary missing)");
        return;
    }

    let temp = TempDir::new().expect("tempdir");
    let lower = temp.path().join("lower");
    fs::create_dir_all(&lower).expect("create lower dir");
    fs::write(lower.join("seed.txt"), b"seed").expect("seed lower dir");

    let mut primary = OverlayFs::new("enum_primary").expect("overlay");
    let merged = match primary.mount(&lower) {
        Ok(path) => path,
        Err(err) => {
            eprintln!("skipping: kernel overlay mount unavailable in this environment: {err:#}");
            return;
        }
    };

    let probe_file = ".substrate_enum_probe";
    fs::write(merged.join(probe_file), b"probe").expect("write probe file");

    let stub_bin_dir = temp.path().join("stub-bin");
    fs::create_dir_all(&stub_bin_dir).expect("create stub bin dir");
    let stub_ls = stub_bin_dir.join("ls");
    fs::write(&stub_ls, "#!/usr/bin/env bash\necho .\necho ..\n").expect("write stub ls");
    let mut perms = fs::metadata(&stub_ls)
        .expect("stub ls metadata")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&stub_ls, perms).expect("chmod stub ls");

    let listing = run_ls_a1(&merged, Some(&stub_bin_dir));
    assert!(
        !listing.lines().any(|line| line.trim() == probe_file),
        "expected simulated enumeration probe failure (stub ls hid probe file): listing={listing:?}"
    );

    primary.cleanup().expect("cleanup primary overlay");

    let mut fallback = OverlayFs::new("enum_fallback").expect("overlay");
    let merged = fallback
        .mount_fuse_only(&lower)
        .expect("mount via fuse-only");
    assert!(
        fallback.is_using_fuse(),
        "expected fuse-only mount to report using_fuse=true"
    );

    fs::write(merged.join(probe_file), b"probe").expect("write probe file (fuse)");
    let listing = run_ls_a1(&merged, None);
    assert!(
        listing.lines().any(|line| line.trim() == probe_file),
        "expected enumeration probe to succeed on fuse mount: listing={listing:?}"
    );
}
