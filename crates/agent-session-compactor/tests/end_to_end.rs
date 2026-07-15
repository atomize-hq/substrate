use anyhow as _;
use blake3 as _;
use std::fs;
use std::sync::Mutex;

use agent_session_compactor::{compact_codex_sessions, RunConfig};
use camino::Utf8Path;
use clap as _;
use codex as _;
use serde as _;
use serde_json as _;
use tempfile::TempDir;
use thiserror as _;
use time as _;
use time::macros::datetime;
use walkdir as _;

const EXPORT_FAIL_AT_ENV: &str = "AGENT_SESSION_COMPACTOR_EXPORT_FAIL_AT";
static EXPORT_ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn end_to_end_compaction_produces_stable_bundle_outputs() {
    with_export_failure(None, || {
        let codex_home = seeded_codex_home();
        let output_dir = TempDir::new().expect("output temp dir");
        let output_path = Utf8Path::from_path(output_dir.path())
            .expect("utf8 output path")
            .join("bundle");

        let config = RunConfig {
            codex_home: Some(
                Utf8Path::from_path(codex_home.path())
                    .expect("utf8 codex home")
                    .to_owned(),
            ),
            session_id: None,
            include_linked_children: false,
            output_dir: output_path.to_owned(),
            generated_at: Some(datetime!(2026-05-29 12:00:00 UTC)),
        };

        let first = compact_codex_sessions(&config).expect("first compaction");
        let first_manifest =
            fs::read_to_string(output_path.join("manifest.json")).expect("manifest");
        let first_archival =
            fs::read_to_string(output_path.join("rows.archival.jsonl")).expect("archival rows");
        let first_compact =
            fs::read_to_string(output_path.join("rows.compact.jsonl")).expect("compact rows");
        let first_audit =
            fs::read_to_string(output_path.join("dedupe-audit.jsonl")).expect("dedupe audit");
        let first_summary = fs::read_to_string(output_path.join("summary.md")).expect("summary");

        let second = compact_codex_sessions(&config).expect("second compaction");

        assert_eq!(first.manifest, second.manifest);
        assert_eq!(
            first_manifest,
            fs::read_to_string(output_path.join("manifest.json")).expect("manifest after rerun")
        );
        assert_eq!(
            first_archival,
            fs::read_to_string(output_path.join("rows.archival.jsonl"))
                .expect("archival rows after rerun")
        );
        assert_eq!(
            first_compact,
            fs::read_to_string(output_path.join("rows.compact.jsonl"))
                .expect("compact rows after rerun")
        );
        assert_eq!(
            first_audit,
            fs::read_to_string(output_path.join("dedupe-audit.jsonl"))
                .expect("dedupe audit after rerun")
        );
        assert_eq!(
            first_summary,
            fs::read_to_string(output_path.join("summary.md")).expect("summary after rerun")
        );

        assert!(first.manifest.archival_row_count >= first.manifest.compact_row_count);
        assert_eq!(first.manifest.session_ids, vec!["session-123".to_string()]);
    });
}

#[test]
fn end_to_end_failed_republish_preserves_the_last_complete_bundle() {
    let codex_home = seeded_codex_home();
    let output_dir = TempDir::new().expect("output temp dir");
    let output_path = Utf8Path::from_path(output_dir.path())
        .expect("utf8 output path")
        .join("bundle");

    let config = RunConfig {
        codex_home: Some(
            Utf8Path::from_path(codex_home.path())
                .expect("utf8 codex home")
                .to_owned(),
        ),
        session_id: None,
        include_linked_children: false,
        output_dir: output_path.to_owned(),
        generated_at: Some(datetime!(2026-05-29 12:00:00 UTC)),
    };

    let first = compact_codex_sessions(&config).expect("first compaction");
    let first_manifest = fs::read_to_string(output_path.join("manifest.json")).expect("manifest");
    let first_archival =
        fs::read_to_string(output_path.join("rows.archival.jsonl")).expect("archival rows");
    let first_compact =
        fs::read_to_string(output_path.join("rows.compact.jsonl")).expect("compact rows");
    let first_audit =
        fs::read_to_string(output_path.join("dedupe-audit.jsonl")).expect("dedupe audit");
    let first_summary = fs::read_to_string(output_path.join("summary.md")).expect("summary");

    let error = with_export_failure(Some("before_publish"), || {
        compact_codex_sessions(&config).expect_err("second compaction should fail before publish")
    });

    assert!(matches!(
        error,
        agent_session_compactor::CompactorError::Export(
            agent_session_compactor::ExportError::InjectedFailure { .. }
        )
    ));
    assert_eq!(
        first_manifest,
        fs::read_to_string(output_path.join("manifest.json")).expect("manifest after failed rerun")
    );
    assert_eq!(
        first_archival,
        fs::read_to_string(output_path.join("rows.archival.jsonl"))
            .expect("archival rows after failed rerun")
    );
    assert_eq!(
        first_compact,
        fs::read_to_string(output_path.join("rows.compact.jsonl"))
            .expect("compact rows after failed rerun")
    );
    assert_eq!(
        first_audit,
        fs::read_to_string(output_path.join("dedupe-audit.jsonl"))
            .expect("dedupe audit after failed rerun")
    );
    assert_eq!(
        first_summary,
        fs::read_to_string(output_path.join("summary.md")).expect("summary after failed rerun")
    );
    assert_eq!(first.manifest.session_ids, vec!["session-123".to_string()]);

    let staging_dirs = staging_entries(
        Utf8Path::from_path(output_dir.path()).expect("utf8 output root"),
        "bundle",
    );
    assert_eq!(staging_dirs.len(), 1);
}

