---
seam_id: SEAM-1
seam_slug: azure-foundry-runtime-transport
type: integration
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-2-closeout.md
    - crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-3-closeout.md
    - crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-4-closeout.md
    - crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-5-closeout.md
  required_threads:
    - THR-06
  stale_triggers:
    - live Azure runtime requirements for auth, deployment path, or `api-version` prove incompatible with the current transport assumptions captured in `gateway/src/providers/openai.rs`
    - the landed `C-03` or `C-04` contracts change the intended `Kimi-K2-Thinking` or `Kimi-K2.5` routing posture
    - non-Azure OpenAI-compatible providers would regress unless Azure transport logic is isolated behind an explicit provider-mode boundary
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
  planned_location: S4
  status: passed
open_remediations: []
---

# SEAM-1 - Azure Foundry Runtime Transport

- **Goal / value**: turn Azure into a real first-class runtime provider by freezing the concrete provider transport contract for auth handling, deployment URL resolution, `api-version`, model/deployment mapping, and request construction without reopening the landed normalized or public contracts.
- **Scope**
  - In:
    - Azure `api-key` versus bearer-auth behavior at the provider/runtime boundary
    - Azure deployment-scoped request-target construction
    - `api-version` support in runtime request construction
    - provider config schema or provider-mode expression needed to make Azure concrete inside the existing registry
    - operator-facing example config surfaces that must reflect the seam-owned transport contract
    - deterministic verification of URL, header, query, and request-body construction for `Kimi-K2-Thinking` and `Kimi-K2.5`
  - Out:
    - re-planning `C-02` normalization semantics
    - redesigning the landed `/v1/messages` surface
    - redesigning planner/executor policy
    - broad cleanup of unrelated OpenAI-compatible provider code
    - live credential verification beyond what is needed to hand off cleanly to `SEAM-2`
- **Primary interfaces**
  - Inputs:
    - landed `C-02`, `C-03`, `C-04`, `C-05`, and `C-06` basis from the upstream closeouts
    - current runtime anchors in `gateway/src/providers/openai.rs`, `gateway/src/providers/registry.rs`, `gateway/src/cli/mod.rs`, `gateway/config/default.example.toml`, `gateway/config/models.example.toml`, and `gateway/README.md`
    - Azure runtime requirements as they become concrete during seam-local review and implementation
  - Outputs:
    - `C-07` Azure Foundry runtime transport contract
    - concrete provider/runtime expression for Azure auth mode, deployment URL shape, `api-version`, and model/deployment mapping
    - deterministic transport verification surfaces that `SEAM-2` can trust as upstream basis
- **Key invariants / rules**:
  - Azure-specific request construction stays below the landed normalized-event and public-surface contracts
  - the seam must preserve one logical backend identity and must not surface planner/executor roles as public provider identities
  - the transport contract must not assume localhost-only deployment or host-only credential posture as architectural truth
  - non-Azure OpenAI-compatible providers must keep their existing behavior unless Azure-specific handling is intentionally scoped to them
  - `Kimi-K2-Thinking` and `Kimi-K2.5` remain internal routing targets expressed through config/model mapping, not new public backend identities
- **Dependencies**
  - Direct blockers:
    - none; the upstream basis is already landed and published
  - Transitive blockers:
    - Azure runtime details may force revalidation if they contradict the currently assumed provider hook
  - Direct consumers:
    - `SEAM-2`
  - Derived consumers:
    - future Azure deployment automation and support tooling outside this pack
- **Touch surface**:
  - `gateway/src/providers/openai.rs`
  - `gateway/src/providers/registry.rs`
  - `gateway/src/providers/mod.rs`
  - `gateway/src/cli/mod.rs`
  - `gateway/config/default.example.toml`
  - `gateway/config/models.example.toml`
  - `gateway/README.md`
  - transport-focused verification surfaces under `gateway/tests/` or equivalent provider test anchors
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - Verify Azure auth is expressed concretely enough that the gateway no longer assumes generic `Authorization: Bearer ...` behavior for Azure runtime calls.
  - Verify the Azure request target is deployment-scoped and `api-version` aware rather than relying on unconditional `{base_url}/chat/completions` appends.
  - Verify the config schema and examples can represent the two intended Azure Kimi deployments and route them through the landed internal policy contract.
  - Verify deterministic transport tests can prove URL, header, query, and request-body construction without requiring real credentials.
- **Risks / unknowns**:
  - Risk: Azure runtime requirements may differ from the generic assumptions in the current provider path.
  - De-risk plan: make the Azure transport contract explicit and give it request-construction tests instead of relying on base-url string concatenation.
  - Risk: Azure handling may accidentally become a broad OpenAI-provider refactor.
  - De-risk plan: keep Azure-specific logic in a clearly named provider mode or helper boundary with limited touch surface.
  - Risk: config examples may drift from the actual runtime transport contract.
  - De-risk plan: treat examples as seam-owned contract surfaces and validate them against the same named fields the runtime uses.
- **Rollout / safety**:
  - land Azure-specific transport behind an explicit provider boundary before asking operators to run live smoke
  - preserve the landed `/v1/messages` and normalized-event behavior while transport work changes beneath them
  - keep diagnostics redacted and capability-oriented so future in-world deployment remains plausible
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `active` because no live Azure verification is credible until the runtime transport contract is concrete
  - Which threads matter most: `THR-06`
  - What the first seam-local review should focus on: auth/header posture, deployment URL resolution, `api-version` handling, model/deployment mapping expression, and non-regression boundaries for other OpenAI-compatible providers
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-07`
  - Threads likely to advance: `THR-06`
  - Review-surface areas likely to shift after landing: `R1`, `R2`, `R3`
  - Downstream seams most likely to require revalidation: `SEAM-2`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
