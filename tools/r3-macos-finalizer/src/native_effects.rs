use std::collections::BTreeMap;
use std::ffi::{c_char, CString};
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, sha256_hex_v2, validate_publisher_pre_removal_receipt_v2,
    DisposableCapabilityControlV2, GenericPasswordRoleV2, HostResourceLocatorV2, HostTargetRoleV2,
    PublisherPreRemovalReceiptV2, TargetSetKindV2, MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2,
};

use crate::darwin::{
    ExactGenericPasswordDeleteOutcome, ExactKeyDeleteOutcome, RetainedExactGenericPassword,
    SecurityUiDenied,
};
use crate::disposable_capability::{
    CanonicalDisposableCapabilityAfterV2, CanonicalDisposableCapabilityInvocationV2,
    CanonicalDisposableCapabilityStateV2, DisposableRecoveryObservationClassV2,
    RetainedExactSystemKey,
};
use crate::engine::{
    ClosedEffects, EffectObservation, EffectObservationPhase, ObservationClass,
    RecoveryEffectPhase, ValidatedRecoveryCursor,
};
use crate::targets::{derive_targets_from_ledger, DerivedTarget, FixedTarget};

const PRODUCT_LAUNCHD_SERVICE: &str = "system/com.substrate.lifecycle.publisher.v1";
const PRODUCT_MACH_ENDPOINT: &str = "com.substrate.lifecycle.publisher.v1";
const LAUNCHCTL_PRINT_NOT_FOUND_EXIT: i32 = 113;
const BOOTSTRAP_SUCCESS: i32 = 0;
const BOOTSTRAP_UNKNOWN_SERVICE: i32 = 1102;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchctlPrintClass {
    Present,
    NotFound,
    Ambiguous(Option<i32>),
}

pub fn classify_launchctl_print(success: bool, code: Option<i32>) -> LaunchctlPrintClass {
    if success {
        LaunchctlPrintClass::Present
    } else if code == Some(LAUNCHCTL_PRINT_NOT_FOUND_EXIT) {
        LaunchctlPrintClass::NotFound
    } else {
        LaunchctlPrintClass::Ambiguous(code)
    }
}

pub fn classify_disposable_control_recovery(
    _control: DisposableCapabilityControlV2,
    cached_denial_witness: bool,
    _platform_denial_and_preservation: bool,
) -> ObservationClass {
    // An unchanged Keychain item can establish preservation, but after an Invoked crash it can
    // never reconstruct whether *any* of the four calls actually denied. ExactFinal requires the
    // same-process canonical raw denial witness cached by invoke_disposable_control.
    if cached_denial_witness {
        ObservationClass::ExactFinal
    } else {
        ObservationClass::Ambiguous
    }
}

fn classify_signer_absence_recovery(
    phase: EffectObservationPhase,
    cached_successful_delete_witness: bool,
) -> ObservationClass {
    if phase == EffectObservationPhase::Invoked && cached_successful_delete_witness {
        ObservationClass::ExactFinal
    } else {
        // Exact absence cannot reconstruct whether an interrupted SecItemDelete returned success.
        // Preserve on every restarted/errored Invoked path; only the same-process status-0 witness
        // authorizes EffectObserved/Complete.
        ObservationClass::Ambiguous
    }
}

