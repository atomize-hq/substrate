use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Value};
use std::ffi::CString;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, OwnedFd, RawFd};
use std::os::unix::fs::{FileTypeExt, OpenOptionsExt, PermissionsExt};
use std::os::unix::net::{UnixDatagram, UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use substrate_common::{
    canonical_action_receipt_bytes_v1, canonical_guest_publisher_pairing_ticket_v1,
    canonical_lifecycle_publisher_protected_state_v1, canonical_lifecycle_signature_payload_v1,
    canonical_managed_action_prepared_record_v1, lifecycle_anchor_sha256_v1,
    managed_action_prepared_record_sha256_v1, parse_p256_spki_der_v1,
    validate_guest_publisher_pairing_ticket_v1, validate_lifecycle_publisher_protected_state_v1,
    validate_managed_action_receipt_signature_v1, validate_managed_lifecycle_publisher_request_v1,
    validate_publisher_bootstrap_authorization_v1, verify_lifecycle_signature_v1,
    verify_p256_p1363_low_s_v1, ExecutorBuildEvidenceV1, GuestPublisherBootstrapHelloV1,
    GuestPublisherBootstrapTranscriptV1, GuestPublisherPairingGuestIntentV1,
    GuestPublisherPairingTicketV1, LifecyclePublisherAnchorV1, LifecyclePublisherProtectedStateV1,
    LifecycleSignatureV1, ManagedActionPreparedRecordV1, ManagedActionReceiptV1, ManagedActionV1,
    ManagedArtifactIdentityV1, ManagedExecutorIdentityV1, ManagedLifecyclePublisherRequestV1,
    PublisherBootstrapAuthorizationV1,
};

const DEFAULT_STATE_ROOT: &str = "/var/lib/substrate";
const DEFAULT_ENDPOINT_PATH: &str = "/run/substrate-lifecycle-publisher-v1.sock";
const DEFAULT_EXECUTOR_PATH: &str = "/usr/libexec/substrate/substrate-lifecycle-linux";
const DEFAULT_SERVICE_UNIT_PATH: &str =
    "/etc/systemd/system/substrate-lifecycle-publisher-v1.service";
const DEFAULT_SOCKET_UNIT_PATH: &str =
    "/etc/systemd/system/substrate-lifecycle-publisher-v1.socket";
const DEFAULT_WORLD_ENDPOINT_PATH: &str = "/run/substrate.sock";
const DEFAULT_WORLD_SERVICE_UNIT: &str = "substrate-world-service.service";
const DEFAULT_WORLD_SOCKET_UNIT: &str = "substrate-world-service.socket";
const DEFAULT_LIFECYCLE_SERVICE_UNIT: &str = "substrate-lifecycle-publisher-v1.service";
const DEFAULT_LIFECYCLE_SOCKET_UNIT: &str = "substrate-lifecycle-publisher-v1.socket";
const GUEST_PAIRING_LITERAL_V1: &str = "PAIR EXACT SUBSTRATE GUEST PUBLISHER";
const PUBLISHER_PROBE_TIMEOUT_V1: Duration = Duration::from_secs(1);
const PUBLISHER_PING_SCHEMA_OWNER_V1: &str = "substrate.lifecycle-publisher-ping";
const PUBLISHER_PING_RESPONSE_SCHEMA_OWNER_V1: &str = "substrate.lifecycle-publisher-ping-response";
const LISTEN_FDS_START: RawFd = 3;
const MAX_FRAME_BYTES: usize = 1024 * 1024;
const ED25519_SPKI_PREFIX: &[u8] = &[
    0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
];

const AF_UNIX_V1: i32 = 1;
const SOCK_SEQPACKET_V1: i32 = 5;
const SOCK_CLOEXEC_V1: i32 = 0o2000000;
const SOL_SOCKET_V1: i32 = 1;
const SO_TYPE_V1: i32 = 3;
const SO_PEERCRED_V1: i32 = 17;
const SHUT_WR_V1: i32 = 1;
const STDIN_FILENO_V1: RawFd = 0;
const O_CLOEXEC_V1: i32 = 0o2000000;
const O_DIRECTORY_V1: i32 = 0o200000;
const O_NOFOLLOW_V1: i32 = 0o400000;
const O_PATH_V1: i32 = 0o10000000;
const O_RDWR_V1: i32 = 0o2;
const O_TMPFILE_V1: i32 = 0o20000000 | O_DIRECTORY_V1;
const AT_EMPTY_PATH_V1: i32 = 0x1000;
const LOCK_EX_V1: i32 = 2;
const LOCK_NB_V1: i32 = 4;
const SINGLE_FRAME_FINISH_TIMEOUT_V1: Duration = Duration::from_secs(1);

#[repr(C)]
struct SockAddrUnV1 {
    sun_family: u16,
    sun_path: [i8; 108],
}

#[repr(C)]
struct UCredV1 {
    pid: i32,
    uid: u32,
    gid: u32,
}

unsafe extern "C" {
    fn socket(domain: i32, kind: i32, protocol: i32) -> i32;
    #[cfg(test)]
    fn socketpair(domain: i32, kind: i32, protocol: i32, sv: *mut i32) -> i32;
    fn shutdown(fd: i32, how: i32) -> i32;
    fn dup(fd: i32) -> i32;
    fn bind(fd: i32, addr: *const core::ffi::c_void, len: u32) -> i32;
    fn listen(fd: i32, backlog: i32) -> i32;
    fn connect(fd: i32, addr: *const core::ffi::c_void, len: u32) -> i32;
    fn getsockopt(
        fd: i32,
        level: i32,
        optname: i32,
        optval: *mut core::ffi::c_void,
        optlen: *mut u32,
    ) -> i32;
    fn open(path: *const i8, flags: i32, mode: u32) -> i32;
    fn flock(fd: i32, operation: i32) -> i32;
    fn linkat(
        olddirfd: i32,
        oldpath: *const i8,
        newdirfd: i32,
        newpath: *const i8,
        flags: i32,
    ) -> i32;
    fn isatty(fd: i32) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransportKindV1 {
    SeqPacket,
    Stream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KillPointV1 {
    None,
    Prepare,
    Observe,
    Receipt,
    Intent,
    Hello,
    Transcript,
    Commit,
}

#[derive(Debug, Clone)]
struct LinuxManagedArtifactExecutorV1 {
    state_root: PathBuf,
    lifecycle_container: PathBuf,
    publisher_directory: PathBuf,
    protected_state_path: PathBuf,
    signing_key_path: PathBuf,
    endpoint_path: PathBuf,
    executor_path: PathBuf,
    service_unit_path: PathBuf,
    socket_unit_path: PathBuf,
    tty_path: PathBuf,
    transport: TransportKindV1,
}

#[derive(Debug, Clone)]
struct LinuxPeerAttestationV1 {
    pid: u32,
    uid: u32,
    gid: u32,
    exe_path: String,
}

#[derive(Debug, Clone)]
struct ServiceStateObservationV1 {
    active: String,
    enabled: String,
    endpoint: Value,
}

#[derive(Debug)]
struct PublisherAttemptLockV1 {
    _file: File,
}

impl LinuxManagedArtifactExecutorV1 {
    fn new(
        state_root: Option<PathBuf>,
        endpoint_path: Option<PathBuf>,
        tty_path: Option<PathBuf>,
        transport: TransportKindV1,
    ) -> Self {
        let state_root = state_root.unwrap_or_else(|| PathBuf::from(DEFAULT_STATE_ROOT));
        let lifecycle_container = state_root.join(".substrate-lifecycle-v1");
        let publisher_directory = lifecycle_container.join("publisher");
        Self {
            state_root,
            lifecycle_container: lifecycle_container.clone(),
            publisher_directory: publisher_directory.clone(),
            protected_state_path: publisher_directory.join("current-anchor.v1.json"),
            signing_key_path: publisher_directory.join("signing-key.v1"),
            endpoint_path: endpoint_path.unwrap_or_else(|| PathBuf::from(DEFAULT_ENDPOINT_PATH)),
            executor_path: PathBuf::from(DEFAULT_EXECUTOR_PATH),
            service_unit_path: PathBuf::from(DEFAULT_SERVICE_UNIT_PATH),
            socket_unit_path: PathBuf::from(DEFAULT_SOCKET_UNIT_PATH),
            tty_path: tty_path.unwrap_or_else(|| PathBuf::from("/dev/tty")),
            transport,
        }
    }
}

fn usage_error_v1() -> Result<()> {
    bail!(
        "usage: substrate-lifecycle-linux <bootstrap-publisher|run-publisher|submit-request|relay-request|begin-guest-bootstrap|commit-guest-bootstrap|retire-test-publisher|service-state>"
    )
}

fn main() -> Result<()> {
    let _ = env_logger::try_init();
    main_impl_v1()
}

fn main_impl_v1() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        return usage_error_v1();
    };

    let mut state_root: Option<PathBuf> = None;
    let mut endpoint_path: Option<PathBuf> = None;
    let mut tty_path: Option<PathBuf> = None;
    let mut transport = TransportKindV1::SeqPacket;
    let mut kill_point = KillPointV1::None;
    let mut publisher_fd: Option<RawFd> = None;
    let mut relay_fd: Option<RawFd> = None;
    let mut service_unit_name: Option<String> = None;
    let mut service_action: Option<String> = None;
    let mut ticket_path: Option<PathBuf> = None;
    let mut transcript_path: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--state-root" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--state-root requires a value"))?;
                state_root = Some(PathBuf::from(value));
            }
            "--socket" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--socket requires a value"))?;
                endpoint_path = Some(PathBuf::from(value));
            }
            "--tty-path" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--tty-path requires a value"))?;
                tty_path = Some(PathBuf::from(value));
            }
            "--transport" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--transport requires a value"))?;
                transport = match value.as_str() {
                    "seqpacket" => TransportKindV1::SeqPacket,
                    "stream" => TransportKindV1::Stream,
                    _ => bail!("unknown transport {value}"),
                };
            }
            "--kill-at" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--kill-at requires a value"))?;
                kill_point = match value.as_str() {
                    "prepare" => KillPointV1::Prepare,
                    "observe" => KillPointV1::Observe,
                    "receipt" => KillPointV1::Receipt,
                    "intent" => KillPointV1::Intent,
                    "hello" => KillPointV1::Hello,
                    "transcript" => KillPointV1::Transcript,
                    "commit" => KillPointV1::Commit,
                    "none" => KillPointV1::None,
                    _ => bail!("unknown kill point {value}"),
                };
            }
            "--publisher-bootstrap-fd" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--publisher-bootstrap-fd requires a value"))?;
                publisher_fd = Some(
                    value
                        .parse::<RawFd>()
                        .with_context(|| format!("parse bootstrap fd {value}"))?,
                );
            }
            "--relay-fd" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--relay-fd requires a value"))?;
                relay_fd = Some(
                    value
                        .parse::<RawFd>()
                        .with_context(|| format!("parse relay fd {value}"))?,
                );
            }
            "--service-unit" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--service-unit requires a value"))?;
                service_unit_name = Some(value);
            }
            "--action" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--action requires a value"))?;
                service_action = Some(value);
            }
            "--ticket-file" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--ticket-file requires a value"))?;
                ticket_path = Some(PathBuf::from(value));
            }
            "--transcript-file" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--transcript-file requires a value"))?;
                transcript_path = Some(PathBuf::from(value));
            }
            other => bail!("unknown argument {other}"),
        }
    }

    let executor =
        LinuxManagedArtifactExecutorV1::new(state_root, endpoint_path, tty_path, transport);
    match command.as_str() {
        "bootstrap-publisher" => {
            let response = if let Some(fd) = publisher_fd {
                let bytes = read_frame_from_fd_v1(fd)?;
                let authorization: PublisherBootstrapAuthorizationV1 =
                    serde_json::from_slice(&bytes).context("decode bootstrap authorization")?;
                bootstrap_linux_publisher_v1(&executor, &authorization, kill_point)?
            } else {
                let authorization: PublisherBootstrapAuthorizationV1 =
                    serde_json::from_slice(&read_bytes_from_stdin_v1()?)
                        .context("decode bootstrap authorization")?;
                bootstrap_linux_publisher_v1(&executor, &authorization, kill_point)?
            };
            if let Some(fd) = publisher_fd {
                write_json_frame_to_fd_v1(fd, &response)?;
            } else {
                print_json_line_v1(&response)?;
            }
        }
        "run-publisher" => {
            run_linux_publisher_v1(&executor, kill_point)?;
        }
        "submit-request" => {
            let request: ManagedLifecyclePublisherRequestV1 =
                serde_json::from_slice(&read_bytes_from_stdin_v1()?)
                    .context("decode publisher request")?;
            let response = submit_request_direct_v1(&executor, &request)?;
            print_json_line_v1(&response)?;
        }
        "relay-request" => {
            let fd = relay_fd.ok_or_else(|| anyhow!("relay-request requires --relay-fd"))?;
            relay_linux_publisher_request_v1(&executor, fd)?;
        }
        "begin-guest-bootstrap" => {
            let ticket = load_ticket_input_v1(ticket_path.as_deref())?;
            let hello = begin_guest_publisher_bootstrap_v1(&executor, &ticket, kill_point)?;
            print_json_line_v1(&serde_json::to_value(hello).context("serialize hello")?)?;
        }
        "commit-guest-bootstrap" => {
            let ticket = load_ticket_input_v1(ticket_path.as_deref())?;
            let transcript = load_transcript_input_v1(transcript_path.as_deref())?;
            let anchor =
                commit_guest_publisher_bootstrap_v1(&executor, &ticket, &transcript, kill_point)?;
            print_json_line_v1(&serde_json::to_value(anchor).context("serialize anchor")?)?;
        }
        "retire-test-publisher" => {
            retire_linux_test_publisher_v1(&executor)?;
            print_json_line_v1(&json!({"retired": true, "state_root": executor.state_root}))?;
        }
        "service-state" => {
            let unit = service_unit_name
                .ok_or_else(|| anyhow!("service-state requires --service-unit"))?;
            let action =
                service_action.ok_or_else(|| anyhow!("service-state requires --action"))?;
            let response = execute_service_state_action_v1(&executor, &unit, &action, kill_point)?;
            print_json_line_v1(&response)?;
        }
        _ => return usage_error_v1(),
    }
    Ok(())
}

fn print_json_line_v1(value: &Value) -> Result<()> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    serde_json::to_writer(&mut handle, value).context("write JSON")?;
    writeln!(handle).context("terminate JSON line")?;
    Ok(())
}

fn load_ticket_input_v1(path: Option<&Path>) -> Result<GuestPublisherPairingTicketV1> {
    let bytes = if let Some(path) = path {
        fs::read(path).with_context(|| format!("read ticket {}", path.display()))?
    } else {
        read_bytes_from_stdin_v1()?
    };
    let ticket: GuestPublisherPairingTicketV1 =
        serde_json::from_slice(&bytes).context("decode pairing ticket")?;
    validate_guest_publisher_pairing_ticket_v1(&ticket)?;
    Ok(ticket)
}

fn load_transcript_input_v1(path: Option<&Path>) -> Result<GuestPublisherBootstrapTranscriptV1> {
    let bytes = if let Some(path) = path {
        fs::read(path).with_context(|| format!("read transcript {}", path.display()))?
    } else {
        read_bytes_from_stdin_v1()?
    };
    serde_json::from_slice(&bytes).context("decode bootstrap transcript")
}

fn load_json_file_v1(path: &Path, label: &str) -> Result<Value> {
    let bytes = fs::read(path).with_context(|| format!("read {label} {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("decode {label}"))
}

fn read_bytes_from_stdin_v1() -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    std::io::stdin()
        .read_to_end(&mut bytes)
        .context("read stdin bytes")?;
    if bytes.is_empty() {
        bail!("stdin did not provide bytes");
    }
    Ok(bytes)
}

fn publisher_bootstrap_request_sha256_v1(
    authorization: &PublisherBootstrapAuthorizationV1,
) -> Result<String> {
    sha256_hex_bytes_v1(&canonical_json_bytes_of_v1(
        serde_json::to_value(authorization).context("serialize bootstrap authorization")?,
    )?)
}

fn current_target_triple_v1() -> Result<String> {
    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "arm") {
        "arm"
    } else if cfg!(target_arch = "riscv64") {
        "riscv64"
    } else {
        bail!("unsupported target architecture for Linux lifecycle attestation");
    };
    let vendor = if cfg!(target_vendor = "unknown") {
        "unknown"
    } else if cfg!(target_vendor = "pc") {
        "pc"
    } else {
        bail!("unsupported target vendor for Linux lifecycle attestation");
    };
    let env = if cfg!(target_env = "gnu") {
        "gnu"
    } else if cfg!(target_env = "musl") {
        "musl"
    } else {
        bail!("unsupported target environment for Linux lifecycle attestation");
    };
    Ok(format!("{arch}-{vendor}-linux-{env}"))
}

fn require_current_bootstrap_authorization_v1(
    authorization: &PublisherBootstrapAuthorizationV1,
) -> Result<()> {
    let now = unix_now_ns_v1()?;
    if now < authorization.issued_at_unix_ns || now > authorization.expires_at_unix_ns {
        bail!("publisher bootstrap authorization is expired or not yet valid");
    }
    Ok(())
}

fn attest_executor_build_evidence_v1(
    authorization: &PublisherBootstrapAuthorizationV1,
) -> Result<()> {
    let evidence = &authorization.executor_build_evidence;
    if authorization.source_commit != evidence.source_commit
        || authorization.source_tree != evidence.source_tree
        || authorization.source_ref != evidence.source_ref
    {
        bail!("bootstrap authorization source fields do not match executor build evidence");
    }

    let current_exe = std::env::current_exe().context("resolve running lifecycle executor")?;
    let current_exe = canonicalize_best_effort_v1(&current_exe);
    let evidence_path = canonicalize_best_effort_v1(Path::new(&evidence.artifact_identity));
    if evidence_path != current_exe {
        bail!("executor build evidence artifact_identity does not match the running executor");
    }

    let current_bytes =
        fs::read(&current_exe).with_context(|| format!("read {}", current_exe.display()))?;
    let current_sha256 = sha256_hex_bytes_v1(&current_bytes)?;
    if current_sha256 != evidence.artifact_sha256 {
        bail!("executor build evidence artifact_sha256 does not match the running executor");
    }

    let current_triple = current_target_triple_v1()?;
    if evidence.target_triple != current_triple {
        bail!("executor build evidence target_triple does not match the running executor");
    }
    Ok(())
}

fn bootstrap_intent_leaf_v1(scope_id: &str) -> String {
    format!(".substrate-lifecycle-bootstrap-intent-v1.{scope_id}.json")
}

fn bootstrap_intent_path_from_scope_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    scope_id: &str,
) -> PathBuf {
    executor.state_root.join(bootstrap_intent_leaf_v1(scope_id))
}

