//! PRIVATE, UNCOMPILED identity/help-only preview. Requires a separate native safety release.
//! No installed authority, actual Host placement or commercial-provider claim.
use super::*;
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::{
    fs::{self, File, OpenOptions},
    io::Read,
    os::{
        fd::AsFd,
        unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    },
    path::Path,
};

fn probe_sandbox(manifest: &Value) -> Result<()> {
    ensure!(std::env::var("TASK11_NATIVE_PROBE_RELEASE")? == "identity-help-only");
    ensure!(unsafe { libc::getuid() } != 0);
    ensure!(unsafe { libc::getuid() } == unsafe { libc::geteuid() });
    ensure!(u64::from(unsafe { libc::getuid() }) == manifest["uid"].as_u64().context("uid")?);
    ensure!(
        fs::metadata("/proc/self/ns/net")?.ino()
            != manifest["host_netns_inode"].as_u64().context("netns")?
    );
    ensure!(
        fs::metadata("/proc/self/ns/pid")?.ino()
            != manifest["host_pidns_inode"].as_u64().context("pidns")?
    );
    let interfaces = fs::read_to_string("/proc/net/dev")?;
    let names: Vec<_> = interfaces
        .lines()
        .filter_map(|line| line.split_once(':').map(|(name, _)| name.trim()))
        .collect();
    ensure!(names == ["lo"], "non-loopback interface");
    ensure!(fs::read_to_string("/proc/self/status")?
        .lines()
        .any(|line| line == "CapEff:\t0000000000000000"));
    for path in [
        "/home/spenser",
        "/root",
        "/run",
        "/sys",
        "/etc/shadow",
        "/etc/gshadow",
    ] {
        ensure!(!Path::new(path).exists(), "host private path visible");
    }
    ensure!(fs::read_dir("/home/probe")?.next().is_none());
    let mut public_metadata: Vec<_> = fs::read_dir("/etc")?
        .map(|entry| entry.map(|value| value.file_name()))
        .collect::<std::io::Result<_>>()?;
    public_metadata.sort();
    ensure!(
        public_metadata
            == [
                std::ffi::OsString::from("group"),
                std::ffi::OsString::from("passwd")
            ],
        "unexpected host configuration binding"
    );
    for name in ["passwd", "group"] {
        probe_native_identity(
            &Path::new("/etc").join(name),
            &manifest["public_metadata"][name],
            false,
        )?;
    }
    for descriptor in fs::read_dir("/proc/self/fd")? {
        let descriptor = descriptor?;
        if let Ok(target) = fs::read_link(descriptor.path()) {
            let target = target.to_string_lossy();
            ensure!(
                !["net:[", "pid:[", "mnt:[", "user:[", "ipc:[", "uts:[", "cgroup:["]
                    .iter()
                    .any(|prefix| target.starts_with(prefix)),
                "namespace descriptor inherited"
            );
        }
        if let Ok(metadata) = fs::metadata(descriptor.path()) {
            ensure!(
                Some(metadata.dev()) != manifest["native_identity"]["device"].as_u64()
                    || Some(metadata.ino()) != manifest["native_identity"]["inode"].as_u64(),
                "raw producer descriptor inherited by harness"
            );
        }
    }
    Ok(())
}

fn probe_native_identity(path: &Path, expected: &Value, executable: bool) -> Result<Value> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    let before = file.metadata()?;
    ensure!(before.is_file() && (!executable || before.mode() & 0o111 != 0));
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    let after = file.metadata()?;
    let named = fs::symlink_metadata(path)?;
    ensure!(
        (
            before.dev(),
            before.ino(),
            before.len(),
            before.uid(),
            before.gid(),
            before.mode(),
            before.nlink(),
            before.ctime(),
            before.ctime_nsec(),
            before.mtime(),
            before.mtime_nsec()
        ) == (
            after.dev(),
            after.ino(),
            after.len(),
            after.uid(),
            after.gid(),
            after.mode(),
            after.nlink(),
            after.ctime(),
            after.ctime_nsec(),
            after.mtime(),
            after.mtime_nsec()
        )
    );
    ensure!(named.is_file() && (named.dev(), named.ino()) == (after.dev(), after.ino()));
    let mountinfo = fs::read_to_string("/proc/self/mountinfo")?;
    ensure!(
        mountinfo.lines().any(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            fields.get(4).is_some_and(|mount| Path::new(mount) == path)
                && fields
                    .get(5)
                    .is_some_and(|options| options.split(',').any(|option| option == "ro"))
        }),
        "held file has no observed read-only bind mount"
    );
    let actual = json!({"device": after.dev(), "inode": after.ino(), "bytes": after.len(), "sha256": format!("{:x}", hash.finalize())});
    ensure!(&actual == expected, "held bind identity differs");
    Ok(actual)
}

