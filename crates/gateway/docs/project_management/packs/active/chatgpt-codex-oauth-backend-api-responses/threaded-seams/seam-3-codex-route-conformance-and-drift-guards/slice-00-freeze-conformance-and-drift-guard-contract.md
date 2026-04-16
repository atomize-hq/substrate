---
slice_id: S00
seam_id: SEAM-3
slice_kind: contract_definition
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - route compatibility or semantic event rules change after the conformance contract baseline is frozen
    - auth-handoff ownership or fallback rules change after the conformance checklist is frozen
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-16
contracts_produced:
  - C-16
contracts_consumed:
  - C-14
  - C-15
open_remediations: []
---
### S00 - Freeze Conformance And Drift-Guard Contract

- **User/system value**: execution starts from one explicit conformance contract instead of rediscovering route and auth proof obligations from test files alone.
- **Scope (in/out)**:
  - In: freeze the deterministic conformance contract target, fixture and regression ownership, auth-source proof obligations, and maintenance-doc stale triggers.
  - Out: landing the final accepted contract artifact, changing route behavior, or publishing seam-exit evidence.
- **Acceptance criteria**:
  - the canonical conformance contract target is fixed at `crates/gateway/docs/contracts/chatgpt-codex-conformance-and-drift-guard.md`
  - the contract baseline names positive and negative route cases, auth-source proofs, fixture namespaces, and maintenance-doc obligations concretely enough for implementation
  - the verification checklist names the exact code, test, and doc anchors that later prove the route stays deterministic
- **Dependencies**: `../../threading.md`, `../../scope_brief.md`, `../../seam-3-codex-route-conformance-and-drift-guards.md`, `../../governance/seam-1-closeout.md`, `../../governance/seam-2-closeout.md`
- **Verification**:
  - a reviewer can explain what the conformance seam must prove without reverse-engineering current implementation diffs
  - pass condition: the owned contract baseline is concrete enough that `SEAM-3` can execute without ambiguity
- **Rollout/safety**: keep the contract deterministic and route-specific; do not hide drift-guard truth inside generic OAuth or OpenAI docs.

Checklist:
- Implement: codify the route-local sync/stream matrix, fixture ownership, and no-silent-degradation rules in `crates/gateway/tests/openai_responses_conformance.rs`
- Implement: prove auth-source and auth-failure posture in route-boundary regressions and shared error-envelope tests
- Implement: align maintenance docs so stale triggers and evidence anchors stay visible outside source code
- Validate: keep the conformance contract descriptive-only at `crates/gateway/docs/contracts/chatgpt-codex-conformance-and-drift-guard.md`
- Publish when landed: record deterministic regression anchors and `THR-16` publication in `../../governance/seam-3-closeout.md`
