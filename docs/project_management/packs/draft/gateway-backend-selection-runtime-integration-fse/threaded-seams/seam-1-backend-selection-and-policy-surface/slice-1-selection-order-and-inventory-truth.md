---
slice_id: S1
seam_id: SEAM-1
slice_kind: implementation
execution_horizon: active
status: decomposed
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
  - Ensures backend selection behaves the same way the published contract says it should, instead of leaving inventory or failure semantics inferred from one backend-specific path.
- **Scope (in/out)**:
  - In: shell selection validation, inventory-root consumption, filename/id consistency enforcement, and invalid-integration classification
  - Out: auth precedence, payload shaping, runtime artifact creation
- **Acceptance criteria**:
  - shell gateway request construction validates selection inputs in the order published by `C-01`
  - invalid integration remains separate from policy denial and dependency unavailable
  - inventory-root and filename/id expectations land in supporting docs and code-facing validation notes
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
  - `crates/shell/src/builtins/world_gateway.rs` and supporting docs use the same ordered decision path from selected backend id through allowlist enforcement.
- **Inputs/outputs**:
  - Inputs: `S00`, `docs/contracts/substrate-gateway-backend-adapter-selection.md`, `crates/shell/src/builtins/world_gateway.rs`
  - Outputs: aligned shell selection flow and supporting ADR-0046 contract/policy text
- **Thread/contract refs**:
  - `THR-01`
  - `C-01`
- **Implementation notes**:
  - keep deny-by-default allowlisting ahead of adapter dispatch
  - keep shell-side validation narrow to the selection boundary owned by this seam
- **Acceptance criteria**:
  - the selected backend id cannot reach runtime realization before the contract-defined pre-dispatch checks finish
- **Test notes**:
  - extend or update `crates/shell/tests/world_gateway.rs`
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
  - inventory roots and filename/id invariants are not only documented but also reflected in executable validation or drift-check notes.
- **Inputs/outputs**:
  - Inputs: canonical `C-01`, supporting ADR-0046 docs, shell validation points
  - Outputs: aligned docs, validation notes, and any shell-side checks required by the published rule
- **Thread/contract refs**:
  - `THR-01`
  - `C-01`
- **Implementation notes**:
  - keep rules descriptive in `docs/contracts/`; use seam-local planning and code/tests for planning IDs and implementation detail
- **Acceptance criteria**:
  - `REM-002` has clear landing evidence targets
- **Test notes**:
  - document the expected mismatch cases and how tests prove them
- **Risk/rollback notes**:
  - weak mismatch handling will let later runtime slices treat filesystem convention as a hidden contract

Checklist:
- Implement:
  - tie inventory discoverability and filename/id invariants to concrete validation points
- Test:
  - capture mismatch scenarios in shell tests or documented drift guards
- Validate:
  - confirm downstream runtime planning no longer has to invent inventory truth
