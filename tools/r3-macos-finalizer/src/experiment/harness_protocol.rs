//! Append-only causal journal schema for the UID-501 disposable harness.

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use substrate_common::macos_retirement_v2::{
    document_sha256_v2, sha256_hex_v2, MAC_R3_COORDINATOR_INBOX_ROOT_V2,
    MAC_R3_COORDINATOR_PATH_V2, MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2, MAC_R3_FINALIZER_ENDPOINT_V2,
    MAC_R3_FINALIZER_JOURNAL_ROOT_V2, MAC_R3_FINALIZER_LAUNCHD_LABEL_V2, MAC_R3_FINALIZER_PATH_V2,
    MAC_R3_FINALIZER_PLIST_PATH_V2, MAC_R3_FINALIZER_REQUEST_PATH_V2,
    MAC_R3_PRODUCT_PUBLISHER_PATH_V2, MAC_R3_RETIREMENT_LATCH_ROOT_V2,
    MAC_R3_TERMINAL_BINDING_PATH_V2,
};

use super::pre_effect::{
    MAC_R3_FINALIZER_JOURNAL_ROOT_LOCK_PATH_V2, TERMINAL_ADMIN_CLEANUP_CLAIM_PATH_V2,
    TERMINAL_ADMIN_RESTORATION_RECEIPT_PATH_V2,
};
use super::{
    HarnessStageV2, RepetitionV2, ALTERNATE_COORDINATOR_PATH_V2, BENIGN_INJECTION_LIBRARY_PATH_V2,
    CANDIDATE_IDENTITY_PACKET_PATH_V2, DISPOSABLE_EXPERIMENT_RUNNER_PATH_V2,
    DISPOSABLE_EXPERIMENT_RUNNER_ROOT_V2, DISPOSABLE_HARNESS_PATH_V2,
    DISPOSABLE_PREPARED_INPUT_PATH_V2, DISPOSABLE_PUBLISHER_ROOT_V2, EXPERIMENT_ID_V2,
    EXPERIMENT_OWNER_V2, EXPERIMENT_ROOT_V2, EXPERIMENT_VERSION_V2, HARNESS_STAGE_SEQUENCE_V2,
    NOBODY_OWNER_PROBE_PATH_V2, PEER_CODE_PROBE_PATH_V2, PEER_CONTROL_IDENTITY_PACKET_PATH_V2,
    SECURITYAGENT_OBSERVER_PATH_V2,
};

const PRESERVED_SCOPE_ID_V2: &str = "019ff983-39ca-7182-9db8-b86aa66fa443";
const PRODUCT_KEYCHAIN_SERVICE_V1: &str = "com.substrate.lifecycle.v1";
const PRODUCT_LAUNCHD_LABEL_V1: &str = "system/com.substrate.lifecycle.publisher.v1";
const PRODUCT_ENDPOINT_V1: &str = "com.substrate.lifecycle.publisher.v1";
const PRODUCT_PLIST_V1: &str = "/Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist";
const PRODUCT_PROVENANCE_V1: &str =
    "/Library/Application Support/Substrate/lifecycle/bootstrap-provenance.v1.json";
const PRODUCT_LIFECYCLE_ROOT_V1: &str = "/Library/Application Support/Substrate/lifecycle-v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum DisposableInventoryNamespaceV2 {
    Scope,
    KeychainApplicationTag,
    KeychainLabel,
    KeychainService,
    KeychainAccount,
    ExecutablePath,
    FilesystemPath,
    LaunchdLabel,
    UnixEndpoint,
    LimaInstance,
    EvidenceMirrorIdentity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct DisposableInventoryEntryV2 {
    pub namespace: DisposableInventoryNamespaceV2,
    pub identity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "rule", rename_all = "snake_case", deny_unknown_fields)]
