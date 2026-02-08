# world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment — plan

## Scope
- Feature directory: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment`
- Existing (base) Planning Pack:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/`
- Orchestration branch (shared with base pack):
  - `feat/world-fs-granular-allow-deny-appendix`
- Add-on purpose: close remaining contract gaps discovered after the base pack was marked “completed”.

## What happened (facts + evidence)
Operationally, the base pack completed (slice flow + checkpoints + CI gates), but operator-facing behavior reveals a contract mismatch:
- `substrate policy show` prints an effective policy whose `world_fs` block is still V2-shaped (`mode/isolation/require_world`) even though Appendix A+B are V3 and explicitly forbid backwards compatibility for operator-facing surfaces.

## Authoritative inputs (source of truth)
- ADR: `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
- Appendix contract (output contract + no-backcompat):
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md`
    - §1.3 Output contract: effective policy display (`substrate policy show`) (Appendix A.6)
    - §5 No backwards compatibility
- Appendix schema (policy patch schema + snapshot schema):
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md`
- Appendix protocol (snapshot transport; V3 lockstep):
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/PROTOCOL.md`
- Appendix env contract (exported state rename + deletion):
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/ENV.md`

## Required research checklist (grounding; inspected during planning)
- Output contract requirements:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md`
- Patch schema requirements (legacy keys invalid):
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md`
- Current effective policy print/serialize path:
  - `crates/broker/src/policy.rs` (V2-shaped `PolicyFileV2`, `impl Serialize for Policy`)
  - `crates/shell/src/execution/policy_cmd.rs` (`print_policy` serializes `Policy`)
- Current effective policy derivation/defaulting:
  - `crates/broker/src/effective_policy.rs` (`validate_and_finalize_effective_policy`)
- Current host snapshot generation:
  - `crates/shell/src/execution/policy_snapshot.rs` (builds `PolicySnapshotV2`)
- Current world-agent snapshot acceptance/rejection behavior + tests:
  - `crates/world-agent/src/service.rs`, `crates/world-agent/src/pty.rs`
  - `crates/world-agent/tests/wfgad1_policy_snapshot_v2_rejection.rs`
- Request/response model carrying snapshots:
  - `crates/agent-api-types/src/lib.rs` (`PolicySnapshotV2`, `ExecuteRequest`)
- Inventory of downstream legacy consumers (seed set; see `impact_map.md`):
  - `crates/shell/src/execution/platform/*.rs` (doctor JSON surfaces)
  - `docs/CONFIGURATION.md`, `docs/WORLD.md`, `docs/reference/env/contract.md`

## Goal (“done means…”)
This add-on is done when:
- `substrate policy show` output matches Appendix A.6 exactly:
  - V3 operator-facing keys (no V2 `world_fs.mode|isolation|require_world|enforcement` keys).
  - When `world_fs.host_visible=false`, `discover/read/write` render explicitly with `allow_list` + `deny_list`, and empty `deny_list` values render as `[]` (YAML + `--json`).
- “No backwards compatibility” is enforced per Appendix contract + schema:
  - Legacy policy keys are rejected as invalid config (exit `2`).
  - World-agent rejects any snapshot `schema_version != 3` as a protocol error and accepts only `PolicySnapshotV3` (`schema_version=3`).
- Host snapshot pipeline emits `PolicySnapshotV3` per Appendix schema/protocol and hashing is deterministic.
- Downstream operator-facing surfaces (doctor/health/docs/trace metadata) are aligned to the post-V3 story and do not present V2 keys as operator-facing.
- Deterministic tests and smoke gates cover the above so regressions fail fast.

## Guardrails (non-negotiable)
- Specs are the single source of truth (see “Authoritative inputs”).
- Planning Pack docs are edited only on the orchestration branch.
- Do not edit planning docs inside task worktrees.
- Preserve hard-error taxonomy: actionable config/policy errors remain exit `2` on host.
- Slice sizing: each triad task must fit within ≤ 108,800 tokens of context.

## Slices / triads
- WFGADAXA0: Effective policy display is V3-shaped and contract-tested (Appendix A.6).
- WFGADAXA1: PolicySnapshotV3 protocol + strict world-agent validation (schema_version=3 only).
- WFGADAXA2: Downstream surfaces + docs alignment (doctor/health/trace/env docs) to remove V2 operator-facing drift.

CI checkpointing:
- Single checkpoint (CP1) after WFGADAXA2: cross-platform compile parity + feature smoke.
