---
slice_id: S00
seam_id: SEAM-2
slice_kind: contract_definition
execution_horizon: active
status: decomposed
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
    contract: failed
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
  - `docs/contracts/chatgpt-codex-auth-handoff-contract.md` exists and is descriptive-only
  - the contract names the integrated-mode owner line, the bounded standalone fallback, explicit `account_id` precedence, and the unresolved-identity failure envelope
  - the contract note is concrete enough that seam-local implementation does not need to guess where auth ownership begins or ends
- **Dependencies**: `../../threading.md`, `../../scope_brief.md`, `../../seam-2-substrate-auth-handoff-and-account-id-provenance.md`, `../../governance/seam-1-closeout.md`, `docs/IMPORTANT_SUBSTRATE_ALIGNMENT.md`
- **Verification**:
  - a reviewer can explain the owner line, fallback order, and failure posture without inspecting implementation diffs
  - pass condition: the contract is concrete enough that `SEAM-2` can be planned and implemented without ambiguity
- **Rollout/safety**: keep the contract narrow and explicit; do not hide auth ownership inside token parsing or gateway-local persistence.
- **Review surface refs**: `../../review_surfaces.md` (`R3`) and `review.md` (`Likely mismatch hotspots`)

#### Frozen canonical artifacts (this slice output)

- Owned auth-handoff contract: `docs/contracts/chatgpt-codex-auth-handoff-contract.md`
- Route contract basis: `docs/contracts/chatgpt-codex-route-contract.md`
- Boundary guardrail: `docs/IMPORTANT_SUBSTRATE_ALIGNMENT.md`

#### Execution-grade freeze for the auth handoff

- **Owner line and precedence**:
  - integrated mode consumes Substrate-delivered auth context first
  - explicit `account_id` is the first resolved source
  - JWT-derived `account_id` is a bounded fallback only when explicit `account_id` is absent
- **Failure posture**:
  - if no valid account identity is available, the gateway fails before the upstream call using the normal error envelope
  - host-local auth-file reads are not a required integrated-mode trust input
- **Provider consumption**:
  - the provider request builder injects `ChatGPT-Account-ID` from resolved auth context
  - the provider path does not become the owner of host credential reads

#### S00.T1 - Freeze The Owner Line And Field Precedence

- **Outcome**: the seam contract names one explicit auth owner line for integrated and standalone mode.
- **Inputs/outputs**: inputs are the route contract basis, ADR 0010 auth assumptions, and current gateway auth surfaces; outputs are the auth-handoff contract note and the owner-line decision record.
- **Thread/contract refs**: `THR-15`, `C-15`
- **Acceptance criteria**: one reviewer can explain which source wins for `account_id`, which fallback is bounded, and which failures stop the request before the upstream call.

#### S00.T2 - Freeze The Failure Envelope And Verification Checklist

- **Outcome**: the seam contract names the failure posture and the seam-local verification anchors.
- **Inputs/outputs**: inputs are the resolved owner line and existing provider/auth code paths; outputs are the failure envelope and verification checklist.
- **Thread/contract refs**: `THR-15`, `C-15`
- **Acceptance criteria**: unresolved identity fails before upstream, and the verification checklist names the exact code and test anchors that will prove it.
