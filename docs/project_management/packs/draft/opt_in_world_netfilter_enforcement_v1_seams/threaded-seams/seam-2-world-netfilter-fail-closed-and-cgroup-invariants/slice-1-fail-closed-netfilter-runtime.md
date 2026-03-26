---
slice_id: S1
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "Any change to WORLD_NETFILTER_ENABLE guard semantics"
    - "Any change to nftables ruleset shape, especially DNS handling for deny-all"
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-02
  - THR-04
contracts_produced: []
contracts_consumed:
  - C-02
  - C-03
open_remediations: []
candidate_subslices: []
---
### S1 - Make requested netfilter enforcement fail closed at runtime

- **User/system value**: when the host requests isolation, the world runtime either installs the correct ruleset or fails before an unfiltered process gets network access.
- **Scope (in/out)**:
  - In:
    - remove warn-and-continue behavior from `WorldSession::setup()` and `setup_network_filter()` when `isolate_network=true`
    - treat missing `WORLD_NETFILTER_ENABLE`, domain-resolution failures, and nftables install failures as hard execution errors
    - make deny-all rules truly deny all egress, including DNS
  - Out:
    - doctor JSON publication (`SEAM-4`)
    - full smoke/conformance suite (`SEAM-5`)
- **Acceptance criteria**:
  - `crates/world/src/session.rs` no longer continues after `setup_network_filter()` fails when isolation was requested.
  - `crates/world/src/netfilter.rs` no longer treats `WORLD_NETFILTER_ENABLE` absence as a soft "not installed" outcome for isolated runs.
  - deny-all (`allowed_domains=[]`) installs no implicit DNS allow rule.
  - restrictive allowlists still resolve domains before rule install, and any resolution failure is surfaced as a hard error.
- **Dependencies**:
  - published upstream routing handoff in `../../governance/seam-1-closeout.md`
  - current runtime hotspots in `crates/world/src/session.rs` and `crates/world/src/netfilter.rs`
- **Verification**:
  - unit/logic tests covering deny-all rule generation and missing-env guard behavior
  - targeted runtime tests asserting isolated execution errors instead of warning-only degradation
- **Rollout/safety**:
  - preserve allow-all/back-compat posture when `isolate_network=false`; fail-closed behavior applies only to requested isolation
- **Review surface refs**:
  - `review.md`

#### S1.T1 - Replace soft netfilter failures with explicit execution errors

- **Outcome**: requested isolation becomes enforce-or-fail all the way through world setup.
- **Files**:
  - `crates/world/src/session.rs`
  - `crates/world/src/netfilter.rs`
- **Thread/contract refs**:
  - `THR-02`
  - `C-02`
  - `C-03`
- **Acceptance criteria**:
  - `setup_network_filter()` returns an error whenever rule installation was requested but not activated.
  - `WORLD_NETFILTER_ENABLE` absence is reported as a failure reason, not a warning-only skip.
- **Test notes**:
  - add focused tests for missing-env and install failure behavior.

Checklist:
- Implement: enforce-or-fail guard path
- Test: missing-env and install failure coverage
- Validate: no isolated execution path degrades to unfiltered warning-only behavior

#### S1.T2 - Make deny-all truly deny DNS and other egress

- **Outcome**: deny-all matches the scope brief and published host/world contract.
- **Files**:
  - `crates/world/src/netfilter.rs`
- **Thread/contract refs**:
  - `THR-02`
  - `C-03`
- **Acceptance criteria**:
  - no unconditional DNS allow rule remains in deny-all posture
  - restrictive allowlists still permit only the resolved destinations needed for their allowed domains
- **Test notes**:
  - assert deny-all emits no DNS allow rule and fails closed on unresolved hosts.

Checklist:
- Implement: deny-all ruleset correction
- Test: ruleset/unit coverage for deny-all and unresolved hosts
- Validate: scope-brief claim "deny-all denies DNS" becomes true in code
