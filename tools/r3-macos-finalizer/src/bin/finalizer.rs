#![cfg_attr(not(target_os = "macos"), allow(dead_code, unused_imports))]

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("substrate R3 macOS evidence finalizer is available only on macOS");
    std::process::exit(78);
}

#[cfg(target_os = "macos")]
fn main() -> anyhow::Result<()> {
    macos::run()
}

#[cfg(target_os = "macos")]
mod macos {
    use std::fs::{self, OpenOptions};
    use std::io::Read;
    use std::os::fd::{AsFd, AsRawFd};
    use std::os::unix::fs::{FileTypeExt, MetadataExt, OpenOptionsExt};
    use std::os::unix::net::UnixStream;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use anyhow::{bail, Context, Result};
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use serde::Serialize;
    use substrate_common::macos_retirement_v2::{
        canonical_bytes_v2, document_sha256_v2, parse_canonical_v2, sha256_hex_v2,
        validate_finalization_request_v2, validate_finalizer_response_v2, FinalizationRequestV2,
        FinalizerResponseStateV2, FinalizerResponseV2, GenericPasswordRoleV2,
        HostResourceLocatorV2, HostTargetRoleV2, PreservingStopClassificationV2,
        MAC_R3_COORDINATOR_PATH_V2, MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2,
        MAC_R3_FINALIZER_ENDPOINT_V2, MAC_R3_FINALIZER_JOURNAL_ROOT_V2, MAC_R3_FINALIZER_PATH_V2,
        MAC_R3_FINALIZER_PLIST_PATH_V2, MAC_R3_FINALIZER_PROTOCOL_OWNER_V2,
        MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
    };
    use substrate_r3_macos_finalizer::contract::{
        validate_compiled_literals, PeerAttestation, TerminalBindingRequest,
        TERMINAL_BINDING_OWNER, TERMINAL_BINDING_VERSION,
    };
    use substrate_r3_macos_finalizer::darwin::{
        accept_listener_peer_on_fd3, activate_listener_socket, ProcessMeasurement,
        SecurityUiDenied, VerifiedCoordinatorProcess,
    };
    use substrate_r3_macos_finalizer::engine::{
        record_accepted_recovery_preparation_failure, replay_existing_claim, AcceptedRequest,
        ClosedEffects, EffectObservation, EffectObservationPhase, ExistingClaimDisposition,
        FinalizerEngine,
    };
    use substrate_r3_macos_finalizer::frame::{read_one_frame_to_eof, write_one_frame};
    use substrate_r3_macos_finalizer::frozen_identity::{
        BUILD_INPUTS_SHA256, CAPABILITY_DIGEST, EXPECTED_COORDINATOR_ACCOUNT,
        EXPECTED_COORDINATOR_BUILD_INPUTS_SHA256, EXPECTED_COORDINATOR_CDHASH,
        EXPECTED_COORDINATOR_GID, EXPECTED_COORDINATOR_REQUIREMENT, EXPECTED_COORDINATOR_SHA256,
        EXPECTED_COORDINATOR_SOURCE_COMMIT, EXPECTED_COORDINATOR_SOURCE_HASHES_SHA256,
        EXPECTED_COORDINATOR_SOURCE_TREE, EXPECTED_COORDINATOR_UID, SOURCE_COMMIT,
        SOURCE_IDENTITY_SHA256, SOURCE_TREE,
    };
    use substrate_r3_macos_finalizer::journal::LockedJournal;
    use substrate_r3_macos_finalizer::native_effects::MacNativeEffects;
    use substrate_r3_macos_finalizer::targets::{derive_targets_from_ledger, FixedTarget};

    const SOCKET_TIMEOUT: Duration = Duration::from_secs(30);
    const SAFE_REJECTION_REQUEST_DOMAIN: &[u8] =
        b"substrate.r3-macos-finalizer.safe-rejection.request.v2\0";
    const NO_ACCEPTED_JOURNAL_DOMAIN: &[u8] =
        b"substrate.r3-macos-finalizer.no-accepted-journal.v2\0";