fn bootstrap_intent_path_from_authorization_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    authorization: &PublisherBootstrapAuthorizationV1,
) -> Result<PathBuf> {
    let leaf = authorization
        .components
        .iter()
        .find(|component| {
            component
                .target_identity
                .contains(".substrate-lifecycle-bootstrap-intent-v1.")
        })
        .map(|component| {
            Path::new(&component.target_identity)
                .file_name()
                .and_then(|leaf| leaf.to_str())
                .map(str::to_owned)
                .ok_or_else(|| {
                    anyhow!(
                        "bootstrap intent target identity is not a valid file leaf: {}",
                        component.target_identity
                    )
                })
        })
        .transpose()?
        .unwrap_or_else(|| bootstrap_intent_leaf_v1(&authorization.scope_id));
    Ok(executor.state_root.join(leaf))
}

fn current_bootstrap_intent_path_v1(executor: &LinuxManagedArtifactExecutorV1) -> Result<PathBuf> {
    let state = open_linux_publisher_protected_state_v1(executor)?;
    Ok(bootstrap_intent_path_from_scope_v1(
        executor,
        &state.current_anchor.scope_id,
    ))
}

fn require_current_guest_pairing_ticket_v1(ticket: &GuestPublisherPairingTicketV1) -> Result<()> {
    if unix_now_ns_v1()? > ticket.challenge.expires_at_unix_ns {
        bail!("guest pairing ticket expired");
    }
    Ok(())
}

fn require_committable_guest_pairing_ticket_v1(
    ticket: &GuestPublisherPairingTicketV1,
    hello: &GuestPublisherBootstrapHelloV1,
) -> Result<()> {
    if unix_now_ns_v1()? <= ticket.challenge.expires_at_unix_ns {
        return Ok(());
    }
    if hello.intent_published_at_unix_ns > ticket.challenge.expires_at_unix_ns {
        bail!("guest pairing ticket expired before the durable hello was published");
    }
    Ok(())
}

fn path_exists_or_symlink_v1(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("stat {}", path.display())),
    }
}

fn canonicalize_best_effort_v1(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn ensure_nofollow_ancestor_chain_v1(path: &Path) -> Result<()> {
    use std::path::Component;

    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) => bail!("unsupported path prefix in {}", path.display()),
            Component::RootDir => {
                current.push(component.as_os_str());
                continue;
            }
            Component::CurDir => continue,
            Component::ParentDir => bail!("parent traversal is not allowed in {}", path.display()),
            Component::Normal(part) => current.push(part),
        }

        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    bail!(
                        "symlinked path component is not allowed: {}",
                        current.display()
                    );
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => return Err(error).with_context(|| format!("stat {}", current.display())),
        }
    }
    Ok(())
}

fn ensure_directory_mode_v1(path: &Path, mode: u32, preserve_existing_mode: bool) -> Result<()> {
    ensure_nofollow_ancestor_chain_v1(path)?;
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.file_type().is_dir() {
                bail!("expected directory at {}", path.display());
            }
            if !preserve_existing_mode {
                set_mode_v1(path, mode)?;
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir_all(path).with_context(|| format!("create {}", path.display()))?;
            set_mode_v1(path, mode)?;
        }
        Err(error) => return Err(error).with_context(|| format!("stat {}", path.display())),
    }
    Ok(())
}

fn validate_linux_greenfield_bootstrap_residue_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    authorization: &PublisherBootstrapAuthorizationV1,
    bootstrap_intent_path: &Path,
) -> Result<()> {
    if !authorization.publisher_expected_absent {
        return Ok(());
    }

    let managed_paths = [
        ("bootstrap-intent", bootstrap_intent_path),
        (
            "lifecycle-container",
            executor.lifecycle_container.as_path(),
        ),
        ("state-directory", executor.publisher_directory.as_path()),
        ("signing-key", executor.signing_key_path.as_path()),
        ("current-anchor", executor.protected_state_path.as_path()),
    ];
    let mut residue = Vec::new();
    for (label, path) in managed_paths {
        if path_exists_or_symlink_v1(path)? {
            residue.push(format!("{label}:{}", path.display()));
        }
    }
    if !residue.is_empty() {
        bail!(
            "publisher_expected_absent rejects pre-existing linux managed residue: {}",
            residue.join(", ")
        );
    }
    Ok(())
}

fn publish_linux_bootstrap_intent_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    authorization: &PublisherBootstrapAuthorizationV1,
    request_sha256: &str,
    bootstrap_intent_path: &Path,
) -> Result<()> {
    let leaf = bootstrap_intent_path
        .file_name()
        .and_then(|leaf| leaf.to_str())
        .ok_or_else(|| anyhow!("bootstrap intent path has no file leaf"))?;
    let record = json!({
        "schema_owner": "substrate.lifecycle-publisher-bootstrap-intent",
        "schema_version": 1,
        "authority_domain": authorization.authority_domain,
        "scope_id": authorization.scope_id,
        "request_sha256": request_sha256,
        "attempt_nonce": authorization.attempt_nonce,
        "requester_principal": authorization.requester_principal,
        "publisher_expected_absent": authorization.publisher_expected_absent,
        "physical_identity": bootstrap_intent_path.display().to_string(),
        "protected_state_path": executor.protected_state_path.display().to_string(),
        "endpoint_path": executor.endpoint_path.display().to_string(),
        "state": "CreatePending",
        "authorization": serde_json::to_value(authorization)
            .context("serialize bootstrap authorization for intent")?,
    });
    let bytes = canonical_json_value_to_vec_v1(&record)?;
    linux_otmpfile_linkat_v1(&executor.state_root, leaf, &bytes, 0o600).with_context(|| {
        format!(
            "publish bootstrap intent {}",
            bootstrap_intent_path.display()
        )
    })
}

fn load_bootstrap_intent_v1(bootstrap_intent_path: &Path) -> Result<Value> {
    let value = load_json_file_v1(bootstrap_intent_path, "bootstrap intent")?;
    if !value.is_object() {
        bail!(
            "bootstrap intent {} is not a JSON object",
            bootstrap_intent_path.display()
        );
    }
    Ok(value)
}

fn bootstrap_intent_string_field_v1(
    bootstrap_intent: &Value,
    bootstrap_intent_path: &Path,
    field: &str,
) -> Result<Option<String>> {
    match bootstrap_intent.get(field) {
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => bail!(
            "bootstrap intent {} field {field} is not a string",
            bootstrap_intent_path.display()
        ),
        None => Ok(None),
    }
}

fn load_bootstrap_intent_request_sha256_v1(bootstrap_intent_path: &Path) -> Result<String> {
    let value = load_bootstrap_intent_v1(bootstrap_intent_path)?;
    bootstrap_intent_string_field_v1(&value, bootstrap_intent_path, "request_sha256")?.ok_or_else(
        || {
            anyhow!(
                "bootstrap intent {} is missing request_sha256",
                bootstrap_intent_path.display()
            )
        },
    )
}

fn persist_bootstrap_intent_v1(bootstrap_intent_path: &Path, record: &Value) -> Result<()> {
    let bytes = canonical_json_value_to_vec_v1(record)?;
    atomic_write_file_v1(bootstrap_intent_path, &bytes, 0o600)
}

fn signing_key_sha256_v1(path: &Path) -> Result<String> {
    sha256_hex_bytes_v1(&fs::read(path).with_context(|| format!("read {}", path.display()))?)
}

fn record_linux_bootstrap_signing_key_identity_v1(
    bootstrap_intent_path: &Path,
    signing_key_path: &Path,
) -> Result<()> {
    let mut record = load_bootstrap_intent_v1(bootstrap_intent_path)?;
    let object = record.as_object_mut().ok_or_else(|| {
        anyhow!(
            "bootstrap intent {} is not a JSON object",
            bootstrap_intent_path.display()
        )
    })?;
    object.insert(
        "signing_key_path".to_string(),
        Value::String(signing_key_path.display().to_string()),
    );
    object.insert(
        "signing_key_public_key".to_string(),
        Value::String(derive_public_key_v1(signing_key_path)?),
    );
    object.insert(
        "signing_key_sha256".to_string(),
        Value::String(signing_key_sha256_v1(signing_key_path)?),
    );
    object.insert("state".to_string(), Value::String("KeyDurable".to_string()));
    persist_bootstrap_intent_v1(bootstrap_intent_path, &record)
}

fn ensure_resumable_linux_bootstrap_signing_key_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    bootstrap_intent_path: &Path,
    bootstrap_intent: &Value,
) -> Result<()> {
    let expected_public_key = bootstrap_intent_string_field_v1(
        bootstrap_intent,
        bootstrap_intent_path,
        "signing_key_public_key",
    )?;
    let expected_sha256 = bootstrap_intent_string_field_v1(
        bootstrap_intent,
        bootstrap_intent_path,
        "signing_key_sha256",
    )?;
    match (expected_public_key, expected_sha256) {
        (Some(expected_public_key), Some(expected_sha256)) => {
            require_existing_linux_signing_key_v1(&executor.signing_key_path)?;
            let actual_public_key = derive_public_key_v1(&executor.signing_key_path)?;
            if actual_public_key != expected_public_key {
                bail!("existing signing key does not exact-join the durable bootstrap intent");
            }
            let actual_sha256 = signing_key_sha256_v1(&executor.signing_key_path)?;
            if actual_sha256 != expected_sha256 {
                bail!("existing signing key does not exact-join the durable bootstrap intent");
            }
        }
        (None, None) => {
            if path_exists_or_symlink_v1(&executor.signing_key_path)? {
                bail!(
                    "bootstrap intent has no durable signing-key identity; refusing to adopt visible signing key {}",
                    executor.signing_key_path.display()
                );
            }
            create_linux_signing_key_v1(&executor.signing_key_path)?;
            record_linux_bootstrap_signing_key_identity_v1(
                bootstrap_intent_path,
                &executor.signing_key_path,
            )?;
        }
        _ => {
            bail!(
                "bootstrap intent {} has an incomplete durable signing-key identity",
                bootstrap_intent_path.display()
            );
        }
    }
    Ok(())
}

fn bootstrap_linux_publisher_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    authorization: &PublisherBootstrapAuthorizationV1,
    kill_point: KillPointV1,
) -> Result<Value> {
    validate_publisher_bootstrap_authorization_v1(authorization)?;
    require_current_bootstrap_authorization_v1(authorization)?;
    attest_executor_build_evidence_v1(authorization)?;
    let request_sha256 = publisher_bootstrap_request_sha256_v1(authorization)?;
    let bootstrap_intent_path =
        bootstrap_intent_path_from_authorization_v1(executor, authorization)?;
    let response = if path_exists_or_symlink_v1(&executor.protected_state_path)? {
        resume_linux_publisher_bootstrap_v1(
            executor,
            authorization,
            &request_sha256,
            &bootstrap_intent_path,
        )?
    } else if path_exists_or_symlink_v1(&bootstrap_intent_path)? {
        resume_pending_linux_publisher_bootstrap_v1(
            executor,
            authorization,
            &request_sha256,
            &bootstrap_intent_path,
        )?
    } else {
        validate_linux_greenfield_bootstrap_residue_v1(
            executor,
            authorization,
            &bootstrap_intent_path,
        )?;
        let state = create_bootstrap_protected_state_v1(
            executor,
            authorization,
            &request_sha256,
            &bootstrap_intent_path,
            kill_point,
        )?;
        persist_protected_state_v1(executor, None, &state)?;
        generation_one_bootstrap_response_v1(
            executor,
            authorization,
            &request_sha256,
            &bootstrap_intent_path,
            &state,
        )?
    };
    Ok(response)
}

fn create_bootstrap_protected_state_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    authorization: &PublisherBootstrapAuthorizationV1,
    request_sha256: &str,
    bootstrap_intent_path: &Path,
    kill_point: KillPointV1,
) -> Result<LifecyclePublisherProtectedStateV1> {
    open_linux_lifecycle_capsule_v1(executor)?;
    publish_linux_bootstrap_intent_v1(
        executor,
        authorization,
        request_sha256,
        bootstrap_intent_path,
    )?;
    if kill_point == KillPointV1::Intent {
        std::process::exit(81);
    }
    ensure_directory_mode_v1(&executor.publisher_directory, 0o700, false)?;
    create_linux_signing_key_v1(&executor.signing_key_path)?;
    record_linux_bootstrap_signing_key_identity_v1(
        bootstrap_intent_path,
        &executor.signing_key_path,
    )?;
    if kill_point == KillPointV1::Commit {
        std::process::exit(82);
    }

    build_bootstrap_protected_state_v1(executor, authorization, request_sha256)
}

fn build_bootstrap_protected_state_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    authorization: &PublisherBootstrapAuthorizationV1,
    request_sha256: &str,
) -> Result<LifecyclePublisherProtectedStateV1> {
    let mut anchor = LifecyclePublisherAnchorV1 {
        schema_owner: "substrate.lifecycle-publisher-anchor".to_string(),
        schema_version: 1,
        authority_domain: authorization.authority_domain.clone(),
        host_context_commitment: authorization.host_context_commitment.clone(),
        platform_mapping_commitment: authorization.platform_mapping_commitment.clone(),
        scope_id: authorization.scope_id.clone(),
        manifest_generation: 1,
        manifest_sha256: "0".repeat(64),
        action_receipt_index_revision: 0,
        action_receipt_index_sha256: "0".repeat(64),
        head_sha256: "0".repeat(64),
        previous_anchor_sha256: None,
        request_sha256: request_sha256.to_string(),
        requester_principal: authorization.requester_principal.clone(),
        attempt_nonce: authorization.attempt_nonce.clone(),
        executor_identity: default_executor_identity_v1(&authorization.executor_build_evidence)?,
        signature: LifecycleSignatureV1 {
            algorithm: "ed25519-v1".to_string(),
            public_key: String::new(),
            signature: String::new(),
        },
    };
    anchor.signature =
        sign_linux_anchor_v1(executor, &anchor).context("sign bootstrap lifecycle anchor")?;

    let state = LifecyclePublisherProtectedStateV1 {
        schema_owner: "substrate.lifecycle-publisher-protected-state".to_string(),
        schema_version: 1,
        current_anchor: anchor,
        counter: 0,
        prepared_record: None,
        previous_protected_state_sha256: None,
        state_revision: 0,
    };
    validate_lifecycle_publisher_protected_state_v1(&state)?;
    Ok(state)
}

fn default_executor_identity_v1(
    build_evidence: &ExecutorBuildEvidenceV1,
) -> Result<ManagedExecutorIdentityV1> {
    Ok(ManagedExecutorIdentityV1 {
        source_commit: build_evidence.source_commit.clone(),
        source_tree: build_evidence.source_tree.clone(),
        source_ref: build_evidence.source_ref.clone(),
        target_triple: build_evidence.target_triple.clone(),
        artifact_sha256: build_evidence.artifact_sha256.clone(),
        artifact_path: DEFAULT_EXECUTOR_PATH.to_string(),
        toolchain: None,
        code_identity: build_evidence
            .code_identity
            .as_ref()
            .map(|value| Value::String(value.clone())),
    })
}

fn resume_linux_publisher_bootstrap_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    authorization: &PublisherBootstrapAuthorizationV1,
    request_sha256: &str,
    bootstrap_intent_path: &Path,
) -> Result<Value> {
    let state = open_linux_publisher_protected_state_v1(executor)?;
    if state.current_anchor.scope_id != authorization.scope_id
        || state.current_anchor.authority_domain != authorization.authority_domain
        || state.current_anchor.host_context_commitment != authorization.host_context_commitment
        || state.current_anchor.platform_mapping_commitment
            != authorization.platform_mapping_commitment
    {
        bail!("existing publisher state does not match the bootstrap authorization");
    }
    if !path_exists_or_symlink_v1(bootstrap_intent_path)? {
        bail!(
            "existing publisher state is missing bootstrap intent {}",
            bootstrap_intent_path.display()
        );
    }
    if state.current_anchor.request_sha256 != request_sha256 {
        bail!("existing publisher state request digest does not match the bootstrap authorization");
    }
    let intent_request_sha256 = load_bootstrap_intent_request_sha256_v1(bootstrap_intent_path)?;
    if intent_request_sha256 != request_sha256 {
        bail!(
            "existing bootstrap intent request digest does not match the bootstrap authorization"
        );
    }
    ensure_existing_signing_key_matches_public_key_v1(
        &executor.signing_key_path,
        &state.current_anchor.signature.public_key,
        "existing publisher signing key does not exact-join the anchored bootstrap state",
    )?;
    let protected_state_sha256 =
        sha256_hex_bytes_v1(&canonical_lifecycle_publisher_protected_state_v1(&state)?)?;
    Ok(json!({
        "authority_domain": authorization.authority_domain,
        "scope_id": authorization.scope_id,
        "request_sha256": request_sha256,
        "protected_state_path": executor.protected_state_path.display().to_string(),
        "protected_state_sha256": protected_state_sha256,
        "bootstrap_intent_path": bootstrap_intent_path.display().to_string(),
        "publisher_directory": executor.publisher_directory.display().to_string(),
        "endpoint_path": executor.endpoint_path.display().to_string(),
        "state": "GenerationOneAnchored",
        "reused": true,
    }))
}

fn resume_pending_linux_publisher_bootstrap_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    authorization: &PublisherBootstrapAuthorizationV1,
    request_sha256: &str,
    bootstrap_intent_path: &Path,
) -> Result<Value> {
    let bootstrap_intent = load_bootstrap_intent_v1(bootstrap_intent_path)?;
    let intent_request_sha256 = bootstrap_intent_string_field_v1(
        &bootstrap_intent,
        bootstrap_intent_path,
        "request_sha256",
    )?
    .ok_or_else(|| {
        anyhow!(
            "bootstrap intent {} is missing request_sha256",
            bootstrap_intent_path.display()
        )
    })?;
    if intent_request_sha256 != request_sha256 {
        bail!(
            "existing bootstrap intent request digest does not match the bootstrap authorization"
        );
    }

    open_linux_lifecycle_capsule_v1(executor)?;
    ensure_directory_mode_v1(&executor.publisher_directory, 0o700, false)?;
    ensure_resumable_linux_bootstrap_signing_key_v1(
        executor,
        bootstrap_intent_path,
        &bootstrap_intent,
    )?;
    let state = build_bootstrap_protected_state_v1(executor, authorization, request_sha256)?;
    persist_protected_state_v1(executor, None, &state)?;

    let mut response = generation_one_bootstrap_response_v1(
        executor,
        authorization,
        request_sha256,
        bootstrap_intent_path,
        &state,
    )?;
    response["reused_pending"] = Value::Bool(true);
    Ok(response)
}

