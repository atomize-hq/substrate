use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
#[cfg(target_os = "macos")]
use std::os::fd::{AsRawFd, FromRawFd};
#[cfg(target_os = "macos")]
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
#[cfg(target_os = "macos")]
use std::process::{Child, Command, Stdio};

/// Internal-only R6 protocol discriminants. The data tag crosses the already-typed data relay;
/// the operator tag is constructed only by this hidden direct terminal path and is never serde,
/// XPC, or ordinary mapped-lifecycle input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DirectGuestPublisherPairingSessionTagV1 {
    GuestPairingDataSession,
    GuestPairingOperatorTtySession,
}

impl DirectGuestPublisherPairingSessionTagV1 {
    fn wire_name(self) -> &'static str {
        match self {
            Self::GuestPairingDataSession => "guest_pairing_data_session",
            Self::GuestPairingOperatorTtySession => "guest_pairing_operator_tty_session",
        }
    }
}
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use substrate_common::{
    canonical_guest_publisher_pairing_operator_launch_v1,
    canonical_guest_publisher_pairing_ticket_v1, lifecycle_anchor_sha256_v1,
    parse_mac_publisher_bootstrap_request_v1,
    validate_guest_publisher_pairing_operator_launch_against_ticket_at_v1,
    validate_guest_publisher_pairing_ticket_v1, validate_lifecycle_publisher_protected_state_v1,
    validate_managed_lifecycle_publisher_request_v1, GuestPublisherPairingOperatorLaunchV1,
    GuestPublisherPairingSessionBindingV1, GuestPublisherPairingTicketV1,
    LifecyclePublisherProtectedStateV1, MacPublisherBootstrapRequestV1, ManagedActionV1,
    ManagedArtifactManifestV1, ManagedLifecyclePublisherRequestV1, ManagedManifestHeadV1,
};
use substrate_shell::execution::managed_lifecycle::submit_guest_pairing_data_session_v1;
use substrate_shell::{
    derive_lifecycle_capsule_locator_v1, open_publisher_bootstrap_channel_v1, publish_manifest_v1,
    resume_action_receipt_commit_v1, validate_mapped_lifecycle_control_request_v1,
    validate_publisher_response_v1, ManagedLifecycleControlRequestV1, MappedLifecycleTagV1,
};

const BOOTSTRAP_CONFIRMATION_LITERAL_V1: &str = "CREATE EXACT SUBSTRATE LIFECYCLE PUBLISHER";
const GUEST_PAIRING_LITERAL_V1: &str = "PAIR EXACT SUBSTRATE GUEST PUBLISHER";

pub fn read_exact_bootstrap_confirmation_v1<R, W>(reader: &mut R, writer: &mut W) -> Result<()>
where
    R: BufRead,
    W: Write,
{
    writeln!(
        writer,
        "Type exactly to continue: {BOOTSTRAP_CONFIRMATION_LITERAL_V1}"
    )
    .context("write bootstrap confirmation prompt")?;
    write!(writer, "> ").context("write bootstrap confirmation prompt marker")?;
    writer
        .flush()
        .context("flush bootstrap confirmation prompt")?;

    let mut line = String::new();
    let bytes_read = reader
        .read_line(&mut line)
        .context("read bootstrap confirmation from controlling terminal")?;
    if bytes_read == 0 {
        bail!("bootstrap confirmation terminal reached EOF");
    }
    if line.trim_end_matches(['\r', '\n']) != BOOTSTRAP_CONFIRMATION_LITERAL_V1 {
        bail!("bootstrap confirmation literal mismatch");
    }
    Ok(())
}

pub fn display_guest_pairing_challenge_v1<W>(
    writer: &mut W,
    ticket: &substrate_common::GuestPublisherPairingTicketV1,
) -> Result<()>
where
    W: Write,
{
    writeln!(
        writer,
        "HOST_KEY_FINGERPRINT_SHA256 {}",
        ticket.challenge.host_key_fingerprint_sha256
    )
    .context("write guest pairing fingerprint")?;
    writeln!(writer, "CHALLENGE_ID {}", ticket.challenge.challenge_id)
        .context("write guest pairing challenge ID")?;
    writeln!(writer, "CHALLENGE {}", ticket.challenge.challenge)
        .context("write guest pairing challenge")?;
    writeln!(writer, "LITERAL {GUEST_PAIRING_LITERAL_V1}")
        .context("write guest pairing literal")?;
    writer.flush().context("flush guest pairing challenge")?;
    Ok(())
}

pub fn publisher_bootstrap_direct_interactive_v1<R, W>(
    reader: &mut R,
    writer: &mut W,
    request: MacPublisherBootstrapRequestV1,
) -> Result<Value>
where
    R: BufRead,
    W: Write,
{
    read_exact_bootstrap_confirmation_v1(reader, writer)?;
    // The closed request is untrusted derivation input, never authorization. After the literal
    // controlling-terminal confirmation, the retained direct path recomputes all authority in
    // memory and gives it exactly one FD3 delivery opportunity.
    substrate_shell::deliver_retained_publisher_bootstrap_authorization_v1(&request)
}