    #[derive(Serialize)]
    struct ProcessStartJoin {
        pid: i32,
        seconds: u64,
        microseconds: u64,
    }

    #[derive(Serialize)]
    struct PhysicalJoin {
        device: u64,
        inode: u64,
        owner_uid: u32,
        owner_gid: u32,
        mode: u32,
        link_count: u64,
        size: u64,
        modified_seconds: i64,
        modified_nanoseconds: i64,
    }

    pub fn run() -> Result<()> {
        require_no_ambient_inputs()?;
        validate_compiled_literals()?;
        clear_process_environment()?;

        // This is deliberately the first call into Security.framework. No code or Keychain
        // attestation is allowed to precede the process-wide no-interaction posture.
        let security = SecurityUiDenied::establish_first()?;
        let listener = activate_listener_socket()?;
        let fd3 = accept_listener_peer_on_fd3(listener)?;
        if fd3.as_raw_fd() != 3 {
            bail!("accepted finalizer connection was not normalized to FD3")
        }
        let peer_before = security.verify_expected_coordinator(fd3.as_fd())?;
        let mut stream = UnixStream::from(fd3);
        stream
            .set_read_timeout(Some(SOCKET_TIMEOUT))
            .context("set fixed finalizer request timeout")?;
        stream
            .set_write_timeout(Some(SOCKET_TIMEOUT))
            .context("set fixed finalizer response timeout")?;
        let request_bytes = match read_one_frame_to_eof(&mut stream) {
            Ok(bytes) => bytes,
            Err(_) => {
                write_response_and_eof(&mut stream, safe_preaccept_response(&[])?)?;
                return Ok(());
            }
        };
        let peer_after = security.verify_expected_coordinator(stream.as_fd())?;
        if peer_before != peer_after {
            bail!("coordinator identity drifted after complete request framing and EOF")
        }

        let response = match parse_canonical_v2::<FinalizationRequestV2>(&request_bytes) {
            Ok(request) if validate_finalization_request_v2(&request, None).is_ok() => {
                handle_initial_request(request, request_bytes, peer_after, security)?
            }
            Ok(_) => safe_preaccept_response(&request_bytes)?,
            Err(_) => match parse_canonical_v2::<TerminalBindingRequest>(&request_bytes) {
                Ok(terminal) => {
                    let authority =
                        decode_base64(&terminal.authority_request, "terminal authority request")
                            .and_then(|bytes| parse_canonical_v2::<FinalizationRequestV2>(&bytes));
                    if terminal.schema_owner == TERMINAL_BINDING_OWNER
                        && terminal.schema_version == TERMINAL_BINDING_VERSION
                        && authority.as_ref().is_ok_and(|authority| {
                            authority.request_digest == terminal.request_digest
                                && validate_finalization_request_v2(authority, None).is_ok()
                        })
                    {
                        handle_terminal_request(terminal, peer_after, security)?
                    } else {
                        safe_preaccept_response(&request_bytes)?
                    }
                }
                Err(_) => safe_preaccept_response(&request_bytes)?,
            },
        };
        write_response_and_eof(&mut stream, response)?;
        Ok(())
    }

    fn safe_preaccept_response(observed_request_bytes: &[u8]) -> Result<FinalizerResponseV2> {
        let mut request_material =
            Vec::with_capacity(SAFE_REJECTION_REQUEST_DOMAIN.len() + observed_request_bytes.len());
        request_material.extend_from_slice(SAFE_REJECTION_REQUEST_DOMAIN);
        request_material.extend_from_slice(observed_request_bytes);
        let response = FinalizerResponseV2 {
            schema_owner: MAC_R3_FINALIZER_PROTOCOL_OWNER_V2.to_string(),
            schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
            request_digest: sha256_hex_v2(&request_material),
            state: FinalizerResponseStateV2::SafePreAcceptanceStop,
            journal_head_sha256: sha256_hex_v2(NO_ACCEPTED_JOURNAL_DOMAIN),
            effects_observation_sha256: None,
            terminal_acknowledgement_sha256: None,
            preserving_classification: Some(
                PreservingStopClassificationV2::IdentityOrAuthorityMismatch,
            ),
        };
        validate_finalizer_response_v2(&response)?;
        Ok(response)
    }

