---
slice_id: S1
seam_id: SEAM-1
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - selection order or inventory resolution semantics change after `S00`
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
contracts_produced:
  - C-01
contracts_consumed:
  - C-01
open_remediations:
  - REM-002
---
### S1 - Land selection order and inventory truth at the shell boundary

- **User/system value**:
  - Makes the shell adopt published `C-01` rules before the world-service sees the request, instead of letting invalid or ambiguous backend selection leak downstream.
- **Scope (in/out)**:
  - In: shell selection validation, inventory-backed backend resolution posture, deny-by-default allowlisting, and invalid-integration classification
  - Out: auth precedence, runtime config rendering, adapter lookup, and process launch
- **Acceptance criteria**:
  - `validate_gateway_lifecycle_config` and `build_gateway_request` validate selection inputs in the order published by `C-01`
  - invalid integration remains separate from policy denial and runtime-owned dependency unavailable
  - shell docs and tests explicitly describe which inventory and mismatch cases are shell-owned versus runtime-owned
- **Dependencies**:
  - `S00`
  - `THR-01`
  - `C-01`
- **Verification**:
  - targeted shell tests around empty default backend, unsupported backend selection, allowlist denial, and inventory mismatch handling
- **Rollout/safety**:
  - fail closed on invalid selection; do not dispatch toward runtime realization on ambiguous inventory state
- **Review surface refs**:
  - `../review.md`
  - `../../review_surfaces.md`

#### S1.T1 - Align shell validation with published selection order

- **Outcome**:
  - `crates/shell/src/builtins/world_gateway.rs` uses the same ordered decision path as `C-01` from selected backend id through allowlist enforcement before runtime dispatch.
- **Inputs/outputs**:
  - Inputs: `S00`, `docs/contracts/substrate-gateway-backend-adapter-selection.md`, `crates/shell/src/builtins/world_gateway.rs`
  - Outputs: aligned shell selection flow, targeted test updates, and supporting ADR-0046 doc notes
- **Thread/contract refs**:
  - `THR-01`
  - `C-01`
- **Implementation notes**:
  - start at `validate_gateway_lifecycle_config` and `build_gateway_request`
  - keep deny-by-default allowlisting ahead of any runtime call
  - do not invent runtime binding/capability rules here; stop at “allowed backend id handed off”
- **Acceptance criteria**:
  - the selected backend id cannot reach runtime realization before shell-owned pre-dispatch checks finish
  - a well-formed but unsupported backend is rejected deterministically at the shell boundary or explicitly documented as runtime-owned
- **Test notes**:
  - preserve `world_gateway_empty_default_backend_uses_exit_code_2`
  - add a test for unsupported-but-well-formed backend rejection before socket contact
  - preserve `world_gateway_policy_failures_use_exit_code_5`
- **Risk/rollback notes**:
  - a partial alignment leaves downstream reviewers with two conflicting selection orders

Checklist:
- Implement:
  - align the shell-side selection path to the published `C-01` order
- Test:
  - exercise invalid and denied paths separately
- Validate:
  - confirm the shell boundary hands off only an allowed backend id to `SEAM-2`

#### S1.T2 - Make inventory discoverability and filename/id mismatch handling executable

- **Outcome**:
  - inventory roots and filename/id invariants are tied to executable validation notes and tests instead of being treated as planning prose only.
- **Inputs/outputs**:
  - Inputs: canonical `C-01`, supporting ADR-0046 docs, shell validation points, and `crates/shell/tests/world_gateway.rs`
  - Outputs: aligned docs, validation notes, and shell-side tests required by the published rule
- **Thread/contract refs**:
  - `THR-01`
  - `C-01`
- **Implementation notes**:
  - keep rules descriptive in `docs/contracts/`; use seam-local planning and code/tests for planning IDs and implementation detail
- **Acceptance criteria**:
  - downstream runtime planning no longer relies on implicit shell behavior for inventory mismatch semantics
  - any remaining inventory validation that cannot be owned by the shell is called out explicitly as `SEAM-2` follow-through
- **Test notes**:
  - add an explicit mismatch/unsupported-backend test or record why that proof remains runtime-owned
- **Risk/rollback notes**:
  - weak mismatch handling will let later runtime slices treat filesystem convention as a hidden contract

Checklist:
- Implement:
  - tie inventory discoverability and filename/id invariants to concrete validation points
- Test:
  - capture mismatch scenarios in shell tests or documented drift guards
- Validate:
  - confirm downstream runtime planning no longer has to invent inventory truth
