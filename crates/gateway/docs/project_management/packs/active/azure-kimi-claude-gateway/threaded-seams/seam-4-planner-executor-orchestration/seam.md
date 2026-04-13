---
seam_id: SEAM-4
seam_slug: planner-executor-orchestration
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-4-planner-executor-orchestration.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-2-closeout.md
    - ../../governance/seam-3-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-04
  stale_triggers:
    - "`docs/foundation/anthropic-messages-c03-contract.md` changes public session/tool-loop guarantees or the thin-adapter boundary in a way that affects internal policy assumptions"
    - "`docs/foundation/azure-kimi-c02-normalized-event-contract.md` changes normalized tool/action/final semantics in a way that affects planning-to-execution handoff"
    - routing work starts depending on provider-specific parsing details or exposing planner/executor roles as public identity
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
# SEAM-4 - Planner Executor Orchestration

## Seam Brief (Restated)

- **Goal / value**: define and implement internal planner/executor routing policy that uses the landed public surface and normalized core without leaking role selection into the external gateway contract.
- **Type**: `integration`
- **Scope**
  - **In**:
    - internal routing and model-role selection policy
    - session-state handoff between planning and execution turns
    - policy diagnostics and configuration that remain internal-only
    - explicit separation between normalized event consumption and orchestration decisions
  - **Out**:
    - Azure provider parsing or raw Kimi normalization
    - public Anthropic Messages contract changes
    - exposing separate external planner and executor backend identities
    - final external identity or downstream structured-event lock-in work owned by `SEAM-5`
- **Touch surface**:
  - `gateway/src/router/`
  - session-state handoff logic and internal config surfaces
  - policy-facing diagnostics and verification notes/tests
  - seam-local contract and review artifacts for `C-04`
- **Verification**:
  - the owned `C-04` policy contract is concrete enough that execution does not invent route-selection or handoff semantics on the fly
  - planning-to-execution handoff remains explainable in terms of normalized events rather than raw provider behavior
  - internal diagnostics may surface route decisions, but public behavior and public docs stay capability-oriented
  - the landed `C-03` surface remains singular even when internal role selection changes
- **Basis posture**:
  - **Currentness**: `current`
  - **Upstream closeouts assumed**: `../../governance/seam-1-closeout.md`, `../../governance/seam-2-closeout.md`, `../../governance/seam-3-closeout.md`
  - **Required threads**: `THR-01`, `THR-02`, `THR-04`
  - **Stale triggers**:
    - `C-03` changes public session/tool-loop guarantees or the thin Responses-later boundary
    - `C-02` changes normalized tool/action/final semantics used by policy handoff
    - orchestration starts depending on provider parsing details or public role exposure
- **Threading constraints**
  - **Upstream blockers**: `THR-01` and `THR-02` remain current inputs and `SEAM-3` closeout now fixes the public-surface basis this seam must not violate
  - **Downstream blocked seams**: `SEAM-5`
  - **Contracts produced**: `C-04`
  - **Contracts consumed**: `C-01`, `C-02`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S3`
- **Why this seam needs an explicit exit gate**: downstream conformance work cannot safely consume internal policy truth until planner/executor boundaries, state handoff, and public-identity containment are closed out with evidence instead of assumptions
- **Expected contracts to publish**: `C-04`
- **Expected threads to publish / advance**: `THR-04` from `identified` to `published`
- **Likely downstream stale triggers**:
  - `SEAM-5` if internal policy leaks into public identity, config, or downstream event assumptions
  - any future adapter seam if planner/executor handoff starts depending on provider parsing or public Anthropic surface details
- **Expected closeout evidence**:
  - a canonical `C-04` policy contract note or equivalent internal source of truth
  - verification for planning-to-execution handoff on normalized events
  - evidence that public behavior remains singular even as internal role selection changes

## Slice index

- `S1` -> `slice-1-freeze-planner-executor-policy-contract.md`
- `S2` -> `slice-2-deliver-policy-handoff-and-verification.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-4-closeout.md`
