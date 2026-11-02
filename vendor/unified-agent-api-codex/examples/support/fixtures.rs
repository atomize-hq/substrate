//! Shared sample payloads for examples and docs.
//!
//! The `--sample` paths in streaming/resume/apply examples pull from these fixtures so updates to
//! the Codex CLI surface only need to be applied in one place.
#![allow(dead_code)]

pub const STREAMING_FIXTURE_PATH: &str = "crates/codex/examples/fixtures/streaming.jsonl";
pub const RESUME_FIXTURE_PATH: &str = "crates/codex/examples/fixtures/resume.jsonl";
pub const DIFF_FIXTURE_PATH: &str = "crates/codex/examples/fixtures/diff.patch";
pub const APPLY_FIXTURE_PATH: &str = "crates/codex/examples/fixtures/apply_result.json";

const STREAMING_EVENTS: &str = include_str!("../fixtures/streaming.jsonl");
const RESUME_EVENTS: &str = include_str!("../fixtures/resume.jsonl");
const SAMPLE_DIFF: &str = include_str!("../fixtures/diff.patch");
const SAMPLE_APPLY_RESULT: &str = include_str!("../fixtures/apply_result.json");

pub fn streaming_events() -> impl Iterator<Item = &'static str> {
    STREAMING_EVENTS
        .lines()
        .filter(|line| !line.trim().is_empty())
}

pub fn resume_events() -> impl Iterator<Item = &'static str> {
    RESUME_EVENTS.lines().filter(|line| !line.trim().is_empty())
}

pub fn sample_diff() -> &'static str {
    SAMPLE_DIFF
}

pub fn apply_result() -> &'static str {
    SAMPLE_APPLY_RESULT
}