fn probe_configured_source(
    root: &Path,
    family: &str,
    binary: &Path,
) -> Result<(
    Arc<config_projection::OrdinaryConfiguredAcceptedHomeAuthorityV1>,
    RuntimeSelectionDescriptor,
    transport_api_types::ResolvedAgentSelectionV1,
)> {
    use crate::execution::agent_runtime::host_session_authority::HostSessionAuthority;
    use config_projection::{
        ConfigProjectionCodecV1 as Codec, InstalledAcceptedHomeBootstrapHeadV1 as Head,
        InstalledAcceptedHomeBootstrapRecordV1 as Record, Timestamp,
    };
    use transport_api_types::{
        InstallBootstrapContextCarrierV1, InstallBootstrapContextV1, PlatformPrincipalV1,
    };
    let home = root.join("accepted");
    let work = root.join("workspace");
    fs::create_dir_all(home.join("agents"))?;
    fs::create_dir(&work)?;
    for path in [&home, &home.join("agents"), &work] {
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    let (principal, _) = crate::execution::install_bootstrap::current_unix_principal_and_home()?;
    let PlatformPrincipalV1::Unix { account, uid } = principal else {
        anyhow::bail!("Unix principal required")
    };
    let carrier = InstallBootstrapContextCarrierV1::from_context(
        InstallBootstrapContextV1::new_unix(home.to_str().context("home")?, &account, uid)?,
    )?;
    let mut record = Record {
        schema_version: 1,
        install_bootstrap_carrier: carrier.encode()?,
        host_context_commitment: carrier.host_context_commitment.clone(),
        intended_account: account,
        intended_uid: u64::from(uid),
        intended_gid: u64::from(unsafe { libc::getegid() }),
        accepted_home: config_projection::CanonicalDirectoryV1::capture_linux_from_fd(
            File::open(&home)?.as_fd(),
        )?,
        installed_at: Timestamp("2026-10-31T00:00:00.000000Z".into()),
        record_hash: String::new(),
    };
    let mut value = serde_json::to_value(&record)?;
    value
        .as_object_mut()
        .context("record")?
        .remove("record_hash");
    record.record_hash = Codec::domain_sha256(
        "substrate.e3.installed-accepted-home-bootstrap.v1",
        &json!({"record": value}),
    )?;
    let mut head = Head {
        schema_version: 1,
        head_record_hash: record.record_hash.clone(),
        predecessor_head_hash: None,
        updated_at: record.installed_at.clone(),
        head_hash: String::new(),
    };
    let mut value = serde_json::to_value(&head)?;
    value.as_object_mut().context("head")?.remove("head_hash");
    head.head_hash = Codec::domain_sha256(
        "substrate.e3.installed-accepted-home-bootstrap-head.v1",
        &json!({"head":value}),
    )?;
    let source = root.join("configured-source");
    fs::create_dir_all(source.join("records"))?;
    for path in [&source, &source.join("records")] {
        fs::set_permissions(path, fs::Permissions::from_mode(0o750))?;
    }
    for (path, bytes, mode) in [
        (source.join("lock"), Vec::new(), 0o600),
        (
            source.join("active.json"),
            Codec::encode_canonical_json(&head)?,
            0o640,
        ),
        (
            source
                .join("records")
                .join(format!("{}.json", record.record_hash)),
            Codec::encode_canonical_json(&record)?,
            0o640,
        ),
    ] {
        fs::write(&path, bytes)?;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    }
    let raw = if family == "codex" {
        include_str!("../../../../../config/agents/codex.yaml")
    } else {
        include_str!("../../../../../config/agents/claude_code.yaml")
    };
    let mut inventory: serde_yaml::Value = serde_yaml::from_str(raw)?;
    inventory["config"]["placements"]["host"]["cli"]["binary"] =
        binary.display().to_string().into();
    inventory["config"]["placements"]["world"]["enabled"] = false.into();
    // Preserve the selected model tuple from the genuine repository source. The fixture is inert.
    let agent = inventory["id"].as_str().context("agent id")?.to_string();
    let inventory_relative = format!("agents/{agent}.yaml");
    let config =
        format!("agents:\n  enabled: true\n  hub:\n    orchestrator_agent_id: {agent}-host\n");
    for (path, bytes) in [
        (home.join("config.yaml"), config.into_bytes()),
        (
            home.join(&inventory_relative),
            serde_yaml::to_string(&inventory)?.into_bytes(),
        ),
    ] {
        fs::write(&path, bytes)?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    let configured = Arc::new(config_projection::OrdinaryConfiguredAcceptedHomeAuthorityV1::from_component_source_for_test(&source, &carrier)?);
    configured.revalidate()?;
    // This explicit-home reader is physically joined to the checked component source.
    // No HSA state/bootstrap/current-run/Host Acquire is fabricated or invoked.
    let authority = HostSessionAuthority::open(&home)?;
    let bootstrap = authority.bootstrap_home();
    ensure!(bootstrap.identity()?.physical_path == configured.accepted_home().physical_path);
    let config_projection::DirectoryPhysicalIdentityV1::Linux { device_id, inode } =
        &configured.accepted_home().physical_identity;
    ensure!(bootstrap.identity()?.physical_identity == crate::execution::agent_runtime::host_session_authority::schema::DirectoryPhysicalIdentityV1::Linux { device_id:*device_id, inode:*inode });
    let config = crate::execution::config_model::resolve_effective_config_for_bootstrap_home(
        &work,
        &Default::default(),
        &bootstrap,
    )?;
    let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
        &work, &bootstrap,
    )?;
    let inventory =
        crate::execution::agent_inventory::load_agent_inventory_snapshot_for_bootstrap_home_v3(
            &work, &policy, &bootstrap,
        )?;
    let selected = inventory.resolve_selection(
        &config,
        Some(&agent),
        None,
        crate::execution::config_model::AgentExecutionScope::Host,
    )?;
    let entry = inventory
        .entries
        .get(&selected.realized_agent_id)
        .context("selected inventory entry")?;
    let descriptor =
        crate::execution::agent_runtime::validator::validate_runtime_realizability(entry, &config)
            .map_err(|_| anyhow::anyhow!("configured native descriptor refused"))?;
    ensure!(descriptor.binary_path == binary);
    ensure!(selected.inventory_source.relative_path == inventory_relative);
    ensure!(
        selected.inventory_source.sha256
            == format!(
                "{:x}",
                Sha256::digest(fs::read(home.join(&inventory_relative))?)
            )
    );
    configured.revalidate()?;
    Ok((configured, descriptor, selected))
}

#[tokio::test]
#[ignore = "requires exact owner-released hermetic native identity/help invocation"]
async fn test_actual_test_configured_native_identity_help() -> Result<()> {
    let manifest: Value = serde_json::from_slice(&fs::read("/probe/manifest.json")?)?;
    probe_sandbox(&manifest)?;
    let root = Path::new("/work/native-caller-proof");
    fs::create_dir(root)?;
    fs::set_permissions(root, fs::Permissions::from_mode(0o700))?;
    let family = manifest["family"].as_str().context("family")?;
    ensure!(["codex", "claude_code"].contains(&family));
    let binary = PathBuf::from(manifest["sandbox_binary"].as_str().context("binary")?);
    let identity = probe_native_identity(&binary, &manifest["native_identity"], true)?;
    let (configured, descriptor, selected) = probe_configured_source(root, family, &binary)?;
    let home = root.join("native-home");
    fs::create_dir(&home)?;
    fs::set_permissions(&home, fs::Permissions::from_mode(0o700))?;
    let work = root.join("workspace");
    configured.revalidate()?;
    probe_native_identity(&descriptor.binary_path, &identity, true)?;
    // Only the existing SDK's real preflight APIs appear in this first preview.
    // There is no ExecStreamRequest, ClaudePrintRequest, inference server, prompt,
    // model invocation, route capability, config-header hypothesis or provider client.
    let (observation, checked) = if family == "codex" {
        let client = codex::CodexClient::builder()
            .binary(&binary)
            .working_dir(&work)
            .codex_home(&home)
            .create_home_dirs(false)
            .timeout(Duration::from_secs(5))
            .bypass_capability_cache(true)
            .build();
        let custody = codex::ExecProcessCustody::new();
        let environment = BTreeMap::from([(
            "TASK11_PRIVATE_NATIVE_PROBE".into(),
            "identity-help-only".into(),
        )]);
        let observation = tokio::time::timeout(
            Duration::from_secs(25),
            client.probe_capabilities_with_env_overrides_custody(&environment, &custody),
        )
        .await;
        if observation.is_err() {
            custody.request_termination();
        }
        let checked = matches!(
            tokio::time::timeout(Duration::from_secs(5), custody.settle()).await,
            Ok(Ok(()))
        );
        let fact = match observation {
            Ok(value) => {
                json!({"probe_completed":true,"version_reported":value.version.is_some(),"snapshot":value})
            }
            Err(_) => json!({"probe_completed":false,"method_deadline_exceeded":true}),
        };
        (fact, checked)
    } else {
        let client = claude_code::ClaudeClient::builder()
            .binary(&binary)
            .working_dir(&work)
            .claude_home(&home)
            .create_home_dirs(false)
            .env("CLAUDE_CONFIG_DIR", home.display().to_string())
            .timeout(Some(Duration::from_secs(5)))
            .mirror_stdout(false)
            .mirror_stderr(false)
            .build();
        let custody = claude_code::ClaudeProcessCustody::new();
        let output =
            tokio::time::timeout(Duration::from_secs(10), client.help_with_custody(&custody)).await;
        if output.is_err() {
            custody.request_termination();
        }
        let checked = matches!(
            tokio::time::timeout(Duration::from_secs(5), custody.settle()).await,
            Ok(Ok(()))
        );
        let fact = match output {
            Ok(Ok(value)) => {
                json!({"probe_completed":true,"successful_exit":value.status.success(),
                "stdout_sha256":format!("{:x}",Sha256::digest(&value.stdout)),
                "stderr_sha256":format!("{:x}",Sha256::digest(&value.stderr)),
                "stdout_bytes":value.stdout.len(),"stderr_bytes":value.stderr.len(),
                "actual_native_help":String::from_utf8_lossy(&value.stdout),
                "actual_native_help_stderr":String::from_utf8_lossy(&value.stderr)})
            }
            Ok(Err(_)) => json!({"probe_completed":false,"native_error":true}),
            Err(_) => json!({"probe_completed":false,"method_deadline_exceeded":true}),
        };
        (fact, checked)
    };
    // A timed-out or failed preflight still needs checked native settlement.
    configured.revalidate()?;
    probe_native_identity(&descriptor.binary_path, &identity, true)?;
    let inventory_after = fs::read(
        root.join("accepted")
            .join(&selected.inventory_source.relative_path),
    )?;
    ensure!(selected.inventory_source.sha256 == format!("{:x}", Sha256::digest(inventory_after)));
    println!(
        "{}",
        json!({"component_only":true,"family":family,
        "original_launcher_and_artifact_provenance":manifest["artifact_provenance"],
        "genuine_test_source":selected.inventory_source,"config_sources":selected.config_sources,
        "selected_contract":selected.harness_contract_revision,"native_identity":identity,
        "sdk_actual_native_wait_and_all_stdio_checked":checked,"native_observation":observation,
        "header_semantics_proved":false,"native_start_resume_proved":false})
    );
    ensure!(checked, "actual native child/stdio settlement unproved");
    ensure!(
        observation["probe_completed"] == true,
        "native preflight incomplete"
    );
    if family == "codex" {
        ensure!(
            observation["version_reported"] == true,
            "native version preflight failed"
        );
    } else {
        ensure!(
            observation["successful_exit"] == true,
            "native help exited unsuccessfully"
        );
    }
    Ok(())
}

// PRIVATE C5 experiment: env_key is a candidate until the pinned native proves
// the exact Authorization behavior below. No production routing is enabled.
fn protocol_cap_present(bytes: &[u8], caps: &[String]) -> bool {
    use base64::Engine;
    caps.iter().any(|cap| {
        bytes.windows(cap.len()).any(|part| part == cap.as_bytes())
            || base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(cap)
                .is_ok_and(|raw| bytes.windows(raw.len()).any(|part| part == raw))
    })
}

fn protocol_scan_tree(path: &Path, caps: &[String]) -> Result<bool> {
    let metadata = fs::symlink_metadata(path)?;
    ensure!(
        !metadata.file_type().is_symlink(),
        "unscanned fixture symlink"
    );
    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            if protocol_scan_tree(&entry?.path(), caps)? {
                return Ok(true);
            }
        }
        Ok(false)
    } else {
        ensure!(metadata.is_file(), "unscanned special fixture file");
        Ok(protocol_cap_present(&fs::read(path)?, caps))
    }
}