fn classify_filesystem_absence_recovery(
    phase: EffectObservationPhase,
    cached_durable_delete_witness: bool,
) -> ObservationClass {
    if phase == EffectObservationPhase::Prepared || cached_durable_delete_witness {
        ObservationClass::ExactFinal
    } else {
        // ENOENT after EffectInvoked cannot prove the unlink/rmdir reached stable storage. Only
        // the same-process witness created after parent fsync + reopen/absence readback may finish.
        ObservationClass::Ambiguous
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MachEndpointClass {
    Present,
    NotFound,
    Ambiguous(i32),
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct FileObservation<'a> {
    path: &'a str,
    device: u64,
    inode: u64,
    uid: u32,
    gid: u32,
    mode: u32,
    link_count: u64,
    size: u64,
    content_sha256: String,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct DisposableWrapperObservation<'a> {
    path: &'a str,
    device: u64,
    inode: u64,
    uid: u32,
    gid: u32,
    mode: u32,
    link_count: u64,
}

const NATIVE_EFFECT_EVIDENCE_OWNER: &str = "substrate.mac-r3-finalizer-native-effect";
const NATIVE_EFFECT_EVIDENCE_VERSION: u32 = 2;

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct DisposableControlEvidenceV2 {
    schema_owner: String,
    schema_version: u32,
    invocation: CanonicalDisposableCapabilityInvocationV2,
    after_observation: CanonicalDisposableCapabilityAfterV2,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct ExactKeyDeleteEvidenceV2 {
    schema_owner: String,
    schema_version: u32,
    scope_id: String,
    target_set_kind: TargetSetKindV2,
    effect_ordinal: u16,
    application_tag_sha256: String,
    expected_identity_sha256: String,
    raw_os_status: i64,
    classification: String,
    exact_absence_observed: bool,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct ExactGenericPasswordDeleteEvidenceV2 {
    schema_owner: String,
    schema_version: u32,
    scope_id: String,
    target_set_kind: TargetSetKindV2,
    effect_ordinal: u16,
    expected_identity_sha256: String,
    raw_os_status: i64,
    classification: String,
    exact_absence_observed: bool,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct ExactFilesystemDeleteEvidenceV2 {
    schema_owner: String,
    schema_version: u32,
    scope_id: String,
    target_set_kind: TargetSetKindV2,
    effect_ordinal: u16,
    path: String,
    expected_identity_sha256: String,
    classification: String,
    parent_fsynced: bool,
    parent_reopened_same_identity: bool,
    exact_absence_reobserved: bool,
}

pub struct MacNativeEffects {
    security: SecurityUiDenied,
    receipt: PublisherPreRemovalReceiptV2,
    signer_access_control_sha256: String,
    current_lock_ordinal: u16,
    held_current_lock: Option<HeldFile>,
    held_terminal_latch: Option<HeldFile>,
    held_resource_locks: BTreeMap<u16, HeldFile>,
    protected_by_lock: BTreeMap<u16, u16>,
    retained_signers: BTreeMap<u16, RetainedExactSystemKey>,
    retained_generic_passwords: BTreeMap<u16, RetainedExactGenericPassword>,
    capability_before:
        BTreeMap<DisposableCapabilityControlV2, CanonicalDisposableCapabilityStateV2>,
    capability_invocations:
        BTreeMap<DisposableCapabilityControlV2, CanonicalDisposableCapabilityInvocationV2>,
    invocation_evidence: BTreeMap<u16, Vec<u8>>,
}

struct HeldFile {
    path: PathBuf,
    file: File,
}

impl MacNativeEffects {
    pub fn new(
        security: SecurityUiDenied,
        current_lock_path: &Path,
        terminal_latch_path: &Path,
        receipt: &PublisherPreRemovalReceiptV2,
        recovery: Option<&ValidatedRecoveryCursor>,
    ) -> Result<Self> {
        validate_publisher_pre_removal_receipt_v2(receipt)
            .context("validate receipt bound to native effects")?;
        require_lower_sha256(
            &receipt.signer_access_control_sha256,
            "signer access-control digest",
        )?;
        let derived = derive_targets_from_ledger(
            receipt.target_set_kind,
            &receipt.scope_id,
            &receipt.target_ledger,
        )?;
        let completed_effect_ordinal = recovery
            .map(ValidatedRecoveryCursor::completed_effect_ordinal)
            .unwrap_or(0);
        if recovery.is_some_and(|cursor| {
            cursor.effect_count() != u16::try_from(derived.len()).unwrap_or(u16::MAX)
                || completed_effect_ordinal > cursor.effect_count()
        }) {
            bail!("validated recovery cursor differs from the native typed effect plan")
        }
        let pending_invoked = |ordinal: u16| {
            recovery
                .and_then(ValidatedRecoveryCursor::pending)
                .is_some_and(|pending| {
                    pending.ordinal() == ordinal && pending.phase() == RecoveryEffectPhase::Invoked
                })
        };
        let expected_for = |ordinal: u16| -> Result<&str> {
            receipt
                .target_ledger
                .iter()
                .find(|identity| identity.ordinal == ordinal)
                .map(|identity| identity.expected_before_sha256.as_str())
                .context("derived lock target lacks its signed expected-before identity")
        };
        let current_target = derived
            .iter()
            .find(|target| target.role == HostTargetRoleV2::CurrentAnchorLock)
            .context("typed target plan lacks current-anchor lock")?;
        let FixedTarget::DerivedFile {
            path: derived_current,
        } = &current_target.target
        else {
            bail!("typed current-anchor lock is not an exact derived file")
        };
        if Path::new(derived_current) != current_lock_path {
            bail!("constructor current-anchor lock path differs from typed target")
        }
        let current_lock = if current_target.ordinal <= completed_effect_ordinal {
            None
        } else {
            open_and_lock_target_maybe(
                current_lock_path,
                expected_for(current_target.ordinal)?,
                "current-anchor lock",
                pending_invoked(current_target.ordinal),
            )?
        };

        let terminal_target = derived
            .iter()
            .find(|target| target.role == HostTargetRoleV2::TerminalLatch)
            .context("typed target plan lacks terminal latch")?;
        let FixedTarget::DerivedFile {
            path: derived_latch,
        } = &terminal_target.target
        else {
            bail!("typed terminal latch is not an exact derived file")
        };
        if Path::new(derived_latch) != terminal_latch_path {
            bail!("constructor terminal latch path differs from typed target")
        }
        let terminal_latch = if terminal_target.ordinal <= completed_effect_ordinal {
            None
        } else {
            open_and_lock_target_maybe(
                terminal_latch_path,
                expected_for(terminal_target.ordinal)?,
                "terminal latch",
                pending_invoked(terminal_target.ordinal),
            )?
        };

        let mut held_resource_locks = BTreeMap::new();
        for target in derived
            .iter()
            .filter(|target| target.role == HostTargetRoleV2::ResourceLock)
        {
            if target.ordinal <= completed_effect_ordinal {
                continue;
            }
            let FixedTarget::DerivedFile { path } = &target.target else {
                bail!("typed resource lock is not an exact derived file")
            };
            let Some(held) = open_and_lock_target_maybe(
                Path::new(path),
                expected_for(target.ordinal)?,
                "typed resource lock",
                pending_invoked(target.ordinal),
            )?
            else {
                continue;
            };
            if held_resource_locks.insert(target.ordinal, held).is_some() {
                bail!("typed target plan duplicates a resource-lock ordinal")
            }
        }
        let protected_by_lock = protected_target_lock_map(receipt)?;
        Ok(Self {
            security,
            receipt: receipt.clone(),
            signer_access_control_sha256: receipt.signer_access_control_sha256.clone(),
            current_lock_ordinal: current_target.ordinal,
            held_current_lock: current_lock,
            held_terminal_latch: terminal_latch,
            held_resource_locks,
            protected_by_lock,
            retained_signers: BTreeMap::new(),
            retained_generic_passwords: BTreeMap::new(),
            capability_before: BTreeMap::new(),
            capability_invocations: BTreeMap::new(),
            invocation_evidence: BTreeMap::new(),
        })
    }

    pub fn current_lock_identity_sha256(&self) -> Result<Option<String>> {
        self.held_current_lock
            .as_ref()
            .map(held_entry_observation)
            .transpose()
    }

    pub fn terminal_latch_identity_sha256(&self) -> Result<Option<String>> {
        self.held_terminal_latch
            .as_ref()
            .map(held_entry_observation)
            .transpose()
    }

    /// Read the one receipt-derived prospective retirement CAS item while every required lock is
    /// already held. The caller cannot supply a service, account, locator, or path.
    pub fn read_receipt_bound_protected_cas_data(&self) -> Result<Option<Vec<u8>>> {
        if self.receipt.target_set_kind != TargetSetKindV2::ProspectiveHost {
            bail!("protected generic-password CAS read crossed into the disposable lane")
        }
        let derived = derive_targets_from_ledger(
            self.receipt.target_set_kind,
            &self.receipt.scope_id,
            &self.receipt.target_ledger,
        )?;
        let mut matches = derived.iter().filter(|target| {
            target.locator
                == (HostResourceLocatorV2::GenericPassword {
                    role: GenericPasswordRoleV2::RetirementCas,
                })
        });
        let target = matches
            .next()
            .context("prospective receipt lacks its exact retirement CAS locator")?;
        if matches.next().is_some() {
            bail!("prospective receipt has multiple retirement CAS locators")
        }
        self.require_generic_password_lock(target.ordinal)?;
        let FixedTarget::ProtectedGenericPassword { service, account } = &target.target else {
            bail!("retirement CAS locator did not expand to a protected generic password")
        };
        self.security
            .read_exact_system_generic_password_data(service, account)
    }

    /// Read the one disposable protected-CAS wrapper only while its receipt-derived current lock
    /// is held and its physical inode identity still exact-matches the signed target ledger.
    pub fn read_receipt_bound_disposable_wrapper_data(&self) -> Result<Vec<u8>> {
        if self.receipt.target_set_kind != TargetSetKindV2::DisposableCapability {
            bail!("disposable protected-wrapper read crossed into the prospective lane")
        }
        let derived = derive_targets_from_ledger(
            self.receipt.target_set_kind,
            &self.receipt.scope_id,
            &self.receipt.target_ledger,
        )?;
        let mut matches = derived
            .iter()
            .filter(|target| target.locator == HostResourceLocatorV2::DisposableProtectedWrapper);
        let target = matches
            .next()
            .context("disposable receipt lacks its exact protected wrapper")?;
        if matches.next().is_some() {
            bail!("disposable receipt has multiple protected wrappers")
        }
        self.require_protecting_lock(target.ordinal)?;
        let expected = self
            .receipt
            .target_ledger
            .iter()
            .find(|identity| identity.ordinal == target.ordinal)
            .context("disposable protected wrapper lacks its signed target identity")?;
        let FixedTarget::DerivedFile { path } = &target.target else {
            bail!("disposable protected wrapper did not expand to one derived file")
        };
        let path = Path::new(path);
        let (_parent, file) = open_exact_file_at(path, false)?;
        let before = disposable_wrapper_observation_from_handle(path, &file)?;
        if before != expected.expected_before_sha256 {
            bail!("live disposable protected-wrapper identity differs before acceptance")
        }
        let mut reader = file
            .try_clone()
            .context("clone exact disposable wrapper for protected-CAS read")?;
        reader
            .seek(SeekFrom::Start(0))
            .context("rewind exact disposable protected wrapper")?;
        let size = reader
            .metadata()
            .context("inspect disposable protected-wrapper size")?
            .len();
        if size == 0 || size > MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2 as u64 {
            bail!("disposable protected-wrapper bytes are outside the fixed bound")
        }
        let mut bytes = Vec::with_capacity(size as usize);
        reader
            .read_to_end(&mut bytes)
            .context("read exact disposable protected-wrapper bytes")?;
        if bytes.len() != size as usize
            || disposable_wrapper_observation_from_handle(path, &file)? != before
        {
            bail!("disposable protected wrapper changed during preacceptance read")
        }
        Ok(bytes)
    }

    fn require_protecting_lock(&self, target_ordinal: u16) -> Result<()> {
        let Some(lock_ordinal) = self.protected_by_lock.get(&target_ordinal).copied() else {
            return Ok(());
        };
        let held = if lock_ordinal == self.current_lock_ordinal {
            self.held_current_lock.is_some()
        } else {
            self.held_resource_locks.contains_key(&lock_ordinal)
        };
        if !held {
            bail!("protected target inspection outlived its exact held lock")
        }
        Ok(())
    }

    fn require_generic_password_lock(&self, target_ordinal: u16) -> Result<()> {
        if !self.protected_by_lock.contains_key(&target_ordinal) {
            bail!("protected generic-password target lacks an exact held CAS lock")
        }
        self.require_protecting_lock(target_ordinal)
    }

    fn observe_disposable_control(
        &mut self,
        effect_ordinal: u16,
        control: DisposableCapabilityControlV2,
        expected_before_sha256: &str,
        phase: EffectObservationPhase,
    ) -> Result<EffectObservation> {
        if self.receipt.target_set_kind != TargetSetKindV2::DisposableCapability {
            bail!("disposable capability control crossed into the prospective lane")
        }
        let current = self
            .security
            .observe_disposable_capability_pre_probe(&self.receipt)?;
        if current.sha256 != expected_before_sha256 {
            return Ok(EffectObservation {
                class: ObservationClass::Ambiguous,
                observation_sha256: current.sha256,
            });
        }
        if phase == EffectObservationPhase::Prepared {
            self.capability_before.insert(control, current.clone());
            return Ok(EffectObservation {
                class: ObservationClass::ExactBefore,
                observation_sha256: current.sha256,
            });
        }

        let after = self
            .security
            .observe_after_disposable_capability_control(&self.receipt, control)?;
        if after.observation.state != current.observation {
            return Ok(EffectObservation {
                class: ObservationClass::Ambiguous,
                observation_sha256: after.sha256,
            });
        }
        if let Some(before) = self.capability_before.get(&control) {
            before.require_same_keys(&current)?;
        }
        let cached_invocation = self.capability_invocations.get(&control).cloned();
        let cached_denial = cached_invocation.as_ref().is_some_and(|invocation| {
            invocation.invocation.control == control
                && invocation.invocation.scope_id == self.receipt.scope_id
                && invocation.invocation.target_set_kind == TargetSetKindV2::DisposableCapability
        });
        let class = classify_disposable_control_recovery(
            control,
            cached_denial,
            after.observation.recovery_class
                == DisposableRecoveryObservationClassV2::DeniedAndPreserved,
        );
        if class == ObservationClass::ExactFinal {
            if let Some(invocation) = cached_invocation {
                let evidence = canonical_bytes_v2(&DisposableControlEvidenceV2 {
                    schema_owner: NATIVE_EFFECT_EVIDENCE_OWNER.to_owned(),
                    schema_version: NATIVE_EFFECT_EVIDENCE_VERSION,
                    invocation,
                    after_observation: after.clone(),
                })?;
                self.record_invocation_evidence(effect_ordinal, evidence)?;
            }
        }
        Ok(EffectObservation {
            class,
            observation_sha256: after.sha256,
        })
    }

    fn invoke_disposable_control(
        &mut self,
        control: DisposableCapabilityControlV2,
        expected_before_sha256: &str,
    ) -> Result<()> {
        if self.receipt.target_set_kind != TargetSetKindV2::DisposableCapability {
            bail!("disposable capability invocation crossed into the prospective lane")
        }
        let before = self
            .capability_before
            .get(&control)
            .context("capability invocation lacks its same-process exact pre-observation")?;
        if before.sha256 != expected_before_sha256 {
            bail!("capability invocation expected-before state changed")
        }
        let invocation = self
            .security
            .invoke_one_disposable_capability_control(&self.receipt, control)?;
        if self
            .capability_invocations
            .insert(control, invocation)
            .is_some()
        {
            bail!("capability control was invoked more than once in one process")
        }
        Ok(())
    }

    fn record_invocation_evidence(&mut self, ordinal: u16, evidence: Vec<u8>) -> Result<()> {
        if ordinal == 0 || evidence.is_empty() {
            bail!("native invocation evidence identity is invalid")
        }
        let _: serde_json::Value =
            substrate_common::macos_retirement_v2::parse_canonical_v2(&evidence)
                .context("validate canonical native invocation evidence")?;
        if let Some(existing) = self.invocation_evidence.get(&ordinal) {
            if existing != &evidence {
                bail!("native invocation evidence changed for one effect ordinal")
            }
            return Ok(());
        }
        self.invocation_evidence.insert(ordinal, evidence);
        Ok(())
    }

    fn gate_filesystem_observation(
        &self,
        ordinal: u16,
        phase: EffectObservationPhase,
        mut observation: EffectObservation,
    ) -> EffectObservation {
        if observation.class == ObservationClass::ExactFinal {
            observation.class = classify_filesystem_absence_recovery(
                phase,
                self.invocation_evidence.contains_key(&ordinal),
            );
        }
        observation
    }

    fn record_filesystem_delete_evidence(
        &mut self,
        target: &DerivedTarget,
        path: &Path,
        expected_before_sha256: &str,
    ) -> Result<()> {
        let path = path
            .to_str()
            .context("filesystem effect path is not UTF-8")?;
        let evidence = canonical_bytes_v2(&ExactFilesystemDeleteEvidenceV2 {
            schema_owner: NATIVE_EFFECT_EVIDENCE_OWNER.to_owned(),
            schema_version: NATIVE_EFFECT_EVIDENCE_VERSION,
            scope_id: self.receipt.scope_id.clone(),
            target_set_kind: self.receipt.target_set_kind,
            effect_ordinal: target.ordinal,
            path: path.to_owned(),
            expected_identity_sha256: expected_before_sha256.to_owned(),
            classification: "durable_parent_fsync_reopen_exact_absence".to_owned(),
            parent_fsynced: true,
            parent_reopened_same_identity: true,
            exact_absence_reobserved: true,
        })?;
        self.record_invocation_evidence(target.ordinal, evidence)
    }
}

impl ClosedEffects for MacNativeEffects {
    fn observe(
        &mut self,
        target: &DerivedTarget,
        expected_before_sha256: &str,
        phase: EffectObservationPhase,
    ) -> Result<EffectObservation> {
        match &target.target {
            FixedTarget::SigningKey { application_tag } => {
                let state = self.security.retain_exact_system_key_identity(
                    application_tag,
                    &self.signer_access_control_sha256,
                    self.receipt.target_set_kind,
                )?;
                let (class, observation_sha256) = match state {
                    Some(retained) => {
                        let identity_sha256 = retained.identity().identity_sha256.clone();
                        let class = if identity_sha256 == expected_before_sha256 {
                            self.retained_signers.insert(target.ordinal, retained);
                            ObservationClass::ExactBefore
                        } else {
                            ObservationClass::Ambiguous
                        };
                        (class, identity_sha256)
                    }
                    None => (
                        classify_signer_absence_recovery(
                            phase,
                            self.invocation_evidence.contains_key(&target.ordinal),
                        ),
                        key_absent_observation(application_tag),
                    ),
                };
                Ok(EffectObservation {
                    class,
                    observation_sha256,
                })
            }
            FixedTarget::DisposableCapabilityControl { control } => self
                .observe_disposable_control(
                    target.ordinal,
                    *control,
                    expected_before_sha256,
                    phase,
                ),
            FixedTarget::LaunchdService { label } => {
                let state = launchd_service_state(label)?;
                let (class, observation_sha256) = match state {
                    LaunchctlPrintClass::NotFound => (
                        ObservationClass::ExactFinal,
                        service_absent_observation(label),
                    ),
                    LaunchctlPrintClass::Present => {
                        let observed = service_present_observation(label);
                        (
                            if phase == EffectObservationPhase::Prepared
                                && observed == expected_before_sha256
                            {
                                ObservationClass::ExactBefore
                            } else {
                                ObservationClass::Ambiguous
                            },
                            observed,
                        )
                    }
                    LaunchctlPrintClass::Ambiguous(code) => (
                        ObservationClass::Ambiguous,
                        service_ambiguous_observation(label, code),
                    ),
                };
                Ok(EffectObservation {
                    class,
                    observation_sha256,
                })
            }
            FixedTarget::MachEndpoint { name } => {
                let state = mach_endpoint_state(name)?;
                let (class, observation_sha256) = match state {
                    MachEndpointClass::NotFound => (
                        ObservationClass::ExactFinal,
                        endpoint_absent_observation(name),
                    ),
                    MachEndpointClass::Present => {
                        let observed = endpoint_present_observation(name);
                        (
                            if observed == expected_before_sha256 {
                                ObservationClass::ExactBefore
                            } else {
                                ObservationClass::Ambiguous
                            },
                            observed,
                        )
                    }
                    MachEndpointClass::Ambiguous(code) => (
                        ObservationClass::Ambiguous,
                        endpoint_ambiguous_observation(name, code),
                    ),
                };
                Ok(EffectObservation {
                    class,
                    observation_sha256,
                })
            }
            FixedTarget::File { path } => Ok(self.gate_filesystem_observation(
                target.ordinal,
                phase,
                observe_file(Path::new(path), expected_before_sha256)?,
            )),
            FixedTarget::DerivedFile { path } => {
                let path = Path::new(path);
                if target.locator == HostResourceLocatorV2::DisposableProtectedWrapper {
                    self.require_protecting_lock(target.ordinal)?;
                    return Ok(self.gate_filesystem_observation(
                        target.ordinal,
                        phase,
                        observe_disposable_wrapper(path, expected_before_sha256)?,
                    ));
                }
                if target.role == HostTargetRoleV2::TerminalLatch {
                    if let Some(held) = &self.held_terminal_latch {
                        if held.path != path {
                            bail!("held terminal latch path differs from closed target")
                        }
                        let observed = held_entry_observation(held)?;
                        return Ok(EffectObservation {
                            class: if observed == expected_before_sha256 {
                                ObservationClass::ExactBefore
                            } else {
                                ObservationClass::Ambiguous
                            },
                            observation_sha256: observed,
                        });
                    }
                    return Ok(self.gate_filesystem_observation(
                        target.ordinal,
                        phase,
                        observe_unheld_invoked_lock(path, expected_before_sha256)?,
                    ));
                }
                if target.role == HostTargetRoleV2::CurrentAnchorLock {
                    if let Some(held) = &self.held_current_lock {
                        if held.path != path {
                            bail!("held current-anchor lock path differs from closed target")
                        }
                        let observed = held_entry_observation(held)?;
                        return Ok(EffectObservation {
                            class: if observed == expected_before_sha256 {
                                ObservationClass::ExactBefore
                            } else {
                                ObservationClass::Ambiguous
                            },
                            observation_sha256: observed,
                        });
                    }
                    return Ok(self.gate_filesystem_observation(
                        target.ordinal,
                        phase,
                        observe_unheld_invoked_lock(path, expected_before_sha256)?,
                    ));
                }
                if target.role == HostTargetRoleV2::ResourceLock {
                    if let Some(held) = self.held_resource_locks.get(&target.ordinal) {
                        if held.path != path {
                            bail!("held resource lock path differs from closed target")
                        }
                        let observed = held_entry_observation(held)?;
                        return Ok(EffectObservation {
                            class: if observed == expected_before_sha256 {
                                ObservationClass::ExactBefore
                            } else {
                                ObservationClass::Ambiguous
                            },
                            observation_sha256: observed,
                        });
                    }
                    return Ok(self.gate_filesystem_observation(
                        target.ordinal,
                        phase,
                        observe_unheld_invoked_lock(path, expected_before_sha256)?,
                    ));
                }
                self.require_protecting_lock(target.ordinal)?;
                Ok(self.gate_filesystem_observation(
                    target.ordinal,
                    phase,
                    observe_file(path, expected_before_sha256)?,
                ))
            }
            FixedTarget::DerivedDirectory { path } => Ok(self.gate_filesystem_observation(
                target.ordinal,
                phase,
                observe_directory(Path::new(path), expected_before_sha256)?,
            )),
            FixedTarget::ProtectedGenericPassword { service, account } => {
                self.require_generic_password_lock(target.ordinal)?;
                let state = self
                    .security
                    .retain_exact_generic_password_identity(service, account)?;
                let (class, observation_sha256) = match state {
                    Some(retained) => {
                        let identity_sha256 = retained.identity().identity_sha256.clone();
                        let class = if identity_sha256 == expected_before_sha256 {
                            self.retained_generic_passwords
                                .insert(target.ordinal, retained);
                            ObservationClass::ExactBefore
                        } else {
                            ObservationClass::Ambiguous
                        };
                        (class, identity_sha256)
                    }
                    None => (
                        ObservationClass::ExactFinal,
                        generic_password_absent_observation(service, account),
                    ),
                };
                Ok(EffectObservation {
                    class,
                    observation_sha256,
                })
            }
        }
    }

    fn invoke(&mut self, target: &DerivedTarget, expected_before_sha256: &str) -> Result<()> {
        match &target.target {
            FixedTarget::SigningKey { application_tag } => {
                if self.held_current_lock.is_none() || self.held_terminal_latch.is_none() {
                    bail!("signer deletion requires the current lock and terminal latch held")
                }
                let retained = self
                    .retained_signers
                    .remove(&target.ordinal)
                    .context("signer invocation lacks its retained exact pre-observation")?;
                if retained.identity().application_tag_sha256 != sha256_hex_v2(application_tag) {
                    bail!("retained signer tag differs from the typed target")
                }
                match self
                    .security
                    .delete_retained_exact_system_key(retained, expected_before_sha256)?
                {
                    ExactKeyDeleteOutcome::DeletedAndAbsent => {
                        let evidence = canonical_bytes_v2(&ExactKeyDeleteEvidenceV2 {
                            schema_owner: NATIVE_EFFECT_EVIDENCE_OWNER.to_owned(),
                            schema_version: NATIVE_EFFECT_EVIDENCE_VERSION,
                            scope_id: self.receipt.scope_id.clone(),
                            target_set_kind: self.receipt.target_set_kind,
                            effect_ordinal: target.ordinal,
                            application_tag_sha256: sha256_hex_v2(application_tag),
                            expected_identity_sha256: expected_before_sha256.to_owned(),
                            raw_os_status: 0,
                            classification: "deleted_and_exact_absence_observed".to_owned(),
                            exact_absence_observed: true,
                        })?;
                        self.record_invocation_evidence(target.ordinal, evidence)
                    }
                    ExactKeyDeleteOutcome::AlreadyAbsent => {
                        bail!("retained exact signer unexpectedly reported already absent")
                    }
                }
            }
            FixedTarget::DisposableCapabilityControl { control } => {
                self.invoke_disposable_control(*control, expected_before_sha256)
            }
            FixedTarget::LaunchdService { label } => {
                if service_present_observation(label) != expected_before_sha256 {
                    bail!("launchd expected-before identity changed before invocation")
                }
                let status = Command::new("/bin/launchctl")
                    .args(["bootout", label])
                    .env_clear()
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .context("invoke fixed publisher launchd bootout")?;
                match launchd_service_state(label)? {
                    LaunchctlPrintClass::NotFound => Ok(()),
                    LaunchctlPrintClass::Present => {
                        bail!("fixed publisher launchd service remains after bootout")
                    }
                    LaunchctlPrintClass::Ambiguous(code) => bail!(
                        "fixed publisher launchd post-bootout state is ambiguous ({code:?}); bootout success={}",
                        status.success()
                    ),
                }
            }
            FixedTarget::MachEndpoint { name } => match mach_endpoint_state(name)? {
                MachEndpointClass::NotFound => Ok(()),
                MachEndpointClass::Present => {
                    bail!("fixed publisher Mach endpoint survives service bootout")
                }
                MachEndpointClass::Ambiguous(code) => {
                    bail!("fixed publisher Mach endpoint state is ambiguous ({code})")
                }
            },
            FixedTarget::File { path } => {
                let path = Path::new(path);
                remove_exact_file(path, expected_before_sha256)?;
                self.record_filesystem_delete_evidence(target, path, expected_before_sha256)
            }
            FixedTarget::DerivedFile { path } => {
                let path = Path::new(path);
                if target.locator == HostResourceLocatorV2::DisposableProtectedWrapper {
                    self.require_protecting_lock(target.ordinal)?;
                    remove_exact_disposable_wrapper(path, expected_before_sha256)?;
                    return self.record_filesystem_delete_evidence(
                        target,
                        path,
                        expected_before_sha256,
                    );
                }
                if target.role == HostTargetRoleV2::CurrentAnchorLock {
                    let held = self
                        .held_current_lock
                        .take()
                        .context("current-anchor lock was not held through signer absence")?;
                    if held.path != path {
                        self.held_current_lock = Some(held);
                        bail!("held current-anchor lock path differs from compiled target")
                    }
                    if let Err(error) = remove_held_exact_file(&held, expected_before_sha256) {
                        self.held_current_lock = Some(held);
                        return Err(error);
                    }
                    drop(held);
                    return self.record_filesystem_delete_evidence(
                        target,
                        path,
                        expected_before_sha256,
                    );
                }
                if target.role == HostTargetRoleV2::TerminalLatch {
                    let held = self
                        .held_terminal_latch
                        .take()
                        .context("terminal latch was not retained through its removal ordinal")?;
                    if held.path != path {
                        self.held_terminal_latch = Some(held);
                        bail!("held terminal latch path differs from closed target")
                    }
                    if let Err(error) = remove_held_exact_file(&held, expected_before_sha256) {
                        self.held_terminal_latch = Some(held);
                        return Err(error);
                    }
                    drop(held);
                    return self.record_filesystem_delete_evidence(
                        target,
                        path,
                        expected_before_sha256,
                    );
                }
                if target.role == HostTargetRoleV2::ResourceLock {
                    let held = self
                        .held_resource_locks
                        .remove(&target.ordinal)
                        .context("resource lock was not held through its protected target")?;
                    if held.path != path {
                        self.held_resource_locks.insert(target.ordinal, held);
                        bail!("held resource lock path differs from typed target")
                    }
                    if let Err(error) = remove_held_exact_file(&held, expected_before_sha256) {
                        self.held_resource_locks.insert(target.ordinal, held);
                        return Err(error);
                    }
                    drop(held);
                    return self.record_filesystem_delete_evidence(
                        target,
                        path,
                        expected_before_sha256,
                    );
                }
                self.require_protecting_lock(target.ordinal)?;
                remove_exact_file(path, expected_before_sha256)?;
                self.record_filesystem_delete_evidence(target, path, expected_before_sha256)
            }
            FixedTarget::DerivedDirectory { path } => {
                let path = Path::new(path);
                remove_exact_directory(path, expected_before_sha256)?;
                self.record_filesystem_delete_evidence(target, path, expected_before_sha256)
            }
            FixedTarget::ProtectedGenericPassword { .. } => {
                self.require_generic_password_lock(target.ordinal)?;
                let retained = self
                    .retained_generic_passwords
                    .remove(&target.ordinal)
                    .context("generic-password invocation lacks retained exact pre-observation")?;
                match self
                    .security
                    .delete_retained_exact_generic_password(retained, expected_before_sha256)?
                {
                    ExactGenericPasswordDeleteOutcome::DeletedAndAbsent => {
                        let evidence = canonical_bytes_v2(&ExactGenericPasswordDeleteEvidenceV2 {
                            schema_owner: NATIVE_EFFECT_EVIDENCE_OWNER.to_owned(),
                            schema_version: NATIVE_EFFECT_EVIDENCE_VERSION,
                            scope_id: self.receipt.scope_id.clone(),
                            target_set_kind: self.receipt.target_set_kind,
                            effect_ordinal: target.ordinal,
                            expected_identity_sha256: expected_before_sha256.to_owned(),
                            raw_os_status: 0,
                            classification: "deleted_and_exact_absence_observed".to_owned(),
                            exact_absence_observed: true,
                        })?;
                        self.record_invocation_evidence(target.ordinal, evidence)
                    }
                }
            }
        }
    }

    fn invocation_evidence(&self, target: &DerivedTarget) -> Result<Option<Vec<u8>>> {
        Ok(self.invocation_evidence.get(&target.ordinal).cloned())
    }
}

fn open_and_lock_target_maybe(
    path: &Path,
    expected_before_sha256: &str,
    label: &str,
    allow_absent: bool,
) -> Result<Option<HeldFile>> {
    if allow_absent {
        let parent = open_exact_parent(path)?;
        let leaf = exact_leaf_cstring(path)?;
        if try_fstatat_nofollow(&parent, &leaf)?.is_none() {
            return Ok(None);
        }
    }
    let (_, file) = open_exact_file_at(path, true)
        .with_context(|| format!("open exact {label} before finalizer acceptance"))?;
    if file_observation_from_handle(path, &file)? != expected_before_sha256 {
        bail!("{label} identity differs before finalizer acceptance")
    }
    // SAFETY: file is a live exact no-follow descriptor and LOCK_EX is a scalar operation.
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } != 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("acquire exact {label}"));
    }
    Ok(Some(HeldFile {
        path: path.to_path_buf(),
        file,
    }))
}

fn protected_target_lock_map(receipt: &PublisherPreRemovalReceiptV2) -> Result<BTreeMap<u16, u16>> {
    let mut result = BTreeMap::new();
    for target in &receipt.target_ledger {
        let expected_lock = match &target.locator {
            HostResourceLocatorV2::GenericPassword { role } => Some(
                receipt
                    .target_ledger
                    .iter()
                    .find(|candidate| {
                        matches!(
                            &candidate.locator,
                            HostResourceLocatorV2::KeychainCasLock { account }
                                if account == role
                        )
                    })
                    .context("protected generic-password target lacks its exact CAS lock")?,
            ),
            HostResourceLocatorV2::LifecycleFile { role } => {
                receipt.target_ledger.iter().find(|candidate| {
                    matches!(
                        &candidate.locator,
                        HostResourceLocatorV2::LifecycleTargetLock { target }
                            if target == role
                    )
                })
            }
            HostResourceLocatorV2::DisposableProtectedWrapper => Some(
                receipt
                    .target_ledger
                    .iter()
                    .find(|candidate| {
                        candidate.locator == HostResourceLocatorV2::DisposableCurrentLock
                    })
                    .context("disposable protected wrapper lacks its current lock")?,
            ),
            _ => None,
        };
        if let Some(lock) = expected_lock {
            if !matches!(
                lock.role,
                HostTargetRoleV2::CurrentAnchorLock | HostTargetRoleV2::ResourceLock
            ) || result.insert(target.ordinal, lock.ordinal).is_some()
            {
                bail!("protected target lock join is ambiguous")
            }
        }
    }
    Ok(result)
}

pub fn file_observation(path: &Path) -> Result<String> {
    let (_parent, file) = open_exact_file_at(path, false)?;
    file_observation_from_handle(path, &file)
}

fn file_observation_from_handle(path: &Path, file: &File) -> Result<String> {
    let mut reader = file
        .try_clone()
        .context("clone exact target for measurement")?;
    reader
        .seek(SeekFrom::Start(0))
        .context("rewind exact target for measurement")?;
    let before = reader.metadata().context("inspect exact target")?;
    validate_root_file(&reader, "exact target")?;
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .context("read exact target")?;
    let after = reader.metadata().context("reinspect exact target")?;
    if before.dev() != after.dev()
        || before.ino() != after.ino()
        || before.len() != after.len()
        || before.mtime() != after.mtime()
        || before.mtime_nsec() != after.mtime_nsec()
    {
        bail!("exact target changed while measuring")
    }
    let path = path.to_str().context("exact target path is not UTF-8")?;
    sha256_canonical(&FileObservation {
        path,
        device: before.dev(),
        inode: before.ino(),
        uid: before.uid(),
        gid: before.gid(),
        mode: before.mode(),
        link_count: before.nlink(),
        size: before.len(),
        content_sha256: sha256_hex_v2(&bytes),
    })
}

fn observe_file(path: &Path, expected_before_sha256: &str) -> Result<EffectObservation> {
    let parent = open_exact_parent(path)?;
    let leaf = exact_leaf_cstring(path)?;
    match try_fstatat_nofollow(&parent, &leaf)? {
        None => Ok(EffectObservation {
            class: ObservationClass::ExactFinal,
            observation_sha256: absent_path_observation(path),
        }),
        Some(_) => match file_observation(path) {
            Ok(observed) if observed == expected_before_sha256 => Ok(EffectObservation {
                class: ObservationClass::ExactBefore,
                observation_sha256: observed,
            }),
            Ok(observed) => Ok(EffectObservation {
                class: ObservationClass::Ambiguous,
                observation_sha256: observed,
            }),
            Err(_) => Ok(EffectObservation {
                class: ObservationClass::Ambiguous,
                observation_sha256: sha256_hex_v2(b"ambiguous-file-observation"),
            }),
        },
    }
}

fn disposable_wrapper_observation_from_handle(path: &Path, file: &File) -> Result<String> {
    let before = file.metadata().context("inspect disposable CAS wrapper")?;
    validate_root_file(file, "disposable CAS wrapper")?;
    if !before.file_type().is_file()
        || before.uid() != 0
        || before.mode() & 0o7777 != 0o400
        || before.nlink() != 1
    {
        bail!("disposable CAS wrapper owner/mode/type/link count differs")
    }
    let after = file
        .metadata()
        .context("reinspect disposable CAS wrapper")?;
    if before.dev() != after.dev()
        || before.ino() != after.ino()
        || before.uid() != after.uid()
        || before.gid() != after.gid()
        || before.mode() != after.mode()
        || before.nlink() != after.nlink()
    {
        bail!("disposable CAS wrapper physical identity changed while observing")
    }
    let path = path
        .to_str()
        .context("disposable CAS wrapper path is not UTF-8")?;
    sha256_canonical(&DisposableWrapperObservation {
        path,
        device: before.dev(),
        inode: before.ino(),
        uid: before.uid(),
        gid: before.gid(),
        mode: before.mode(),
        link_count: before.nlink(),
    })
}

fn observe_disposable_wrapper(
    path: &Path,
    expected_before_sha256: &str,
) -> Result<EffectObservation> {
    let parent = open_exact_parent(path)?;
    let leaf = exact_leaf_cstring(path)?;
    if try_fstatat_nofollow(&parent, &leaf)?.is_none() {
        return Ok(EffectObservation {
            class: ObservationClass::ExactFinal,
            observation_sha256: absent_path_observation(path),
        });
    }
    let (_parent, file) = open_exact_file_at(path, false)?;
    let observed = disposable_wrapper_observation_from_handle(path, &file)?;
    Ok(EffectObservation {
        class: if observed == expected_before_sha256 {
            ObservationClass::ExactBefore
        } else {
            ObservationClass::Ambiguous
        },
        observation_sha256: observed,
    })
}

fn remove_exact_disposable_wrapper(path: &Path, expected_before_sha256: &str) -> Result<()> {
    let (parent, file) = open_exact_file_at(path, false)?;
    let observed = disposable_wrapper_observation_from_handle(path, &file)?;
    if observed != expected_before_sha256 {
        bail!("disposable CAS wrapper changed before removal")
    }
    unlink_held_entry(&parent, path, &file, 0)
}

fn observe_unheld_invoked_lock(
    path: &Path,
    expected_before_sha256: &str,
) -> Result<EffectObservation> {
    let mut observation = observe_file(path, expected_before_sha256)?;
    if observation.class != ObservationClass::ExactFinal {
        observation.class = ObservationClass::Ambiguous;
    }
    Ok(observation)
}

fn observe_directory(path: &Path, expected_before_sha256: &str) -> Result<EffectObservation> {
    let parent = open_exact_parent(path)?;
    let leaf = exact_leaf_cstring(path)?;
    match try_fstatat_nofollow(&parent, &leaf)? {
        None => Ok(EffectObservation {
            class: ObservationClass::ExactFinal,
            observation_sha256: absent_path_observation(path),
        }),
        Some(stat) => {
            require_stat_kind(&stat, libc::S_IFDIR, "observed exact directory")?;
            let metadata = fs::symlink_metadata(path).context("inspect exact directory target")?;
            let observed = directory_observation(path, &metadata);
            Ok(EffectObservation {
                class: if metadata.file_type().is_dir()
                    && metadata.uid() == 0
                    && observed == expected_before_sha256
                {
                    ObservationClass::ExactBefore
                } else {
                    ObservationClass::Ambiguous
                },
                observation_sha256: observed,
            })
        }
    }
}

fn directory_observation(path: &Path, metadata: &Metadata) -> String {
    sha256_hex_v2(
        format!(
            "directory\0{}\0{}\0{}\0{}\0{}\0{}",
            path.display(),
            metadata.dev(),
            metadata.ino(),
            metadata.uid(),
            metadata.gid(),
            metadata.mode()
        )
        .as_bytes(),
    )
}

fn require_lower_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_digit() && !(b'a'..=b'f').contains(&byte))
    {
        bail!("{label} is not an exact lowercase SHA-256")
    }
    Ok(())
}

