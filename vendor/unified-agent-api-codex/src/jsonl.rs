use std::{
    future::Future,
    io::{self as stdio, BufRead, Write},
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::Stream;
use tokio::{
    fs,
    fs::OpenOptions,
    io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader, BufWriter},
    sync::mpsc,
    task, time,
};

use crate::{CodexError, ExecStreamError, ThreadEvent};

#[derive(Clone, Debug, Default)]
pub(crate) struct StreamContext {
    current_thread_id: Option<String>,
    current_turn_id: Option<String>,
    next_synthetic_turn: u32,
}

/// Parses Codex `--json` JSONL logs into typed [`ThreadEvent`] values.
///
/// This API is synchronous and line-oriented (v1 contract).
#[derive(Clone, Debug, Default)]
pub struct JsonlThreadEventParser {
    context: StreamContext,
}

impl JsonlThreadEventParser {
    /// Constructs a new parser with no established context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears any thread/turn context and resets synthetic turn counters.
    pub fn reset(&mut self) {
        self.context = StreamContext::default();
    }

    /// Parses a single logical JSONL line.
    ///
    /// - Returns `Ok(None)` for empty / whitespace-only lines.
    /// - Otherwise returns `Ok(Some(ThreadEvent))` on success.
    /// - Returns `Err(ExecStreamError)` on JSON parse / normalization / typed parse failures.
    pub fn parse_line(&mut self, line: &str) -> Result<Option<ThreadEvent>, ExecStreamError> {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.chars().all(|ch| ch.is_whitespace()) {
            return Ok(None);
        }

        normalize_thread_event(line, &mut self.context).map(Some)
    }
}

#[derive(Debug)]
pub struct ThreadEventJsonlRecord {
    /// 1-based line number in the underlying source (file/reader).
    pub line_number: usize,
    /// The parse outcome for this line (success or failure).
    pub outcome: Result<ThreadEvent, ExecStreamError>,
}

impl Clone for ThreadEventJsonlRecord {
    fn clone(&self) -> Self {
        Self {
            line_number: self.line_number,
            outcome: match &self.outcome {
                Ok(event) => Ok(event.clone()),
                Err(err) => Err(clone_exec_stream_error(err)),
            },
        }
    }
}

fn clone_exec_stream_error(err: &ExecStreamError) -> ExecStreamError {
    match err {
        ExecStreamError::Codex(source) => ExecStreamError::Codex(clone_codex_error(source)),
        ExecStreamError::Parse { line, source } => ExecStreamError::Parse {
            line: line.clone(),
            source: <serde_json::Error as serde::de::Error>::custom(source.to_string()),
        },
        ExecStreamError::Normalize { line, message } => ExecStreamError::Normalize {
            line: line.clone(),
            message: message.clone(),
        },
        ExecStreamError::IdleTimeout { idle_for } => ExecStreamError::IdleTimeout {
            idle_for: *idle_for,
        },
        ExecStreamError::ChannelClosed => ExecStreamError::ChannelClosed,
    }
}

