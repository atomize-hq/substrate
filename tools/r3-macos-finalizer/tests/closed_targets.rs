use substrate_common::macos_retirement_v2::{
    host_target_role_for_locator_v2, DisposableCapabilityControlV2, FixedFileRoleV2,
    GenericPasswordRoleV2, HostResourceLocatorV2, HostTargetIdentityV2, HostTargetRoleV2,
    LifecycleDirectoryRoleV2, LifecycleFileRoleV2, TargetSetKindV2,
};
use substrate_r3_macos_finalizer::targets::{
    derive_targets, derive_targets_from_ledger, FixedTarget, FINALIZER_LATCH_ROOT_V2,
    GUEST_RETIREMENT_CAS_SUFFIX_V2, PRODUCT_KEYCHAIN_SERVICE_V1, PRODUCT_LIFECYCLE_ROOT_V1,
    RETIREMENT_CAS_SUFFIX_V2, RETIREMENT_RESOURCE_INDEX_SUFFIX_V2,
};

const SCOPE: &str = "019fffe0-0000-7000-8000-000000000001";

fn identity(index: usize, locator: HostResourceLocatorV2) -> HostTargetIdentityV2 {
    HostTargetIdentityV2 {
        ordinal: u16::try_from(index + 1).unwrap(),
        role: host_target_role_for_locator_v2(&locator),
        locator,
        expected_before_sha256: format!("{:02x}", index + 1).repeat(32),
    }
}

fn prospective_ledger() -> Vec<HostTargetIdentityV2> {
    use FixedFileRoleV2 as F;
    use GenericPasswordRoleV2 as G;
    use HostResourceLocatorV2 as L;
    let locators = vec![
        L::PublisherService,
        L::PublisherEndpoint,
        L::GenericPassword {
            role: G::CurrentAnchor,
        },
        L::GenericPassword {
            role: G::GuestRetirementCas,
        },
        L::GenericPassword {
            role: G::RetirementResourceIndex,
        },
        L::GenericPassword {
            role: G::RetirementCas,
        },
        L::GenericPassword {
            role: G::R6TerminalAcknowledgement,
        },
        L::LifecycleFile {
            role: LifecycleFileRoleV2::Head,
        },
        L::SigningKey,
        L::FixedFile {
            role: F::PublisherHelper,
        },
        L::FixedFile {
            role: F::PublisherLaunchdPlist,
        },
        L::FixedFile {
            role: F::InstallProvenance,
        },
        L::KeychainCasLock {
            account: G::CurrentAnchor,
        },
        L::KeychainCasLock {
            account: G::GuestRetirementCas,
        },
        L::KeychainCasLock {
            account: G::RetirementResourceIndex,
        },
        L::KeychainCasLock {
            account: G::RetirementCas,
        },
        L::KeychainCasLock {
            account: G::R6TerminalAcknowledgement,
        },
        L::LifecycleTargetLock {
            target: LifecycleFileRoleV2::Head,
        },
        L::LifecycleDirectory {
            role: LifecycleDirectoryRoleV2::LifecycleRoot,
        },
        L::RetirementTerminalLatch,
    ];
    locators
        .into_iter()
        .enumerate()
        .map(|(index, locator)| identity(index, locator))
        .collect()
}