#[test]
fn end_to_end_linked_child_opt_in_uses_exact_reciprocal_metadata() {
    with_export_failure(None, || {
        let codex_home = TempDir::new().expect("codex home temp dir");
        let sessions = codex_home.path().join("sessions/2026/07/15");
        fs::create_dir_all(&sessions).expect("create sessions directory");
        fs::write(
            sessions.join("rollout-session-parent-echo.jsonl"),
            include_str!("fixtures/delegation_links/multi_child/rollout-parent.jsonl"),
        )
        .expect("write parent fixture");
        fs::write(
        sessions.join("rollout-generic-child-one.jsonl"),
        format!(
            "{}{}",
            include_str!("fixtures/delegation_links/multi_child/rollout-child-one.jsonl"),
            "{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"sanitized child one work\"}]}}\n"
        ),
    )
    .expect("write first child fixture");
        fs::write(
        sessions.join("rollout-generic-child-two.jsonl"),
        format!(
            "{}{}",
            include_str!("fixtures/delegation_links/multi_child/rollout-child-two.jsonl"),
            "{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"sanitized child two work\"}]}}\n"
        ),
    )
    .expect("write second child fixture");
        fs::write(
            sessions.join("rollout-session-child-echo-one-decoy.jsonl"),
            include_str!("fixtures/delegation_links/single_agent/rollout-session.jsonl"),
        )
        .expect("write filename-only decoy");

        let output_dir = TempDir::new().expect("output temp dir");
        let codex_home = Utf8Path::from_path(codex_home.path())
            .expect("utf8 codex home")
            .to_owned();
        let ordinary_output = Utf8Path::from_path(output_dir.path())
            .expect("utf8 output")
            .join("ordinary");
        let linked_output = ordinary_output.with_file_name("linked");

        let ordinary = compact_codex_sessions(&RunConfig {
            codex_home: Some(codex_home.clone()),
            session_id: Some("session-parent-echo".to_string()),
            include_linked_children: false,
            output_dir: ordinary_output,
            generated_at: Some(datetime!(2026-07-15 12:00:00 UTC)),
        })
        .expect("ordinary parent-only compaction");
        assert_eq!(ordinary.manifest.session_ids, vec!["session-parent-echo"]);
        assert_eq!(ordinary.manifest.files.len(), 1);

        let linked_config = RunConfig {
            codex_home: Some(codex_home),
            session_id: Some("session-parent-echo".to_string()),
            include_linked_children: true,
            output_dir: linked_output,
            generated_at: Some(datetime!(2026-07-15 12:00:00 UTC)),
        };
        let linked =
            compact_codex_sessions(&linked_config).expect("linked direct-child compaction");

        assert_eq!(
            linked.manifest.session_ids,
            vec![
                "session-child-echo-one",
                "session-child-echo-two",
                "session-parent-echo",
            ]
        );
        assert_eq!(linked.manifest.files.len(), 3);
        assert!(linked.manifest.files.iter().all(|file| {
            file.session_id.as_deref() != Some("session-single-golf")
                && !file.path.as_str().contains("decoy")
        }));
        assert_eq!(linked.manifest.delegation_links.len(), 2);
        assert!(linked.manifest.delegation_links.iter().all(|link| {
            link.parent_session_id == "session-parent-echo"
                && link.state == agent_session_compactor::DelegationLinkState::Verified
        }));
        let repeated = compact_codex_sessions(&linked_config)
            .expect("deterministic linked-child recompaction");
        assert_eq!(linked.manifest, repeated.manifest);
    });
}

