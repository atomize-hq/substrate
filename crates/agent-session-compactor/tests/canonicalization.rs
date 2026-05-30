use anyhow as _;

use agent_session_compactor::canonicalize::{canonicalize_row_text, canonicalize_text};
use blake3 as _;
use camino as _;
use clap as _;
use codex as _;
use serde as _;
use serde_json as _;
use tempfile as _;
use thiserror as _;
use time as _;
use walkdir as _;

#[test]
fn canonicalization_strips_ansi_and_normalizes_whitespace_deterministically() {
    let raw = "\u{1b}[31mHello\u{1b}[0m  \r\nWorld\t \r\n";
    let canonical = canonicalize_text(raw);

    assert_eq!(canonical, "Hello\nWorld");
}

#[test]
fn canonicalization_hash_is_stable_for_equivalent_renderings() {
    let (left_text, left_hash) = canonicalize_row_text("Hello\r\nWorld  ");
    let (right_text, right_hash) = canonicalize_row_text("\u{1b}[32mHello\u{1b}[0m\nWorld");

    assert_eq!(left_text, "Hello\nWorld");
    assert_eq!(left_text, right_text);
    assert_eq!(left_hash, right_hash);
    assert_eq!(left_hash.len(), 64);
}