fn clone_codex_error(err: &CodexError) -> CodexError {
    match err {
        CodexError::Spawn { binary, source } => CodexError::Spawn {
            binary: binary.clone(),
            source: clone_io_error(source),
        },
        CodexError::Wait { source } => CodexError::Wait {
            source: clone_io_error(source),
        },
        CodexError::Timeout { timeout } => CodexError::Timeout { timeout: *timeout },
        CodexError::NonZeroExit { status, stderr } => CodexError::NonZeroExit {
            status: *status,
            stderr: stderr.clone(),
        },
        CodexError::InvalidUtf8(source) => {
            let io_err = std::io::Error::new(std::io::ErrorKind::InvalidData, source.to_string());
            CodexError::CaptureIo(io_err)
        }
        CodexError::JsonParse {
            context,
            stdout,
            source,
        } => CodexError::JsonParse {
            context,
            stdout: stdout.clone(),
            source: <serde_json::Error as serde::de::Error>::custom(source.to_string()),
        },
        CodexError::ExecPolicyParse { stdout, source } => CodexError::ExecPolicyParse {
            stdout: stdout.clone(),
            source: <serde_json::Error as serde::de::Error>::custom(source.to_string()),
        },
        CodexError::FeatureListParse { reason, stdout } => CodexError::FeatureListParse {
            reason: reason.clone(),
            stdout: stdout.clone(),
        },
        CodexError::ResponsesApiProxyInfoRead { path, source } => {
            CodexError::ResponsesApiProxyInfoRead {
                path: path.clone(),
                source: clone_io_error(source),
            }
        }
        CodexError::ResponsesApiProxyInfoParse { path, source } => {
            CodexError::ResponsesApiProxyInfoParse {
                path: path.clone(),
                source: <serde_json::Error as serde::de::Error>::custom(source.to_string()),
            }
        }
        CodexError::EmptyPrompt => CodexError::EmptyPrompt,
        CodexError::EmptySandboxCommand => CodexError::EmptySandboxCommand,
        CodexError::EmptyExecPolicyCommand => CodexError::EmptyExecPolicyCommand,
        CodexError::EmptyApiKey => CodexError::EmptyApiKey,
        CodexError::EmptyTaskId => CodexError::EmptyTaskId,
        CodexError::EmptyEnvId => CodexError::EmptyEnvId,
        CodexError::EmptyMcpServerName => CodexError::EmptyMcpServerName,
        CodexError::EmptyMcpCommand => CodexError::EmptyMcpCommand,
        CodexError::EmptyMcpUrl => CodexError::EmptyMcpUrl,
        CodexError::EmptySocketPath => CodexError::EmptySocketPath,
        CodexError::TempDir(source) => CodexError::TempDir(clone_io_error(source)),
        CodexError::WorkingDirectory { source } => CodexError::WorkingDirectory {
            source: clone_io_error(source),
        },
        CodexError::PrepareOutputDirectory { path, source } => CodexError::PrepareOutputDirectory {
            path: path.clone(),
            source: clone_io_error(source),
        },
        CodexError::PrepareCodexHome { path, source } => CodexError::PrepareCodexHome {
            path: path.clone(),
            source: clone_io_error(source),
        },
        CodexError::StdoutUnavailable => CodexError::StdoutUnavailable,
        CodexError::StderrUnavailable => CodexError::StderrUnavailable,
        CodexError::StdinUnavailable => CodexError::StdinUnavailable,
        CodexError::CaptureIo(source) => CodexError::CaptureIo(clone_io_error(source)),
        CodexError::StdinWrite(source) => CodexError::StdinWrite(clone_io_error(source)),
        CodexError::Join(source) => {
            let io_err = std::io::Error::other(source.to_string());
            CodexError::CaptureIo(io_err)
        }
    }
}

fn clone_io_error(err: &std::io::Error) -> std::io::Error {
    std::io::Error::new(err.kind(), err.to_string())
}

pub struct ThreadEventJsonlReader<R: BufRead> {
    reader: R,
    parser: JsonlThreadEventParser,
    line_number: usize,
    buffer: String,
    done: bool,
}

impl<R: BufRead> ThreadEventJsonlReader<R> {
    /// Creates a reader-backed iterator with a fresh parser.
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            parser: JsonlThreadEventParser::new(),
            line_number: 0,
            buffer: String::new(),
            done: false,
        }
    }

    /// Consumes the iterator and returns the wrapped reader.
    pub fn into_inner(self) -> R {
        self.reader
    }
}

impl<R: BufRead> Iterator for ThreadEventJsonlReader<R> {
    type Item = ThreadEventJsonlRecord;

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
                            return Some(ThreadEventJsonlRecord {
                                line_number,
                                outcome: Ok(event),
                            });
                        }
                        Err(err) => {
                            return Some(ThreadEventJsonlRecord {
                                line_number,
                                outcome: Err(err),
                            });
                        }
                    }
                }
                Err(err) => {
                    self.done = true;
                    self.line_number = line_number;
                    return Some(ThreadEventJsonlRecord {
                        line_number,
                        outcome: Err(ExecStreamError::from(CodexError::CaptureIo(err))),
                    });
                }
            }
        }
    }
}