fn validate_root_file(file: &File, label: &str) -> Result<()> {
    let metadata = file
        .metadata()
        .with_context(|| format!("inspect {label}"))?;
    if !metadata.file_type().is_file()
        || metadata.uid() != 0
        || metadata.nlink() != 1
        || metadata.mode() & 0o022 != 0
    {
        bail!("{label} is not one immutable-name root-owned regular file")
    }
    Ok(())
}

fn open_exact_parent(path: &Path) -> Result<File> {
    let parent = path.parent().context("exact target lacks parent")?;
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(parent)
        .context("open exact target parent no-follow")?;
    let metadata = file.metadata().context("inspect exact target parent")?;
    if !metadata.file_type().is_dir()
        || metadata.uid() != 0
        || metadata.nlink() < 2
        || metadata.mode() & 0o022 != 0
    {
        bail!("exact target parent is not one retained root-owned directory")
    }
    Ok(file)
}

fn exact_leaf_cstring(path: &Path) -> Result<CString> {
    let leaf = path
        .file_name()
        .context("exact target has no final component")?;
    if leaf.as_bytes().is_empty() || leaf == "." || leaf == ".." || leaf.as_bytes().contains(&b'/')
    {
        bail!("exact target final component is unsafe")
    }
    CString::new(leaf.as_bytes()).context("exact target final component contains NUL")
}

