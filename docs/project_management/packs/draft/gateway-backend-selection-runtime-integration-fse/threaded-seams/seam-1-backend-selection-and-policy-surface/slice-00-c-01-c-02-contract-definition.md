---
slice_id: S00
seam_id: SEAM-1
slice_kind: contract_definition
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - canonical contract publication changes inventory roots, filename rules, or auth precedence outside the planned task order
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
contracts_produced:
  - C-01
  - C-02
contracts_consumed: []
open_remediations:
  - REM-001
  - REM-002
---
### S00 - Define the `C-01` and `C-02` contract baseline

- **User/system value**:
  - Make the producer seam concrete enough that runtime realization can consume published selection and policy truth instead of reverse-engineering the current `cli:codex` path.
- **Scope (in/out)**:
  - In: canonical contract language for selection order, inventory discoverability, filename/id invariants, trusted-input boundaries, auth precedence, and failure taxonomy
  - Out: adapter protocol details, runtime artifact semantics, or parity proof
- **Acceptance criteria**:
  - `docs/contracts/substrate-gateway-backend-adapter-selection.md` explicitly names backend inventory discoverability roots and filename/id invariants tightly enough to close `REM-002`
  - `docs/contracts/substrate-gateway-policy-evaluation.md` explicitly names one precedence rule between allowlisted env material and allowlisted host credential files tightly enough to close `REM-001`
  - supporting ADR-0046 docs reference and align to the canonical contract docs rather than acting as competing publication surfaces
  - the verification checklist names concrete code and test surfaces that must remain aligned during landing
- **Dependencies**:
  - none inbound; this is the first producer seam in the pack
- **Verification**:
  - compare contract text against `../../threading.md`, `../../seam-1-backend-selection-and-policy-surface.md`, `docs/contracts/substrate-gateway-backend-adapter-selection.md`, `docs/contracts/substrate-gateway-policy-evaluation.md`, and `crates/shell/src/builtins/world_gateway.rs`
- **Rollout/safety**:
  - preserves fail-closed behavior and keeps gateway-local state out of authorization truth
- **Review surface refs**:
  - `../review.md`
  - `../../review_surfaces.md`

For a `slice_kind: contract_definition` slice that produces an owned contract:

- make the contract rules concrete enough that the producer seam can later satisfy `gates.pre_exec.contract`
- include a narrow verification plan with test locations, edge cases, and pass/fail conditions
- do not require the final accepted contract artifact to exist before the producer seam can become `exec-ready`

#### S00.T1 - Freeze the `C-01` selection and inventory rules

- **Outcome**:
  - `C-01` names one ordered selection path, one explicit discoverability-root policy, and one filename/id consistency rule that downstream seams can consume without local invention.
- **Inputs/outputs**:
  - Inputs: `../../threading.md`, `../../seam-1-backend-selection-and-policy-surface.md`, `docs/contracts/substrate-gateway-backend-adapter-selection.md`
  - Outputs: updated canonical selection contract plus aligned supporting ADR-0046 contract/policy text
- **Thread/contract refs**:
  - `THR-01`
  - `C-01`
- **Implementation notes**:
  - keep the backend id as a stable adapter selector only
  - make discoverability roots explicit without widening tuple or status surfaces
  - preserve deny-by-default allowlisting before adapter dispatch
- **Acceptance criteria**:
  - malformed backend ids, unknown ids, inventory mismatches, and policy denial remain distinct outcomes
  - inventory roots and filename/id invariants are explicit enough that `SEAM-2` can rely on them
- **Test notes**:
  - review against shell gateway config/policy resolution and existing contract docs
- **Risk/rollback notes**:
  - a vague inventory rule will re-open `REM-002` and force downstream stale triggers immediately

Checklist:
- Implement:
  - publish the `C-01` contract details in the canonical selection doc
- Test:
  - verify the ordered selection path matches the shell gateway entrypoint
- Validate:
  - confirm `SEAM-2` can cite `C-01` without inheriting hidden inventory assumptions

#### S00.T2 - Freeze the `C-02` auth precedence and trusted-input rules

- **Outcome**:
  - `C-02` defines one explicit env-versus-file precedence rule and one trusted-input boundary for policy evaluation.
- **Inputs/outputs**:
  - Inputs: `../../threading.md`, `../../seam-1-backend-selection-and-policy-surface.md`, `docs/contracts/substrate-gateway-policy-evaluation.md`, `crates/shell/src/builtins/world_gateway.rs`
  - Outputs: updated canonical policy-evaluation contract plus aligned supporting ADR-0046 policy/env-var text
- **Thread/contract refs**:
  - `THR-01`
  - `C-02`
- **Implementation notes**:
  - keep policy explanations and fail-closed behavior Substrate-owned
  - preserve the boundary that gateway-local config, admin, and persistence are not trusted policy inputs
  - make the precedence rule explicit rather than leaving the current `cli:codex` branch as accidental truth
  - state explicitly that precedence chooses the handoff content, not the long-term host-to-world carrier; current env delivery may remain as v1 transport, while the preferred additive direction is a secret-channel payload plus inherited FD/pipe auth bundle
- **Acceptance criteria**:
  - complete allowlisted env auth material is primary, host credential file reads are fallback-only when env auth is absent, and mixed-source completion is forbidden
  - partial env auth remains invalid integration rather than silent file fallback
  - no-host-fallback when in-world execution is required stays explicit
  - trusted-input boundaries remain aligned across canonical and supporting docs
- **Test notes**:
  - compare contract text to the current shell env/file selection behavior
- **Risk/rollback notes**:
  - if precedence is left ambiguous, `REM-001` remains blocking and downstream runtime auth shaping cannot become deterministic

- **Verification plan**:
  - Preserve `world_gateway_sync_builds_integrated_auth_payload_from_host_auth_file`, `world_gateway_status_builds_integrated_auth_payload_from_allowed_env_override`, `world_gateway_incomplete_env_override_uses_exit_code_2`, and `world_gateway_host_credential_policy_denials_use_exit_code_5` in `crates/shell/tests/world_gateway.rs`.
  - Add `world_gateway_status_prefers_allowed_env_auth_over_host_auth_file` in `crates/shell/tests/world_gateway.rs` so both sources can exist on the host while env-token presence still suppresses file-derived substitution.
  - Preserve `integrated_source_requires_substrate_handoff` and `integrated_source_does_not_read_local_auth_files` in `crates/gateway/src/auth/codex_auth_context.rs` so integrated mode never reintroduces local auth-file fallback.
  - Cover edge cases: env token with optional account id, file-only auth state, account-id-only env, policy-denied host credential file read, and integrated runtime missing or incomplete handoff.
  - Pass/fail: env token present means the request carries env-derived auth only; env absent means allowed file fallback can populate the handoff; partial env fails as invalid integration; blocked file reads fail as policy denial; integrated mode rejects missing handoff before any local auth-file read.

Checklist:
- Implement:
  - publish the `C-02` precedence and trusted-input rules in the canonical policy doc
- Test:
  - verify the rule is consistent with or intentionally supersedes the current shell behavior
- Validate:
  - confirm `SEAM-2` can consume `C-02` without reopening auth-source ownership questions