// This oracle has no forwarding client, provider credentials or outbound path.
// Every accepted request receives the same inert text, with no tool call.
fn protocol_oracle(
    listener: std::net::TcpListener,
    expected: Arc<std::sync::Mutex<(String, usize)>>,
    expected_model: String,
    caps: Vec<String>,
    stop: Arc<std::sync::atomic::AtomicBool>,
) -> Result<Vec<Value>> {
    use std::io::Write;
    use std::sync::atomic::Ordering;
    listener.set_nonblocking(true)?;
    let mut observations = Vec::new();
    while !stop.load(Ordering::Acquire) {
        let (mut stream, peer) = match listener.accept() {
            Ok(value) => value,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(5));
                continue;
            }
            Err(error) => return Err(error.into()),
        };
        ensure!(peer.ip().is_loopback());
        stream.set_read_timeout(Some(Duration::from_secs(1)))?;
        stream.set_write_timeout(Some(Duration::from_secs(1)))?;
        let mut head = Vec::new();
        while !head.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            stream.read_exact(&mut byte)?;
            head.push(byte[0]);
        }
        let head = String::from_utf8(head)?;
        let mut lines = head.split("\r\n");
        let request = lines.next().context("request line")?;
        let mut auth = Vec::new();
        let mut body_length = None;
        let mut other_header_leak = false;
        for line in lines.filter(|line| !line.is_empty()) {
            let (name, value) = line.split_once(':').context("HTTP header")?;
            let value = value.trim();
            if name.eq_ignore_ascii_case("authorization") {
                auth.push(value.to_owned());
            } else {
                other_header_leak |= protocol_cap_present(value.as_bytes(), &caps)
                    || protocol_cap_present(name.as_bytes(), &caps);
            }
            if name.eq_ignore_ascii_case("content-length") {
                ensure!(body_length.is_none(), "duplicate body length");
                body_length = Some(value.parse::<usize>()?);
            }
            ensure!(
                !name.eq_ignore_ascii_case("transfer-encoding"),
                "unsupported fixture transfer encoding"
            );
        }
        let mut body = vec![0; body_length.unwrap_or(0)];
        stream.read_exact(&mut body)?;
        let body_leak = protocol_cap_present(&body, &caps);
        let wanted = expected
            .lock()
            .map_err(|_| anyhow::anyhow!("oracle lock"))?
            .clone();
        let correct = auth.len() == 1 && auth[0] == format!("Bearer {}", wanted.0);
        let route = request == "POST /v1/responses HTTP/1.1";
        let path_leak = protocol_cap_present(request.as_bytes(), &caps);
        let accepted = correct && route && !body_leak && !other_header_leak && !path_leak;
        let parsed: Option<Value> = serde_json::from_slice(&body).ok();
        observations.push(
            json!({"accepted":accepted,"correct_bearer":correct,"authorization_count":auth.len(),
            "response_route":route,"cap_in_body":body_leak,"cap_in_other_header":other_header_leak,"cap_in_request_line":path_leak,
            "phase":wanted.1,
            "model_matches_selected":parsed.as_ref().and_then(|value|value.get("model")).and_then(Value::as_str)==Some(expected_model.as_str()),
            "stream":parsed.as_ref().and_then(|value|value.get("stream")).and_then(Value::as_bool),
            "body_sha256":format!("{:x}",Sha256::digest(&body))}),
        );
        let (status, mime, payload) = if accepted {
            let item = json!({"id":"msg_task11_local","type":"message","role":"assistant","status":"completed",
                "content":[{"type":"output_text","text":"task11 inert local protocol","annotations":[]}]});
            let created = json!({"type":"response.created","response":{"id":"resp_task11_local","object":"response","status":"in_progress","output":[]}});
            let done = json!({"type":"response.output_item.done","output_index":0,"item":item});
            let completed = json!({"type":"response.completed","response":{"id":"resp_task11_local","object":"response","status":"completed","output":[item],"usage":{"input_tokens":1,"output_tokens":1,"total_tokens":2}}});
            ("200 OK", "text/event-stream", format!("event: response.created\ndata: {created}\n\nevent: response.output_item.done\ndata: {done}\n\nevent: response.completed\ndata: {completed}\n\n"))
        } else {
            ("401 Unauthorized", "application/json", "{\"error\":{\"message\":\"local route refused\",\"type\":\"authentication_error\"}}".to_string())
        };
        // No header, credential or request body is reflected into the response.
        write!(stream,"HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",payload.len())?;
        stream.flush()?;
    }
    Ok(observations)
}

