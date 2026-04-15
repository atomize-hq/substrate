---
seam_id: SEAM-3
seam_slug: codex-route-conformance-and-drift-guards
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-3-codex-route-conformance-and-drift-guards.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-2-closeout.md
  required_threads:
    - THR-14
    - THR-15
  stale_triggers:
    - route-level request compatibility or semantic event rules change after `C-14` publishes
    - auth-handoff ownership, field IDs, or fallback rules change after `C-15` publishes
    - public normalized-core behavior changes in a way that invalidates the route-local fixture expectations
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
  planned_location: S99
  status: pending
open_remediations: []
---
# SEAM-3 - Codex Route Conformance And Drift Guards

## Seam Brief (Restated)

- **Goal / value**: lock the ChatGPT Codex route into deterministic regression proof so future edits cannot silently change request shaping, auth provenance, sync/stream parity, or reasoning visibility on this route.
- **Type**: `conformance`
- **Scope**
  - In:
    - deterministic sync and streaming conformance for the route-local compatibility matrix
    - regression coverage for accepted and rejected request controls, continuation synthesis/order, reasoning gating, and minimal header posture
    - verification that integrated mode does not require gateway-local auth-file reads while standalone mode remains bounded fallback
    - route-specific maintenance docs and evidence anchors for future revalidation
  - Out:
    - broad OpenAI compatibility redesign beyond this route
    - live production monitoring or continuous external probe automation
    - reopening route or auth contract ownership after `C-14` and `C-15` publish
- **Touch surface**:
  - `crates/gateway/tests/openai_responses_conformance.rs`
  - `crates/gateway/tests/openai_shared_parity.rs`
  - `crates/gateway/src/server/openai_conformance_test_support.rs`
  - `crates/gateway/docs/openai-compatibility.md`
  - `crates/gateway/docs/OAUTH_SETUP.md`
  - `crates/gateway/docs/OAUTH_TESTING.md`
  - `crates/gateway/docs/contracts/chatgpt-codex-conformance-and-drift-guard.md`
- **Verification**:
  - keep the drift-guard contract concrete enough to enumerate positive and negative route cases, fixture namespaces, auth-source proofs, and required documentation updates
  - prove sync and streaming derive from the same semantic upstream event source and that caller-visible controls are not silently stripped or degraded
  - prove integrated auth stays Substrate-owned while standalone remains bounded fallback
- **Basis posture**:
  - **Currentness**: `current` (revalidated against the landed route and auth closeouts plus the current route test/doc surfaces)
  - **Upstream closeouts assumed**:
    - `../../governance/seam-1-closeout.md`
    - `../../governance/seam-2-closeout.md`
  - **Required threads**:
    - `THR-14`
    - `THR-15`
- **Threading constraints**
  - **Upstream blockers**: none at pre-exec; `THR-14` and `THR-15` are published and revalidated
  - **Downstream blocked seams**: none in this pack; this seam publishes `THR-16` for future maintenance work
  - **Contracts produced**: `C-16`
  - **Contracts consumed**: `C-14`, `C-15`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99`
- **Why this seam needs an explicit exit gate**: future maintenance should consume closeout-backed conformance truth instead of reverse-engineering fixtures, docs, and parity rules from scattered tests
- **Expected contracts to publish**: `C-16`
- **Expected threads to publish / advance**: `THR-16`
- **Likely downstream stale triggers**: route compatibility changes, auth-source rule changes, or drift-guard evidence anchors moving
- **Expected closeout evidence**: the conformance contract baseline, deterministic regression anchors, auth-source proof, and route-specific maintenance guidance

## Slice index

- `S00` -> `slice-00-freeze-conformance-and-drift-guard-contract.md`
- `S1` -> `slice-1-lock-route-matrix-and-fixture-coverage.md`
- `S2` -> `slice-2-verify-auth-provenance-and-failure-envelope.md`
- `S3` -> `slice-3-align-maintenance-docs-and-evidence-anchors.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-3-closeout.md`
