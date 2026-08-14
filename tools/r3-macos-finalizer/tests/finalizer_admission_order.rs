#[test]
fn locks_precede_live_cas_and_retirement_cas_selection_is_typed() {
    let source = include_str!("../src/bin/finalizer.rs");
    let initial = source
        .split("fn handle_initial_request(")
        .nth(1)
        .expect("initial request handler")
        .split("fn handle_terminal_request(")
        .next()
        .expect("initial request boundary");
    let lock_construction = initial
        .find("MacNativeEffects::new(")
        .expect("lock constructor");
    let live_cas = initial
        .find("verify_live_protected_cas(")
        .expect("live CAS verification");
    let engine_acceptance = initial
        .find("FinalizerEngine::new(")
        .expect("engine acceptance");
    assert!(lock_construction < live_cas);
    assert!(live_cas < engine_acceptance);

    let verifier = source
        .split("fn verify_live_protected_cas(")
        .nth(1)
        .expect("live CAS verifier")
        .split("fn target_path(")
        .next()
        .expect("live CAS verifier boundary");
    assert!(verifier.contains("HostResourceLocatorV2::DisposableProtectedWrapper"));
    assert!(verifier.contains("GenericPasswordRoleV2::RetirementCas"));
    assert!(verifier.contains("wrappers.next().is_some()"));
    assert!(verifier.contains("read_receipt_bound_disposable_wrapper_data"));
    assert!(!verifier.contains("target.role == HostTargetRoleV2::ProtectedWrapper"));

    let native = include_str!("../src/native_effects.rs");
    let wrapper_read = native
        .split("pub fn read_receipt_bound_disposable_wrapper_data")
        .nth(1)
        .expect("receipt-bound disposable wrapper reader")
        .split("fn require_protecting_lock")
        .next()
        .expect("receipt-bound disposable wrapper boundary");
    assert!(wrapper_read.contains("self.require_protecting_lock(target.ordinal)"));
    assert!(wrapper_read.contains("expected.expected_before_sha256"));
    assert!(wrapper_read.contains("disposable_wrapper_observation_from_handle"));
}

#[test]
fn disposable_wrapper_identity_breaks_the_receipt_cas_cycle_under_current_lock() {
    let source = include_str!("../src/native_effects.rs");
    let identity = source
        .split("struct DisposableWrapperObservation")
        .nth(1)
        .expect("disposable wrapper observation")
        .split("pub struct MacNativeEffects")
        .next()
        .expect("disposable wrapper observation boundary");
    for physical_field in [
        "path:",
        "device:",
        "inode:",
        "uid:",
        "gid:",
        "mode:",
        "link_count:",
    ] {
        assert!(identity.contains(physical_field));
    }
    assert!(!identity.contains("content_sha256"));
    assert!(!identity.contains("size:"));
    assert!(!identity.contains("modified"));

    let lock_map = source
        .split("fn protected_target_lock_map(")
        .nth(1)
        .expect("protected target lock map")
        .split("pub fn file_observation(")
        .next()
        .expect("protected target lock map boundary");
    assert!(lock_map.contains("HostResourceLocatorV2::DisposableProtectedWrapper"));
    assert!(lock_map.contains("HostResourceLocatorV2::DisposableCurrentLock"));
}
