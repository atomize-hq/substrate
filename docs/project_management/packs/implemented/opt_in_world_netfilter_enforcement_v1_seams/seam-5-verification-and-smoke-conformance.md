---
seam_id: SEAM-5
seam_slug: verification-and-smoke-conformance
type: conformance
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - seam-1-closeout.md
    - seam-2-closeout.md
    - seam-3-closeout.md
    - seam-4-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
    - THR-04
    - THR-05
  stale_triggers:
    - "Any change to net_allowed canonicalization/validation rules or world_network routing semantics requires SEAM-5 revalidation."
    - "Any change to world.net.filter precedence, override applicability, or exported parity env semantics requires SEAM-5 revalidation."
    - "Any change to runtime failure taxonomy, attach-or-fail behavior, or deny-all DNS semantics requires SEAM-5 revalidation."
    - "Any change to doctor endpoint schema, field naming, or shell-side rendering/passthrough requires SEAM-5 revalidation."
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

# SEAM-5 - Conformance: tests + smoke prevent drift

- **Goal / value**: Lock in correctness and back-compat with unit tests, privileged integration tests, and manual smoke guidance (including macOS Lima) so enforcement cannot silently regress.
- **Scope**
  - In:
    - Unit tests for Snapshot V3 canonicalization/validation.
    - Tests for host snapshot builder populating `net_allowed`.
    - Tests that world-service uses snapshot allowlists (not broker state).
    - Logic-level tests for deny-all implying no DNS allow rule.
    - Privileged integration tests (ignored) for actual nft/cgroup behavior.
    - Manual smoke playbook for macOS Lima: deny-all fails ping (both command and REPL), allow-all succeeds.
  - Out:
    - Adding new enforcement functionality (owned by earlier seams).
- **Primary interfaces**
  - Inputs:
    - Published contracts from `SEAM-1..SEAM-4`.
  - Outputs:
    - Regression suite and repeatable manual smoke instructions.
- **Key invariants / rules**:
  - “Fail-closed when requested” must remain invariant across all execution modes.
  - Back-compat default must remain invariant unless explicitly opted in.
- **Dependencies**
  - Direct blockers:
    - none; `SEAM-1` through `SEAM-4` now publish the contracts and threads this seam consumes.
  - Transitive blockers:
    - none
- **Touch surface**:
  - `transport-api-types` tests
  - `crates/shell` tests around snapshot builder and routing decisions
  - `crates/world` tests around rule generation
  - `docs/reference/world/verification` or related smoke areas
- **Verification**:
  - This seam is itself the verification surface and is now concrete enough to execute.
- **Risks / unknowns**:
  - Risk: privileged tests are fragile across environments and may be skipped.
  - De-risk plan: keep logic-level tests comprehensive and ensure smoke playbooks are crisp and minimal.
- **Rollout / safety**:
  - Gate broad opt-in behind having the smoke and unit-level invariants in place.
- **Downstream decomposition context**:
  - Why this seam is now `active`: `SEAM-4` closeout records `seam_exit_gate.status: passed`, `promotion_readiness: ready`, `gates.post_exec.landing: passed`, and `THR-05` publication, so the final conformance seam can now plan against landed upstream truth instead of provisional handoffs.
  - Which threads matter most: `THR-05` for the doctor contract revalidation, plus `THR-01` through `THR-04` as the upstream config, routing, and runtime invariants the regression/smoke surfaces must keep intact.
  - What the seam-local review now focuses on: cross-seam drift between config/routing/runtime/doctor contracts, privileged Linux verification posture, and the macOS Lima smoke flow.
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - none (consumes contracts)
  - Threads likely to advance:
    - `THR-*` to `revalidated`/`closed` as test coverage lands
  - Review-surface areas likely to shift after landing:
    - smoke playbook and doctor expectations
  - Downstream seams most likely to require revalidation:
    - none (terminal conformance seam)
