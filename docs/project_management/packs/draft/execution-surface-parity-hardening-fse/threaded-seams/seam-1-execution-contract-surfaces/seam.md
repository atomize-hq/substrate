---
seam_id: SEAM-1
seam_slug: execution-contract-surfaces
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-1-execution-contract-surfaces.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - `crates/shell/src/execution/policy_snapshot.rs` changes the canonical four-case world-network routing rules or helper surface
    - `crates/replay/src/replay/executor.rs` continues deriving replay routing from replay-local heuristics instead of the shared policy-snapshot contract
    - `crates/shell/src/execution/manager.rs`, `crates/shell/src/scripts/bash_preexec.rs`, or execution-mode routing changes alter `SUBSTRATE_ENABLE_PREEXEC`, `builtin_command`, or canonical trace omission semantics
    - `docs/project_management/packs/active/world_process_exec_tracing_parity/manual_testing_playbook.md` or `smoke/_core.sh` changes Case B expectations without updating the published behavior matrix
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S99
  status: passed
open_remediations: []
---
# SEAM-1 - Execution contract surfaces

## Seam Brief (Restated)

- **Goal / value**: publish one authoritative execution contract for replay routing and command-mode tracing validation so replay, tracing docs, and the active WPEP pack stop reasoning from divergent local assumptions.
- **Type**: integration
- **Scope**
  - In:
    - define the producer-side contract for `C-01` and `C-02` tightly enough that implementation can proceed without local re-interpretation
    - align replay request construction with the canonical `policy_snapshot.net_allowed` and world-network routing rules used by shell execution
    - make the mode-by-platform behavior matrix explicit for `world_process_*`, `builtin_command`, and `SUBSTRATE_ENABLE_PREEXEC`
    - update the validation surfaces that own WPEP Case B so they assert the published contract instead of a proxy
    - publish downstream-facing docs and governance evidence for `THR-01`
  - Out:
    - interactive REPL terminal-loss handling and exit taxonomy changes owned by `SEAM-2`
    - downstream docs lock-in and drift-guard work owned by `SEAM-3`
    - broad trace-schema redesign, new CLI/config surface, or backend architecture work beyond what the two owned contracts require
- **Touch surface**: `crates/replay/src/replay/executor.rs`, `crates/shell/src/execution/policy_snapshot.rs`, `crates/shell/src/execution/manager.rs`, `crates/shell/src/scripts/bash_preexec.rs`, `crates/shell/src/execution/routing/dispatch/exec.rs`, `docs/REPLAY.md`, `docs/TRACE.md`, `docs/internals/env/inventory.md`, and `docs/project_management/packs/active/world_process_exec_tracing_parity/`.
- **Verification**:
  - `C-01` becomes concrete enough that replay-local and agent-backed replay can prove the same four-case routing outcome as the shell/world path.
  - `C-02` becomes concrete enough that the runtime, playbook, and smoke checks can all answer the same questions about `world_process_*`, `builtin_command`, and `SUBSTRATE_ENABLE_PREEXEC` without leaking command bodies.
  - The producer seam does not require its own final accepted contract artifact as a pre-exec input; publication and accepted evidence are handled through `S99` and closeout.
- **Basis posture**:
  - Currentness: current
  - Upstream closeouts assumed: none
  - Required threads: `THR-01`
  - Stale triggers: see frontmatter `basis.stale_triggers`
- **Threading constraints**
  - Upstream blockers: existing policy-snapshot semantics, trace omission rules, and the current WPEP validation surfaces already in the repo
  - Downstream blocked seams: `SEAM-3`
  - Contracts produced: `C-01`, `C-02`
  - Contracts consumed: the current shell/world routing semantics and the existing trace-safety posture already documented in `docs/TRACE.md` and the WPEP pack

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99` (`slice-99-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: `SEAM-1` is the producer seam for both `THR-01` contracts. Downstream conformance work cannot promote on inferred replay or tracing semantics.
- **Expected contracts to publish**: `C-01`, `C-02`
- **Expected threads to publish / advance**: `THR-01` from `identified` to `published`
- **Likely downstream stale triggers**: world-network routing-rule drift, replay helper bypass, preexec/builtin emission changes, or WPEP Case B assertion changes that no longer match runtime truth
- **Expected closeout evidence**: landed replay-helper integration and tests, landed behavior-matrix publication and Case B validation updates, plus explicit `THR-01` publication accounting

## Slice index

- `S00` -> `slice-00-routing-and-tracing-contract-definition.md`
- `S1` -> `slice-1-replay-routing-parity.md`
- `S2` -> `slice-2-tracing-behavior-matrix.md`
- `S3` -> `slice-3-contract-publication-surfaces.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
