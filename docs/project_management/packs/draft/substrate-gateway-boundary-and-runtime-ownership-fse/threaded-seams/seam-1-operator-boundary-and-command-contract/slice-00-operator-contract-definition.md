---
slice_id: S00
seam_id: SEAM-1
slice_kind: contract_definition
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
contracts_produced:
  - C-01
contracts_consumed: []
open_remediations: []
---
### S00 - Operator contract definition

- **User/system value**: downstream seams and later implementation slices work from one explicit operator contract instead of archived gateway drafts or local CLI interpretation.
- **Scope (in/out)**:
  - In:
    - define the `sync|status|restart` command family
    - define `status --json` as the authoritative machine-readable wiring surface
    - define the stable non-secret env semantics, absent-state posture, exit taxonomy, and ownership split
    - define where the contract publishes in the feature pack, operator docs, tests, and durable contract refs
  - Out:
    - field-by-field `status --json` schema and policy decision tables
    - post-exec publication evidence and closeout accounting, which belong to `S99`
- **Acceptance criteria**:
  - `C-01` names the exact command family and rejects archived alternate command ordering.
  - `C-01` states that `status --json` is the authoritative machine-readable wiring surface and human-readable `status` may abbreviate but must not redefine it.
  - `C-01` states the stable semantics of `SUBSTRATE_LLM_OPENAI_BASE_URL` and `SUBSTRATE_LLM_ANTHROPIC_BASE_URL`.
  - `C-01` maps exit `0|2|3|4|5` to success, invalid config/integration, transient runtime failure, missing required gateway/world component, and policy/safety denial.
  - `C-01` names the Substrate-owned versus `substrate-gateway`-owned behavior split and the non-contract rule for gateway-local config/admin/persistence surfaces.
- **Dependencies**:
  - `threading.md` contract registry for `C-01` and `THR-01`
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/minimal_spec_draft.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/workstream_triage.md`
- **Verification**:
  - the contract rules below must map directly to the feature-local `contract.md`, CLI/builtin surfaces, regression tests, and operator docs named in later slices
- **Rollout/safety**: preserve fail-closed posture and keep gateway-internal behavior out of the Substrate operator contract.
- **Review surface refs**: `../../review_surfaces.md` R1

#### C-01 contract rules

1. **Command authority**: the only operator command family in scope is `substrate world gateway sync`, `substrate world gateway status`, and `substrate world gateway restart`.
2. **Machine-readable authority**: `substrate world gateway status --json` remains the authoritative Substrate-owned wiring surface; human-readable `status` output may abbreviate but must not redefine the machine-readable meaning.
3. **Stable non-secret env semantics**:
   - `SUBSTRATE_LLM_OPENAI_BASE_URL` and `SUBSTRATE_LLM_ANTHROPIC_BASE_URL` remain the only stable non-secret wiring env outputs in scope for this seam.
   - those values point to Substrate-managed gateway endpoints rather than upstream provider endpoints.
4. **Exit taxonomy**:
   - `0`: success
   - `2`: invalid configuration, invalid policy, or invalid integration state
   - `3`: transient runtime failure
   - `4`: required gateway or world component unavailable
   - `5`: policy or safety failure
5. **Ownership split**:
   - Substrate owns policy evaluation, world placement, lifecycle control, host-to-world secret delivery, operator UX, and canonical tracing.
   - `substrate-gateway` owns the in-world front door, provider/planner/executor internals, and normalized event generation.
   - gateway-local config, admin mutation, and token persistence are not required Substrate operator surfaces.
6. **Verification checklist**:
   - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md` publishes the operator contract without archived alternate command ordering.
   - `crates/shell/src/execution/cli.rs`, `crates/shell/src/builtins/mod.rs`, and `crates/shell/src/builtins/world_gateway.rs` enforce one command family and one status entrypoint.
   - `crates/shell/tests/world_gateway.rs` protects command spelling, absent-state behavior, and exit taxonomy.
   - `docs/USAGE.md` matches the contract wording without redefining the machine-readable surface.
   - the durable contract reference remains `docs/contracts/gateway/operator-contract.md`.

#### S00.T1 - Record the concrete operator contract for `C-01`

- **Outcome**: later implementation and docs slices can land against one explicit operator contract without re-reading ADR-0040 or archived gateway planning as source code.
- **Inputs/outputs**:
  - Inputs: ADR-0040 user contract, spec manifest, impact map, minimal spec draft, workstream triage
  - Outputs: locked command-family, status-authority, env-semantics, exit-taxonomy, and ownership rules for `C-01`
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**:
  - keep the contract tight to operator-facing surfaces only
  - reserve field-level JSON schema and fail-closed policy decision tables for `SEAM-2`
- **Acceptance criteria**:
  - no later slice needs to decide what the command family or exit taxonomy means
  - the contract explicitly rejects archived command ordering and gateway-local control-plane drift
- **Test notes**:
  - use ADR-0040 plus the pre-planning docs as the basis readback before writing code or docs
- **Risk/rollback notes**:
  - if a follow-on slice uncovers a missing operator rule, fix `C-01` here first rather than improvising in CLI or docs text

Checklist:
- Implement: N/A in this slice
- Test: N/A in this slice
- Validate: cross-check `C-01` against ADR-0040 and the pre-planning pack
- Cleanup: none

#### S00.T2 - Record the publication and verification surfaces for `C-01`

- **Outcome**: the producer seam can later prove the contract landed and publish `THR-01` without ambiguity.
- **Inputs/outputs**:
  - Inputs: `threading.md`, seam brief touch surface, impact-map create/edit surfaces
  - Outputs: explicit publication surfaces, named tests, and durable contract refs for the rest of the seam
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**:
  - treat `crates/shell/src/builtins/world_gateway.rs` and `crates/shell/tests/world_gateway.rs` as intentional create surfaces
  - keep durable contract refs descriptive-only under `docs/contracts/`
- **Acceptance criteria**:
  - each later slice has named files and readback checks tied to `C-01`
  - downstream seams can point at the same contract publication surfaces during promotion
- **Test notes**:
  - targeted CLI and operator-doc readbacks should be named now even if they land later
- **Risk/rollback notes**:
  - avoid spreading operator truth across ADRs, archived packs, and CLI help text without one owning publication path

Checklist:
- Implement: N/A in this slice
- Test: N/A in this slice
- Validate: confirm each publication surface belongs to `SEAM-1`
- Cleanup: none
