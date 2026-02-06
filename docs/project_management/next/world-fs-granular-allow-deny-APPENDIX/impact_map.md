# world-fs-granular-allow-deny-appendix — impact map

Authoring standard:
- `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`

## Goal (single sentence)
- Implement ADR-0018 Appendix A + B as a breaking policy surface update with deterministic validation, routing fail-closed semantics, REPL caging requirements, and REPL exit behavior.

## Authoritative touch set (files and components)

### Policy schema + validation
- `crates/broker` (policy patch parsing + validation; hard errors; defaults; effective config export)
- `crates/agent-api-*` (shared models for patch/snapshot, when used)

### Policy snapshot emission + protocol transport
- `crates/shell` (policy snapshot construction, routing behavior, REPL behavior, exported env var emission)
- `crates/world-agent` (snapshot consumption and schema validation)

### Env var contract surfaces
- `crates/shell` (exported state env vars for telemetry and subprocesses)
- `crates/shim` (propagation of exported state env vars to shimmed subprocesses)
- `crates/world` + `crates/world-agent` (execution wrapper env vars remain authoritative for isolation helpers)

### REPL caging and exit behavior
- `crates/shell/src/repl/*` (caging boundary checks; exit note; integration with shell integration hooks)
- `docs/project_management/standards/EXIT_CODE_TAXONOMY.md` (exit code bindings)

## Cascading implications (must be resolved in code and docs)

### Break policy keys (no compatibility)
- Legacy `world_fs.*` keys replaced by Appendix A + B MUST become hard errors.
- `PolicySnapshotV2` MUST be replaced by a new snapshot schema version.

### Routing fail-closed becomes operator intent
- `world_fs.fail_closed.routing=true` MUST prevent any host fallback execution path.
- A runtime routing failure under fail-closed MUST map to exit code `3` or `4` per Appendix B.

### REPL safety boundary (caging)
- When `world_fs.caged_required=true` is effective for the entered cwd scope, uncaged execution MUST be rejected before execution (exit `2`).
- `world.anchor_mode=follow-cwd` MUST be rejected under caging-required policy.

### Exported state env var rename
- `SUBSTRATE_WORLD_REQUIRE_WORLD` MUST be deleted.
- `SUBSTRATE_WORLD_FAIL_CLOSED_ROUTING=1|0` MUST be emitted as output-only state.

## Cross-platform requirements
- Linux: behavior parity smoke is required.
- macOS + Windows: CI parity compile must remain green for every checkpoint boundary.

## Cross-queue scan (2026-02-06)
Scan scope:
- `docs/project_management/next/sequencing.json` `sprints[]` list (feature ids, titles, and directories).
- Planning packs under `docs/project_management/next/` referenced by ADR-0018.

Overlaps found and explicit resolution:
- `world_fs_granular_allow_deny` (order `33`) and `world_fs_granular_allow_deny_appendix` (order `34`) both touch policy schema/snapshot + routing behavior in `crates/broker`, `crates/shell`, and `crates/world-agent`.
  - Resolution: the appendix workstream is ordered after the baseline workstream in `docs/project_management/next/sequencing.json` and MUST NOT execute concurrently with it.
  - Contract precedence: for Appendix A + B deltas, contracts under `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/` are authoritative and supersede prior naming (`world_fs.require_world`, `PolicySnapshotV2`) for this track.

No other overlapping queued workstreams were found in `docs/project_management/next/sequencing.json` that match this feature’s touch set keywords (`world_fs`, `policy snapshot`, `routing`, `caged`, `repl.exit_cwd`).
