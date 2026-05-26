---
slice_id: S2
seam_id: SEAM-1
slice_kind: documentation
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - the adapter-visible `status --json` owner line changes
    - ADR-0041 authority paths drift again after cleanup
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
  - C-02
contracts_consumed: []
open_remediations:
  - REM-005
---
### S2 - Guard the status owner line and repair ADR authority drift

#### Goal

Keep the now-recorded v1 status owner line aligned while repairing the stale ADR path references that still point downstream work at the wrong authority set.

#### Dependencies

- `S00` defines the contract bundle and target owner surfaces
- external authorities:
  - `docs/contracts/gateway/status-schema.md`
  - `docs/contracts/gateway/operator-contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`

#### S2.T1 - Guard the additive adapter-visible status owner line

- **Outcome**:
  - the seam-owned docs identify one explicit owner for additive adapter-visible `status --json` fields and state the v1 boundary: no additive adapter-visible field family is currently published beyond the existing schema.
- **Files**:
  - `../../contract.md`
  - `../../policy-spec.md`
  - `docs/contracts/gateway/status-schema.md`
  - `docs/contracts/gateway/operator-contract.md`
- **Thread/contract refs**:
  - `THR-01`
  - `C-02`
- **Acceptance criteria**:
  - `REM-001` can be resolved with concrete owner-line evidence
  - the current field-family boundary is explicit even though no additive adapter-visible family ships in v1
  - no existing status-schema family is silently repurposed
- **Test notes**:
  - compare the final boundary against `../../pre-planning/workstream_triage.md`

Checklist:
- Implement:
  - write the owner line and bounded field-family statement
  - document which status surfaces stay external
- Test:
  - review for overlap with existing status-schema and operator-contract ownership
- Validate:
  - confirm `SEAM-2` can consume the result without inventing a second schema owner

#### S2.T2 - Repair ADR-0041 authority-path drift

- **Outcome**:
  - ADR-0041 points at the implemented config-policy pack paths used in this checkout, and the seam-local docs cite the corrected authority set consistently.
- **Files**:
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
  - `../../contract.md`
  - `../../policy-spec.md`
- **Thread/contract refs**:
  - none
- **Acceptance criteria**:
  - `packs/active/llm_and_agent_config_policy_surface/*` references are replaced with the live `packs/implemented/...` paths
  - no seam-local doc cites the stale path after the cleanup
  - `REM-005` has explicit resolution evidence or an evidence-only defer decision
- **Test notes**:
  - repo-wide search for the stale ADR-0041 path references

Checklist:
- Implement:
  - update ADR-0041 path references
  - align the seam-owned docs to the same authority set
- Test:
  - run a targeted search for lingering stale `packs/active/llm_and_agent_config_policy_surface` references
- Validate:
  - confirm downstream planning will cite the implemented pack, not the retired path
