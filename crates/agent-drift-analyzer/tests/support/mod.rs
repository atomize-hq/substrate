#![allow(dead_code, unreachable_pub)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use agent_drift_analyzer::{
    analyze_bundle, AnalyzeRequest, AnalyzeResult, Checkpoint, DriftClass, DriftScore, DriftState,
    InputBundle,
};
use agent_session_compactor::{
    BundleFileV0_2, BundleManifest, CompactionKind, CompactionRow, DedupeGroup, DedupeGroupV0_2,
    ExportRowV0_2, RowRef, RowRefV0_2, SourceKind, UserMessageRole,
};
use camino::{Utf8Path, Utf8PathBuf};
use tempfile::TempDir;

pub const ACCEPTANCE_CASE_IDS: [&str; 4] = [
    "019e93fa-60d4-73d1-9092-014130b60e14",
    "019e940c-a91b-7fe0-a967-b0bdd595b581",
    "019e943c-668e-7a03-992b-6a98cf3055da",
    "019e894a-86c9-71e3-b57b-e3d3285f0988",
];

pub const ACCEPTANCE_EXCLUDED_CASES: [(&str, &str); 3] = [
    (
        "019e93f8-a5e9-7490-ac1a-955b74c92ad0",
        "delegated session remains outside the analyzer-local success-tail corpus",
    ),
    (
        "019e9406-6736-79a2-946b-8a603e557422",
        "delegated session remains outside the analyzer-local success-tail corpus",
    ),
    (
        "019e9401-9d69-7190-a43e-9ee3be08b369",
        "non-success-tail session remains outside the analyzer-local success-tail corpus",
    ),
];

const ACCEPTANCE_FIXTURE_ROOT: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/acceptance");

pub struct BundleFixture {
    pub _temp_dir: TempDir,
    pub input_dir: Utf8PathBuf,
    pub output_dir: Utf8PathBuf,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
pub struct AcceptanceExpectedScore {
    pub state: DriftState,
    pub flagged: bool,
    pub raw_score: u8,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
pub struct AcceptanceExpected {
    pub session_id: String,
    pub source_artifact: String,
    pub final_dead_end_thrash: AcceptanceExpectedScore,
}

pub struct AcceptanceFixture {
    pub _temp_dir: TempDir,
    pub input_dir: Utf8PathBuf,
    pub output_dir: Utf8PathBuf,
    pub expected: AcceptanceExpected,
}

impl BundleFixture {
    pub fn sample() -> Self {
        Self::from_rows(
            sample_archival_rows(),
            sample_compact_rows(),
            sample_dedupe_groups(),
        )
    }

    pub fn clean_recovery() -> Self {
        Self::from_rows(
            sample_clean_recovery_archival_rows(),
            sample_clean_recovery_compact_rows(),
            sample_dedupe_groups(),
        )
    }

    pub fn from_compact_rows(compact_rows: Vec<CompactionRow>) -> Self {
        let (archival_rows, dedupe_groups) = synthesize_contract_valid_repetition(&compact_rows);
        Self::from_rows(archival_rows, compact_rows, dedupe_groups)
    }

    pub fn from_rows(
        archival_rows: Vec<CompactionRow>,
        compact_rows: Vec<CompactionRow>,
        dedupe_groups: Vec<DedupeGroup>,
    ) -> Self {
        let temp_dir = TempDir::new().expect("temp dir");
        let root = Utf8Path::from_path(temp_dir.path()).expect("utf8 temp dir");
        let input_dir = root.join("input");
        let output_dir = root.join("output");
        fs::create_dir_all(&input_dir).expect("create input dir");
        let file_registry = build_file_registry(&archival_rows, &compact_rows, &dedupe_groups);

        let manifest = BundleManifest {
            schema_version: "v0.2".to_string(),
            generated_at: time::OffsetDateTime::from_unix_timestamp(1_717_000_000)
                .expect("timestamp"),
            codex_home: Utf8PathBuf::from("/tmp/.codex"),
            output_dir: input_dir.clone(),
            discovered_file_count: 1,
            archival_row_count: archival_rows.len(),
            compact_row_count: compact_rows.len(),
            dedupe_group_count: dedupe_groups.len(),
            session_ids: collect_session_ids(&archival_rows, &compact_rows),
            files: file_registry.files.clone(),
        };

        fs::write(
            input_dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest).expect("manifest json"),
        )
        .expect("write manifest");
        write_jsonl(
            input_dir.join("rows.archival.jsonl"),
            &export_rows(&archival_rows, &file_registry),
        );
        write_jsonl(
            input_dir.join("rows.compact.jsonl"),
            &export_rows(&compact_rows, &file_registry),
        );
        write_jsonl(
            input_dir.join("dedupe-audit.jsonl"),
            &export_dedupe_groups(&dedupe_groups, &file_registry),
        );