fn generation_one_bootstrap_response_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    authorization: &PublisherBootstrapAuthorizationV1,
    request_sha256: &str,
    bootstrap_intent_path: &Path,
    state: &LifecyclePublisherProtectedStateV1,
) -> Result<Value> {
    let state_sha256 =
        sha256_hex_bytes_v1(&canonical_lifecycle_publisher_protected_state_v1(state)?)?;
    Ok(json!({
        "authority_domain": authorization.authority_domain,
        "scope_id": authorization.scope_id,
        "request_sha256": request_sha256,
        "protected_state_path": executor.protected_state_path.display().to_string(),
        "protected_state_sha256": state_sha256,
        "bootstrap_intent_path": bootstrap_intent_path.display().to_string(),
        "publisher_directory": executor.publisher_directory.display().to_string(),
        "endpoint_path": executor.endpoint_path.display().to_string(),
        "state": "GenerationOneAnchored",
        "confirmation": authorization.confirmation,
    }))
}

fn run_linux_publisher_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    kill_point: KillPointV1,
) -> Result<()> {
    let listen_pid_matches = std::env::var("LISTEN_PID")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        == Some(std::process::id());
    let listener = if listen_pid_matches {
        let listen_fds = std::env::var("LISTEN_FDS")
            .ok()
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or(0);
        if listen_fds < 1 {
            bail!("LISTEN_PID matched but LISTEN_FDS did not advertise a socket");
        }
        // SAFETY: systemd handed us an already-open descriptor at fd 3.
        unsafe { UnixListener::from_raw_fd(LISTEN_FDS_START) }
    } else {
        match executor.transport {
            TransportKindV1::Stream => {
                if let Some(parent) = executor.endpoint_path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("create {}", parent.display()))?;
                }
                if executor.endpoint_path.exists() {
                    let metadata = fs::symlink_metadata(&executor.endpoint_path)
                        .with_context(|| format!("stat {}", executor.endpoint_path.display()))?;
                    if metadata.file_type().is_socket() {
                        fs::remove_file(&executor.endpoint_path).with_context(|| {
                            format!("remove stale socket {}", executor.endpoint_path.display())
                        })?;
                    } else {
                        bail!(
                            "publisher endpoint path is already occupied by a non-socket: {}",
                            executor.endpoint_path.display()
                        );
                    }
                }
                let listener = UnixListener::bind(&executor.endpoint_path).with_context(|| {
                    format!(
                        "bind test stream listener {}",
                        executor.endpoint_path.display()
                    )
                })?;
                fs::set_permissions(&executor.endpoint_path, fs::Permissions::from_mode(0o600))
                    .with_context(|| format!("chmod {}", executor.endpoint_path.display()))?;
                listener
            }
            TransportKindV1::SeqPacket => bind_seqpacket_listener_v1(&executor.endpoint_path)?,
        }
    };

    loop {
        let (stream, _) = accept_linux_publisher_connection_v1(&listener)?;
        handle_linux_publisher_request_v1(executor, stream, kill_point)?;
    }
}

fn accept_linux_publisher_connection_v1(
    listener: &UnixListener,
) -> Result<(UnixStream, LinuxPeerAttestationV1)> {
    let (stream, _addr) = listener.accept().context("accept publisher connection")?;
    let peer = attest_linux_publisher_peer_v1(stream.as_raw_fd())?;
    Ok((stream, peer))
}

fn attest_linux_publisher_peer_v1(fd: RawFd) -> Result<LinuxPeerAttestationV1> {
    let peer = linux_peer_credentials_v1(fd)?;
    let exe_path = fs::read_link(format!("/proc/{}/exe", peer.pid))
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "<unresolved>".to_string());
    Ok(LinuxPeerAttestationV1 {
        pid: peer.pid as u32,
        uid: peer.uid,
        gid: peer.gid,
        exe_path,
    })
}

fn relay_linux_publisher_request_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    relay_fd: RawFd,
) -> Result<()> {
    let request_bytes = read_frame_from_fd_v1(relay_fd)?;
    let caller = attest_linux_publisher_peer_v1(relay_fd)?;
    let response_bytes = match executor.transport {
        TransportKindV1::SeqPacket => {
            let publisher_stream = connect_seqpacket_socket_v1(&executor.endpoint_path)
                .with_context(|| {
                    format!(
                        "connect seqpacket publisher {}",
                        executor.endpoint_path.display()
                    )
                })?;
            send_seqpacket_frame_v1(&publisher_stream, &request_bytes)?;
            read_seqpacket_frame_v1(&publisher_stream)?
        }
        TransportKindV1::Stream => {
            let publisher_stream =
                UnixStream::connect(&executor.endpoint_path).with_context(|| {
                    format!("connect test stream {}", executor.endpoint_path.display())
                })?;
            send_frame_v1(&publisher_stream, &request_bytes)?;
            publisher_stream
                .shutdown(Shutdown::Write)
                .context("half-close test stream request")?;
            read_frame_v1(&publisher_stream)?
        }
    };

    let mut response_value: Value =
        serde_json::from_slice(&response_bytes).context("decode publisher relay response")?;
    response_value["relay_attestation"] = json!({
        "caller_pid": caller.pid,
        "caller_uid": caller.uid,
        "caller_gid": caller.gid,
        "caller_exe": caller.exe_path,
    });
    write_json_frame_to_fd_v1(relay_fd, &response_value)?;
    Ok(())
}

fn handle_linux_publisher_request_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    stream: UnixStream,
    kill_point: KillPointV1,
) -> Result<()> {
    let mut stream = Some(stream);
    let mut seqpacket_socket = None;
    let request_bytes = match executor.transport {
        TransportKindV1::SeqPacket => {
            let socket = UnixDatagram::from(OwnedFd::from(
                stream.take().expect("seqpacket connection must exist"),
            ));
            let bytes = read_seqpacket_frame_v1(&socket)?;
            seqpacket_socket = Some(socket);
            bytes
        }
        TransportKindV1::Stream => read_frame_v1(
            stream
                .as_ref()
                .expect("stream connection must exist for stream transport"),
        )?,
    };
    let request_value: Value =
        serde_json::from_slice(&request_bytes).context("decode publisher request frame")?;
    if is_linux_publisher_ping_v1(&request_value) {
        let response_bytes = serde_json::to_vec(&linux_publisher_ping_response_v1())
            .context("encode ping response")?;
        if let Some(socket) = seqpacket_socket.as_ref() {
            send_seqpacket_frame_v1(socket, &response_bytes)?;
        } else {
            stream
                .as_mut()
                .expect("stream connection must exist for stream transport")
                .write_all(&response_bytes)
                .context("write ping response")?;
        }
        return Ok(());
    }
    let request: ManagedLifecyclePublisherRequestV1 =
        serde_json::from_value(request_value).context("decode publisher request")?;
    validate_managed_lifecycle_publisher_request_v1(&request)?;
    let _attempt_lock = acquire_publisher_attempt_lock_v1(executor, &request.attempt_nonce)?;

    let request_sha256 = sha256_hex_bytes_v1(&canonical_json_bytes_of_v1(
        serde_json::to_value(&request).context("serialize publisher request")?,
    )?)?;
    let response_cache_path = publisher_response_cache_path_v1(executor, &request.attempt_nonce)?;
    if response_cache_path.exists() {
        let cached = load_cached_publisher_response_v1(&response_cache_path, &request_sha256)?;
        write_publisher_response_v1(
            seqpacket_socket.as_ref(),
            stream.as_mut(),
            &cached,
            &format!("write cached response {}", response_cache_path.display()),
        )?;
        return Ok(());
    }

    let current_state = open_linux_publisher_protected_state_v1(executor)?;
    let current_anchor_sha256 = lifecycle_anchor_sha256_v1(&current_state.current_anchor)?;
    let (working_state, prepared) = if let Some(prepared) = current_state.prepared_record.clone() {
        if prepared.entry_id != request.attempt_nonce {
            bail!("protected state already has an in-flight prepared record");
        }
        if prepared.request_sha256 != request_sha256 {
            bail!("request digest does not match the reserved prepared record");
        }
        if current_anchor_sha256 != request.current_anchor_sha256 {
            bail!("request current_anchor_sha256 does not match protected state");
        }
        if request.current_anchor_counter.checked_add(1) != Some(current_state.counter) {
            bail!("request current_anchor_counter does not match reserved protected state");
        }
        persist_prepared_record_v1(executor, &prepared)?;
        (current_state, prepared)
    } else {
        if current_state.current_anchor.attempt_nonce == request.attempt_nonce {
            if current_state.current_anchor.request_sha256 != request_sha256 {
                bail!("attempt_nonce already committed for a different request");
            }
            let response_bytes = reconstruct_committed_publisher_response_v1(
                executor,
                &request,
                &current_state,
                &request_sha256,
            )?;
            atomic_write_file_v1(&response_cache_path, &response_bytes, 0o600)?;
            write_publisher_response_v1(
                seqpacket_socket.as_ref(),
                stream.as_mut(),
                &response_bytes,
                &format!(
                    "write reconstructed cached response {}",
                    response_cache_path.display()
                ),
            )?;
            return Ok(());
        }
        if current_state.counter != request.current_anchor_counter {
            bail!("request current_anchor_counter does not match protected state");
        }
        if current_anchor_sha256 != request.current_anchor_sha256 {
            bail!("request current_anchor_sha256 does not match protected state");
        }
        let prepared =
            build_prepared_record_v1(executor, &current_state, &request, &request_sha256)?;
        let reserved_state = reserve_prepared_request_state_v1(&current_state, &prepared)?;
        compare_and_swap_linux_publisher_protected_state_v1(
            executor,
            &current_state,
            &reserved_state,
        )?;
        persist_prepared_record_v1(executor, &prepared)?;
        (reserved_state, prepared)
    };
    if kill_point == KillPointV1::Prepare {
        std::process::exit(91);
    }

    let receipt_path =
        publisher_receipt_path_v1(executor, request.manifest_generation, &prepared.receipt_id);
    let receipt = if receipt_path.exists() {
        load_linux_action_receipt_v1(&receipt_path, Some(&prepared), &request, &request_sha256)?
    } else {
        let observation = execute_linux_managed_action_v1(executor, &request, false)?;
        if kill_point == KillPointV1::Observe {
            std::process::exit(92);
        }

        let receipt = build_action_receipt_v1(
            executor,
            &working_state,
            &request,
            &prepared,
            &request_sha256,
            observation,
        )?;
        publish_linux_action_receipt_v1(executor, &receipt)?;
        receipt
    };
    if kill_point == KillPointV1::Receipt {
        std::process::exit(93);
    }

    let next_state = advance_protected_state_v1(
        executor,
        &working_state,
        &request,
        &receipt,
        &request_sha256,
    )?;
    compare_and_swap_linux_publisher_protected_state_v1(executor, &working_state, &next_state)?;

    let response_bytes = build_linux_publisher_response_bytes_v1(
        &receipt.prepared_record_sha256,
        &receipt_path,
        &next_state,
        &request_sha256,
    )?;
    atomic_write_file_v1(&response_cache_path, &response_bytes, 0o600)?;
    write_publisher_response_v1(
        seqpacket_socket.as_ref(),
        stream.as_mut(),
        &response_bytes,
        "write publisher response",
    )?;
    Ok(())
}

fn write_publisher_response_v1(
    seqpacket_socket: Option<&UnixDatagram>,
    stream: Option<&mut UnixStream>,
    response_bytes: &[u8],
    context: &str,
) -> Result<()> {
    if let Some(socket) = seqpacket_socket {
        send_seqpacket_frame_v1(socket, response_bytes)
    } else {
        stream
            .ok_or_else(|| anyhow!("stream connection must exist for stream transport"))?
            .write_all(response_bytes)
            .with_context(|| context.to_string())
    }
}

fn load_cached_publisher_response_v1(path: &Path, request_sha256: &str) -> Result<Vec<u8>> {
    let cached =
        fs::read(path).with_context(|| format!("read cached response {}", path.display()))?;
    let cached_request_sha256 = publisher_response_request_sha256_v1(&cached, path)?;
    if cached_request_sha256 != request_sha256 {
        bail!("cached response request_sha256 does not match the supplied request");
    }
    Ok(cached)
}

fn publisher_response_request_sha256_v1(bytes: &[u8], path: &Path) -> Result<String> {
    let value: Value = serde_json::from_slice(bytes)
        .with_context(|| format!("decode cached response {}", path.display()))?;
    value
        .get("request_sha256")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            anyhow!(
                "cached response {} is missing request_sha256",
                path.display()
            )
        })
}

fn reserve_prepared_request_state_v1(
    current_state: &LifecyclePublisherProtectedStateV1,
    prepared: &ManagedActionPreparedRecordV1,
) -> Result<LifecyclePublisherProtectedStateV1> {
    Ok(LifecyclePublisherProtectedStateV1 {
        schema_owner: "substrate.lifecycle-publisher-protected-state".to_string(),
        schema_version: 1,
        current_anchor: current_state.current_anchor.clone(),
        counter: prepared.allocated_counter,
        prepared_record: Some(prepared.clone()),
        previous_protected_state_sha256: Some(sha256_hex_bytes_v1(
            &canonical_lifecycle_publisher_protected_state_v1(current_state)?,
        )?),
        state_revision: current_state.state_revision + 1,
    })
}

fn persist_prepared_record_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    prepared: &ManagedActionPreparedRecordV1,
) -> Result<PathBuf> {
    let path = publisher_prepared_record_path_v1(executor, &prepared.entry_id)?;
    let bytes = canonical_managed_action_prepared_record_v1(prepared)?;
    atomic_write_file_v1(&path, &bytes, 0o600)?;
    Ok(path)
}

fn load_linux_action_receipt_v1(
    path: &Path,
    prepared: Option<&ManagedActionPreparedRecordV1>,
    request: &ManagedLifecyclePublisherRequestV1,
    request_sha256: &str,
) -> Result<ManagedActionReceiptV1> {
    let bytes = fs::read(path).with_context(|| format!("read receipt {}", path.display()))?;
    let receipt: ManagedActionReceiptV1 = serde_json::from_slice(&bytes)
        .with_context(|| format!("decode receipt {}", path.display()))?;
    validate_managed_action_receipt_signature_v1(&receipt)
        .with_context(|| format!("validate receipt {}", path.display()))?;
    if receipt.request_sha256 != request_sha256 {
        bail!("published receipt request_sha256 does not match the supplied request");
    }
    if receipt.entry_id != request.attempt_nonce {
        bail!("published receipt entry_id does not match the supplied request");
    }
    if receipt.action != request.action {
        bail!("published receipt action does not match the supplied request");
    }
    if receipt.scope_id != request.scope_id
        || receipt.manifest_generation != request.manifest_generation
        || receipt.manifest_sha256 != request.manifest_sha256
    {
        bail!("published receipt scope does not match the supplied request");
    }
    if receipt.executor_identity != request.expected_executor_build {
        bail!("published receipt executor identity does not match the supplied request");
    }
    if let Some(prepared) = prepared {
        let prepared_sha256 = managed_action_prepared_record_sha256_v1(prepared)?;
        if receipt.prepared_record_sha256 != prepared_sha256 {
            bail!("published receipt prepared_record_sha256 does not match the reserved prepared record");
        }
        if receipt.attempt_id != prepared.attempt_id || receipt.receipt_id != prepared.receipt_id {
            bail!("published receipt identity does not match the reserved prepared record");
        }
    }
    Ok(receipt)
}

fn build_linux_publisher_response_bytes_v1(
    prepared_record_sha256: &str,
    receipt_path: &Path,
    state: &LifecyclePublisherProtectedStateV1,
    request_sha256: &str,
) -> Result<Vec<u8>> {
    serde_json::to_vec(&json!({
        "request_sha256": request_sha256,
        "prepared_record_sha256": prepared_record_sha256,
        "receipt_path": receipt_path.display().to_string(),
        "state_revision": state.state_revision,
        "counter": state.counter,
    }))
    .context("encode publisher response")
}

fn reconstruct_committed_publisher_response_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    request: &ManagedLifecyclePublisherRequestV1,
    current_state: &LifecyclePublisherProtectedStateV1,
    request_sha256: &str,
) -> Result<Vec<u8>> {
    if current_state.current_anchor.request_sha256 != request_sha256
        || current_state.current_anchor.attempt_nonce != request.attempt_nonce
    {
        bail!("protected state does not match the supplied committed request");
    }
    let receipt_id = deterministic_uuid_v7_for_request_v1("receipt", request_sha256)?;
    let receipt_path =
        publisher_receipt_path_v1(executor, request.manifest_generation, &receipt_id);
    let receipt = load_linux_action_receipt_v1(&receipt_path, None, request, request_sha256)?;
    let receipt_sha256 = sha256_hex_bytes_v1(&canonical_action_receipt_bytes_v1(&receipt)?)?;
    if current_state.current_anchor.action_receipt_index_sha256 != receipt_sha256 {
        bail!("current anchor receipt digest does not match the published receipt");
    }
    build_linux_publisher_response_bytes_v1(
        &receipt.prepared_record_sha256,
        &receipt_path,
        current_state,
        request_sha256,
    )
}

fn submit_request_direct_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    request: &ManagedLifecyclePublisherRequestV1,
) -> Result<Value> {
    validate_managed_lifecycle_publisher_request_v1(request)?;
    let response = match executor.transport {
        TransportKindV1::Stream => {
            let stream = UnixStream::connect(&executor.endpoint_path)
                .with_context(|| format!("connect {}", executor.endpoint_path.display()))?;
            let bytes = serde_json::to_vec(request).context("encode request")?;
            send_frame_v1(&stream, &bytes)?;
            stream
                .shutdown(Shutdown::Write)
                .context("half-close stream request")?;
            let response_bytes = read_frame_v1(&stream)?;
            serde_json::from_slice(&response_bytes).context("decode direct response")?
        }
        TransportKindV1::SeqPacket => {
            let stream =
                connect_seqpacket_socket_v1(&executor.endpoint_path).with_context(|| {
                    format!("connect seqpacket {}", executor.endpoint_path.display())
                })?;
            let bytes = serde_json::to_vec(request).context("encode request")?;
            send_seqpacket_frame_v1(&stream, &bytes)?;
            let response_bytes = read_seqpacket_frame_v1(&stream)?;
            serde_json::from_slice(&response_bytes).context("decode direct response")?
        }
    };
    Ok(response)
}