pub type ThreadEventJsonlFileReader = ThreadEventJsonlReader<std::io::BufReader<std::fs::File>>;

/// Convenience constructor for reader-backed parsing.
pub fn thread_event_jsonl_reader<R: BufRead>(reader: R) -> ThreadEventJsonlReader<R> {
    ThreadEventJsonlReader::new(reader)
}

/// Convenience constructor for file-backed parsing.
pub fn thread_event_jsonl_file(
    path: impl AsRef<Path>,
) -> Result<ThreadEventJsonlFileReader, ExecStreamError> {
    let file = std::fs::File::open(path.as_ref())
        .map_err(|err| ExecStreamError::from(CodexError::CaptureIo(err)))?;
    Ok(ThreadEventJsonlReader::new(std::io::BufReader::new(file)))
}

pub(crate) async fn prepare_json_log(
    path: Option<PathBuf>,
) -> Result<Option<JsonLogSink>, ExecStreamError> {
    match path {
        Some(path) => {
            let sink = JsonLogSink::new(path)
                .await
                .map_err(|err| ExecStreamError::from(CodexError::CaptureIo(err)))?;
            Ok(Some(sink))
        }
        None => Ok(None),
    }
}

#[derive(Debug)]
pub(crate) struct JsonLogSink {
    writer: BufWriter<fs::File>,
}

impl JsonLogSink {
    pub(crate) async fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).await?;
            }
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await?;

        Ok(Self {
            writer: BufWriter::new(file),
        })
    }

    async fn write_line(&mut self, line: &str) -> Result<(), std::io::Error> {
        self.writer.write_all(line.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await
    }
}

pub(crate) struct EventChannelStream {
    rx: mpsc::Receiver<Result<ThreadEvent, ExecStreamError>>,
    idle_timeout: Option<std::time::Duration>,
    idle_timer: Option<Pin<Box<time::Sleep>>>,
}

impl EventChannelStream {
    pub(crate) fn new(
        rx: mpsc::Receiver<Result<ThreadEvent, ExecStreamError>>,
        idle_timeout: Option<std::time::Duration>,
    ) -> Self {
        Self {
            rx,
            idle_timeout,
            idle_timer: None,
        }
    }

    fn reset_timer(&mut self) {
        self.idle_timer = self
            .idle_timeout
            .map(|duration| Box::pin(time::sleep(duration)));
    }
}

impl Stream for EventChannelStream {
    type Item = Result<ThreadEvent, ExecStreamError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if let Some(timer) = this.idle_timer.as_mut() {
            if let Poll::Ready(()) = timer.as_mut().poll(cx) {
                let idle_for = this.idle_timeout.expect("idle_timer implies timeout");
                this.idle_timer = None;
                return Poll::Ready(Some(Err(ExecStreamError::IdleTimeout { idle_for })));
            }
        }

        match this.rx.poll_recv(cx) {
            Poll::Ready(Some(item)) => {
                if this.idle_timeout.is_some() {
                    this.reset_timer();
                }
                Poll::Ready(Some(item))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => {
                if this.idle_timer.is_none() {
                    if let Some(duration) = this.idle_timeout {
                        let mut sleep = Box::pin(time::sleep(duration));
                        if let Poll::Ready(()) = sleep.as_mut().poll(cx) {
                            return Poll::Ready(Some(Err(ExecStreamError::IdleTimeout {
                                idle_for: duration,
                            })));
                        }
                        this.idle_timer = Some(sleep);
                    }
                }
                Poll::Pending
            }
        }
    }
}