        Self {
            _temp_dir: temp_dir,
            input_dir,
            output_dir,
        }
    }
}

pub fn analyze_sample_bundle() -> agent_drift_analyzer::AnalyzeResult {
    let fixture = BundleFixture::sample();
    analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle")
}

pub fn analyze_clean_recovery_bundle() -> agent_drift_analyzer::AnalyzeResult {
    let fixture = BundleFixture::clean_recovery();
    analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze clean recovery bundle")
}

pub fn load_sample_bundle() -> InputBundle {
    let fixture = BundleFixture::sample();
    agent_drift_analyzer::input::load_bundle(&fixture.input_dir).expect("load bundle")
}

pub fn read_checkpoints(path: &Utf8Path) -> Vec<Checkpoint> {
    fs::read_to_string(path)
        .expect("read checkpoints")
        .lines()
        .map(|line| serde_json::from_str(line).expect("checkpoint json"))
        .collect()
}

pub fn assert_score_flagged(checkpoint: &Checkpoint, class: DriftClass) {
    assert!(
        checkpoint
            .drift_scores
            .iter()
            .any(|score| score.class == class && score.flagged),
        "expected {class:?} to be flagged"
    );
}

impl AcceptanceFixture {
    pub fn load(case_id: &str) -> Self {
        let input_dir = acceptance_fixture_dir(case_id);
        let expected = read_json(input_dir.join("expected.json").as_ref());
        let temp_dir = TempDir::new().expect("temp dir");
        let root = Utf8Path::from_path(temp_dir.path()).expect("utf8 temp dir");
        let output_dir = root.join("output");
        fs::create_dir_all(&output_dir).expect("create output dir");

        Self {
            _temp_dir: temp_dir,
            input_dir,
            output_dir,
            expected,
        }
    }

    pub fn request(&self) -> AnalyzeRequest {
        AnalyzeRequest {
            input_dir: self.input_dir.clone(),
            output_dir: self.output_dir.clone(),
        }
    }

    pub fn analyze(&self) -> AnalyzeResult {
        analyze_bundle(&self.request()).expect("analyze acceptance case")
    }

    pub fn final_dead_end_thrash<'a>(&self, result: &'a AnalyzeResult) -> &'a DriftScore {
        result
            .sessions
            .iter()
            .find(|session| session.session_id == self.expected.session_id)
            .and_then(|session| session.checkpoints.last())
            .and_then(|checkpoint| {
                checkpoint
                    .drift_scores
                    .iter()
                    .find(|score| score.class == DriftClass::DeadEndThrash)
            })
            .expect("final dead_end_thrash score")
    }
}

pub fn load_acceptance_case(case_id: &str) -> AcceptanceFixture {
    AcceptanceFixture::load(case_id)
}

