#![cfg(all(unix, target_os = "linux"))]

use std::fs;
use std::process::Command;

fn landlock_supported() -> bool {
    world::landlock::detect_support().supported
}

fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[test]
fn landlock_exec_subprocess_entry() {
    if std::env::var("SUBSTRATE_TEST_RUN_LANDLOCK_EXEC")
        .ok()
        .as_deref()
        != Some("1")
    {
        return;
    }

    match world_agent::internal_exec::run_landlock_exec() {
        Ok(()) => {
            eprintln!("unexpected success: landlock exec wrapper returned Ok(())");
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("unexpected error return (expected process exit): {err:#}");
            std::process::exit(1);
        }
    }
}

#[test]
fn discover_allowlist_allows_listing_but_blocks_reading_files() {
    if !landlock_supported() {
        eprintln!("skipping discover allowlist test: Landlock unsupported");
        return;
    }
    if !is_root() {
        eprintln!("skipping discover allowlist test: requires root (full-isolation helper base allowlists include root-owned paths)");
        return;
    }

    let tmp = tempfile::Builder::new()
        .prefix("substrate-wfgad4-discover-")
        .tempdir_in("/var/tmp")
        .or_else(|_| tempfile::tempdir())
        .expect("tempdir");

    let secrets_dir = tmp.path().join("secrets");
    fs::create_dir_all(&secrets_dir).expect("create secrets dir");
    fs::write(secrets_dir.join("secret.txt"), b"super secret\n").expect("write secret file");

    let exe = std::env::current_exe().expect("current_exe");
    let uid = uuid::Uuid::now_v7().to_string();
    let test_root = tmp.path().display().to_string();

    let inner_cmd = r#"set -eu
dir="$SUBSTRATE_TEST_DISCOVER_DIR"
ls "$dir/secrets" | grep -qx "secret.txt"

err="/tmp/substrate-landlock-err-$SUBSTRATE_TEST_UUID"
rm -f "$err"

set +e
cat "$dir/secrets/secret.txt" >/dev/null 2>"$err"
status=$?
set -e

if [ "$status" -eq 0 ]; then
  echo UNEXPECTED_READ
  exit 42
fi

grep -q "Permission denied" "$err" || {
  echo "EXPECTED_EACCES_BUT_GOT: $(cat "$err" || true)"
  exit 43
}

rm -f "$err"
echo READ_BLOCKED
"#;

    let output = Command::new(exe)
        .arg("landlock_exec_subprocess_entry")
        .arg("--exact")
        .arg("--nocapture")
        .env("SUBSTRATE_TEST_RUN_LANDLOCK_EXEC", "1")
        .env("SUBSTRATE_WORLD_FS_ISOLATION", "full")
        .env("SUBSTRATE_MOUNT_FS_MODE", "read_only")
        .env("SUBSTRATE_MOUNT_PROJECT_DIR", "/project")
        .env("SUBSTRATE_MOUNT_CWD", "/")
        .env("SUBSTRATE_INNER_CMD", inner_cmd)
        .env("SUBSTRATE_INNER_LOGIN_SHELL", "0")
        .env("SUBSTRATE_TEST_DISCOVER_DIR", test_root)
        .env("SUBSTRATE_TEST_UUID", uid)
        // Trigger Landlock restriction application, while keeping our test dir out of the read allowlist.
        .env("SUBSTRATE_WORLD_FS_LANDLOCK_READ_ALLOWLIST", "/proc")
        // WFGAD4 contract: discover allowlist must allow READ_DIR without implying READ_FILE.
        .env(
            "SUBSTRATE_WORLD_FS_LANDLOCK_DISCOVER_ALLOWLIST",
            tmp.path().display().to_string(),
        )
        .output()
        .expect("run world-agent landlock exec wrapper");

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected discover to allow listing but block reads; status={:?} stdout={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