fn build_prepared_record_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    current_state: &LifecyclePublisherProtectedStateV1,
    request: &ManagedLifecyclePublisherRequestV1,
    request_sha256: &str,
) -> Result<ManagedActionPreparedRecordV1> {
    let attempt_id = deterministic_uuid_v7_for_request_v1("attempt", &request.attempt_nonce)?;
    let receipt_id = deterministic_uuid_v7_for_request_v1("receipt", request_sha256)?;
    let relative_receipt =
        publisher_receipt_relative_path_v1(request.manifest_generation, &receipt_id);
    let executor_identity = executor_identity_from_request_v1(request);
    let mut record = ManagedActionPreparedRecordV1 {
        schema_owner: "substrate.managed-action-prepared-record".to_string(),
        schema_version: 1,
        authority_domain: current_state.current_anchor.authority_domain.clone(),
        scope_id: request.scope_id.clone(),
        installation_id: request.scope_id.clone(),
        manifest_generation: request.manifest_generation,
        manifest_sha256: request.manifest_sha256.clone(),
        receipt_id,
        receipt_relative_path: relative_receipt,
        entry_id: request.attempt_nonce.clone(),
        action: request.action,
        attempt_id,
        request_sha256: request_sha256.to_string(),
        before_observation: Value::Null,
        executor_identity,
        allocated_counter: current_state.counter + 1,
        previous_record_sha256: current_state
            .prepared_record
            .as_ref()
            .map(managed_action_prepared_record_sha256_v1)
            .transpose()?,
        state: "Prepared".to_string(),
        signature: LifecycleSignatureV1 {
            algorithm: "ed25519-v1".to_string(),
            public_key: String::new(),
            signature: String::new(),
        },
    };
    record.signature = sign_linux_struct_v1(
        &executor.signing_key_path,
        &record.schema_owner,
        serde_json::to_value(&record).context("serialize prepared record")?,
    )?;
    Ok(record)
}

fn build_action_receipt_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    current_state: &LifecyclePublisherProtectedStateV1,
    request: &ManagedLifecyclePublisherRequestV1,
    prepared: &ManagedActionPreparedRecordV1,
    request_sha256: &str,
    observation: Value,
) -> Result<ManagedActionReceiptV1> {
    let relative_receipt =
        publisher_receipt_relative_path_v1(request.manifest_generation, &prepared.receipt_id);
    let mut receipt = ManagedActionReceiptV1 {
        schema_owner: "substrate.managed-action-receipt".to_string(),
        schema_version: 1,
        authority_domain: current_state.current_anchor.authority_domain.clone(),
        scope_id: request.scope_id.clone(),
        installation_id: request.scope_id.clone(),
        manifest_generation: request.manifest_generation,
        manifest_sha256: request.manifest_sha256.clone(),
        receipt_id: prepared.receipt_id.clone(),
        receipt_relative_path: relative_receipt,
        entry_id: prepared.entry_id.clone(),
        action: request.action,
        attempt_id: prepared.attempt_id.clone(),
        prepared_record_sha256: managed_action_prepared_record_sha256_v1(prepared)?,
        allocated_counter: prepared.allocated_counter,
        request_sha256: request_sha256.to_string(),
        pre_observation: Value::Null,
        effect_observation: observation.clone(),
        post_observation: observation,
        restoration_status: None,
        error_class: None,
        executor_identity: executor_identity_from_request_v1(request),
        signature: LifecycleSignatureV1 {
            algorithm: "ed25519-v1".to_string(),
            public_key: String::new(),
            signature: String::new(),
        },
    };
    receipt.signature = sign_linux_struct_v1(
        &executor.signing_key_path,
        &receipt.schema_owner,
        serde_json::to_value(&receipt).context("serialize action receipt")?,
    )?;
    Ok(receipt)
}

fn advance_protected_state_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    current_state: &LifecyclePublisherProtectedStateV1,
    request: &ManagedLifecyclePublisherRequestV1,
    receipt: &ManagedActionReceiptV1,
    request_sha256: &str,
) -> Result<LifecyclePublisherProtectedStateV1> {
    let next_counter = if let Some(prepared) = current_state.prepared_record.as_ref() {
        if prepared.entry_id != request.attempt_nonce {
            bail!("protected state prepared record does not match the supplied request");
        }
        if prepared.request_sha256 != request_sha256 {
            bail!("protected state prepared record request_sha256 does not match the supplied request");
        }
        if receipt.allocated_counter != prepared.allocated_counter {
            bail!("action receipt allocated_counter does not match the reserved prepared record");
        }
        current_state.counter
    } else {
        if receipt.allocated_counter != current_state.counter + 1 {
            bail!("action receipt allocated_counter does not advance the protected state counter");
        }
        current_state.counter + 1
    };
    let current_anchor_sha256 = lifecycle_anchor_sha256_v1(&current_state.current_anchor)?;
    let mut anchor = LifecyclePublisherAnchorV1 {
        schema_owner: "substrate.lifecycle-publisher-anchor".to_string(),
        schema_version: 1,
        authority_domain: current_state.current_anchor.authority_domain.clone(),
        host_context_commitment: request.host_context_commitment.clone(),
        platform_mapping_commitment: request.platform_mapping_commitment.clone(),
        scope_id: request.scope_id.clone(),
        manifest_generation: request.manifest_generation,
        manifest_sha256: request.manifest_sha256.clone(),
        action_receipt_index_revision: current_state.current_anchor.action_receipt_index_revision
            + 1,
        action_receipt_index_sha256: sha256_hex_bytes_v1(&canonical_action_receipt_bytes_v1(
            receipt,
        )?)?,
        head_sha256: current_anchor_sha256.clone(),
        previous_anchor_sha256: Some(current_anchor_sha256),
        request_sha256: request_sha256.to_string(),
        requester_principal: request.requester_principal.clone(),
        attempt_nonce: request.attempt_nonce.clone(),
        executor_identity: executor_identity_from_request_v1(request),
        signature: LifecycleSignatureV1 {
            algorithm: "ed25519-v1".to_string(),
            public_key: String::new(),
            signature: String::new(),
        },
    };
    anchor.signature =
        sign_linux_anchor_v1(executor, &anchor).context("sign advanced lifecycle anchor")?;

    Ok(LifecyclePublisherProtectedStateV1 {
        schema_owner: "substrate.lifecycle-publisher-protected-state".to_string(),
        schema_version: 1,
        current_anchor: anchor,
        counter: next_counter,
        prepared_record: None,
        previous_protected_state_sha256: Some(sha256_hex_bytes_v1(
            &canonical_lifecycle_publisher_protected_state_v1(current_state)?,
        )?),
        state_revision: current_state.state_revision + 1,
    })
}

fn executor_identity_from_request_v1(
    request: &ManagedLifecyclePublisherRequestV1,
) -> ManagedExecutorIdentityV1 {
    request.expected_executor_build.clone()
}

fn execute_linux_managed_action_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    request: &ManagedLifecyclePublisherRequestV1,
    allow_live_probe: bool,
) -> Result<Value> {
    if request.action == ManagedActionV1::Restore {
        return restore_linux_managed_role_v1(executor, request, allow_live_probe);
    }

    match request.role.0.as_str() {
        "linux.host.service-state(service)" => {
            let observation = execute_service_unit_action_v1(
                executor,
                DEFAULT_WORLD_SERVICE_UNIT,
                DEFAULT_WORLD_ENDPOINT_PATH,
                request.action,
                allow_live_probe,
            )?;
            Ok(json!({
                "role": request.role.0,
                "observation": observation,
            }))
        }
        "linux.host.service-state(socket)" => {
            let observation = execute_service_unit_action_v1(
                executor,
                DEFAULT_WORLD_SOCKET_UNIT,
                DEFAULT_WORLD_ENDPOINT_PATH,
                request.action,
                allow_live_probe,
            )?;
            Ok(json!({
                "role": request.role.0,
                "observation": observation,
            }))
        }
        "linux.publisher.service-state(service)" => {
            let observation = execute_service_unit_action_v1(
                executor,
                DEFAULT_LIFECYCLE_SERVICE_UNIT,
                executor
                    .endpoint_path
                    .to_str()
                    .unwrap_or(DEFAULT_ENDPOINT_PATH),
                request.action,
                allow_live_probe,
            )?;
            Ok(json!({
                "role": request.role.0,
                "observation": observation,
            }))
        }
        "linux.publisher.service-state(socket)" => {
            let observation = execute_service_unit_action_v1(
                executor,
                DEFAULT_LIFECYCLE_SOCKET_UNIT,
                executor
                    .endpoint_path
                    .to_str()
                    .unwrap_or(DEFAULT_ENDPOINT_PATH),
                request.action,
                allow_live_probe,
            )?;
            Ok(json!({
                "role": request.role.0,
                "observation": observation,
            }))
        }
        _ => Ok(json!({
            "role": request.role.0,
            "action": managed_action_name_v1(request.action),
            "object_identity": request.object_identity.physical_identity,
            "message": "bounded packet model path did not mutate non-service-state roles",
        })),
    }
}

fn restore_linux_managed_role_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    request: &ManagedLifecyclePublisherRequestV1,
    allow_live_probe: bool,
) -> Result<Value> {
    let (unit, endpoint_path) = match request.role.0.as_str() {
        "linux.host.service-state(service)" => (
            DEFAULT_WORLD_SERVICE_UNIT,
            DEFAULT_WORLD_ENDPOINT_PATH.to_string(),
        ),
        "linux.host.service-state(socket)" => (
            DEFAULT_WORLD_SOCKET_UNIT,
            DEFAULT_WORLD_ENDPOINT_PATH.to_string(),
        ),
        "linux.publisher.service-state(service)" => (
            DEFAULT_LIFECYCLE_SERVICE_UNIT,
            executor.endpoint_path.display().to_string(),
        ),
        "linux.publisher.service-state(socket)" => (
            DEFAULT_LIFECYCLE_SOCKET_UNIT,
            executor.endpoint_path.display().to_string(),
        ),
        _ => {
            return Ok(json!({
                "role": request.role.0,
                "action": "restore",
                "message": "bounded packet model path did not mutate non-service-state roles",
            }));
        }
    };

    let before = observe_service_state_v1(unit, &endpoint_path)?;
    let restore_target = request
        .object_identity
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.get("restore_state"))
        .cloned()
        .unwrap_or(Value::Null);
    let enabled_target = restore_target.get("enabled").and_then(Value::as_str);
    let active_target = restore_target.get("active").and_then(Value::as_str);

    if matches!(active_target, Some("active"))
        && matches!(enabled_target, Some("masked" | "masked-runtime"))
    {
        let _ = run_systemctl_v1("unmask", unit);
        restore_service_active_state_v1(
            executor,
            unit,
            &endpoint_path,
            "active",
            allow_live_probe,
        )?;
        restore_service_enabled_state_v1(
            unit,
            enabled_target.expect("masked active restore target must include enabled"),
        )?;
    } else {
        if let Some(enabled) = enabled_target {
            restore_service_enabled_state_v1(unit, enabled)?;
        }
        if let Some(active) = active_target {
            restore_service_active_state_v1(
                executor,
                unit,
                &endpoint_path,
                active,
                allow_live_probe,
            )?;
        }
    }

    let after = observe_service_state_v1(unit, &endpoint_path)?;
    Ok(json!({
        "unit": unit,
        "before": {
            "active": before.active,
            "enabled": before.enabled,
            "endpoint": before.endpoint,
        },
        "restore_target": restore_target,
        "after": {
            "active": after.active,
            "enabled": after.enabled,
            "endpoint": after.endpoint,
        },
    }))
}

fn execute_service_unit_action_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    unit: &str,
    endpoint_path: &str,
    action: ManagedActionV1,
    allow_live_probe: bool,
) -> Result<Value> {
    let before = observe_service_state_v1(unit, endpoint_path)?;
    match action {
        ManagedActionV1::Start => {
            start_service_unit_v1(executor, unit, endpoint_path, allow_live_probe)?;
        }
        ManagedActionV1::Stop => {
            run_systemctl_v1("stop", unit)?;
        }
        ManagedActionV1::Enable => {
            run_systemctl_v1("enable", unit)?;
        }
        ManagedActionV1::Disable => {
            run_systemctl_v1("disable", unit)?;
        }
        ManagedActionV1::Restore
        | ManagedActionV1::Create
        | ManagedActionV1::Replace
        | ManagedActionV1::Remove => {}
    }
    let after = observe_service_state_v1(unit, endpoint_path)?;
    Ok(json!({
        "unit": unit,
        "before": {
            "active": before.active,
            "enabled": before.enabled,
            "endpoint": before.endpoint,
        },
        "after": {
            "active": after.active,
            "enabled": after.enabled,
            "endpoint": after.endpoint,
        },
    }))
}

fn start_service_unit_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    unit: &str,
    endpoint_path: &str,
    allow_live_probe: bool,
) -> Result<()> {
    if allow_live_probe
        && unit == DEFAULT_LIFECYCLE_SERVICE_UNIT
        && publisher_listener_is_live_v1(executor, Path::new(endpoint_path))?
    {
        trigger_linux_publisher_ready_probe_v1(executor)?;
    }
    run_systemctl_v1("start", unit)
}

fn restore_service_enabled_state_v1(unit: &str, enabled: &str) -> Result<()> {
    match enabled {
        "enabled" => {
            let _ = run_systemctl_v1("unmask", unit);
            run_systemctl_v1("enable", unit)?;
        }
        "enabled-runtime" => {
            let _ = run_systemctl_v1("unmask", unit);
            run_systemctl_args_v1(&["enable", "--runtime", unit])?;
        }
        "static" => {}
        "disabled" => {
            let _ = run_systemctl_v1("unmask", unit);
            run_systemctl_v1("disable", unit)?;
        }
        "masked" => {
            run_systemctl_v1("mask", unit)?;
        }
        "masked-runtime" => {
            run_systemctl_args_v1(&["mask", "--runtime", unit])?;
        }
        "unknown" => {}
        other => bail!("unsupported exact restore_state.enabled value {other}"),
    }
    Ok(())
}

fn restore_service_active_state_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    unit: &str,
    endpoint_path: &str,
    active: &str,
    allow_live_probe: bool,
) -> Result<()> {
    match active {
        "active" => start_service_unit_v1(executor, unit, endpoint_path, allow_live_probe)?,
        "inactive" => run_systemctl_v1("stop", unit)?,
        "unknown" => {}
        other => bail!("unsupported exact restore_state.active value {other}"),
    }
    Ok(())
}

fn publish_linux_action_receipt_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    receipt: &ManagedActionReceiptV1,
) -> Result<PathBuf> {
    let receipt_path =
        publisher_receipt_path_v1(executor, receipt.manifest_generation, &receipt.receipt_id);
    let bytes = canonical_action_receipt_bytes_v1(receipt)?;
    atomic_write_file_v1(&receipt_path, &bytes, 0o600)?;
    Ok(receipt_path)
}

fn open_linux_publisher_protected_state_v1(
    executor: &LinuxManagedArtifactExecutorV1,
) -> Result<LifecyclePublisherProtectedStateV1> {
    let bytes = fs::read(&executor.protected_state_path).with_context(|| {
        format!(
            "read protected state {}",
            executor.protected_state_path.display()
        )
    })?;
    let state: LifecyclePublisherProtectedStateV1 =
        serde_json::from_slice(&bytes).context("decode protected state")?;
    validate_lifecycle_publisher_protected_state_v1(&state)?;
    Ok(state)
}

fn compare_and_swap_linux_publisher_protected_state_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    expected: &LifecyclePublisherProtectedStateV1,
    next: &LifecyclePublisherProtectedStateV1,
) -> Result<()> {
    let current = open_linux_publisher_protected_state_v1(executor)?;
    let current_sha256 =
        sha256_hex_bytes_v1(&canonical_lifecycle_publisher_protected_state_v1(&current)?)?;
    let expected_sha256 =
        sha256_hex_bytes_v1(&canonical_lifecycle_publisher_protected_state_v1(expected)?)?;
    if current_sha256 != expected_sha256 {
        bail!("protected state compare-and-swap mismatch");
    }
    persist_protected_state_v1(executor, Some(&current), next)
}

fn persist_protected_state_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    current: Option<&LifecyclePublisherProtectedStateV1>,
    next: &LifecyclePublisherProtectedStateV1,
) -> Result<()> {
    if let Some(parent) = executor.protected_state_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let bytes = canonical_lifecycle_publisher_protected_state_v1(next)?;
    if let Some(existing) = current {
        let existing_sha =
            sha256_hex_bytes_v1(&canonical_lifecycle_publisher_protected_state_v1(existing)?)?;
        let disk_sha = sha256_hex_bytes_v1(
            &fs::read(&executor.protected_state_path)
                .with_context(|| format!("read {}", executor.protected_state_path.display()))?,
        )?;
        if existing_sha != disk_sha {
            bail!("protected state changed before write");
        }
    }
    atomic_write_file_v1(&executor.protected_state_path, &bytes, 0o600)
}

fn begin_guest_publisher_bootstrap_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    ticket: &GuestPublisherPairingTicketV1,
    kill_point: KillPointV1,
) -> Result<GuestPublisherBootstrapHelloV1> {
    verify_guest_publisher_pairing_ticket_v1(ticket)?;
    if guest_consumption_marker_path_v1(executor, &ticket.challenge.challenge_id).exists() {
        bail!("guest pairing challenge already reached a consumed terminal state");
    }
    let intent_path = guest_intent_path_v1(executor, &ticket.challenge.challenge_id);
    let hello_path = guest_bootstrap_hello_path_v1(executor, &ticket.challenge.challenge_id);
    let intent = if path_exists_or_symlink_v1(&intent_path)? {
        publish_guest_pairing_intent_v1(executor, ticket)?
    } else {
        if path_exists_or_symlink_v1(&hello_path)? {
            bail!(
                "guest bootstrap hello exists without a durable guest pairing intent: {}",
                hello_path.display()
            );
        }
        require_current_guest_pairing_ticket_v1(ticket)?;
        let _ = open_guest_controlling_tty_v1(executor)?;
        let confirmation = read_guest_publisher_pairing_confirmation_v1(executor, ticket)?;
        if confirmation
            != format!(
                "{}\n{}\n{}",
                ticket.challenge.host_key_fingerprint_sha256,
                ticket.challenge.challenge,
                GUEST_PAIRING_LITERAL_V1
            )
        {
            bail!("guest pairing confirmation literal mismatch");
        }
        publish_guest_pairing_intent_v1(executor, ticket)?
    };
    if kill_point == KillPointV1::Intent {
        std::process::exit(101);
    }
    let hello = emit_guest_publisher_bootstrap_hello_v1(executor, ticket, &intent)?;
    if kill_point == KillPointV1::Hello {
        std::process::exit(102);
    }
    Ok(hello)
}