/// Decode the one hidden direct-bootstrap request. It is deliberately separate from the ordinary
/// mapped request decoder and contains only the exact IH carrier; no tag, manifest, source,
/// profile, action, PM, or authority field can enter this path.
fn read_exact_publisher_bootstrap_request_from_stdin_v1() -> Result<MacPublisherBootstrapRequestV1>
{
    let mut bytes = Vec::new();
    io::stdin()
        .read_to_end(&mut bytes)
        .context("read one direct bootstrap request from stdin")?;
    if bytes.is_empty() {
        bail!("direct bootstrap request is absent");
    }
    parse_mac_publisher_bootstrap_request_v1(&bytes)
        .context("decode canonical closed direct bootstrap request")
}

pub fn guest_publisher_pairing_direct_interactive_v1<W>(
    writer: &mut W,
    seed: ManagedLifecycleControlRequestV1,
) -> Result<Value>
where
    W: Write,
{
    if validate_mapped_lifecycle_control_request_v1(&seed)? != MappedLifecycleTagV1::PostPmAction {
        bail!("R6 direct pairing seed must be one admitted post-PM mapping");
    }
    let mut issued = issue_lima_guest_pairing_ticket_v1(&seed)?;
    if issued.4 == "expired_preserved" {
        // The first issue call is intentionally no-mutation evidence preservation. This second,
        // fully record-bound direct call is the distinct explicit fresh-attempt admission.
        issued = issue_lima_guest_pairing_fresh_attempt_v1(
            &seed,
            &issued.0,
            &issued.1,
            issued.1.host_record_generation,
            issued.2,
            &issued.3,
        )?;
    }
    let (ticket, binding, record_generation, record_sha256, record_state, operator_launch) = issued;
    // Pre-intent closure is preserved evidence, never a displayable/reusable pairing challenge.
    match record_state.as_str() {
        "pre_intent_closed" => {
            bail!("R6 pre-intent record is preserved; it cannot be resumed or silently adopted")
        }
        "expired_preserved" => {
            bail!("R6 explicit fresh-attempt admission did not allocate a fresh record")
        }
        _ => {}
    }
    // The retained host terminal is the only complete-value display source. The independent
    // operator child sees only the signed, canonical launch envelope and reads confirmation from
    // its own guest controlling TTY; the data child cannot begin before that child exits cleanly.
    display_guest_pairing_challenge_v1(writer, &ticket)?;
    let operator_observation = match record_state.as_str() {
        "sessions_opened" => {
            let launch = operator_launch.as_ref().ok_or_else(|| {
                anyhow!("R6 sessions_opened issue response is missing direct operator capability")
            })?;
            if let Err(error) = launch_direct_guest_pairing_operator_tty_v1(
                &seed,
                &ticket,
                &binding,
                record_generation,
                &record_sha256,
                launch,
                DirectGuestPublisherPairingSessionTagV1::GuestPairingOperatorTtySession,
            ) {
                // Persist only the fixed outcome through the data relay: never terminal bytes,
                // error text, confirmation material, or a caller-selected observation.
                let close = record_lima_guest_pairing_operator_failure_v1(
                    &seed,
                    &binding,
                    binding.host_record_generation,
                    record_generation,
                    &record_sha256,
                );
                return Err(error.context(format!(
                    "direct R6 operator child failed; durable fixed failure observation: {}",
                    close
                        .err()
                        .map(|value| format!("{value:#}"))
                        .unwrap_or_else(|| "recorded".to_string())
                )));
            }
            json!({
                "status": "direct_guest_operator_tty_succeeded",
                "session_id": launch.operator_session_id,
            })
        }
        // Once the proof has been committed by the data-session pre-intent handshake, only data
        // transport observation can reopen. It must keep the original ticket, nonce, and record.
        "operator_proof_verified"
        | "guest_state_root_durable"
        | "hello_durable"
        | "transcript_durable"
        | "ticket_consumed" => json!({"status":"operator_proof_already_durable"}),
        other => bail!(
            "R6 ticket issue response returned a non-rejoinable protected record state {other}"
        ),
    };
    let data_response = advance_lima_guest_pairing_record_v1(
        &seed,
        &ticket,
        &binding,
        binding.host_record_generation,
        record_generation,
        &record_sha256,
    )?;
    Ok(json!({
        "status": "guest_pairing_sessions_started",
        "data_session": data_response,
        "operator_tty_session": operator_observation,
    }))
}

/// Open the fixed data-session issue form. It accepts no ticket, record, key, session ID, nonce,
/// selector, or launch capability from a caller; the protected macOS owner mints all of them after
/// exact PM admission.
pub fn issue_lima_guest_pairing_ticket_v1(
    seed: &ManagedLifecycleControlRequestV1,
) -> Result<(
    GuestPublisherPairingTicketV1,
    GuestPublisherPairingSessionBindingV1,
    u64,
    String,
    String,
    Option<GuestPublisherPairingOperatorLaunchV1>,
)> {
    if DirectGuestPublisherPairingSessionTagV1::GuestPairingDataSession.wire_name()
        != "guest_pairing_data_session"
    {
        bail!("R6 direct data tag is not canonical");
    }
    let mut request = r6_pairing_request_from_admitted_seed_v1(
        seed,
        MappedLifecycleTagV1::GuestPairingDataSession,
    );
    request.pairing_ticket = None;
    let response = submit_guest_pairing_data_session_v1(&request)?;
    decode_r6_pairing_issue_response_v1(response)
}

