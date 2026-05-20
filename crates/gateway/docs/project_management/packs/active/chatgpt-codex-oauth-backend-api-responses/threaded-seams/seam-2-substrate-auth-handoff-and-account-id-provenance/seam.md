---
seam_id: SEAM-2
seam_slug: substrate-auth-handoff-and-account-id-provenance
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-2-substrate-auth-handoff-and-account-id-provenance.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
  required_threads:
    - THR-14
    - THR-15
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
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S99
  status: passed
open_remediations: []
---
# SEAM-2 - Substrate Auth Handoff And Account-Id Provenance

## Seam Brief (Restated)

- **Goal / value**: freeze the integrated-versus-standalone auth owner line so `ChatGPT-Account-ID` and access-token delivery are explicit, policy-compatible, and reviewable instead of being inferred from provider-local token parsing.
- **Type**: `integration`
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
- **Touch surface**:
  - `crates/gateway/src/auth/oauth.rs`
  - `crates/gateway/src/auth/token_store.rs`
  - `crates/gateway/src/server/oauth_handlers.rs`
  - `crates/gateway/src/providers/openai.rs`
  - `crates/gateway/src/server/mod.rs`
  - `crates/gateway/docs/OAUTH_SETUP.md`
  - `crates/gateway/docs/OAUTH_TESTING.md`
  - `crates/gateway/docs/contracts/chatgpt-codex-auth-handoff-contract.md`
- **Verification**:
  - For owned contracts, describe what must be concrete in seam-local planning before execution.
  - Reserve accepted or published contract artifact evidence for seam exit and closeout.
  - The canonical contract text for an owned contract must live in `crates/gateway/docs/contracts/chatgpt-codex-auth-handoff-contract.md`; seam-local planning may reference that path, but may not treat planning-pack docs as canonical.
  - Verify the owner line, field precedence, delivery posture, and failure envelope without depending on gateway-local auth-file reads inside the in-world runtime.
- **Basis posture**:
  - **Currentness**: `current` (revalidated against `THR-14` publication and the landed seam-1 closeout-backed route contract)
  - **Upstream closeouts assumed**: `../../governance/seam-1-closeout.md`
  - **Required threads**: `THR-14`
  - **Stale triggers**:
    - the route contract changes the required header contract or the exact auth-context fields the provider path consumes
    - Substrate delivery posture changes for auth bundles, secret-channel transport, or in-world consumption
    - standalone compatibility sources differ from the current ADR assumptions about `~/.codex/auth.json` and JWT fallback
- **Threading constraints**
  - **Upstream blockers**: none at pre-exec; `THR-14` is published by `SEAM-1` and has now been revalidated against the landed route contract and current provider/auth evidence anchors
  - **Downstream blocked seams**: none in this pack; `THR-15` is now published and consumed by the landed `SEAM-3` closeout-backed conformance baseline
  - **Contracts produced**: `C-15`
  - **Contracts consumed**: `C-14`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99`
- **Why this seam needs an explicit exit gate**: downstream work must consume a closeout-backed auth owner line, explicit fallback posture, and a canonical contract record rather than inferring ownership from provider code.
- **Expected contracts to publish**: `C-15`
- **Expected threads to publish / advance**: `THR-15`
- **Likely downstream stale triggers**: changes to the owner line, field precedence, fallback rules, delivery posture, or provider injection contract
- **Expected closeout evidence**: the canonical auth-handoff contract note, resolved owner-line verification, provider injection proof, and a final `THR-15` publication decision

## Slice index

- `S00` -> `slice-00-freeze-auth-handoff-contract.md`
- `S1` -> `slice-1-resolve-auth-context-and-owner-line.md`
- `S2` -> `slice-2-inject-account-id-and-bounded-fallback.md`
- `S3` -> `slice-3-verify-auth-source-and-failure-posture.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-2-closeout.md`
