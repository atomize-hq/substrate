use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::{json, Value};
use tempfile::{tempdir, TempDir};

use super::super::normalize_add_dirs_v1;
use crate::AgentWrapperError;
#[cfg(windows)]
use std::path::{Component, Prefix};

// Top-level shape

#[test]
fn ad_c02_absent_key_returns_empty_vec() {
    let fixtures = AddDirFixtures::new();

    let normalized = normalize_add_dirs_v1(None, Some(fixtures.effective_working_dir()))
        .expect("absent key normalizes");

    assert!(normalized.is_empty());
}

#[test]
fn ad_c02_non_object_payload_uses_safe_root_message_without_leakage() {
    let fixtures = AddDirFixtures::new();
    let secret = "/tmp/secret-path";
    let payload = Value::String(secret.to_string());

    assert_invalid_message(
        Some(&payload),
        Some(fixtures.effective_working_dir()),
        "invalid agent_api.exec.add_dirs.v1",
        &[secret],
    );
}

// Closed-schema / container shape

#[test]
fn ad_c02_unknown_key_uses_safe_root_message_without_leakage() {
    let fixtures = AddDirFixtures::new();
    let secret = "/tmp/secret-path";
    let payload = json!({ "unexpected": secret });

    assert_invalid_message(
        Some(&payload),
        Some(fixtures.effective_working_dir()),
        "invalid agent_api.exec.add_dirs.v1",
        &[secret],
    );
}

#[test]
fn ad_c02_missing_dirs_uses_safe_root_message() {
    let fixtures = AddDirFixtures::new();
    let payload = json!({});

    assert_invalid_message(
        Some(&payload),
        Some(fixtures.effective_working_dir()),
        "invalid agent_api.exec.add_dirs.v1",
        &[],
    );
}

#[test]
fn ad_c02_non_array_dirs_uses_safe_container_message_without_leakage() {
    let fixtures = AddDirFixtures::new();
    let secret = "/tmp/not-an-array";
    let payload = json!({ "dirs": secret });

    assert_invalid_message(
        Some(&payload),
        Some(fixtures.effective_working_dir()),
        "invalid agent_api.exec.add_dirs.v1.dirs",
        &[secret],
    );
}

#[test]
fn ad_c02_empty_dirs_uses_safe_container_message() {
    let fixtures = AddDirFixtures::new();
    let payload = add_dirs_payload(Vec::<Value>::new());

    assert_invalid_message(
        Some(&payload),
        Some(fixtures.effective_working_dir()),
        "invalid agent_api.exec.add_dirs.v1.dirs",
        &[],
    );
}

#[test]
fn ad_c02_too_many_dirs_uses_safe_container_message_without_leakage() {
    let fixtures = AddDirFixtures::new();
    let oversized: Vec<String> = (0..17)
        .map(|index| format!("/tmp/secret-path-{index}"))
        .collect();
    let payload = add_dirs_payload(
        oversized
            .iter()
            .map(|path| Value::String(path.clone()))
            .collect(),
    );

    assert_invalid_message(
        Some(&payload),
        Some(fixtures.effective_working_dir()),
        "invalid agent_api.exec.add_dirs.v1.dirs",
        oversized
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice(),
    );
}

// Entry validation

#[test]
fn ad_c02_non_string_entry_uses_safe_indexed_message_without_leakage() {
    let fixtures = AddDirFixtures::new();
    let payload = add_dirs_payload(vec![json!(5)]);

    assert_invalid_message(
        Some(&payload),
        Some(fixtures.effective_working_dir()),
        "invalid agent_api.exec.add_dirs.v1.dirs[0]",
        &["5"],
    );
}

#[test]
fn ad_c02_trimmed_empty_entry_uses_safe_indexed_message_without_leakage() {
    let fixtures = AddDirFixtures::new();
    let raw = " \t\n ";
    let payload = add_dirs_payload(vec![json!(raw)]);

    assert_invalid_message(
        Some(&payload),
        Some(fixtures.effective_working_dir()),
        "invalid agent_api.exec.add_dirs.v1.dirs[0]",
        &[raw],
    );
}

#[test]
fn ad_c02_over_byte_limit_entry_uses_safe_indexed_message_without_leakage() {
    let fixtures = AddDirFixtures::new();
    let oversized = "a".repeat(1025);
    let payload = add_dirs_payload(vec![json!(oversized)]);

    assert_invalid_message(
        Some(&payload),
        Some(fixtures.effective_working_dir()),
        "invalid agent_api.exec.add_dirs.v1.dirs[0]",
        &[&"a".repeat(64)],
    );
}

