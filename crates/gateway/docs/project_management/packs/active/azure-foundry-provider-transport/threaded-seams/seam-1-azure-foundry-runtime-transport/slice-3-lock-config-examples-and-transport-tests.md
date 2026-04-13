---
slice_id: S3
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - config or README examples diverge from the landed `C-07` contract
    - deterministic tests fail to cover one of the planned Azure Kimi deployments or the non-regression boundary
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-06
contracts_produced:
  - C-07
contracts_consumed:
  - C-03
  - C-04
  - C-05
  - C-06
open_remediations: []
candidate_subslices: []
---
### S3 - Lock Config Examples And Transport Tests

- **User/system value**: the seam publishes transport truth in a form operators and downstream work can trust, without waiting for live Azure credentials to prove every request-construction detail.
- **Scope (in/out)**:
  - In: align example config and README surfaces with `C-07`, add deterministic transport tests for headers/URL/query/body construction, and prove non-regression boundaries for other OpenAI-compatible providers.
  - Out: real credential smoke tests, redacted operator evidence, or troubleshooting matrices that belong to `SEAM-2`.
- **Acceptance criteria**:
  - example config surfaces can express both intended Azure Kimi deployments and the required Azure transport fields without hidden assumptions.
  - deterministic tests prove Azure auth headers, deployment URL, `api-version`, request body, and model/deployment resolution for think and default routes.
  - tests or assertions show non-Azure OpenAI-compatible providers keep their existing generic behavior.
  - README or adjacent operator-facing notes describe the transport contract surfaces clearly enough that `SEAM-2` can consume them as basis for live smoke.
- **Dependencies**: `S1`, `S2`, `gateway/config/default.example.toml`, `gateway/config/models.example.toml`, `gateway/README.md`, and provider/runtime test anchors under `gateway/src/providers/` or `gateway/tests/`
- **Verification**:
  - pass condition: a reviewer can reconcile example config, runtime code, and deterministic tests without finding any contract drift
  - the Azure request-construction assertions do not require real credentials or a live Azure endpoint
  - failure conditions are explicit: examples depend on undocumented fields, tests only cover one Kimi deployment, or non-Azure providers now require Azure-specific config
- **Rollout/safety**: examples should remain deployment-boundary compatible and redact secrets; do not imply localhost-only topology or live-credential requirements as part of the transport contract itself.
- **Review surface refs**: `../../review_surfaces.md` (`R2`, `R3`) and `review.md` (`R2`, `Likely mismatch hotspots`)

#### S3.T1 - Align Config And README Contract Surfaces

- **Outcome**: example config and operator-facing notes match the frozen `C-07` contract exactly enough to serve as downstream basis.
- **Inputs/outputs**: inputs are `C-07`, runtime/config code from `S2`, and current example surfaces; output is aligned Azure examples in `gateway/config/default.example.toml`, `gateway/config/models.example.toml`, and `gateway/README.md`.
- **Thread/contract refs**: `THR-06`, `C-07`
- **Implementation notes**: examples should show how the think/default model names route to Azure deployments without leaking planner/executor as public provider identities or treating ad hoc custom headers as the primary contract.

#### S3.T2 - Add Deterministic Azure Transport Coverage

- **Outcome**: tests prove the Azure transport builder emits the correct URL, headers, query string, and request body for both intended Kimi deployments.
- **Inputs/outputs**: inputs are `C-07`, runtime code from `S2`, and current provider test anchors; output is deterministic unit or integration coverage under `gateway/src/providers/` tests or `gateway/tests/`.
- **Thread/contract refs**: `THR-06`, `C-07`
- **Implementation notes**: keep the tests request-construction focused and hermetic; they should assert transport semantics without calling live Azure.

#### S3.T3 - Prove The Non-Regression Boundary

- **Outcome**: Azure-specific work ships with drift guards showing that other OpenAI-compatible providers did not inherit Azure-only behavior.
- **Inputs/outputs**: inputs are the Azure transport implementation and existing generic provider behavior; output is regression coverage or explicit assertions proving generic providers keep bearer auth plus generic target rules unless intentionally configured otherwise.
- **Thread/contract refs**: `THR-06`, `C-07`
- **Implementation notes**: do not bury this in prose alone; use tests or narrowly scoped assertions so the isolation boundary remains enforceable after the seam lands.