fn open_guest_controlling_tty_v1(
    executor: &LinuxManagedArtifactExecutorV1,
) -> Result<(File, File)> {
    if stdin_tty_matches_path_v1(&executor.tty_path) {
        let reader_fd = dup_fd_v1(STDIN_FILENO_V1).context("duplicate controlling tty reader")?;
        ensure_tty_fd_v1(reader_fd, &executor.tty_path, "reader")?;
        let writer_fd = dup_fd_v1(STDIN_FILENO_V1).context("duplicate controlling tty writer")?;
        ensure_tty_fd_v1(writer_fd, &executor.tty_path, "writer")?;
        // SAFETY: dup returned owned file descriptors for this process.
        let reader = unsafe { File::from_raw_fd(reader_fd) };
        // SAFETY: dup returned owned file descriptors for this process.
        let writer = unsafe { File::from_raw_fd(writer_fd) };
        return Ok((reader, writer));
    }
    let reader = OpenOptions::new()
        .read(true)
        .open(&executor.tty_path)
        .with_context(|| format!("open tty reader {}", executor.tty_path.display()))?;
    ensure_tty_fd_v1(reader.as_raw_fd(), &executor.tty_path, "reader")?;
    let writer = OpenOptions::new()
        .write(true)
        .open(&executor.tty_path)
        .with_context(|| format!("open tty writer {}", executor.tty_path.display()))?;
    ensure_tty_fd_v1(writer.as_raw_fd(), &executor.tty_path, "writer")?;
    Ok((reader, writer))
}

fn read_guest_publisher_pairing_confirmation_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    ticket: &GuestPublisherPairingTicketV1,
) -> Result<String> {
    let (mut reader, mut writer) = open_guest_controlling_tty_v1(executor)?;
    writeln!(
        writer,
        "HOST_KEY_FINGERPRINT_SHA256 {}",
        ticket.challenge.host_key_fingerprint_sha256
    )
    .context("write pairing fingerprint")?;
    writeln!(writer, "CHALLENGE {}", ticket.challenge.challenge)
        .context("write pairing challenge")?;
    writeln!(writer, "LITERAL {}", GUEST_PAIRING_LITERAL_V1).context("write pairing literal")?;
    writer.flush().context("flush pairing prompt")?;

    let mut bytes = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_len) => bytes.extend_from_slice(&chunk[..read_len]),
            Err(error) if error.raw_os_error() == Some(5) => break,
            Err(error) => return Err(error).context("read pairing confirmation"),
        }
    }
    let text = String::from_utf8(bytes).context("pairing confirmation was not UTF-8")?;
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    Ok(normalized.trim_end_matches('\n').to_string())
}

fn verify_guest_publisher_pairing_ticket_v1(ticket: &GuestPublisherPairingTicketV1) -> Result<()> {
    validate_guest_publisher_pairing_ticket_v1(ticket)
}

fn emit_guest_publisher_bootstrap_hello_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    ticket: &GuestPublisherPairingTicketV1,
    intent: &GuestPublisherPairingGuestIntentV1,
) -> Result<GuestPublisherBootstrapHelloV1> {
    let hello_path = guest_bootstrap_hello_path_v1(executor, &ticket.challenge.challenge_id);
    if path_exists_or_symlink_v1(&hello_path)? {
        let hello = load_guest_bootstrap_hello_v1(&hello_path)?;
        validate_guest_bootstrap_hello_resume_v1(executor, ticket, intent, &hello)?;
        if intent.hello.as_ref() != Some(&hello) {
            let mut durable_intent = intent.clone();
            durable_intent.hello = Some(hello.clone());
            durable_intent.state = "HelloDurable".to_string();
            persist_guest_pairing_intent_v1(
                executor,
                &ticket.challenge.challenge_id,
                &durable_intent,
            )?;
        }
        return Ok(hello);
    }

    let hello_source_intent =
        guest_hello_source_intent_v1(executor, &ticket.challenge.challenge_id, intent)?;
    let ticket_sha256 = sha256_hex_bytes_v1(&canonical_guest_publisher_pairing_ticket_v1(ticket)?)?;
    let intent_bytes = canonical_json_bytes_of_v1(
        serde_json::to_value(&hello_source_intent).context("serialize guest pairing intent")?,
    )?;
    let intent_sha256 = sha256_hex_bytes_v1(&intent_bytes)?;
    let mut hello = GuestPublisherBootstrapHelloV1 {
        schema_owner: "substrate.guest-publisher-bootstrap-hello".to_string(),
        schema_version: 1,
        ticket_sha256,
        intent_sha256,
        guest_test_retirement_commitment: ticket.guest_test_retirement_commitment.clone(),
        guest_machine_identity: ticket.challenge.guest_machine_identity.clone(),
        guest_artifact_sha256: intent.guest_artifact_sha256.clone(),
        intent_published_at_unix_ns: unix_now_ns_v1()?,
        intent_parent_fsync_observed: true,
        nonce: intent.nonce.clone(),
        guest_public_key: intent.public_key.clone(),
        signature: LifecycleSignatureV1 {
            algorithm: "ed25519-v1".to_string(),
            public_key: String::new(),
            signature: String::new(),
        },
    };
    hello.signature = sign_linux_struct_v1(
        &guest_signing_key_path_v1(executor, &ticket.challenge.challenge_id),
        &hello.schema_owner,
        serde_json::to_value(&hello).context("serialize guest bootstrap hello")?,
    )?;
    write_guest_artifact_v1(
        executor,
        &ticket.challenge.challenge_id,
        "hello.json",
        &serde_json::to_value(&hello).context("serialize guest bootstrap hello")?,
    )?;
    let mut durable_intent = intent.clone();
    durable_intent.hello = Some(hello.clone());
    durable_intent.state = "HelloDurable".to_string();
    persist_guest_pairing_intent_v1(executor, &ticket.challenge.challenge_id, &durable_intent)?;
    Ok(hello)
}

fn publish_guest_pairing_intent_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    ticket: &GuestPublisherPairingTicketV1,
) -> Result<GuestPublisherPairingGuestIntentV1> {
    open_guest_pairing_intent_parent_v1(executor)?;
    let ticket_sha256 = sha256_hex_bytes_v1(&canonical_guest_publisher_pairing_ticket_v1(ticket)?)?;
    let confirmation_commitment = sha256_hex_bytes_v1(
        format!(
            "{}:{}:{}",
            ticket.challenge.host_key_fingerprint_sha256,
            ticket.challenge.challenge,
            GUEST_PAIRING_LITERAL_V1
        )
        .as_bytes(),
    )?;
    let intent_path = guest_intent_path_v1(executor, &ticket.challenge.challenge_id);
    if path_exists_or_symlink_v1(&intent_path)? {
        let intent = resume_guest_publisher_pairing_v1(executor, &ticket.challenge.challenge_id)?;
        let signing_key_path = guest_signing_key_path_v1(executor, &ticket.challenge.challenge_id);
        require_existing_linux_signing_key_v1(&signing_key_path)?;
        let public_key = derive_public_key_v1(&signing_key_path)?;
        validate_resumable_guest_pairing_intent_v1(
            ticket,
            &intent,
            &confirmation_commitment,
            &public_key,
        )?;
        return Ok(intent);
    }
    let signing_key_path = guest_signing_key_path_v1(executor, &ticket.challenge.challenge_id);
    if path_exists_or_symlink_v1(&signing_key_path)? {
        bail!(
            "guest signing key exists without a durable guest pairing intent: {}",
            signing_key_path.display()
        );
    }
    let signing_key_path =
        open_or_join_guest_signing_key_v1(executor, &ticket.challenge.challenge_id)?;
    let public_key = derive_public_key_v1(&signing_key_path)?;
    let mut intent = GuestPublisherPairingGuestIntentV1 {
        schema_owner: "substrate.guest-publisher-pairing-guest-intent".to_string(),
        schema_version: 1,
        scope_id: ticket.current_anchor.scope_id.clone(),
        challenge_id: ticket.challenge.challenge_id.clone(),
        ticket_sha256,
        confirmation_commitment,
        guest_machine_identity: ticket.challenge.guest_machine_identity.clone(),
        guest_artifact_sha256: ticket.challenge.executor_build_evidence_sha256.clone(),
        seed: sha256_hex_bytes_v1(ticket.challenge.challenge.as_bytes())?,
        public_key: public_key.clone(),
        public_key_sha256: sha256_hex_bytes_v1(&base64url_decode_v1(&public_key)?)?,
        nonce: sha256_hex_bytes_v1(ticket.challenge.challenge_id.as_bytes())?,
        guest_test_retirement_commitment: ticket.guest_test_retirement_commitment.clone(),
        transcript: None,
        hello: None,
        state: "PairingIntentDurable".to_string(),
        signature: LifecycleSignatureV1 {
            algorithm: "ed25519-v1".to_string(),
            public_key: String::new(),
            signature: String::new(),
        },
    };
    intent.signature = sign_linux_struct_v1(
        &signing_key_path,
        &intent.schema_owner,
        serde_json::to_value(&intent).context("serialize guest pairing intent")?,
    )?;
    publish_guest_pairing_intent_otmpfile_v1(executor, &ticket.challenge.challenge_id, &intent)?;
    Ok(intent)
}

fn open_guest_pairing_intent_parent_v1(executor: &LinuxManagedArtifactExecutorV1) -> Result<()> {
    ensure_directory_mode_v1(&executor.state_root, 0o750, true)
}

fn publish_guest_pairing_intent_otmpfile_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    challenge_id: &str,
    intent: &GuestPublisherPairingGuestIntentV1,
) -> Result<()> {
    let bytes = canonical_json_bytes_of_v1(
        serde_json::to_value(intent).context("serialize guest pairing intent")?,
    )?;
    let leaf = guest_intent_leaf_v1(challenge_id);
    linux_otmpfile_linkat_v1(&executor.state_root, &leaf, &bytes, 0o600)
}

fn validate_resumable_guest_pairing_intent_v1(
    ticket: &GuestPublisherPairingTicketV1,
    intent: &GuestPublisherPairingGuestIntentV1,
    confirmation_commitment: &str,
    public_key: &str,
) -> Result<()> {
    let ticket_sha256 = sha256_hex_bytes_v1(&canonical_guest_publisher_pairing_ticket_v1(ticket)?)?;
    if intent.scope_id != ticket.current_anchor.scope_id
        || intent.challenge_id != ticket.challenge.challenge_id
    {
        bail!("durable guest pairing intent does not match the supplied ticket");
    }
    if intent.ticket_sha256 != ticket_sha256 {
        bail!("durable guest pairing intent ticket_sha256 does not match the supplied ticket");
    }
    if intent.confirmation_commitment != confirmation_commitment {
        bail!(
            "durable guest pairing intent confirmation_commitment does not match the supplied ticket"
        );
    }
    if intent.guest_machine_identity != ticket.challenge.guest_machine_identity {
        bail!("durable guest pairing intent machine identity does not match the supplied ticket");
    }
    if intent.guest_artifact_sha256 != ticket.challenge.executor_build_evidence_sha256 {
        bail!("durable guest pairing intent executor digest does not match the supplied ticket");
    }
    if intent.public_key != public_key {
        bail!("durable guest pairing intent public key does not match the durable guest key file");
    }
    if intent.public_key_sha256 != sha256_hex_bytes_v1(&base64url_decode_v1(public_key)?)? {
        bail!("durable guest pairing intent public_key_sha256 does not match its public key");
    }
    if intent.nonce != sha256_hex_bytes_v1(ticket.challenge.challenge_id.as_bytes())? {
        bail!("durable guest pairing intent nonce does not match the supplied ticket");
    }
    if intent.seed != sha256_hex_bytes_v1(ticket.challenge.challenge.as_bytes())? {
        bail!("durable guest pairing intent seed does not match the supplied ticket");
    }
    if intent.guest_test_retirement_commitment != ticket.guest_test_retirement_commitment {
        bail!(
            "durable guest pairing intent retirement commitment does not match the supplied ticket"
        );
    }
    Ok(())
}

fn guest_hello_source_intent_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    challenge_id: &str,
    intent: &GuestPublisherPairingGuestIntentV1,
) -> Result<GuestPublisherPairingGuestIntentV1> {
    let mut normalized = intent.clone();
    normalized.transcript = None;
    normalized.hello = None;
    normalized.state = "PairingIntentDurable".to_string();
    normalized.signature = LifecycleSignatureV1 {
        algorithm: "ed25519-v1".to_string(),
        public_key: String::new(),
        signature: String::new(),
    };
    normalized.signature = sign_linux_struct_v1(
        &guest_signing_key_path_v1(executor, challenge_id),
        &normalized.schema_owner,
        serde_json::to_value(&normalized).context("serialize guest pairing intent")?,
    )?;
    Ok(normalized)
}

fn load_guest_bootstrap_hello_v1(path: &Path) -> Result<GuestPublisherBootstrapHelloV1> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let hello: GuestPublisherBootstrapHelloV1 =
        serde_json::from_slice(&bytes).context("decode guest bootstrap hello")?;
    verify_lifecycle_signature_v1(&hello.schema_owner, &hello, &hello.signature)
        .context("verify guest bootstrap hello signature")?;
    Ok(hello)
}

fn validate_guest_bootstrap_hello_resume_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    ticket: &GuestPublisherPairingTicketV1,
    intent: &GuestPublisherPairingGuestIntentV1,
    hello: &GuestPublisherBootstrapHelloV1,
) -> Result<()> {
    let ticket_sha256 = sha256_hex_bytes_v1(&canonical_guest_publisher_pairing_ticket_v1(ticket)?)?;
    if hello.ticket_sha256 != ticket_sha256 {
        bail!("persisted guest bootstrap hello ticket_sha256 does not match the pairing ticket");
    }
    let hello_source_intent =
        guest_hello_source_intent_v1(executor, &ticket.challenge.challenge_id, intent)?;
    let expected_intent_sha256 = sha256_hex_bytes_v1(&canonical_json_bytes_of_v1(
        serde_json::to_value(&hello_source_intent).context("serialize guest pairing intent")?,
    )?)?;
    if hello.intent_sha256 != expected_intent_sha256 {
        bail!("persisted guest bootstrap hello intent_sha256 does not match the durable intent");
    }
    if hello.guest_machine_identity != intent.guest_machine_identity
        || hello.guest_artifact_sha256 != intent.guest_artifact_sha256
        || hello.nonce != intent.nonce
        || hello.guest_public_key != intent.public_key
    {
        bail!("persisted guest bootstrap hello does not match the pairing intent");
    }
    Ok(())
}

fn open_or_join_guest_signing_key_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    challenge_id: &str,
) -> Result<PathBuf> {
    let path = guest_signing_key_path_v1(executor, challenge_id);
    ensure_guest_artifact_directory_v1(executor, challenge_id)?;
    load_or_create_linux_signing_key_v1(&path)?;
    Ok(path)
}

fn materialize_guest_signing_key_v1(guest_key_path: &Path, final_key_path: &Path) -> Result<()> {
    match fs::symlink_metadata(final_key_path) {
        Ok(metadata) if !metadata.file_type().is_file() => {
            bail!(
                "publisher signing key path is not a regular file: {}",
                final_key_path.display()
            )
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| format!("stat {}", final_key_path.display()))
        }
    }

    let parent = final_key_path
        .parent()
        .ok_or_else(|| anyhow!("publisher signing key path has no parent"))?;
    fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    let leaf = final_key_path
        .file_name()
        .and_then(|leaf| leaf.to_str())
        .ok_or_else(|| anyhow!("publisher signing key path has no file name"))?;
    let tmp_path = parent.join(format!(".{leaf}.tmp-{}", unix_now_ns_v1()?));
    fs::copy(guest_key_path, &tmp_path).with_context(|| {
        format!(
            "copy guest key {} -> {}",
            guest_key_path.display(),
            tmp_path.display()
        )
    })?;
    fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("chmod {}", tmp_path.display()))?;
    fs::rename(&tmp_path, final_key_path).with_context(|| {
        format!(
            "rename guest key {} -> {}",
            tmp_path.display(),
            final_key_path.display()
        )
    })?;
    let final_file = OpenOptions::new()
        .read(true)
        .open(final_key_path)
        .with_context(|| format!("open {}", final_key_path.display()))?;
    final_file
        .sync_all()
        .with_context(|| format!("sync {}", final_key_path.display()))?;
    linux_fsync_parent_v1(final_key_path)?;
    Ok(())
}

fn ensure_guest_signing_key_matches_durable_intent_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    challenge_id: &str,
    intent: &GuestPublisherPairingGuestIntentV1,
) -> Result<PathBuf> {
    let guest_key_path = guest_signing_key_path_v1(executor, challenge_id);
    require_existing_linux_signing_key_v1(&guest_key_path)?;
    let guest_public_key = derive_public_key_v1(&guest_key_path)?;
    if guest_public_key != intent.public_key {
        bail!("guest signing key does not match the durable guest pairing intent");
    }
    let guest_public_key_sha256 = sha256_hex_bytes_v1(&base64url_decode_v1(&guest_public_key)?)?;
    if guest_public_key_sha256 != intent.public_key_sha256 {
        bail!("guest signing key does not match the durable guest pairing intent");
    }
    Ok(guest_key_path)
}

fn ensure_existing_signing_key_matches_public_key_v1(
    path: &Path,
    expected_public_key: &str,
    mismatch_message: &str,
) -> Result<()> {
    require_existing_linux_signing_key_v1(path)?;
    let public_key = derive_public_key_v1(path)?;
    if public_key != expected_public_key {
        bail!("{mismatch_message}");
    }
    Ok(())
}

fn resume_guest_publisher_pairing_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    challenge_id: &str,
) -> Result<GuestPublisherPairingGuestIntentV1> {
    let path = guest_intent_path_v1(executor, challenge_id);
    let bytes = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
    let intent: GuestPublisherPairingGuestIntentV1 =
        serde_json::from_slice(&bytes).context("decode guest pairing intent")?;
    verify_lifecycle_signature_v1(&intent.schema_owner, &intent, &intent.signature)
        .context("verify guest pairing intent signature")?;
    Ok(intent)
}