#[test]
fn ad_c02_over_utf8_byte_limit_entry_with_sub_1024_chars_uses_safe_indexed_message_without_leakage()
{
    let fixtures = AddDirFixtures::new();
    let oversized = "é".repeat(600);
    assert!(
        oversized.chars().count() < 1024,
        "test requires sub-1024 chars"
    );
    assert!(
        oversized.len() > 1024,
        "test requires over-1024 UTF-8 bytes"
    );
    let payload = add_dirs_payload(vec![json!(oversized)]);

    assert_invalid_message(
        Some(&payload),
        Some(fixtures.effective_working_dir()),
        "invalid agent_api.exec.add_dirs.v1.dirs[0]",
        &["éééé"],
    );
}

// Filesystem validation

#[test]
fn ad_c02_missing_directory_uses_safe_indexed_message_without_leakage() {
    let fixtures = AddDirFixtures::new();
    let missing = fixtures.missing_relative("missing-dir");
    let payload = add_dirs_payload(vec![json!(missing)]);

    assert_invalid_message(
        Some(&payload),
        Some(fixtures.effective_working_dir()),
        "invalid agent_api.exec.add_dirs.v1.dirs[0]",
        &[&missing],
    );
}

#[test]
fn ad_c02_non_directory_path_uses_safe_indexed_message_without_leakage() {
    let fixtures = AddDirFixtures::new();
    let file_path = fixtures.create_file("not-a-dir.txt");
    let file_text = file_path.to_string_lossy().to_string();
    let payload = add_dirs_payload(vec![json!(file_text)]);

    assert_invalid_message(
        Some(&payload),
        Some(fixtures.effective_working_dir()),
        "invalid agent_api.exec.add_dirs.v1.dirs[0]",
        &[&file_path.to_string_lossy()],
    );
}

// Success-path normalization

#[test]
fn ad_c02_accepts_absolute_directory_entries_without_effective_working_dir() {
    let fixtures = AddDirFixtures::new();
    let absolute_dir = fixtures.create_root_dir("absolute");
    let absolute_text = absolute_dir.to_string_lossy().to_string();
    let payload = add_dirs_payload(vec![json!(absolute_text)]);

    let normalized = normalize_add_dirs_v1(Some(&payload), None).expect("normalize");

    assert_eq!(normalized, vec![absolute_dir]);
}

#[test]
fn ad_c02_relative_entries_without_effective_working_dir_use_safe_indexed_message() {
    let payload = add_dirs_payload(vec![json!("docs")]);

    assert_invalid_message(
        Some(&payload),
        None,
        "invalid agent_api.exec.add_dirs.v1.dirs[0]",
        &["docs"],
    );
}

#[test]
fn ad_c02_mixed_absolute_and_relative_entries_without_effective_working_dir_fail_on_relative_entry()
{
    let fixtures = AddDirFixtures::new();
    let absolute_dir = fixtures.create_root_dir("absolute");
    let absolute_text = absolute_dir.to_string_lossy().to_string();
    let relative_text = "docs";
    let payload = add_dirs_payload(vec![json!(absolute_text), json!(relative_text)]);

    assert_invalid_message(
        Some(&payload),
        None,
        "invalid agent_api.exec.add_dirs.v1.dirs[1]",
        &[&absolute_text, relative_text],
    );
}

#[test]
fn ad_c02_trims_unicode_whitespace_before_relative_resolution() {
    let fixtures = AddDirFixtures::new();
    let docs = fixtures.create_effective_dir("docs");
    let payload = add_dirs_payload(vec![json!("\u{2003}docs\u{2002}")]);

    let normalized = normalize_add_dirs_v1(Some(&payload), Some(fixtures.effective_working_dir()))
        .expect("normalize");

    assert_eq!(normalized, vec![docs]);
}

#[test]
fn ad_c02_resolves_relative_entries_from_effective_working_dir_only() {
    let fixtures = AddDirFixtures::new();
    let docs = fixtures.create_effective_dir("docs");
    let decoy_root = fixtures.create_root_dir("docs");
    let payload = add_dirs_payload(vec![json!("docs")]);

    let normalized = normalize_add_dirs_v1(Some(&payload), Some(fixtures.effective_working_dir()))
        .expect("normalize");

    assert_eq!(normalized, vec![docs]);
    assert_ne!(normalized, vec![decoy_root]);
}

#[cfg(windows)]
#[test]
fn ad_c02_resolves_drive_relative_entries_from_effective_working_dir_only() {
    let fixtures = AddDirFixtures::new();
    let docs = fixtures.create_effective_dir("docs");
    let drive_relative_docs = windows_drive_relative("docs", fixtures.effective_working_dir());
    let payload = add_dirs_payload(vec![json!(drive_relative_docs
        .to_string_lossy()
        .to_string())]);

    let normalized = normalize_add_dirs_v1(Some(&payload), Some(fixtures.effective_working_dir()))
        .expect("normalize");

    assert_eq!(normalized, vec![docs]);
}

