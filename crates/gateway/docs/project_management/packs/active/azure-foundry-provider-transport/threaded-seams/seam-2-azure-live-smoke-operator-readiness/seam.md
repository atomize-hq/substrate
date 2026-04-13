---
seam_id: SEAM-2
seam_slug: azure-live-smoke-operator-readiness
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-2-azure-live-smoke-operator-readiness.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-3-closeout.md
    - docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-4-closeout.md
    - docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-5-closeout.md
  required_threads:
    - THR-06
    - THR-07
  stale_triggers:
    - `docs/foundation/azure-foundry-c07-runtime-transport-contract.md` changes Azure auth, base URL, request-body invariance, or deployment-selection rules in a way that invalidates the smoke path
    - the landed `/v1/messages` or internal routing behavior changes the practical think/default smoke route
    - live Azure evidence reveals new operator-facing failure modes or redaction requirements not covered by the seam-local verification plan
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
# SEAM-2 - Azure Live Smoke And Operator Readiness

## Seam Brief (Restated)

- **Goal / value**: turn the published `C-07` Azure transport contract into a real operator verification path so a user with real Azure credentials can configure the gateway, route think and default traffic correctly, and understand failures without reopening the landed public or normalization seams.
- **Type**: `conformance`
- **Scope**
  - In:
    - define the owned `C-08` operator verification contract for live smoke flow, redacted evidence, success signals, and troubleshooting taxonomy
    - document and deliver the real `/v1/messages` smoke path for `Kimi-K2-Thinking` and `Kimi-K2.5`
    - add any bounded runtime or operator-facing diagnostics needed to distinguish auth, URL, mapping, and route failures without leaking secrets
  - Out:
    - redefining `C-07` transport truth
    - redesigning router policy or public API semantics
    - broad observability platform work beyond the bounded operator-readiness surface
- **Touch surface**:
  - `gateway/README.md`
  - `gateway/config/default.example.toml`
  - `gateway/config/models.example.toml`
  - `gateway/src/cli/mod.rs`
  - `gateway/src/server/mod.rs`
  - seam-owned operator verification notes, smoke harnesses, or redacted evidence surfaces under `docs/` or `gateway/tests/`
- **Verification**:
  - the seam is `exec-ready` because `SEAM-1` has published `C-07`, the live-smoke and troubleshooting contract is concrete in seam-local planning, and the pre-exec review gates can now be evaluated against landed upstream reality
  - execution must prove a real `/v1/messages` smoke path for both Azure Kimi routes, redacted success evidence, and troubleshooting guidance for auth, URL, deployment, and mapping failures
  - accepted or published `C-08` evidence belongs to landing, closeout, and outbound `THR-07` publication rather than this pre-exec baseline
- **Basis posture**:
  - **Currentness**: `current`
  - **Upstream closeouts assumed**:
    - `../../governance/seam-1-closeout.md`
    - `docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-3-closeout.md`
    - `docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-4-closeout.md`
    - `docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-5-closeout.md`
  - **Required threads**: `THR-06`, `THR-07`
  - **Stale triggers**:
    - `C-07` changes Azure auth, base URL, deployment-selection, or request-body invariance in a way that invalidates the smoke path
    - the landed `/v1/messages` route or routing policy changes the operator flow for think/default traffic
    - live Azure evidence reveals new failure signatures or redaction constraints not covered by the seam-local troubleshooting plan
- **Threading constraints**
  - **Upstream blockers**: none; `THR-06` is now published from `SEAM-1`
  - **Downstream blocked seams**: none extracted yet; this seam publishes `THR-07` for future Azure operations
  - **Contracts produced**: `C-08`
  - **Contracts consumed**: `C-07`, `C-03`, `C-04`, `C-05`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S3`
- **Why this seam needs an explicit exit gate**: future Azure operators should only consume `SEAM-2` once the smoke path, redacted evidence chain, and troubleshooting taxonomy have landed in closeout-backed form
- **Expected contracts to publish**: `C-08`
- **Expected threads to publish / advance**: `THR-07`
- **Likely downstream stale triggers**:
  - live Azure success or failure signatures change materially after the smoke path lands
  - `C-07` or the `/v1/messages` route changes in a way that invalidates the recorded operator steps
  - a future operational consumer needs new deployment or secret-topology assumptions that the seam does not publish
- **Expected closeout evidence**:
  - canonical `C-08` operator verification contract
  - redacted live smoke evidence for both Azure Kimi routes
  - troubleshooting guidance for auth, URL, deployment, and mapping failures
  - closeout accounting for any planned-versus-landed deltas that affect future operators

## Slice index

- `S1` -> `slice-1-freeze-operator-verification-contract.md`
- `S2` -> `slice-2-deliver-live-smoke-procedure-and-troubleshooting.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-2-closeout.md`