/// Submit the one explicit fresh-attempt admission only after a first no-mutation expiry response.
/// Full prior ticket/binding/generation/digest evidence makes this distinct from ordinary issue and
/// data advance; it carries no selector, operator bytes, or mutable observation.
fn issue_lima_guest_pairing_fresh_attempt_v1(
    seed: &ManagedLifecycleControlRequestV1,
    ticket: &GuestPublisherPairingTicketV1,
    binding: &GuestPublisherPairingSessionBindingV1,
    binding_record_generation: u64,
    current_record_generation: u64,
    current_record_sha256: &str,
) -> Result<(
    GuestPublisherPairingTicketV1,
    GuestPublisherPairingSessionBindingV1,
    u64,
    String,
    String,
    Option<GuestPublisherPairingOperatorLaunchV1>,
)> {
    let mut request = r6_pairing_request_from_admitted_seed_v1(
        seed,
        MappedLifecycleTagV1::GuestPairingDataSession,
    );
    request.pairing_ticket = Some(ticket.clone());
    request.pairing_session_binding_v1 = Some(binding.clone());
    request.pairing_host_record_generation = Some(binding_record_generation);
    request.pairing_record_expected_generation_v1 = Some(current_record_generation);
    request.pairing_host_record_sha256 = Some(current_record_sha256.to_string());
    decode_r6_pairing_issue_response_v1(submit_guest_pairing_data_session_v1(&request)?)
}

/// Close only the exact direct operator child failure. This ticket-less, complete-record form is
/// a fixed outcome signal, not a data session and not an operator transport/terminal API.
fn record_lima_guest_pairing_operator_failure_v1(
    seed: &ManagedLifecycleControlRequestV1,
    binding: &GuestPublisherPairingSessionBindingV1,
    binding_record_generation: u64,
    current_record_generation: u64,
    current_record_sha256: &str,
) -> Result<()> {
    let mut request = r6_pairing_request_from_admitted_seed_v1(
        seed,
        MappedLifecycleTagV1::GuestPairingDataSession,
    );
    request.pairing_session_binding_v1 = Some(binding.clone());
    request.pairing_host_record_generation = Some(binding_record_generation);
    request.pairing_record_expected_generation_v1 = Some(current_record_generation);
    request.pairing_host_record_sha256 = Some(current_record_sha256.to_string());
    let response = submit_guest_pairing_data_session_v1(&request)?;
    if response.get("status").and_then(Value::as_str) != Some("pre_intent_closed") {
        bail!("R6 operator failure did not receive its fixed durable closure acknowledgement");
    }
    Ok(())
}

fn decode_r6_pairing_issue_response_v1(
    response: Value,
) -> Result<(
    GuestPublisherPairingTicketV1,
    GuestPublisherPairingSessionBindingV1,
    u64,
    String,
    String,
    Option<GuestPublisherPairingOperatorLaunchV1>,
)> {
    let ticket: GuestPublisherPairingTicketV1 = serde_json::from_value(
        response
            .get("ticket")
            .cloned()
            .ok_or_else(|| anyhow!("R6 ticket issue response is missing ticket"))?,
    )
    .context("decode R6 ticket issue response ticket")?;
    validate_guest_publisher_pairing_ticket_v1(&ticket)?;
    let binding: GuestPublisherPairingSessionBindingV1 = serde_json::from_value(
        response
            .get("binding")
            .cloned()
            .ok_or_else(|| anyhow!("R6 ticket issue response is missing binding"))?,
    )
    .context("decode R6 ticket issue response binding")?;
    substrate_common::validate_guest_publisher_pairing_session_binding_v1(&binding)?;
    let record_generation = response
        .get("record_generation")
        .and_then(Value::as_u64)
        .ok_or_else(|| anyhow!("R6 ticket issue response is missing record generation"))?;
    let record_sha256 = response
        .get("record_sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("R6 ticket issue response is missing record digest"))?
        .to_string();
    let record_state = response
        .get("record_state")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("R6 ticket issue response is missing record state"))?
        .to_string();
    let operator_launch = response
        .get("operator_launch")
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .context("decode R6 direct operator launch capability")?;
    if record_generation < binding.host_record_generation
        || record_sha256.len() != 64
        || !record_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        bail!("R6 ticket issue response does not carry an exact protected record binding");
    }
    match record_state.as_str() {
        "sessions_opened" => {
            let launch = operator_launch.as_ref().ok_or_else(|| {
                anyhow!("R6 ticket issue response is missing its direct operator launch")
            })?;
            validate_guest_publisher_pairing_operator_launch_against_ticket_at_v1(
                &ticket,
                launch,
                r6_now_unix_ns_v1()?,
            )
            .context("verify R6 issued operator launch against signed ticket")?;
            if launch.binding != binding
                || launch.binding.host_record_generation != binding.host_record_generation
            {
                bail!("R6 operator launch does not join immutable issued binding");
            }
        }
        "operator_proof_verified"
        | "guest_state_root_durable"
        | "hello_durable"
        | "transcript_durable"
        | "ticket_consumed"
        | "expired_preserved"
        | "pre_intent_closed" => {
            if operator_launch.is_some() {
                bail!("R6 rejoin issue response must not mint or relay an operator launch");
            }
        }
        other => bail!(
            "R6 ticket issue response returned a non-rejoinable protected record state {other}"
        ),
    }
    Ok((
        ticket,
        binding,
        record_generation,
        record_sha256,
        record_state,
        operator_launch,
    ))
}

