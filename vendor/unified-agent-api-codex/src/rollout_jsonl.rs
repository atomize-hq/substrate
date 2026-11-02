use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

/// Errors that may occur while parsing Codex CLI rollout JSONL logs (`rollout-*.jsonl`).
#[derive(Debug, thiserror::Error)]
pub enum RolloutJsonlError {
    #[error("failed to parse codex rollout JSONL record: {source}: `{line}`")]
    Parse {
        line: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("codex rollout JSONL record missing required field: {message}: `{line}`")]
    Normalize { line: String, message: String },
    #[error("failed to read codex rollout JSONL: {source}")]
    Io {
        #[source]
        source: std::io::Error,
    },
}

#[derive(Clone, Debug, Default)]
pub struct RolloutJsonlParser;

impl RolloutJsonlParser {
    pub fn new() -> Self {
        Self
    }

    /// Parses a single logical JSONL line.
    ///
    /// - Returns `Ok(None)` for empty / whitespace-only lines.
    /// - Otherwise returns `Ok(Some(RolloutEvent))` on success.
    /// - Returns `Err(RolloutJsonlError)` on JSON parse / typed parse failures.
    pub fn parse_line(&mut self, line: &str) -> Result<Option<RolloutEvent>, RolloutJsonlError> {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.chars().all(|ch| ch.is_whitespace()) {
            return Ok(None);
        }

        let raw: RawRolloutLine =
            serde_json::from_str(line).map_err(|source| RolloutJsonlError::Parse {
                line: line.to_string(),
                source,
            })?;

        let record_type = raw
            .record_type
            .ok_or_else(|| RolloutJsonlError::Normalize {
                line: line.to_string(),
                message: "record missing `type`".to_string(),
            })?;

        let payload = raw.payload.unwrap_or(serde_json::Value::Null);
        let event = match record_type.as_str() {
            "session_meta" => RolloutEvent::SessionMeta(RolloutSessionMeta {
                timestamp: raw.timestamp,
                payload: serde_json::from_value(payload).map_err(|source| {
                    RolloutJsonlError::Parse {
                        line: line.to_string(),
                        source,
                    }
                })?,
                extra: raw.extra,
            }),
            "event_msg" => RolloutEvent::EventMsg(RolloutEventMsg {
                timestamp: raw.timestamp,
                payload: serde_json::from_value(payload).map_err(|source| {
                    RolloutJsonlError::Parse {
                        line: line.to_string(),
                        source,
                    }
                })?,
                extra: raw.extra,
            }),
            "response_item" => RolloutEvent::ResponseItem(RolloutResponseItem {
                timestamp: raw.timestamp,
                payload: serde_json::from_value(payload).map_err(|source| {
                    RolloutJsonlError::Parse {
                        line: line.to_string(),
                        source,
                    }
                })?,
                extra: raw.extra,
            }),
            _ => RolloutEvent::Unknown(RolloutUnknown {
                timestamp: raw.timestamp,
                record_type,
                payload,
                extra: raw.extra,
            }),
        };

        Ok(Some(event))
    }
}

