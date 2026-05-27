---
seam_id: SEAM-2
status: landed
closeout_version: v1
seam_exit_gate:
  source_ref: crates/gateway/docs/project_management/packs/active/chatgpt-codex-oauth-backend-api-responses/threaded-seams/seam-2-substrate-auth-handoff-and-account-id-provenance/slice-99-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - governance/seam-1-closeout.md
  required_threads:
    - THR-14
    - THR-15
  stale_triggers:
    - the auth-handoff owner line changes after S99 records closeout truth
    - Substrate delivery posture or secret-channel semantics change for the Codex auth bundle
    - the closed `cli:codex` field set or JWT fallback rule changes after the canonical auth-handoff contract is recorded
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-2 Substrate Auth Handoff And Account-Id Provenance

## Seam-exit gate record

- **Source artifact**: landed seam-exit slice at `threaded-seams/seam-2-substrate-auth-handoff-and-account-id-provenance/slice-99-seam-exit-gate.md`
- **Landed evidence**:
  - Canonical contract:
    - `crates/gateway/docs/contracts/chatgpt-codex-auth-handoff-contract.md` (`C-15`)
  - Auth-context resolution and owner-line tests:
    - `crates/gateway/src/auth/codex_auth_context.rs`
    - `integrated_mode_uses_substrate_account_id_first`
    - `integrated_mode_uses_jwt_fallback_when_explicit_account_id_is_absent`
    - `standalone_mode_uses_explicit_account_id_first`
    - `standalone_mode_uses_jwt_fallback_when_explicit_account_id_is_absent`
    - `auth_context_resolution_fails_when_account_id_is_unresolvable`
  - Provider injection proof:
    - `crates/gateway/src/providers/openai.rs`
    - `codex_oauth_request_builder_emits_only_the_minimal_header_contract`
  - Documentation evidence:
    - `crates/gateway/docs/OAUTH_SETUP.md`
    - `crates/gateway/docs/OAUTH_TESTING.md`
  - Closeout basis and thread registry:
    - `threading.md`
    - `review.md`
    - `governance/seam-2-closeout.md`
- **Contracts published or changed**: `C-15` is recorded as the canonical auth-handoff contract; no new contract text changed in S99
- **Threads published / advanced**: `THR-15` published
- **Review-surface delta**: `R3` now has closeout-backed evidence for integrated-mode auth ownership, bounded JWT fallback, and provider-side `ChatGPT-Account-ID` injection without expanding the provider trust boundary
- **Planned-vs-landed delta**: planned closeout truth required evidence that integrated mode consumes Substrate-delivered auth context first, fallback remains bounded, unresolved identity fails before upstream, and `THR-15` is published; landed evidence now confirms those assertions
- **Downstream stale triggers raised**:
  - `THR-15` or `C-15` changes after this closeout would require revalidation in `SEAM-3`
  - any later change to Substrate auth-bundle delivery, secret-channel semantics, or the `cli:codex` field set would stale this closeout
- **Remediation disposition**: `REM-001` is resolved by the landed S1/S2/S3 evidence plus this closeout record; no new remediation was opened by S99
- **Promotion blockers**: none
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: none
- **Carried-forward remediations**: none
