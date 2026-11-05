//! Ordinary child-local home handles. This protocol grants no filesystem rights.
//! Only the live parent owner can answer a one-use, current-effect handoff.
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::linux::net::SocketAddrExt;
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::{SocketAddr, UnixStream};
use std::time::Duration;

pub(crate) const HANDOFF_ENV: &str = "SUBSTRATE_ORDINARY_HOME_HANDOFF";
// Parent callback time for mandatory current-effect checks, not a native run deadline.
pub(crate) const HOME_REPLY_TIMEOUT: Duration = Duration::from_secs(30);
// The child can wait in the listener queue before that callback starts.
// Reserve one additional callback window without extending authority validity.
const HOME_CHILD_REPLY_TIMEOUT: Duration = HOME_REPLY_TIMEOUT.saturating_mul(2);

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Locator {
    pub socket: String,
    pub nonce: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HomeReply {
    pub nonce: String,
    pub service_pid: u32,
    pub child_pid: u32,
    pub source_fd: i32,
    pub device: u64,
    pub inode: u64,
    pub uid: u32,
    pub config_name: String,
    pub config_sha256: String,
    pub sdk_temp: String,
    pub binary: String,
    pub args: Vec<String>,
}

pub(crate) fn peer_credentials(fd: RawFd) -> Result<libc::ucred> {
    let mut credentials: libc::ucred = unsafe { std::mem::zeroed() };
    let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    if unsafe {
        libc::getsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&mut credentials as *mut libc::ucred).cast(),
            &mut length,
        )
    } != 0
        || length as usize != std::mem::size_of::<libc::ucred>()
    {
        bail!("ordinary home peer credentials unavailable");
    }
    Ok(credentials)
}

pub(crate) fn require_direct_child(child: i32, parent: u32) -> Result<()> {
    if child <= 0 {
        bail!("ordinary home handoff omitted its child");
    }
    let stat = std::fs::read_to_string(format!("/proc/{child}/stat"))?;
    // The comm field can contain spaces and ')'; fields after its final ')' are fixed.
    let tail = stat
        .rsplit_once(')')
        .context("ordinary child stat shape")?
        .1;
    let actual: u32 = tail
        .split_whitespace()
        .nth(1)
        .context("ordinary child parent missing")?
        .parse()?;
    if actual != parent {
        bail!("ordinary home peer is not the issuer's direct child");
    }
    Ok(())
}