#[derive(Clone, Debug, Deserialize)]
struct RawRolloutLine {
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(rename = "type")]
    record_type: Option<String>,
    #[serde(default)]
    payload: Option<serde_json::Value>,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RolloutEvent {
    SessionMeta(RolloutSessionMeta),
    EventMsg(RolloutEventMsg),
    ResponseItem(RolloutResponseItem),
    Unknown(RolloutUnknown),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RolloutSessionMeta {
    pub timestamp: Option<String>,
    pub payload: RolloutSessionMetaPayload,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RolloutEventMsg {
    pub timestamp: Option<String>,
    pub payload: RolloutEventMsgPayload,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RolloutResponseItem {
    pub timestamp: Option<String>,
    pub payload: RolloutResponseItemPayload,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RolloutUnknown {
    pub timestamp: Option<String>,
    #[serde(rename = "type")]
    pub record_type: String,
    pub payload: serde_json::Value,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RolloutSessionMetaPayload {
    pub id: Option<String>,
    pub timestamp: Option<String>,
    pub cwd: Option<String>,
    pub originator: Option<String>,
    pub cli_version: Option<String>,
    pub source: Option<String>,
    pub model_provider: Option<String>,
    pub base_instructions: Option<RolloutBaseInstructions>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RolloutBaseInstructions {
    pub text: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RolloutEventMsgPayload {
    #[serde(rename = "type")]
    pub kind: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RolloutResponseItemPayload {
    #[serde(rename = "type")]
    pub kind: Option<String>,

    pub role: Option<String>,
    pub content: Option<Vec<RolloutContentPart>>,
    pub summary: Option<Vec<RolloutContentPart>>,

    pub name: Option<String>,
    pub arguments: Option<String>,
    pub call_id: Option<String>,
    pub output: Option<String>,
    pub encrypted_content: Option<String>,

    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RolloutContentPart {
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub text: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug)]
pub struct RolloutJsonlRecord {
    /// 1-based line number in the underlying source (file/reader).
    pub line_number: usize,
    /// The parse outcome for this line (success or failure).
    pub outcome: Result<RolloutEvent, RolloutJsonlError>,
}

pub struct RolloutJsonlReader<R: BufRead> {
    reader: R,
    parser: RolloutJsonlParser,
    line_number: usize,
    buffer: String,
    done: bool,
}

impl<R: BufRead> RolloutJsonlReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            parser: RolloutJsonlParser::new(),
            line_number: 0,
            buffer: String::new(),
            done: false,
        }
    }
}

impl<R: BufRead> Iterator for RolloutJsonlReader<R> {
    type Item = RolloutJsonlRecord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        loop {
            self.buffer.clear();
            let line_number = self.line_number.saturating_add(1);
            match self.reader.read_line(&mut self.buffer) {
                Ok(0) => {
                    self.done = true;
                    return None;
                }
                Ok(_) => {
                    self.line_number = line_number;
                    if self.buffer.ends_with('\n') {
                        self.buffer.pop();
                    }

                    match self.parser.parse_line(&self.buffer) {
                        Ok(None) => continue,
                        Ok(Some(event)) => {
                            return Some(RolloutJsonlRecord {
                                line_number,
                                outcome: Ok(event),
                            });
                        }
                        Err(err) => {
                            return Some(RolloutJsonlRecord {
                                line_number,
                                outcome: Err(err),
                            });
                        }
                    }
                }
                Err(err) => {
                    self.done = true;
                    self.line_number = line_number;
                    return Some(RolloutJsonlRecord {
                        line_number,
                        outcome: Err(RolloutJsonlError::Io { source: err }),
                    });
                }
            }
        }
    }
}

pub type RolloutJsonlFileReader = RolloutJsonlReader<std::io::BufReader<std::fs::File>>;

pub fn rollout_jsonl_reader<R: BufRead>(reader: R) -> RolloutJsonlReader<R> {
    RolloutJsonlReader::new(reader)
}

pub fn rollout_jsonl_file(
    path: impl AsRef<Path>,
) -> Result<RolloutJsonlFileReader, RolloutJsonlError> {
    let file =
        std::fs::File::open(path.as_ref()).map_err(|source| RolloutJsonlError::Io { source })?;
    Ok(RolloutJsonlReader::new(std::io::BufReader::new(file)))
}

pub fn find_rollout_files(root: impl AsRef<Path>) -> Vec<PathBuf> {
    let root = root.as_ref();
    let mut out = Vec::new();
    let sessions = root.join("sessions");
    if !sessions.exists() {
        return out;
    }

    let mut stack = vec![sessions];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if name.starts_with("rollout-") && name.ends_with(".jsonl") {
                    out.push(path);
                }
            }
        }
    }

    out
}

pub fn find_rollout_file_by_id(root: impl AsRef<Path>, id: &str) -> Option<PathBuf> {
    let root = root.as_ref();
    let needle = id.strip_prefix("urn:uuid:").unwrap_or(id);
    let files = find_rollout_files(root);

    for path in &files {
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            if name.contains(needle) {
                return Some(path.clone());
            }
        }
    }

    for path in files {
        let file = std::fs::File::open(&path).ok()?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        for _ in 0..32 {
            line.clear();
            let n = reader.read_line(&mut line).ok()?;
            if n == 0 {
                break;
            }
            if line.ends_with('\n') {
                line.pop();
            }
            let logical = line.strip_suffix('\r').unwrap_or(&line);
            if logical.chars().all(|ch| ch.is_whitespace()) {
                continue;
            }

            let value: serde_json::Value = match serde_json::from_str(logical) {
                Ok(value) => value,
                Err(_) => continue,
            };
            if value.get("type").and_then(|v| v.as_str()) != Some("session_meta") {
                continue;
            }
            let Some(session_id) = value
                .get("payload")
                .and_then(|p| p.get("id"))
                .and_then(|v| v.as_str())
            else {
                continue;
            };
            let session_id = session_id.strip_prefix("urn:uuid:").unwrap_or(session_id);
            if session_id == needle {
                return Some(path);
            }
        }
    }

    None
}