fn fstatat_nofollow(parent: &File, leaf: &CString) -> Result<libc::stat> {
    try_fstatat_nofollow(parent, leaf)?.context("exact target is absent")
}

fn try_fstatat_nofollow(parent: &File, leaf: &CString) -> Result<Option<libc::stat>> {
    let mut stat = MaybeUninit::<libc::stat>::zeroed();
    // SAFETY: parent/leaf are live and stat is writable.
    if unsafe {
        libc::fstatat(
            parent.as_raw_fd(),
            leaf.as_ptr(),
            stat.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    } != 0
    {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ENOENT) {
            return Ok(None);
        }
        return Err(error).context("fstatat exact target no-follow");
    }
    // SAFETY: successful fstatat initialized the complete structure.
    Ok(Some(unsafe { stat.assume_init() }))
}

fn require_stat_kind(stat: &libc::stat, kind: libc::mode_t, label: &str) -> Result<()> {
    if stat.st_mode & libc::S_IFMT != kind
        || stat.st_uid != 0
        || stat.st_nlink == 0
        || (kind == libc::S_IFDIR && stat.st_nlink < 2)
        || stat.st_mode & 0o022 != 0
    {
        bail!("{label} directory entry has unsafe type/owner/mode/link identity")
    }
    Ok(())
}

