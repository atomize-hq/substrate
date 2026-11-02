use super::*;

#[test]
fn parses_version_output_fields() {
    let parsed = version::parse_version_output("codex v3.4.5-nightly (commit abc1234)");
    assert_eq!(parsed.semantic, Some((3, 4, 5)));
    assert_eq!(parsed.channel, CodexReleaseChannel::Nightly);
    assert_eq!(parsed.commit.as_deref(), Some("abc1234"));
    assert_eq!(
        parsed.raw,
        "codex v3.4.5-nightly (commit abc1234)".to_string()
    );
}

#[test]
fn update_advisory_detects_newer_release() {
    let capabilities = capabilities_with_version("codex 1.0.0");
    let latest = CodexLatestReleases {
        stable: Some(Version::parse("1.1.0").unwrap()),
        ..Default::default()
    };
    let advisory = update_advisory_from_capabilities(&capabilities, &latest);
    assert_eq!(advisory.status, CodexUpdateStatus::UpdateRecommended);
    assert!(advisory.is_update_recommended());
    assert_eq!(
        advisory
            .latest_release
            .as_ref()
            .map(|release| release.version.clone()),
        latest.stable
    );
}

#[test]
fn update_advisory_handles_unknown_local_version() {
    let capabilities = capabilities_without_version();
    let latest = CodexLatestReleases {
        stable: Some(Version::parse("3.2.1").unwrap()),
        ..Default::default()
    };
    let advisory = update_advisory_from_capabilities(&capabilities, &latest);
    assert_eq!(advisory.status, CodexUpdateStatus::UnknownLocalVersion);
    assert!(advisory.is_update_recommended());
    assert!(advisory
        .notes
        .iter()
        .any(|note| note.contains("could not be parsed")));
}

#[test]
fn update_advisory_marks_up_to_date() {
    let capabilities = capabilities_with_version("codex 2.0.1");
    let latest = CodexLatestReleases {
        stable: Some(Version::parse("2.0.1").unwrap()),
        ..Default::default()
    };
    let advisory = update_advisory_from_capabilities(&capabilities, &latest);
    assert_eq!(advisory.status, CodexUpdateStatus::UpToDate);
    assert!(!advisory.is_update_recommended());
}

#[test]
fn update_advisory_falls_back_when_channel_missing() {
    let capabilities = capabilities_with_version("codex 2.0.0-beta");
    let latest = CodexLatestReleases {
        stable: Some(Version::parse("2.0.1").unwrap()),
        ..Default::default()
    };
    let advisory = update_advisory_from_capabilities(&capabilities, &latest);
    assert_eq!(advisory.comparison_channel, CodexReleaseChannel::Stable);
    assert_eq!(advisory.status, CodexUpdateStatus::UpdateRecommended);
    assert!(advisory
        .notes
        .iter()
        .any(|note| note.contains("comparing against stable")));
}

#[test]
fn update_advisory_handles_local_newer_than_known() {
    let capabilities = capabilities_with_version("codex 2.0.0");
    let latest = CodexLatestReleases {
        stable: Some(Version::parse("1.9.9").unwrap()),
        ..Default::default()
    };
    let advisory = update_advisory_from_capabilities(&capabilities, &latest);
    assert_eq!(advisory.status, CodexUpdateStatus::LocalNewerThanKnown);
    assert!(!advisory.is_update_recommended());
    assert!(advisory
        .notes
        .iter()
        .any(|note| note.contains("newer than provided")));
}

#[test]
fn update_advisory_handles_missing_latest_metadata() {
    let capabilities = capabilities_with_version("codex 1.0.0");
    let latest = CodexLatestReleases::default();
    let advisory = update_advisory_from_capabilities(&capabilities, &latest);
    assert_eq!(advisory.status, CodexUpdateStatus::UnknownLatestVersion);
    assert!(!advisory.is_update_recommended());
    assert!(advisory
        .notes
        .iter()
        .any(|note| note.contains("advisory unavailable")));
}