pub(crate) async fn forward_json_events<R>(
    reader: R,
    sender: mpsc::Sender<Result<ThreadEvent, ExecStreamError>>,
    mirror_stdout: bool,
    mut log: Option<JsonLogSink>,
) -> Result<(), ExecStreamError>
where
    R: AsyncRead + Unpin,
{
    let mut lines = BufReader::new(reader).lines();
    let mut context = StreamContext::default();
    loop {
        let line = match lines.next_line().await {
            Ok(Some(line)) => line,
            Ok(None) => break,
            Err(err) => {
                return Err(CodexError::CaptureIo(err).into());
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        if let Some(sink) = log.as_mut() {
            sink.write_line(&line)
                .await
                .map_err(|err| ExecStreamError::from(CodexError::CaptureIo(err)))?;
        }

        if mirror_stdout {
            if let Err(err) = task::block_in_place(|| {
                let mut out = stdio::stdout();
                out.write_all(line.as_bytes())?;
                out.write_all(b"\n")?;
                out.flush()
            }) {
                return Err(CodexError::CaptureIo(err).into());
            }
        }

        let event = normalize_thread_event(&line, &mut context);
        if sender.send(event).await.is_err() {
            break;
        }
    }

    Ok(())
}

pub(crate) fn normalize_thread_event(
    line: &str,
    context: &mut StreamContext,
) -> Result<ThreadEvent, ExecStreamError> {
    let mut value: serde_json::Value =
        serde_json::from_str(line).map_err(|source| ExecStreamError::Parse {
            line: line.to_string(),
            source,
        })?;

    let event_type = value
        .get("type")
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| ExecStreamError::Normalize {
            line: line.to_string(),
            message: "event missing `type`".to_string(),
        })?;

    match event_type.as_str() {
        "thread.started" | "thread.resumed" => {
            let thread_id = extract_str_from_keys(&value, &["thread_id", "conversation_id", "id"])
                .ok_or_else(|| missing(&event_type, "thread_id", line))?;
            context.current_thread_id = Some(thread_id.to_string());
            context.current_turn_id = None;
        }
        "turn.started" => {
            let turn_id = extract_str_from_keys(&value, &["turn_id", "id"])
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    let next = context.next_synthetic_turn.max(1);
                    let id = format!("synthetic-turn-{next}");
                    context.next_synthetic_turn = next.saturating_add(1);
                    id
                });
            let thread_id = extract_str_from_keys(&value, &["thread_id", "conversation_id"])
                .map(|s| s.to_string())
                .or_else(|| context.current_thread_id.clone())
                .ok_or_else(|| missing("turn.started", "thread_id", line))?;
            set_str(&mut value, "turn_id", turn_id.clone());
            set_str(&mut value, "thread_id", thread_id.clone());
            context.current_thread_id = Some(thread_id);
            context.current_turn_id = Some(turn_id);
        }
        "turn.completed" | "turn.failed" => {
            let turn_id = extract_str_from_keys(&value, &["turn_id", "id"])
                .map(|s| s.to_string())
                .or_else(|| context.current_turn_id.clone())
                .ok_or_else(|| missing(&event_type, "turn_id", line))?;
            let thread_id = extract_str_from_keys(&value, &["thread_id", "conversation_id"])
                .map(|s| s.to_string())
                .or_else(|| context.current_thread_id.clone())
                .ok_or_else(|| missing(&event_type, "thread_id", line))?;
            set_str(&mut value, "turn_id", turn_id.clone());
            set_str(&mut value, "thread_id", thread_id.clone());
            context.current_turn_id = None;
            context.current_thread_id = Some(thread_id);
        }
        t if t.starts_with("item.") => {
            normalize_item_payload(&mut value);
            if event_type == "item.delta" || event_type == "item.updated" {
                normalize_item_delta_payload(&mut value);
            }
            let turn_id = extract_str(&value, "turn_id")
                .map(|s| s.to_string())
                .or_else(|| context.current_turn_id.clone())
                .ok_or_else(|| missing(&event_type, "turn_id", line))?;
            let thread_id = extract_str_from_keys(&value, &["thread_id", "conversation_id"])
                .map(|s| s.to_string())
                .or_else(|| context.current_thread_id.clone())
                .ok_or_else(|| missing(&event_type, "thread_id", line))?;
            set_str(&mut value, "turn_id", turn_id);
            set_str(&mut value, "thread_id", thread_id);
        }
        _ => {}
    }

    serde_json::from_value::<ThreadEvent>(value).map_err(|source| ExecStreamError::Parse {
        line: line.to_string(),
        source,
    })
}

