#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("substrate R3 macOS evidence coordinator is available only on macOS");
    std::process::exit(78);
}

#[cfg(target_os = "macos")]
fn main() -> anyhow::Result<()> {
    macos::run()
}

#[cfg(target_os = "macos")]
mod macos {
    use std::fs::OpenOptions;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::os::fd::{AsFd, AsRawFd, OwnedFd};
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    use std::os::unix::net::UnixStream;
    use std::path::Path;
    use std::process::{Child, Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    use anyhow::{bail, Context, Result};
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use substrate_common::macos_retirement_v2::{
        parse_canonical_v2, validate_finalization_request_v2, validate_finalizer_response_v2,
        FinalizationRequestV2, FinalizerResponseStateV2, FinalizerResponseV2, TargetSetKindV2,
        MAC_R3_FINALIZER_ENDPOINT_V2, MAC_R3_FINALIZER_REQUEST_PATH_V2,
    };
    use substrate_r3_macos_finalizer::contract::{TerminalBindingRequest, TERMINAL_BINDING_PATH};
    use substrate_r3_macos_finalizer::darwin::{
        normalize_accepted_socket_to_fd3, SecurityUiDenied,
    };
    use substrate_r3_macos_finalizer::experiment::controls::{
        build_transport_control_probe_v2, expected_safe_rejection_request_digest_v2,
        sent_material_sha256_v2, ControlDeadlineClassificationV2, ControlFramingV2,
        DisposableNativeArmReceiptV2, DisposableNativeArmV2, PeerControlIdentityPacketV2,
        PeerControlSetReceiptV2, SecurityAgentArmEvidenceV2, SecurityAgentRawReportEvidenceV2,
        TransportControlReceiptV2, TransportControlSetReceiptV2, TransportControlsCompleteMarkerV2,
        TransportControlsObservationReceiptV2, FINALIZER_PROTOCOL_DEADLINE_MS_V2,
        TRANSPORT_CONTROL_RECEIPT_OWNER_V2, TRANSPORT_CONTROL_SEQUENCE_V2,
    };
    use substrate_r3_macos_finalizer::experiment::durable::ExperimentStoreV2;
    use substrate_r3_macos_finalizer::experiment::peer_probe::{
        exchange_one_peer_probe_v2, require_no_peer_probe_ambient_input,
        stop_before_peer_probe_exchange_v2,
    };
    use substrate_r3_macos_finalizer::experiment::process::attest_fixed_peer_process_v2;
    use substrate_r3_macos_finalizer::experiment::publisher_protocol::{
        CandidateIdentityPacketV2, PublisherArtifactV2,
    };
    use substrate_r3_macos_finalizer::experiment::{
        RepetitionV2, ALTERNATE_COORDINATOR_PATH_V2, CANDIDATE_IDENTITY_PACKET_PATH_V2,
        DISPOSABLE_HARNESS_ACCOUNT_V2, DISPOSABLE_HARNESS_GID_V2, DISPOSABLE_HARNESS_UID_V2,
        EXPERIMENT_ID_V2, EXPERIMENT_VERSION_V2, PEER_CONTROL_IDENTITY_PACKET_PATH_V2,
        SECURITYAGENT_OBSERVER_PATH_V2,
    };
    use substrate_r3_macos_finalizer::fixed_inbox::read_fixed_coordinator_inbox_v2;
    use substrate_r3_macos_finalizer::frame::{read_one_frame_to_eof, write_one_frame};

    const SOCKET_TIMEOUT: Duration = Duration::from_secs(30);
    // The exact process identity is signed into the receipt. Keep that one surviving coordinator
    // available across bounded manual publisher handoff and UID-501 harness restart windows.
    const TERMINAL_DELIVERY_TIMEOUT: Duration = Duration::from_secs(3_600);
    const OBSERVER_REARM_INTERVAL: Duration = Duration::from_secs(3);
    const OBSERVER_INITIAL_READY_TIMEOUT: Duration = Duration::from_secs(10);
    const OBSERVER_REARM_READY_TIMEOUT: Duration = Duration::from_secs(1);

    pub fn run() -> Result<()> {
        let executable_path = std::fs::canonicalize(std::env::current_exe()?)?;
        let alternate_path = executable_path == Path::new(ALTERNATE_COORDINATOR_PATH_V2);
        let root_caller = executable_path
            == Path::new(substrate_common::macos_retirement_v2::MAC_R3_COORDINATOR_PATH_V2)
            && unsafe { libc::geteuid() } == 0;
        if alternate_path || root_caller {
            require_no_peer_probe_ambient_input(
                executable_path
                    .to_str()
                    .context("peer-probe executable path is not UTF-8")?,
            )?;
            let candidate: CandidateIdentityPacketV2 =
                read_root_installed_packet(Path::new(CANDIDATE_IDENTITY_PACKET_PATH_V2))?;
            let peer_identities: PeerControlIdentityPacketV2 =
                read_root_installed_packet(Path::new(PEER_CONTROL_IDENTITY_PACKET_PATH_V2))?;
            peer_identities.validate(&candidate)?;
            let (identity, uid, gid, account) = if root_caller {
                (&peer_identities.alternate_caller_identity, 0, 0, "root")
            } else {
                (
                    &peer_identities.alternate_path_identity,
                    DISPOSABLE_HARNESS_UID_V2,
                    DISPOSABLE_HARNESS_GID_V2,
                    DISPOSABLE_HARNESS_ACCOUNT_V2,
                )
            };
            attest_fixed_peer_process_v2(
                i32::try_from(std::process::id()).context("peer coordinator PID exceeds i32")?,
                identity,
                uid,
                gid,
                account,
            )?;
            clear_process_environment()?;
            let request_bytes = wait_for_fixed_file(
                Path::new(MAC_R3_FINALIZER_REQUEST_PATH_V2),
                TERMINAL_DELIVERY_TIMEOUT,
                "sealed coordinator peer-probe request",
            )?;
            let request: FinalizationRequestV2 = parse_canonical_v2(&request_bytes)?;
            stop_before_peer_probe_exchange_v2()?;
            let result = exchange_one_peer_probe_v2(&request)?;
            write_stdout_frame(
                &canonical_bytes(&result)?,
                "alternate-path peer-probe result",
            )?;
            return Ok(());
        }
        require_no_ambient_inputs()?;
        let candidate: CandidateIdentityPacketV2 =
            read_root_installed_packet(Path::new(CANDIDATE_IDENTITY_PACKET_PATH_V2))?;
        let peer_identities: PeerControlIdentityPacketV2 =
            read_root_installed_packet(Path::new(PEER_CONTROL_IDENTITY_PACKET_PATH_V2))?;
        peer_identities.validate(&candidate)?;
        let coordinator_attestation = attest_fixed_peer_process_v2(
            i32::try_from(std::process::id()).context("coordinator PID exceeds i32")?,
            &candidate.coordinator_identity,
            DISPOSABLE_HARNESS_UID_V2,
            DISPOSABLE_HARNESS_GID_V2,
            DISPOSABLE_HARNESS_ACCOUNT_V2,
        )?;
        clear_process_environment()?;
        write_stdout_frame(
            &canonical_bytes(&coordinator_attestation)?,
            "coordinator self-measured process attestation",
        )?;
        // Keep the coordinator noninteractive too; this is its first Security.framework call.
        let security = SecurityUiDenied::establish_first()?;
        let request_bytes = wait_for_fixed_file(
            Path::new(MAC_R3_FINALIZER_REQUEST_PATH_V2),
            TERMINAL_DELIVERY_TIMEOUT,
            "initial finalization request",
        )?;
        let request: FinalizationRequestV2 = parse_canonical_v2(&request_bytes)?;
        let (receipt, _, _) = validate_finalization_request_v2(&request, None)?;
        if receipt.coordinator_process != coordinator_attestation.process_identity() {
            bail!("signed request does not exact-bind the running coordinator PID/start identity")
        }
        let repetition = repetition_for_scope(&receipt.scope_id);
        if receipt.target_set_kind == TargetSetKindV2::DisposableCapability {
            let repetition = repetition.context("disposable request scope is not frozen")?;
            let mut control_receipts = Vec::with_capacity(TRANSPORT_CONTROL_SEQUENCE_V2.len());
            for (index, control) in TRANSPORT_CONTROL_SEQUENCE_V2.into_iter().enumerate() {
                let probe = build_transport_control_probe_v2(&request, &receipt, control)?;
                let sent_material_sha256 = sent_material_sha256_v2(&probe)?;
                let expected_rejection_request_digest =
                    expected_safe_rejection_request_digest_v2(&probe);
                let framing = probe.framing;
                let ((response_bytes, response_eof), securityagent_evidence, elapsed) =
                    observe_securityagent_while(|| {
                        exchange_probe(
                            &security,
                            &receipt.finalizer_identity,
                            &probe.body,
                            probe.framing,
                            probe.second_body.as_deref(),
                        )
                    })?;
                let response: FinalizerResponseV2 = parse_canonical_v2(&response_bytes)?;
                let elapsed_ms = u64::try_from(elapsed.as_millis())
                    .context("transport control elapsed milliseconds overflow")?;
                let securityagent_observation_sha256 =
                    substrate_common::macos_retirement_v2::document_sha256_v2(
                        &securityagent_evidence,
                    )?;
                let control_receipt = TransportControlReceiptV2 {
                    schema_owner: TRANSPORT_CONTROL_RECEIPT_OWNER_V2.to_string(),
                    schema_version: EXPERIMENT_VERSION_V2,
                    experiment_id: EXPERIMENT_ID_V2.to_string(),
                    repetition: repetition.ordinal(),
                    scope_id: repetition.scope_id().to_string(),
                    sequence_ordinal: u8::try_from(index + 1)
                        .expect("fixed control sequence fits u8"),
                    control,
                    framing,
                    sent_material_sha256,
                    expected_rejection_request_digest,
                    canonical_response_sha256:
                        substrate_common::macos_retirement_v2::sha256_hex_v2(&response_bytes),
                    response,
                    response_eof_observed: response_eof,
                    finalizer_protocol_deadline_ms: FINALIZER_PROTOCOL_DEADLINE_MS_V2,
                    elapsed_ms,
                    deadline_classification: if control
                        == substrate_r3_macos_finalizer::experiment::controls::TransportControlV2::MissingEof
                    {
                        ControlDeadlineClassificationV2::ReturnedAfterFinalizerDeadline
                    } else {
                        ControlDeadlineClassificationV2::ReturnedBeforeFinalizerDeadline
                    },
                    securityagent_evidence,
                    securityagent_observation_sha256,
                };
                control_receipt.validate(repetition)?;
                write_stdout_frame(
                    &canonical_bytes(&control_receipt)?,
                    "transport control receipt",
                )?;
                control_receipts.push(control_receipt);
            }
            let control_receipt_sha256 = control_receipts
                .iter()
                .map(substrate_common::macos_retirement_v2::document_sha256_v2)
                .collect::<Result<Vec<_>>>()?;
            let store = ExperimentStoreV2::open_fixed()?;
            let peer_controls: PeerControlSetReceiptV2 = wait_for_publisher_input(
                &store,
                repetition,
                PublisherArtifactV2::PeerControlSetReceipt,
                TERMINAL_DELIVERY_TIMEOUT,
                "peer-substitution control set",
            )?;
            peer_controls.validate(repetition, &request, &peer_identities)?;
            let marker = TransportControlsCompleteMarkerV2 {
                schema_owner: TRANSPORT_CONTROL_RECEIPT_OWNER_V2.to_string(),
                schema_version: EXPERIMENT_VERSION_V2,
                experiment_id: EXPERIMENT_ID_V2.to_string(),
                repetition: repetition.ordinal(),
                scope_id: repetition.scope_id().to_string(),
                request_digest: request.request_digest.clone(),
                control_receipt_sha256: control_receipt_sha256.clone(),
                peer_control_set_sha256: substrate_common::macos_retirement_v2::document_sha256_v2(
                    &peer_controls,
                )?,
                before_observation_sha256: receipt.quiesced_observation_sha256.clone(),
            };
            marker.validate(repetition, &control_receipts)?;
            store.persist_publisher_input(
                repetition,
                PublisherArtifactV2::TransportControlsCompleteMarker,
                &marker,
            )?;
            let observation: TransportControlsObservationReceiptV2 = wait_for_publisher_output(
                &store,
                repetition,
                PublisherArtifactV2::TransportControlsObservation,
                TERMINAL_DELIVERY_TIMEOUT,
                "transport controls no-mutation observation",
            )?;
            observation.validate(repetition, &marker)?;
            let mut ui_observations = control_receipts
                .iter()
                .map(|control| control.securityagent_observation_sha256.clone())
                .collect::<Vec<_>>();
            ui_observations.push(observation.securityagent_observation_sha256.clone());
            let set_receipt = TransportControlSetReceiptV2 {
                schema_owner: TRANSPORT_CONTROL_RECEIPT_OWNER_V2.to_string(),
                schema_version: EXPERIMENT_VERSION_V2,
                experiment_id: EXPERIMENT_ID_V2.to_string(),
                repetition: repetition.ordinal(),
                scope_id: repetition.scope_id().to_string(),
                control_receipt_sha256,
                before_observation_sha256: marker.before_observation_sha256.clone(),
                after_observation_sha256: observation.after_observation_sha256,
                accepted_journal_absent: observation.accepted_journal_absent,
                securityagent_observation_set_sha256:
                    substrate_common::macos_retirement_v2::document_sha256_v2(&ui_observations)?,
            };
            set_receipt.validate(repetition, &control_receipts)?;
            write_stdout_frame(
                &canonical_bytes(&set_receipt)?,
                "transport control set receipt",
            )?;
        }
        let (response_bytes, effects_securityagent) = if let Some(repetition) = repetition {
            let ((bytes, eof), observation, _) = observe_securityagent_while(|| {
                exchange(&security, &receipt.finalizer_identity, &request_bytes)
            })?;
            if !eof {
                bail!("finalizer effects response lacked mandatory EOF")
            }
            (bytes, Some((repetition, observation)))
        } else {
            let (bytes, eof) = exchange(&security, &receipt.finalizer_identity, &request_bytes)?;
            if !eof {
                bail!("finalizer effects response lacked mandatory EOF")
            }
            (bytes, None)
        };
        let response: FinalizerResponseV2 = parse_canonical_v2(&response_bytes)?;
        validate_finalizer_response_v2(&response)?;
        if response.request_digest != request.request_digest {
            bail!("finalizer response does not bind the sent request digest")
        }
        if let Some((repetition, securityagent_evidence)) = effects_securityagent {
            let receipt = native_arm_receipt(
                repetition,
                DisposableNativeArmV2::FinalizationEffects,
                &response,
                securityagent_evidence,
            )?;
            write_stdout_frame(&canonical_bytes(&receipt)?, "effects native-arm receipt")?;
        }
        write_stdout_frame(&response_bytes, "harness effects frame")?;
        if response.state != FinalizerResponseStateV2::EffectsComplete {
            return Ok(());
        }

        let terminal_bytes = wait_for_terminal_binding()?;
        let terminal: TerminalBindingRequest = parse_canonical_v2(&terminal_bytes)?;
        if terminal.request_digest != request.request_digest {
            bail!("terminal binding does not name the accepted request")
        }
        let (complete_bytes, terminal_securityagent) = if let Some(repetition) = repetition {
            let ((bytes, eof), observation, _) = observe_securityagent_while(|| {
                exchange(&security, &receipt.finalizer_identity, &terminal_bytes)
            })?;
            if !eof {
                bail!("finalizer terminal response lacked mandatory EOF")
            }
            (bytes, Some((repetition, observation)))
        } else {
            let (bytes, eof) = exchange(&security, &receipt.finalizer_identity, &terminal_bytes)?;
            if !eof {
                bail!("finalizer terminal response lacked mandatory EOF")
            }
            (bytes, None)
        };
        let complete: FinalizerResponseV2 = parse_canonical_v2(&complete_bytes)?;
        validate_finalizer_response_v2(&complete)?;
        if complete.request_digest != request.request_digest
            || complete.state != FinalizerResponseStateV2::HostComplete
        {
            bail!("terminal exchange did not return exact HostComplete")
        }
        if let Some((repetition, securityagent_evidence)) = terminal_securityagent {
            let receipt = native_arm_receipt(
                repetition,
                DisposableNativeArmV2::TerminalBinding,
                &complete,
                securityagent_evidence,
            )?;
            write_stdout_frame(&canonical_bytes(&receipt)?, "terminal native-arm receipt")?;
        }
        write_stdout_frame(&complete_bytes, "harness terminal frame")?;
        Ok(())
    }

    fn exchange(
        security: &SecurityUiDenied,
        identity: &substrate_common::macos_retirement_v2::ExecutableIdentityV2,
        request_bytes: &[u8],
    ) -> Result<(Vec<u8>, bool)> {
        exchange_probe(
            security,
            identity,
            request_bytes,
            ControlFramingV2::OneFrameAndEof,
            None,
        )
    }

    fn exchange_probe(
        security: &SecurityUiDenied,
        identity: &substrate_common::macos_retirement_v2::ExecutableIdentityV2,
        request_bytes: &[u8],
        framing: ControlFramingV2,
        second_body: Option<&[u8]>,
    ) -> Result<(Vec<u8>, bool)> {
        let connected = UnixStream::connect(MAC_R3_FINALIZER_ENDPOINT_V2)
            .context("connect fixed R3 finalizer endpoint")?;
        connected
            .set_read_timeout(Some(
                if framing == ControlFramingV2::OneFrameWithoutRequestEof {
                    SOCKET_TIMEOUT + Duration::from_secs(5)
                } else {
                    SOCKET_TIMEOUT
                },
            ))
            .context("set coordinator response timeout")?;
        connected
            .set_write_timeout(Some(SOCKET_TIMEOUT))
            .context("set coordinator request timeout")?;
        let owned: OwnedFd = connected.into();
        let fd3 = normalize_accepted_socket_to_fd3(owned)?;
        if fd3.as_raw_fd() != 3 {
            bail!("coordinator connection was not normalized to FD3")
        }
        let server_before = security.verify_expected_finalizer_peer(fd3.as_fd(), identity)?;
        let mut stream = UnixStream::from(fd3);
        write_one_frame(&mut stream, request_bytes)?;
        match framing {
            ControlFramingV2::OneFrameAndEof => {}
            ControlFramingV2::OneFrameTrailingByteAndEof => {
                stream
                    .write_all(&[0])
                    .context("write fixed trailing-byte control")?;
                stream
                    .flush()
                    .context("flush fixed trailing-byte control")?;
            }
            ControlFramingV2::TwoFramesAndEof => {
                write_one_frame(
                    &mut stream,
                    second_body.context("second-frame control lacks its fixed second body")?,
                )?;
            }
            ControlFramingV2::OneFrameWithoutRequestEof => {}
        }
        if framing != ControlFramingV2::OneFrameWithoutRequestEof {
            stream
                .shutdown(std::net::Shutdown::Write)
                .context("send mandatory coordinator request EOF")?;
        }
        let response_bytes = read_one_frame_to_eof(&mut stream)?;
        let server_after = security.verify_expected_finalizer_peer(stream.as_fd(), identity)?;
        if server_before != server_after {
            bail!("finalizer identity drifted across response framing and EOF")
        }
        Ok((response_bytes, true))
    }

    fn observe_securityagent_while<T, F>(
        operation: F,
    ) -> Result<(T, SecurityAgentArmEvidenceV2, Duration)>
    where
        T: Send,
        F: FnOnce() -> Result<T> + Send,
    {
        let (mut observer, mut observer_started) =
            start_securityagent_observer(OBSERVER_INITIAL_READY_TIMEOUT)?;
        thread::scope(
            |scope| -> Result<(T, SecurityAgentArmEvidenceV2, Duration)> {
                let operation = scope.spawn(|| {
                    let started = Instant::now();
                    (operation(), started.elapsed())
                });
                let mut reports = Vec::new();
                loop {
                    let rearm_at = observer_started + OBSERVER_REARM_INTERVAL;
                    if let Some(remaining) = rearm_at.checked_duration_since(Instant::now()) {
                        thread::sleep(remaining);
                    }
                    if operation.is_finished() {
                        reports.push(finish_securityagent_observer(observer).unwrap_or_else(
                            |_| {
                                // An observer/report failure during a native arm cannot allow the arm to
                                // continue or be retried without bounded UI evidence.
                                std::process::exit(78)
                            },
                        ));
                        break;
                    }
                    // Establish the replacement's baseline and receive READY while the current
                    // observer is still sampling. Only then may the current report be collected.
                    let (replacement, replacement_started) =
                        start_securityagent_observer(OBSERVER_REARM_READY_TIMEOUT)
                            .unwrap_or_else(|_| std::process::exit(78));
                    reports.push(
                        finish_securityagent_observer(observer)
                            .unwrap_or_else(|_| std::process::exit(78)),
                    );
                    observer = replacement;
                    observer_started = replacement_started;
                }
                let (result, elapsed) = operation
                    .join()
                    .map_err(|_| anyhow::anyhow!("observed coordinator operation panicked"))?;
                let result = result?;
                let observation = SecurityAgentArmEvidenceV2 {
                    schema_owner: "substrate.r3-macos-securityagent-arm-observation".to_string(),
                    schema_version: EXPERIMENT_VERSION_V2,
                    observer_path: SECURITYAGENT_OBSERVER_PATH_V2.to_string(),
                    report_set_sha256: substrate_common::macos_retirement_v2::document_sha256_v2(
                        &reports,
                    )?,
                    overlap_rearm_count: u32::try_from(reports.len() - 1)
                        .context("SecurityAgent rearm count exceeds u32")?,
                    reports,
                    gap_free_rearm_coverage: true,
                    unexpected_ui_observed: false,
                };
                observation.validate()?;
                Ok((result, observation, elapsed))
            },
        )
    }

    fn start_securityagent_observer(ready_timeout: Duration) -> Result<(Child, Instant)> {
        require_root_owned_fixed_observer()?;
        let mut command = Command::new(SECURITYAGENT_OBSERVER_PATH_V2);
        command
            .current_dir("/")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env_clear();
        let mut child = command
            .spawn()
            .context("spawn fixed SecurityAgent observer")?;
        let stderr = child
            .stderr
            .take()
            .context("fixed SecurityAgent observer lacks stderr")?;
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        thread::spawn(move || {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            let result = reader.read_line(&mut line).map(|_| line.clone());
            let _ = sender.send(result);
            if line == "ALERT\n" {
                std::process::exit(86);
            }
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => {}
                Ok(_) if line == "ALERT\n" => std::process::exit(86),
                Ok(_) => std::process::exit(78),
                Err(_) => std::process::exit(78),
            }
        });
        match receiver.recv_timeout(ready_timeout) {
            Ok(Ok(line)) if line == "READY\n" => Ok((child, Instant::now())),
            Ok(Ok(line)) if line == "ALERT\n" => std::process::exit(86),
            Ok(Ok(_)) => {
                let _ = child.kill();
                let _ = child.wait();
                bail!("SecurityAgent observer emitted an alternate ready marker")
            }
            Ok(Err(error)) => {
                let _ = child.kill();
                let _ = child.wait();
                Err(error).context("read SecurityAgent observer ready marker")
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                bail!("SecurityAgent observer ready deadline expired; arm is inconclusive")
            }
        }
    }