fn verify_guest_publisher_bootstrap_transcript_v1(
    ticket: &GuestPublisherPairingTicketV1,
    intent: &GuestPublisherPairingGuestIntentV1,
    hello: &GuestPublisherBootstrapHelloV1,
    transcript: &GuestPublisherBootstrapTranscriptV1,
) -> Result<()> {
    let signer_der = base64url_decode_v1(&ticket.signer_spki_der)?;
    let normalized_der = parse_p256_spki_der_v1(&signer_der)?;
    let ticket_sha256 = sha256_hex_bytes_v1(&canonical_guest_publisher_pairing_ticket_v1(ticket)?)?;
    if transcript.ticket_sha256 != ticket_sha256 {
        bail!("guest bootstrap transcript ticket_sha256 does not match the pairing ticket");
    }
    let hello_sha256 = sha256_hex_bytes_v1(&canonical_json_bytes_of_v1(
        serde_json::to_value(hello).context("serialize guest bootstrap hello")?,
    )?)?;
    if transcript.hello_sha256 != hello_sha256 {
        bail!("guest bootstrap transcript hello_sha256 does not match the persisted hello");
    }
    if transcript.guest_machine_identity != ticket.challenge.guest_machine_identity
        || transcript.guest_machine_identity != intent.guest_machine_identity
    {
        bail!("guest bootstrap transcript machine identity does not match the pairing intent");
    }
    if transcript.staged_executor_sha256 != intent.guest_artifact_sha256 {
        bail!(
            "guest bootstrap transcript staged executor digest does not match the pairing intent"
        );
    }
    if transcript.guest_nonce != intent.nonce {
        bail!("guest bootstrap transcript guest_nonce does not match the pairing intent");
    }
    if transcript.guest_public_key_sha256 != intent.public_key_sha256 {
        bail!("guest bootstrap transcript public key digest does not match the pairing intent");
    }
    if transcript.guest_test_retirement_commitment != ticket.guest_test_retirement_commitment
        || transcript.guest_test_retirement_commitment != intent.guest_test_retirement_commitment
    {
        bail!("guest bootstrap transcript retirement commitment does not match the pairing intent");
    }
    if hello.guest_machine_identity != intent.guest_machine_identity
        || hello.guest_artifact_sha256 != intent.guest_artifact_sha256
        || hello.nonce != intent.nonce
        || hello.guest_public_key != intent.public_key
    {
        bail!("persisted guest bootstrap hello does not match the pairing intent");
    }
    match intent.hello.as_ref() {
        Some(intent_hello) if intent_hello == hello => {}
        Some(_) => bail!("durable guest pairing intent hello does not match the persisted hello"),
        None => bail!("durable guest pairing intent is missing the persisted hello"),
    }
    let payload = canonical_lifecycle_signature_payload_v1(&transcript.schema_owner, transcript)?;
    let signature_bytes = base64url_decode_v1(&transcript.signature.signature)?;
    verify_p256_p1363_low_s_v1(&normalized_der, &payload, &signature_bytes)
        .context("verify guest bootstrap transcript signature")
}

fn join_guest_host_consumption_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    challenge_id: &str,
) -> Result<()> {
    let marker = guest_consumption_marker_path_v1(executor, challenge_id);
    ensure_guest_artifact_directory_v1(executor, challenge_id)?;
    atomic_write_file_v1(&marker, b"consumed", 0o600)?;
    Ok(())
}

fn commit_guest_publisher_bootstrap_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    ticket: &GuestPublisherPairingTicketV1,
    transcript: &GuestPublisherBootstrapTranscriptV1,
    kill_point: KillPointV1,
) -> Result<LifecyclePublisherAnchorV1> {
    let intent = resume_guest_publisher_pairing_v1(executor, &ticket.challenge.challenge_id)?;
    if guest_consumption_marker_path_v1(executor, &ticket.challenge.challenge_id).exists() {
        bail!("guest pairing challenge already reached a consumed terminal state");
    }
    let hello_path = guest_bootstrap_hello_path_v1(executor, &ticket.challenge.challenge_id);
    let hello: GuestPublisherBootstrapHelloV1 = serde_json::from_slice(
        &fs::read(&hello_path).with_context(|| format!("read {}", hello_path.display()))?,
    )
    .context("decode persisted guest bootstrap hello")?;
    verify_guest_publisher_bootstrap_transcript_v1(ticket, &intent, &hello, transcript)?;
    require_committable_guest_pairing_ticket_v1(ticket, &hello)?;
    if intent.challenge_id != ticket.challenge.challenge_id
        || intent.scope_id != ticket.current_anchor.scope_id
    {
        bail!("guest pairing intent does not match the supplied ticket");
    }
    if kill_point == KillPointV1::Transcript {
        std::process::exit(103);
    }
    let mut durable_intent = intent.clone();
    durable_intent.transcript = Some(transcript.clone());
    durable_intent.state = "TranscriptDurable".to_string();
    let durable_intent =
        persist_guest_pairing_intent_v1(executor, &ticket.challenge.challenge_id, &durable_intent)?;

    let current_state = open_linux_publisher_protected_state_v1(executor)?;
    let current_anchor_sha256 = lifecycle_anchor_sha256_v1(&current_state.current_anchor)?;
    let guest_key_path = ensure_guest_signing_key_matches_durable_intent_v1(
        executor,
        &ticket.challenge.challenge_id,
        &durable_intent,
    )?;
    fs::create_dir_all(&executor.publisher_directory)
        .with_context(|| format!("create {}", executor.publisher_directory.display()))?;
    set_mode_v1(&executor.publisher_directory, 0o700)?;

    let mut anchor = LifecyclePublisherAnchorV1 {
        schema_owner: "substrate.lifecycle-publisher-anchor".to_string(),
        schema_version: 1,
        authority_domain: "linux_system".to_string(),
        host_context_commitment: ticket.challenge.host_context_commitment.clone(),
        platform_mapping_commitment: ticket.challenge.platform_mapping_commitment.clone(),
        scope_id: ticket.current_anchor.scope_id.clone(),
        manifest_generation: 1,
        manifest_sha256: ticket.current_anchor.manifest_sha256.clone(),
        action_receipt_index_revision: 0,
        action_receipt_index_sha256: ticket.current_anchor.action_receipt_index_sha256.clone(),
        head_sha256: ticket.current_anchor.head_sha256.clone(),
        previous_anchor_sha256: Some(ticket.current_anchor_sha256.clone()),
        request_sha256: ticket.challenge_sha256.clone(),
        requester_principal: "guest-bootstrap".to_string(),
        attempt_nonce: ticket.challenge.challenge_id.clone(),
        executor_identity: ManagedExecutorIdentityV1 {
            source_commit: ticket.challenge.source_commit.clone(),
            source_tree: ticket.challenge.source_tree.clone(),
            source_ref: ticket.challenge.source_ref.clone(),
            target_triple: std::env::consts::ARCH.to_string(),
            artifact_sha256: durable_intent.guest_artifact_sha256.clone(),
            artifact_path: executor.executor_path.display().to_string(),
            toolchain: None,
            code_identity: None,
        },
        signature: LifecycleSignatureV1 {
            algorithm: "ed25519-v1".to_string(),
            public_key: String::new(),
            signature: String::new(),
        },
    };
    anchor.signature = sign_linux_struct_v1(
        &guest_key_path,
        &anchor.schema_owner,
        serde_json::to_value(&anchor).context("serialize guest lifecycle anchor")?,
    )?;
    if anchor.signature.public_key != durable_intent.public_key {
        bail!("guest lifecycle anchor key does not match the durable guest pairing intent");
    }

    let state = LifecyclePublisherProtectedStateV1 {
        schema_owner: "substrate.lifecycle-publisher-protected-state".to_string(),
        schema_version: 1,
        current_anchor: anchor.clone(),
        counter: 1,
        prepared_record: None,
        previous_protected_state_sha256: None,
        state_revision: 1,
    };
    let expected_state_sha256 =
        sha256_hex_bytes_v1(&canonical_lifecycle_publisher_protected_state_v1(&state)?)?;
    let final_key_path = executor.signing_key_path.clone();
    if current_anchor_sha256 == ticket.current_anchor_sha256 {
        ensure_existing_signing_key_matches_public_key_v1(
            &final_key_path,
            &current_state.current_anchor.signature.public_key,
            "existing publisher signing key does not exact-join the ticket current anchor",
        )?;
        materialize_guest_signing_key_v1(&guest_key_path, &final_key_path)?;
        compare_and_swap_linux_publisher_protected_state_v1(executor, &current_state, &state)?;
    } else {
        let current_state_sha256 = sha256_hex_bytes_v1(
            &canonical_lifecycle_publisher_protected_state_v1(&current_state)?,
        )?;
        if current_state_sha256 != expected_state_sha256 {
            bail!("existing publisher protected state does not exact-join the durable guest bootstrap transcript");
        }
        let guest_key_sha256 = signing_key_sha256_v1(&guest_key_path)?;
        require_existing_linux_signing_key_v1(&final_key_path)?;
        let final_key_sha256 = signing_key_sha256_v1(&final_key_path)?;
        if final_key_sha256 != guest_key_sha256 {
            bail!("existing publisher signing key does not exact-join the durable guest pairing intent");
        }
    }
    anchor.signature = sign_linux_struct_v1(
        &final_key_path,
        &anchor.schema_owner,
        serde_json::to_value(&anchor).context("serialize guest lifecycle anchor")?,
    )?;
    if anchor.signature.public_key != durable_intent.public_key {
        bail!("guest lifecycle anchor key does not match the durable guest pairing intent");
    }
    if kill_point == KillPointV1::Commit {
        std::process::exit(104);
    }
    join_guest_host_consumption_v1(executor, &ticket.challenge.challenge_id)?;
    Ok(anchor)
}

fn retire_linux_test_publisher_v1(executor: &LinuxManagedArtifactExecutorV1) -> Result<()> {
    let files = [
        executor.protected_state_path.as_path(),
        executor.signing_key_path.as_path(),
        executor.endpoint_path.as_path(),
    ];
    for path in files {
        if path.exists() {
            if fs::symlink_metadata(path)
                .with_context(|| format!("stat {}", path.display()))?
                .file_type()
                .is_socket()
            {
                fs::remove_file(path)
                    .with_context(|| format!("remove socket {}", path.display()))?;
            } else {
                fs::remove_file(path).with_context(|| format!("remove {}", path.display()))?;
            }
        }
    }
    if executor.publisher_directory.exists() {
        fs::remove_dir_all(&executor.publisher_directory)
            .with_context(|| format!("remove {}", executor.publisher_directory.display()))?;
    }
    Ok(())
}

fn execute_service_state_action_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    unit: &str,
    action: &str,
    kill_point: KillPointV1,
) -> Result<Value> {
    let managed_action = match action {
        "start" => ManagedActionV1::Start,
        "stop" => ManagedActionV1::Stop,
        "restore" => ManagedActionV1::Restore,
        "enable" => ManagedActionV1::Enable,
        "disable" => ManagedActionV1::Disable,
        other => bail!("unknown service-state action {other}"),
    };
    let role = match unit {
        DEFAULT_WORLD_SERVICE_UNIT => "linux.host.service-state(service)",
        DEFAULT_WORLD_SOCKET_UNIT => "linux.host.service-state(socket)",
        DEFAULT_LIFECYCLE_SERVICE_UNIT => "linux.publisher.service-state(service)",
        DEFAULT_LIFECYCLE_SOCKET_UNIT => "linux.publisher.service-state(socket)",
        _ => bail!("unknown service unit {unit}"),
    };
    let request = ManagedLifecyclePublisherRequestV1 {
        host_context_commitment: "0".repeat(64),
        platform_mapping_commitment: None,
        scope_id: "00000000-0000-7000-8000-000000000001".to_string(),
        current_anchor_counter: 0,
        current_anchor_sha256: "0".repeat(64),
        manifest_generation: 1,
        manifest_sha256: "0".repeat(64),
        role: substrate_common::ManagedArtifactRoleV1(role.to_string()),
        action: managed_action,
        object_identity: ManagedArtifactIdentityV1 {
            scope_id: "00000000-0000-7000-8000-000000000001".to_string(),
            parent_identity: "/".to_string(),
            name_identity: unit.to_string(),
            physical_identity: join_linux_role_identity_v1(executor, role)?
                .display()
                .to_string(),
            metadata: None,
        },
        requester_principal: "test-harness".to_string(),
        attempt_nonce: format!(
            "00000000-0000-7000-8000-{}",
            &sha256_hex_bytes_v1(format!("{unit}:{action}").as_bytes())?[..12]
        ),
        expected_executor_build: ManagedExecutorIdentityV1 {
            source_commit: "0".repeat(40),
            source_tree: "0".repeat(40),
            source_ref: "refs/heads/test".to_string(),
            target_triple: format!("{}-unknown-linux-gnu", std::env::consts::ARCH),
            artifact_sha256: sha256_hex_bytes_v1(b"test-executor")?,
            artifact_path: executor.executor_path.display().to_string(),
            toolchain: None,
            code_identity: None,
        },
    };
    let observation = execute_linux_managed_action_v1(executor, &request, true)?;
    let observation = observation
        .get("observation")
        .cloned()
        .unwrap_or(observation);
    if kill_point == KillPointV1::Observe {
        std::process::exit(111);
    }
    Ok(json!({
        "role": role,
        "action": action,
        "observation": observation,
    }))
}

fn open_linux_lifecycle_capsule_v1(executor: &LinuxManagedArtifactExecutorV1) -> Result<()> {
    ensure_directory_mode_v1(&executor.state_root, 0o750, true)?;
    ensure_directory_mode_v1(&executor.lifecycle_container, 0o700, false)
}

fn join_linux_role_identity_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    role: &str,
) -> Result<PathBuf> {
    Ok(match role {
        "linux.publisher.executor" => executor.executor_path.clone(),
        "linux.publisher.lifecycle-container" => executor.lifecycle_container.clone(),
        "linux.publisher.state-directory" => executor.publisher_directory.clone(),
        "linux.publisher.service-unit" => executor.service_unit_path.clone(),
        "linux.publisher.socket-unit" => executor.socket_unit_path.clone(),
        "linux.publisher.endpoint" => executor.endpoint_path.clone(),
        "linux.publisher.signing-key" => executor.signing_key_path.clone(),
        "linux.publisher.current-anchor" => executor.protected_state_path.clone(),
        "linux.publisher.bootstrap-intent" => current_bootstrap_intent_path_v1(executor)?,
        "linux.host.service-state(service)" => PathBuf::from(DEFAULT_WORLD_SERVICE_UNIT),
        "linux.host.service-state(socket)" => PathBuf::from(DEFAULT_WORLD_SOCKET_UNIT),
        "linux.publisher.service-state(service)" => PathBuf::from(DEFAULT_LIFECYCLE_SERVICE_UNIT),
        "linux.publisher.service-state(socket)" => PathBuf::from(DEFAULT_LIFECYCLE_SOCKET_UNIT),
        other => bail!("unsupported linux managed role {other}"),
    })
}

fn publisher_prepared_record_path_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    attempt_nonce: &str,
) -> Result<PathBuf> {
    Ok(executor.publisher_directory.join("prepared").join(format!(
        "{}.json",
        sha256_hex_bytes_v1(attempt_nonce.as_bytes())?
    )))
}

fn publisher_receipt_path_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    manifest_generation: u64,
    receipt_id: &str,
) -> PathBuf {
    executor
        .publisher_directory
        .join(publisher_receipt_relative_path_v1(
            manifest_generation,
            receipt_id,
        ))
}

fn publisher_receipt_relative_path_v1(manifest_generation: u64, receipt_id: &str) -> String {
    format!("receipts/{manifest_generation}/receipt.{receipt_id}.json")
}

fn publisher_response_cache_path_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    attempt_nonce: &str,
) -> Result<PathBuf> {
    Ok(executor.publisher_directory.join("responses").join(format!(
        "{}.json",
        sha256_hex_bytes_v1(attempt_nonce.as_bytes())?
    )))
}

fn guest_artifact_directory_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    challenge_id: &str,
) -> PathBuf {
    executor
        .state_root
        .join(".substrate-lifecycle-guest-pairings")
        .join(challenge_id)
}

fn guest_signing_key_path_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    challenge_id: &str,
) -> PathBuf {
    guest_artifact_directory_v1(executor, challenge_id).join("guest-signing-key.v1")
}

fn guest_bootstrap_hello_path_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    challenge_id: &str,
) -> PathBuf {
    guest_artifact_directory_v1(executor, challenge_id).join("hello.json")
}

fn guest_consumption_marker_path_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    challenge_id: &str,
) -> PathBuf {
    guest_artifact_directory_v1(executor, challenge_id).join("consumed.marker")
}

fn guest_intent_leaf_v1(challenge_id: &str) -> String {
    format!(".substrate-lifecycle-pairing-intent-v1.{challenge_id}.json")
}

fn guest_intent_path_v1(executor: &LinuxManagedArtifactExecutorV1, challenge_id: &str) -> PathBuf {
    executor.state_root.join(guest_intent_leaf_v1(challenge_id))
}

fn write_guest_artifact_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    challenge_id: &str,
    leaf: &str,
    value: &Value,
) -> Result<()> {
    let directory = ensure_guest_artifact_directory_v1(executor, challenge_id)?;
    let path = directory.join(leaf);
    let bytes = canonical_json_value_to_vec_v1(value)?;
    atomic_write_file_v1(&path, &bytes, 0o600)?;
    Ok(())
}

fn ensure_guest_artifact_directory_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    challenge_id: &str,
) -> Result<PathBuf> {
    let directory = guest_artifact_directory_v1(executor, challenge_id);
    fs::create_dir_all(&directory).with_context(|| format!("create {}", directory.display()))?;
    set_mode_v1(&directory, 0o700)?;
    Ok(directory)
}

fn persist_guest_pairing_intent_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    challenge_id: &str,
    intent: &GuestPublisherPairingGuestIntentV1,
) -> Result<GuestPublisherPairingGuestIntentV1> {
    let path = guest_intent_path_v1(executor, challenge_id);
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_file() => {}
        Ok(_) => bail!(
            "guest pairing intent path is not a regular file: {}",
            path.display()
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bail!("missing guest pairing intent {}", path.display())
        }
        Err(error) => return Err(error).with_context(|| format!("stat {}", path.display())),
    }
    let signing_key_path = guest_signing_key_path_v1(executor, challenge_id);
    let mut durable_intent = intent.clone();
    durable_intent.signature = sign_linux_struct_v1(
        &signing_key_path,
        &durable_intent.schema_owner,
        serde_json::to_value(&durable_intent).context("serialize guest pairing intent")?,
    )?;
    let bytes = canonical_json_bytes_of_v1(
        serde_json::to_value(&durable_intent).context("serialize guest pairing intent")?,
    )?;
    atomic_write_file_v1(&path, &bytes, 0o600)?;
    Ok(durable_intent)
}

fn load_or_create_linux_signing_key_v1(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.file_type().is_file() {
                bail!("signing key is not a regular file: {}", path.display());
            }
            return Ok(());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error).with_context(|| format!("stat {}", path.display())),
    }
    create_linux_signing_key_at_absent_path_v1(path)
}

fn create_linux_signing_key_v1(path: &Path) -> Result<()> {
    if path_exists_or_symlink_v1(path)? {
        bail!(
            "refusing to overwrite existing signing key {}",
            path.display()
        );
    }
    create_linux_signing_key_at_absent_path_v1(path)
}

