use substrate_replay::replay::diagnostics::CleanupReport;

#[test]
fn cleanup_report_detects_netns_and_nft_leftovers() {
    let netns_output = "\
substrate-abc (id: 42)
ns-dev
substrate-002
";
    let nft_output = r#"
table inet substrate_abc {
    chain output { }
}
table ip filter
table bridge substrate_extra
"#;

    let report = CleanupReport::from_outputs(netns_output, nft_output);
    assert_eq!(
        report.netns_leftovers(),
        &["substrate-002".to_string(), "substrate-abc".to_string()]
    );
    assert_eq!(
        report.nft_table_leftovers(),
        &[
            ("bridge".to_string(), "substrate_extra".to_string()),
            ("inet".to_string(), "substrate_abc".to_string())
        ]
    );

    let descriptions = report.describe();
    assert!(
        descriptions
            .iter()
            .any(|line| line.contains("netns detected: substrate-abc")),
        "expected substrate-abc netns description"
    );
    assert!(
        descriptions
            .iter()
            .any(|line| line.contains("table=substrate_extra")),
        "expected substrate_extra table description"
    );
}

#[test]
fn cleanup_report_handles_clean_state() {
    let report = CleanupReport::from_outputs("ns-default\n", "table ip filter\n");
    assert!(!report.has_findings());
    assert!(report.describe().is_empty());
}