fn require_same_entry(metadata: &Metadata, stat: &libc::stat, label: &str) -> Result<()> {
    if metadata.dev() != stat.st_dev as u64
        || metadata.ino() != stat.st_ino
        || metadata.uid() != stat.st_uid
        || metadata.gid() != stat.st_gid
        || metadata.mode() != u32::from(stat.st_mode)
        || metadata.nlink() != stat.st_nlink as u64
    {
        bail!("{label} path entry differs from its held descriptor")
    }
    Ok(())
}

fn open_exact_file_at(path: &Path, write: bool) -> Result<(File, File)> {
    let parent = open_exact_parent(path)?;
    let leaf = exact_leaf_cstring(path)?;
    let entry = fstatat_nofollow(&parent, &leaf)?;
    require_stat_kind(&entry, libc::S_IFREG, "exact file")?;
    let access = if write { libc::O_RDWR } else { libc::O_RDONLY };
    // SAFETY: parent/leaf are live; flags prohibit symlink traversal and return a new descriptor.
    let fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            leaf.as_ptr(),
            access | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error()).context("open exact file via parent FD");
    }
    // SAFETY: openat returned a new owned descriptor.
    let file = unsafe { File::from_raw_fd(fd) };
    let metadata = file.metadata().context("inspect opened exact file")?;
    require_same_entry(&metadata, &entry, "exact file")?;
    validate_root_file(&file, "exact file")?;
    Ok((parent, file))
}

