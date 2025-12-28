use world::landlock::{apply_filesystem_policy, detect_support, LandlockFilesystemPolicy};

#[test]
fn landlock_detect_support_is_consistent() {
    let support = detect_support();
    if support.supported {
        assert!(
            support.abi.is_some(),
            "supported=true should include an ABI version"
        );
        assert!(
            support.reason.is_none(),
            "supported=true should not include a reason"
        );
    } else {
        assert!(
            support.abi.is_none(),
            "supported=false should not include an ABI version"
        );
    }
}

#[test]
fn landlock_empty_policy_is_noop() {
    let policy = LandlockFilesystemPolicy {
        exec_paths: Vec::new(),
        read_paths: Vec::new(),
        write_paths: Vec::new(),
    };

    let report = apply_filesystem_policy(&policy);

    assert!(!report.attempted);
    assert!(!report.applied);
    assert_eq!(report.rules_added, 0);

    if report.support.supported {
        assert_eq!(
            report.reason.as_deref(),
            Some("landlock policy was empty; skipping")
        );
    }
}
