# WFGADAXA1-spec — PolicySnapshotV3 protocol + validation (no compat)

## Scope
- Migrate the shell↔world-agent policy snapshot protocol from V2 to **V3** as defined by the Appendix authoritative docs.
- Enforce “no backwards compatibility” for snapshot schema versions.

## Authoritative requirements
- Snapshot schema: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md` §2
  - `schema_version` MUST be `3`
  - unknown fields MUST be rejected by world-agent (HTTP 400 / fatal WS error frame)
- Protocol surfaces: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/PROTOCOL.md`
  - `ExecuteRequest.policy_snapshot` MUST be `PolicySnapshotV3`
  - `start_session.policy_snapshot` MUST be `PolicySnapshotV3`
  - No compat lockstep (V3 shell ↔ V3 world-agent only)
- No backwards compatibility: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/contract.md` §5

## Behavior
Required end-state:
1. `crates/agent-api-types` defines `PolicySnapshotV3` (and world-fs nested structs) with:
   - strict `serde(deny_unknown_fields)` where appropriate
   - `validate()` rejects `schema_version != 3`
2. `crates/shell` emits `PolicySnapshotV3` for world-agent requests and computes drift hash per Appendix schema.
3. `crates/world-agent`:
   - requires `policy_snapshot` for `/v1/execute` and WS `start_session`
   - validates it as `PolicySnapshotV3`
   - rejects:
     - missing `policy_snapshot`
     - `schema_version != 3` (including `2`)
     - unknown fields

## Acceptance criteria
- Tests:
  - World-agent tests cover:
    - reject schema_version 2
    - reject schema_version 1
    - reject unknown fields
    - reject missing policy_snapshot (HTTP + WS)
- Commands (integration gate):
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p agent-api-types -p world-agent -p substrate-shell --tests -- --nocapture`
  - `make integ-checks`

## Out of scope
- Expanding `PolicySnapshotV3` beyond what Appendix schema defines (see decision DR-AXA-0003).
- Changing isolation enforcement semantics beyond what is required for strict V3 protocol validation.

