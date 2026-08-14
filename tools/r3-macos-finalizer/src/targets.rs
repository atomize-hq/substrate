use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use substrate_common::macos_retirement_v2::{
    derive_host_effect_plan_v2, DisposableCapabilityControlV2, FixedFileRoleV2,
    GenericPasswordRoleV2, HostResourceLocatorV2, HostTargetIdentityV2, HostTargetRoleV2,
    LifecycleDirectoryRoleV2, LifecycleFileRoleV2, PostPmArtifactLabelV2, PostPmJournalStateV2,
    TargetSetKindV2,
};
pub use substrate_common::macos_retirement_v2::{
    MAC_R3_GUEST_RETIREMENT_CAS_ACCOUNT_SUFFIX_V2 as GUEST_RETIREMENT_CAS_SUFFIX_V2,
    MAC_R3_PRODUCT_PUBLISHER_PATH_V2 as PRODUCT_HELPER_V1,
    MAC_R3_R6_TERMINAL_ACK_ACCOUNT_SUFFIX_V2 as R6_TERMINAL_ACKNOWLEDGEMENT_SUFFIX_V2,
    MAC_R3_RETIREMENT_CAS_ACCOUNT_SUFFIX_V2 as RETIREMENT_CAS_SUFFIX_V2,
    MAC_R3_RETIREMENT_LATCH_ROOT_V2 as FINALIZER_LATCH_ROOT_V2,
    MAC_R3_RETIREMENT_RESOURCE_INDEX_ACCOUNT_SUFFIX_V2 as RETIREMENT_RESOURCE_INDEX_SUFFIX_V2,
};

pub const PRODUCT_SERVICE_V1: &str = "system/com.substrate.lifecycle.publisher.v1";
pub const PRODUCT_ENDPOINT_V1: &str = "com.substrate.lifecycle.publisher.v1";
pub const PRODUCT_PLIST_V1: &str =
    "/Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist";
pub const PRODUCT_PROVENANCE_V1: &str =
    "/Library/Application Support/Substrate/lifecycle/bootstrap-provenance.v1.json";