    fn write_response_and_eof(
        stream: &mut UnixStream,
        response: FinalizerResponseV2,
    ) -> Result<()> {
        validate_finalizer_response_v2(&response)?;
        let response_bytes = canonical_bytes_v2(&response)?;
        write_one_frame(stream, &response_bytes)?;
        stream
            .shutdown(std::net::Shutdown::Write)
            .context("send mandatory finalizer response EOF")
    }

    fn handle_initial_request(
        request: FinalizationRequestV2,
        request_bytes: Vec<u8>,
        peer: VerifiedCoordinatorProcess,
        security: SecurityUiDenied,
    ) -> Result<FinalizerResponseV2> {
        let first_acceptance =
            !existing_claim_is_possible(&request.successor_capsule.intent.scope_id)?;
        let now = first_acceptance.then(now_unix_ns).transpose()?;
        let (receipt, _acknowledgement, binding) =
            match validate_finalization_request_v2(&request, now) {
                Ok(validated) => validated,
                Err(_) => return safe_preaccept_response(&request_bytes),
            };
        let peer_mode = if first_acceptance {
            RuntimePeerMode::Initial
        } else {
            RuntimePeerMode::Rejoin
        };
        if validate_frozen_runtime_identity(&receipt, &peer, &security, peer_mode).is_err() {
            return safe_preaccept_response(&request_bytes);
        }

        let peer_attestation = peer_attestation(&peer)?;
        let peer_attestation_bytes = canonical_bytes_v2(&peer_attestation)?;
        let admitted = if first_acceptance {
            AcceptedRequest::admit_canonical_request(
                request_bytes.clone(),
                peer_attestation_bytes,
                now,
            )
        } else {
            AcceptedRequest::admit_canonical_rejoin_request(
                request_bytes.clone(),
                peer_attestation_bytes,
            )
        };
        let accepted = match admitted {
            Ok(accepted) => accepted,
            Err(_) => return safe_preaccept_response(&request_bytes),
        };
        let receipt = accepted.validated_receipt()?;
        let existing =
            replay_existing_claim(Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2), 0, &accepted)?;
        let (first_claim, recovery_cursor) = match existing {
            ExistingClaimDisposition::Response(response) => {
                validate_finalizer_response_v2(&response)?;
                return Ok(response);
            }
            ExistingClaimDisposition::RequiresEffectsRecovery(cursor) => (false, Some(cursor)),
            ExistingClaimDisposition::NoExistingClaim => (true, None),
        };
        let recovery = recovery_cursor.as_ref();
        let targets = derive_targets_from_ledger(
            receipt.target_set_kind,
            &receipt.scope_id,
            &receipt.target_ledger,
        )?;
        let current_lock_path = target_path(
            targets
                .iter()
                .find(|target| target.role == HostTargetRoleV2::CurrentAnchorLock)
                .context("closed target plan lacks current-anchor lock")?,
        )?;
        let terminal_latch_path = target_path(
            targets
                .iter()
                .find(|target| target.role == HostTargetRoleV2::TerminalLatch)
                .context("closed target plan lacks terminal latch")?,
        )?;
        let effects = match MacNativeEffects::new(
            security,
            &current_lock_path,
            &terminal_latch_path,
            receipt,
            recovery,
        ) {
            Ok(effects) => effects,
            Err(_) if recovery.is_some() => {
                return record_accepted_recovery_preparation_failure(
                    Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2),
                    0,
                    &accepted,
                )
            }
            Err(_) => return safe_preaccept_response(accepted.canonical_request()),
        };
        if first_claim {
            let live_preaccept = (|| -> Result<()> {
                if effects.current_lock_identity_sha256()?.as_deref()
                    != Some(receipt.current_lock_identity_sha256.as_str())
                {
                    bail!("live current-anchor lock identity differs before finalizer acceptance")
                }
                let observed = effects
                    .terminal_latch_identity_sha256()?
                    .context("live terminal latch is absent before first acceptance")?;
                let expected = receipt
                    .target_ledger
                    .iter()
                    .find(|identity| identity.role == HostTargetRoleV2::TerminalLatch)
                    .context("closed ledger lacks terminal latch identity")?;
                if observed != expected.expected_before_sha256 {
                    bail!("live terminal latch identity differs before acceptance")
                }
                verify_live_protected_cas(&request, &binding, &targets, &effects)
            })();
            if live_preaccept.is_err() {
                return safe_preaccept_response(accepted.canonical_request());
            }
        }
        let response =
            FinalizerEngine::new(Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2), 0, effects)
                .execute(&accepted)?;
        validate_finalizer_response_v2(&response)?;
        Ok(response)
    }

    fn handle_terminal_request(
        terminal: TerminalBindingRequest,
        peer: VerifiedCoordinatorProcess,
        security: SecurityUiDenied,
    ) -> Result<FinalizerResponseV2> {
        if terminal.schema_owner != TERMINAL_BINDING_OWNER
            || terminal.schema_version != TERMINAL_BINDING_VERSION
        {
            bail!("terminal binding owner or version differs")
        }
        let authority_bytes = decode_base64(&terminal.authority_request, "authority request")?;
        let attestation = peer_attestation(&peer)?;
        let peer_bytes = canonical_bytes_v2(&attestation)?;
        let accepted =
            AcceptedRequest::admit_canonical_rejoin_request(authority_bytes, peer_bytes)?;
        let receipt = accepted.validated_receipt()?;
        if terminal.request_digest != accepted.request_digest() {
            bail!("terminal binding request digest differs from accepted authority")
        }
        validate_frozen_runtime_identity(receipt, &peer, &security, RuntimePeerMode::Rejoin)?;
        let effects_bytes = decode_base64(&terminal.effects_response, "effects response")?;
        let parity_bytes = decode_base64(&terminal.parity_proof, "parity proof")?;
        let acknowledgement_bytes = decode_base64(
            &terminal.terminal_acknowledgement,
            "terminal acknowledgement",
        )?;
        let harness_public_key = receipt.harness_public_key.clone();
        let response = FinalizerEngine::new(
            Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2),
            0,
            TerminalNoEffects,
        )
        .bind_terminal_proofs(
            &accepted,
            &effects_bytes,
            &parity_bytes,
            &acknowledgement_bytes,
            &harness_public_key,
        )?;
        validate_finalizer_response_v2(&response)?;
        Ok(response)
    }

    struct TerminalNoEffects;

    impl ClosedEffects for TerminalNoEffects {
        fn observe(
            &mut self,
            _target: &substrate_r3_macos_finalizer::targets::DerivedTarget,
            _expected_before_sha256: &str,
            _phase: EffectObservationPhase,
        ) -> Result<EffectObservation> {
            bail!("terminal acknowledgement cannot inspect a target")
        }

        fn invoke(
            &mut self,
            _target: &substrate_r3_macos_finalizer::targets::DerivedTarget,
            _expected_before_sha256: &str,
        ) -> Result<()> {
            bail!("terminal acknowledgement cannot invoke an effect")
        }
    }

    fn decode_base64(value: &str, label: &str) -> Result<Vec<u8>> {
        if value.is_empty() || value.contains('=') {
            bail!("{label} is not canonical base64url")
        }
        let bytes = URL_SAFE_NO_PAD
            .decode(value)
            .with_context(|| format!("decode {label}"))?;
        if URL_SAFE_NO_PAD.encode(&bytes) != value {
            bail!("{label} is not canonical base64url")
        }
        Ok(bytes)
    }

    fn require_no_ambient_inputs() -> Result<()> {
        if std::env::args_os().count() != 1 {
            bail!("finalizer accepts no argv")
        }
        let cwd = std::env::current_dir().context("read finalizer cwd")?;
        if cwd != Path::new("/") {
            bail!("finalizer cwd is not the fixed launchd WorkingDirectory")
        }
        let stdin = fs::metadata("/dev/fd/0").context("inspect finalizer stdin descriptor")?;
        let dev_null = fs::metadata("/dev/null").context("inspect fixed /dev/null")?;
        if stdin.file_type() != dev_null.file_type()
            || stdin.dev() != dev_null.dev()
            || stdin.ino() != dev_null.ino()
            || stdin.rdev() != dev_null.rdev()
        {
            bail!("finalizer stdin is not the fixed /dev/null EOF source")
        }
        Ok(())
    }

    fn clear_process_environment() -> Result<()> {
        substrate_r3_macos_finalizer::ambient::clear_and_require_empty_v2("finalizer")
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum RuntimePeerMode {
        Initial,
        Rejoin,
    }

    fn validate_frozen_runtime_identity(
        receipt: &substrate_common::macos_retirement_v2::PublisherPreRemovalReceiptV2,
        peer: &VerifiedCoordinatorProcess,
        security: &SecurityUiDenied,
        mode: RuntimePeerMode,
    ) -> Result<()> {
        if receipt.capability_digest != CAPABILITY_DIGEST
            || receipt.finalizer_identity.source_hashes_sha256 != SOURCE_IDENTITY_SHA256
            || receipt.finalizer_identity.build_inputs_sha256 != BUILD_INPUTS_SHA256
            || receipt.finalizer_identity.source_commit != SOURCE_COMMIT
            || receipt.finalizer_identity.source_tree != SOURCE_TREE
            || receipt.coordinator_identity.executable_sha256 != EXPECTED_COORDINATOR_SHA256
            || receipt.coordinator_identity.source_commit != EXPECTED_COORDINATOR_SOURCE_COMMIT
            || receipt.coordinator_identity.source_tree != EXPECTED_COORDINATOR_SOURCE_TREE
            || receipt.coordinator_identity.source_hashes_sha256
                != EXPECTED_COORDINATOR_SOURCE_HASHES_SHA256
            || receipt.coordinator_identity.build_inputs_sha256
                != EXPECTED_COORDINATOR_BUILD_INPUTS_SHA256
            || receipt.coordinator_identity.cdhash != EXPECTED_COORDINATOR_CDHASH
            || receipt.coordinator_identity.designated_requirement
                != EXPECTED_COORDINATOR_REQUIREMENT
            || receipt.coordinator_identity.intended_path != MAC_R3_COORDINATOR_PATH_V2
            || receipt.coordinator_identity.signing_identifier
                != MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2
            || receipt.coordinator_process.effective_uid != EXPECTED_COORDINATOR_UID
            || receipt.coordinator_process.effective_gid != EXPECTED_COORDINATOR_GID
            || receipt.coordinator_process.canonical_account != EXPECTED_COORDINATOR_ACCOUNT
            || peer.peer.effective_gid != EXPECTED_COORDINATOR_GID
            || peer.measurement.executable_sha256 != EXPECTED_COORDINATOR_SHA256
            || peer.measurement.executable_file.size != receipt.coordinator_identity.executable_size
            || physical_identity_sha256(&peer.measurement)?
                != receipt.coordinator_identity.physical_identity_sha256
        {
            bail!("signed receipt does not match the frozen source/build/capability/coordinator")
        }
        if (mode == RuntimePeerMode::Initial
            && receipt.coordinator_process.process_start_identity_sha256
                != process_start_sha256(peer)?)
            || receipt.coordinator_process.executable_identity_sha256
                != document_sha256_v2(&receipt.coordinator_identity)?
        {
            bail!("signed coordinator process identity does not match its audited live process")
        }
        let self_identity = security.verify_finalizer_self(&receipt.finalizer_identity)?;
        if self_identity.measurement.executable_path != Path::new(MAC_R3_FINALIZER_PATH_V2)
            || self_identity.measurement.executable_sha256
                != receipt.finalizer_identity.executable_sha256
            || self_identity.measurement.executable_file.size
                != receipt.finalizer_identity.executable_size
            || physical_identity_sha256(&self_identity.measurement)?
                != receipt.finalizer_identity.physical_identity_sha256
        {
            bail!("running finalizer does not match its signed executable identity")
        }
        verify_launch_artifacts(receipt)
    }

    fn peer_attestation(peer: &VerifiedCoordinatorProcess) -> Result<PeerAttestation> {
        let mut audit = Vec::with_capacity(32);
        for value in peer.peer.audit_token {
            audit.extend_from_slice(&value.to_be_bytes());
        }
        Ok(PeerAttestation {
            schema_owner: "substrate.r3-macos-finalizer-peer-attestation".to_string(),
            schema_version: 1,
            effective_uid: peer.peer.effective_uid,
            effective_gid: peer.peer.effective_gid,
            canonical_account: peer.peer.canonical_account.clone(),
            pid: peer.peer.pid,
            audit_token_sha256: sha256_hex_v2(&audit),
            process_start_sha256: process_start_sha256(peer)?,
            executable_path: peer.measurement.executable_path.display().to_string(),
            executable_physical_identity_sha256: physical_identity_sha256(&peer.measurement)?,
            executable_sha256: peer.measurement.executable_sha256.clone(),
            cdhash: hex_lower(&peer.cdhash),
        })
    }

    fn process_start_sha256(peer: &VerifiedCoordinatorProcess) -> Result<String> {
        Ok(sha256_hex_v2(&canonical_bytes_v2(&ProcessStartJoin {
            pid: peer.peer.pid,
            seconds: peer.measurement.start.seconds,
            microseconds: peer.measurement.start.microseconds,
        })?))
    }

    fn physical_identity_sha256(measurement: &ProcessMeasurement) -> Result<String> {
        let value = &measurement.executable_file;
        Ok(sha256_hex_v2(&canonical_bytes_v2(&PhysicalJoin {
            device: value.device,
            inode: value.inode,
            owner_uid: value.owner_uid,
            owner_gid: value.owner_gid,
            mode: value.mode,
            link_count: value.link_count,
            size: value.size,
            modified_seconds: value.modified_seconds,
            modified_nanoseconds: value.modified_nanoseconds,
        })?))
    }

    fn verify_launch_artifacts(
        receipt: &substrate_common::macos_retirement_v2::PublisherPreRemovalReceiptV2,
    ) -> Result<()> {
        let plist = read_root_file(Path::new(MAC_R3_FINALIZER_PLIST_PATH_V2), 0o644)?;
        if sha256_hex_v2(&plist) != receipt.launch_identity.launchd_plist_sha256 {
            bail!("installed launchd plist bytes differ from signed launch identity")
        }
        let endpoint = fs::symlink_metadata(MAC_R3_FINALIZER_ENDPOINT_V2)
            .context("inspect fixed finalizer endpoint")?;
        if !endpoint.file_type().is_socket()
            || endpoint.uid() != receipt.launch_identity.endpoint_owner_uid
            || endpoint.gid() != receipt.launch_identity.endpoint_group_gid
            || endpoint.mode() & 0o7777 != 0o660
        {
            bail!("finalizer endpoint physical owner/group/mode/type differs")
        }
        Ok(())
    }

    fn verify_live_protected_cas(
        request: &FinalizationRequestV2,
        binding: &substrate_common::macos_retirement_v2::ProtectedCasBindingV2,
        targets: &[substrate_r3_macos_finalizer::targets::DerivedTarget],
        effects: &MacNativeEffects,
    ) -> Result<()> {
        let mut wrappers = targets.iter().filter(|target| {
            matches!(
                &target.locator,
                HostResourceLocatorV2::DisposableProtectedWrapper
                    | HostResourceLocatorV2::GenericPassword {
                        role: GenericPasswordRoleV2::RetirementCas
                    }
            )
        });
        let wrapper = wrappers
            .next()
            .context("closed target plan lacks its exact retirement CAS wrapper")?;
        if wrappers.next().is_some() {
            bail!("closed target plan has multiple retirement CAS wrappers")
        }
        let encoded = URL_SAFE_NO_PAD
            .decode(&request.protected_cas_binding)
            .context("decode protected CAS authority bytes")?;
        match &wrapper.target {
            FixedTarget::DerivedFile { .. } => {
                if wrapper.locator != HostResourceLocatorV2::DisposableProtectedWrapper {
                    bail!("derived protected wrapper is not the disposable CAS locator")
                }
                let bytes = effects.read_receipt_bound_disposable_wrapper_data()?;
                if bytes != encoded
                    || parse_canonical_v2::<
                        substrate_common::macos_retirement_v2::ProtectedCasBindingV2,
                    >(&bytes)?
                        != *binding
                {
                    bail!("live disposable protected CAS differs before acceptance")
                }
            }
            FixedTarget::ProtectedGenericPassword { .. } => {
                if wrapper.locator
                    != (HostResourceLocatorV2::GenericPassword {
                        role: GenericPasswordRoleV2::RetirementCas,
                    })
                {
                    bail!("protected generic password is not the retirement CAS locator")
                }
                let bytes = effects
                    .read_receipt_bound_protected_cas_data()?
                    .context("prospective product protected CAS is absent before acceptance")?;
                if bytes != encoded
                    || parse_canonical_v2::<
                        substrate_common::macos_retirement_v2::ProtectedCasBindingV2,
                    >(&bytes)?
                        != *binding
                {
                    bail!("live prospective protected CAS differs before acceptance")
                }
            }
            _ => bail!("protected wrapper role expanded to the wrong target kind"),
        }
        Ok(())
    }

    fn target_path(
        target: &substrate_r3_macos_finalizer::targets::DerivedTarget,
    ) -> Result<PathBuf> {
        match &target.target {
            FixedTarget::DerivedFile { path } => Ok(PathBuf::from(path)),
            _ => bail!("closed target is not a scope file"),
        }
    }

    fn read_root_file(path: &Path, mode: u32) -> Result<Vec<u8>> {
        let mut file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path)
            .with_context(|| format!("open exact root file {}", path.display()))?;
        let metadata = file.metadata().context("inspect exact root file")?;
        if !metadata.file_type().is_file()
            || metadata.uid() != 0
            || metadata.mode() & 0o7777 != mode
            || metadata.nlink() != 1
        {
            bail!("exact root file owner/mode/type/link count differs")
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .context("read exact root file")?;
        Ok(bytes)
    }

    fn existing_claim_is_possible(scope: &str) -> Result<bool> {
        existing_claim_is_possible_at(Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2), scope, 0)
    }

    fn existing_claim_is_possible_at(root: &Path, scope: &str, expected_uid: u32) -> Result<bool> {
        let Some(journal) = LockedJournal::open_existing_fixed(root, scope, expected_uid)? else {
            return Ok(false);
        };
        // `head()` is the authoritative crash-recovery read. In particular, it validates and
        // installs HEAD for the one exact durable generation-1 orphan left by a crash between the
        // generation fsync and HEAD CAS. A raw HEAD-path probe would misclassify that accepted
        // claim as a new first acceptance and reject a legitimate restarted coordinator.
        Ok(journal.head()?.is_some())
    }

    fn now_unix_ns() -> Result<u64> {
        let value = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system time precedes epoch")?
            .as_nanos();
        u64::try_from(value).context("system time exceeds protocol range")
    }

    fn hex_lower(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn safe_preaccept_response_is_closed_deterministic_and_input_bound() {
            let first = safe_preaccept_response(br#"{"unknown":true}"#).unwrap();
            let same = safe_preaccept_response(br#"{"unknown":true}"#).unwrap();
            let different = safe_preaccept_response(br#"{"unknown":false}"#).unwrap();
            assert_eq!(first, same);
            assert_ne!(first.request_digest, different.request_digest);
            assert_eq!(first.state, FinalizerResponseStateV2::SafePreAcceptanceStop);
            assert_eq!(
                first.preserving_classification,
                Some(PreservingStopClassificationV2::IdentityOrAuthorityMismatch)
            );
            validate_finalizer_response_v2(&first).unwrap();
        }

        #[test]
        fn claim_probe_initializes_the_precreated_empty_activation_root_without_a_claim() {
            use std::os::unix::fs::PermissionsExt as _;

            let temporary = tempfile::tempdir().unwrap();
            let root = temporary.path().join("journal");
            let scope = "019ffec6-95f6-7d30-80bc-8003ce27d5ba";
            let uid = unsafe { libc::geteuid() };
            fs::create_dir(&root).unwrap();
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();

            assert!(!existing_claim_is_possible_at(&root, scope, uid).unwrap());
            assert!(root.join("journal-root.lock").is_file());
            assert!(!root.join(scope).exists());
        }

        #[test]
        fn claim_probe_recovers_generation_one_without_head_as_rejoin() {
            let temporary = tempfile::tempdir().unwrap();
            let root = temporary.path().join("journal");
            let scope = "019ffec6-95f6-7d30-80bc-8003ce27d5ba";
            // SAFETY: scalar identity query used only to own this disposable test journal.
            let uid = unsafe { libc::geteuid() };
            let journal = LockedJournal::open_fixed(&root, scope, uid).unwrap();
            let claim = substrate_r3_macos_finalizer::journal::JournalClaimIdentity {
                request_digest: "1".repeat(64),
                authority_bytes_sha256: "2".repeat(64),
                successor_capsule_sha256: "3".repeat(64),
                accepted_peer_attestation_sha256: "4".repeat(64),
                accepted_peer_identity_sha256: "5".repeat(64),
            };
            journal
                .append(
                    None,
                    &claim,
                    substrate_r3_macos_finalizer::contract::JournalEvent {
                        kind: substrate_r3_macos_finalizer::contract::JournalEventKind::FinalizerAccepted,
                        host_state: substrate_common::macos_retirement_v2::HostRetirementStateV2::FinalizerAccepted,
                        effect_ordinal: None,
                        effect_role: None,
                        effect_identity_sha256: None,
                    effect_invocation_attempt: None,
                        observation_sha256: None,
                        response_sha256: None,
                        preserving_classification: None,
                    },
                )
                .unwrap();
            drop(journal);
            fs::remove_file(root.join(scope).join("HEAD")).unwrap();

            assert!(existing_claim_is_possible_at(&root, scope, uid).unwrap());
            assert!(root.join(scope).join("HEAD").is_file());
        }

        #[test]
        fn source_clears_the_complete_environment_before_security_or_acceptance() {
            let source = include_str!("finalizer.rs");
            let clear = source.find("clear_process_environment()?").unwrap();
            let security = source.find("SecurityUiDenied::establish_first()?").unwrap();
            let accept = source
                .find("let fd3 = accept_listener_peer_on_fd3")
                .unwrap();
            assert!(clear < security && clear < accept);
            let ambient = include_str!("../ambient.rs");
            assert!(ambient.contains("libc::unsetenv(name.as_ptr())"));
            assert!(ambient.contains("vars_os().next().is_some()"));
        }
    }
}