fn unlink_held_entry(parent: &File, path: &Path, held: &File, flags: i32) -> Result<()> {
    let leaf = exact_leaf_cstring(path)?;
    let metadata = held.metadata().context("inspect held exact target")?;
    let rechecked = fstatat_nofollow(parent, &leaf)?;
    require_same_entry(&metadata, &rechecked, "rechecked exact target")?;
    // SAFETY: parent/leaf name the rechecked entry; flags are either zero or AT_REMOVEDIR.
    if unsafe { libc::unlinkat(parent.as_raw_fd(), leaf.as_ptr(), flags) } != 0 {
        return Err(std::io::Error::last_os_error()).context("unlink exact target via parent FD");
    }
    require_entry_absent(parent, &leaf)?;
    fsync_reopen_and_confirm_absence(parent, path, &leaf)
}

fn fsync_reopen_and_confirm_absence(parent: &File, path: &Path, leaf: &CString) -> Result<()> {
    let before = parent
        .metadata()
        .context("inspect exact target parent before fsync")?;
    parent.sync_all().context("fsync exact target parent")?;
    let reopened = open_exact_parent(path).context("reopen exact target parent after fsync")?;
    let after = reopened
        .metadata()
        .context("inspect reopened exact target parent")?;
    if before.dev() != after.dev()
        || before.ino() != after.ino()
        || before.uid() != after.uid()
        || before.gid() != after.gid()
        || before.mode() != after.mode()
    {
        bail!("exact target parent changed across fsync/reopen")
    }
    require_entry_absent(&reopened, leaf)
        .context("reobserve exact target absence through reopened durable parent")
}

