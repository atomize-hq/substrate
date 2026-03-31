---
seam_id: SEAM-1
seam_slug: effective-config-classifier
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-1-effective-config-classifier.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - docs/reference/env/contract.md changes effective-config precedence or the workspace override-ignore rule
    - resolve_effective_config semantics or signature change in crates/shell/src/execution/config_model.rs
    - diagnostics routing or exit-code handling changes for user/config failures
    - adjacent diagnostics packs modify health or shim-doctor call paths before this seam publishes THR-01
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S3
  status: pending
open_remediations: []
---
# SEAM-1 - Effective config classifier

## Seam Brief (Restated)

- **Goal / value**: produce one canonical diagnostics-side answer to "is the world enabled?" so downstream seams can distinguish disabled-by-choice from enabled-but-broken without duplicating precedence logic.
- **Type**: integration (shared config classification + fail-fast routing)
- **Scope**
  - In:
    - Map CLI overrides `--world` / `--no-world` into the shared effective-config resolver inputs
    - Resolve `world.enabled` once for both `substrate shim doctor` and `substrate health`
    - Fail fast with stderr + exit `2` on config-resolution errors before any probe or output happens
    - Define and publish the producer-side contract `C-01` so downstream seams consume one classifier instead of local heuristics
  - Out:
    - Disabled/skipped text copy and JSON status publication (owned by downstream seams)
    - Health summary aggregation rules (owned by downstream seams)
    - Cross-platform smoke evidence beyond proving the classifier and config-error posture
- **Touch surface**: `crates/shell/src/execution/config_model.rs`, `crates/shell/src/execution/routing.rs`, `crates/shell/src/builtins/shim_doctor/report.rs`, `crates/shell/src/builtins/health.rs`, and resolver precedence/error tests.
- **Verification**:
  - Config-resolution failure exits `2` for **both** entrypoints, emits stderr, and performs no probe or user-facing diagnostic output.
  - Precedence and workspace override-ignore rules remain authoritative to `resolve_effective_config` (no ad-hoc local precedence).
  - The produced contract `C-01` is concrete enough for downstream planning and implementation (rules, boundaries, and a verification checklist exist), without requiring post-exec publication/acceptance evidence as a pre-exec input.
- **Basis posture**:
  - Currentness: current (revalidated against current repo docs/code; cross-queue risk remains captured as stale triggers)
  - Upstream closeouts assumed: none
  - Required threads: `THR-01`
  - Stale triggers: see frontmatter `basis.stale_triggers`
- **Threading constraints**
  - Upstream blockers: `docs/reference/env/contract.md` precedence contract; existing resolver + routing behavior in `crates/shell/src/execution/*`
  - Downstream blocked seams: `SEAM-2`, `SEAM-3`
  - Contracts produced: `C-01`
  - Contracts consumed: external precedence contract (`docs/reference/env/contract.md`) and existing resolver semantics

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S3` (`slice-3-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: it is the producer seam for the shared classifier + config-error posture; downstream seams must not proceed on inferred semantics.
- **Expected contracts to publish**: `C-01`
- **Expected threads to publish / advance**: `THR-01`
- **Likely downstream stale triggers**: effective-config precedence drift; diagnostics routing drift; divergence between `health` and `shim doctor` classification/error paths.
- **Expected closeout evidence**: landed helper + call-site integration, plus tests proving exit-2-before-probe/output for both commands, and a recorded “thread published” statement for `THR-01`.

## Slice index

- `S1` -> `slice-1-contract-definition-c-01.md`
- `S2` -> `slice-2-integrate-classifier-and-tests.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