async fn protocol_sdk_turn(
    client: &codex::CodexClient,
    selector: Option<&str>,
    cap: Option<&str>,
    caps: &[String],
) -> Value {
    use futures::StreamExt;
    use std::io::{Seek, SeekFrom};
    use std::os::fd::{AsRawFd, FromRawFd};
    let stdout_fd =
        unsafe { libc::memfd_create(c"task11-private-jsonl".as_ptr(), libc::MFD_CLOEXEC) };
    if stdout_fd < 0 {
        return json!({"capture_setup_failed":true,"sdk_actual_native_wait_and_all_stdio_checked":false});
    }
    let mut raw_stdout = unsafe { File::from_raw_fd(stdout_fd) };
    let raw_log = PathBuf::from(format!("/proc/self/fd/{}", raw_stdout.as_raw_fd()));
    let custody = codex::ExecProcessCustody::new();
    let mut env = BTreeMap::new();
    if let Some(cap) = cap {
        env.insert("TASK11_LOCAL_ROUTE_CAP".into(), cap.to_string());
    }
    // All errors remain in memory. No error line or native event is logged.
    let operation = async {
        let control = if let Some(id) = selector {
            let mut request = codex::ResumeRequest::with_id(id)
                .prompt("Reply with the fixed local fixture marker.");
            request.json_event_log = Some(raw_log.clone());
            client
                .stream_resume_with_env_overrides_custody(request, &env, &custody)
                .await
        } else {
            client
                .stream_exec_with_env_overrides_custody(
                    codex::ExecStreamRequest {
                        prompt: "Reply with the fixed local fixture marker.".into(),
                        ephemeral: false,
                        ignore_rules: false,
                        ignore_user_config: false,
                        idle_timeout: Some(Duration::from_secs(10)),
                        output_last_message: None,
                        output_schema: None,
                        json_event_log: Some(raw_log.clone()),
                    },
                    &env,
                    &custody,
                )
                .await
        };
        let control = match control {
            Ok(value) => value,
            Err(error) => {
                return json!({"started":false,"successful_exit":false,"turn_completed":false,
                "cap_in_event_or_error":protocol_cap_present(format!("{error:?}").as_bytes(),caps)})
            }
        };
        let mut events = control.events;
        let collect = async {
            let mut id = None;
            let mut completed = false;
            let mut cap_leak = false;
            let mut parse_error = false;
            let mut event_count = 0;
            let mut stream_diagnostics = Vec::new();
            while let Some(event) = events.next().await {
                event_count += 1;
                match event {
                    Ok(event) => {
                        let decoded_kind = match &event {
                            codex::ThreadEvent::ThreadStarted(_) => "ThreadStarted",
                            codex::ThreadEvent::TurnStarted(_) => "TurnStarted",
                            codex::ThreadEvent::TurnCompleted(_) => "TurnCompleted",
                            codex::ThreadEvent::TurnFailed(_) => "TurnFailed",
                            codex::ThreadEvent::ItemStarted(_) => "ItemStarted",
                            codex::ThreadEvent::ItemDelta(_) => "ItemDelta",
                            codex::ThreadEvent::ItemCompleted(_) => "ItemCompleted",
                            codex::ThreadEvent::ItemFailed(_) => "ItemFailed",
                            codex::ThreadEvent::Error(_) => "Error",
                        };
                        let decoded_item_kind = match &event {
                            codex::ThreadEvent::ItemStarted(item)
                            | codex::ThreadEvent::ItemCompleted(item) => {
                                Some(match &item.item.payload {
                                    codex::ItemPayload::AgentMessage(_) => "AgentMessage",
                                    codex::ItemPayload::Reasoning(_) => "Reasoning",
                                    codex::ItemPayload::CommandExecution(_) => "CommandExecution",
                                    codex::ItemPayload::FileChange(_) => "FileChange",
                                    codex::ItemPayload::McpToolCall(_) => "McpToolCall",
                                    codex::ItemPayload::WebSearch(_) => "WebSearch",
                                    codex::ItemPayload::TodoList(_) => "TodoList",
                                    codex::ItemPayload::Error(_) => "Error",
                                })
                            }
                            _ => None,
                        };
                        stream_diagnostics.push(json!({"index":event_count,
                            "decoded_kind":decoded_kind,"decoded_item_kind":decoded_item_kind}));
                        cap_leak |= protocol_cap_present(
                            &serde_json::to_vec(&event).unwrap_or_default(),
                            caps,
                        );
                        if let Some(value) = event.thread_id() {
                            id = Some(value.to_string());
                        }
                        completed |= matches!(event, codex::ThreadEvent::TurnCompleted(_));
                    }
                    Err(error) => {
                        parse_error = true;
                        cap_leak |= protocol_cap_present(format!("{error:?}").as_bytes(), caps);
                        // Retain only public structural error properties. No raw line,
                        // arbitrary enum value, prompt, payload or diagnostic string escapes.
                        let mut diagnostic = match &error {
                            codex::ExecStreamError::Parse { source, line } => {
                                let cause = source.to_string();
                                let cause_has_cap = protocol_cap_present(cause.as_bytes(), caps);
                                let cause_class = if cause_has_cap {
                                    "redacted_cap"
                                } else if cause.starts_with("missing field `") {
                                    "missing_field"
                                } else if cause.starts_with("unknown variant `") {
                                    "unknown_variant_input_withheld"
                                } else if cause.starts_with("invalid type:") {
                                    "invalid_type_input_withheld"
                                } else if cause.starts_with("invalid value:") {
                                    "invalid_value_input_withheld"
                                } else {
                                    "other_details_withheld"
                                };
                                let missing_field = [
                                    "type",
                                    "thread_id",
                                    "turn_id",
                                    "item_id",
                                    "item_type",
                                    "content",
                                    "delta",
                                    "text",
                                    "message",
                                    "error",
                                    "usage",
                                ]
                                .into_iter()
                                .find(|field| {
                                    !cause_has_cap
                                        && cause.starts_with(&format!("missing field `{field}`"))
                                });
                                json!({"error_variant":"Parse","line_bytes":line.len(),
                                    "line_sha256":format!("{:x}",Sha256::digest(line.as_bytes())),
                                    "serde_category":format!("{:?}",source.classify()),
                                    "serde_line":source.line(),"serde_column":source.column(),
                                    "cause_class":cause_class,"missing_field":missing_field,
                                    "cause_contains_cap":cause_has_cap})
                            }
                            codex::ExecStreamError::Normalize { message, line } => {
                                let cause_has_cap = protocol_cap_present(message.as_bytes(), caps);
                                let missing_field = ["type", "thread_id", "turn_id", "item_id"]
                                    .into_iter().find(|field| !cause_has_cap && (message==&format!("event missing `{field}`")
                                        || message.ends_with(&format!(" missing `{field}` and no prior context to infer it"))));
                                let event_kind = [
                                    "thread.started",
                                    "thread.resumed",
                                    "turn.started",
                                    "turn.completed",
                                    "turn.failed",
                                    "item.started",
                                    "item.created",
                                    "item.delta",
                                    "item.updated",
                                    "item.completed",
                                    "item.failed",
                                ]
                                .into_iter()
                                .find(|kind| {
                                    !cause_has_cap
                                        && message.starts_with(&format!("{kind} missing `"))
                                });
                                json!({"error_variant":"Normalize","line_bytes":line.len(),
                                    "line_sha256":format!("{:x}",Sha256::digest(line.as_bytes())),
                                    "cause_class":if cause_has_cap {"redacted_cap"} else if missing_field.is_some() {"missing_context_or_type"} else {"other_details_withheld"},
                                    "missing_field":missing_field,"known_event_kind":event_kind,
                                    "cause_contains_cap":cause_has_cap})
                            }
                            codex::ExecStreamError::IdleTimeout { idle_for } => {
                                json!({"error_variant":"IdleTimeout","idle_millis":idle_for.as_millis()})
                            }
                            codex::ExecStreamError::ChannelClosed => {
                                json!({"error_variant":"ChannelClosed"})
                            }
                            codex::ExecStreamError::Codex(_) => {
                                json!({"error_variant":"Codex","cause_class":"nested_backend_details_withheld"})
                            }
                        };
                        diagnostic["index"] = event_count.into();
                        stream_diagnostics.push(diagnostic);
                    }
                }
            }
            (
                id,
                completed,
                cap_leak,
                parse_error,
                event_count,
                stream_diagnostics,
            )
        };
        let (events, completion) = tokio::join!(collect, control.completion);
        let (id, completed, mut cap_leak, parse_error, event_count, stream_diagnostics) = events;
        let success = match completion {
            Ok(value) => {
                if let Some(last) = value.last_message {
                    cap_leak |= protocol_cap_present(last.as_bytes(), caps);
                }
                value.status.success()
            }
            Err(error) => {
                cap_leak |= protocol_cap_present(format!("{error:?}").as_bytes(), caps);
                false
            }
        };
        if id
            .as_ref()
            .is_some_and(|id| protocol_cap_present(id.as_bytes(), caps))
        {
            cap_leak = true;
        }
        // Suppress even an anomalous ID if it contains a cap.
        json!({"started":true,"successful_exit":success,"turn_completed":completed,
            "thread_id":if cap_leak {None} else {id},"cap_in_event_or_error":cap_leak,
            "event_parse_error":parse_error,"event_count":event_count,
            "stream_diagnostics":stream_diagnostics})
    };
    let result = tokio::time::timeout(Duration::from_secs(20), operation).await;
    if result.is_err() {
        custody.request_termination();
    }
    let checked = matches!(
        tokio::time::timeout(Duration::from_secs(5), custody.settle()).await,
        Ok(Ok(()))
    );
    let mut fact = result.unwrap_or_else(
        |_| json!({"deadline_exceeded":true,"successful_exit":false,"turn_completed":false}),
    );
    fact["sdk_actual_native_wait_and_all_stdio_checked"] = checked.into();
    let mut raw = Vec::new();
    let raw_capture_checked = checked
        && raw_stdout.seek(SeekFrom::Start(0)).is_ok()
        && raw_stdout.read_to_end(&mut raw).is_ok();
    fact["raw_jsonl_capture_checked"] = raw_capture_checked.into();
    fact["cap_in_raw_jsonl"] = protocol_cap_present(&raw, caps).into();
    fact["raw_jsonl_sha256"] = format!("{:x}", Sha256::digest(&raw)).into();
    fact
}

