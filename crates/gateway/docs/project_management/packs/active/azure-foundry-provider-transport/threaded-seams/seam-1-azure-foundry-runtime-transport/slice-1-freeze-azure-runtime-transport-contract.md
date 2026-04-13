---
slice_id: S1
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - Azure runtime requirements prove the frozen auth, deployment URL, or `api-version` rules incomplete
    - the contract still leaves model-to-deployment mapping or provider-mode isolation ambiguous enough that later code would invent transport semantics
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
### S1 - Freeze The Azure Runtime Transport Contract

- **User/system value**: `SEAM-2` and later operators inherit one concrete Azure transport contract instead of rediscovering auth, deployment, and `api-version` rules from code or trial-and-error.
- **Scope (in/out)**:
  - In: define the owned `C-07` contract for Azure auth mode, deployment-scoped request target construction, `api-version`, routing-ready model/deployment mapping, provider-mode isolation, and a narrow verification checklist.
  - Out: implementation wiring, operator live smoke execution, post-exec closeout, public Anthropic surface changes, planner/executor policy changes, and broad non-Azure provider refactors.
- **Acceptance criteria**:
  - `docs/foundation/azure-foundry-c07-runtime-transport-contract.md` is the canonical source for `C-07`.
  - the contract names one canonical Azure auth/header rule set, one deployment URL composition rule set, one `api-version` rule, and one model/deployment mapping posture for `Kimi-K2-Thinking` and `Kimi-K2.5`.
  - the contract explicitly states what stays Azure-specific versus what must remain generic for other OpenAI-compatible providers.
  - the contract includes a verification checklist with pass/fail conditions for URL, header, query, request-body, and config-example alignment.
  - the contract keeps the landed `/v1/messages`, normalized-event, and planner/executor contracts above the provider seam unchanged.
- **Dependencies**: `../../threading.md`, ADR 0006, the upstream gateway closeouts listed in `seam.md`, `gateway/src/providers/openai.rs`, `gateway/src/providers/registry.rs`, `gateway/src/cli/mod.rs`, and the landed `C-03` through `C-06` basis documents
- **Verification**:
  - a reviewer can explain how Azure auth, deployment URL, `api-version`, and Kimi deployment mapping work by reading the `C-07` contract alone
  - pass condition: implementation work can proceed without guessing which fields belong in config, which headers belong on Azure requests, or how non-Azure providers stay isolated
  - edge cases are explicit: missing deployment identifier, mismatched `api-version`, incorrect auth mode, and misaligned model/deployment mapping
- **Rollout/safety**: keep Azure transport truth below the landed public surface; do not encode planner/executor roles as public provider identities or make host-local topology part of the contract.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`) and `review.md` (`R1`, `Likely mismatch hotspots`)

#### S1.T1 - Freeze Auth And Request-Target Rules

- **Outcome**: `C-07` explicitly names how Azure requests authenticate and how the deployment-scoped target plus `api-version` are constructed.
- **Inputs/outputs**: inputs are the seam brief, current `OpenAIProvider` behavior, and Azure runtime requirements; output is contract language in `docs/foundation/azure-foundry-c07-runtime-transport-contract.md` for auth headers, URL shape, query composition, and failure modes.
- **Thread/contract refs**: `THR-06`, `C-07`
- **Implementation notes**: do not leave auth behavior implied by examples or generic `headers`; the contract must say when bearer auth applies, when `api-key` style auth applies, and how Azure-specific target building differs from the generic `{base_url}/chat/completions` path.

#### S1.T2 - Freeze Mapping And Isolation Boundaries

- **Outcome**: the contract says how `Kimi-K2-Thinking` and `Kimi-K2.5` map onto Azure deployments and how Azure-specific logic stays behind a provider-mode boundary.
- **Inputs/outputs**: inputs are the landed routing basis from `C-03` and `C-04`, current registry/config structures, and ADR 0006; output is contract language for config fields, model/deployment mapping rules, and non-Azure isolation.
- **Thread/contract refs**: `THR-06`, `C-07`, `C-03`, `C-04`
- **Implementation notes**: the seam can introduce provider-mode or helper concepts, but it must not create new public backend identities or make Azure-specific branching the generic OpenAI default.

#### S1.T3 - Freeze The Verification Checklist

- **Outcome**: the producing seam has the narrow verification checklist required to pass later contract review and publish `THR-06`.
- **Inputs/outputs**: inputs are the frozen transport rules and known seam risks; output is a pass/fail checklist in the `C-07` artifact covering request construction, config-example alignment, and non-regression boundaries.
- **Thread/contract refs**: `THR-06`, `C-07`
- **Implementation notes**: keep the checklist pre-exec focused; live Azure proof belongs to `SEAM-2`, while deterministic request-construction proof belongs here.