#[test]
fn prospective_targets_derive_exactly_from_the_signed_repeatable_ledger() {
    let targets = derive_targets_from_ledger(
        TargetSetKindV2::ProspectiveHost,
        SCOPE,
        &prospective_ledger(),
    )
    .unwrap();
    assert_eq!(targets.len(), 20);
    assert_eq!(targets[3].role, HostTargetRoleV2::ProtectedWrapper);
    assert_eq!(targets[4].role, HostTargetRoleV2::ProtectedWrapper);
    assert_ne!(
        targets[3].effect_identity_sha256,
        targets[4].effect_identity_sha256
    );
    assert!(matches!(
        &targets[3].target,
        FixedTarget::ProtectedGenericPassword { service, account }
            if *service == PRODUCT_KEYCHAIN_SERVICE_V1
                && account == &format!("{SCOPE}:{GUEST_RETIREMENT_CAS_SUFFIX_V2}")
    ));
    assert!(matches!(
        &targets[4].target,
        FixedTarget::ProtectedGenericPassword { service, account }
            if *service == PRODUCT_KEYCHAIN_SERVICE_V1
                && account == &format!("{SCOPE}:{RETIREMENT_RESOURCE_INDEX_SUFFIX_V2}")
    ));
    assert!(matches!(
        &targets[5].target,
        FixedTarget::ProtectedGenericPassword { account, .. }
            if account == &format!("{SCOPE}:{RETIREMENT_CAS_SUFFIX_V2}")
    ));
    assert!(matches!(
        &targets[7].target,
        FixedTarget::DerivedFile { path }
            if path == &format!("{PRODUCT_LIFECYCLE_ROOT_V1}/head.v1.json")
    ));
    assert!(matches!(
        &targets[17].target,
        FixedTarget::DerivedFile { path }
            if path.starts_with(&format!("{PRODUCT_LIFECYCLE_ROOT_V1}/lifecycle-target-"))
                && path.ends_with(".lock")
    ));
    assert!(matches!(
        &targets[19].target,
        FixedTarget::DerivedFile { path }
            if path == &format!("{FINALIZER_LATCH_ROOT_V2}/{SCOPE}.retirement-terminal.v2.latch")
    ));
    let encoded = serde_json::to_string(&targets).unwrap();
    assert!(!encoded.contains("lifecycle-v2"));
}

#[test]
fn disposable_targets_are_the_exact_sealed_control_and_surrogate_sequence() {
    let targets = derive_targets(TargetSetKindV2::DisposableCapability, SCOPE).unwrap();
    assert_eq!(targets.len(), 8);
    assert_eq!(
        targets.iter().map(|target| target.role).collect::<Vec<_>>(),
        vec![
            HostTargetRoleV2::CapabilitySignControl,
            HostTargetRoleV2::CapabilityExportControl,
            HostTargetRoleV2::CapabilityAclMutationControl,
            HostTargetRoleV2::CapabilityWrongKeyDeleteControl,
            HostTargetRoleV2::ProtectedWrapper,
            HostTargetRoleV2::Signer,
            HostTargetRoleV2::CurrentAnchorLock,
            HostTargetRoleV2::TerminalLatch,
        ]
    );
    assert!(matches!(
        targets[0].target,
        FixedTarget::DisposableCapabilityControl {
            control: DisposableCapabilityControlV2::Sign
        }
    ));
    assert!(matches!(
        &targets[5].target,
        FixedTarget::SigningKey { application_tag }
            if application_tag == format!("{SCOPE}:signing-key").as_bytes()
    ));
    let encoded = serde_json::to_string(&targets).unwrap();
    for product_literal in [
        "com.substrate.lifecycle.publisher.v1",
        "/Library/Application Support/Substrate/lifecycle-v1",
    ] {
        assert!(!encoded.contains(product_literal));
    }
}

#[test]
fn caller_cannot_substitute_scope_path_or_cross_target_lanes() {
    for invalid_scope in [
        "",
        ".",
        "..",
        "../product",
        "/absolute/product",
        "019fffe0-0000-6000-8000-000000000001",
        "019FFFE0-0000-7000-8000-000000000001",
        "019fffe0-0000-7000-c000-000000000001",
        "019fffe0-0000-7000-8000-000000000001/extra",
    ] {
        assert!(derive_targets(TargetSetKindV2::DisposableCapability, invalid_scope).is_err());
    }
    assert!(derive_targets_from_ledger(
        TargetSetKindV2::DisposableCapability,
        SCOPE,
        &prospective_ledger(),
    )
    .is_err());
}