fn write_jsonl<T: serde::Serialize>(path: Utf8PathBuf, items: &[T]) {
    let body = items
        .iter()
        .map(|item| serde_json::to_string(item).expect("json line"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(path, format!("{body}\n")).expect("write jsonl");
}

fn acceptance_fixture_dir(case_id: &str) -> Utf8PathBuf {
    let input_dir = Utf8PathBuf::from(ACCEPTANCE_FIXTURE_ROOT).join(case_id);
    assert!(
        input_dir.is_dir(),
        "missing acceptance fixture {case_id}; Packet R2 tests must not fall back to target/ or ~/.codex"
    );
    input_dir
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Utf8Path) -> T {
    serde_json::from_str(&fs::read_to_string(path).expect("read json artifact"))
        .expect("parse json artifact")
}

struct TestFileRegistry {
    files: Vec<BundleFileV0_2>,
    ids_by_path: BTreeMap<Utf8PathBuf, u32>,
    turn_ids_by_path: BTreeMap<Utf8PathBuf, BTreeMap<String, u16>>,
}

fn build_file_registry(
    archival_rows: &[CompactionRow],
    compact_rows: &[CompactionRow],
    dedupe_groups: &[DedupeGroup],
) -> TestFileRegistry {
    let mut paths = archival_rows
        .iter()
        .map(|row| row.source_file.clone())
        .chain(compact_rows.iter().map(|row| row.source_file.clone()))
        .collect::<BTreeSet<_>>();
    let mut turns_by_path: BTreeMap<Utf8PathBuf, BTreeSet<String>> = BTreeMap::new();
    for row in archival_rows.iter().chain(compact_rows.iter()) {
        if let Some(turn_id) = row.turn_id.as_ref() {
            turns_by_path
                .entry(row.source_file.clone())
                .or_default()
                .insert(turn_id.clone());
        }
    }
    for group in dedupe_groups {
        paths.insert(group.representative.source_file.clone());
        for duplicate in &group.duplicates {
            paths.insert(duplicate.source_file.clone());
        }
    }

    let mut files = Vec::with_capacity(paths.len());
    let mut ids_by_path = BTreeMap::new();
    let mut turn_ids_by_path = BTreeMap::new();
    let session_ids_by_path = collect_session_ids_by_path(archival_rows, compact_rows);
    for (index, path) in paths.into_iter().enumerate() {
        let id = u32::try_from(index).expect("test file id");
        let turns = turns_by_path
            .remove(&path)
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>();
        let mut turn_ids = BTreeMap::new();
        for (turn_index, turn_id) in turns.iter().enumerate() {
            turn_ids.insert(
                turn_id.clone(),
                u16::try_from(turn_index).expect("test turn id"),
            );
        }
        files.push(BundleFileV0_2 {
            id,
            path: path.clone(),
            session_id: session_ids_by_path.get(&path).cloned().flatten(),
            turns,
        });
        ids_by_path.insert(path, id);
        turn_ids_by_path.insert(files.last().expect("file").path.clone(), turn_ids);
    }

    TestFileRegistry {
        files,
        ids_by_path,
        turn_ids_by_path,
    }
}

fn collect_session_ids(
    archival_rows: &[CompactionRow],
    compact_rows: &[CompactionRow],
) -> Vec<String> {
    archival_rows
        .iter()
        .chain(compact_rows.iter())
        .filter_map(|row| row.session_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn collect_session_ids_by_path(
    archival_rows: &[CompactionRow],
    compact_rows: &[CompactionRow],
) -> BTreeMap<Utf8PathBuf, Option<String>> {
    let mut session_ids_by_path = BTreeMap::new();
    for row in archival_rows.iter().chain(compact_rows.iter()) {
        let previous = session_ids_by_path.insert(row.source_file.clone(), row.session_id.clone());
        assert!(
            previous
                .as_ref()
                .is_none_or(|session_id| session_id == &row.session_id),
            "test fixture path {} reused across sessions",
            row.source_file
        );
    }
    session_ids_by_path
}

fn export_rows(rows: &[CompactionRow], registry: &TestFileRegistry) -> Vec<ExportRowV0_2> {
    rows.iter()
        .map(|row| ExportRowV0_2 {
            source_file_id: *registry
                .ids_by_path
                .get(&row.source_file)
                .expect("registered path"),
            source_kind: row.source_kind,
            turn_id_ref: row.turn_id.as_ref().map(|turn_id| {
                *registry
                    .turn_ids_by_path
                    .get(&row.source_file)
                    .and_then(|turn_ids| turn_ids.get(turn_id))
                    .expect("registered turn id")
            }),
            event_index: row.event_index,
            row_ordinal: row.row_ordinal,
            timestamp: row.timestamp,
            kind: row.kind,
            user_message_role: row.user_message_role,
            dedupe_identity: row.dedupe_identity.clone(),
            text: row.text.clone(),
            text_hash_hex: row.text_hash_hex.clone(),
        })
        .collect()
}

fn export_dedupe_groups(
    groups: &[DedupeGroup],
    registry: &TestFileRegistry,
) -> Vec<DedupeGroupV0_2> {
    groups
        .iter()
        .map(|group| DedupeGroupV0_2 {
            kind: group.kind,
            canonical_text_hash_hex: group.canonical_text_hash_hex.clone(),
            representative: export_row_ref(&group.representative, registry),
            duplicates: group
                .duplicates
                .iter()
                .map(|row_ref| export_row_ref(row_ref, registry))
                .collect(),
        })
        .collect()
}

fn export_row_ref(row_ref: &RowRef, registry: &TestFileRegistry) -> RowRefV0_2 {
    RowRefV0_2 {
        source_file_id: *registry
            .ids_by_path
            .get(&row_ref.source_file)
            .expect("registered ref path"),
        event_index: row_ref.event_index,
        row_ordinal: row_ref.row_ordinal,
    }
}

fn synthesize_contract_valid_repetition(
    compact_rows: &[CompactionRow],
) -> (Vec<CompactionRow>, Vec<DedupeGroup>) {
    let mut archival_rows = compact_rows.to_vec();
    let Some(representative_seed) = compact_rows
        .iter()
        .find(|row| row.kind == CompactionKind::ToolCall)
        .or_else(|| compact_rows.last())
    else {
        return (archival_rows, Vec::new());
    };

    let max_event_index = compact_rows
        .iter()
        .map(|row| row.event_index)
        .max()
        .unwrap_or(0);
    let max_line_number = compact_rows
        .iter()
        .map(|row| row.line_number)
        .max()
        .unwrap_or(0);
    let representative = synthetic_archival_repeat_row(
        representative_seed,
        max_event_index + 1,
        max_line_number + 1,
    );
    let duplicate = synthetic_archival_repeat_row(
        representative_seed,
        max_event_index + 2,
        max_line_number + 2,
    );
    let dedupe_groups = vec![DedupeGroup {
        kind: representative.kind,
        canonical_text_hash_hex: format!("synthetic-{}", representative.text_hash_hex),
        representative: RowRef::from_row(&representative),
        duplicates: vec![RowRef::from_row(&duplicate)],
    }];
    archival_rows.push(representative);
    archival_rows.push(duplicate);
    (archival_rows, dedupe_groups)
}

fn synthetic_archival_repeat_row(
    seed: &CompactionRow,
    event_index: usize,
    line_number: usize,
) -> CompactionRow {
    let mut row = seed.clone();
    row.event_index = event_index;
    row.line_number = line_number;
    row.row_ordinal = 0;
    row
}

fn sample_archival_rows() -> Vec<CompactionRow> {
    vec![
        row(
            0,
            1,
            0,
            CompactionKind::UserMessage,
            "/goal Implement Packet 6 using docs/specs/agent-drift-analyzer-v0.1-plan.md and docs/specs/agent-drift-analyzer-v0.1-spec.md. Verify with `cargo build -p agent-drift-analyzer` and `cargo test -p agent-drift-analyzer -- --nocapture`.",
            None,
        ),
        row(
            1,
            2,
            0,
            CompactionKind::SystemMessage,
            "Use crates/agent-drift-analyzer/src/lib.rs and crates/agent-drift-analyzer/src/input.rs as the library-first surface.",
            None,
        ),
        tool_row(
            2,
            3,
            "{\"command\":\"sed -n '1,200p' README.md\",\"workdir\":\"/repo\"}",
            Some("{\"call_id\":\"call-1\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"),
        ),
        tool_row(
            3,
            4,
            "{\"command\":\"cargo build -p agent-drift-analyzer\",\"workdir\":\"/repo\"}",
            Some("{\"call_id\":\"call-2\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"),
        ),
        tool_row(
            4,
            5,
            "{\"command\":\"cargo build -p agent-drift-analyzer\",\"workdir\":\"/repo\"}",
            Some("{\"call_id\":\"call-3\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"),
        ),
        tool_row(
            5,
            6,
            "{\"command\":\"cargo build -p agent-drift-analyzer\",\"workdir\":\"/repo\"}",
            Some("{\"call_id\":\"call-4\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"),
        ),
        row(
            6,
            7,
            0,
            CompactionKind::ToolOutput,
            "error: failed to compile analyzer",
            None,
        ),
        row(
            7,
            8,
            0,
            CompactionKind::ToolOutput,
            "error: failed to compile analyzer",
            None,
        ),
        tool_row(
            8,
            9,
            "{\"command\":\"sed -n '1,120p' crates/agent-drift-analyzer/src/lib.rs\",\"workdir\":\"/repo\"}",
            Some("{\"call_id\":\"call-6\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"),
        ),
        row(
            9,
            10,
            0,
            CompactionKind::AssistantMessage,
            "I found the build loop. Next I’m patching lib.rs and rerunning the analyzer checks.",
            None,
        ),
        tool_row(
            10,
            11,
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/lib.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
            Some("{\"call_id\":\"call-5\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"),
        ),
        tool_row(
            11,
            12,
            "{\"command\":\"cargo test -p agent-drift-analyzer -- --nocapture\",\"workdir\":\"/repo\"}",
            Some("{\"call_id\":\"call-7\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"),
        ),
        tool_row(
            12,
            13,
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Add File: /tmp/offscope-notes.md\\n+rogue\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
            Some("{\"call_id\":\"call-8\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"),
        ),
    ]
}

fn sample_compact_rows() -> Vec<CompactionRow> {
    sample_archival_rows()
        .into_iter()
        .enumerate()
        .filter(|(index, _)| !matches!(index, 4 | 7))
        .map(|(_, row)| row)
        .collect()
}

fn sample_clean_recovery_archival_rows() -> Vec<CompactionRow> {
    sample_archival_rows()
        .into_iter()
        .filter(|row| row.event_index != 12)
        .collect()
}

fn sample_clean_recovery_compact_rows() -> Vec<CompactionRow> {
    sample_clean_recovery_archival_rows()
        .into_iter()
        .enumerate()
        .filter(|(index, _)| !matches!(index, 4 | 7))
        .map(|(_, row)| row)
        .collect()
}

fn sample_dedupe_groups() -> Vec<DedupeGroup> {
    vec![DedupeGroup {
        kind: CompactionKind::ToolCall,
        canonical_text_hash_hex: "dup-hash".to_string(),
        representative: RowRef {
            source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
            event_index: 3,
            row_ordinal: 0,
        },
        duplicates: vec![RowRef {
            source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
            event_index: 4,
            row_ordinal: 0,
        }],
    }]
}

fn row(
    event_index: usize,
    line_number: usize,
    row_ordinal: usize,
    kind: CompactionKind,
    text: &str,
    dedupe_identity: Option<&str>,
) -> CompactionRow {
    CompactionRow {
        source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some("session-alpha".to_string()),
        turn_id: Some("turn-001".to_string()),
        event_index,
        line_number,
        row_ordinal,
        timestamp: None,
        kind,
        user_message_role: matches!(kind, CompactionKind::UserMessage)
            .then_some(UserMessageRole::Prompt),
        dedupe_identity: dedupe_identity.map(ToOwned::to_owned),
        text: text.to_string(),
        canonical_text: text.to_string(),
        text_hash_hex: format!("hash-{}", text.split_whitespace().collect::<String>()),
    }
}

fn tool_row(
    event_index: usize,
    line_number: usize,
    text: &str,
    dedupe_identity: Option<&str>,
) -> CompactionRow {
    row(
        event_index,
        line_number,
        0,
        CompactionKind::ToolCall,
        text,
        dedupe_identity,
    )
}
