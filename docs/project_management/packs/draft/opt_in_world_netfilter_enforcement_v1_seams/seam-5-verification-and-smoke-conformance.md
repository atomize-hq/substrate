---
seam_id: SEAM-5
seam_slug: verification-and-smoke-conformance
type: conformance
status: proposed
execution_horizon: next
plan_version: v1
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads: []
  stale_triggers:
    - "Any change to enforcement semantics or config gating"
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: reserved_final_slice
  status: pending
open_remediations: []
---

# SEAM-5 - Conformance: tests + smoke prevent drift

- **Goal / value**: Lock in correctness and back-compat with unit tests, privileged integration tests, and manual smoke guidance (including macOS Lima) so enforcement cannot silently regress.
- **Scope**
  - In:
    - Unit tests for Snapshot V3 canonicalization/validation.
    - Tests for host snapshot builder populating `net_allowed`.
    - Tests that world-agent uses snapshot allowlists (not broker state).
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
    - `SEAM-1` and `SEAM-2` for core enforcement semantics
  - Transitive blockers:
    - `SEAM-3` and `SEAM-4` for operational end-to-end confidence
- **Touch surface**:
  - `agent-api-types` tests
  - `crates/shell` tests around snapshot builder and routing decisions
  - `crates/world` tests around rule generation
  - `docs/manual_verification` or related smoke areas
- **Verification**:
  - This seam is itself the verification surface.
- **Risks / unknowns**:
  - Risk: privileged tests are fragile across environments and may be skipped.
  - De-risk plan: keep logic-level tests comprehensive and ensure smoke playbooks are crisp and minimal.
- **Rollout / safety**:
  - Gate broad opt-in behind having the smoke and unit-level invariants in place.
- **Downstream decomposition context**:
  - Why this seam is `next`: core config/routing/runtime handoffs are now landed, but it still waits on `SEAM-4` to publish the doctor observability contract before active planning can fully revalidate against the final operator surface.
  - Which threads matter most: whichever threads remain `identified` after landing earlier seams.
  - What the first seam-local review should focus on: what is “must not regress” and how to make privileged tests reliable/optional.
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - none (consumes contracts)
  - Threads likely to advance:
    - `THR-*` to `revalidated`/`closed` as test coverage lands
  - Review-surface areas likely to shift after landing:
    - smoke playbook and doctor expectations
  - Downstream seams most likely to require revalidation:
    - none (terminal conformance seam)
