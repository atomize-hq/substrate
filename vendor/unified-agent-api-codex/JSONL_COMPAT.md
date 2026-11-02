# JSONL Compatibility (Codex CLI parity)

Status: **Normative**  
Scope: normalization semantics for `ThreadEvent` JSONL parsing (streaming + offline)

This crate consumes Codex CLI `--json` output as a JSONL stream (one JSON object per stdout line).
The streaming APIs `CodexClient::stream_exec` and `CodexClient::stream_resume` yield a stream of
`Result<ThreadEvent, ExecStreamError>`.

This document is the single source of truth for **normalization behavior**. It applies to:

- Live streaming parsing (`stream_exec`, `stream_resume`)
- Offline JSONL log parsing APIs (ADR 0005)

## Normative language

This document uses RFC 2119-style requirement keywords (`MUST`, `MUST NOT`).

## Goals
- Prefer typed `ThreadEvent` parsing for ergonomics.
- Tolerate upstream drift (field renames, nesting changes, missing context) via normalization and
  unknown-field capture.
- Do not terminate the entire stream on the first malformed/unrecognized line when it is possible
  to continue.

## Line handling (normative)

- Parsers MUST ignore empty / whitespace-only lines (no events emitted for them).
- Parsers MUST tolerate Windows CRLF JSONL logs by trimming a single trailing `\r` from each line
  prior to JSON parsing (i.e., parse the logical line content, not the line ending).
- Parsers MUST NOT apply a full `.trim()` before JSON parsing. JSON parsing already tolerates
  leading/trailing whitespace, and preserving the original bytes improves debugging/audit fidelity.

## Normalization (what + when)

Normalization is applied to every non-empty JSONL line before attempting to deserialize it into
`ThreadEvent`. It is heuristic-driven (based on which fields are present/missing), not gated on an
explicit Codex CLI version.

### Event type aliases
The parser accepts multiple upstream names for the same typed variants:
- `thread.started` and `thread.resumed` → `ThreadEvent::ThreadStarted`
- `item.started` and `item.created` → `ThreadEvent::ItemStarted`
- `item.delta` and `item.updated` → `ThreadEvent::ItemDelta`

### Context inference (thread/turn ids)
The stream maintains the most recently observed `thread_id` and `turn_id`:
- If a `turn.*` or `item.*` event is missing `thread_id` and a prior `thread.started`/`thread.resumed`
  established context, the missing `thread_id` is filled from context.
- If an `item.*` event is missing `turn_id` and a prior `turn.started` established context, the
  missing `turn_id` is filled from context.
- If `turn.started` is missing `turn_id`, a synthetic id `synthetic-turn-N` is generated.

Additional context rules (normative):
- When a `thread.started` or `thread.resumed` event is observed, parsers MUST:
  - set the current thread context to that `thread_id`, and
  - clear any existing current turn context (a new `turn.started` establishes the next one).
- Synthetic `turn_id` generation MUST use a monotonic counter scoped to the parser instance and MUST
  NOT reset on `thread.started` / `thread.resumed`. The counter resets only when the parser itself
  is reset (e.g., a fresh parser instance).

### Item envelope normalization
For `item.*` events, older Codex CLI versions may nest item fields under `{"item": {...}}`. The
parser normalizes by:
- Moving fields from `item` to the top level.
- Renaming `item.type` → `item_type`.
- For text-shaped items (`agent_message`, `reasoning`), wrapping legacy `content: "<string>"`
  payloads into the typed `{"text": "<string>"}` form expected by `TextContent`.
- If `item_type` is command-shaped and `content` is missing, synthesizing `content` from legacy
  fields like `text`, `command`, `aggregated_output`, `exit_code`, and `stderr`.

For delta-shaped item events (`item.delta` / `item.updated`), if `delta` is absent but `content` is
present, `content` is treated as the delta payload (`content` → `delta`).
For text-shaped deltas, legacy `delta: "<string>"` payloads are wrapped into
`{"text_delta": "<string>"}` for `TextDelta`.

### Field aliases
Common legacy field names are accepted during deserialization:
- `item_id`: `item_id` or `id`
- text deltas: `text` or `text_delta`
- command/file output: `aggregated_output` / `output` → `stdout`; `error_output` / `err` → `stderr`
- file change: `file_path` → `path`; `patch` → `diff`
- MCP tool call: `server` → `server_name`; `tool` → `tool_name`

## Unknown field capture
Many event/payload structs include an `extra: BTreeMap<String, serde_json::Value>` field annotated
with `#[serde(flatten)]`. Any fields not understood by the typed schema are preserved there (rather
than dropped silently) so callers can inspect or forward them.

## Error surfacing and stream behavior
- Each non-empty JSONL line produces exactly one parse outcome:
  - Success: a `ThreadEvent` when the line normalizes and deserializes successfully.
  - Failure: an `ExecStreamError` when JSON parsing, normalization, or typed deserialization fails.
- Unknown or unrecognized `type` values MUST surface as per-line parse failures (e.g.
  `ExecStreamError::Parse`) and MUST NOT stop consumption of subsequent lines.
- Malformed/unrecognized lines do not stop the stream; subsequent lines are still read and emitted
  when possible.
- The stream can still terminate early for transport-level issues (e.g., stdout read failures or the
  Codex process exiting).
