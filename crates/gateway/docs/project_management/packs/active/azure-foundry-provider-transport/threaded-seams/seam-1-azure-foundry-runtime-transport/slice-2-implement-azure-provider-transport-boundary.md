---
slice_id: S2
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - the implementation needs Azure transport behavior that was not frozen into `C-07`
    - Azure-specific code changes generic OpenAI-compatible behavior instead of staying inside the planned boundary
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
### S2 - Implement The Azure Provider Transport Boundary

- **User/system value**: the gateway gains a concrete Azure transport path that honors the frozen `C-07` contract instead of treating Azure as a generic OpenAI-compatible provider.
- **Scope (in/out)**:
  - In: implement the provider-mode or helper boundary for Azure auth resolution, deployment-scoped target construction, `api-version` propagation, and model/deployment mapping in runtime/provider code.
  - Out: config examples and transport tests as primary deliverables, live smoke execution, closeout accounting, and unrelated provider cleanup beyond what the Azure boundary requires.
- **Acceptance criteria**:
  - Azure request construction no longer depends on unconditional bearer auth plus `{base_url}/chat/completions` appends in the generic path.
  - provider and registry code can express the Azure-specific fields or mode required by `C-07` without reopening public or planner contracts.
  - the implementation keeps Azure-specific branching behind an explicit provider-mode or helper boundary so non-Azure OpenAI-compatible providers retain their existing behavior.
  - request-body construction for think/default traffic still honors the landed routing and public-surface basis.
- **Dependencies**: `S1`, `../../threading.md`, `gateway/src/providers/openai.rs`, `gateway/src/providers/registry.rs`, `gateway/src/providers/mod.rs`, `gateway/src/cli/mod.rs`, and the landed upstream basis captured in `seam.md`
- **Verification**:
  - the code path for Azure requests is identifiable and bounded in provider/runtime code
  - pass condition: reviewers can trace config/model input to Azure request target, auth headers, query string, and body construction without encountering ambiguous generic fallthrough
  - failure conditions are explicit: Azure depends on ad hoc custom headers alone, routing/mapping is still implicit, or non-Azure providers now require Azure-specific assumptions
- **Rollout/safety**: preserve one logical backend identity and keep deployment-boundary assumptions replaceable; do not hard-code localhost-only or host-credential-only architecture into the transport path.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`) and `review.md` (`R1`, `R2`, `Likely mismatch hotspots`)

#### S2.T1 - Add Azure Transport Configuration And Provider-Mode Plumbing

- **Outcome**: provider configuration can express the fields or mode needed to distinguish Azure transport from the generic OpenAI path.
- **Inputs/outputs**: inputs are `C-07`, current `ProviderConfig`, registry loading, and CLI/config parsing; output is bounded config/runtime plumbing in `gateway/src/providers/mod.rs`, `gateway/src/providers/registry.rs`, and `gateway/src/cli/mod.rs`.
- **Thread/contract refs**: `THR-06`, `C-07`
- **Implementation notes**: prefer a narrow Azure-specific mode or helper boundary over a generic-header hack; the routing-facing model names stay `Kimi-K2-Thinking` and `Kimi-K2.5`, while Azure deployment details stay inside provider/config surfaces.

#### S2.T2 - Implement Auth, Target, And Query Construction

- **Outcome**: Azure requests use the contract-defined auth posture, deployment-scoped URL, and `api-version` handling.
- **Inputs/outputs**: inputs are `C-07` and the current OpenAI request builder; output is Azure-specific request construction in `gateway/src/providers/openai.rs` or a seam-local helper reachable from that provider.
- **Thread/contract refs**: `THR-06`, `C-07`
- **Implementation notes**: make the Azure path explicit and testable; do not leave auth or target rules split across generic base URL string concatenation and free-form custom headers.

#### S2.T3 - Wire Model-To-Deployment Mapping Without Reopening Upstream Contracts

- **Outcome**: think/default traffic can resolve to the intended Azure deployments while preserving the landed routing and public-surface contracts above the provider seam.
- **Inputs/outputs**: inputs are the landed routing basis, config plumbing, and Azure transport builder; output is a concrete mapping path from external model names to Azure deployment identifiers and actual model names.
- **Thread/contract refs**: `THR-06`, `C-07`, `C-03`, `C-04`
- **Implementation notes**: keep mapping rules configuration-driven and internal; do not create new public backend identities or let Azure deployment names leak above the provider boundary.