fn require_entry_absent(parent: &File, leaf: &CString) -> Result<()> {
    let mut stat = MaybeUninit::<libc::stat>::zeroed();
    // SAFETY: parent/leaf are live and stat is writable.
    let result = unsafe {
        libc::fstatat(
            parent.as_raw_fd(),
            leaf.as_ptr(),
            stat.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if result == 0 {
        bail!("exact target remains after unlinkat")
    }
    let error = std::io::Error::last_os_error();
    if error.raw_os_error() != Some(libc::ENOENT) {
        return Err(error).context("prove exact target absence after unlinkat");
    }
    Ok(())
}

fn remove_exact_file(path: &Path, expected_before_sha256: &str) -> Result<()> {
    let (parent, file) = open_exact_file_at(path, false)?;
    if file_observation_from_handle(path, &file)? != expected_before_sha256 {
        bail!("exact file target changed after EffectInvoked was journaled")
    }
    unlink_held_entry(&parent, path, &file, 0)
}

fn remove_held_exact_file(held: &HeldFile, expected_before_sha256: &str) -> Result<()> {
    let parent = open_exact_parent(&held.path)?;
    if file_observation_from_handle(&held.path, &held.file)? != expected_before_sha256 {
        bail!("held exact file changed after EffectInvoked was journaled")
    }
    // The held current-lock descriptor remains flocked until after unlinkat and parent fsync.
    unlink_held_entry(&parent, &held.path, &held.file, 0)
}

fn held_entry_observation(held: &HeldFile) -> Result<String> {
    let parent = open_exact_parent(&held.path)?;
    let leaf = exact_leaf_cstring(&held.path)?;
    let entry = fstatat_nofollow(&parent, &leaf)?;
    let metadata = held.file.metadata().context("inspect held exact file")?;
    require_same_entry(&metadata, &entry, "held exact file")?;
    file_observation_from_handle(&held.path, &held.file)
}

fn remove_exact_directory(path: &Path, expected_before_sha256: &str) -> Result<()> {
    let parent = open_exact_parent(path)?;
    let leaf = exact_leaf_cstring(path)?;
    let before = fstatat_nofollow(&parent, &leaf)?;
    require_stat_kind(&before, libc::S_IFDIR, "exact directory")?;
    let fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            leaf.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error()).context("open exact directory via parent FD");
    }
    // SAFETY: openat returned a new owned descriptor.
    let directory = unsafe { File::from_raw_fd(fd) };
    let metadata = directory
        .metadata()
        .context("inspect opened exact directory")?;
    require_same_entry(&metadata, &before, "exact directory")?;
    let observed = directory_observation(path, &metadata);
    if observed != expected_before_sha256 {
        bail!("exact directory changed after EffectInvoked was journaled")
    }
    let rechecked = fstatat_nofollow(&parent, &leaf)?;
    require_same_entry(&metadata, &rechecked, "rechecked exact directory")?;
    if unsafe { libc::unlinkat(parent.as_raw_fd(), leaf.as_ptr(), libc::AT_REMOVEDIR) } != 0 {
        return Err(std::io::Error::last_os_error())
            .context("unlink exact directory via parent FD");
    }
    require_entry_absent(&parent, &leaf)?;
    fsync_reopen_and_confirm_absence(&parent, path, &leaf)
}