#[cfg(windows)]
#[test]
fn ad_c02_rejects_mismatched_drive_relative_entries_before_filesystem_probe() {
    let fixtures = AddDirFixtures::new();
    fixtures.create_effective_dir("docs");
    let drive_relative_docs =
        windows_drive_relative_on_other_drive("docs", fixtures.effective_working_dir());
    let payload = add_dirs_payload(vec![json!(drive_relative_docs
        .to_string_lossy()
        .to_string())]);

    assert_invalid_message(
        Some(&payload),
        Some(fixtures.effective_working_dir()),
        "invalid agent_api.exec.add_dirs.v1.dirs[0]",
        &[&drive_relative_docs.to_string_lossy()],
    );
}

#[test]
fn ad_c02_lexically_normalizes_and_deduplicates_while_preserving_first_order() {
    let fixtures = AddDirFixtures::new();
    let first = fixtures.create_effective_dir("nested");
    let second = fixtures.create_effective_dir("other");
    let payload = add_dirs_payload(vec![
        json!("./nested/./"),
        json!("nested/../nested"),
        json!("./other/../other"),
    ]);

    let normalized = normalize_add_dirs_v1(Some(&payload), Some(fixtures.effective_working_dir()))
        .expect("normalize");

    assert_eq!(normalized, vec![first, second]);
}

#[cfg(windows)]
#[test]
fn ad_c02_deduplicates_windows_entries_case_insensitively() {
    let fixtures = AddDirFixtures::new();
    let docs = fixtures.create_effective_dir("docs");
    let payload = add_dirs_payload(vec![json!("docs"), json!("DOCS")]);

    let normalized = normalize_add_dirs_v1(Some(&payload), Some(fixtures.effective_working_dir()))
        .expect("normalize");

    assert_eq!(normalized, vec![docs]);
}

#[test]
fn ad_c02_allows_resolved_directories_outside_effective_working_dir() {
    let fixtures = AddDirFixtures::new();
    let outside = fixtures.create_root_dir("shared");
    let payload = add_dirs_payload(vec![json!("../shared")]);

    let normalized = normalize_add_dirs_v1(Some(&payload), Some(fixtures.effective_working_dir()))
        .expect("normalize");

    assert_eq!(normalized, vec![outside]);
}

fn add_dirs_payload(dirs: Vec<Value>) -> Value {
    json!({ "dirs": dirs })
}

#[cfg(windows)]
fn windows_drive_relative(relative: &str, absolute_path: &Path) -> PathBuf {
    let prefix = absolute_path
        .components()
        .find_map(|component| match component {
            Component::Prefix(value) => Some(value.as_os_str().to_string_lossy().into_owned()),
            _ => None,
        })
        .expect("absolute windows path should include a prefix");
    PathBuf::from(format!("{prefix}{relative}"))
}

#[cfg(windows)]
fn windows_drive_relative_on_other_drive(relative: &str, absolute_path: &Path) -> PathBuf {
    let current_drive = absolute_path
        .components()
        .find_map(|component| match component {
            Component::Prefix(value) => match value.kind() {
                Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) => {
                    Some(drive.to_ascii_lowercase())
                }
                _ => None,
            },
            _ => None,
        })
        .expect("absolute windows path should include a disk prefix");
    let alternate_drive = if current_drive == b'c' { 'd' } else { 'c' };
    PathBuf::from(format!("{alternate_drive}:{relative}"))
}

fn assert_invalid_message(
    raw: Option<&Value>,
    effective_working_dir: Option<&Path>,
    expected_message: &str,
    leaked_texts: &[&str],
) {
    let err = normalize_add_dirs_v1(raw, effective_working_dir).expect_err("expected invalid");
    match &err {
        AgentWrapperError::InvalidRequest { message } => {
            assert_eq!(message, expected_message);
        }
        other => panic!("expected InvalidRequest, got: {other:?}"),
    }

    for leaked_text in leaked_texts {
        assert!(
            !err.to_string().contains(leaked_text),
            "error display leaked sensitive text: {leaked_text}"
        );
    }
}

struct AddDirFixtures {
    root: TempDir,
    effective_working_dir: PathBuf,
}

impl AddDirFixtures {
    fn new() -> Self {
        let root = tempdir().expect("tempdir");
        let effective_working_dir = root.path().join("repo");
        fs::create_dir_all(&effective_working_dir).expect("create effective working dir");
        Self {
            root,
            effective_working_dir,
        }
    }

    fn effective_working_dir(&self) -> &Path {
        &self.effective_working_dir
    }

    fn create_effective_dir(&self, relative: &str) -> PathBuf {
        let path = self.effective_working_dir.join(relative);
        fs::create_dir_all(&path).expect("create effective dir");
        path
    }

    fn create_root_dir(&self, relative: &str) -> PathBuf {
        let path = self.root.path().join(relative);
        fs::create_dir_all(&path).expect("create root dir");
        path
    }

    fn create_file(&self, relative: &str) -> PathBuf {
        let path = self.root.path().join(relative);
        fs::write(&path, "content").expect("write file");
        path
    }

    fn missing_relative(&self, relative: &str) -> String {
        self.effective_working_dir
            .join(relative)
            .to_string_lossy()
            .to_string()
    }
}
