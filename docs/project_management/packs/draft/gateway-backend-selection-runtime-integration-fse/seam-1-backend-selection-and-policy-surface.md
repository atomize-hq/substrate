---
seam_id: SEAM-1
seam_slug: backend-selection-and-policy-surface
type: integration
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - canonical `C-01` or `C-02` rules change outside this seam
    - shell selection or auth-resolution logic changes outside the planned slice order
    - failure-bucket wording drifts between shell docs, shell tests, and runtime markers
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

# SEAM-1 - Backend selection and policy surface

- **Goal / value**:
  - Realize the already-published backend-selection and policy-evaluation contracts at the shell boundary so downstream runtime work consumes executable behavior instead of Codex-only side effects.
  - Close the gap between published `C-01` / `C-02` truth and current shell behavior, tests, and seam-exit evidence.
- **Scope**
  - In:
    - shell-side selected-backend validation from existing config, policy, and inventory posture
    - deny-by-default backend allowlisting before runtime dispatch
    - shell-side adoption of published inventory roots, filename/id invariants, and failure buckets from `C-01`
    - shell-side adoption of published env-primary/file-fallback precedence and fail-closed policy rules from `C-02`
    - deterministic tests and drift guards for `crates/shell/src/builtins/world_gateway.rs`
    - minimum supporting ADR-0046 alignment needed so implementation docs defer to canonical `docs/contracts/` refs
  - Out:
    - new canonical contract publication for `C-01` or `C-02`
    - integrated adapter binding metadata and capability gates inside `crates/world-service/src/gateway_runtime.rs`
    - runtime config rendering, managed artifact naming, or process lifecycle ownership
    - tuple metadata, tuple-policy keys, status-schema widening, or secret-channel redesign
- **Primary interfaces**
  - Inputs:
    - ADR-0046 goals and non-goals
    - canonical `C-01` in `docs/contracts/substrate-gateway-backend-adapter-selection.md`
    - canonical `C-02` in `docs/contracts/substrate-gateway-policy-evaluation.md`
    - shell request construction and validation in `crates/shell/src/builtins/world_gateway.rs`
    - shell lifecycle tests in `crates/shell/tests/world_gateway.rs`
  - Outputs:
    - landed shell behavior that matches the published `C-01` / `C-02` rules
    - deterministic shell tests covering selection, precedence, and failure-bucket behavior
    - aligned supporting ADR-0046 docs that clearly defer to canonical `docs/contracts/` ownership
    - closeout-ready `THR-01` evidence for `SEAM-2` and `SEAM-3`
- **Key invariants / rules**:
  - backend ids remain stable `<kind>:<name>` selectors only
  - gateway-local config, admin mutation, and persistence are not trusted authorization inputs
  - selection must stay on existing ADR-0027 config/policy roots
  - this seam must not widen `status --json` or pull ADR-0042/0043 surfaces into scope
  - shell behavior must adopt published contract truth instead of treating current `cli:codex` branches as implicit authority
- **Dependencies**
  - Direct blockers:
    - none inside the pack
  - Transitive blockers:
    - canonical `C-01` / `C-02` remain authoritative and must not be shadowed by feature-local prose
  - Direct consumers:
    - `SEAM-2`
    - `SEAM-3`
  - Derived consumers:
    - shell gateway requests
    - downstream runtime realization
    - closeout and rollout artifacts
- **Touch surface**:
  - `crates/shell/src/builtins/world_gateway.rs`
  - `crates/shell/tests/world_gateway.rs`
  - `docs/contracts/substrate-gateway-backend-adapter-selection.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
  - future subordinate ADR-0046 support docs under `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/`, if created later
- **Verification**:
  - `C-01` and `C-02` are already published; this seam verifies shell adoption, not fresh contract publication.
  - Current pre-exec gate posture is:
    - `review: passed` because the seam-local review bundle still exposes falsifiable selected-backend and auth-boundary flows.
    - `contract: passed` because canonical `docs/contracts/` refs already publish the selection, inventory, precedence, and fail-closed rules this seam needs.
    - `revalidation: passed` because the current shell still preserves the main failure buckets and Codex auth precedence rules, even though generic backend realization remains unimplemented.
    - `status: landed` is now justified because the seam-exit gate passed, `THR-01` published, post-exec gates passed, and `REM-001` / `REM-002` were resolved in closeout.
  - Later seam-local verification should prove:
    - `validate_gateway_lifecycle_config` and `build_gateway_request` reject empty, malformed, unknown, or disallowed selected backends before runtime dispatch, with landed evidence captured by `world_gateway_missing_inventory_uses_exit_code_2_before_socket_dispatch`, `world_gateway_inventory_filename_id_mismatch_uses_exit_code_2`, and `world_gateway_allowlist_denial_uses_exit_code_5`
    - `resolve_integrated_auth_payload` and `resolve_cli_codex_integrated_auth` enforce env-primary/file-fallback/no-mixed-source auth precedence
    - `ensure_backend_allowed` and `ensure_env_name_allowed` preserve policy-denial behavior without weakening fail-closed posture, with landed evidence captured by `world_gateway_env_auth_blocked_by_policy_denies_without_file_fallback` in addition to the existing auth precedence tests
    - `crates/shell/tests/world_gateway.rs` proves the distinction between invalid integration, policy denial, component unavailable, and transient runtime failure where the shell owns that distinction
    - any later subordinate ADR-0046 support docs remain descriptive implementation notes and do not compete with canonical `docs/contracts/` ownership
- **Canonical contract refs**:
  - `docs/contracts/substrate-gateway-backend-adapter-selection.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
- **Risks / unknowns**:
  - Risk:
    - current shell behavior still special-cases `cli:codex` and does not yet realize generic inventory-backed backend validation
  - De-risk plan:
    - land shell-side selection checks and deterministic tests before handing off to runtime realization
  - Risk:
    - supporting ADR-0046 docs may continue reading like competing contract owners instead of implementation notes
  - De-risk plan:
    - align supporting docs behind canonical `docs/contracts/` refs during conformance slices
  - Risk:
    - shell and runtime may classify the same failure differently at the world boundary
  - De-risk plan:
    - keep shell-owned buckets explicit and hand runtime-owned availability/binding questions to `SEAM-2`
- **Rollout / safety**:
  - This seam landed safely by tightening shell-side validation and evidence without widening operator surface area.
  - Safety depends on failing closed before runtime launch and keeping gateway-local state out of authorization truth.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`
    - this seam is now `future` only because it has left the forward planning window; it remains landed and continues to supply the published `THR-01` handoff to downstream seams.
  - Which threads matter most
    - `THR-01`
  - What the first seam-local review should focus on
    - whether shell validation matches published selection order
    - whether auth precedence remains env-primary and fail-closed
    - whether new tests prove shell-owned failure buckets deterministically
    - whether supporting ADR-0046 docs are clearly non-canonical
- **Expected seam-exit concerns**:
  - Contracts likely to consume:
    - `C-01`
    - `C-02`
  - Threads likely to advance:
    - `THR-01`
  - Review-surface areas likely to shift after landing:
    - selected-backend flow
    - auth-source diagram
    - failure-taxonomy wording
  - Downstream seams most likely to require revalidation:
    - `SEAM-2`
    - `SEAM-3`
  - Seam exit should record landed shell behavior, test evidence, and any supporting ADR alignment used to verify adoption of `C-01` / `C-02`.
