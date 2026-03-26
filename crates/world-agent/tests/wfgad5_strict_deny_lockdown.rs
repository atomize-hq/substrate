#![cfg(all(unix, target_os = "linux"))]

use agent_api_types::{
    ExecuteRequest, PolicySnapshotV3, PolicySnapshotWorldFsDimensionV3,
    PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3,
    WorldFsDenyEnforcementV3,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use substrate_broker::{set_global_broker, BrokerHandle};
use tempfile::tempdir;
use tokio::runtime::Runtime;
use world_agent::WorldAgentService;

fn decode(b64: &str) -> String {
    String::from_utf8_lossy(
        &BASE64
            .decode(b64.as_bytes())
            .unwrap_or_else(|_| Vec::from(b"<invalid base64>")),
    )
    .into_owned()
}

fn combined_output(resp: &agent_api_types::ExecuteResponse) -> String {
    format!("{}{}", decode(&resp.stdout_b64), decode(&resp.stderr_b64))
}

fn overlay_available() -> bool {
    (unsafe { libc::geteuid() == 0 })
        && fs::read_to_string("/proc/filesystems")
            .map(|data| data.contains("overlay"))
            .unwrap_or(false)
}

fn snapshot_strict_read_only_with_read_deny(deny_list: Vec<String>) -> PolicySnapshotV3 {
    PolicySnapshotV3 {
        schema_version: 3,
        net_allowed: Vec::new(),
        world_fs: PolicySnapshotWorldFsV3 {
            host_visible: false,
            fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: true },
            deny_enforcement: Some(WorldFsDenyEnforcementV3::Strict),
            caged_required: false,
            discover: None,
            read: Some(PolicySnapshotWorldFsDimensionV3 {
                allow_list: vec![".".to_string()],
                deny_list,
            }),
            write: PolicySnapshotWorldFsWriteV3 {
                enabled: false,
                allow_list: vec![".".to_string()],
                deny_list: Vec::new(),
            },
        },
    }
}

fn execute_non_pty(
    service: &WorldAgentService,
    cwd: &Path,
    cmd: &str,
    env: HashMap<String, String>,
    policy_snapshot: PolicySnapshotV3,
) -> Option<agent_api_types::ExecuteResponse> {
    let req = ExecuteRequest {
        profile: None,
        cmd: cmd.to_string(),
        cwd: Some(cwd.display().to_string()),
        env: Some(env),
        pty: false,
        agent_id: "wfgad5-test".to_string(),
        budget: None,
        policy_snapshot,
        world_network: None,
        world_fs_mode: None,
    };

    let rt = Runtime::new().expect("runtime");
    match rt.block_on(service.execute(req)) {
        Ok(resp) => Some(resp),
        Err(err) => {
            eprintln!("skipping WFGAD5 test: execute failed: {err}");
            None
        }
    }
}