fn open_reply(reply: &HomeReply, expected_parent: u32, nonce: &str) -> Result<OwnedFd> {
    if reply.service_pid != expected_parent
        || reply.child_pid != std::process::id()
        || reply.nonce != nonce
        || reply.source_fd < 3
        || reply.uid != unsafe { libc::geteuid() }
        || !matches!(reply.config_name.as_str(), "config.toml" | "settings.json")
        || reply.config_sha256.len() != 64
        || !reply
            .config_sha256
            .bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    {
        bail!("ordinary home reply binding changed");
    }
    let source = format!("/proc/{expected_parent}/fd/{}", reply.source_fd);
    for key in ["HOME", "XDG_CONFIG_HOME"] {
        if std::env::var(key).ok().as_deref() != Some(source.as_str()) {
            bail!("ordinary home alias conflicts with its issued source");
        }
    }
    let harness_key = if reply.config_name == "config.toml" {
        "CODEX_HOME"
    } else {
        "CLAUDE_CONFIG_DIR"
    };
    if std::env::var(harness_key).ok().as_deref() != Some(source.as_str()) {
        bail!("ordinary harness home conflicts with its issued source");
    }
    // This one proc link is authenticated by the live direct-parent response.
    // It is not passed to the strict Landlock path resolver or made a policy root.
    let source = std::ffi::CString::new(source)?;
    let raw = unsafe {
        libc::open(
            source.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC,
        )
    };
    if raw < 0 {
        return Err(std::io::Error::last_os_error())
            .context("open issued ordinary home before restrictions");
    }
    let home = unsafe { std::fs::File::from_raw_fd(raw) };
    let metadata = home.metadata()?;
    if !metadata.is_dir()
        || (
            metadata.dev(),
            metadata.ino(),
            metadata.uid(),
            metadata.mode() & 0o777,
        ) != (reply.device, reply.inode, reply.uid, 0o700)
    {
        bail!("ordinary child-held home physical identity changed");
    }
    let physical = std::fs::read_link(format!("/proc/self/fd/{}", home.as_raw_fd()))?;
    let sdk_temp = std::path::Path::new(&reply.sdk_temp);
    if !physical.starts_with("/tmp")
        || !sdk_temp.starts_with("/tmp")
        || std::fs::canonicalize(&physical)? != physical
        || std::fs::canonicalize(sdk_temp)? != sdk_temp
    {
        bail!("ordinary home/output lack admitted existing scratch coverage");
    }
    let name = std::ffi::CString::new(reply.config_name.as_str())?;
    let raw = unsafe {
        libc::openat(
            home.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if raw < 0 {
        return Err(std::io::Error::last_os_error()).context("open issued ordinary config");
    }
    let config = unsafe { std::fs::File::from_raw_fd(raw) };
    let metadata = config.metadata()?;
    if !metadata.is_file()
        || metadata.uid() != reply.uid
        || metadata.mode() & 0o777 != 0o600
        || metadata.len() > 65536
    {
        bail!("ordinary child-held config shape changed");
    }
    let mut bytes = Vec::new();
    config.take(65537).read_to_end(&mut bytes)?;
    if format!("{:x}", Sha256::digest(&bytes)) != reply.config_sha256 {
        bail!("ordinary child-held config differs from current issued rendering");
    }
    use std::os::fd::IntoRawFd;
    Ok(unsafe { OwnedFd::from_raw_fd(home.into_raw_fd()) })
}

/// Called only before workload restriction and exec in the existing helper.
/// The returned child-local descriptor adds no Landlock permission.
pub(crate) struct OrdinaryNativeChildV1 {
    // Kept across helper restriction and final exec; only this owned child FD
    // has CLOEXEC cleared. It adds no filesystem permission.
    pub _home: OwnedFd,
    pub binary: String,
    pub args: Vec<String>,
}

pub(crate) fn consume_ordinary_home_handoff() -> Result<OrdinaryNativeChildV1> {
    let marker = std::env::var_os(HANDOFF_ENV).context("ordinary required handoff missing")?;
    let parent = unsafe { libc::getppid() } as u32;
    // Consume control input before any final native exec, including refusal.
    std::env::remove_var(HANDOFF_ENV);
    if std::env::var("SUBSTRATE_WORLD_FS_ISOLATION").is_ok_and(|v| v.eq_ignore_ascii_case("full"))
        || std::env::var("SUBSTRATE_MOUNT_FS_MODE")
            .is_ok_and(|v| v.eq_ignore_ascii_case("read_only"))
        || std::env::var_os(crate::enforcement_plan::WORLD_FS_ENFORCEMENT_PLAN_B64_ENV).is_some()
    {
        bail!("ordinary child-home handoff does not support this filesystem posture");
    }
    let marker = marker
        .into_string()
        .map_err(|_| anyhow::anyhow!("ordinary handoff is not UTF-8"))?;
    if marker.len() > 1024 {
        bail!("ordinary handoff locator too large");
    }
    let locator: Locator = serde_json::from_str(&marker)?;
    if locator.nonce.len() != 64
        || !locator
            .nonce
            .bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        || locator.socket != format!("substrate-ordinary-home-{}", &locator.nonce[..32])
    {
        bail!("ordinary handoff locator shape changed");
    }
    let address = SocketAddr::from_abstract_name(locator.socket.as_bytes())?;
    let mut stream = UnixStream::connect_addr(&address)?;
    stream.set_read_timeout(Some(HOME_CHILD_REPLY_TIMEOUT))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let peer = peer_credentials(stream.as_raw_fd())?;
    if peer.pid != parent as i32 || peer.uid != unsafe { libc::geteuid() } {
        bail!("ordinary handoff server is not this helper's live parent");
    }
    require_direct_child(std::process::id() as i32, parent)?;
    stream.write_all(locator.nonce.as_bytes())?;
    stream.shutdown(std::net::Shutdown::Write)?;
    let mut reply = Vec::new();
    stream.take(4097).read_to_end(&mut reply)?;
    if reply.len() > 4096 {
        bail!("ordinary handoff reply too large");
    }
    let reply: HomeReply = serde_json::from_slice(&reply)?;
    if reply.args != std::env::args().skip(2).collect::<Vec<_>>()
        || std::env::var("SUBSTRATE_E2_TURN_ACTUAL_BINARY")
            .ok()
            .as_deref()
            != Some(reply.binary.as_str())
        || !std::path::Path::new(&reply.binary).is_absolute()
    {
        bail!("ordinary issued executable argv differs from actual child invocation");
    }
    let home = open_reply(&reply, parent, &locator.nonce)?;
    let local = format!("/proc/self/fd/{}", home.as_raw_fd());
    for key in [
        "HOME",
        "XDG_CONFIG_HOME",
        if reply.config_name == "config.toml" {
            "CODEX_HOME"
        } else {
            "CLAUDE_CONFIG_DIR"
        },
    ] {
        std::env::set_var(key, &local);
    }
    let flags = unsafe { libc::fcntl(home.as_raw_fd(), libc::F_GETFD) };
    if flags < 0
        || unsafe { libc::fcntl(home.as_raw_fd(), libc::F_SETFD, flags & !libc::FD_CLOEXEC) } != 0
    {
        return Err(std::io::Error::last_os_error())
            .context("retain only child-local ordinary home across exec");
    }
    for key in [
        "SUBSTRATE_E2_TURN_ACTUAL_BINARY",
        "SUBSTRATE_E2_TURN_WORLD_ENTRY",
        "SUBSTRATE_WORLD_ENTRY_BINARY",
        "SUBSTRATE_LANDLOCK_HELPER_SRC",
        "SUBSTRATE_LANDLOCK_HELPER_PATH",
        "SUBSTRATE_INNER_CMD",
        "SUBSTRATE_INNER_LOGIN_SHELL",
    ] {
        std::env::remove_var(key);
    }
    Ok(OrdinaryNativeChildV1 {
        _home: home,
        binary: reply.binary,
        args: reply.args,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    use std::os::unix::net::UnixListener;
    use std::process::Command;

    const CHILD: &str =
        "internal_exec::ordinary_home_handoff::tests::test_ordinary_home_child_entry";

    fn record_ordinary_home_unit_tmp_for_test(path: &std::path::Path, kind: &str) {
        let Some(root) = std::env::var_os("SUBSTRATE_TEST_ORDINARY_EVIDENCE_ROOT") else {
            return;
        };
        let metadata = std::fs::symlink_metadata(path).unwrap();
        assert!(metadata.is_dir());
        assert_eq!(metadata.uid(), unsafe { libc::geteuid() });
        assert_eq!(metadata.mode() & 0o777, 0o700);
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join(format!("owned-unit-tmp-{kind}.json")),
            serde_json::to_vec(&serde_json::json!({"path":path,"dev":metadata.dev(),
                "ino":metadata.ino(),"uid":metadata.uid(),"mode":metadata.mode()}))
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    #[ignore = "owned subprocess of ordinary home protocol unit tests"]
    fn test_ordinary_home_child_entry() {
        if std::env::var("ORDINARY_HOME_TEST_MODE").as_deref() == Ok("foreign-server") {
            let socket = std::env::var("ORDINARY_HOME_TEST_SOCKET").unwrap();
            let listener = UnixListener::bind_addr(
                &SocketAddr::from_abstract_name(socket.as_bytes()).unwrap(),
            )
            .unwrap();
            std::fs::write(
                std::env::var_os("ORDINARY_HOME_TEST_READY").unwrap(),
                b"ready",
            )
            .unwrap();
            let (mut peer, _) = listener.accept().unwrap();
            peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
            let mut bytes = Vec::new();
            peer.read_to_end(&mut bytes).unwrap();
            assert!(
                bytes.is_empty(),
                "wrong-parent server must receive no handoff nonce"
            );
            return;
        }
        let consumer_started = std::time::Instant::now();
        let result = consume_ordinary_home_handoff();
        let effect = std::env::var_os("ORDINARY_HOME_TEST_EFFECT").unwrap();
        if std::env::var("ORDINARY_HOME_TEST_EXPECT").as_deref() == Ok("allow") {
            let child = result.unwrap();
            let home = child._home;
            assert!(std::env::var_os(HANDOFF_ENV).is_none());
            assert_eq!(
                std::env::var("HOME").unwrap(),
                format!("/proc/self/fd/{}", home.as_raw_fd())
            );
            assert_eq!(
                unsafe { libc::fcntl(home.as_raw_fd(), libc::F_GETFD) } & libc::FD_CLOEXEC,
                0
            );
            std::fs::write(
                std::path::Path::new(&std::env::var("HOME").unwrap()).join("child-write"),
                b"same held home",
            )
            .unwrap();
            std::fs::write(effect, b"after successful checked handoff").unwrap();
        } else {
            let error = result
                .err()
                .expect("actual consumer must refuse before the test effect");
            if std::env::var("ORDINARY_HOME_TEST_EXPECT").as_deref() == Ok("read-timeout") {
                let io = error.downcast_ref::<std::io::Error>().unwrap_or_else(|| {
                    panic!(
                        "withheld reply must retain the actual typed consumer I/O error: \
                             {error:#}; consumer_elapsed_ms={}",
                        consumer_started.elapsed().as_millis()
                    )
                });
                assert!(matches!(
                    io.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ));
                assert!(consumer_started.elapsed() >= HOME_CHILD_REPLY_TIMEOUT);
                println!(
                    "ORDINARY_HOME_REPLY_READ_FAILURE kind={:?} errno={:?} consumer_elapsed_ms={}",
                    io.kind(),
                    io.raw_os_error(),
                    consumer_started.elapsed().as_millis()
                );
            } else {
                println!("EXPECTED_REFUSAL:{error:#}");
            }
            assert!(!std::path::Path::new(&effect).exists());
        }
    }

    #[test]
    fn test_ordinary_home_handoff_refuses_bad_or_missing_inputs() {
        let temp = tempfile::Builder::new()
            .prefix("ordinary-home-negative-")
            .tempdir_in("/tmp")
            .unwrap();
        std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        record_ordinary_home_unit_tmp_for_test(temp.path(), "negative");
        let cross = format!("/proc/{}/fd/9", std::process::id());
        for (case, marker, posture) in [
            ("missing", None, None),
            ("stripped-repacked", None, None),
            ("malformed", Some("{}"), None),
            (
                "unknown",
                Some("{\"socket\":\"x\",\"nonce\":\"y\",\"extra\":true}"),
                None,
            ),
            (
                "full",
                Some("{}"),
                Some(("SUBSTRATE_WORLD_FS_ISOLATION", "full")),
            ),
            (
                "readonly",
                Some("{}"),
                Some(("SUBSTRATE_MOUNT_FS_MODE", "read_only")),
            ),
            (
                "deny",
                Some("{}"),
                Some((
                    crate::enforcement_plan::WORLD_FS_ENFORCEMENT_PLAN_B64_ENV,
                    "present",
                )),
            ),
        ] {
            let mut child = Command::new(std::env::current_exe().unwrap());
            child
                .args(["--exact", CHILD, "--ignored", "--nocapture"])
                .env("HOME", &cross)
                .env("CODEX_HOME", &cross)
                .env("XDG_CONFIG_HOME", &cross)
                .env("ORDINARY_HOME_TEST_EXPECT", "refuse")
                .env("ORDINARY_HOME_TEST_EFFECT", temp.path().join(case))
                .env_remove(HANDOFF_ENV)
                .env_remove("SUBSTRATE_WORLD_FS_ISOLATION")
                .env_remove("SUBSTRATE_MOUNT_FS_MODE")
                .env_remove(crate::enforcement_plan::WORLD_FS_ENFORCEMENT_PLAN_B64_ENV);
            if case == "stripped-repacked" {
                child
                    .env("HOME", "/tmp/repacked-home")
                    .env("CODEX_HOME", "/tmp/repacked-home")
                    .env("XDG_CONFIG_HOME", "/tmp/repacked-home");
            }
            if let Some(marker) = marker {
                child.env(HANDOFF_ENV, marker);
            }
            if let Some((key, value)) = posture {
                child.env(key, value);
            }
            let output = child.output().unwrap();
            assert!(
                output.status.success(),
                "{case}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(String::from_utf8_lossy(&output.stdout).contains("EXPECTED_REFUSAL:"));
            assert!(!temp.path().join(case).exists());
        }
    }

    #[test]
    fn test_ordinary_home_handoff_authenticates_parent_and_one_use() {
        let temp = tempfile::Builder::new()
            .prefix("ordinary-home-protocol-")
            .tempdir_in("/tmp")
            .unwrap();
        std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        record_ordinary_home_unit_tmp_for_test(temp.path(), "protocol");
        let config_path = temp.path().join("config.toml");
        std::fs::write(&config_path, b"synthetic checked config").unwrap();
        std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let held = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC)
            .open(temp.path())
            .unwrap();
        let metadata = held.metadata().unwrap();
        let source = format!("/proc/{}/fd/{}", std::process::id(), held.as_raw_fd());
        // These protocol peers are deliberately mock owners: they test actual
        // kernel parent/nonce/one-use joins, never claim sealed World admission.
        for case in [
            "allow",
            "delayed-allow",
            "delayed-wronginode",
            "wrongnonce",
            "wronghome",
            "wronginode",
            "wrongchild",
            "wrongservice",
            "changedconfig",
            "owner-refused",
            "reply-timeout",
            "queued-delayed-allow",
        ] {
            let nonce = format!(
                "{:x}",
                Sha256::digest(uuid::Uuid::now_v7().to_string().as_bytes())
            );
            let socket = format!("substrate-ordinary-home-{}", &nonce[..32]);
            let listener = UnixListener::bind_addr(
                &SocketAddr::from_abstract_name(socket.as_bytes()).unwrap(),
            )
            .unwrap();
            let source_fd = held.as_raw_fd();
            let effect = temp.path().join(format!("effect-{case}"));
            let withheld_effect = effect.clone();
            let expected = nonce.clone();
            let metadata = metadata.clone();
            let sdk_temp = temp.path().display().to_string();
            let (release_reply, child_terminal) = std::sync::mpsc::channel();
            let owner = std::thread::spawn(move || {
                if case == "queued-delayed-allow" {
                    std::thread::sleep(Duration::from_secs(8));
                }
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let peer = peer_credentials(stream.as_raw_fd()).unwrap();
                require_direct_child(peer.pid, std::process::id()).unwrap();
                let mut presented = Vec::new();
                (&mut stream).take(65).read_to_end(&mut presented).unwrap();
                if presented != expected.as_bytes() || case == "owner-refused" {
                    return;
                }
                if case == "reply-timeout" {
                    // Failure-only mock owner: no HomeReply, home FD, or effect permit.
                    // SO_RCVTIMEO may wake after its nominal duration. Keep
                    // this peer open until the actual child has been waited,
                    // so an early EOF cannot substitute for its I/O timeout.
                    child_terminal
                        .recv_timeout(HOME_CHILD_REPLY_TIMEOUT.saturating_mul(2))
                        .expect("timeout mock must receive the actual child terminal signal");
                    return;
                }
                let reply = HomeReply {
                    nonce: expected,
                    service_pid: std::process::id() + u32::from(case == "wrongservice"),
                    child_pid: peer.pid as u32 + u32::from(case == "wrongchild"),
                    source_fd,
                    device: metadata.dev(),
                    inode: metadata.ino()
                        + u64::from(matches!(case, "wronginode" | "delayed-wronginode")),
                    uid: metadata.uid(),
                    binary: "/bin/true".into(),
                    args: std::fs::read(format!("/proc/{}/cmdline", peer.pid))
                        .unwrap()
                        .split(|b| *b == 0)
                        .filter(|b| !b.is_empty())
                        .skip(2)
                        .map(|b| String::from_utf8(b.to_vec()).unwrap())
                        .collect(),
                    config_name: "config.toml".into(),
                    config_sha256: format!(
                        "{:x}",
                        Sha256::digest(if case == "changedconfig" {
                            b"different".as_slice()
                        } else {
                            b"synthetic checked config".as_slice()
                        })
                    ),
                    sdk_temp,
                };
                if case == "queued-delayed-allow" {
                    assert!(Duration::from_secs(26) < HOME_REPLY_TIMEOUT);
                    std::thread::sleep(Duration::from_secs(26));
                    assert!(!withheld_effect.exists());
                }
                if case.starts_with("delayed-") {
                    // Reproduce a valid owner's reply taking longer than the
                    // former five-second child timeout; metadata checks remain.
                    std::thread::sleep(Duration::from_secs(6));
                    assert!(
                        !withheld_effect.exists(),
                        "no effect before an issued reply"
                    );
                }
                stream
                    .write_all(&serde_json::to_vec(&reply).unwrap())
                    .unwrap();
            });
            let mut presented = nonce.clone();
            if case == "wrongnonce" {
                presented.replace_range(63..64, if &presented[63..] == "0" { "1" } else { "0" });
            }
            let locator = serde_json::to_string(&Locator {
                socket: socket.clone(),
                nonce: presented,
            })
            .unwrap();
            let output = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", CHILD, "--ignored", "--nocapture"])
                .env(HANDOFF_ENV, &locator)
                .env(
                    "HOME",
                    if case == "wronghome" {
                        "/tmp/wrong-home"
                    } else {
                        &source
                    },
                )
                .env("SUBSTRATE_E2_TURN_ACTUAL_BINARY", "/bin/true")
                .env("CODEX_HOME", &source)
                .env("XDG_CONFIG_HOME", &source)
                .env(
                    "ORDINARY_HOME_TEST_EXPECT",
                    if matches!(case, "allow" | "delayed-allow" | "queued-delayed-allow") {
                        "allow"
                    } else if case == "reply-timeout" {
                        "read-timeout"
                    } else {
                        "refuse"
                    },
                )
                .env("ORDINARY_HOME_TEST_EFFECT", &effect)
                .env_remove("SUBSTRATE_WORLD_FS_ISOLATION")
                .env_remove("SUBSTRATE_MOUNT_FS_MODE")
                .env_remove(crate::enforcement_plan::WORLD_FS_ENFORCEMENT_PLAN_B64_ENV)
                .output()
                .unwrap();
            let released = (case == "reply-timeout").then(|| release_reply.send(()));
            let owner_joined = owner.join();
            // Retain the actual consumer error even if the old peer already
            // closed and the mock owner's write consequently failed too.
            assert!(
                output.status.success(),
                "{case}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            owner_joined.unwrap();
            if let Some(released) = released {
                released.unwrap();
            }
            assert_eq!(
                effect.exists(),
                matches!(case, "allow" | "delayed-allow" | "queued-delayed-allow")
            );
            if matches!(case, "allow" | "delayed-allow" | "queued-delayed-allow") {
                assert_eq!(
                    std::fs::read(temp.path().join("child-write")).unwrap(),
                    b"same held home"
                );
                let replay = Command::new(std::env::current_exe().unwrap())
                    .args(["--exact", CHILD, "--ignored", "--nocapture"])
                    .env(HANDOFF_ENV, locator)
                    .env("HOME", &source)
                    .env("SUBSTRATE_E2_TURN_ACTUAL_BINARY", "/bin/true")
                    .env("CODEX_HOME", &source)
                    .env("XDG_CONFIG_HOME", &source)
                    .env("ORDINARY_HOME_TEST_EXPECT", "refuse")
                    .env(
                        "ORDINARY_HOME_TEST_EFFECT",
                        temp.path().join("replayed-effect"),
                    )
                    .output()
                    .unwrap();
                assert!(replay.status.success());
                assert!(!temp.path().join("replayed-effect").exists());
            }
        }
        // An independently running same-UID server is insufficient, even with
        // the exact route/nonce and perfectly valid owned home metadata.
        let nonce = format!(
            "{:x}",
            Sha256::digest(uuid::Uuid::now_v7().to_string().as_bytes())
        );
        let socket = format!("substrate-ordinary-home-{}", &nonce[..32]);
        let ready = temp.path().join("foreign-ready");
        let mut server = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD, "--ignored", "--nocapture"])
            .env("ORDINARY_HOME_TEST_MODE", "foreign-server")
            .env("ORDINARY_HOME_TEST_SOCKET", &socket)
            .env("ORDINARY_HOME_TEST_READY", &ready)
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while !ready.exists() {
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(10));
        }
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD, "--ignored", "--nocapture"])
            .env(
                HANDOFF_ENV,
                serde_json::to_string(&Locator { socket, nonce }).unwrap(),
            )
            .env("HOME", &source)
            .env("SUBSTRATE_E2_TURN_ACTUAL_BINARY", "/bin/true")
            .env("CODEX_HOME", &source)
            .env("XDG_CONFIG_HOME", &source)
            .env("ORDINARY_HOME_TEST_EXPECT", "refuse")
            .env(
                "ORDINARY_HOME_TEST_EFFECT",
                temp.path().join("foreign-effect"),
            )
            .output()
            .unwrap();
        assert!(server.wait().unwrap().success());
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("not this helper's live parent"));
        assert!(!temp.path().join("foreign-effect").exists());
    }
}
