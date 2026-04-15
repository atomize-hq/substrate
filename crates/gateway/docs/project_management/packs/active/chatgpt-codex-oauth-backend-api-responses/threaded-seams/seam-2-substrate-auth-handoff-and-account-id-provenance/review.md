---
seam_id: SEAM-2
review_phase: pre_exec
execution_horizon: active
basis_ref: seam.md#basis
---
# Review Bundle - SEAM-2 Substrate Auth Handoff And Account-Id Provenance

This artifact feeds `gates.pre_exec.review`.
`../../review_surfaces.md` is pack orientation only.

## Falsification questions

- Can integrated mode still claim to be Substrate-owned if the in-world gateway runtime reads host-local auth files to recover account identity?
- Does the plan still preserve explicit `account_id` precedence over JWT fallback, or would the provider path quietly redefine ownership by parsing whatever token shape happens to exist?
- Does the failure posture still stop before the upstream call when no valid account identity exists, or does the seam risk deferring the error until provider-side behavior fails later?

## R1 - Auth ownership flow that must land

```mermaid
flowchart LR
  HOST["Host-side Substrate credential preflight"] --> BUNDLE["Secret-channel auth bundle"]
  BUNDLE --> WORLD["In-world gateway runtime"]
  WORLD --> AUTH["Resolved auth context"]
  AUTH --> PROVIDER["Provider request builder"]
  PROVIDER --> HEADER["ChatGPT-Account-ID injection"]
  HEADER --> UP["backend-api/codex/responses"]
```

## R2 - Ownership and fallback boundary that must remain true

```mermaid
flowchart LR
  AUTH["Resolved auth context"] --> OWNER["Explicit account_id"]
  OWNER --> PROVIDER["Provider request builder"]
  AUTH --> JWT["JWT-derived fallback"]
  JWT --> PROVIDER
  PROVIDER --> FAIL["Fail before upstream call when identity is unresolved"]
```

## Likely mismatch hotspots

- The current token store schema still lacks a first-class `account_id` field, which makes it easy for fallback behavior to masquerade as integrated ownership.
- The provider path already extracts account identity in `openai.rs`, but this seam must keep that as a consumer action rather than a trust-boundary decision.
- The canonical auth-handoff contract doc is not written yet, so contract readiness remains blocked until the owner line and fallback rules are made concrete in `docs/contracts/`.

## Pre-exec findings

- `THR-14` is already published, so the route basis for `ChatGPT-Account-ID` injection is now current.
- The seam now has enough input detail to plan integration and verification, but the owned canonical contract artifact is still missing from `docs/contracts/chatgpt-codex-auth-handoff-contract.md`.
- No blocking remediation existed before this promotion run, so `REM-001` is being opened to track the missing canonical contract baseline.

## Pre-exec gate disposition

- **Review gate**: `passed`
- **Contract gate**: `failed`
  - the owned auth-handoff contract baseline is not yet written into the canonical contract path
- **Revalidation gate**: `passed`
- **Opened remediations**: `REM-001`

## Planned seam-exit gate focus

- **What must be true before downstream promotion is legal**: the canonical auth-handoff contract must exist, integrated mode must consume Substrate-delivered auth context first, JWT fallback must remain bounded, and the provider path must inject `ChatGPT-Account-ID` from resolved context without reintroducing host-local auth ownership.
- **Which outbound contracts/threads matter most**: `C-15`, `THR-15`
- **Which review-surface deltas would force downstream revalidation**: changes to owner-line precedence, fallback behavior, auth-field identifiers, delivery posture, or provider injection order