#[test]
fn end_to_end_linked_child_opt_in_records_deeper_residue_without_importing_it() {
    with_export_failure(None, || {
        let codex_home = TempDir::new().expect("codex home temp dir");
        let sessions = codex_home.path().join("sessions/2026/07/15");
        fs::create_dir_all(&sessions).expect("create sessions directory");
        fs::write(
            sessions.join("rollout-session-parent-foxtrot.jsonl"),
            include_str!("fixtures/delegation_links/nested_depth/rollout-parent.jsonl"),
        )
        .expect("write parent fixture");
        fs::write(
            sessions.join("rollout-generic-deeper-child.jsonl"),
            include_str!("fixtures/delegation_links/nested_depth/rollout-child.jsonl"),
        )
        .expect("write deeper child fixture");

        let output_dir = TempDir::new().expect("output temp dir");
        let result = compact_codex_sessions(&RunConfig {
            codex_home: Some(
                Utf8Path::from_path(codex_home.path())
                    .expect("utf8 codex home")
                    .to_owned(),
            ),
            session_id: Some("session-parent-foxtrot".to_string()),
            include_linked_children: true,
            output_dir: Utf8Path::from_path(output_dir.path())
                .expect("utf8 output")
                .join("bundle"),
            generated_at: Some(datetime!(2026-07-15 12:00:00 UTC)),
        })
        .expect("bounded linked-child compaction");

        assert_eq!(result.manifest.session_ids, vec!["session-parent-foxtrot"]);
        assert_eq!(result.manifest.files.len(), 1);
        assert_eq!(result.manifest.delegation_links.len(), 1);
        assert_eq!(
            result.manifest.delegation_links[0].state,
            agent_session_compactor::DelegationLinkState::DeeperResidue
        );
        assert_eq!(result.manifest.delegation_links[0].depth, Some(2));
    });
}

#[test]
fn end_to_end_linked_child_opt_in_fail_closes_non_verified_candidates() {
    let cases = [
        (
            "missing",
            "session-parent-bravo",
            vec![(
                "rollout-parent.jsonl",
                include_str!("fixtures/delegation_links/parent_only/rollout-parent.jsonl"),
            )],
            agent_session_compactor::DelegationLinkState::ParentOnly,
        ),
        (
            "conflict",
            "session-parent-delta",
            vec![
                (
                    "rollout-parent.jsonl",
                    include_str!("fixtures/delegation_links/conflict/rollout-parent.jsonl"),
                ),
                (
                    "rollout-child.jsonl",
                    include_str!("fixtures/delegation_links/conflict/rollout-child.jsonl"),
                ),
            ],
            agent_session_compactor::DelegationLinkState::ConflictingParent,
        ),
        (
            "duplicate",
            "session-parent-alpha",
            vec![
                (
                    "rollout-parent.jsonl",
                    include_str!("fixtures/delegation_links/reciprocal/rollout-parent.jsonl"),
                ),
                (
                    "rollout-child-a.jsonl",
                    include_str!("fixtures/delegation_links/reciprocal/rollout-child.jsonl"),
                ),
                (
                    "rollout-child-b.jsonl",
                    include_str!("fixtures/delegation_links/reciprocal/rollout-child.jsonl"),
                ),
            ],
            agent_session_compactor::DelegationLinkState::Duplicate,
        ),
        (
            "self",
            "session-parent-self",
            vec![(
                "rollout-parent.jsonl",
                concat!(
                    "{\"type\":\"session_meta\",\"payload\":{\"id\":\"session-parent-self\"}}\n",
                    "{\"type\":\"response_item\",\"payload\":{\"type\":\"function_call\",\"name\":\"spawn_agent\",\"call_id\":\"call-self\"}}\n",
                    "{\"type\":\"response_item\",\"payload\":{\"type\":\"function_call_output\",\"call_id\":\"call-self\",\"output\":\"{\\\"agent_id\\\":\\\"session-parent-self\\\"}\"}}\n",
                ),
            )],
            agent_session_compactor::DelegationLinkState::SelfLink,
        ),
        (
            "malformed",
            "session-parent-malformed",
            vec![(
                "rollout-parent.jsonl",
                concat!(
                    "{\"type\":\"session_meta\",\"payload\":{\"id\":\"session-parent-malformed\"}}\n",
                    "{\"type\":\"response_item\",\"payload\":{\"type\":\"function_call\",\"name\":\"spawn_agent\",\"call_id\":\"call-malformed\"}}\n",
                    "{\"type\":\"response_item\",\"payload\":{\"type\":\"function_call_output\",\"call_id\":\"call-malformed\",\"output\":\"{\\\"agent_id\\\":\\\"malformed child\\\"}\"}}\n",
                ),
            )],
            agent_session_compactor::DelegationLinkState::MalformedSessionId,
        ),
    ];

    for (name, root_session_id, files, expected_state) in cases {
        let result = compact_linked_fixture_case(name, root_session_id, &files);
        assert_eq!(result.manifest.session_ids, vec![root_session_id], "{name}");
        assert_eq!(result.manifest.delegation_links.len(), 1, "{name}");
        assert_eq!(
            result.manifest.delegation_links[0].state, expected_state,
            "{name}"
        );
    }
}

