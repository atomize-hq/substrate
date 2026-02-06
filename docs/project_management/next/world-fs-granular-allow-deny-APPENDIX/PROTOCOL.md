# Protocol — PolicySnapshotV3 for World FS Appendix (Authoritative)

This document is authoritative for Appendix A + B of:
- `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

It specifies the protocol surfaces that carry `PolicySnapshotV3` to world-agent.

## Scope
- No backwards compatibility: world-agent rejects prior snapshot schema versions.

## 1) HTTP `/v1/execute` (JSON)
`ExecuteRequest.policy_snapshot` MUST use `PolicySnapshotV3`.

Fail-closed rules:
- World-agent MUST validate `policy_snapshot` against `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md`.
- On any schema violation, world-agent MUST reject the request as invalid (HTTP 400).

Rejection response shape:
```json
{ "error": "<human-readable diagnostic>" }
```

## 2) WebSocket `/v1/stream` `start_session`
Rules:
- `start_session.policy_snapshot` MUST be `PolicySnapshotV3` (`schema_version=3`).
- World-agent MUST validate the snapshot.
- On schema violation, world-agent MUST reject session start as invalid.

Fatal error frame shape:
```json
{
  "type": "error",
  "code": "bad_request|unsupported_protocol_version",
  "message": "<human-readable diagnostic>",
  "fatal": true
}
```

## 3) No compat policy (version lockstep)
- A V3-emitting shell talking to a V2-only world-agent fails.
- A V3-only world-agent receiving V2 snapshots fails.

