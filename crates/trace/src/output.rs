use anyhow::Result;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::span::Span;

pub struct TraceOutput {
    pub(crate) writer: BufWriter<File>,
    pub(crate) path: PathBuf,
}

impl TraceOutput {
    pub(crate) fn new(path: impl AsRef<Path>) -> Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(&path)?;

        Ok(TraceOutput {
            writer: BufWriter::new(file),
            path: path.as_ref().to_path_buf(),
        })
    }

    pub(crate) fn max_bytes() -> u64 {
        const DEFAULT_MB: u64 = 100; // ~100MB
                                     // Accept both TRACE_LOG_MAX_MB (preferred) and legacy SHIM_TRACE_LOG_MAX_MB for tests/back-compat
        let mb = env::var("TRACE_LOG_MAX_MB")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .or_else(|| {
                env::var("SHIM_TRACE_LOG_MAX_MB")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
            })
            .unwrap_or(DEFAULT_MB);
        mb * 1024 * 1024
    }

    pub(crate) fn keep_files() -> usize {
        // Keep last N rotated files
        env::var("TRACE_LOG_KEEP")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(3)
    }

    pub(crate) fn rotate_if_needed(&mut self) -> Result<()> {
        // Flush current writer to ensure size is accurate
        self.writer.flush()?;
        let path = self.path.clone();

        if let Ok(meta) = fs::metadata(&path) {
            if meta.len() >= Self::max_bytes() {
                // Shift older rotations first while honoring retention
                // Remove the oldest file (".keep") if present, then shift (keep-1)->keep, ..., 1->2
                let keep = Self::keep_files();
                if keep > 0 {
                    let oldest = path.with_extension(format!("jsonl.{}", keep));
                    let _ = fs::remove_file(&oldest);
                    for i in (2..=keep).rev() {
                        let from = path.with_extension(format!("jsonl.{}", i - 1));
                        let to = path.with_extension(format!("jsonl.{}", i));
                        let _ = fs::rename(&from, &to);
                    }
                }
                // Rename current to .1 (writer still holds the fd; rename is allowed on Unix)
                let bak = path.with_extension("jsonl.1");
                let _ = fs::rename(&path, &bak);

                // Recreate fresh file and swap writer to new handle
                let file = OpenOptions::new().create(true).append(true).open(&path)?;
                self.writer = BufWriter::new(file);
            }
        }
        Ok(())
    }

    pub(crate) fn write_span(&mut self, span: &Span) -> Result<()> {
        self.rotate_if_needed()?;

        // Serialize span and ensure both `cmd` and legacy `command` keys exist for compat.
        let mut value = serde_json::to_value(span)?;
        if let Some(obj) = value.as_object_mut() {
            if !obj.contains_key("command") {
                if let Some(cmd_value) = obj.get("cmd").cloned() {
                    obj.insert("command".to_string(), cmd_value);
                }
            }
        }

        let json = serde_json::to_string(&value)?;
        writeln!(self.writer, "{}", json)?;

        if env::var("SHIM_FSYNC").unwrap_or_default() == "1" {
            self.writer.flush()?;
            self.writer.get_ref().sync_all()?;
        } else {
            self.writer.flush()?;
        }

        Ok(())
    }
}