fn extract_str<'a>(value: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
}

fn extract_str_from_keys<'a>(value: &'a serde_json::Value, keys: &[&str]) -> Option<&'a str> {
    for key in keys {
        if let Some(found) = extract_str(value, key) {
            return Some(found);
        }
    }
    None
}

fn set_str(value: &mut serde_json::Value, key: &str, new_value: String) {
    if let Some(map) = value.as_object_mut() {
        map.insert(key.to_string(), serde_json::Value::String(new_value));
    }
}

fn normalize_item_delta_payload(value: &mut serde_json::Value) {
    let Some(map) = value.as_object_mut() else {
        return;
    };

    if !map.contains_key("delta") {
        if let Some(content) = map.remove("content") {
            map.insert("delta".to_string(), content);
        }
    }

    let Some(item_type) = map.get("item_type").and_then(|value| value.as_str()) else {
        return;
    };

    if !matches!(item_type, "agent_message" | "reasoning") {
        return;
    }

    let Some(delta) = map.get_mut("delta") else {
        return;
    };

    if let Some(text_delta) = delta.as_str() {
        *delta = serde_json::json!({ "text_delta": text_delta });
    }
}

fn normalize_item_payload(value: &mut serde_json::Value) {
    let mut item_object = match value
        .get_mut("item")
        .and_then(|item| item.as_object_mut())
        .map(|map| map.clone())
    {
        Some(map) => map,
        None => return,
    };

    if !item_object.contains_key("item_type") {
        if let Some(item_type) = item_object.remove("type") {
            item_object.insert("item_type".to_string(), item_type);
        }
    }

    if !item_object.contains_key("content") {
        let mut content: Option<serde_json::Value> = None;
        if let Some(text) = item_object.remove("text") {
            if let Some(text_str) = text.as_str() {
                content = Some(serde_json::json!({ "text": text_str }));
            } else {
                content = Some(text);
            }
        } else if let Some(command) = item_object.get("command").cloned() {
            let mut map = serde_json::Map::new();
            map.insert("command".to_string(), command);
            if let Some(stdout) = item_object.remove("aggregated_output") {
                map.insert("stdout".to_string(), stdout);
            }
            if let Some(exit_code) = item_object.remove("exit_code") {
                map.insert("exit_code".to_string(), exit_code);
            }
            if let Some(stderr) = item_object.remove("stderr") {
                map.insert("stderr".to_string(), stderr);
            }
            content = Some(serde_json::Value::Object(map));
        }

        if let Some(content_value) = content {
            item_object.insert("content".to_string(), content_value);
        }
    }

    let item_type = item_object
        .get("item_type")
        .and_then(|value| value.as_str())
        .or_else(|| item_object.get("type").and_then(|value| value.as_str()))
        .map(|value| value.to_string());

    if matches!(item_type.as_deref(), Some("agent_message" | "reasoning")) {
        if let Some(content) = item_object.get_mut("content") {
            if let Some(text) = content.as_str() {
                *content = serde_json::json!({ "text": text });
            }
        }
    }

    if let Some(root) = value.as_object_mut() {
        for (mut key, mut v) in item_object {
            if key == "type" {
                key = "item_type".to_string();
            }
            root.insert(key, v.take());
        }
        root.remove("item");
    }
}

fn missing(event: &str, field: &str, line: &str) -> ExecStreamError {
    ExecStreamError::Normalize {
        line: line.to_string(),
        message: format!("{event} missing `{field}` and no prior context to infer it"),
    }
}
