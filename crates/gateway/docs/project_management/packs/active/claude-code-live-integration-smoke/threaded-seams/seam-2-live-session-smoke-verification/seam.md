---
seam_id: SEAM-2
seam_slug: live-session-smoke-verification
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-2-live-session-smoke-verification.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - crates/gateway/docs/project_management/packs/active/claude-code-live-integration-smoke/governance/seam-1-closeout.md
    - crates/gateway/docs/project_management/packs/active/azure-foundry-provider-transport/governance/seam-2-closeout.md
    - crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-3-closeout.md
    - crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-4-closeout.md
  required_threads:
    - THR-08
  stale_triggers:
    - docs/foundation/claude-code-c09-operator-bootstrap-contract.md changes the canonical bootstrap path, evidence hooks, or Claude Code attachment rules that the live smoke scenarios assume
    - gateway/README.md, gateway/src/router/mod.rs, or gateway/src/server/mod.rs drift enough that the observable normal, think, or continuation paths no longer match the planned smoke coverage
    - Claude Code session behavior or route-evidence posture changes materially enough that the planned redacted evidence set is no longer sufficient
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
  planned_location: S3
  status: passed
open_remediations: []
---
# SEAM-2 - Live Session Smoke Verification

## Seam Brief (Restated)

- **Goal / value**: turn the published bootstrap path into concrete Claude Code proof for normal execution, think/planner, and tool-loop continuation behavior so the live path is verified through the real client operators use, not gateway-only probes.
- **Type**: `conformance`
- **Scope**
  - In:
    - define the owned `C-10` live smoke verification contract and its canonical landing path
    - make the normal, think/planner, and tool-loop continuation branches concrete enough to execute without rereading runtime code
    - freeze the minimum redacted evidence set for each branch, including statusline, route, and optional trace surfaces
    - align operator-facing smoke procedures and checklists with the landed bootstrap contract from `SEAM-1`
  - Out:
    - redefining the bootstrap assets owned by `SEAM-1`
    - redefining the public `/v1/messages` contract or internal route policy
    - broad troubleshooting ownership work that belongs to `SEAM-3`
- **Touch surface**:
  - `docs/foundation/claude-code-c10-live-session-smoke-verification-contract.md`
  - `gateway/README.md`
  - any bounded smoke procedure or evidence-manifest surface introduced for live Claude Code verification
  - `gateway/src/router/mod.rs`
  - `gateway/src/server/mod.rs`
- **Verification**:
  - because this seam produces owned `C-10`, pre-exec verification is about making the live smoke contract, scenario coverage, and evidence posture concrete enough that implementation can proceed without guessing
  - a reviewer can state the three required live branches and the minimum redacted evidence for each without reading runtime code
  - a reviewer can identify which route or continuation anchors in the repo support the planned smoke proof
  - the smoke story stays client-real and capability-oriented rather than falling back to gateway-only or provider-only checks
- **Basis posture**:
  - **Currentness**: `current`
  - **Upstream closeouts assumed**:
    - `crates/gateway/docs/project_management/packs/active/claude-code-live-integration-smoke/governance/seam-1-closeout.md`
    - `crates/gateway/docs/project_management/packs/active/azure-foundry-provider-transport/governance/seam-2-closeout.md`
    - `crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-3-closeout.md`
    - `crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-4-closeout.md`
  - **Required threads**:
    - `THR-08`
  - **Stale triggers**:
    - `docs/foundation/claude-code-c09-operator-bootstrap-contract.md` changes the canonical bootstrap path, evidence hooks, or Claude Code attachment rules that the live smoke scenarios assume
    - `gateway/README.md`, `gateway/src/router/mod.rs`, or `gateway/src/server/mod.rs` drift enough that the observable normal, think, or continuation paths no longer match the planned smoke coverage
    - Claude Code session behavior or route-evidence posture changes materially enough that the planned redacted evidence set is no longer sufficient
- **Threading constraints**
  - **Upstream blockers**: `SEAM-1` has now published `THR-08` and `C-09`, so the seam may execute against current upstream truth
  - **Downstream blocked seams**: `SEAM-3` depends on `THR-09` and cannot safely execute until `C-10` lands and publishes live smoke truth
  - **Contracts produced**: `C-10`
  - **Contracts consumed**: `C-03`, `C-04`, `C-08`, `C-09`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S3`
- **Why this seam needs an explicit exit gate**: downstream troubleshooting work should only consume live smoke truth after the landed contract, smoke procedure, and evidence expectations agree with current runtime behavior and are recorded in closeout-backed form
- **Expected contracts to publish**: `C-10`
- **Expected threads to publish / advance**: `THR-09`
- **Likely downstream stale triggers**:
  - the smoke scenario set changes enough to alter what counts as normal, think, or continuation coverage
  - route-evidence or redaction expectations change enough to alter downstream troubleshooting intake
  - Claude Code session behavior changes enough that the live smoke proof must be rewritten
- **Expected closeout evidence**:
  - the landed `C-10` contract artifact
  - operator-facing smoke procedure or checklist surfaces aligned with runtime behavior
  - redacted evidence expectations aligned with actual route and continuation anchors
  - closeout accounting for any planned-versus-landed delta that affects `SEAM-3`

## Slice index

- `S1` -> `slice-1-freeze-live-session-smoke-contract-and-coverage.md`
- `S2` -> `slice-2-deliver-smoke-procedure-and-evidence-manifest.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-2-closeout.md`