fn create_linux_signing_key_at_absent_path_v1(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let leaf = path
        .file_name()
        .and_then(|leaf| leaf.to_str())
        .ok_or_else(|| anyhow!("signing key path has no file name"))?;
    let tmp = path
        .parent()
        .ok_or_else(|| anyhow!("signing key path has no parent"))?
        .join(format!(".{leaf}.tmp-{}", unix_now_ns_v1()?));
    let output = Command::new("openssl")
        .args(["genpkey", "-algorithm", "ed25519", "-out"])
        .arg(&tmp)
        .output()
        .context("spawn openssl genpkey")?;
    if !output.status.success() {
        bail!(
            "openssl genpkey failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("chmod {}", tmp.display()))?;
    let tmp_file = OpenOptions::new()
        .read(true)
        .open(&tmp)
        .with_context(|| format!("open {}", tmp.display()))?;
    tmp_file
        .sync_all()
        .with_context(|| format!("sync {}", tmp.display()))?;
    fs::rename(&tmp, path)
        .with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
    let final_file = OpenOptions::new()
        .read(true)
        .open(path)
        .with_context(|| format!("open {}", path.display()))?;
    final_file
        .sync_all()
        .with_context(|| format!("sync {}", path.display()))?;
    linux_fsync_parent_v1(path)?;
    Ok(())
}

fn require_existing_linux_signing_key_v1(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(()),
        Ok(_) => bail!("signing key is not a regular file: {}", path.display()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bail!("missing signing key {}", path.display())
        }
        Err(error) => Err(error).with_context(|| format!("stat {}", path.display())),
    }
}

fn derive_public_key_v1(path: &Path) -> Result<String> {
    let output = Command::new("openssl")
        .args(["pkey", "-in"])
        .arg(path)
        .args(["-pubout", "-outform", "DER"])
        .output()
        .with_context(|| format!("derive public key from {}", path.display()))?;
    if !output.status.success() {
        bail!(
            "openssl pkey failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let raw_key = extract_ed25519_public_key_v1(&output.stdout)?;
    Ok(base64url_encode_v1(&raw_key))
}

fn sign_linux_anchor_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    anchor: &LifecyclePublisherAnchorV1,
) -> Result<LifecycleSignatureV1> {
    sign_linux_struct_v1(
        &executor.signing_key_path,
        &anchor.schema_owner,
        serde_json::to_value(anchor).context("serialize lifecycle anchor")?,
    )
}

fn sign_linux_struct_v1(
    key_path: &Path,
    schema_owner: &str,
    value: Value,
) -> Result<LifecycleSignatureV1> {
    require_existing_linux_signing_key_v1(key_path)?;
    let payload = canonical_lifecycle_signature_payload_v1(schema_owner, &value)?;
    let payload_path = unique_temp_path_v1("payload");
    let signature_path = unique_temp_path_v1("signature");
    fs::write(&payload_path, &payload)
        .with_context(|| format!("write {}", payload_path.display()))?;
    let output = Command::new("openssl")
        .args(["pkeyutl", "-sign", "-inkey"])
        .arg(key_path)
        .args(["-rawin", "-in"])
        .arg(&payload_path)
        .args(["-out"])
        .arg(&signature_path)
        .output()
        .with_context(|| format!("sign payload with {}", key_path.display()))?;
    if !output.status.success() {
        let _ = fs::remove_file(&payload_path);
        let _ = fs::remove_file(&signature_path);
        bail!(
            "openssl pkeyutl failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let signature_bytes =
        fs::read(&signature_path).with_context(|| format!("read {}", signature_path.display()))?;
    let public_key = derive_public_key_v1(key_path)?;
    let signature = LifecycleSignatureV1 {
        algorithm: "ed25519-v1".to_string(),
        public_key,
        signature: base64url_encode_v1(&signature_bytes),
    };
    let _ = fs::remove_file(&payload_path);
    let _ = fs::remove_file(&signature_path);
    Ok(signature)
}

fn linux_seqpacket_socket_v1() -> Result<OwnedFd> {
    // SAFETY: libc socket is called with fixed AF_UNIX/SOCK_SEQPACKET parameters.
    let fd = unsafe { socket(AF_UNIX_V1, SOCK_SEQPACKET_V1 | SOCK_CLOEXEC_V1, 0) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error()).context("create seqpacket socket");
    }
    // SAFETY: fd is a fresh file descriptor returned by socket.
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

fn bind_seqpacket_listener_v1(path: &Path) -> Result<UnixListener> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    if path.exists() {
        let metadata =
            fs::symlink_metadata(path).with_context(|| format!("stat {}", path.display()))?;
        if metadata.file_type().is_socket() {
            if live_seqpacket_listener_exists_v1(path)? {
                bail!(
                    "publisher endpoint already has a live listener: {}",
                    path.display()
                );
            }
            fs::remove_file(path)
                .with_context(|| format!("remove stale socket {}", path.display()))?;
        } else {
            bail!(
                "publisher endpoint path is already occupied by a non-socket: {}",
                path.display()
            );
        }
    }

    let socket_fd = linux_seqpacket_socket_v1()?;
    let sockaddr = sockaddr_un_for_path_v1(path)?;
    let length = sockaddr_len_v1(path)?;
    // SAFETY: bind is called with a valid AF_UNIX sockaddr and open socket fd.
    let bind_rc = unsafe {
        bind(
            socket_fd.as_raw_fd(),
            (&sockaddr as *const SockAddrUnV1).cast(),
            length,
        )
    };
    if bind_rc != 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("bind seqpacket listener {}", path.display()));
    }
    // SAFETY: listen is called on the successfully bound socket fd.
    let listen_rc = unsafe { listen(socket_fd.as_raw_fd(), 16) };
    if listen_rc != 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("listen seqpacket listener {}", path.display()));
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("chmod {}", path.display()))?;
    // SAFETY: socket_fd remains owned by this process and is transferred into UnixListener.
    Ok(unsafe { UnixListener::from_raw_fd(socket_fd.into_raw_fd()) })
}

fn probe_error_kind_v1(error: &anyhow::Error) -> Option<std::io::ErrorKind> {
    error
        .downcast_ref::<std::io::Error>()
        .map(std::io::Error::kind)
}

fn is_unavailable_listener_probe_error_v1(error: &anyhow::Error) -> bool {
    matches!(
        probe_error_kind_v1(error),
        Some(std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::NotFound)
    )
}

fn is_live_listener_probe_error_v1(error: &anyhow::Error) -> bool {
    matches!(
        probe_error_kind_v1(error),
        Some(
            std::io::ErrorKind::BrokenPipe
                | std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::TimedOut
                | std::io::ErrorKind::UnexpectedEof
                | std::io::ErrorKind::WouldBlock
        )
    )
}

fn live_seqpacket_listener_exists_v1(path: &Path) -> Result<bool> {
    let socket = match connect_seqpacket_socket_v1(path) {
        Ok(socket) => socket,
        Err(error) if is_unavailable_listener_probe_error_v1(&error) => return Ok(false),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("probe seqpacket listener {}", path.display()))
        }
    };
    socket
        .set_read_timeout(Some(PUBLISHER_PROBE_TIMEOUT_V1))
        .with_context(|| format!("set probe read timeout for {}", path.display()))?;
    socket
        .set_write_timeout(Some(PUBLISHER_PROBE_TIMEOUT_V1))
        .with_context(|| format!("set probe write timeout for {}", path.display()))?;
    let ping_bytes = publisher_ping_bytes_v1()?;
    match send_seqpacket_frame_v1(&socket, &ping_bytes) {
        Ok(()) => {}
        Err(error) if is_live_listener_probe_error_v1(&error) => return Ok(true),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("probe seqpacket listener {}", path.display()))
        }
    }
    let mut response = [0u8; 1];
    match socket.recv(&mut response) {
        Ok(_) => Ok(true),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::BrokenPipe
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::TimedOut
                    | std::io::ErrorKind::UnexpectedEof
                    | std::io::ErrorKind::WouldBlock
            ) =>
        {
            Ok(true)
        }
        Err(error) => {
            Err(error).with_context(|| format!("probe seqpacket listener {}", path.display()))
        }
    }
}

fn live_stream_listener_exists_v1(path: &Path) -> Result<bool> {
    let mut stream = match UnixStream::connect(path) {
        Ok(stream) => stream,
        Err(error) if error.kind() == std::io::ErrorKind::ConnectionRefused => return Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(error).with_context(|| format!("probe stream listener {}", path.display()))
        }
    };
    stream
        .set_read_timeout(Some(PUBLISHER_PROBE_TIMEOUT_V1))
        .with_context(|| format!("set probe read timeout for {}", path.display()))?;
    stream
        .set_write_timeout(Some(PUBLISHER_PROBE_TIMEOUT_V1))
        .with_context(|| format!("set probe write timeout for {}", path.display()))?;
    let ping_bytes = publisher_ping_bytes_v1()?;
    match send_frame_v1(&stream, &ping_bytes) {
        Ok(()) => {}
        Err(error) if is_live_listener_probe_error_v1(&error) => return Ok(true),
        Err(error) => {
            return Err(error).with_context(|| format!("probe stream listener {}", path.display()))
        }
    }
    match stream.shutdown(Shutdown::Write) {
        Ok(()) => {}
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::BrokenPipe
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::TimedOut
                    | std::io::ErrorKind::UnexpectedEof
                    | std::io::ErrorKind::WouldBlock
            ) =>
        {
            return Ok(true);
        }
        Err(error) => {
            return Err(error).with_context(|| format!("probe stream listener {}", path.display()))
        }
    }
    let mut response = [0u8; 1];
    match stream.read(&mut response) {
        Ok(_) => Ok(true),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::BrokenPipe
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::TimedOut
                    | std::io::ErrorKind::UnexpectedEof
                    | std::io::ErrorKind::WouldBlock
            ) =>
        {
            Ok(true)
        }
        Err(error) => {
            Err(error).with_context(|| format!("probe stream listener {}", path.display()))
        }
    }
}

fn publisher_listener_is_live_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    path: &Path,
) -> Result<bool> {
    if endpoint_kind_v1(&observe_endpoint_v1(path)?) != Some("socket") {
        return Ok(false);
    }
    match executor.transport {
        TransportKindV1::SeqPacket => live_seqpacket_listener_exists_v1(path),
        TransportKindV1::Stream => live_stream_listener_exists_v1(path),
    }
}

fn linux_peer_credentials_v1(fd: RawFd) -> Result<UCredV1> {
    let mut cred = UCredV1 {
        pid: 0,
        uid: 0,
        gid: 0,
    };
    let mut len = std::mem::size_of::<UCredV1>() as u32;
    // SAFETY: getsockopt writes a fixed-size ucred structure into the provided buffer.
    let rc = unsafe {
        getsockopt(
            fd,
            SOL_SOCKET_V1,
            SO_PEERCRED_V1,
            (&mut cred as *mut UCredV1).cast(),
            &mut len,
        )
    };
    if rc != 0 {
        return Err(std::io::Error::last_os_error()).context("query peer credentials");
    }
    Ok(cred)
}

fn linux_openat2_nofollow_v1(path: &Path) -> Result<File> {
    let c_path = CString::new(path.as_os_str().as_encoded_bytes())
        .with_context(|| format!("path contains NUL: {}", path.display()))?;
    // SAFETY: open is called with fixed flags and a NUL-terminated path.
    let fd = unsafe {
        open(
            c_path.as_ptr(),
            O_PATH_V1 | O_DIRECTORY_V1 | O_CLOEXEC_V1 | O_NOFOLLOW_V1,
            0,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("open nofollow path {}", path.display()));
    }
    // SAFETY: fd is a fresh descriptor returned by open.
    Ok(unsafe { File::from_raw_fd(fd) })
}

fn linux_otmpfile_linkat_v1(parent: &Path, leaf: &str, bytes: &[u8], mode: u32) -> Result<()> {
    let parent_file = linux_openat2_nofollow_v1(parent)?;
    let c_parent = CString::new(parent.as_os_str().as_encoded_bytes())
        .with_context(|| format!("path contains NUL: {}", parent.display()))?;
    // SAFETY: open is called against a validated directory path with O_TMPFILE.
    let tmp_fd = unsafe {
        open(
            c_parent.as_ptr(),
            O_TMPFILE_V1 | O_RDWR_V1 | O_CLOEXEC_V1,
            mode,
        )
    };
    if tmp_fd < 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("open O_TMPFILE under {}", parent.display()));
    }
    // SAFETY: tmp_fd is a fresh descriptor returned by open.
    let mut tmp_file = unsafe { File::from_raw_fd(tmp_fd) };
    tmp_file
        .write_all(bytes)
        .with_context(|| format!("write O_TMPFILE under {}", parent.display()))?;
    tmp_file
        .sync_all()
        .with_context(|| format!("sync O_TMPFILE under {}", parent.display()))?;
    let c_empty = CString::new("").expect("empty CString");
    let c_leaf = CString::new(leaf).with_context(|| format!("leaf contains NUL: {leaf}"))?;
    // SAFETY: linkat is called with AT_EMPTY_PATH to atomically link the unnamed tmpfile.
    let rc = unsafe {
        linkat(
            tmp_file.as_raw_fd(),
            c_empty.as_ptr(),
            parent_file.as_raw_fd(),
            c_leaf.as_ptr(),
            AT_EMPTY_PATH_V1,
        )
    };
    if rc != 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("link O_TMPFILE into {}/{}", parent.display(), leaf));
    }
    drop(tmp_file);
    linux_fsync_parent_v1(&parent.join(leaf))
}

fn linux_fsync_parent_v1(path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("path {} has no parent", path.display()))?;
    let directory = OpenOptions::new()
        .read(true)
        .custom_flags(O_DIRECTORY_V1)
        .open(parent)
        .with_context(|| format!("open parent directory {}", parent.display()))?;
    directory
        .sync_all()
        .with_context(|| format!("fsync parent directory {}", parent.display()))
}

fn atomic_write_file_v1(path: &Path, bytes: &[u8], mode: u32) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("path {} has no parent", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    let leaf = path
        .file_name()
        .and_then(|leaf| leaf.to_str())
        .ok_or_else(|| anyhow!("path {} has no file name", path.display()))?;
    let temp_path = parent.join(format!(
        ".{leaf}.tmp-{}-{}",
        std::process::id(),
        unix_now_ns_v1()?
    ));
    let mut temp_file = OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .mode(mode)
        .open(&temp_path)
        .with_context(|| format!("create {}", temp_path.display()))?;
    temp_file
        .write_all(bytes)
        .with_context(|| format!("write {}", temp_path.display()))?;
    fs::set_permissions(&temp_path, fs::Permissions::from_mode(mode))
        .with_context(|| format!("chmod {}", temp_path.display()))?;
    temp_file
        .sync_all()
        .with_context(|| format!("sync {}", temp_path.display()))?;
    drop(temp_file);
    fs::rename(&temp_path, path)
        .with_context(|| format!("rename {} -> {}", temp_path.display(), path.display()))?;
    linux_fsync_parent_v1(path)
}

fn publisher_attempt_lock_path_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    attempt_nonce: &str,
) -> Result<PathBuf> {
    Ok(executor.publisher_directory.join("locks").join(format!(
        "{}.lock",
        sha256_hex_bytes_v1(attempt_nonce.as_bytes())?
    )))
}

fn acquire_publisher_attempt_lock_v1(
    executor: &LinuxManagedArtifactExecutorV1,
    attempt_nonce: &str,
) -> Result<PublisherAttemptLockV1> {
    let path = publisher_attempt_lock_path_v1(executor, attempt_nonce)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .mode(0o600)
        .open(&path)
        .with_context(|| format!("open attempt lock {}", path.display()))?;
    // SAFETY: flock operates on the open lock file descriptor for this process.
    let rc = unsafe { flock(file.as_raw_fd(), LOCK_EX_V1 | LOCK_NB_V1) };
    if rc != 0 {
        let error = std::io::Error::last_os_error();
        if error.kind() == std::io::ErrorKind::WouldBlock {
            bail!("request attempt is already in progress");
        }
        return Err(error).with_context(|| format!("lock attempt {}", path.display()));
    }
    Ok(PublisherAttemptLockV1 { _file: file })
}

fn connect_seqpacket_socket_v1(path: &Path) -> Result<UnixDatagram> {
    let socket_fd = linux_seqpacket_socket_v1()?;
    let sockaddr = sockaddr_un_for_path_v1(path)?;
    let length = sockaddr_len_v1(path)?;
    // SAFETY: connect is called with a valid AF_UNIX sockaddr and open socket fd.
    let rc = unsafe {
        connect(
            socket_fd.as_raw_fd(),
            (&sockaddr as *const SockAddrUnV1).cast(),
            length,
        )
    };
    if rc != 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("connect seqpacket {}", path.display()));
    }
    Ok(UnixDatagram::from(socket_fd))
}

fn dup_fd_v1(fd: RawFd) -> Result<RawFd> {
    // SAFETY: dup duplicates an existing file descriptor owned by this process.
    let duplicated = unsafe { dup(fd) };
    if duplicated < 0 {
        return Err(std::io::Error::last_os_error()).with_context(|| format!("dup fd {fd}"));
    }
    Ok(duplicated)
}

fn stdin_tty_matches_path_v1(path: &Path) -> bool {
    if unsafe { isatty(STDIN_FILENO_V1) } != 1 {
        return false;
    }
    fs::read_link("/proc/self/fd/0")
        .map(|current| current == path)
        .unwrap_or(false)
}

fn sockaddr_un_for_path_v1(path: &Path) -> Result<SockAddrUnV1> {
    let path_bytes = path.as_os_str().as_encoded_bytes();
    if path_bytes.len() >= 108 {
        bail!("unix socket path is too long: {}", path.display());
    }
    let mut addr = SockAddrUnV1 {
        sun_family: AF_UNIX_V1 as u16,
        sun_path: [0; 108],
    };
    for (index, byte) in path_bytes.iter().enumerate() {
        addr.sun_path[index] = *byte as i8;
    }
    Ok(addr)
}

fn sockaddr_len_v1(path: &Path) -> Result<u32> {
    let path_bytes = path.as_os_str().as_encoded_bytes();
    if path_bytes.len() >= 108 {
        bail!("unix socket path is too long: {}", path.display());
    }
    Ok((std::mem::size_of::<u16>() + path_bytes.len() + 1) as u32)
}

fn read_frame_from_fd_v1(fd: RawFd) -> Result<Vec<u8>> {
    if socket_type_v1(fd)? == SOCK_SEQPACKET_V1 {
        // SAFETY: fd is owned by the current process for the duration of this read.
        let socket = unsafe { UnixDatagram::from(OwnedFd::from_raw_fd(fd)) };
        let result = read_seqpacket_frame_v1(&socket);
        let _ = socket.into_raw_fd();
        result
    } else {
        // SAFETY: fd is owned by the current process for the duration of this read.
        let file = unsafe { File::from_raw_fd(fd) };
        let result = read_frame_v1(&file);
        let _ = file.into_raw_fd();
        result
    }
}