/// Advance only the data-session branch. Its ticket never enters an operator-TTY relay.
pub fn advance_lima_guest_pairing_record_v1(
    seed: &ManagedLifecycleControlRequestV1,
    ticket: &GuestPublisherPairingTicketV1,
    binding: &GuestPublisherPairingSessionBindingV1,
    binding_record_generation: u64,
    expected_record_generation: u64,
    record_sha256: &str,
) -> Result<Value> {
    let mut request = r6_pairing_request_from_admitted_seed_v1(
        seed,
        MappedLifecycleTagV1::GuestPairingDataSession,
    );
    request.pairing_ticket = Some(ticket.clone());
    request.pairing_session_binding_v1 = Some(binding.clone());
    // Immutable initial generation and mutable current generation deliberately differ after the
    // proof-CAS and every later durable transition. Never overwrite the former with the latter.
    request.pairing_host_record_generation = Some(binding_record_generation);
    request.pairing_record_expected_generation_v1 = Some(expected_record_generation);
    request.pairing_host_record_sha256 = Some(record_sha256.to_string());
    submit_guest_pairing_data_session_v1(&request)
}

#[cfg(target_os = "macos")]
const R6_DIRECT_OPERATOR_CHILD_TIMEOUT_V1: Duration = Duration::from_secs(120);

/// Own the direct controlling-terminal guest operator child. The guard kills and reaps it on any
/// validation/timeout/Drop boundary so its guest TTY cannot outlive rejected host authority.
#[cfg(target_os = "macos")]
struct DirectOperatorChildGuardV1 {
    child: Option<Child>,
}

#[cfg(target_os = "macos")]
impl DirectOperatorChildGuardV1 {
    fn new(child: Child) -> Self {
        Self { child: Some(child) }
    }