    fn finish_securityagent_observer(child: Child) -> Result<SecurityAgentRawReportEvidenceV2> {
        let output = child
            .wait_with_output()
            .context("wait for fixed SecurityAgent observer")?;
        if !output.status.success() {
            bail!("fixed SecurityAgent observer did not complete successfully")
        }
        let mut bytes = output.stdout;
        if bytes.pop() != Some(b'\n') || bytes.last() == Some(&b'\n') {
            bail!("SecurityAgent observer output is not one newline-terminated report")
        }
        let report: serde_json::Value = parse_canonical_v2(&bytes)?;
        let object = report
            .as_object()
            .context("SecurityAgent observation is not an object")?;
        if object
            .get("schemaOwner")
            .and_then(serde_json::Value::as_str)
            != Some("substrate.r3-macos-securityagent-observation")
            || object
                .get("schemaVersion")
                .and_then(serde_json::Value::as_u64)
                != Some(1)
            || object
                .get("sampleCount")
                .and_then(serde_json::Value::as_u64)
                != Some(101)
            || object
                .get("sampleIntervalMilliseconds")
                .and_then(serde_json::Value::as_u64)
                != Some(50)
        {
            bail!("SecurityAgent observer schema or fixed sampling bounds changed")
        }
        if object
            .get("unexpectedUiObserved")
            .and_then(serde_json::Value::as_bool)
            != Some(false)
        {
            bail!("unexpected SecurityAgent process, activation, or window was observed")
        }
        let evidence = SecurityAgentRawReportEvidenceV2 {
            raw_report_base64url: URL_SAFE_NO_PAD.encode(&bytes),
            raw_report_sha256: substrate_common::macos_retirement_v2::sha256_hex_v2(&bytes),
            raw_report_byte_length: u64::try_from(bytes.len())
                .context("SecurityAgent report length exceeds u64")?,
        };
        evidence.validate()?;
        Ok(evidence)
    }