#[tokio::test]
#[ignore = "requires separate exact owner-released hermetic C5 protocol invocation"]
async fn test_actual_configured_codex_local_cap_start_resume() -> Result<()> {
    use base64::Engine;
    use std::io::{Seek, SeekFrom};
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::sync::atomic::{AtomicBool, Ordering};
    // Keep the complete existing C1 containment gate and add an independent
    // protocol release. The C1 identity/help flag alone cannot reach this test.
    ensure!(std::env::var("TASK11_NATIVE_PROTOCOL_PROBE_RELEASE")? == "local-cap-codex-only");
    let manifest: Value = serde_json::from_slice(&fs::read("/probe/manifest.json")?)?;
    probe_sandbox(&manifest)?;
    ensure!(manifest["family"] == "codex");
    let root = Path::new("/work/native-caller-proof");
    fs::create_dir(root)?;
    fs::set_permissions(root, fs::Permissions::from_mode(0o700))?;
    let binary = PathBuf::from(manifest["sandbox_binary"].as_str().context("binary")?);
    let identity = probe_native_identity(&binary, &manifest["native_identity"], true)?;
    let (configured, descriptor, selected) = probe_configured_source(root, "codex", &binary)?;
    let home = root.join("native-home");
    fs::create_dir(&home)?;
    fs::set_permissions(&home, fs::Permissions::from_mode(0o700))?;
    // Caps are generated in the sandbox and never enter a recipe, file or argv.
    let mut random = File::open("/dev/urandom")?;
    let mut caps = Vec::new();
    for _ in 0..3 {
        let mut bytes = [0u8; 32];
        random.read_exact(&mut bytes)?;
        caps.push(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes));
    }
    ensure!(caps[0] != caps[1] && caps[1] != caps[2] && caps[0] != caps[2]);
    let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))?;
    let url = format!("http://{}/v1", listener.local_addr()?);
    let expected = Arc::new(std::sync::Mutex::new((caps[0].clone(), 0)));
    let stop = Arc::new(AtomicBool::new(false));
    let oracle_expected = Arc::clone(&expected);
    let oracle_stop = Arc::clone(&stop);
    let oracle_caps = caps.clone();
    let expected_model = selected.routed_model_id.clone();
    let oracle = std::thread::spawn(move || {
        protocol_oracle(
            listener,
            oracle_expected,
            expected_model,
            oracle_caps,
            oracle_stop,
        )
    });
    let client = codex::CodexClient::builder()
        .binary(&binary)
        .working_dir(root.join("workspace"))
        .codex_home(&home)
        .create_home_dirs(false)
        .timeout(Duration::from_secs(15))
        .mirror_stdout(false)
        .quiet(false)
        .model(&selected.routed_model_id)
        .config_override("model_provider", "\"task11_local\"")
        .config_override(
            "model_providers.task11_local.name",
            "\"Task11 inert local fixture\"",
        )
        .config_override("model_providers.task11_local.base_url", format!("{url:?}"))
        .config_override("model_providers.task11_local.wire_api", "\"responses\"")
        .config_override("model_providers.task11_local.requires_openai_auth", "false")
        .config_override("model_providers.task11_local.supports_websockets", "false")
        .config_override(
            "model_providers.task11_local.env_key",
            "\"TASK11_LOCAL_ROUTE_CAP\"",
        )
        .build();
    configured.revalidate()?;
    probe_native_identity(&descriptor.binary_path, &identity, true)?;
    // SDK quiet=false mirrors stderr. Keep that diagnostic sink anonymous and
    // scan it only after checked native settlement, including successful stderr.
    let stderr_fd =
        unsafe { libc::memfd_create(c"task11-private-diagnostics".as_ptr(), libc::MFD_CLOEXEC) };
    ensure!(stderr_fd >= 0, "diagnostic memfd failed");
    let mut diagnostics = unsafe { File::from_raw_fd(stderr_fd) };
    let saved_stderr = unsafe { libc::fcntl(libc::STDERR_FILENO, libc::F_DUPFD_CLOEXEC, 3) };
    ensure!(saved_stderr >= 0, "stderr custody failed");
    let original_stderr = unsafe { File::from_raw_fd(saved_stderr) };
    ensure!(
        unsafe { libc::dup2(diagnostics.as_raw_fd(), libc::STDERR_FILENO) } >= 0,
        "stderr capture failed"
    );
    let mut turns = Vec::new();
    let first = protocol_sdk_turn(&client, None, Some(&caps[0]), &caps).await;
    let first_ok = first["successful_exit"] == true && first["turn_completed"] == true;
    let id = first["thread_id"].as_str().map(str::to_string);
    let mut all_settled = first["sdk_actual_native_wait_and_all_stdio_checked"] == true;
    turns.push(json!({"case":"start_cap_A","result":first}));
    if first_ok && all_settled && id.is_some() {
        *expected
            .lock()
            .map_err(|_| anyhow::anyhow!("oracle lock"))? = (caps[1].clone(), 1);
        for (offset, (name, cap)) in [
            ("resume_absent", None),
            ("resume_wrong", Some(caps[2].as_str())),
            ("resume_stale_A", Some(caps[0].as_str())),
            ("resume_cap_B", Some(caps[1].as_str())),
        ]
        .into_iter()
        .enumerate()
        {
            expected
                .lock()
                .map_err(|_| anyhow::anyhow!("oracle lock"))?
                .1 = offset + 1;
            let fact = protocol_sdk_turn(&client, id.as_deref(), cap, &caps).await;
            all_settled &= fact["sdk_actual_native_wait_and_all_stdio_checked"] == true;
            turns.push(json!({"case":name,"result":fact}));
            if !all_settled {
                break;
            }
        }
    }
    stop.store(true, Ordering::Release);
    let observed = oracle
        .join()
        .map_err(|_| anyhow::anyhow!("oracle thread failed"))??;
    // Unknown child custody never restores an outer diagnostic sink or claims cleanup.
    ensure!(
        all_settled,
        "native custody unknown; diagnostics retained anonymously"
    );
    diagnostics.seek(SeekFrom::Start(0))?;
    let mut diagnostic_bytes = Vec::new();
    diagnostics.read_to_end(&mut diagnostic_bytes)?;
    let diagnostic_leak = protocol_cap_present(&diagnostic_bytes, &caps);
    let persistent_leak = protocol_scan_tree(Path::new("/work"), &caps)?
        || protocol_scan_tree(Path::new("/home/probe"), &caps)?
        || protocol_scan_tree(Path::new("/tmp"), &caps)?;
    configured.revalidate()?;
    probe_native_identity(&descriptor.binary_path, &identity, true)?;
    ensure!(
        unsafe { libc::dup2(original_stderr.as_raw_fd(), libc::STDERR_FILENO) } >= 0,
        "stderr restoration failed"
    );
    let all_events_clean = turns.iter().all(|turn| {
        turn["result"]["cap_in_event_or_error"] == false
            && turn["result"]["raw_jsonl_capture_checked"] == true
            && turn["result"]["cap_in_raw_jsonl"] == false
    });
    let accepted = observed
        .iter()
        .filter(|request| request["accepted"] == true)
        .count();
    let phase_admission = (0..5)
        .map(|phase| {
            observed
                .iter()
                .filter(|request| request["phase"] == phase && request["accepted"] == true)
                .count()
        })
        .collect::<Vec<_>>();
    let succeeded = phase_admission == [1, 0, 0, 0, 1]
        && turns.len() == 5
        && turns[0]["result"]["successful_exit"] == true
        && turns[4]["result"]["successful_exit"] == true
        && turns[4]["result"]["turn_completed"] == true
        && turns[1..4]
            .iter()
            .all(|turn| turn["result"]["successful_exit"] == false)
        && turns[4]["result"]["thread_id"].as_str() == id.as_deref();
    println!(
        "{}",
        json!({"component_only":true,"candidate_field":"model_providers.task11_local.env_key",
        "genuine_test_source":selected.inventory_source,"selected_contract":selected.harness_contract_revision,
        "native_identity":identity,"turns":turns,"request_facts":observed,"accepted_requests":accepted,"phase_admissions":phase_admission,
        "sdk_actual_native_wait_and_all_stdio_checked":all_settled,"cap_in_diagnostics":diagnostic_leak,
        "cap_in_persistent_fixture_files":persistent_leak,"cap_in_native_events_or_errors":!all_events_clean,
        "native_start_resume_observed":succeeded,"header_semantics_proved":succeeded && accepted==2 && !diagnostic_leak && !persistent_leak && all_events_clean,
        "production_host_or_upstream_strip_proved":false})
    );
    ensure!(
        succeeded && accepted == 2,
        "native start/resume or negative-cap experiment failed"
    );
    ensure!(observed
        .iter()
        .all(|request| request["cap_in_body"] == false
            && request["cap_in_other_header"] == false
            && request["cap_in_request_line"] == false
            && (request["accepted"] != true || request["model_matches_selected"] == true)));
    ensure!(
        !diagnostic_leak && !persistent_leak && all_events_clean,
        "local cap disclosure observed"
    );
    Ok(())
}