#[test]
fn end_to_end_linked_child_opt_in_rejects_ambiguous_root_artifacts() {
    with_export_failure(None, || {
        let codex_home = TempDir::new().expect("codex home temp dir");
        let sessions = codex_home.path().join("sessions/2026/07/15");
        fs::create_dir_all(&sessions).expect("create sessions directory");
        let root = "{\"type\":\"session_meta\",\"payload\":{\"id\":\"session-ambiguous\"}}\n";
        fs::write(sessions.join("rollout-root-a.jsonl"), root).expect("write first root");
        fs::write(sessions.join("rollout-root-b.jsonl"), root).expect("write second root");
        let output_dir = TempDir::new().expect("output temp dir");

        let error = compact_codex_sessions(&RunConfig {
            codex_home: Some(
                Utf8Path::from_path(codex_home.path())
                    .expect("utf8 codex home")
                    .to_owned(),
            ),
            session_id: Some("session-ambiguous".to_string()),
            include_linked_children: true,
            output_dir: Utf8Path::from_path(output_dir.path())
                .expect("utf8 output")
                .join("bundle"),
            generated_at: Some(datetime!(2026-07-15 12:00:00 UTC)),
        })
        .expect_err("ambiguous linked root must fail closed");

        assert!(matches!(
            error,
            agent_session_compactor::CompactorError::Discovery(
                agent_session_compactor::DiscoveryError::AmbiguousLinkedSession { .. }
            )
        ));
    });
}

fn compact_linked_fixture_case(
    name: &str,
    root_session_id: &str,
    files: &[(&str, &str)],
) -> agent_session_compactor::CompactionRunResult {
    let codex_home = TempDir::new().expect("codex home temp dir");
    let sessions = codex_home.path().join("sessions/2026/07/15");
    fs::create_dir_all(&sessions).expect("create sessions directory");
    for (file_name, contents) in files {
        fs::write(sessions.join(file_name), contents).expect("write sanitized fixture");
    }
    let output_dir = TempDir::new().expect("output temp dir");
    with_export_failure(None, || {
        compact_codex_sessions(&RunConfig {
            codex_home: Some(
                Utf8Path::from_path(codex_home.path())
                    .expect("utf8 codex home")
                    .to_owned(),
            ),
            session_id: Some(root_session_id.to_string()),
            include_linked_children: true,
            output_dir: Utf8Path::from_path(output_dir.path())
                .expect("utf8 output")
                .join(format!("bundle-{name}")),
            generated_at: Some(datetime!(2026-07-15 12:00:00 UTC)),
        })
        .expect("bounded linked-child compaction")
    })
}

fn seeded_codex_home() -> TempDir {
    let temp_dir = TempDir::new().expect("codex home temp dir");
    let rollout_dir = temp_dir.path().join("sessions/2026/05/29");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");
    fs::write(
        rollout_dir.join("rollout-session-123.jsonl"),
        concat!(
            "{\"timestamp\":\"2026-05-29T12:00:00Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"session-123\",\"base_instructions\":{\"text\":\"Base instructions\"}}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:01Z\",\"type\":\"turn_context\",\"payload\":{\"turn_id\":\"turn-abc\",\"user_instructions\":\"Repo-local rules\"}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:02Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"message\":\"Ship the packet\"}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:03Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"Packet complete\"}]}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:04Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"Packet complete\"}]}}\n"
        ),
    )
    .expect("write rollout fixture");
    temp_dir
}

fn staging_entries(parent_dir: &Utf8Path, bundle_name: &str) -> Vec<std::path::PathBuf> {
    let mut entries = fs::read_dir(parent_dir)
        .expect("read parent dir")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&format!(".{bundle_name}.staging-")))
        })
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn with_export_failure<T>(point: Option<&str>, f: impl FnOnce() -> T) -> T {
    let _guard = EXPORT_ENV_MUTEX.lock().expect("env mutex");
    let previous = std::env::var(EXPORT_FAIL_AT_ENV).ok();
    match point {
        Some(point) => std::env::set_var(EXPORT_FAIL_AT_ENV, point),
        None => std::env::remove_var(EXPORT_FAIL_AT_ENV),
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    match previous {
        Some(previous) => std::env::set_var(EXPORT_FAIL_AT_ENV, previous),
        None => std::env::remove_var(EXPORT_FAIL_AT_ENV),
    }
    match result {
        Ok(result) => result,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}