    fn finish_successfully(&mut self) -> Result<()> {
        let child = self
            .child
            .as_mut()
            .ok_or_else(|| anyhow!("direct R6 operator child was already finalized"))?;
        let deadline = Instant::now() + R6_DIRECT_OPERATOR_CHILD_TIMEOUT_V1;
        loop {
            if let Some(status) = child.try_wait().context("poll direct R6 operator child")? {
                if status.success() {
                    self.child.take();
                    return Ok(());
                }
                bail!("direct R6 operator child exited unsuccessfully: {status}");
            }
            if Instant::now() >= deadline {
                bail!("direct R6 operator child exceeded fixed terminal deadline");
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

#[cfg(target_os = "macos")]
impl Drop for DirectOperatorChildGuardV1 {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[cfg(target_os = "macos")]
fn duplicate_direct_controlling_terminal_v1(file: &File) -> Result<File> {
    let duplicated = unsafe { libc::dup(file.as_raw_fd()) };
    if duplicated < 0 {
        return Err(std::io::Error::last_os_error()).context("duplicate retained /dev/tty");
    }
    // SAFETY: `dup` allocated one owned descriptor and this File takes exactly that ownership.
    Ok(unsafe { File::from_raw_fd(duplicated) })
}

/// Re-measure only the absolute tool image already signed into the launch. This is not a selector:
/// the capability fixed its path and digest before the direct terminal path began.
#[cfg(target_os = "macos")]
fn remeasure_direct_operator_limactl_v1(
    launch: &GuestPublisherPairingOperatorLaunchV1,
) -> Result<()> {
    let metadata = std::fs::metadata(&launch.limactl_absolute_path).with_context(|| {
        format!(
            "stat signed direct limactl path {}",
            launch.limactl_absolute_path
        )
    })?;
    if !metadata.file_type().is_file()
        || metadata.uid() != 0
        || metadata.nlink() != 1
        || metadata.mode() & 0o022 != 0
    {
        bail!("signed direct limactl image is not a root-owned immutable regular file");
    }
    let bytes = std::fs::read(&launch.limactl_absolute_path)
        .context("read signed direct limactl image for remeasurement")?;
    let measured = format!("{:x}", Sha256::digest(bytes));
    if measured != launch.limactl_sha256 {
        bail!("signed direct limactl image digest changed before terminal launch");
    }
    Ok(())
}

/// Run precisely one direct retained-limactl operator session. Its stdin/stdout/stderr are direct
/// duplicated `/dev/tty` descriptors; it never opens an XPC terminal FD, pipe, reader, writer, or
/// output projection. The sole argv envelope is canonical signed launch authority.
#[cfg(target_os = "macos")]
fn launch_direct_guest_pairing_operator_tty_v1(
    seed: &ManagedLifecycleControlRequestV1,
    ticket: &GuestPublisherPairingTicketV1,
    binding: &GuestPublisherPairingSessionBindingV1,
    current_record_generation: u64,
    current_record_sha256: &str,
    launch: &GuestPublisherPairingOperatorLaunchV1,
    session_tag: DirectGuestPublisherPairingSessionTagV1,
) -> Result<()> {
    if session_tag != DirectGuestPublisherPairingSessionTagV1::GuestPairingOperatorTtySession
        || session_tag.wire_name() != "guest_pairing_operator_tty_session"
    {
        bail!("direct R6 operator launch received a non-operator internal protocol tag");
    }
    validate_guest_publisher_pairing_operator_launch_against_ticket_at_v1(
        ticket,
        launch,
        r6_now_unix_ns_v1()?,
    )
    .context("verify signed direct operator capability")?;
    if &launch.binding != binding
        || current_record_generation < binding.host_record_generation
        || current_record_sha256.len() != 64
        || !current_record_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        bail!(
            "direct operator capability does not join immutable binding and protected generation"
        );
    }
    if launch.fixed_operator_command != "guest-pairing-operator-tty-session-v1" {
        bail!("direct operator capability selected a non-fixed guest command");
    }
    let (home, lima_root) = direct_r6_lima_environment_v1(seed)?;
    remeasure_direct_operator_limactl_v1(launch)?;
    let canonical_launch = canonical_guest_publisher_pairing_operator_launch_v1(launch)?;
    let launch_envelope = base64url_encode_control_v1(&canonical_launch);
    let terminal = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .context("open retained /dev/tty for direct R6 operator launch")?;
    // direct duplicated /dev/tty handles are intentionally given to the child, not relayed over
    // XPC. The host retains no reader/writer for guest confirmation or output.
    let stdin = duplicate_direct_controlling_terminal_v1(&terminal)?;
    let stdout = duplicate_direct_controlling_terminal_v1(&terminal)?;
    let stderr = terminal;
    let limactl_parent = Path::new(&launch.limactl_absolute_path)
        .parent()
        .ok_or_else(|| anyhow!("signed direct limactl path has no parent"))?;
    let child = Command::new(&launch.limactl_absolute_path)
        .env_clear()
        .env("HOME", &home)
        .env("LIMA_HOME", &lima_root)
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin:/usr/sbin:/sbin", limactl_parent.display()),
        )
        .arg("shell")
        .arg("--tty=true")
        .arg(&launch.admitted_instance_name)
        .arg("--")
        .arg(&launch.guest_executable_path)
        .arg(&launch.fixed_operator_command)
        .arg(format!("--operator-launch-v1={launch_envelope}"))
        .stdin(Stdio::from(stdin))
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .context("spawn fixed direct retained-limactl operator session")?;
    let mut guarded = DirectOperatorChildGuardV1::new(child);
    guarded.finish_successfully()
}

#[cfg(not(target_os = "macos"))]
fn launch_direct_guest_pairing_operator_tty_v1(
    _seed: &ManagedLifecycleControlRequestV1,
    _ticket: &GuestPublisherPairingTicketV1,
    _binding: &GuestPublisherPairingSessionBindingV1,
    _current_record_generation: u64,
    _current_record_sha256: &str,
    _launch: &GuestPublisherPairingOperatorLaunchV1,
    _session_tag: DirectGuestPublisherPairingSessionTagV1,
) -> Result<()> {
    bail!("direct R6 guest pairing operator terminal is available only on macOS")
}

/// Derive the only direct `limactl` environment from the already-admitted PM mapping. The signed
/// launch fixes the tool/instance; this closes ambient HOME/LIMA_HOME redirection to another VM.
#[cfg(target_os = "macos")]
fn direct_r6_lima_environment_v1(
    seed: &ManagedLifecycleControlRequestV1,
) -> Result<(PathBuf, PathBuf)> {
    let lima_root = PathBuf::from(
        seed.host_platform_control_root
            .as_deref()
            .ok_or_else(|| anyhow!("direct R6 pairing seed lacks PM Lima control root"))?,
    );
    if !lima_root.is_absolute()
        || lima_root.file_name().and_then(|name| name.to_str()) != Some(".lima")
    {
        bail!("direct R6 pairing PM root is not the fixed account Lima root");
    }
    let home = lima_root
        .parent()
        .filter(|path| path.is_absolute())
        .ok_or_else(|| anyhow!("direct R6 pairing Lima root has no absolute account home"))?
        .to_path_buf();
    Ok((home, lima_root))
}

fn r6_now_unix_ns_v1() -> Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("read R6 direct pairing time")?
        .as_nanos()
        .try_into()
        .map_err(|_| anyhow!("R6 direct pairing time is outside u64 nanoseconds"))
}

/// Encode the sole canonical signed operator envelope without introducing a caller-selectable
/// codec, framing, or transport dependency.
fn base64url_encode_control_v1(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut output = String::new();
    let mut index = 0;
    while index + 3 <= bytes.len() {
        let chunk = ((bytes[index] as u32) << 16)
            | ((bytes[index + 1] as u32) << 8)
            | bytes[index + 2] as u32;
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

fn r6_pairing_request_from_admitted_seed_v1(
    seed: &ManagedLifecycleControlRequestV1,
    tag: MappedLifecycleTagV1,
) -> ManagedLifecycleControlRequestV1 {
    ManagedLifecycleControlRequestV1 {
        tag: Some(tag),
        authority_domain: "mac_lima_guest".to_string(),
        scope_id: seed.scope_id.clone(),
        selected_host_prefix: seed.selected_host_prefix.clone(),
        requester_principal: seed.requester_principal.clone(),
        host_context_commitment: seed.host_context_commitment.clone(),
        platform_mapping_commitment: seed.platform_mapping_commitment.clone(),
        host_platform_control_root: seed.host_platform_control_root.clone(),
        manifest: None,
        action_receipt: None,
        publisher_protected_state: None,
        publisher_request: None,
        install_bootstrap_context_v1: seed.install_bootstrap_context_v1.clone(),
        platform_bootstrap_mapping_v1: seed.platform_bootstrap_mapping_v1.clone(),
        executor_build_evidence: seed.executor_build_evidence.clone(),
        lima_stage_one_authorization_v1: None,
        pairing_ticket: None,
        pairing_session_binding_v1: None,
        pairing_host_record_generation: None,
        pairing_record_expected_generation_v1: None,
        pairing_host_record_sha256: None,
    }
}

fn validate_manifest_binding_v1(
    request: &ManagedLifecycleControlRequestV1,
    manifest: &ManagedArtifactManifestV1,
) -> Result<()> {
    if manifest.authority_domain != request.authority_domain {
        bail!("manifest authority_domain does not match the control request");
    }
    if manifest.installation_id != request.scope_id {
        bail!("manifest installation_id does not match the control request scope_id");
    }
    if manifest.selected_host_prefix != request.selected_host_prefix {
        bail!("manifest selected_host_prefix does not match the control request");
    }
    if manifest.intended_principal != request.requester_principal {
        bail!("manifest intended_principal does not match the control request requester_principal");
    }
    if let Some(host_context_commitment) = request.host_context_commitment.as_ref() {
        if manifest.host_context_commitment != *host_context_commitment {
            bail!("manifest host_context_commitment does not match the control request");
        }
    }
    if manifest.platform_mapping_commitment != request.platform_mapping_commitment {
        bail!("manifest platform_mapping_commitment does not match the control request");
    }
    Ok(())
}

fn validate_submit_authority_domain_v1(authority_domain: &str) -> Result<()> {
    match authority_domain {
        "unix_a_local" | "windows_a_local" => Ok(()),
        "linux_system" | "mac_lima_guest" | "windows_wsl_guest" | "mac_host_shared"
        | "windows_host_shared" => bail!(
            "submit is not authorized for {authority_domain} until its dedicated platform packet lands"
        ),
        other => bail!("unknown lifecycle authority_domain {other}"),
    }
}

fn load_manifest_head_from_capsule_v1(capsule_root: &Path) -> Result<ManagedManifestHeadV1> {
    let path = capsule_root.join("head.v1.json");
    let bytes =
        std::fs::read(&path).with_context(|| format!("read manifest head {}", path.display()))?;
    serde_json::from_slice(&bytes).context("decode manifest head")
}

fn validate_publisher_request_binding_v1(
    request: &ManagedLifecycleControlRequestV1,
    manifest: &ManagedArtifactManifestV1,
    head: &ManagedManifestHeadV1,
    protected_state: &LifecyclePublisherProtectedStateV1,
    publisher_request: &ManagedLifecyclePublisherRequestV1,
) -> Result<()> {
    validate_managed_lifecycle_publisher_request_v1(publisher_request)?;
    validate_lifecycle_publisher_protected_state_v1(protected_state)?;
    if publisher_request.scope_id != request.scope_id {
        bail!("publisher_request scope_id does not match the control request");
    }
    if publisher_request.requester_principal != request.requester_principal {
        bail!("publisher_request requester_principal does not match the control request");
    }
    if publisher_request.scope_id != manifest.installation_id {
        bail!("publisher_request scope_id does not match the manifest installation_id");
    }
    if publisher_request.manifest_generation != manifest.manifest_generation {
        bail!("publisher_request manifest_generation does not match the manifest");
    }
    if publisher_request.manifest_sha256 != manifest.manifest_sha256 {
        bail!("publisher_request manifest_sha256 does not match the manifest");
    }
    if publisher_request.host_context_commitment != manifest.host_context_commitment {
        bail!("publisher_request host_context_commitment does not match the manifest");
    }
    if publisher_request.platform_mapping_commitment != manifest.platform_mapping_commitment {
        bail!("publisher_request platform_mapping_commitment does not match the manifest");
    }
    if let Some(host_context_commitment) = request.host_context_commitment.as_ref() {
        if publisher_request.host_context_commitment != *host_context_commitment {
            bail!("publisher_request host_context_commitment does not match the control request");
        }
    }
    if head.scope_id != manifest.installation_id
        || head.manifest_generation != manifest.manifest_generation
        || head.manifest_sha256 != manifest.manifest_sha256
    {
        bail!("manifest head does not match the selected manifest");
    }
    if protected_state.current_anchor.authority_domain != manifest.authority_domain {
        bail!("publisher protected state authority_domain does not match the manifest");
    }
    if protected_state.current_anchor.scope_id != manifest.installation_id {
        bail!("publisher protected state scope_id does not match the manifest");
    }
    if protected_state.current_anchor.manifest_generation != manifest.manifest_generation
        || protected_state.current_anchor.manifest_sha256 != manifest.manifest_sha256
    {
        bail!("publisher protected state manifest reference does not match the manifest");
    }
    if protected_state.current_anchor.host_context_commitment != manifest.host_context_commitment {
        bail!("publisher protected state host_context_commitment does not match the manifest");
    }
    if protected_state.current_anchor.platform_mapping_commitment
        != manifest.platform_mapping_commitment
    {
        bail!("publisher protected state platform_mapping_commitment does not match the manifest");
    }
    if publisher_request.current_anchor_counter != protected_state.counter {
        bail!("publisher_request current_anchor_counter does not match the protected state");
    }
    if publisher_request.current_anchor_sha256
        != lifecycle_anchor_sha256_v1(&protected_state.current_anchor)?
    {
        bail!("publisher_request current_anchor_sha256 does not match the protected state");
    }

    let mut matching_entries = manifest.entries.iter().filter(|entry| {
        entry.logical_role == publisher_request.role
            && entry.identity == publisher_request.object_identity
    });
    let entry = matching_entries.next().ok_or_else(|| {
        anyhow!("publisher_request object_identity is not present in the manifest")
    })?;
    if matching_entries.next().is_some() {
        bail!("publisher_request object_identity is ambiguous within the manifest");
    }

    let planned = manifest.planned_action_receipts.iter().any(|planned| {
        let Some(object) = planned.as_object() else {
            return false;
        };
        object.get("entry_id").and_then(Value::as_str) == Some(entry.object_id.as_str())
            && object.get("action").and_then(Value::as_str)
                == Some(managed_action_name_v1(publisher_request.action))
    });
    if !planned {
        bail!("publisher_request action is not present in the manifest planned_action_receipts");
    }
    Ok(())
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

pub fn submit_managed_lifecycle_request_v1(
    request: ManagedLifecycleControlRequestV1,
) -> Result<Value> {
    validate_submit_authority_domain_v1(&request.authority_domain)?;
    let capsule_root = derive_lifecycle_capsule_locator_v1(
        &request.authority_domain,
        Path::new(&request.selected_host_prefix),
        request.platform_mapping_commitment.as_deref(),
        request.host_platform_control_root.as_deref().map(Path::new),
    )?;

    let mut response = json!({
        "capsule_root": capsule_root.display().to_string(),
        "authority_domain": request.authority_domain,
        "scope_id": request.scope_id,
        "requester_principal": request.requester_principal,
    });

    if let Some(manifest) = request.manifest.as_ref() {
        validate_manifest_binding_v1(&request, manifest)?;
        let canonical = publish_manifest_v1(&capsule_root, manifest)?;
        response["manifest"] = json!({
            "generation": manifest.manifest_generation,
            "manifest_sha256": canonical.manifest_sha256,
            "byte_length": canonical.bytes.len(),
        });
    }

    if let Some(receipt) = request.action_receipt.as_ref() {
        let protected_state = request
            .publisher_protected_state
            .as_ref()
            .ok_or_else(|| anyhow!("action_receipt requires publisher_protected_state"))?;
        let manifest = if let Some(manifest) = request.manifest.as_ref() {
            validate_manifest_binding_v1(&request, manifest)?;
            manifest.clone()
        } else {
            let manifest =
                substrate_shell::load_manifest_v1(&capsule_root, receipt.manifest_generation)?;
            validate_manifest_binding_v1(&request, &manifest)?;
            manifest
        };
        let head =
            resume_action_receipt_commit_v1(&capsule_root, &manifest, protected_state, receipt)?;
        response["action_receipt_head"] =
            serde_json::to_value(head).context("serialize action receipt head")?;
        if request.manifest.is_none() {
            response["manifest"] = json!({
                "generation": manifest.manifest_generation,
                "manifest_sha256": manifest.manifest_sha256,
            });
        }
    }

    if let Some(publisher_request) = request.publisher_request.as_ref() {
        let protected_state = request
            .publisher_protected_state
            .as_ref()
            .ok_or_else(|| anyhow!("publisher_request requires publisher_protected_state"))?;
        let manifest = if let Some(manifest) = request.manifest.as_ref() {
            validate_manifest_binding_v1(&request, manifest)?;
            manifest.clone()
        } else {
            let manifest = substrate_shell::load_manifest_v1(
                &capsule_root,
                publisher_request.manifest_generation,
            )?;
            validate_manifest_binding_v1(&request, &manifest)?;
            manifest
        };
        let head = load_manifest_head_from_capsule_v1(&capsule_root)?;
        validate_publisher_request_binding_v1(
            &request,
            &manifest,
            &head,
            protected_state,
            publisher_request,
        )?;
        let client = open_publisher_bootstrap_channel_v1(&request.authority_domain)?;
        let raw_response = client.submit_publisher_request_v1(publisher_request)?;
        let response_bytes =
            serde_json::to_vec(&raw_response).context("encode publisher response")?;
        response["publisher_response"] =
            validate_publisher_response_v1(&response_bytes, publisher_request)?;
    }

    Ok(response)
}

/// Submit one of the ordinary R5 mapped-lifecycle branches. The tag decoder runs before any
/// platform client/XPC action; wrapper callers cannot select a raw operation or an R6 session.
pub fn submit_mapped_lifecycle_v1(request: ManagedLifecycleControlRequestV1) -> Result<Value> {
    match validate_mapped_lifecycle_control_request_v1(&request)? {
        MappedLifecycleTagV1::StageOneCreate => {
            substrate_shell::submit_stage_one_absent_instance_create_v1(&request)
        }
        MappedLifecycleTagV1::PostPmAction => {
            substrate_shell::submit_post_pm_managed_action_v1(&request)
        }
        MappedLifecycleTagV1::GuestPairingDataSession => {
            bail!(
                "R6 pairing protocol tags are accepted only by the hidden direct-interactive path"
            )
        }
    }
}

fn read_request_from_stdin_v1() -> Result<ManagedLifecycleControlRequestV1> {
    let mut buffer = Vec::new();
    io::stdin()
        .read_to_end(&mut buffer)
        .context("read request JSON from stdin")?;
    if buffer.is_empty() {
        bail!("stdin did not provide a lifecycle control request");
    }
    serde_json::from_slice(&buffer).context("decode ManagedLifecycleControlRequestV1")
}

/// Decode the hidden R6 direct seed.  The seed is still forced through the ordinary closed
/// post-PM validator before this path derives any pairing tag, ticket, nonce, or host record.
fn read_exact_guest_pairing_seed_from_stdin_v1() -> Result<ManagedLifecycleControlRequestV1> {
    let seed = read_request_from_stdin_v1().context("read hidden R6 direct pairing seed")?;
    if seed.tag != Some(MappedLifecycleTagV1::PostPmAction)
        || seed.pairing_ticket.is_some()
        || seed.pairing_session_binding_v1.is_some()
        || seed.pairing_host_record_generation.is_some()
        || seed.pairing_record_expected_generation_v1.is_some()
        || seed.pairing_host_record_sha256.is_some()
    {
        bail!("hidden R6 direct pairing seed must contain only an ordinary closed post-PM request");
    }
    Ok(seed)
}

fn print_json_line_v1(value: &Value) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    serde_json::to_writer(&mut handle, value).context("write JSON response")?;
    writeln!(handle).context("terminate JSON response")?;
    Ok(())
}

fn print_ticket_v1(ticket: &substrate_common::GuestPublisherPairingTicketV1) -> Result<()> {
    let canonical = canonical_guest_publisher_pairing_ticket_v1(ticket)?;
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle
        .write_all(&canonical)
        .context("write canonical pairing ticket")?;
    writeln!(handle).context("terminate canonical pairing ticket")?;
    Ok(())
}

fn open_controlling_terminal_reader_v1() -> Result<BufReader<std::fs::File>> {
    #[cfg(windows)]
    let path = "CONIN$";
    #[cfg(not(windows))]
    let path = "/dev/tty";

    let file = OpenOptions::new()
        .read(true)
        .open(path)
        .with_context(|| format!("open controlling terminal for read at {path}"))?;
    Ok(BufReader::new(file))
}

fn open_controlling_terminal_writer_v1() -> Result<std::fs::File> {
    #[cfg(windows)]
    let path = "CONOUT$";
    #[cfg(not(windows))]
    let path = "/dev/tty";

    OpenOptions::new()
        .write(true)
        .open(path)
        .with_context(|| format!("open controlling terminal for write at {path}"))
}

fn open_controlling_terminal_duplex_v1() -> Result<std::fs::File> {
    #[cfg(windows)]
    let path = "CONOUT$";
    #[cfg(not(windows))]
    let path = "/dev/tty";

    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .with_context(|| format!("open retained controlling terminal at {path}"))
}

fn usage_error_v1() -> Result<()> {
    bail!("usage: substrate-lifecycle-control <submit-mapped-lifecycle-v1|publisher-bootstrap>")
}

fn main_impl_v1() -> Result<()> {
    let mut args = std::env::args();
    let _program = args.next();
    let Some(command) = args.next() else {
        return usage_error_v1();
    };
    if args.next().is_some() {
        return usage_error_v1();
    }

    match command.as_str() {
        "publisher-bootstrap" => {
            // This branch is deliberately before ordinary mapped stdin handling. It admits one
            // canonical *seed* only; the complete bootstrap authorization remains
            // nonserialized and only travels in one FD3 seqpacket frame.
            let request = read_exact_publisher_bootstrap_request_from_stdin_v1()?;
            let mut tty_reader = open_controlling_terminal_reader_v1()?;
            let mut tty_writer = open_controlling_terminal_writer_v1()?;
            let response = publisher_bootstrap_direct_interactive_v1(
                &mut tty_reader,
                &mut tty_writer,
                request,
            )?;
            print_json_line_v1(&response)?;
        }
        "guest-publisher-pairing-direct-interactive-v1" => {
            // The hidden R6 operator path is a macOS-only PM/Lima control surface. Other hosts
            // fail before opening a terminal, socket, or provider state.
            let seed = read_exact_guest_pairing_seed_from_stdin_v1()?;
            #[cfg(target_os = "macos")]
            {
                let mut tty = open_controlling_terminal_duplex_v1()?;
                let response = guest_publisher_pairing_direct_interactive_v1(&mut tty, seed)?;
                print_json_line_v1(&response)?;
            }
            #[cfg(not(target_os = "macos"))]
            {
                let _ = seed;
                bail!(
                    "direct R6 guest pairing is available only on macOS before terminal admission"
                );
            }
        }
        "submit-mapped-lifecycle-v1" => {
            let request = read_request_from_stdin_v1()?;
            let response = submit_mapped_lifecycle_v1(request)?;
            print_json_line_v1(&response)?;
        }
        _ => return usage_error_v1(),
    }
    Ok(())
}

fn main() -> Result<()> {
    let _ = env_logger::try_init();
    main_impl_v1()
}