pub const PRODUCT_LIFECYCLE_ROOT_V1: &str = "/Library/Application Support/Substrate/lifecycle-v1";
pub const PRODUCT_KEYCHAIN_SERVICE_V1: &str = "com.substrate.lifecycle.v1";
pub const FINALIZER_CAPABILITY_ROOT_V2: &str =
    "/private/var/db/com.atomize.substrate.r3-macos-evidence-finalizer.v2/capability";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum FixedTarget {
    LaunchdService {
        label: &'static str,
    },
    MachEndpoint {
        name: &'static str,
    },
    ProtectedGenericPassword {
        service: &'static str,
        account: String,
    },
    SigningKey {
        application_tag: Vec<u8>,
    },
    DisposableCapabilityControl {
        control: DisposableCapabilityControlV2,
    },
    File {
        path: &'static str,
    },
    DerivedFile {
        path: String,
    },
    DerivedDirectory {
        path: String,
    },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DerivedTarget {
    pub ordinal: u16,
    pub role: HostTargetRoleV2,
    pub locator: HostResourceLocatorV2,
    pub effect_identity_sha256: String,
    pub target: FixedTarget,
}

/// Expand a signed target ledger into exact physical targets. The common effect-plan derivation is
/// the authority for ordering, repeated roles, locator validity, and each target identity digest.
pub fn derive_targets_from_ledger(
    kind: TargetSetKindV2,
    scope: &str,
    target_ledger: &[HostTargetIdentityV2],
) -> Result<Vec<DerivedTarget>> {
    require_uuid_v7(scope)?;
    let effect_plan = derive_host_effect_plan_v2(kind, target_ledger)?;
    target_ledger
        .iter()
        .zip(effect_plan)
        .map(|(identity, effect)| {
            let target = derive_one_target(kind, scope, &identity.locator)?;
            Ok(DerivedTarget {
                ordinal: identity.ordinal,
                role: identity.role,
                locator: identity.locator.clone(),
                effect_identity_sha256: effect.target_identity_sha256,
                target,
            })
        })
        .collect()
}

/// Compatibility helper used only by the sealed disposable capability probe. Prospective target
/// expansion is necessarily ledger-bound because its signed inventory is repeatable and complete.
pub fn derive_targets(kind: TargetSetKindV2, scope: &str) -> Result<Vec<DerivedTarget>> {
    if kind != TargetSetKindV2::DisposableCapability {
        bail!("prospective targets require the complete signed target ledger")
    }
    let ledger = disposable_target_ledger(scope)?;
    derive_targets_from_ledger(kind, scope, &ledger)
}

fn disposable_target_ledger(scope: &str) -> Result<Vec<HostTargetIdentityV2>> {
    require_uuid_v7(scope)?;
    use DisposableCapabilityControlV2 as C;
    use HostResourceLocatorV2 as L;
    let locators = vec![
        L::DisposableCapabilityControl { control: C::Sign },
        L::DisposableCapabilityControl {
            control: C::ExportPrivate,
        },
        L::DisposableCapabilityControl {
            control: C::ReplaceAccess,
        },
        L::DisposableCapabilityControl {
            control: C::DeleteWrongKey,
        },
        L::DisposableProtectedWrapper,
        L::SigningKey,
        L::DisposableCurrentLock,
        L::RetirementTerminalLatch,
    ];
    Ok(locators
        .into_iter()
        .enumerate()
        .map(|(index, locator)| HostTargetIdentityV2 {
            ordinal: u16::try_from(index + 1).expect("fixed disposable plan fits u16"),
            role: substrate_common::macos_retirement_v2::host_target_role_for_locator_v2(&locator),
            locator,
            // This helper derives only predicates/tags for the separately receipt-bound probe.
            // The engine always receives the signed exact expected-before digest instead.
            expected_before_sha256: "00".repeat(32),
        })
        .collect())
}

fn derive_one_target(
    kind: TargetSetKindV2,
    scope: &str,
    locator: &HostResourceLocatorV2,
) -> Result<FixedTarget> {
    use HostResourceLocatorV2 as L;
    match locator {
        L::PublisherService => require_lane(kind, TargetSetKindV2::ProspectiveHost).map(|()| {
            FixedTarget::LaunchdService {
                label: PRODUCT_SERVICE_V1,
            }
        }),
        L::PublisherEndpoint => require_lane(kind, TargetSetKindV2::ProspectiveHost).map(|()| {
            FixedTarget::MachEndpoint {
                name: PRODUCT_ENDPOINT_V1,
            }
        }),
        L::GenericPassword { role } => {
            require_lane(kind, TargetSetKindV2::ProspectiveHost)?;
            Ok(FixedTarget::ProtectedGenericPassword {
                service: PRODUCT_KEYCHAIN_SERVICE_V1,
                account: generic_password_account(scope, role)?,
            })
        }
        L::LifecycleFile { role } => {
            require_lane(kind, TargetSetKindV2::ProspectiveHost)?;
            Ok(FixedTarget::DerivedFile {
                path: lifecycle_file_path(scope, role)?
                    .to_string_lossy()
                    .into_owned(),
            })
        }
        L::LifecycleDirectory { role } => {
            require_lane(kind, TargetSetKindV2::ProspectiveHost)?;
            Ok(FixedTarget::DerivedDirectory {
                path: lifecycle_directory_path(scope, role)?
                    .to_string_lossy()
                    .into_owned(),
            })
        }
        L::SigningKey => Ok(FixedTarget::SigningKey {
            application_tag: format!("{scope}:signing-key").into_bytes(),
        }),
        L::FixedFile { role } => {
            require_lane(kind, TargetSetKindV2::ProspectiveHost)?;
            Ok(FixedTarget::File {
                path: match role {
                    FixedFileRoleV2::PublisherHelper => PRODUCT_HELPER_V1,
                    FixedFileRoleV2::PublisherLaunchdPlist => PRODUCT_PLIST_V1,
                    FixedFileRoleV2::InstallProvenance => PRODUCT_PROVENANCE_V1,
                },
            })
        }
        L::KeychainCasLock { account } => {
            require_lane(kind, TargetSetKindV2::ProspectiveHost)?;
            let account = generic_password_account(scope, account)?;
            Ok(FixedTarget::DerivedFile {
                path: format!(
                    "{PRODUCT_LIFECYCLE_ROOT_V1}/keychain-cas-{:x}.lock",
                    Sha256::digest(account.as_bytes())
                ),
            })
        }
        L::LifecycleTargetLock { target } => {
            require_lane(kind, TargetSetKindV2::ProspectiveHost)?;
            if !matches!(
                target,
                LifecycleFileRoleV2::Manifest { .. }
                    | LifecycleFileRoleV2::ReceiptIndex { .. }
                    | LifecycleFileRoleV2::Head
                    | LifecycleFileRoleV2::Receipt { .. }
            ) {
                bail!("prospective lifecycle target has no retained target lock")
            }
            let target = lifecycle_file_path(scope, target)?;
            let target = target
                .to_str()
                .context("lifecycle target-lock path is not UTF-8")?;
            Ok(FixedTarget::DerivedFile {
                path: format!(
                    "{PRODUCT_LIFECYCLE_ROOT_V1}/lifecycle-target-{:x}.lock",
                    Sha256::digest(target.as_bytes())
                ),
            })
        }
        L::RetirementTerminalLatch => Ok(FixedTarget::DerivedFile {
            path: format!("{FINALIZER_LATCH_ROOT_V2}/{scope}.retirement-terminal.v2.latch"),
        }),
        L::DisposableCapabilityControl { control } => {
            require_lane(kind, TargetSetKindV2::DisposableCapability)?;
            Ok(FixedTarget::DisposableCapabilityControl { control: *control })
        }
        L::DisposableProtectedWrapper => {
            require_lane(kind, TargetSetKindV2::DisposableCapability)?;
            Ok(FixedTarget::DerivedFile {
                path: format!("{FINALIZER_CAPABILITY_ROOT_V2}/{scope}/surrogate-wrapper.v2"),
            })
        }
        L::DisposableCurrentLock => {
            require_lane(kind, TargetSetKindV2::DisposableCapability)?;
            Ok(FixedTarget::DerivedFile {
                path: format!("{FINALIZER_CAPABILITY_ROOT_V2}/{scope}/surrogate.lock"),
            })
        }
    }
}

fn require_lane(actual: TargetSetKindV2, expected: TargetSetKindV2) -> Result<()> {
    if actual != expected {
        bail!("target locator crossed the product/disposable lane boundary")
    }
    Ok(())
}

fn generic_password_account(scope: &str, role: &GenericPasswordRoleV2) -> Result<String> {
    use GenericPasswordRoleV2 as G;
    let account = match role {
        G::ControlAdmission => "mac-control-admission-authority.v1".to_string(),
        G::CurrentAnchor => scoped_account(scope, "current-anchor"),
        G::PublisherServiceState => scoped_account(scope, "publisher-service-state"),
        G::PublisherBootstrapIntent => scoped_account(scope, "publisher-bootstrap-intent"),
        G::BootstrapAttemptLocator { attempt_key_sha256 } => {
            format!("mac-publisher-bootstrap-attempt-locator-v1:{attempt_key_sha256}")
        }
        G::LimaStageOneCapsule => scoped_account(scope, "lima-stage-one-capsule"),
        G::R6PredecessorState => scoped_account(scope, "r6-pairing-predecessor-state"),
        G::R6Predecessor { generation } => {
            scoped_account(scope, &format!("r6-pairing-predecessor-{generation}"))
        }
        G::R6Continuation { generation } => {
            scoped_account(scope, &format!("r6-pairing-continuation-{generation}"))
        }
        G::R6Activation { generation } => {
            scoped_account(scope, &format!("r6-pairing-activation-{generation}"))
        }
        G::R6Tombstone { generation } => {
            scoped_account(scope, &format!("r6-pairing-tombstone-{generation}"))
        }
        G::R6MissedActivation { generation } => {
            scoped_account(scope, &format!("r6-pairing-missed-activation-{generation}"))
        }
        G::R6ActiveGuestRecord => scoped_account(scope, "guest-pairing-r6-active-record"),
        G::R6GuestHostRecord { challenge_id } => {
            format!("{challenge_id}:guest-pairing-r6-record")
        }
        G::R6OperatorLaunch { challenge_id } => {
            scoped_account(scope, &format!("r6-pairing-operator-launch-{challenge_id}"))
        }
        G::GuestRetirementCas => scoped_account(scope, GUEST_RETIREMENT_CAS_SUFFIX_V2),
        G::RetirementResourceIndex => scoped_account(scope, RETIREMENT_RESOURCE_INDEX_SUFFIX_V2),
        G::RetirementCas => scoped_account(scope, RETIREMENT_CAS_SUFFIX_V2),
        G::R6TerminalAcknowledgement => {
            scoped_account(scope, R6_TERMINAL_ACKNOWLEDGEMENT_SUFFIX_V2)
        }
    };
    Ok(account)
}

fn scoped_account(scope: &str, suffix: &str) -> String {
    format!("{scope}:{suffix}")
}

fn lifecycle_file_path(scope: &str, role: &LifecycleFileRoleV2) -> Result<PathBuf> {
    let root = Path::new(PRODUCT_LIFECYCLE_ROOT_V1);
    Ok(match role {
        LifecycleFileRoleV2::Manifest { generation } => {
            root.join(format!("manifest.{generation}.json"))
        }
        LifecycleFileRoleV2::ReceiptIndex { generation } => {
            root.join(format!("action-receipts.{generation}.v1.json"))
        }
        LifecycleFileRoleV2::Head => root.join("head.v1.json"),
        LifecycleFileRoleV2::Receipt {
            generation,
            receipt_id,
        } => root
            .join("receipts")
            .join(generation.to_string())
            .join(format!("receipt.{receipt_id}.json")),
        LifecycleFileRoleV2::StageOneProfile { attempt_id } => root
            .join("stage-one-profiles")
            .join(scope)
            .join(format!("{attempt_id}.yaml")),
        LifecycleFileRoleV2::PostPmJournal { receipt_id, state } => root
            .join("post-pm-attempts")
            .join(receipt_id)
            .join(format!("{}.v1.json", post_pm_state_literal(*state))),
        LifecycleFileRoleV2::PostPmArtifact {
            receipt_id,
            label,
            sha256,
        } => root
            .join("post-pm-artifacts")
            .join(receipt_id)
            .join(format!("{}.{}.bin", post_pm_artifact_label(label), sha256)),
    })
}

fn lifecycle_directory_path(scope: &str, role: &LifecycleDirectoryRoleV2) -> Result<PathBuf> {
    let root = Path::new(PRODUCT_LIFECYCLE_ROOT_V1);
    Ok(match role {
        LifecycleDirectoryRoleV2::ReceiptGeneration { generation } => {
            root.join("receipts").join(generation.to_string())
        }
        LifecycleDirectoryRoleV2::ReceiptsRoot => root.join("receipts"),
        LifecycleDirectoryRoleV2::StageOneScope => root.join("stage-one-profiles").join(scope),
        LifecycleDirectoryRoleV2::StageOneProfilesRoot => root.join("stage-one-profiles"),
        LifecycleDirectoryRoleV2::PostPmAttempt { receipt_id } => {
            root.join("post-pm-attempts").join(receipt_id)
        }
        LifecycleDirectoryRoleV2::PostPmAttemptsRoot => root.join("post-pm-attempts"),
        LifecycleDirectoryRoleV2::PostPmArtifact { receipt_id } => {
            root.join("post-pm-artifacts").join(receipt_id)
        }
        LifecycleDirectoryRoleV2::PostPmArtifactsRoot => root.join("post-pm-artifacts"),
        LifecycleDirectoryRoleV2::GuestPairingsRoot => root.join("guest-pairings"),
        LifecycleDirectoryRoleV2::LifecycleRoot => root.to_path_buf(),
    })
}

fn post_pm_state_literal(state: PostPmJournalStateV2) -> &'static str {
    match state {
        PostPmJournalStateV2::Prepared => "Prepared",
        PostPmJournalStateV2::EffectStarted => "EffectStarted",
        PostPmJournalStateV2::EffectObserved => "EffectObserved",
        PostPmJournalStateV2::Completed => "Completed",
    }
}