#[test]
fn strict_mode_blocks_mount_syscalls_with_eperm_and_denies_remain_enforced() {
    if !overlay_available() {
        eprintln!("skipping WFGAD5 test: overlay support or privileges missing");
        return;
    }

    let _ = set_global_broker(BrokerHandle::new());
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping WFGAD5 test: service init failed: {err}");
            return;
        }
    };

    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    fs::create_dir_all(cwd.join("secrets")).expect("create secrets dir");
    fs::write(cwd.join("secrets/secret.txt"), b"super secret\n").expect("write secret file");
    fs::create_dir_all(cwd.join("docs")).expect("create docs dir");
    fs::write(cwd.join("docs/public.txt"), b"public\n").expect("write public file");

    let policy_snapshot =
        snapshot_strict_read_only_with_read_deny(vec!["./secrets/**".to_string()]);

    let mut env = HashMap::new();
    env.insert("SUBSTRATE_WORLD_REQUIRE_WORLD".to_string(), "1".to_string());

    let resp = match execute_non_pty(
        &service,
        &cwd,
        r#"sh -lc 'set -eu
seccomp="$(awk '\''/^Seccomp:/ { print $2 }'\'' /proc/self/status 2>/dev/null || true)"
echo "SECCOMP=$seccomp"
if [ "$seccomp" != "2" ]; then
  echo "EXPECTED_SECCOMP_2_GOT=$seccomp"
  exit 60
fi

mp="${SUBSTRATE_MOUNT_PROJECT_DIR:-}"
if [ -z "$mp" ]; then
  echo MISSING_SUBSTRATE_MOUNT_PROJECT_DIR
  exit 61
fi

set +e
umount_err="$(umount /project/secrets 2>&1)"
umount_rc=$?
set -e
echo "UMOUNT_RC=$umount_rc"
echo "UMOUNT_ERR=$umount_err"
if [ "$umount_rc" -eq 0 ]; then
  echo UNEXPECTED_UMOUNT_OK
  exit 62
fi
echo "$umount_err" | grep -qi "Operation not permitted" || {
  echo "EXPECTED_EPERM_BUT_GOT=$umount_err"
  exit 63
}

set +e
nested_err="$(sh -lc '\''umount /project/secrets'\'' 2>&1)"
nested_rc=$?
set -e
echo "NESTED_UMOUNT_RC=$nested_rc"
echo "NESTED_UMOUNT_ERR=$nested_err"
if [ "$nested_rc" -eq 0 ]; then
  echo UNEXPECTED_NESTED_UMOUNT_OK
  exit 64
fi
echo "$nested_err" | grep -qi "Operation not permitted" || {
  echo "EXPECTED_NESTED_EPERM_BUT_GOT=$nested_err"
  exit 65
}

if command -v mount >/dev/null 2>&1; then
  set +e
  mount_err="$(mount --bind /project/docs /project/secrets 2>&1)"
  mount_rc=$?
  set -e
  echo "MOUNT_RC=$mount_rc"
  echo "MOUNT_ERR=$mount_err"
  if [ "$mount_rc" -eq 0 ]; then
    echo UNEXPECTED_MOUNT_OK
    exit 66
  fi
  echo "$mount_err" | grep -qi "Operation not permitted" || {
    echo "EXPECTED_MOUNT_EPERM_BUT_GOT=$mount_err"
    exit 67
  }
else
  echo MISSING_MOUNT_BINARY
fi

err_file="$(mktemp /tmp/substrate-wfgad5-eacces.XXXXXX)"
trap '\''rm -f "$err_file"'\'' EXIT

set +e
cat ./secrets/secret.txt >/dev/null 2>"$err_file"
rc=$?
set -e
if [ "$rc" -eq 0 ]; then
  echo UNEXPECTED_SECRET_READ_OK
  exit 68
fi
grep -qi "Permission denied" "$err_file" || {
  echo "EXPECTED_EACCES_BUT_GOT: $(cat "$err_file" || true)"
  exit 69
}

rm -f "$err_file"
set +e
cat "$mp/secrets/secret.txt" >/dev/null 2>"$err_file"
rc=$?
set -e
if [ "$rc" -eq 0 ]; then
  echo UNEXPECTED_MOUNT_VIEW_SECRET_READ_OK
  exit 70
fi
grep -qi "Permission denied" "$err_file" || {
  echo "EXPECTED_EACCES_MOUNT_VIEW_BUT_GOT: $(cat "$err_file" || true)"
  exit 71
}

cat ./docs/public.txt >/dev/null
echo OK
'"#,
        env,
        policy_snapshot,
    ) {
        Some(resp) => resp,
        None => return,
    };

    let out = combined_output(&resp);
    assert_eq!(
        resp.exit, 0,
        "expected strict-mode lockdown to block umount/mount with EPERM while preserving read denies; exit={} output={out:?}",
        resp.exit
    );
    assert!(
        out.contains("OK"),
        "expected OK marker from inner script; output={out:?}"
    );
}
