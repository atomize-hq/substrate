# Protocol — PolicySnapshotV2 for World FS Allow/Deny (Authoritative)

This document is authoritative for:
- `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

It specifies the protocol surfaces that carry `PolicySnapshotV2` to world-agent.

## Scope
- Linux full isolation only (`world_fs.isolation=full`) for deny enforcement.
- No backwards compatibility: world-agent rejects `PolicySnapshotV1`.

## 1) HTTP `/v1/execute` (JSON)
`ExecuteRequest.policy_snapshot` MUST use `PolicySnapshotV2`.

Fail-closed rules:
- World-agent MUST validate `policy_snapshot` against `docs/project_management/next/world-fs-granular-allow-deny/SCHEMA.md`.
- On any schema violation (wrong `schema_version`, unknown fields, invalid key combinations, or invalid patterns), world-agent MUST reject the request as invalid (HTTP 400).

### 1.1 Rejection response shape (grounded in `crates/world-agent/src/handlers.rs`)
For HTTP 400 rejections, the response body MUST be JSON with the following shape:
```json
{ "error": "<human-readable diagnostic>" }
```

## 2) WebSocket `/v1/stream` `start_session`
The REPL persistent-session protocol includes `start_session.policy_snapshot`.

Rules:
- `start_session.policy_snapshot` MUST be `PolicySnapshotV2` (schema_version 2).
- World-agent MUST validate `start_session.policy_snapshot` against `docs/project_management/next/world-fs-granular-allow-deny/SCHEMA.md`.
- On any schema violation, world-agent MUST reject the session start as invalid.

### 2.1 Rejection signaling (grounded in `crates/world-agent/src/pty.rs`)
For invalid `start_session` frames (including invalid JSON, unsupported `protocol_version`, or invalid `policy_snapshot`):
- World-agent MUST send a single fatal error frame with the following JSON shape:
  ```json
  {
    "type": "error",
    "code": "bad_request|unsupported_protocol_version",
    "message": "<human-readable diagnostic>",
    "fatal": true
  }
  ```
- `seq` MUST be omitted for `start_session` failures.
- After sending the fatal error frame, world-agent MUST close the WebSocket connection (no session is established).

## 3) Snapshot hashing and drift
Host-side drift detection hashes the entire snapshot JSON payload.

Migration note:
- Switching from `PolicySnapshotV1` to `PolicySnapshotV2` changes the snapshot hash by construction.
- Existing drift logic in `crates/shell/src/repl/async_repl.rs` remains authoritative once it hashes V2 snapshots.

## 4) No compat policy (version lockstep)
Because compat is explicitly forbidden by ADR-0018:
- A V2-emitting shell talking to a V1-only world-agent will fail.
- A V2-only world-agent receiving V1 snapshots will fail.
- Operator rollout MUST treat shell + world-agent as a lockstep update.