fn post_pm_artifact_label(label: &PostPmArtifactLabelV2) -> String {
    match label {
        PostPmArtifactLabelV2::MeasuredArtifact { entry_id } => {
            format!("measured-artifact-{entry_id}")
        }
        PostPmArtifactLabelV2::GuestServiceUnit => "guest-service-unit".to_string(),
        PostPmArtifactLabelV2::GuestSocketUnit => "guest-socket-unit".to_string(),
        PostPmArtifactLabelV2::FixedPublisherArtifact => "fixed-publisher-artifact".to_string(),
    }
}

fn require_uuid_v7(value: &str) -> Result<()> {
    let bytes = value.as_bytes();
    if bytes.len() != 36
        || ![8_usize, 13, 18, 23]
            .iter()
            .all(|offset| bytes[*offset] == b'-')
        || bytes[14] != b'7'
        || !matches!(bytes[19], b'8' | b'9' | b'a' | b'b')
        || bytes.iter().enumerate().any(|(offset, byte)| {
            !matches!(offset, 8 | 13 | 18 | 23)
                && (!byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
        })
    {
        bail!("finalizer target scope must be an exact UUIDv7")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use substrate_common::macos_retirement_v2::host_target_role_for_locator_v2;

    const SCOPE: &str = "019fffe0-0000-7000-8000-000000000001";

    fn target(ordinal: u16, locator: HostResourceLocatorV2) -> HostTargetIdentityV2 {
        HostTargetIdentityV2 {
            ordinal,
            role: host_target_role_for_locator_v2(&locator),
            locator,
            expected_before_sha256: format!("{ordinal:02x}").repeat(32),
        }
    }

    #[test]
    fn disposable_target_set_is_closed_and_lane_separated() {
        let disposable = derive_targets(TargetSetKindV2::DisposableCapability, SCOPE).unwrap();
        assert_eq!(disposable.len(), 8);
        assert!(matches!(
            disposable[0].target,
            FixedTarget::DisposableCapabilityControl {
                control: DisposableCapabilityControlV2::Sign
            }
        ));
        assert!(matches!(
            disposable[5].target,
            FixedTarget::SigningKey { .. }
        ));
        assert_eq!(disposable[7].role, HostTargetRoleV2::TerminalLatch);
        assert!(derive_targets(TargetSetKindV2::ProspectiveHost, SCOPE).is_err());
        assert!(derive_targets(TargetSetKindV2::DisposableCapability, "../product").is_err());
    }

    #[test]
    fn repeatable_lifecycle_locators_keep_ordinal_and_identity() {
        use HostResourceLocatorV2 as L;
        let ledger = vec![
            target(1, L::PublisherService),
            target(2, L::PublisherEndpoint),
            target(
                3,
                L::GenericPassword {
                    role: GenericPasswordRoleV2::CurrentAnchor,
                },
            ),
            target(
                4,
                L::GenericPassword {
                    role: GenericPasswordRoleV2::GuestRetirementCas,
                },
            ),
            target(
                5,
                L::GenericPassword {
                    role: GenericPasswordRoleV2::RetirementResourceIndex,
                },
            ),
            target(
                6,
                L::GenericPassword {
                    role: GenericPasswordRoleV2::RetirementCas,
                },
            ),
            target(
                7,
                L::GenericPassword {
                    role: GenericPasswordRoleV2::R6TerminalAcknowledgement,
                },
            ),
            target(
                8,
                L::LifecycleFile {
                    role: LifecycleFileRoleV2::Head,
                },
            ),
            target(9, L::SigningKey),
            target(
                10,
                L::FixedFile {
                    role: FixedFileRoleV2::PublisherHelper,
                },
            ),
            target(
                11,
                L::FixedFile {
                    role: FixedFileRoleV2::PublisherLaunchdPlist,
                },
            ),
            target(
                12,
                L::FixedFile {
                    role: FixedFileRoleV2::InstallProvenance,
                },
            ),
            target(
                13,
                L::KeychainCasLock {
                    account: GenericPasswordRoleV2::CurrentAnchor,
                },
            ),
            target(
                14,
                L::KeychainCasLock {
                    account: GenericPasswordRoleV2::GuestRetirementCas,
                },
            ),
            target(
                15,
                L::KeychainCasLock {
                    account: GenericPasswordRoleV2::RetirementResourceIndex,
                },
            ),
            target(
                16,
                L::KeychainCasLock {
                    account: GenericPasswordRoleV2::RetirementCas,
                },
            ),
            target(
                17,
                L::KeychainCasLock {
                    account: GenericPasswordRoleV2::R6TerminalAcknowledgement,
                },
            ),
            target(
                18,
                L::LifecycleTargetLock {
                    target: LifecycleFileRoleV2::Head,
                },
            ),
            target(
                19,
                L::LifecycleDirectory {
                    role: LifecycleDirectoryRoleV2::LifecycleRoot,
                },
            ),
            target(20, L::RetirementTerminalLatch),
        ];
        let targets =
            derive_targets_from_ledger(TargetSetKindV2::ProspectiveHost, SCOPE, &ledger).unwrap();
        assert_eq!(targets.len(), 20);
        assert_eq!(targets[3].role, HostTargetRoleV2::ProtectedWrapper);
        assert_eq!(targets[4].role, HostTargetRoleV2::ProtectedWrapper);
        assert_ne!(
            targets[3].effect_identity_sha256,
            targets[4].effect_identity_sha256
        );
        assert!(matches!(
            &targets[3].target,
            FixedTarget::ProtectedGenericPassword { account, .. }
                if account == &format!("{SCOPE}:{GUEST_RETIREMENT_CAS_SUFFIX_V2}")
        ));
        assert!(matches!(
            &targets[4].target,
            FixedTarget::ProtectedGenericPassword { account, .. }
                if account == &format!("{SCOPE}:{RETIREMENT_RESOURCE_INDEX_SUFFIX_V2}")
        ));
        assert!(matches!(
            &targets[17].target,
            FixedTarget::DerivedFile { path }
                if path.starts_with(&format!("{PRODUCT_LIFECYCLE_ROOT_V1}/lifecycle-target-"))
                    && path.ends_with(".lock")
                    && path.len() == PRODUCT_LIFECYCLE_ROOT_V1.len() + 18 + 64 + 5
        ));
        assert!(matches!(
            &targets[19].target,
            FixedTarget::DerivedFile { path }
                if path == &format!("{FINALIZER_LATCH_ROOT_V2}/{SCOPE}.retirement-terminal.v2.latch")
        ));
    }

    #[test]
    fn target_lock_derivation_rejects_roles_without_a_retained_lock() {
        let locator = HostResourceLocatorV2::LifecycleTargetLock {
            target: LifecycleFileRoleV2::StageOneProfile {
                attempt_id: "019fffe0-0000-7000-8000-000000000002".to_string(),
            },
        };
        assert!(derive_one_target(TargetSetKindV2::ProspectiveHost, SCOPE, &locator).is_err());
    }
}
