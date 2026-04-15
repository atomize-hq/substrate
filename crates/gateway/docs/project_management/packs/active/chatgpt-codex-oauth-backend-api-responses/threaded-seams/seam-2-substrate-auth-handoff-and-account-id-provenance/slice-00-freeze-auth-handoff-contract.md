---
slice_id: S00
seam_id: SEAM-2
slice_kind: contract_definition
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - the route contract changes the minimal header contract, account-id input shape, or auth-context field expectations
    - Substrate delivery posture changes for auth bundles or in-world secret consumption
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-15
contracts_produced:
  - C-15
contracts_consumed:
  - C-14
open_remediations:
  - REM-001
---
### S00 - Freeze Auth Handoff Contract

- **User/system value**: implementation starts from one explicit auth owner line instead of rediscovering provenance rules inside provider code or auth helpers.
- **Scope (in/out)**:
  - In: freeze the canonical auth-handoff contract note, the integrated-mode owner line, the standalone fallback path, the required field identifiers, and the pre-upstream failure posture.
  - Out: final published seam-exit evidence, route contract changes, and generic OAuth UX redesign.
- **Acceptance criteria**:
  - `crates/gateway/docs/contracts/chatgpt-codex-auth-handoff-contract.md` exists and is descriptive-only
  - the contract names the integrated-mode owner line, the bounded standalone fallback, explicit `account_id` precedence, and the unresolved-identity failure envelope
  - the contract note is concrete enough that seam-local implementation does not need to guess where auth ownership begins or ends
  - the verification checklist names the exact code and test anchors that later prove the owner line without making gateway-local token persistence authoritative for integrated mode
- **Dependencies**: `../../threading.md`, `../../scope_brief.md`, `../../seam-2-substrate-auth-handoff-and-account-id-provenance.md`, `../../governance/seam-1-closeout.md`, `docs/IMPORTANT_SUBSTRATE_ALIGNMENT.md`
- **Verification**:
  - a reviewer can explain the owner line, fallback order, and failure posture without inspecting implementation diffs
  - pass condition: the contract is concrete enough that `SEAM-2` can be planned and implemented without ambiguity
- **Rollout/safety**: keep the contract narrow and explicit; do not hide auth ownership inside token parsing or gateway-local persistence.
- **Review surface refs**: `../../review_surfaces.md` (`R3`) and `review.md` (`Likely mismatch hotspots`)

#### Frozen canonical artifacts (this slice output)

- Owned auth-handoff contract: `crates/gateway/docs/contracts/chatgpt-codex-auth-handoff-contract.md`
- Route contract basis: `docs/contracts/chatgpt-codex-route-contract.md`
- Boundary guardrail: `docs/IMPORTANT_SUBSTRATE_ALIGNMENT.md`

#### Execution-grade freeze for the auth handoff

- **Integrated-mode owner line**:
  - Substrate owns host credential reads, auth-state validation, and host-to-world delivery for ChatGPT Codex auth material
  - the gateway consumes resolved auth context and does not become the owner of host credential reads or trust-boundary decisions
  - secret-bearing delivery remains an auth bundle or equivalent secret channel, with env vars limited to non-secret pointer semantics
- **Field identifiers and precedence**:
  - the canonical integrated-mode field identifiers are `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID` and `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN`
  - integrated mode resolves explicit `account_id` first, then JWT-derived fallback from the same delivered access token, then fails before upstream
  - standalone mode resolves explicit `account_id` from local Codex auth state first, then JWT-derived fallback from the same local OAuth access token, then fails before upstream
  - if explicit and JWT-derived account ids disagree, the explicit value wins
- **Failure posture**:
  - if no valid account identity is available, the gateway fails before the upstream call using the normal error envelope
  - host-local auth-file reads are not a required integrated-mode trust input
- **Provider consumption**:
  - the provider request builder injects `ChatGPT-Account-ID` from resolved auth context
  - the provider path does not become the owner of host credential reads
  - the current JWT-only helper in `crates/gateway/src/providers/openai.rs` is a bounded fallback implementation detail, not the owner-line contract

#### S00.T1 - Freeze The Owner Line And Field Precedence

- **Outcome**: the seam contract names one explicit auth owner line for integrated and standalone mode.
- **Inputs/outputs**: inputs are the route contract basis, ADR 0010 auth assumptions, and current gateway auth surfaces; outputs are the auth-handoff contract note and the owner-line decision record.
- **Thread/contract refs**: `THR-15`, `C-15`
- **Implementation notes**: keep integrated delivery tied to the Substrate-owned handoff fields and keep standalone local auth state subordinate to that owner line.
- **Acceptance criteria**: one reviewer can explain which source wins for `account_id`, which fallback is bounded, and which failures stop the request before the upstream call.
- **Test notes**: name positive and negative verification cases for explicit-over-JWT precedence, bounded fallback, and no-upstream-call failure behavior.

#### S00.T2 - Freeze The Failure Envelope And Verification Checklist

- **Outcome**: the seam contract names the failure posture and the seam-local verification anchors.
- **Inputs/outputs**: inputs are the resolved owner line and existing provider/auth code paths; outputs are the failure envelope and verification checklist.
- **Thread/contract refs**: `THR-15`, `C-15`
- **Acceptance criteria**: unresolved identity fails before upstream, and the verification checklist names the exact code and test anchors that will prove it.

Checklist:
- Implement: add a resolved auth-context carrier that surfaces explicit `account_id` plus access token for the selected mode without making gateway-local persistence authoritative for integrated operation
- Implement: update `crates/gateway/src/providers/openai.rs` so the Codex request builder prefers explicit `account_id`, uses JWT only as bounded fallback, and fails before upstream when identity is unresolved
- Test: add `codex_oauth_request_builder_prefers_explicit_account_id_over_jwt_fallback` in `crates/gateway/src/providers/openai.rs`
- Test: add `codex_oauth_request_builder_uses_jwt_fallback_only_when_explicit_account_id_is_absent` in `crates/gateway/src/providers/openai.rs`
- Test: add `codex_oauth_request_builder_fails_before_upstream_when_account_id_is_unresolvable` in `crates/gateway/src/providers/openai.rs`
- Validate: keep auth-failure classification aligned with `crates/gateway/src/server/mod.rs` so unresolved identity stays in the normal `auth` error envelope
- Validate: update `crates/gateway/docs/OAUTH_SETUP.md` and `crates/gateway/docs/OAUTH_TESTING.md` so standalone local OAuth/token material is documented as compatibility-only, not the integrated trust boundary
- Publish when landed: record the landed owner-line evidence in `../../governance/seam-2-closeout.md` and advance `THR-15` only after the canonical contract, provider wiring, and verification surfaces all land
