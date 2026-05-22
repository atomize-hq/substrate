---
seam_id: SEAM-1
seam_slug: operator-boundary-and-command-contract
type: integration
status: landed
execution_horizon: future
plan_version: v3
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - command spelling changes for `substrate world gateway sync|status|restart`
    - absent-state wording or exit-code mapping changes
    - stable wiring env semantics change
    - Substrate versus `substrate-gateway` ownership split changes
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

# SEAM-1 - Operator boundary and command contract

- **Goal / value**:
  - Lock one Substrate-owned operator boundary for gateway lifecycle, wiring discovery, absent-state behavior, stable env semantics, and exit-code taxonomy.
  - Prevent archived ADR-0023 wording, gateway-internal behavior, or stale path references from becoming the de facto operator contract.
- **Scope**
  - In:
    - `substrate world gateway sync`, `substrate world gateway status`, and `substrate world gateway restart`
    - `status --json` as the authoritative wiring entrypoint
    - stable non-secret env outputs `SUBSTRATE_LLM_OPENAI_BASE_URL` and `SUBSTRATE_LLM_ANTHROPIC_BASE_URL`
    - exit-code boundaries for `0`, `2`, `3`, `4`, and `5`
    - the durable ownership split between Substrate and `substrate-gateway`
  - Out:
    - the detailed `status --json` field list
    - the policy decision table over ADR-0027 keys
    - typed world-service endpoint shapes or parity evidence
    - provisioning changes or final docs/quality-gate lock-in
- **Primary interfaces**
  - Inputs:
    - ADR-0040 user contract language
    - `pre-planning/spec_manifest.md`
    - `pre-planning/minimal_spec_draft.md`
    - external-owner constraints from ADR-0027, ADR-0017, ADR-0028, ADR-0041, and ADR-0042
  - Outputs:
    - one authoritative operator contract for the command family
    - one ownership table for Substrate-owned versus gateway-owned behavior
    - downstream-ready boundaries for status schema, policy evaluation, runtime transport, and docs validation
- **Key invariants / rules**:
  - `substrate world gateway status --json` is the authoritative machine-readable wiring surface
  - stable env names remain fixed and point to Substrate-managed gateway endpoints rather than upstream providers
  - no new config family or policy file family is introduced
  - gateway-local config, admin mutation, and token persistence are not required Substrate operator surfaces
  - Substrate owns policy, world placement, lifecycle, secret delivery, operator UX, and canonical tracing
  - `substrate-gateway` owns runtime internals and normalized event generation
- **Dependencies**
  - Direct blockers:
    - none inside this extracted pack
  - Transitive blockers:
    - stale ADR / archived-pack references can still confuse downstream planners until this seam publishes the authoritative contract
  - Direct consumers:
    - `SEAM-2`
    - `SEAM-3`
    - `SEAM-4`
  - Derived consumers:
    - operator docs
    - shell builtins
    - quality-gate evidence
- **Touch surface**:
  - Primary planning surfaces:
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md`
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/spec_manifest.md`
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md`
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/workstream_triage.md`
    - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - Likely downstream code and operator-doc surfaces once execution starts:
    - `crates/shell/src/execution/cli.rs`
    - `crates/shell/src/builtins/mod.rs`
    - `crates/shell/src/builtins/world_gateway.rs`
    - `crates/shell/tests/world_gateway.rs`
    - `docs/USAGE.md`
- **Verification**:
  - This seam produces owned contract `C-01`. Pre-exec verification now passes because the seam-local contract-definition bundle makes the contract concrete enough for execution: exact command spellings, absent-state rules, stable env semantics, exit-code split, ownership table, and downstream publication surfaces.
  - Execution must still prove:
    - the command family is singular and stable
    - archived command orderings are excluded from the current contract
    - exit `2`, `3`, `4`, and `5` are distinguished cleanly
    - the stable env names and `status --json` authority rule stay synchronized
- **Canonical contract refs**:
  - `docs/contracts/substrate-gateway-operator-contract.md`
- **Risks / unknowns**:
  - Risk:
    - archived planning still carries alternate command ordering and older ownership assumptions
  - De-risk plan:
    - make the accepted command family and ownership split explicit in the first seam-local review and carry the same wording into operator docs
  - Risk:
    - exit-code text can collapse policy denial, dependency failure, and invalid integration state into one bucket
  - De-risk plan:
    - keep the `0|2|3|4|5` split explicit in the contract before downstream seams reuse it
  - Risk:
    - stale `packs/active/...` references can leak back into downstream planning
  - De-risk plan:
    - treat stale links as a seam-local review item and record correction follow-ups in closeout if they remain external
- **Rollout / safety**:
  - This seam is safe to land first because it clarifies operator-facing ownership without depending on typed runtime details.
  - Safety depends on preventing gateway-internal surfaces from becoming required operator inputs.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`
    - `future` because this seam has landed with a passed seam-exit gate and left the forward planning window.
  - Which threads matter most
    - `THR-01`
  - What the first seam-local review should focus on
    - command spelling
    - absent-state behavior
    - stable env semantics
    - exit-code taxonomy
    - ownership split wording
    - the `S00` contract-definition bundle that makes `C-01` concrete before implementation slices run
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-01`
  - Threads likely to advance:
    - `THR-01`
  - Review-surface areas likely to shift after landing:
    - operator workflow
    - ownership boundary diagram
    - status-entrypoint wording
  - Downstream seams most likely to require revalidation:
    - `SEAM-2`
    - `SEAM-3`
    - `SEAM-4`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
