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
- If `policy_snapshot.schema_version != 2`, world-agent MUST reject the request as invalid (HTTP 400).
- If `world_fs.isolation=workspace` includes deny/strict fields, world-agent MUST reject the request as invalid (HTTP 400).

## 2) WebSocket `/v1/stream` `start_session`
The REPL persistent-session protocol includes `start_session.policy_snapshot`.

Rules:
- `start_session.policy_snapshot` MUST be `PolicySnapshotV2` (schema_version 2).
- World-agent MUST reject the session start if schema validation fails.

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

