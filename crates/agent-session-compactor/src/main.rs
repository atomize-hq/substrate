use anyhow as _;
use blake3 as _;
use camino as _;
use clap as _;
use codex as _;
use serde as _;
use serde_json as _;
#[cfg(test)]
use tempfile as _;
use thiserror as _;
use time as _;
use walkdir as _;

fn main() -> anyhow::Result<()> {
    agent_session_compactor::run()
}
