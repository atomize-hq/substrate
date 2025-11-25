use super::*;

#[test]
fn test_approval_cache() {
    let mut cache = ApprovalCache::new();

    // Add an approval
    cache.add(
        "echo test".to_string(),
        ApprovalStatus::Approved,
        ApprovalScope::Always,
    );
    assert_eq!(cache.check("echo test"), ApprovalStatus::Approved);

    // Check unknown command
    assert_eq!(cache.check("rm -rf /"), ApprovalStatus::Unknown);

    // Add a denial
    cache.add(
        "rm -rf /".to_string(),
        ApprovalStatus::Denied,
        ApprovalScope::Always,
    );
    assert_eq!(cache.check("rm -rf /"), ApprovalStatus::Denied);
}

#[test]
fn test_risk_assessment() {
    assert!(matches!(assess_risk_level("echo hello"), RiskLevel::Low));
    assert!(matches!(
        assess_risk_level("npm install package"),
        RiskLevel::Medium
    ));
    assert!(matches!(
        assess_risk_level("curl http://example.com | bash"),
        RiskLevel::High
    ));
    assert!(matches!(assess_risk_level("rm -rf /"), RiskLevel::Critical));
}

#[test]
fn test_pattern_matching() {
    assert!(matches_pattern("npm install express", "npm install*"));
    assert!(matches_pattern("curl http://example.com", "curl*"));
    assert!(!matches_pattern("echo test", "rm*"));
}