    fn require_root_owned_fixed_observer() -> Result<()> {
        let path = Path::new(SECURITYAGENT_OBSERVER_PATH_V2);
        let mut current = std::path::PathBuf::from("/");
        for component in path.components().skip(1) {
            current.push(component.as_os_str());
            let metadata = std::fs::symlink_metadata(&current).with_context(|| {
                format!("inspect observer path component {}", current.display())
            })?;
            if metadata.file_type().is_symlink()
                || metadata.uid() != 0
                || metadata.mode() & 0o022 != 0
            {
                bail!("SecurityAgent observer path is not root-owned immutable no-follow state")
            }
            if current == path {
                if !metadata.file_type().is_file() || metadata.nlink() != 1 {
                    bail!("SecurityAgent observer is not one regular-file identity")
                }
            } else if !metadata.is_dir() {
                bail!("SecurityAgent observer parent is not a directory")
            }
        }
        Ok(())
    }

    fn native_arm_receipt(
        repetition: RepetitionV2,
        arm: DisposableNativeArmV2,
        response: &FinalizerResponseV2,
        securityagent_evidence: SecurityAgentArmEvidenceV2,
    ) -> Result<DisposableNativeArmReceiptV2> {
        let securityagent_observation_sha256 =
            substrate_common::macos_retirement_v2::document_sha256_v2(&securityagent_evidence)?;
        let receipt = DisposableNativeArmReceiptV2 {
            schema_owner: TRANSPORT_CONTROL_RECEIPT_OWNER_V2.to_string(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_string(),
            repetition: repetition.ordinal(),
            scope_id: repetition.scope_id().to_string(),
            arm,
            request_digest: response.request_digest.clone(),
            canonical_response_sha256: substrate_common::macos_retirement_v2::document_sha256_v2(
                response,
            )?,
            response_eof_observed: true,
            securityagent_evidence,
            securityagent_observation_sha256,
        };
        receipt.validate(repetition, response)?;
        Ok(receipt)
    }

    fn repetition_for_scope(scope: &str) -> Option<RepetitionV2> {
        RepetitionV2::ALL
            .into_iter()
            .find(|repetition| repetition.scope_id() == scope)
    }

    fn canonical_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>> {
        substrate_common::macos_retirement_v2::canonical_bytes_v2(value)
    }

    fn write_stdout_frame(bytes: &[u8], label: &str) -> Result<()> {
        write_one_frame(&mut std::io::stdout(), bytes)?;
        std::io::stdout()
            .flush()
            .with_context(|| format!("flush {label}"))
    }

    fn require_no_ambient_inputs() -> Result<()> {
        if std::env::args_os().count() != 1 {
            bail!("coordinator accepts no argv")
        }
        if std::env::current_dir().context("read coordinator cwd")? != Path::new("/") {
            bail!("coordinator cwd is not the fixed root directory")
        }
        let stdin = std::fs::metadata("/dev/fd/0").context("inspect coordinator stdin")?;
        let dev_null = std::fs::metadata("/dev/null").context("inspect fixed /dev/null")?;
        if stdin.file_type() != dev_null.file_type()
            || stdin.dev() != dev_null.dev()
            || stdin.ino() != dev_null.ino()
            || stdin.rdev() != dev_null.rdev()
        {
            bail!("coordinator stdin is not the fixed /dev/null EOF source")
        }
        if std::env::vars_os().any(|(key, _)| key.to_string_lossy().starts_with("SUBSTRATE_")) {
            bail!("coordinator rejects ambient SUBSTRATE_* environment input")
        }
        if std::fs::canonicalize(std::env::current_exe()?)?
            != Path::new(substrate_common::macos_retirement_v2::MAC_R3_COORDINATOR_PATH_V2)
        {
            bail!("coordinator is not running from its frozen physical path")
        }
        Ok(())
    }

    fn clear_process_environment() -> Result<()> {
        substrate_r3_macos_finalizer::ambient::clear_and_require_empty_v2("coordinator")
    }

    fn read_fixed_file(path: &Path) -> Result<Vec<u8>> {
        read_fixed_coordinator_inbox_v2(path)
    }

    fn wait_for_terminal_binding() -> Result<Vec<u8>> {
        wait_for_fixed_file(
            Path::new(TERMINAL_BINDING_PATH),
            TERMINAL_DELIVERY_TIMEOUT,
            "terminal binding request",
        )
    }

    fn wait_for_publisher_output<T>(
        store: &ExperimentStoreV2,
        repetition: RepetitionV2,
        artifact: PublisherArtifactV2,
        timeout: Duration,
        label: &str,
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned + serde::Serialize,
    {
        let deadline = Instant::now() + timeout;
        loop {
            match store.read_publisher_output::<T>(repetition, artifact) {
                Ok((value, _)) => return Ok(value),
                Err(error)
                    if error
                        .chain()
                        .filter_map(|cause| cause.downcast_ref::<std::io::Error>())
                        .any(|cause| cause.kind() == std::io::ErrorKind::NotFound) => {}
                Err(error) => return Err(error),
            }
            if Instant::now() >= deadline {
                bail!("{label} delivery timed out; result is inconclusive")
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    fn wait_for_publisher_input<T>(
        store: &ExperimentStoreV2,
        repetition: RepetitionV2,
        artifact: PublisherArtifactV2,
        timeout: Duration,
        label: &str,
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned + serde::Serialize,
    {
        let deadline = Instant::now() + timeout;
        loop {
            match store.read_publisher_input::<T>(repetition, artifact) {
                Ok((value, _)) => return Ok(value),
                Err(error)
                    if error
                        .chain()
                        .filter_map(|cause| cause.downcast_ref::<std::io::Error>())
                        .any(|cause| cause.kind() == std::io::ErrorKind::NotFound) => {}
                Err(error) => return Err(error),
            }
            if Instant::now() >= deadline {
                bail!("{label} delivery timed out; result is inconclusive")
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    fn read_root_installed_packet<T>(path: &Path) -> Result<T>
    where
        T: serde::de::DeserializeOwned + serde::Serialize,
    {
        let mut current = std::path::PathBuf::from("/");
        for component in path.components().skip(1) {
            current.push(component.as_os_str());
            let metadata = std::fs::symlink_metadata(&current)
                .with_context(|| format!("inspect installed packet path {}", current.display()))?;
            if metadata.file_type().is_symlink()
                || metadata.uid() != 0
                || metadata.mode() & 0o022 != 0
            {
                bail!("installed packet path is not root-owned immutable no-follow state")
            }
            if current == path {
                if !metadata.file_type().is_file()
                    || metadata.mode() & 0o7777 != 0o444
                    || metadata.nlink() != 1
                {
                    bail!("installed packet is not one root-owned 0444 regular file")
                }
            } else if !metadata.is_dir() {
                bail!("installed packet parent is not a directory")
            }
        }
        let mut file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path)
            .with_context(|| format!("open installed packet {}", path.display()))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .context("read installed coordinator packet")?;
        parse_canonical_v2(&bytes)
    }

    fn wait_for_fixed_file(path: &Path, timeout: Duration, label: &str) -> Result<Vec<u8>> {
        let deadline = Instant::now() + timeout;
        loop {
            match read_fixed_file(path) {
                Ok(bytes) => return Ok(bytes),
                Err(error)
                    if error
                        .chain()
                        .filter_map(|cause| cause.downcast_ref::<std::io::Error>())
                        .any(|cause| cause.kind() == std::io::ErrorKind::NotFound) => {}
                Err(error) => return Err(error),
            }
            if Instant::now() >= deadline {
                bail!("{label} delivery timed out; result is inconclusive")
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}
