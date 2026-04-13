---
seam_id: SEAM-2
seam_slug: azure-live-smoke-operator-readiness
type: conformance
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - docs/project_management/packs/active/azure-foundry-provider-transport/governance/seam-1-closeout.md
    - docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-3-closeout.md
    - docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-4-closeout.md
    - docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-5-closeout.md
  required_threads:
    - THR-06
    - THR-07
  stale_triggers:
    - `docs/foundation/azure-foundry-c07-runtime-transport-contract.md` changes Azure auth, base URL, deployment-selection, or request-body invariance in a way that invalidates the planned smoke path
    - the landed `/v1/messages` or internal routing behavior changes the practical smoke path for think versus default traffic
    - live Azure evidence reveals new operator-facing failure modes that the recorded troubleshooting surfaces do not explain
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S3
  status: pending
open_remediations: []
---

# SEAM-2 - Azure Live Smoke And Operator Readiness

- **Goal / value**: convert the seam-owned Azure transport contract into a real operator verification path so a user with real Azure credentials can configure the gateway, route think and default traffic correctly, and understand failures without reopening the landed public or normalization seams.
- **Scope**
  - In:
    - live smoke-test path against real Azure-hosted `Kimi-K2-Thinking` and `Kimi-K2.5` deployments
    - operator-facing verification steps for config, startup, request routing, and success signals through `/v1/messages`
    - redacted evidence expectations for live Azure verification
    - troubleshooting surfaces for auth mismatch, deployment URL mismatch, `api-version` mismatch, and model/deployment mapping mismatch
    - any minimal runtime or documentation diagnostics needed to make failures understandable to operators
  - Out:
    - redefining the Azure runtime transport contract itself
    - redesigning router policy or public API semantics
    - broad observability platform work beyond the operator verification surface needed for Azure setup
    - future automation beyond the bounded operator workflow this seam owns
- **Primary interfaces**
  - Inputs:
    - published `C-07` Azure transport contract from `SEAM-1`
    - landed `C-03` Anthropic Messages surface
    - landed `C-04` planner/executor internal policy contract
    - landed `C-05` one-backend deployment-boundary constraints
  - Outputs:
    - `C-08` operator verification contract
    - a redacted live smoke procedure tied to the real Azure Kimi path
    - troubleshooting and success-signal surfaces that future operators can reuse
- **Key invariants / rules**:
  - live proof must run through the real gateway path Claude Code consumes, not a provider-only bypass
  - operator surfaces must stay capability-oriented and must not leak internal backend identities as public contract
  - troubleshooting output must help distinguish auth, URL, `api-version`, and mapping failures without exposing secrets
  - host-local development may be used for the smoke path, but the documentation and verification posture must preserve the replaceable deployment boundary
  - this seam consumes `C-07`; it does not redefine it
- **Dependencies**
  - Direct blockers:
    - none; `SEAM-1` has published `C-07` and closed out `THR-06`
  - Transitive blockers:
    - any stale trigger on landed `C-03`, `C-04`, or `C-05` that changes the smoke path or boundary posture
  - Direct consumers:
    - future Azure operations and deployment work outside this pack
  - Derived consumers:
    - later support runbooks and deployment automation
- **Touch surface**:
  - `gateway/README.md`
  - `gateway/config/default.example.toml`
  - `gateway/config/models.example.toml`
  - `gateway/src/cli/mod.rs`
  - `gateway/src/server/mod.rs`
  - any bounded smoke harness, test procedure, or operator-facing verification notes introduced for Azure runtime validation
  - redacted diagnostics or troubleshooting anchors in provider/runtime code when needed to explain failures
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - Verify an operator can express Azure credentials, deployment identifiers, and the intended think/default model mappings without consulting code.
  - Verify live smoke steps use the landed `/v1/messages` path and demonstrate both `Kimi-K2-Thinking` and `Kimi-K2.5` routes.
  - Verify the troubleshooting surface can distinguish at least auth, deployment URL, `api-version`, and mapping failures with redacted evidence.
  - Verify the seam records enough live evidence that future operators do not need to rediscover the successful Azure setup.
- **Risks / unknowns**:
  - Risk: live Azure verification may depend on credentials or deployments unavailable in CI.
  - De-risk plan: define a redacted operator evidence contract and keep deterministic transport verification in `SEAM-1`.
  - Risk: runtime failures may still be too opaque for operators.
  - De-risk plan: make the troubleshooting taxonomy an explicit seam output rather than assuming raw logs are sufficient.
  - Risk: operator docs could imply a localhost-only architecture.
  - De-risk plan: keep the smoke path framed as one verification route while preserving the landed replaceable-boundary contract.
- **Rollout / safety**:
  - do not ask operators for live smoke until `SEAM-1` has frozen the Azure transport contract
  - keep live evidence redacted and bounded to the minimum needed to prove the route and diagnose failures
  - treat success criteria as gateway-backed operator readiness, not merely a passing provider call
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `active` because `SEAM-1` has now published `C-07` and the next safe work is the operator-readiness seam that consumes it
  - Which threads matter most: `THR-06`, `THR-07`
  - What the first seam-local review should focus on: redacted live verification flow, router/provider success signals for think and default routes, and whether the troubleshooting surface actually matches the seam-1 transport contract
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-08`
  - Threads likely to advance: `THR-07`
  - Review-surface areas likely to shift after landing: `R1`, `R3`
  - Downstream seams most likely to require revalidation: future Azure operations and deployment work outside this pack
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
