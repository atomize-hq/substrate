use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Value};
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;
use substrate_common::{
    canonical_guest_publisher_pairing_ticket_v1, lifecycle_anchor_sha256_v1,
    validate_guest_publisher_pairing_ticket_v1, validate_lifecycle_publisher_protected_state_v1,
    validate_managed_lifecycle_publisher_request_v1, LifecyclePublisherProtectedStateV1,
    ManagedActionV1, ManagedArtifactManifestV1, ManagedLifecyclePublisherRequestV1,
    ManagedManifestHeadV1,
};
use substrate_shell::{
    derive_lifecycle_capsule_locator_v1, open_publisher_bootstrap_channel_v1, publish_manifest_v1,
    resume_action_receipt_commit_v1, validate_publisher_response_v1,
    ManagedLifecycleControlRequestV1,
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
    request: &ManagedLifecycleControlRequestV1,
) -> Result<Value>
where
    R: BufRead,
    W: Write,
{
    let authorization = substrate_shell::issue_publisher_bootstrap_authorization_v1(request)?;
    read_exact_bootstrap_confirmation_v1(reader, writer)?;
    let client = open_publisher_bootstrap_channel_v1(&authorization.authority_domain)?;
    client.bootstrap_publisher_v1(&authorization)
}

pub fn guest_publisher_pairing_direct_interactive_v1<W>(
    writer: &mut W,
    request: &ManagedLifecycleControlRequestV1,
) -> Result<substrate_common::GuestPublisherPairingTicketV1>
where
    W: Write,
{
    let ticket = if let Some(ticket) = request.pairing_ticket.as_ref() {
        validate_guest_publisher_pairing_ticket_v1(ticket)?;
        ticket.clone()
    } else {
        let publisher_request = request
            .publisher_request
            .as_ref()
            .ok_or_else(|| anyhow!("publisher_request or pairing_ticket is required"))?;
        let client = open_publisher_bootstrap_channel_v1(&request.authority_domain)?;
        client.issue_guest_publisher_pairing_ticket_v1(publisher_request)?
    };
    display_guest_pairing_challenge_v1(writer, &ticket)?;
    Ok(ticket)
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

fn usage_error_v1() -> Result<()> {
    bail!("usage: substrate-lifecycle-control <submit|publisher-bootstrap|guest-publisher-pairing>")
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

    let request = read_request_from_stdin_v1()?;
    match command.as_str() {
        "submit" => {
            let response = submit_managed_lifecycle_request_v1(request)?;
            print_json_line_v1(&response)?;
        }
        "publisher-bootstrap" => {
            let mut tty_reader = open_controlling_terminal_reader_v1()?;
            let mut tty_writer = open_controlling_terminal_writer_v1()?;
            let response = publisher_bootstrap_direct_interactive_v1(
                &mut tty_reader,
                &mut tty_writer,
                &request,
            )?;
            print_json_line_v1(&response)?;
        }
        "guest-publisher-pairing" => {
            let mut tty_writer = open_controlling_terminal_writer_v1()?;
            let ticket = guest_publisher_pairing_direct_interactive_v1(&mut tty_writer, &request)?;
            print_ticket_v1(&ticket)?;
        }
        _ => return usage_error_v1(),
    }
    Ok(())
}

fn main() -> Result<()> {
    let _ = env_logger::try_init();
    main_impl_v1()
}
