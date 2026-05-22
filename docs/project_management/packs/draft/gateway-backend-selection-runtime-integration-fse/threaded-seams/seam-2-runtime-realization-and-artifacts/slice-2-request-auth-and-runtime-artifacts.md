---
slice_id: S2
seam_id: SEAM-2
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - integrated auth or bounded request-schema expectations change after S1
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
  - THR-02
contracts_produced:
  - C-04
contracts_consumed:
  - C-02
  - C-03
  - C-04
open_remediations:
  - REM-004
---
### S2 - Land request/auth widening and runtime artifact semantics

- **User/system value**:
  - Replaces the current `cli_codex`-only request/auth and config/artifact path with a backend-aware runtime surface that still respects the existing auth precedence contract.
- **Scope (in/out)**:
  - In: shared request/auth payload shape, bounded auth validation, config render inputs, runtime manifest/log semantics, and artifact inspectability
  - Out: backend selection policy, capability-gate ordering already owned by S1, and parity/rollout proof
- **Acceptance criteria**:
  - shared lifecycle request types can carry the bounded integrated auth needed for supported integrated backends without redefining selection or policy ownership
  - runtime config render and managed artifacts use one explicit backend-aware shape and path contract
  - env-primary/file-fallback auth precedence remains consumed from canonical policy docs rather than redefined locally
- **Dependencies**:
  - `S1`
  - `THR-01`
  - `C-02`
  - `C-04`
- **Verification**:
  - targeted shell/world-service/shared-type tests around payload serialization, bounded auth validation, config render, and runtime manifest/log behavior
- **Rollout/safety**:
  - keep auth-carrier redesign out of scope; widen only the bounded request/artifact shapes required to land runtime behavior
- **Review surface refs**:
  - `../review.md`
  - `../../review_surfaces.md`

#### S2.T1 - Generalize bounded request and auth payloads

- **Outcome**:
  - request preparation supports more than `cli_codex` while preserving the policy-owned precedence boundary from `THR-01`.
- **Inputs/outputs**:
  - Inputs: `docs/contracts/substrate-gateway-backend-adapter-schema.md`, `docs/contracts/substrate-gateway-policy-evaluation.md`, `crates/transport-api-types/src/lib.rs`, `crates/shell/src/builtins/world_gateway.rs`, `crates/world-service/src/service.rs`
  - Outputs: widened shared types, backend-aware request construction, bounded auth validation, test coverage
- **Thread/contract refs**:
  - `THR-01`
  - `THR-02`
  - `C-02`
  - `C-04`
- **Implementation notes**:
  - keep selection and allowlist ownership in shell
  - keep auth precedence owned by canonical policy docs
  - widen only the runtime-owned handoff shape
- **Acceptance criteria**:
  - backend-aware request handling does not require backend-specific shell selection logic
  - incomplete or invalid request-provided auth still fails as runtime-owned invalid request or invalid integration in the bounded contract bucket
- **Test notes**:
  - add serialization and request-preparation tests for more than one integrated backend shape
- **Risk/rollback notes**:
  - ad hoc auth widening can turn backend-specific payload quirks into accidental contract truth

Checklist:
- Implement:
  - widen shared request/auth shapes and their preparation path
- Test:
  - cover serialization and bounded auth validation
- Validate:
  - confirm shell still passes a fixed selected backend id plus bounded auth handoff only

#### S2.T2 - Make config render and managed artifacts explicit runtime behavior

- **Outcome**:
  - config paths, manifests, and logs become backend-aware runtime artifacts instead of incidental side effects of the Codex-specific launch path.
- **Inputs/outputs**:
  - Inputs: runtime manager behavior, filesystem semantics spec, world-service tests
  - Outputs: explicit render/artifact implementation and coverage
- **Thread/contract refs**:
  - `THR-02`
  - `C-04`
- **Implementation notes**:
  - keep runtime artifact roots and permissions explicit
  - preserve operator inspectability without widening status output
  - treat manifest/log naming as one owned runtime-artifact surface
- **Acceptance criteria**:
  - runtime artifacts have stable roots, names, and permissions for supported integrated backends
  - inspectability stays available through existing operator diagnostics
- **Test notes**:
  - add tests for runtime config path, manifest persistence, and managed log behavior
- **Risk/rollback notes**:
  - implicit artifact semantics will make restart and parity work depend on hidden runtime state

Checklist:
- Implement:
  - make config render and runtime artifacts backend-aware and explicit
- Test:
  - capture manifest, config, and log behavior
- Validate:
  - confirm artifact semantics no longer depend on the Codex-only branch
