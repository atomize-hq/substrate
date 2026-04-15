---
seam_id: SEAM-2
seam_slug: substrate-auth-handoff-and-account-id-provenance
type: integration
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
  required_threads:
    - THR-14
  stale_triggers:
    - the route contract changes the required header contract or the exact auth-context fields the provider path consumes
    - Substrate delivery posture changes for auth bundles, secret-channel transport, or in-world consumption
    - standalone compatibility sources differ from the current ADR assumptions about `~/.codex/auth.json` and JWT fallback
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
  planned_location: S99
  status: pending
open_remediations:
  - REM-001
---

# SEAM-2 - Substrate Auth Handoff And Account-Id Provenance

- **Goal / value**: freeze the integrated-versus-standalone auth owner line so `ChatGPT-Account-ID` and access-token delivery are explicit, policy-compatible, and reviewable instead of being inferred from provider-local token parsing.
- **Scope**
  - In:
    - Substrate-owned integrated-mode auth delivery and account-id ownership
    - standalone compatibility posture and its bounded fallback order
    - the closed `cli:codex` auth field set and gateway-consumed field identifiers
    - gateway auth-context resolution and pre-upstream failure behavior when no valid account identity exists
    - provider request-builder injection of `ChatGPT-Account-ID` from resolved auth context
  - Out:
    - redesigning the OAuth browser flow or token-refresh UX
    - turning standalone local auth files into integrated-mode architecture
    - broad Substrate deployment planning beyond the gateway-side auth-handoff boundary this route needs
- **Primary interfaces**
  - Inputs:
    - `C-14` route contract and its minimal header contract
    - current gateway auth surfaces in `crates/gateway/src/auth/*` and `crates/gateway/src/server/oauth_handlers.rs`
    - current provider-side account-id extraction behavior in `crates/gateway/src/providers/openai.rs`
    - Substrate boundary constraints from `docs/IMPORTANT_SUBSTRATE_ALIGNMENT.md`, ADR 0005, and ADR 0006
  - Outputs:
    - `C-15` ChatGPT Codex auth-handoff contract
    - explicit integrated-mode and standalone-mode account-id resolution order
    - verification surfaces for host preflight, delivery, gateway consumption, and provider injection
- **Key invariants / rules**:
  - integrated mode consumes Substrate-delivered auth context and must not require direct reads of `~/.codex/auth.json` or other host-local auth files inside the in-world gateway runtime
  - explicit `account_id` wins over JWT-derived fallback whenever both exist
  - JWT extraction is compatibility fallback only and must not redefine auth ownership
  - the gateway consumes resolved auth context; it does not become the authority for host credential reads or trust-boundary decisions
  - if no valid account identity can be resolved for the selected mode, the gateway fails before the upstream call using the normal error envelope
- **Dependencies**
  - Direct blockers:
    - none at pre-exec; the owned auth-handoff contract baseline and execution checklist now exist, and the remaining open work is the landing-phase checklist tracked by `REM-001`
  - Transitive blockers:
    - any later conformance work depends on this seam freezing integrated-versus-standalone ownership and failure posture
  - Direct consumers:
    - `SEAM-3`
  - Derived consumers:
    - future Substrate-managed deployment and maintenance work outside this pack
- **Touch surface**:
  - `crates/gateway/src/auth/oauth.rs`
  - `crates/gateway/src/auth/token_store.rs`
  - `crates/gateway/src/server/oauth_handlers.rs`
  - `crates/gateway/src/providers/openai.rs`
  - `crates/gateway/docs/OAUTH_SETUP.md`
  - `crates/gateway/docs/OAUTH_TESTING.md`
  - auth-handoff contract docs reserved under `docs/contracts/`
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - Verify the auth-handoff contract is concrete enough to name the integrated-mode owner line, the standalone fallback path, the required field identifiers, and the exact pre-upstream failure posture.
  - Verify the gateway-side resolution path can distinguish integrated and standalone mode without letting gateway-local persistence become a required integrated trust input.
  - Verify the provider path can inject `ChatGPT-Account-ID` from resolved auth context first and use JWT parsing only as bounded fallback.
- **Canonical contract refs**:
  - `crates/gateway/docs/contracts/chatgpt-codex-auth-handoff-contract.md`
- **Risks / unknowns**:
  - Risk: the current token store schema lacks explicit `account_id`, making it easy for standalone compatibility to leak into integrated architecture.
  - De-risk plan: freeze the owner line and field set before seam-local implementation work starts.
  - Risk: auth-handoff work spreads into generic OAuth UX or unrelated provider cleanup.
  - De-risk plan: keep the seam scoped to account-id provenance, delivery, injection, and failure posture for the Codex route.
  - Risk: env vars become secret-bearing architecture instead of pointer semantics.
  - De-risk plan: keep any env-var usage limited to bundle-pointer semantics and document the real secret channel explicitly.
- **Rollout / safety**:
  - preserve standalone compatibility while making integrated mode authoritative
  - fail before the upstream call when account identity is unresolved or inconsistent
  - keep secret-bearing data out of public docs and non-secret pointer semantics only in process env
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `active` because the route contract is landed and the auth-handoff contract baseline is now concrete enough for execution-grade slices
  - Which threads matter most: `THR-14`, `THR-15`
  - What the first seam-local review should focus on: integrated-versus-standalone mode selection, field IDs, resolution precedence, provider injection ownership, and failure classification
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-15`
  - Threads likely to advance: `THR-15`
  - Review-surface areas likely to shift after landing: `R3` will tighten around concrete bundle shape and provider-consumption points; `R1` may annotate integrated-versus-standalone selection
  - Downstream seams most likely to require revalidation: `SEAM-3`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
