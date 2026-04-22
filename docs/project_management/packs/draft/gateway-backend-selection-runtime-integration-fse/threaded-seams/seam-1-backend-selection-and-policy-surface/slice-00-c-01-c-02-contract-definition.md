---
slice_id: S00
seam_id: SEAM-1
slice_kind: contract_definition
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - canonical `C-01` or `C-02` rules change after the baseline check
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
contracts_produced: []
contracts_consumed:
  - C-01
  - C-02
open_remediations:
  - REM-001
  - REM-002
---
### S00 - Confirm the published `C-01` / `C-02` baseline and lock the implementation boundary

- **User/system value**:
  - Starts the seam from already-published contract truth so implementation can begin immediately instead of reopening contract drafting.
- **Scope (in/out)**:
  - In: confirm that `C-01` / `C-02` already cover the shell-owned rules, map those rules to exact code/test surfaces, and record any residual alignment as non-blocking follow-through
  - Out: new canonical contract publication, runtime protocol details, runtime artifact semantics, or parity proof
- **Acceptance criteria**:
  - the slice records that `C-01` / `C-02` are sufficient for SEAM-1 and that the remaining work is shell implementation plus evidence
  - `validate_gateway_lifecycle_config`, `build_gateway_request`, `resolve_integrated_auth_payload`, and `resolve_cli_codex_integrated_auth` are named as the primary code surfaces
  - the slice names exact shell tests to preserve and the new tests required to prove adoption
  - supporting ADR-0046 docs, if later created, are treated as implementation notes that defer to canonical `docs/contracts/` ownership; their current absence is a seam-local documentation inconsistency, not a contract-truth blocker
- **Dependencies**:
  - none inbound; this is the first producer seam in the pack
- **Verification**:
  - compare the seam plan against `docs/contracts/substrate-gateway-backend-adapter-selection.md`, `docs/contracts/substrate-gateway-policy-evaluation.md`, `crates/shell/src/builtins/world_gateway.rs`, and `crates/shell/tests/world_gateway.rs`
  - treat the missing non-fse ADR-0046 support-doc files as future subordinate material, not as current evidence required to validate the baseline
- **Rollout/safety**:
  - preserves fail-closed behavior by preventing the seam from broadening scope into runtime ownership
- **Review surface refs**:
  - `../review.md`
  - `../../review_surfaces.md`

#### S00.T1 - Confirm the `C-01` shell-owned baseline and identify executable gaps

- **Outcome**:
  - The seam records that `C-01` already fixes selection order, inventory roots, and filename/id invariants, and that SEAM-1 now only needs shell-side adoption and tests.
- **Inputs/outputs**:
  - Inputs: `../../threading.md`, `docs/contracts/substrate-gateway-backend-adapter-selection.md`, `crates/shell/src/builtins/world_gateway.rs`
  - Outputs: a narrowed SEAM-1 implementation plan and explicit code/test ownership
- **Thread/contract refs**:
  - `THR-01`
  - `C-01`
- **Implementation notes**:
  - `validate_gateway_lifecycle_config` currently only checks enabled/mode/non-empty backend and must become the shell’s starting gate, not the whole `C-01` story
  - `build_gateway_request` is the handoff point where selection must already be legal before runtime dispatch
  - keep `llm.routing.default_backend` and `llm.allowed_backends` as the only selection inputs
- **Acceptance criteria**:
  - the slice no longer claims fresh contract publication is needed to begin implementation
  - SEAM-1 names explicit shell tests for empty backend, unsupported backend, allowlist denial, and inventory mismatch handling
  - the slice makes clear that the shell code/test surfaces named above are the implementation boundary for S00 and that no shell edits are required by S00 itself
- **Test notes**:
  - preserve `world_gateway_empty_default_backend_uses_exit_code_2`
  - preserve `world_gateway_invalid_integration_uses_exit_code_2`
- **Risk/rollback notes**:
  - if this slice leaves inventory adoption vague, the runtime will keep surfacing shell-owned invalid-integration cases

Checklist:
- Implement:
  - record the exact shell code surfaces that must adopt `C-01`
- Test:
  - verify the selection path is fully validated before the world-agent call
- Validate:
  - confirm `SEAM-2` inherits an allowed backend id, not unresolved shell validation work

#### S00.T2 - Confirm the `C-02` shell-owned baseline and identify evidence gaps

- **Outcome**:
  - The seam records that `C-02` already fixes precedence and fail-closed posture, and that SEAM-1 now owns adoption plus evidence.
- **Inputs/outputs**:
  - Inputs: `../../threading.md`, `docs/contracts/substrate-gateway-policy-evaluation.md`, `crates/shell/src/builtins/world_gateway.rs`, `crates/shell/tests/world_gateway.rs`
  - Outputs: a narrowed SEAM-1 plan for precedence/fail-closed adoption and tests
- **Thread/contract refs**:
  - `THR-01`
  - `C-02`
- **Implementation notes**:
  - `resolve_integrated_auth_payload` and `resolve_cli_codex_integrated_auth` are the primary shell adoption surfaces
  - keep precedence attached to policy-allowed sources only
  - do not pull secret-carrier redesign into this seam
- **Acceptance criteria**:
  - the slice records that env-primary/file-fallback/no-mixed-source is already canonical and no longer blocks implementation
  - the slice names the remaining missing evidence: “both env and file present; env wins without merge”
  - the slice makes clear that any future ADR-0046 support docs remain subordinate to `docs/contracts/` and are not required to establish the baseline
- **Test notes**:
  - preserve `world_gateway_sync_builds_integrated_auth_payload_from_host_auth_file`
  - preserve `world_gateway_status_builds_integrated_auth_payload_from_allowed_env_override`
  - preserve `world_gateway_host_credential_policy_denials_use_exit_code_5`
  - preserve `world_gateway_incomplete_env_override_uses_exit_code_2`
- **Risk/rollback notes**:
  - if evidence stays incomplete, `REM-001` will linger as seam-exit follow-through even after code lands

- **Verification plan**:
  - Add `world_gateway_status_prefers_allowed_env_auth_over_host_auth_file` in `crates/shell/tests/world_gateway.rs`.
  - Add a shell test proving env auth blocked by `llm.secrets.env_allowed` denies without file fallback.
  - Keep carrier and runtime-side auth handling out of SEAM-1; those remain `SEAM-2` follow-through.

Checklist:
- Implement:
  - record the exact shell code surfaces that must adopt `C-02`
- Test:
  - verify shell precedence and fail-closed evidence is complete enough for seam exit
- Validate:
  - confirm `SEAM-2` inherits a stable shell auth boundary rather than an unresolved precedence question
