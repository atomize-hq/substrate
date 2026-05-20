---
seam_id: SEAM-3
seam_slug: codex-route-conformance-and-drift-guards
type: conformance
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
    - governance/seam-2-closeout.md
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
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S99
  status: passed
open_remediations: []
---

# SEAM-3 - Codex Route Conformance And Drift Guards

- **Goal / value**: lock the ChatGPT Codex route into deterministic regression proof so future edits cannot silently change request shaping, auth provenance, sync/stream parity, or reasoning visibility on this route.
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
- **Primary interfaces**
  - Inputs:
    - `C-14`
    - `C-15`
    - provider and public-route test surfaces under `crates/gateway/tests/`
    - route-facing docs under `crates/gateway/docs/`
  - Outputs:
    - `C-16` Codex-route conformance and drift-guard contract
    - deterministic fixtures and regression tests
    - route-specific maintenance guidance that names what must be revalidated when upstream behavior drifts
- **Key invariants / rules**:
  - route regressions should be deterministic and offline wherever possible
  - no live network dependency is required for the core drift-guard suite
  - the suite must prove no caller-visible control is silently stripped or degraded on the Codex route
  - the suite must prove sync and streaming derive from the same semantic upstream event source
  - the suite must prove encrypted reasoning remains non-public on this route
- **Dependencies**
  - Direct blockers:
    - none at pre-exec; `THR-14` and `THR-15` are published and revalidated against the landed closeouts
  - Transitive blockers:
    - any auth or route ambiguity left open by earlier seams will invalidate conformance expectations
  - Direct consumers:
    - future maintenance work outside this pack
  - Derived consumers:
    - future operator troubleshooting, route expansion, and release validation work
- **Touch surface**:
  - `crates/gateway/tests/openai_responses_conformance.rs`
  - `crates/gateway/tests/openai_shared_parity.rs`
  - `crates/gateway/src/server/openai_conformance_test_support.rs`
  - `crates/gateway/docs/openai-compatibility.md`
  - `crates/gateway/docs/OAUTH_SETUP.md`
  - `crates/gateway/docs/OAUTH_TESTING.md`
  - conformance contract docs reserved under `docs/contracts/`
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - Verify the drift-guard contract is concrete enough to enumerate route-local positive and negative cases, fixture namespaces, auth-source proofs, and required documentation updates.
  - Verify the test plan can cover sync/stream parity, `stream = true`, `store = false`, reasoning gates, continuation legality/order, and the integrated-mode no-local-read posture.
  - Verify docs and test surfaces line up so route-specific maintenance does not depend on re-reading the ADR to know what drift matters.
- **Canonical contract refs**:
  - `crates/gateway/docs/contracts/chatgpt-codex-conformance-and-drift-guard.md`
- **Risks / unknowns**:
  - Risk: fixture coverage misses an upstream semantic event or rejected-field edge and gives false confidence.
  - De-risk plan: derive the suite directly from the published route and auth contracts, not from current implementation quirks alone.
  - Risk: live upstream drift happens faster than fixtures are refreshed.
  - De-risk plan: make revalidation triggers explicit in closeout and route docs so maintenance work knows when to revisit probe evidence.
  - Risk: auth-source tests accidentally bake in standalone local behavior as integrated truth.
  - De-risk plan: keep integrated-mode and standalone-mode assertions separate and contract-backed.
- **Rollout / safety**:
  - prefer deterministic regression coverage over ad hoc manual verification
  - keep route-specific docs aligned with the same contract terms the tests assert
  - record clear stale triggers so future changes reopen the right seam instead of patching around drift
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `active` because route and auth ownership are now published, giving the conformance seam a current basis for deterministic execution planning
  - Which threads matter most: `THR-14`, `THR-15`, `THR-16`
  - What the first seam-local review should focus on: fixture boundaries, deterministic coverage strategy, integrated-versus-standalone assertions, and doc/test ownership
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-16`
  - Threads likely to advance: `THR-16`
  - Review-surface areas likely to shift after landing: `R1`, `R2`, and `R3` should each gain concrete evidence anchors and stale-trigger notes
  - Downstream seams most likely to require revalidation: future Codex-route maintenance outside this pack
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
