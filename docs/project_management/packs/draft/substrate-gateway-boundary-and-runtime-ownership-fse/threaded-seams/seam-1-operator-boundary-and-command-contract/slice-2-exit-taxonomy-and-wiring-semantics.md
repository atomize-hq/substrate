---
slice_id: S2
seam_id: SEAM-1
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers: []
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
contracts_produced: []
contracts_consumed:
  - C-01
open_remediations: []
---
### S2 - Exit taxonomy and wiring semantics

- **User/system value**: operators and downstream seams get one explicit answer for stable wiring discovery and the meaning of exit `0|2|3|4|5`, rather than inferring it from ADR prose or runtime-private behavior.
- **Scope (in/out)**:
  - In:
    - publish the stable semantics of `SUBSTRATE_LLM_OPENAI_BASE_URL` and `SUBSTRATE_LLM_ANTHROPIC_BASE_URL`
    - lock the rule that `status --json` is the machine-readable wiring authority and human-readable `status` may abbreviate but not redefine it
    - align absent-state wording with the exit taxonomy
    - update operator-facing docs/tests so invalid integration, transient runtime failure, dependency unavailability, and policy denial stay distinct
  - Out:
    - `client_wiring.*` field inventory and omission rules
    - fail-closed decision logic and trust-boundary policy tables
- **Acceptance criteria**:
  - both stable env names have one published meaning and still point at Substrate-managed gateway endpoints
  - human-readable wiring output does not replace the `status --json` authority
  - exit `2|3|4|5` remain distinct and testable
  - operator docs and CLI output surfaces use the same taxonomy
- **Dependencies**:
  - `C-01`
  - `THR-01`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md`
  - `docs/USAGE.md`
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - touched CLI/builtin/test surfaces from `S1`
- **Verification**:
  - targeted readback of `contract.md` and `docs/USAGE.md`
  - CLI/status tests that distinguish exit boundaries and absent-state posture
- **Rollout/safety**: preserve fail-closed posture and avoid leaking upstream-provider or gateway-local semantics into the stable operator contract.
- **Review surface refs**: `../../review_surfaces.md` R1 and R2

#### S2.T1 - Publish stable wiring discovery semantics

- **Outcome**: operators and in-world clients inherit one stable discovery rule for gateway endpoints.
- **Inputs/outputs**:
  - Inputs: `C-01`, ADR-0040 client-wiring contract, feature-local `contract.md`, `docs/USAGE.md`
  - Outputs: explicit stable env semantics plus the rule that `status --json` is authoritative
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**:
  - keep this slice at the operator-contract layer; do not pull field-by-field schema detail forward from `SEAM-2`
- **Acceptance criteria**:
  - the env names remain stable and non-secret
  - the docs do not imply direct upstream-provider or guaranteed host reachability semantics
- **Test notes**:
  - add readback or CLI coverage for status text/json authority boundaries where practical
- **Risk/rollback notes**:
  - do not let text output become a second wiring contract

Checklist:
- Implement: contract/docs wording for stable env semantics
- Test: targeted readback or CLI assertions
- Validate: compare wording to ADR-0040
- Cleanup: remove stale wiring-discovery phrasing from touched docs

#### S2.T2 - Lock the gateway lifecycle/status exit taxonomy

- **Outcome**: runtime and docs work inherit one explicit operator-visible failure split.
- **Inputs/outputs**:
  - Inputs: `C-01`, ADR-0040 exit codes, operator absent-state posture
  - Outputs: aligned exit-taxonomy wording plus test expectations for `0|2|3|4|5`
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**:
  - keep invalid integration distinct from dependency unavailability and policy denial
  - prefer named regression cases over prose-only acceptance
- **Acceptance criteria**:
  - the taxonomy is visible in contract/docs/test surfaces
  - absent-state behavior and exit taxonomy do not contradict each other
- **Test notes**:
  - add or update gateway lifecycle/status regression coverage to protect the four non-success branches
- **Risk/rollback notes**:
  - do not collapse multiple failure modes into one generic “gateway unavailable” bucket

Checklist:
- Implement: exit-taxonomy wording and status behavior
- Test: exit-boundary regression coverage
- Validate: compare outputs to `C-01`
- Cleanup: remove conflicting failure wording from touched surfaces