fn launchd_service_state(label: &str) -> Result<LaunchctlPrintClass> {
    if label != PRODUCT_LAUNCHD_SERVICE {
        bail!("launchd observation label is not compiled")
    }
    let status = Command::new("/bin/launchctl")
        .args(["print", label])
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("observe fixed publisher launchd registration")?;
    Ok(classify_launchctl_print(status.success(), status.code()))
}

fn mach_endpoint_state(name: &str) -> Result<MachEndpointClass> {
    if name != PRODUCT_MACH_ENDPOINT {
        bail!("Mach endpoint observation name is not compiled")
    }
    let name = CString::new(name).context("encode fixed Mach endpoint")?;
    let mut service_port = 0_u32;
    // SAFETY: bootstrap_port is a process-lifetime system port; name is NUL-terminated and the
    // output pointer is writable.
    let status = unsafe { bootstrap_look_up(bootstrap_port, name.as_ptr(), &mut service_port) };
    match status {
        BOOTSTRAP_SUCCESS if service_port != 0 => {
            // SAFETY: successful lookup returned one send right owned by this process.
            let release = unsafe { mach_port_deallocate(mach_task_self_, service_port) };
            if release != 0 {
                bail!("release fixed Mach endpoint send right failed with kern_return {release}")
            }
            Ok(MachEndpointClass::Present)
        }
        BOOTSTRAP_SUCCESS => Ok(MachEndpointClass::Ambiguous(status)),
        BOOTSTRAP_UNKNOWN_SERVICE => Ok(MachEndpointClass::NotFound),
        other => Ok(MachEndpointClass::Ambiguous(other)),
    }
}

pub fn key_absent_observation(tag: &[u8]) -> String {
    joined_observation(b"system-key-absent\0", tag)
}

fn generic_password_absent_observation(service: &str, account: &str) -> String {
    joined_observation(
        b"system-generic-password-absent\0",
        format!("{service}\0{account}").as_bytes(),
    )
}

fn service_present_observation(label: &str) -> String {
    joined_observation(b"launchd-service-present\0", label.as_bytes())
}

fn service_absent_observation(label: &str) -> String {
    joined_observation(b"launchd-service-absent\0", label.as_bytes())
}

fn service_ambiguous_observation(label: &str, code: Option<i32>) -> String {
    joined_observation(
        b"launchd-service-ambiguous\0",
        format!("{label}\0{code:?}").as_bytes(),
    )
}

fn endpoint_present_observation(name: &str) -> String {
    joined_observation(b"mach-endpoint-present\0", name.as_bytes())
}

fn endpoint_absent_observation(name: &str) -> String {
    joined_observation(b"mach-endpoint-absent\0", name.as_bytes())
}

fn endpoint_ambiguous_observation(name: &str, code: i32) -> String {
    joined_observation(
        b"mach-endpoint-ambiguous\0",
        format!("{name}\0{code}").as_bytes(),
    )
}

fn absent_path_observation(path: &Path) -> String {
    joined_observation(b"path-absent\0", path.as_os_str().as_encoded_bytes())
}

fn joined_observation(prefix: &[u8], value: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(prefix);
    digest.update(value);
    format!("{:x}", digest.finalize())
}

fn sha256_canonical<T: Serialize>(value: &T) -> Result<String> {
    Ok(sha256_hex_v2(&canonical_bytes_v2(value)?))
}

#[link(name = "System")]
unsafe extern "C" {
    static bootstrap_port: u32;
    static mach_task_self_: u32;
    fn bootstrap_look_up(
        bootstrap_port: u32,
        service_name: *const c_char,
        service: *mut u32,
    ) -> i32;
    fn mach_port_deallocate(task: u32, name: u32) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launchctl_only_exact_not_found_is_absence() {
        assert_eq!(
            classify_launchctl_print(true, Some(0)),
            LaunchctlPrintClass::Present
        );
        assert_eq!(
            classify_launchctl_print(false, Some(113)),
            LaunchctlPrintClass::NotFound
        );
        for code in [None, Some(1), Some(3), Some(5), Some(125)] {
            assert_eq!(
                classify_launchctl_print(false, code),
                LaunchctlPrintClass::Ambiguous(code)
            );
        }
    }

    #[test]
    fn disposable_recovery_requires_cached_or_reconstructable_denial() {
        use DisposableCapabilityControlV2 as C;
        for control in [
            C::Sign,
            C::ExportPrivate,
            C::ReplaceAccess,
            C::DeleteWrongKey,
        ] {
            assert_eq!(
                classify_disposable_control_recovery(control, true, true),
                ObservationClass::ExactFinal
            );
        }
        for control in [
            C::Sign,
            C::ExportPrivate,
            C::ReplaceAccess,
            C::DeleteWrongKey,
        ] {
            assert_eq!(
                classify_disposable_control_recovery(control, false, true),
                ObservationClass::Ambiguous
            );
            assert_eq!(
                classify_disposable_control_recovery(control, false, false),
                ObservationClass::Ambiguous
            );
        }
    }

    #[test]
    fn signer_absence_requires_same_process_successful_delete_witness() {
        assert_eq!(
            classify_signer_absence_recovery(EffectObservationPhase::Invoked, true),
            ObservationClass::ExactFinal
        );
        for (phase, witness) in [
            (EffectObservationPhase::Prepared, false),
            (EffectObservationPhase::Prepared, true),
            (EffectObservationPhase::Invoked, false),
        ] {
            assert_eq!(
                classify_signer_absence_recovery(phase, witness),
                ObservationClass::Ambiguous
            );
        }
    }

    #[test]
    fn filesystem_absence_after_invoked_requires_same_process_durable_parent_witness() {
        assert_eq!(
            classify_filesystem_absence_recovery(EffectObservationPhase::Prepared, false),
            ObservationClass::ExactFinal
        );
        assert_eq!(
            classify_filesystem_absence_recovery(EffectObservationPhase::Invoked, false),
            ObservationClass::Ambiguous
        );
        assert_eq!(
            classify_filesystem_absence_recovery(EffectObservationPhase::Invoked, true),
            ObservationClass::ExactFinal
        );
        let production = include_str!("native_effects.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        assert!(production.contains("parent.sync_all().context(\"fsync exact target parent\")?"));
        assert!(production.contains("reopen exact target parent after fsync"));
        assert!(
            production.contains("reobserve exact target absence through reopened durable parent")
        );
        assert!(production.contains("record_filesystem_delete_evidence"));
    }

    #[test]
    fn invoked_launchd_service_presence_never_authorizes_attempt_two() {
        let production = include_str!("native_effects.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        assert!(production.contains(
            "phase == EffectObservationPhase::Prepared\n                                && observed == expected_before_sha256"
        ));
        assert!(!production.contains(
            "phase == EffectObservationPhase::Invoked\n                                && observed == expected_before_sha256"
        ));
    }

    #[test]
    fn native_surface_holds_exact_locks_and_has_no_legacy_targets() {
        let source = include_str!("native_effects.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        assert!(production.contains("protected_target_lock_map(receipt)"));
        assert!(production.contains("libc::flock"));
        assert!(production.contains("retained_signers"));
        assert!(production.contains("retained_generic_passwords"));
        assert!(!production.contains("FixedTarget::ScopeFile"));
        assert!(!production.contains("FixedTarget::ScopeDirectory"));
        assert!(!production.contains("BLOCKED_SCHEMA"));
    }
}