fn write_json_frame_to_fd_v1(fd: RawFd, value: &Value) -> Result<()> {
    let bytes = serde_json::to_vec(value).context("encode JSON frame")?;
    if socket_type_v1(fd)? == SOCK_SEQPACKET_V1 {
        // SAFETY: fd is owned by the current process for the duration of this write.
        let socket = unsafe { UnixDatagram::from(OwnedFd::from_raw_fd(fd)) };
        let result = send_seqpacket_frame_v1(&socket, &bytes).context("write JSON frame");
        let _ = socket.into_raw_fd();
        result
    } else {
        // SAFETY: fd is owned by the current process for the duration of this write.
        let mut file = unsafe { File::from_raw_fd(fd) };
        let result = file.write_all(&bytes).context("write JSON frame");
        let _ = file.into_raw_fd();
        result
    }
}

fn read_frame_v1<R>(mut reader: R) -> Result<Vec<u8>>
where
    R: Read,
{
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).context("read frame")?;
    if bytes.is_empty() {
        bail!("frame was empty");
    }
    if bytes.len() > MAX_FRAME_BYTES {
        bail!("frame exceeded the 1 MiB packet contract");
    }
    Ok(bytes)
}

fn read_seqpacket_frame_v1(socket: &UnixDatagram) -> Result<Vec<u8>> {
    let mut bytes = vec![0u8; MAX_FRAME_BYTES + 1];
    let read_len = socket.recv(&mut bytes).context("read seqpacket frame")?;
    if read_len == 0 {
        bail!("frame was empty");
    }
    if read_len > MAX_FRAME_BYTES {
        bail!("frame exceeded the 1 MiB packet contract");
    }
    bytes.truncate(read_len);
    let prior_timeout = socket.read_timeout().context("read seqpacket timeout")?;
    socket
        .set_read_timeout(Some(SINGLE_FRAME_FINISH_TIMEOUT_V1))
        .context("set seqpacket finish timeout")?;
    let mut probe = [0u8; 1];
    let trailer = socket.recv(&mut probe);
    socket
        .set_read_timeout(prior_timeout)
        .context("restore seqpacket timeout")?;
    match trailer {
        Ok(0) => {}
        Ok(_) => bail!("seqpacket channel carried more than one frame"),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
            ) =>
        {
            bail!("seqpacket peer did not terminate the single-frame exchange");
        }
        Err(error) => return Err(error).context("verify seqpacket frame termination"),
    }
    Ok(bytes)
}

fn send_seqpacket_frame_v1(socket: &UnixDatagram, bytes: &[u8]) -> Result<()> {
    if bytes.len() > MAX_FRAME_BYTES {
        bail!("frame exceeded the 1 MiB packet contract");
    }
    let written = socket.send(bytes).context("write seqpacket frame")?;
    if written != bytes.len() {
        bail!("seqpacket frame write was truncated");
    }
    shutdown_seqpacket_write_v1(socket)?;
    Ok(())
}

fn shutdown_seqpacket_write_v1(socket: &UnixDatagram) -> Result<()> {
    // SAFETY: shutdown only affects the supplied connected socket descriptor.
    let rc = unsafe { shutdown(socket.as_raw_fd(), SHUT_WR_V1) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error()).context("shutdown seqpacket write");
    }
    Ok(())
}

fn socket_type_v1(fd: RawFd) -> Result<i32> {
    let mut socket_type = 0_i32;
    let mut option_len = std::mem::size_of::<i32>() as u32;
    // SAFETY: getsockopt writes at most option_len bytes into socket_type for the supplied fd.
    let rc = unsafe {
        getsockopt(
            fd,
            SOL_SOCKET_V1,
            SO_TYPE_V1,
            &mut socket_type as *mut i32 as *mut core::ffi::c_void,
            &mut option_len,
        )
    };
    if rc != 0 {
        return Err(std::io::Error::last_os_error()).context("query socket type");
    }
    if option_len != std::mem::size_of::<i32>() as u32 {
        bail!("unexpected SO_TYPE length {option_len}");
    }
    Ok(socket_type)
}

fn send_frame_v1<W>(mut writer: W, bytes: &[u8]) -> Result<()>
where
    W: Write,
{
    if bytes.len() > MAX_FRAME_BYTES {
        bail!("frame exceeded the 1 MiB packet contract");
    }
    writer.write_all(bytes).context("write frame")?;
    writer.flush().context("flush frame")
}

fn is_linux_publisher_ping_v1(value: &Value) -> bool {
    value.get("schema_owner").and_then(Value::as_str) == Some(PUBLISHER_PING_SCHEMA_OWNER_V1)
        && value.get("schema_version").and_then(Value::as_u64) == Some(1)
}

fn publisher_ping_bytes_v1() -> Result<Vec<u8>> {
    serde_json::to_vec(&json!({
        "schema_owner": PUBLISHER_PING_SCHEMA_OWNER_V1,
        "schema_version": 1,
    }))
    .context("encode publisher ping")
}

fn linux_publisher_ping_response_v1() -> Value {
    json!({
        "schema_owner": PUBLISHER_PING_RESPONSE_SCHEMA_OWNER_V1,
        "schema_version": 1,
        "status": "ready",
    })
}

fn trigger_linux_publisher_ready_probe_v1(executor: &LinuxManagedArtifactExecutorV1) -> Result<()> {
    let ping_bytes = publisher_ping_bytes_v1()?;
    let response_bytes = match executor.transport {
        TransportKindV1::Stream => {
            let stream = UnixStream::connect(&executor.endpoint_path)
                .with_context(|| format!("connect {}", executor.endpoint_path.display()))?;
            stream
                .set_read_timeout(Some(PUBLISHER_PROBE_TIMEOUT_V1))
                .with_context(|| {
                    format!(
                        "set probe read timeout for {}",
                        executor.endpoint_path.display()
                    )
                })?;
            stream
                .set_write_timeout(Some(PUBLISHER_PROBE_TIMEOUT_V1))
                .with_context(|| {
                    format!(
                        "set probe write timeout for {}",
                        executor.endpoint_path.display()
                    )
                })?;
            send_frame_v1(&stream, &ping_bytes)?;
            stream
                .shutdown(Shutdown::Write)
                .context("half-close stream ping")?;
            read_frame_v1(&stream)?
        }
        TransportKindV1::SeqPacket => {
            let socket =
                connect_seqpacket_socket_v1(&executor.endpoint_path).with_context(|| {
                    format!("connect seqpacket {}", executor.endpoint_path.display())
                })?;
            socket
                .set_read_timeout(Some(PUBLISHER_PROBE_TIMEOUT_V1))
                .with_context(|| {
                    format!(
                        "set probe read timeout for {}",
                        executor.endpoint_path.display()
                    )
                })?;
            socket
                .set_write_timeout(Some(PUBLISHER_PROBE_TIMEOUT_V1))
                .with_context(|| {
                    format!(
                        "set probe write timeout for {}",
                        executor.endpoint_path.display()
                    )
                })?;
            send_seqpacket_frame_v1(&socket, &ping_bytes)?;
            read_seqpacket_frame_v1(&socket)?
        }
    };
    let response: Value =
        serde_json::from_slice(&response_bytes).context("decode publisher ping response")?;
    if response.get("schema_owner").and_then(Value::as_str)
        != Some(PUBLISHER_PING_RESPONSE_SCHEMA_OWNER_V1)
        || response.get("status").and_then(Value::as_str) != Some("ready")
    {
        bail!("publisher ping probe returned an unexpected response");
    }
    Ok(())
}

fn run_systemctl_v1(verb: &str, unit: &str) -> Result<()> {
    run_systemctl_args_v1(&[verb, unit])
}

fn run_systemctl_args_v1(args: &[&str]) -> Result<()> {
    let status = Command::new("systemctl")
        .args(args)
        .status()
        .with_context(|| format!("spawn systemctl {}", args.join(" ")))?;
    if !status.success() {
        bail!("systemctl {} failed with status {status}", args.join(" "));
    }
    Ok(())
}

fn systemctl_output_v1(verb: &str, unit: &str) -> String {
    Command::new("systemctl")
        .arg(verb)
        .arg(unit)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn observe_service_state_v1(unit: &str, endpoint_path: &str) -> Result<ServiceStateObservationV1> {
    Ok(ServiceStateObservationV1 {
        active: systemctl_output_v1("is-active", unit),
        enabled: systemctl_output_v1("is-enabled", unit),
        endpoint: observe_endpoint_v1(Path::new(endpoint_path))?,
    })
}

fn observe_endpoint_v1(path: &Path) -> Result<Value> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            let file_type = metadata.file_type();
            let kind = if file_type.is_socket() {
                "socket"
            } else if file_type.is_dir() {
                "directory"
            } else if file_type.is_file() {
                "file"
            } else if file_type.is_symlink() {
                "symlink"
            } else {
                "other"
            };
            Ok(json!({
                "path": path.display().to_string(),
                "kind": kind,
                "mode": format!("{:04o}", metadata.permissions().mode() & 0o7777),
                "size": metadata.len(),
            }))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(json!({
            "path": path.display().to_string(),
            "kind": "missing",
        })),
        Err(error) => Err(error).with_context(|| format!("stat {}", path.display())),
    }
}

fn endpoint_kind_v1(value: &Value) -> Option<&str> {
    value.get("kind").and_then(Value::as_str)
}

fn ensure_tty_fd_v1(fd: RawFd, path: &Path, direction: &str) -> Result<()> {
    // SAFETY: isatty only inspects the supplied file descriptor.
    let rc = unsafe { isatty(fd) };
    if rc == 1 {
        return Ok(());
    }
    bail!(
        "guest pairing {} path is not a controlling tty: {}",
        direction,
        path.display()
    );
}

fn managed_action_name_v1(action: ManagedActionV1) -> &'static str {
    match action {
        ManagedActionV1::Create => "create",
        ManagedActionV1::Replace => "replace",
        ManagedActionV1::Remove => "remove",
        ManagedActionV1::Restore => "restore",
        ManagedActionV1::Enable => "enable",
        ManagedActionV1::Disable => "disable",
        ManagedActionV1::Start => "start",
        ManagedActionV1::Stop => "stop",
    }
}

fn canonical_json_bytes_of_v1(value: Value) -> Result<Vec<u8>> {
    canonical_json_value_to_vec_v1(&value)
}

fn canonical_json_value_to_vec_v1(value: &Value) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    encode_json_value_v1(value, &mut output)?;
    Ok(output)
}

fn encode_json_value_v1(value: &Value, output: &mut Vec<u8>) -> Result<()> {
    match value {
        Value::Null => output.extend_from_slice(b"null"),
        Value::Bool(true) => output.extend_from_slice(b"true"),
        Value::Bool(false) => output.extend_from_slice(b"false"),
        Value::Number(number) => output.extend_from_slice(number.to_string().as_bytes()),
        Value::String(string) => output.extend_from_slice(
            serde_json::to_string(string)
                .context("encode string")?
                .as_bytes(),
        ),
        Value::Array(array) => {
            output.push(b'[');
            for (index, item) in array.iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                encode_json_value_v1(item, output)?;
            }
            output.push(b']');
        }
        Value::Object(object) => {
            output.push(b'{');
            let mut first = true;
            let mut keys: Vec<_> = object.keys().collect();
            keys.sort();
            for key in keys {
                if !first {
                    output.push(b',');
                }
                first = false;
                output.extend_from_slice(
                    serde_json::to_string(key).context("encode key")?.as_bytes(),
                );
                output.push(b':');
                encode_json_value_v1(
                    object
                        .get(key)
                        .ok_or_else(|| anyhow!("canonical JSON key {key} disappeared"))?,
                    output,
                )?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

fn sha256_hex_bytes_v1(bytes: &[u8]) -> Result<String> {
    let output = Command::new("openssl")
        .args(["dgst", "-sha256", "-binary"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn openssl dgst")?;
    let mut child = output;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(bytes).context("write digest payload")?;
    }
    let output = child.wait_with_output().context("wait for openssl dgst")?;
    if !output.status.success() {
        bail!(
            "openssl dgst failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(lower_hex_v1(&output.stdout))
}

fn base64url_encode_v1(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut output = String::new();
    let mut index = 0;
    while index + 3 <= bytes.len() {
        let chunk = ((bytes[index] as u32) << 16)
            | ((bytes[index + 1] as u32) << 8)
            | (bytes[index + 2] as u32);
        output.push(TABLE[((chunk >> 18) & 0x3f) as usize] as char);
        output.push(TABLE[((chunk >> 12) & 0x3f) as usize] as char);
        output.push(TABLE[((chunk >> 6) & 0x3f) as usize] as char);
        output.push(TABLE[(chunk & 0x3f) as usize] as char);
        index += 3;
    }
    match bytes.len() - index {
        1 => {
            let chunk = (bytes[index] as u32) << 16;
            output.push(TABLE[((chunk >> 18) & 0x3f) as usize] as char);
            output.push(TABLE[((chunk >> 12) & 0x3f) as usize] as char);
        }
        2 => {
            let chunk = ((bytes[index] as u32) << 16) | ((bytes[index + 1] as u32) << 8);
            output.push(TABLE[((chunk >> 18) & 0x3f) as usize] as char);
            output.push(TABLE[((chunk >> 12) & 0x3f) as usize] as char);
            output.push(TABLE[((chunk >> 6) & 0x3f) as usize] as char);
        }
        _ => {}
    }
    output
}

fn base64url_decode_v1(value: &str) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut quartet = [0u8; 4];
    let mut quartet_len = 0usize;
    for byte in value.bytes() {
        let six = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            _ => bail!("invalid base64url byte {}", byte as char),
        };
        quartet[quartet_len] = six;
        quartet_len += 1;
        if quartet_len == 4 {
            let chunk = ((quartet[0] as u32) << 18)
                | ((quartet[1] as u32) << 12)
                | ((quartet[2] as u32) << 6)
                | (quartet[3] as u32);
            output.push(((chunk >> 16) & 0xff) as u8);
            output.push(((chunk >> 8) & 0xff) as u8);
            output.push((chunk & 0xff) as u8);
            quartet_len = 0;
        }
    }
    match quartet_len {
        0 => {}
        2 => {
            if quartet[1] & 0x0f != 0 {
                bail!("invalid base64url tail");
            }
            let chunk = ((quartet[0] as u32) << 18) | ((quartet[1] as u32) << 12);
            output.push(((chunk >> 16) & 0xff) as u8);
        }
        3 => {
            if quartet[2] & 0x03 != 0 {
                bail!("invalid base64url tail");
            }
            let chunk = ((quartet[0] as u32) << 18)
                | ((quartet[1] as u32) << 12)
                | ((quartet[2] as u32) << 6);
            output.push(((chunk >> 16) & 0xff) as u8);
            output.push(((chunk >> 8) & 0xff) as u8);
        }
        _ => bail!("invalid base64url length"),
    }
    Ok(output)
}

fn extract_ed25519_public_key_v1(der: &[u8]) -> Result<[u8; 32]> {
    if der.len() != ED25519_SPKI_PREFIX.len() + 32 {
        bail!("unexpected Ed25519 SPKI DER length {}", der.len());
    }
    if !der.starts_with(ED25519_SPKI_PREFIX) {
        bail!("unexpected Ed25519 SPKI DER prefix");
    }
    let mut raw = [0u8; 32];
    raw.copy_from_slice(&der[ED25519_SPKI_PREFIX.len()..]);
    Ok(raw)
}

fn unique_temp_path_v1(stem: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "substrate-lifecycle-linux-{}-{}-{}",
        stem,
        std::process::id(),
        unix_now_ns_v1().unwrap_or(0)
    ))
}

fn lower_hex_v1(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn deterministic_uuid_v7_for_request_v1(label: &str, seed: &str) -> Result<String> {
    let mut hex = sha256_hex_bytes_v1(format!("{label}\0{seed}").as_bytes())?;
    hex.truncate(32);
    let mut bytes = hex.into_bytes();
    bytes[12] = b'7';
    bytes[16] = b'a';
    let hex = String::from_utf8(bytes).context("deterministic uuid digest was not UTF-8")?;
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    ))
}

fn unix_now_ns_v1() -> Result<u64> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("system time before UNIX_EPOCH")?;
    Ok(now.as_secs() * 1_000_000_000 + u64::from(now.subsec_nanos()))
}

fn set_mode_v1(path: &Path, mode: u32) -> Result<()> {
    let metadata = fs::metadata(path).with_context(|| format!("metadata {}", path.display()))?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(mode);
    fs::set_permissions(path, permissions).with_context(|| format!("chmod {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn open_seqpacket_socketpair_v1() -> Result<(OwnedFd, OwnedFd)> {
        let mut fds = [-1_i32; 2];
        // SAFETY: socketpair is called with fixed AF_UNIX/SOCK_SEQPACKET flags and a
        // two-element output array.
        let rc = unsafe {
            socketpair(
                AF_UNIX_V1,
                SOCK_SEQPACKET_V1 | SOCK_CLOEXEC_V1,
                0,
                fds.as_mut_ptr(),
            )
        };
        if rc != 0 {
            return Err(std::io::Error::last_os_error())
                .context("open test SOCK_SEQPACKET socketpair");
        }
        // SAFETY: both descriptors are freshly returned by socketpair.
        let left = unsafe { OwnedFd::from_raw_fd(fds[0]) };
        // SAFETY: both descriptors are freshly returned by socketpair.
        let right = unsafe { OwnedFd::from_raw_fd(fds[1]) };
        Ok((left, right))
    }

    #[test]
    fn fd_frame_bridge_preserves_large_seqpacket_round_trip_v1() {
        let request_bytes = vec![b'q'; 128 * 1024];
        let response_value = serde_json::json!({
            "response": "s".repeat(96 * 1024),
        });
        let expected_request = request_bytes.clone();
        let expected_response = serde_json::to_vec(&response_value).unwrap();

        let (left, right) = open_seqpacket_socketpair_v1().unwrap();
        let sender_socket = UnixDatagram::from(left);
        let worker = thread::spawn(move || {
            send_seqpacket_frame_v1(&sender_socket, &request_bytes).unwrap();
            read_seqpacket_frame_v1(&sender_socket).unwrap()
        });

        let received_request = read_frame_from_fd_v1(right.as_raw_fd()).unwrap();
        assert_eq!(received_request, expected_request);

        write_json_frame_to_fd_v1(right.as_raw_fd(), &response_value).unwrap();
        let received_response = worker.join().unwrap();
        assert_eq!(received_response, expected_response);
    }

    #[test]
    fn fd_frame_bridge_rejects_additional_seqpacket_frame_v1() {
        let (left, right) = open_seqpacket_socketpair_v1().unwrap();
        let sender_socket = UnixDatagram::from(left);
        let worker = thread::spawn(move || {
            sender_socket.send(b"first").unwrap();
            sender_socket.send(b"second").unwrap();
            shutdown_seqpacket_write_v1(&sender_socket).unwrap();
        });

        let error = read_frame_from_fd_v1(right.as_raw_fd()).unwrap_err();
        worker.join().unwrap();
        assert!(error.to_string().contains("more than one frame"));
    }
}