pub enum CompiledPreservedDenylistRuleV2 {
    PreservedScopeOrDerivedIdentity {
        scope_id: String,
    },
    ForbiddenNamespace {
        namespace: DisposableInventoryNamespaceV2,
    },
    ForbiddenExact {
        namespace: DisposableInventoryNamespaceV2,
        identity: String,
    },
    ForbiddenFilesystemPrefix {
        path: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DisposableDenylistProofV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub frozen_inventory: Vec<DisposableInventoryEntryV2>,
    pub compiled_preserved_denylist: Vec<CompiledPreservedDenylistRuleV2>,
    pub frozen_inventory_sha256: String,
    pub compiled_preserved_denylist_sha256: String,
    pub exhaustive_comparison_count: u64,
    pub exhaustive_disjoint_comparison_sha256: String,
}

impl DisposableDenylistProofV2 {
    pub fn validate(&self, repetition: RepetitionV2) -> Result<()> {
        require_header(self, repetition)?;
        let expected_inventory = frozen_inventory_v2(repetition);
        let expected_denylist = compiled_preserved_denylist_v2();
        if self.frozen_inventory != expected_inventory
            || self.compiled_preserved_denylist != expected_denylist
        {
            bail!("disposable proof does not contain the exact compiled inventory and denylist")
        }
        if self.frozen_inventory_sha256 != document_sha256_v2(&expected_inventory)?
            || self.compiled_preserved_denylist_sha256 != document_sha256_v2(&expected_denylist)?
        {
            bail!("disposable proof inventory or denylist digest changed")
        }
        let (count, comparison_sha256) =
            exhaustive_disjoint_comparison_v2(&expected_inventory, &expected_denylist)?;
        if self.exhaustive_comparison_count != count
            || self.exhaustive_disjoint_comparison_sha256 != comparison_sha256
        {
            bail!("disposable proof does not bind every inventory/denylist comparison")
        }
        Ok(())
    }
}

pub fn build_disposable_denylist_proof_v2(
    repetition: RepetitionV2,
) -> Result<DisposableDenylistProofV2> {
    let frozen_inventory = frozen_inventory_v2(repetition);
    let compiled_preserved_denylist = compiled_preserved_denylist_v2();
    let frozen_inventory_sha256 = document_sha256_v2(&frozen_inventory)?;
    let compiled_preserved_denylist_sha256 = document_sha256_v2(&compiled_preserved_denylist)?;
    let (exhaustive_comparison_count, exhaustive_disjoint_comparison_sha256) =
        exhaustive_disjoint_comparison_v2(&frozen_inventory, &compiled_preserved_denylist)?;
    let proof = DisposableDenylistProofV2 {
        schema_owner: EXPERIMENT_OWNER_V2.to_string(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_string(),
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_string(),
        frozen_inventory,
        compiled_preserved_denylist,
        frozen_inventory_sha256,
        compiled_preserved_denylist_sha256,
        exhaustive_comparison_count,
        exhaustive_disjoint_comparison_sha256,
    };
    proof.validate(repetition)?;
    Ok(proof)
}

fn inventory_entry(
    namespace: DisposableInventoryNamespaceV2,
    identity: impl Into<String>,
) -> DisposableInventoryEntryV2 {
    DisposableInventoryEntryV2 {
        namespace,
        identity: identity.into(),
    }
}

pub(crate) fn frozen_inventory_v2(repetition: RepetitionV2) -> Vec<DisposableInventoryEntryV2> {
    use DisposableInventoryNamespaceV2 as N;
    let scope = repetition.scope_id();
    let target = format!("{scope}:signing-key");
    let wrong = format!("{scope}:wrong-surrogate-signing-key");
    let capability_scope = format!("{MAC_R3_FINALIZER_JOURNAL_ROOT_V2}/capability/{scope}");
    let repetition_root = format!(
        "{EXPERIMENT_ROOT_V2}/repetitions/{}",
        repetition.directory_name()
    );
    vec![
        inventory_entry(N::Scope, scope),
        inventory_entry(N::KeychainApplicationTag, &target),
        inventory_entry(N::KeychainLabel, target),
        inventory_entry(N::KeychainApplicationTag, &wrong),
        inventory_entry(N::KeychainLabel, wrong),
        inventory_entry(N::ExecutablePath, MAC_R3_FINALIZER_PATH_V2),
        inventory_entry(N::ExecutablePath, MAC_R3_COORDINATOR_PATH_V2),
        inventory_entry(N::ExecutablePath, MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2),
        inventory_entry(N::ExecutablePath, DISPOSABLE_HARNESS_PATH_V2),
        inventory_entry(N::ExecutablePath, DISPOSABLE_EXPERIMENT_RUNNER_PATH_V2),
        inventory_entry(N::ExecutablePath, PEER_CODE_PROBE_PATH_V2),
        inventory_entry(N::ExecutablePath, BENIGN_INJECTION_LIBRARY_PATH_V2),
        inventory_entry(N::ExecutablePath, NOBODY_OWNER_PROBE_PATH_V2),
        inventory_entry(N::ExecutablePath, ALTERNATE_COORDINATOR_PATH_V2),
        inventory_entry(N::ExecutablePath, SECURITYAGENT_OBSERVER_PATH_V2),
        inventory_entry(N::FilesystemPath, EXPERIMENT_ROOT_V2),
        inventory_entry(N::FilesystemPath, repetition_root),
        inventory_entry(N::FilesystemPath, DISPOSABLE_PUBLISHER_ROOT_V2),
        inventory_entry(N::FilesystemPath, DISPOSABLE_EXPERIMENT_RUNNER_ROOT_V2),
        inventory_entry(N::FilesystemPath, DISPOSABLE_PREPARED_INPUT_PATH_V2),
        inventory_entry(N::FilesystemPath, CANDIDATE_IDENTITY_PACKET_PATH_V2),
        inventory_entry(N::FilesystemPath, PEER_CONTROL_IDENTITY_PACKET_PATH_V2),
        inventory_entry(N::FilesystemPath, MAC_R3_FINALIZER_PLIST_PATH_V2),
        inventory_entry(N::FilesystemPath, MAC_R3_FINALIZER_ENDPOINT_V2),
        inventory_entry(N::FilesystemPath, MAC_R3_FINALIZER_JOURNAL_ROOT_V2),
        inventory_entry(
            N::FilesystemPath,
            MAC_R3_FINALIZER_JOURNAL_ROOT_LOCK_PATH_V2,
        ),
        inventory_entry(N::FilesystemPath, TERMINAL_ADMIN_CLEANUP_CLAIM_PATH_V2),
        inventory_entry(
            N::FilesystemPath,
            TERMINAL_ADMIN_RESTORATION_RECEIPT_PATH_V2,
        ),
        inventory_entry(
            N::FilesystemPath,
            format!("{MAC_R3_FINALIZER_JOURNAL_ROOT_V2}/{scope}"),
        ),
        inventory_entry(N::FilesystemPath, &capability_scope),
        inventory_entry(
            N::FilesystemPath,
            format!("{capability_scope}/surrogate-wrapper.v2"),
        ),
        inventory_entry(
            N::FilesystemPath,
            format!("{capability_scope}/surrogate.lock"),
        ),
        inventory_entry(N::FilesystemPath, MAC_R3_RETIREMENT_LATCH_ROOT_V2),
        inventory_entry(
            N::FilesystemPath,
            format!("{MAC_R3_RETIREMENT_LATCH_ROOT_V2}/{scope}.retirement-terminal.v2.latch"),
        ),
        inventory_entry(N::FilesystemPath, MAC_R3_COORDINATOR_INBOX_ROOT_V2),
        inventory_entry(N::FilesystemPath, MAC_R3_FINALIZER_REQUEST_PATH_V2),
        inventory_entry(N::FilesystemPath, MAC_R3_TERMINAL_BINDING_PATH_V2),
        inventory_entry(N::LaunchdLabel, MAC_R3_FINALIZER_LAUNCHD_LABEL_V2),
        inventory_entry(N::UnixEndpoint, MAC_R3_FINALIZER_ENDPOINT_V2),
    ]
}

pub(crate) fn compiled_preserved_denylist_v2() -> Vec<CompiledPreservedDenylistRuleV2> {
    use CompiledPreservedDenylistRuleV2 as R;
    use DisposableInventoryNamespaceV2 as N;
    vec![
        R::PreservedScopeOrDerivedIdentity {
            scope_id: PRESERVED_SCOPE_ID_V2.to_string(),
        },
        R::ForbiddenNamespace {
            namespace: N::KeychainService,
        },
        R::ForbiddenNamespace {
            namespace: N::KeychainAccount,
        },
        R::ForbiddenNamespace {
            namespace: N::LimaInstance,
        },
        R::ForbiddenNamespace {
            namespace: N::EvidenceMirrorIdentity,
        },
        R::ForbiddenExact {
            namespace: N::KeychainService,
            identity: PRODUCT_KEYCHAIN_SERVICE_V1.to_string(),
        },
        R::ForbiddenExact {
            namespace: N::ExecutablePath,
            identity: MAC_R3_PRODUCT_PUBLISHER_PATH_V2.to_string(),
        },
        R::ForbiddenExact {
            namespace: N::FilesystemPath,
            identity: MAC_R3_PRODUCT_PUBLISHER_PATH_V2.to_string(),
        },
        R::ForbiddenExact {
            namespace: N::FilesystemPath,
            identity: PRODUCT_PLIST_V1.to_string(),
        },
        R::ForbiddenExact {
            namespace: N::FilesystemPath,
            identity: PRODUCT_PROVENANCE_V1.to_string(),
        },
        R::ForbiddenFilesystemPrefix {
            path: PRODUCT_LIFECYCLE_ROOT_V1.to_string(),
        },
        R::ForbiddenExact {
            namespace: N::LaunchdLabel,
            identity: PRODUCT_LAUNCHD_LABEL_V1.to_string(),
        },
        R::ForbiddenExact {
            namespace: N::UnixEndpoint,
            identity: PRODUCT_ENDPOINT_V1.to_string(),
        },
    ]
}

pub(crate) fn exhaustive_disjoint_comparison_v2(
    inventory: &[DisposableInventoryEntryV2],
    denylist: &[CompiledPreservedDenylistRuleV2],
) -> Result<(u64, String)> {
    let mut joins = Vec::with_capacity(inventory.len().saturating_mul(denylist.len()));
    for entry in inventory {
        if entry.identity.is_empty() || entry.identity.contains(['\0', '\n', '\r']) {
            bail!("disposable inventory contains an unsafe or empty identity")
        }
        for rule in denylist {
            if inventory_conflicts_with_rule_v2(entry, rule) {
                bail!("disposable inventory intersects a compiled preserved-state denylist rule")
            }
            joins.push(sha256_hex_v2(
                format!(
                    "substrate.r3-macos-disposable-denylist-disjoint.v2\0{}\0{}",
                    document_sha256_v2(entry)?,
                    document_sha256_v2(rule)?
                )
                .as_bytes(),
            ));
        }
    }
    let count = u64::try_from(joins.len()).map_err(|_| {
        anyhow::anyhow!("disposable inventory/denylist comparison count exceeds u64")
    })?;
    Ok((count, document_sha256_v2(&joins)?))
}

fn inventory_conflicts_with_rule_v2(
    entry: &DisposableInventoryEntryV2,
    rule: &CompiledPreservedDenylistRuleV2,
) -> bool {
    use CompiledPreservedDenylistRuleV2 as R;
    use DisposableInventoryNamespaceV2 as N;
    match rule {
        R::PreservedScopeOrDerivedIdentity { scope_id } => entry.identity.contains(scope_id),
        R::ForbiddenNamespace { namespace } => entry.namespace == *namespace,
        R::ForbiddenExact {
            namespace,
            identity,
        } => {
            entry.namespace == *namespace
                && if *namespace == N::FilesystemPath {
                    entry.identity == *identity
                        || entry
                            .identity
                            .strip_prefix(identity)
                            .is_some_and(|suffix| suffix.starts_with('/'))
                } else {
                    entry.identity == *identity
                }
        }
        R::ForbiddenFilesystemPrefix { path } => {
            entry.namespace == N::FilesystemPath
                && (entry.identity == *path
                    || entry
                        .identity
                        .strip_prefix(path)
                        .is_some_and(|suffix| suffix.starts_with('/')))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExperimentBaselineV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub experiment_root_identity_sha256: String,
    pub prepared_input_sha256: String,
    pub candidate_identity_packet_sha256: String,
    pub peer_control_identity_packet_sha256: String,
    pub global_pre_effect_packet_sha256: String,
    pub creator_route_receipt_set_sha256: String,
    pub global_created_object_inventory_sha256: String,
    pub global_native_arm_plan_sha256: String,
    pub publisher_process_attestation_sha256: String,
    pub harness_process_attestation_sha256: String,
    pub publisher_genesis_progress_sha256: String,
    pub exact_surrogate_predicates_sha256: String,
    pub frozen_inventory_sha256: String,
    pub compiled_preserved_denylist_sha256: String,
    pub denylist_proof_sha256: String,
}

impl ExperimentBaselineV2 {
    pub fn validate(
        &self,
        repetition: RepetitionV2,
        denylist_proof: &DisposableDenylistProofV2,
        global_pre_effect: &super::pre_effect::GlobalPreEffectPacketV2,
        creator_receipts: &super::pre_effect::CreatorRouteReceiptSetV2,
    ) -> Result<()> {
        require_header(self, repetition)?;
        denylist_proof.validate(repetition)?;
        if self.global_pre_effect_packet_sha256 != document_sha256_v2(global_pre_effect)?
            || self.creator_route_receipt_set_sha256 != document_sha256_v2(creator_receipts)?
            || self.global_created_object_inventory_sha256
                != global_pre_effect.created_object_inventory_sha256
            || self.global_native_arm_plan_sha256 != global_pre_effect.native_arm_plan_sha256
            || self.compiled_preserved_denylist_sha256
                != global_pre_effect.compiled_preserved_denylist_sha256
        {
            bail!("disposable baseline does not exact-bind the global pre-effect packet")
        }
        if self.frozen_inventory_sha256 != denylist_proof.frozen_inventory_sha256
            || self.compiled_preserved_denylist_sha256
                != denylist_proof.compiled_preserved_denylist_sha256
            || self.denylist_proof_sha256 != document_sha256_v2(denylist_proof)?
        {
            bail!("disposable baseline does not exact-bind its derived denylist proof")
        }
        for (digest, label) in [
            (
                &self.experiment_root_identity_sha256,
                "experiment root identity",
            ),
            (&self.prepared_input_sha256, "prepared input"),
            (
                &self.candidate_identity_packet_sha256,
                "candidate identity packet",
            ),
            (
                &self.peer_control_identity_packet_sha256,
                "peer-control identity packet",
            ),
            (
                &self.global_pre_effect_packet_sha256,
                "global pre-effect packet",
            ),
            (
                &self.creator_route_receipt_set_sha256,
                "creator route receipt set",
            ),
            (
                &self.global_created_object_inventory_sha256,
                "global created-object inventory",
            ),
            (
                &self.global_native_arm_plan_sha256,
                "global native-arm plan",
            ),
            (
                &self.publisher_process_attestation_sha256,
                "publisher process attestation",
            ),
            (
                &self.harness_process_attestation_sha256,
                "harness process attestation",
            ),
            (
                &self.publisher_genesis_progress_sha256,
                "publisher genesis progress",
            ),
            (
                &self.exact_surrogate_predicates_sha256,
                "exact surrogate predicates",
            ),
            (&self.frozen_inventory_sha256, "frozen experiment inventory"),
            (
                &self.compiled_preserved_denylist_sha256,
                "compiled preserved denylist",
            ),
            (&self.denylist_proof_sha256, "derived denylist proof"),
        ] {
            require_digest(digest, label)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SigningSeedRemovalObservationV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub seed_file_identity_sha256: String,
    pub removed_after_complete_and_restoration: bool,
    pub exact_path_absent: bool,
}

impl SigningSeedRemovalObservationV2 {
    pub fn validate(&self, repetition: RepetitionV2) -> Result<()> {
        require_header(self, repetition)?;
        if !self.removed_after_complete_and_restoration || !self.exact_path_absent {
            bail!("harness signing seed was not removed only after closed restoration")
        }
        require_digest(
            &self.seed_file_identity_sha256,
            "harness signing seed identity",
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RestorationManifestV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub baseline_packet_sha256: String,
    pub surrogate_before_observation_sha256: String,
    pub exact_after_observation_sha256: String,
    pub restoration_receipt_sha256: String,
    pub native_evidence_export_sha256: String,
    pub native_evidence_artifact_set_sha256: String,
    pub raw_securityagent_archive_sha256: String,
    pub native_evidence_acknowledgement_sha256: String,
    pub native_evidence_cleanup_receipt_sha256: String,
    pub journal_absence_observation_sha256: String,
    pub capability_absence_observation_sha256: String,
    pub latch_absence_observation_sha256: String,
    pub peer_control_restoration_receipt_sha256: String,
    pub complete_response_sha256: String,
    pub denylist_proof_sha256: String,
    pub restoration_safety_binding_sha256: String,
}

impl RestorationManifestV2 {
    pub fn validate(
        &self,
        repetition: RepetitionV2,
        denylist_proof: &DisposableDenylistProofV2,
    ) -> Result<()> {
        require_header(self, repetition)?;
        denylist_proof.validate(repetition)?;
        if self.surrogate_before_observation_sha256 != self.exact_after_observation_sha256 {
            bail!("restoration manifest does not prove exact disposable baseline parity")
        }
        if self.denylist_proof_sha256 != document_sha256_v2(denylist_proof)?
            || self.restoration_safety_binding_sha256
                != restoration_safety_binding_sha256_v2(self, denylist_proof)?
        {
            bail!("restoration manifest does not derive safety from its exact denylist proof")
        }
        for (digest, label) in [
            (&self.baseline_packet_sha256, "baseline packet"),
            (
                &self.surrogate_before_observation_sha256,
                "surrogate before observation",
            ),
            (
                &self.exact_after_observation_sha256,
                "exact after observation",
            ),
            (&self.restoration_receipt_sha256, "restoration receipt"),
            (
                &self.native_evidence_export_sha256,
                "native evidence export",
            ),
            (
                &self.native_evidence_artifact_set_sha256,
                "native evidence artifact set",
            ),
            (
                &self.raw_securityagent_archive_sha256,
                "raw SecurityAgent archive",
            ),
            (
                &self.native_evidence_acknowledgement_sha256,
                "native evidence acknowledgement",
            ),
            (
                &self.native_evidence_cleanup_receipt_sha256,
                "native evidence cleanup receipt",
            ),
            (
                &self.journal_absence_observation_sha256,
                "journal absence observation",
            ),
            (
                &self.capability_absence_observation_sha256,
                "capability absence observation",
            ),
            (
                &self.latch_absence_observation_sha256,
                "latch absence observation",
            ),
            (
                &self.peer_control_restoration_receipt_sha256,
                "peer-control restoration receipt",
            ),
            (&self.complete_response_sha256, "complete response"),
            (&self.denylist_proof_sha256, "derived denylist proof"),
            (
                &self.restoration_safety_binding_sha256,
                "restoration safety binding",
            ),
        ] {
            require_digest(digest, label)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestorationManifestInputV2 {
    pub baseline_packet_sha256: String,
    pub surrogate_before_observation_sha256: String,
    pub exact_after_observation_sha256: String,
    pub restoration_receipt_sha256: String,
    pub native_evidence_export_sha256: String,
    pub native_evidence_artifact_set_sha256: String,
    pub raw_securityagent_archive_sha256: String,
    pub native_evidence_acknowledgement_sha256: String,
    pub native_evidence_cleanup_receipt_sha256: String,
    pub journal_absence_observation_sha256: String,
    pub capability_absence_observation_sha256: String,
    pub latch_absence_observation_sha256: String,
    pub peer_control_restoration_receipt_sha256: String,
    pub complete_response_sha256: String,
}

pub fn build_restoration_manifest_v2(
    repetition: RepetitionV2,
    input: RestorationManifestInputV2,
    denylist_proof: &DisposableDenylistProofV2,
) -> Result<RestorationManifestV2> {
    denylist_proof.validate(repetition)?;
    let mut manifest = RestorationManifestV2 {
        schema_owner: EXPERIMENT_OWNER_V2.to_string(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_string(),
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_string(),
        baseline_packet_sha256: input.baseline_packet_sha256,
        surrogate_before_observation_sha256: input.surrogate_before_observation_sha256,
        exact_after_observation_sha256: input.exact_after_observation_sha256,
        restoration_receipt_sha256: input.restoration_receipt_sha256,
        native_evidence_export_sha256: input.native_evidence_export_sha256,
        native_evidence_artifact_set_sha256: input.native_evidence_artifact_set_sha256,
        raw_securityagent_archive_sha256: input.raw_securityagent_archive_sha256,
        native_evidence_acknowledgement_sha256: input.native_evidence_acknowledgement_sha256,
        native_evidence_cleanup_receipt_sha256: input.native_evidence_cleanup_receipt_sha256,
        journal_absence_observation_sha256: input.journal_absence_observation_sha256,
        capability_absence_observation_sha256: input.capability_absence_observation_sha256,
        latch_absence_observation_sha256: input.latch_absence_observation_sha256,
        peer_control_restoration_receipt_sha256: input.peer_control_restoration_receipt_sha256,
        complete_response_sha256: input.complete_response_sha256,
        denylist_proof_sha256: document_sha256_v2(denylist_proof)?,
        restoration_safety_binding_sha256: String::new(),
    };
    manifest.restoration_safety_binding_sha256 =
        restoration_safety_binding_sha256_v2(&manifest, denylist_proof)?;
    manifest.validate(repetition, denylist_proof)?;
    Ok(manifest)
}

fn restoration_safety_binding_sha256_v2(
    manifest: &RestorationManifestV2,
    denylist_proof: &DisposableDenylistProofV2,
) -> Result<String> {
    #[derive(Serialize)]
    #[serde(deny_unknown_fields)]
    struct RestorationSafetyBindingV2<'a> {
        domain: &'static str,
        repetition: u8,
        scope_id: &'a str,
        baseline_packet_sha256: &'a str,
        surrogate_before_observation_sha256: &'a str,
        exact_after_observation_sha256: &'a str,
        restoration_receipt_sha256: &'a str,
        native_evidence_export_sha256: &'a str,
        native_evidence_artifact_set_sha256: &'a str,
        raw_securityagent_archive_sha256: &'a str,
        native_evidence_acknowledgement_sha256: &'a str,
        native_evidence_cleanup_receipt_sha256: &'a str,
        journal_absence_observation_sha256: &'a str,
        capability_absence_observation_sha256: &'a str,
        latch_absence_observation_sha256: &'a str,
        peer_control_restoration_receipt_sha256: &'a str,
        complete_response_sha256: &'a str,
        denylist_proof_sha256: &'a str,
        frozen_inventory_sha256: &'a str,
        compiled_preserved_denylist_sha256: &'a str,
        exhaustive_disjoint_comparison_sha256: &'a str,
    }
    document_sha256_v2(&RestorationSafetyBindingV2 {
        domain: "substrate.r3-macos-disposable-restoration-safety.v2",
        repetition: manifest.repetition,
        scope_id: &manifest.scope_id,
        baseline_packet_sha256: &manifest.baseline_packet_sha256,
        surrogate_before_observation_sha256: &manifest.surrogate_before_observation_sha256,
        exact_after_observation_sha256: &manifest.exact_after_observation_sha256,
        restoration_receipt_sha256: &manifest.restoration_receipt_sha256,
        native_evidence_export_sha256: &manifest.native_evidence_export_sha256,
        native_evidence_artifact_set_sha256: &manifest.native_evidence_artifact_set_sha256,
        raw_securityagent_archive_sha256: &manifest.raw_securityagent_archive_sha256,
        native_evidence_acknowledgement_sha256: &manifest.native_evidence_acknowledgement_sha256,
        native_evidence_cleanup_receipt_sha256: &manifest.native_evidence_cleanup_receipt_sha256,
        journal_absence_observation_sha256: &manifest.journal_absence_observation_sha256,
        capability_absence_observation_sha256: &manifest.capability_absence_observation_sha256,
        latch_absence_observation_sha256: &manifest.latch_absence_observation_sha256,
        peer_control_restoration_receipt_sha256: &manifest.peer_control_restoration_receipt_sha256,
        complete_response_sha256: &manifest.complete_response_sha256,
        denylist_proof_sha256: &manifest.denylist_proof_sha256,
        frozen_inventory_sha256: &denylist_proof.frozen_inventory_sha256,
        compiled_preserved_denylist_sha256: &denylist_proof.compiled_preserved_denylist_sha256,
        exhaustive_disjoint_comparison_sha256: &denylist_proof
            .exhaustive_disjoint_comparison_sha256,
    })
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HarnessInputArtifactV2 {
    PreparedInput,
    CandidateIdentity,
    PeerControlIdentity,
    GlobalPreEffectPacket,
    CreatorRouteReceiptSet,
    PublisherProcessAttestation,
    HarnessProcessAttestation,
    HarnessSigningSeedObservation,
    DisposableDenylistProof,
    CoordinatorProcessAttestation,
    SurrogateCreationReceipt,
    SignedPublisherReceipt,
    HarnessAcknowledgement,
    SignedProtectedCas,
    FrozenFinalizationRequest,
    RequestPublicationReceipt,
    FinalizerAcceptanceEvidence,
    EffectsResponse,
    EffectsNativeArmReceipt,
    ResidualCleanupReceipt,
    ParityProof,
    TerminalAcknowledgement,
    TerminalPublicationReceipt,
    CompleteResponse,
    CompleteNativeArmReceipt,
    RestorationReceipt,
    NativeEvidenceExport,
    NativeEvidenceExportAcknowledgement,
    NativeEvidenceCleanupReceipt,
    PeerControlRestorationReceipt,
    RestorationManifest,
    SigningSeedRemovalObservation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HarnessInputArtifactDigestV2 {
    pub artifact: HarnessInputArtifactV2,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HarnessInputBundleV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub prior_stage: HarnessStageV2,
    pub next_stage: HarnessStageV2,
    pub artifacts: Vec<HarnessInputArtifactDigestV2>,
}

impl HarnessInputBundleV2 {
    pub fn validate(&self, repetition: RepetitionV2) -> Result<()> {
        require_header(self, repetition)?;
        self.prior_stage.require_successor(self.next_stage)?;
        let expected = expected_harness_artifacts_v2(self.next_stage);
        if self.artifacts.len() != expected.len()
            || !self
                .artifacts
                .iter()
                .zip(expected)
                .all(|(actual, expected)| actual.artifact == *expected)
        {
            bail!("harness input bundle is not the exact ordered stage artifact set")
        }
        for artifact in &self.artifacts {
            require_digest(&artifact.sha256, "harness input artifact")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HarnessStepReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub prior_stage: HarnessStageV2,
    pub new_stage: HarnessStageV2,
    pub input_bundle_sha256: String,
    pub output_sha256: String,
}

impl HarnessStepReceiptV2 {
    pub fn validate(&self, repetition: RepetitionV2, bundle: &HarnessInputBundleV2) -> Result<()> {
        require_header(self, repetition)?;
        bundle.validate(repetition)?;
        if self.prior_stage != bundle.prior_stage
            || self.new_stage != bundle.next_stage
            || self.input_bundle_sha256 != document_sha256_v2(bundle)?
        {
            bail!("harness step receipt does not bind its exact input bundle")
        }
        require_digest(&self.output_sha256, "harness stage output")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HarnessProgressV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub generation: u64,
    pub stage: HarnessStageV2,
    pub predecessor_progress_sha256: String,
    pub accepted_input_bundle_sha256: String,
    pub step_receipt_sha256: String,
}

impl HarnessProgressV2 {
    pub fn genesis(repetition: RepetitionV2) -> Self {
        Self {
            schema_owner: EXPERIMENT_OWNER_V2.to_string(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_string(),
            repetition: repetition.ordinal(),
            scope_id: repetition.scope_id().to_string(),
            generation: 1,
            stage: HarnessStageV2::Prepared,
            predecessor_progress_sha256: harness_genesis_sha256_v2(repetition, "progress-genesis"),
            accepted_input_bundle_sha256: harness_genesis_sha256_v2(repetition, "no-input-yet"),
            step_receipt_sha256: harness_genesis_sha256_v2(repetition, "no-step-yet"),
        }
    }

    pub fn validate(&self, repetition: RepetitionV2) -> Result<()> {
        require_header(self, repetition)?;
        if self.generation != harness_stage_generation_v2(self.stage) {
            bail!("harness progress generation does not exact-match its closed stage")
        }
        for (digest, label) in [
            (
                &self.predecessor_progress_sha256,
                "harness predecessor progress",
            ),
            (
                &self.accepted_input_bundle_sha256,
                "harness accepted input bundle",
            ),
            (&self.step_receipt_sha256, "harness step receipt"),
        ] {
            require_digest(digest, label)?;
        }
        if self.stage == HarnessStageV2::Prepared && self != &Self::genesis(repetition) {
            bail!("harness Prepared progress is not the exact scope-bound genesis")
        }
        Ok(())
    }

    pub fn validate_successor(
        &self,
        repetition: RepetitionV2,
        predecessor: &Self,
        bundle: &HarnessInputBundleV2,
        step: &HarnessStepReceiptV2,
    ) -> Result<()> {
        self.validate(repetition)?;
        predecessor.validate(repetition)?;
        predecessor.stage.require_successor(self.stage)?;
        step.validate(repetition, bundle)?;
        if self.generation != predecessor.generation + 1
            || bundle.prior_stage != predecessor.stage
            || bundle.next_stage != self.stage
            || self.predecessor_progress_sha256 != document_sha256_v2(predecessor)?
            || self.accepted_input_bundle_sha256 != document_sha256_v2(bundle)?
            || self.step_receipt_sha256 != document_sha256_v2(step)?
        {
            bail!("harness progress replacement does not exact-bind predecessor, input, and step")
        }
        Ok(())
    }
}

pub fn build_harness_transition_v2(
    repetition: RepetitionV2,
    predecessor: &HarnessProgressV2,
    next_stage: HarnessStageV2,
    artifacts: Vec<HarnessInputArtifactDigestV2>,
) -> Result<(
    HarnessInputBundleV2,
    HarnessStepReceiptV2,
    HarnessProgressV2,
)> {
    predecessor.validate(repetition)?;
    let bundle = HarnessInputBundleV2 {
        schema_owner: EXPERIMENT_OWNER_V2.to_string(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_string(),
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_string(),
        prior_stage: predecessor.stage,
        next_stage,
        artifacts,
    };
    bundle.validate(repetition)?;
    let output_sha256 = document_sha256_v2(
        &bundle
            .artifacts
            .iter()
            .map(|artifact| &artifact.sha256)
            .collect::<Vec<_>>(),
    )?;
    let step = HarnessStepReceiptV2 {
        schema_owner: EXPERIMENT_OWNER_V2.to_string(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_string(),
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_string(),
        prior_stage: predecessor.stage,
        new_stage: next_stage,
        input_bundle_sha256: document_sha256_v2(&bundle)?,
        output_sha256,
    };
    let progress = HarnessProgressV2 {
        schema_owner: EXPERIMENT_OWNER_V2.to_string(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_string(),
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_string(),
        generation: harness_stage_generation_v2(next_stage),
        stage: next_stage,
        predecessor_progress_sha256: document_sha256_v2(predecessor)?,
        accepted_input_bundle_sha256: document_sha256_v2(&bundle)?,
        step_receipt_sha256: document_sha256_v2(&step)?,
    };
    progress.validate_successor(repetition, predecessor, &bundle, &step)?;
    Ok((bundle, step, progress))
}

pub fn harness_stage_generation_v2(stage: HarnessStageV2) -> u64 {
    u64::try_from(
        HARNESS_STAGE_SEQUENCE_V2
            .iter()
            .position(|candidate| *candidate == stage)
            .expect("closed harness stage is in its sequence"),
    )
    .expect("closed harness stage count fits u64")
        + 1
}

pub fn harness_genesis_sha256_v2(repetition: RepetitionV2, label: &str) -> String {
    sha256_hex_v2(
        format!(
            "substrate.r3-macos-disposable-harness.v2\0{}\0{label}",
            repetition.scope_id()
        )
        .as_bytes(),
    )
}

pub const fn expected_harness_artifacts_v2(
    stage: HarnessStageV2,
) -> &'static [HarnessInputArtifactV2] {
    use HarnessInputArtifactV2 as A;
    match stage {
        HarnessStageV2::Prepared => &[],
        HarnessStageV2::CoordinatorAttested => &[
            A::PreparedInput,
            A::CandidateIdentity,
            A::PeerControlIdentity,
            A::GlobalPreEffectPacket,
            A::CreatorRouteReceiptSet,
            A::PublisherProcessAttestation,
            A::HarnessProcessAttestation,
            A::HarnessSigningSeedObservation,
            A::DisposableDenylistProof,
            A::CoordinatorProcessAttestation,
        ],
        HarnessStageV2::SurrogatesCreated => &[A::SurrogateCreationReceipt],
        HarnessStageV2::ReceiptExternallyDurable => &[A::SignedPublisherReceipt],
        HarnessStageV2::AcknowledgementExternallyDurable => &[A::HarnessAcknowledgement],
        HarnessStageV2::AcknowledgementCasBound => &[A::SignedProtectedCas],
        HarnessStageV2::FinalizationRequestFrozen => &[A::FrozenFinalizationRequest],
        HarnessStageV2::FinalizationRequestPublished => &[A::RequestPublicationReceipt],
        HarnessStageV2::FinalizerAccepted => &[A::FinalizerAcceptanceEvidence],
        HarnessStageV2::EffectsComplete => &[A::EffectsResponse, A::EffectsNativeArmReceipt],
        HarnessStageV2::HarnessResidualRemoving => &[A::ResidualCleanupReceipt],
        HarnessStageV2::ParityExternallyDurable => &[A::ParityProof],
        HarnessStageV2::TerminalAcknowledgementBound => &[A::TerminalAcknowledgement],
        HarnessStageV2::TerminalBindingPublished => &[A::TerminalPublicationReceipt],
        HarnessStageV2::Complete => &[A::CompleteResponse, A::CompleteNativeArmReceipt],
        HarnessStageV2::RestorationComplete => &[
            A::RestorationReceipt,
            A::NativeEvidenceExport,
            A::NativeEvidenceExportAcknowledgement,
            A::NativeEvidenceCleanupReceipt,
            A::PeerControlRestorationReceipt,
            A::RestorationManifest,
            A::SigningSeedRemovalObservation,
        ],
    }
}

trait HeaderV2 {
    fn schema_owner(&self) -> &str;
    fn schema_version(&self) -> u32;
    fn experiment_id(&self) -> &str;
    fn repetition(&self) -> u8;
    fn scope_id(&self) -> &str;
}

macro_rules! impl_header {
    ($type:ty) => {
        impl HeaderV2 for $type {
            fn schema_owner(&self) -> &str {
                &self.schema_owner
            }
            fn schema_version(&self) -> u32 {
                self.schema_version
            }
            fn experiment_id(&self) -> &str {
                &self.experiment_id
            }
            fn repetition(&self) -> u8 {
                self.repetition
            }
            fn scope_id(&self) -> &str {
                &self.scope_id
            }
        }
    };
}

impl_header!(HarnessInputBundleV2);
impl_header!(HarnessStepReceiptV2);
impl_header!(HarnessProgressV2);
impl_header!(DisposableDenylistProofV2);
impl_header!(ExperimentBaselineV2);
impl_header!(SigningSeedRemovalObservationV2);
impl_header!(RestorationManifestV2);

fn require_header<T: HeaderV2>(value: &T, repetition: RepetitionV2) -> Result<()> {
    if value.schema_owner() != EXPERIMENT_OWNER_V2
        || value.schema_version() != EXPERIMENT_VERSION_V2
        || value.experiment_id() != EXPERIMENT_ID_V2
    {
        bail!("harness protocol owner, version, or experiment changed")
    }
    repetition.validate_binding(value.repetition(), value.scope_id())
}

fn require_digest(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} is not an exact lowercase SHA-256 digest")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(seed: u8) -> String {
        format!("{seed:02x}").repeat(32)
    }

    #[test]
    fn every_harness_stage_exact_binds_predecessor_artifacts_and_step() {
        let repetition = RepetitionV2::Two;
        let mut progress = HarnessProgressV2::genesis(repetition);
        for (index, stage) in HARNESS_STAGE_SEQUENCE_V2
            .iter()
            .copied()
            .skip(1)
            .enumerate()
        {
            let artifacts = expected_harness_artifacts_v2(stage)
                .iter()
                .copied()
                .enumerate()
                .map(|(artifact_index, artifact)| HarnessInputArtifactDigestV2 {
                    artifact,
                    sha256: digest(u8::try_from(index + artifact_index + 1).unwrap()),
                })
                .collect();
            let (_, _, successor) =
                build_harness_transition_v2(repetition, &progress, stage, artifacts).unwrap();
            let mut skipped = successor.clone();
            skipped.generation += 1;
            assert!(skipped.validate(repetition).is_err());
            progress = successor;
        }
        assert_eq!(progress.stage, HarnessStageV2::RestorationComplete);
        assert_eq!(progress.generation, 16);
    }

    #[test]
    fn derived_inventory_is_exhaustively_disjoint_from_compiled_preserved_rules() {
        for repetition in RepetitionV2::ALL {
            let proof = build_disposable_denylist_proof_v2(repetition).unwrap();
            proof.validate(repetition).unwrap();
            assert_eq!(
                proof.exhaustive_comparison_count,
                u64::try_from(
                    proof.frozen_inventory.len() * proof.compiled_preserved_denylist.len()
                )
                .unwrap()
            );
            assert!(proof
                .frozen_inventory
                .iter()
                .all(|entry| !entry.identity.contains(PRESERVED_SCOPE_ID_V2)));

            let mut widened = proof.clone();
            widened.frozen_inventory.push(inventory_entry(
                DisposableInventoryNamespaceV2::LimaInstance,
                "substrate",
            ));
            assert!(widened.validate(repetition).is_err());
        }

        let preserved = inventory_entry(
            DisposableInventoryNamespaceV2::KeychainApplicationTag,
            format!("{PRESERVED_SCOPE_ID_V2}:signing-key"),
        );
        assert!(inventory_conflicts_with_rule_v2(
            &preserved,
            &CompiledPreservedDenylistRuleV2::PreservedScopeOrDerivedIdentity {
                scope_id: PRESERVED_SCOPE_ID_V2.to_string(),
            }
        ));
    }

    #[test]
    fn restoration_safety_is_derived_from_proof_and_exact_parity() {
        let repetition = RepetitionV2::One;
        let proof = build_disposable_denylist_proof_v2(repetition).unwrap();
        let parity = digest(2);
        let manifest = build_restoration_manifest_v2(
            repetition,
            RestorationManifestInputV2 {
                baseline_packet_sha256: digest(1),
                surrogate_before_observation_sha256: parity.clone(),
                exact_after_observation_sha256: parity,
                restoration_receipt_sha256: digest(3),
                native_evidence_export_sha256: digest(4),
                native_evidence_artifact_set_sha256: digest(5),
                raw_securityagent_archive_sha256: digest(6),
                native_evidence_acknowledgement_sha256: digest(7),
                native_evidence_cleanup_receipt_sha256: digest(8),
                journal_absence_observation_sha256: digest(9),
                capability_absence_observation_sha256: digest(10),
                latch_absence_observation_sha256: digest(11),
                peer_control_restoration_receipt_sha256: digest(12),
                complete_response_sha256: digest(13),
            },
            &proof,
        )
        .unwrap();
        manifest.validate(repetition, &proof).unwrap();

        let mut substituted = manifest.clone();
        substituted.complete_response_sha256 = digest(14);
        assert!(substituted.validate(repetition, &proof).is_err());

        let source = include_str!("harness_protocol.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        assert!(!production.contains("preserved_denylist_disjoint: bool"));
        assert!(!production.contains("preserved_denylist_untouched: bool"));
    }
}
