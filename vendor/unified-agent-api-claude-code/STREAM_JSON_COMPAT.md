# Claude stream-json Compatibility Notes (v1)

This document describes the compatibility posture for parsing Claude Code stream-json outputs.

Normative contract:

- `docs/specs/claude-stream-json-parser-contract.md`

## Compatibility posture

- The parser is tolerant and line-oriented: errors are per-line and do not stop ingestion.
- Unknown outer `type` strings are treated as `Unknown` events (not errors) to allow upstream drift.
- `Normalize` is narrowly scoped in v1: it is emitted only for `type=="result"` subtype/is_error
  cross-check inconsistencies when `is_error` is present.

## Field aliases

- `session_id` may also be provided as `sessionId`.

