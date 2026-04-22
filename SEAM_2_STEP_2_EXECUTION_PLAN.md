# SEAM-2 Step 2 Execution Plan

Date: 2026-04-22
Scope: planning only
Primary input: [SEAM_2_BLOCKER_AUDIT.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/SEAM_2_BLOCKER_AUDIT.md)

## Objective

Unblock `SEAM-2` by turning the current Codex-specific runtime handoff into one bounded, backend-aware runtime handoff that can honestly publish `THR-02` and allow `SEAM-3` promotion.

This plan covers the implementation, validation, and governance work needed to get from the current blocked `SEAM-2` closeout state to:

- `THR-02` published
- `seam_exit_gate.status: passed`
- `promotion_readiness: ready`
- `SEAM-3` legally promotable

## Decision Summary

1. Boundary hardening alone is not enough.
   A generic request/auth boundary without one real supported non-`cli:codex` runtime path still leaves `THR-02` unpublished.

2. The shared auth shape should be backend-aware but not OpenAI-specific by default.
   The recommended primary shape is a closed, backend-neutral `api` env-auth facet driven by inventory-declared env names, while using `api:openai` as the first concrete proof target because the repo already has inventory and test fixtures for it.

3. `SEAM-2` does not need Linux/macOS/Windows rollout proof.
   Cross-platform rollout and later compatibility proof remain in `SEAM-3`. `SEAM-2` needs one publishable runtime handoff, not full rollout governance.

4. Governance must move with the code.
   `seam-2-closeout.md` and `remediation-log.md` currently contradict the real blocker posture. They must not be updated optimistically; they must be updated only after the code and tests justify publication.

## Non-Goals

- No `status --json` widening
- No tuple-surface widening
- No secret-carrier redesign beyond the bounded request/auth surface needed for runtime realization
- No new canonical contract-authoring phase unless implementation proves the existing canonical docs are insufficient
- No cross-platform rollout proof in `SEAM-2`

## Work Packages

## WP0 - Lock the Proof Target and Guardrails

**Goal**

Start with one explicit proof target and one explicit boundary model so later work does not drift.

**Decision**

- Use `api:openai` as the first non-Codex runtime proof target for `SEAM-2`.
- Keep the shared auth schema generic enough that this does not become an OpenAI-only contract.
- Preserve `cli:codex` as the regression floor.

**Files**

- [SEAM_2_BLOCKER_AUDIT.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/SEAM_2_BLOCKER_AUDIT.md)
- [seam.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threaded-seams/seam-2-runtime-realization-and-artifacts/seam.md)
- [slice-1-binding-lookup-and-capability-gates.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threaded-seams/seam-2-runtime-realization-and-artifacts/slice-1-binding-lookup-and-capability-gates.md)
- [slice-2-request-auth-and-runtime-artifacts.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threaded-seams/seam-2-runtime-realization-and-artifacts/slice-2-request-auth-and-runtime-artifacts.md)
- [slice-3-lifecycle-conformance-and-drift-guards.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threaded-seams/seam-2-runtime-realization-and-artifacts/slice-3-lifecycle-conformance-and-drift-guards.md)

**Deliverables**

- One explicit runtime proof target: `api:openai`
- One explicit schema posture:
  - keep `backend_id`
  - keep `cli_codex`
  - add one closed backend-neutral API env-auth facet driven by inventory
  - add `deny_unknown_fields` at the lifecycle request boundary

**Done When**

- The team is aligned that boundary hardening is necessary but not sufficient
- The team is aligned that publishing `THR-02` requires one real non-Codex supported backend path

## WP1 - Generalize Runtime Binding Lookup and Pre-Spawn Capability Gates

**Goal**

Remove the `cli:codex`-only runtime binding assumption without changing failure-bucket ownership.

**Files**

- [gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs)
- [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)

**Tasks**

1. Replace the singleton `match DEFAULT_BACKEND` binding resolution with a small runtime registry keyed by `backend_id`.
2. Keep `resolve_gateway_runtime_binding()` strictly keyed off `prepared.selected_backend`.
3. Preserve current failure buckets:
   - invalid request / invalid integration remain request-boundary failures
   - missing runtime binding remains `unavailable`
   - unsupported capability remains a pre-spawn runtime failure
4. Make capability-gate ordering explicit before port allocation, artifact creation, auth handoff, or spawn.

**Implementation Notes**

- Do not re-derive backend selection in runtime code.
- Do not add fallback to `DEFAULT_BACKEND`.
- Keep runtime ids, manifest paths, and restart continuity keyed by the selected backend.

**Validation**

- Unit tests in `gateway_runtime.rs` for:
  - successful lookup of the supported second backend
  - missing binding stays `None`
  - capability gating happens before runtime artifacts are created
- Service tests in `service.rs` for:
  - selected backend survives request prep and binding resolution
  - unbound backend stays `unavailable`, not `invalid_integration`

**Done When**

- Runtime binding is no longer structurally Codex-only
- Missing binding is still explicit and non-fallback
- Capability gates remain deterministic and pre-spawn

## WP2 - Harden the Shared Request/Auth Boundary

**Goal**

Turn the lifecycle request/auth surface into a bounded schema that can carry more than one backend path without becoming an open-ended auth bag.

**Files**

- [lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs)

**Recommended Shape**

- Keep `GatewayIntegratedAuthPayloadV1.backend_id`
- Keep `cli_codex`
- Add one closed API env-auth facet for inventory-declared env auth
- Add `#[serde(deny_unknown_fields)]` to `GatewayLifecycleRequestV1`

**Tasks**

1. Add top-level request hardening to `GatewayLifecycleRequestV1`.
2. Add shared validation helpers for `GatewayIntegratedAuthPayloadV1`:
   - `backend_id` must be non-empty
   - exactly one facet must be populated
   - the populated facet must match `backend_id`
   - required fields must be non-empty
   - unknown or extra facet data must fail closed
3. Call the shared validation helper from `prepare_gateway_runtime_request()`.
4. Keep backend-specific extraction checks in `gateway_runtime.rs` as defense in depth, not first-line validation.

**Key Decision**

Primary recommendation: use a backend-neutral API env-auth facet instead of an `api_openai` one-off.

Fallback recommendation: if implementation shows the generic API env-auth facet cannot represent the real supported path cleanly, introduce a closed `api_openai` facet only after that limitation is demonstrated in code.

**Validation**

- `cargo test -p agent-api-types -- --nocapture`
- `cargo test -p world-agent --lib -- --nocapture`

Add tests for:

- unknown top-level request fields
- multiple auth facets set
- backend/facet mismatch
- empty required auth value
- valid Codex payload
- valid supported non-Codex payload

**Done When**

- The request/auth boundary is explicitly closed
- Incomplete or contradictory request-provided auth fails before runtime execution
- The wire shape supports Codex and one additional runtime path without becoming unbounded

## WP3 - Generalize Shell Auth Emission Without Re-Opening SEAM-1

**Goal**

Make shell request construction backend-aware while keeping selection, allowlist, and precedence ownership exactly where `SEAM-1` put it.

**Files**

- [world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs)
- [agent_inventory.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs)
- [world_gateway.rs tests](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/world_gateway.rs)

**Tasks**

1. Refactor backend validation so `build_gateway_request()` receives the resolved inventory entry, not just a pass/fail side effect.
2. Change `resolve_integrated_auth_payload()` to dispatch on the selected backend entry instead of returning `None` for all non-Codex backends.
3. Preserve current Codex behavior exactly:
   - env-primary
   - file-fallback
   - policy-gated sourcing
4. For API backends:
   - read only inventory-declared env names
   - enforce `llm.secrets.env_allowed`
   - emit bounded API env-auth only when the required set is complete
   - do not add host-file fallback unless canonical policy docs publish it

**Validation**

- `cargo test -p shell --test world_gateway -- --nocapture`

Update tests to:

- keep all existing Codex precedence proofs
- replace the brittle assertion that non-Codex requests always send `integrated_auth == None`
- add:
  - selected-backend continuity for non-Codex lifecycle requests
  - no accidental `cli_codex` facet emission for non-Codex backends
  - positive emission of bounded API env-auth when inventory and policy allow it
  - negative emission when env auth is incomplete or policy-disallowed

**Done When**

- Shell request construction is backend-aware
- `SEAM-1` ownership boundaries remain intact
- Non-Codex auth emission no longer depends on the Codex special case

## WP4 - Prove One Real Non-Codex Runtime Path End to End

**Goal**

Publish a real multi-backend runtime handoff instead of a generic boundary with no supported second path.

**Files**

- [gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs)
- [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [gateway_runtime_parity.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/gateway_runtime_parity.rs)
- [world_gateway.rs tests](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/world_gateway.rs)

**Tasks**

1. Add one supported non-Codex runtime binding for the chosen proof target.
2. Carry the bounded auth handoff through request prep, runtime binding, start, status, sync, restart, and manifest recovery.
3. Keep unsupported or unbound backends explicit `unavailable`.
4. Preserve `cli:codex` as the regression floor.

**Proof Target**

- Recommended: `api:openai`
  - already present in inventory fixtures
  - already used in existing generic-backend tests
  - lowest repo churn for the first supported non-Codex runtime path

**Required Proof Matrix**

For the supported non-Codex backend:

- `status` before sync = unavailable
- `sync` = available
- `status` after sync = available
- repeated `sync` is idempotent
- `restart` preserves selected-backend continuity and recycles runtime state cleanly
- manifest recovery works
- child exit returns unavailable
- startup crash and ready-timeout remain transient and clean up correctly

For unsupported/unbound backends:

- no fallback to Codex
- existing explicit unavailable behavior remains intact

**Validation**

- `cargo test -p world-agent --test gateway_runtime_parity -- --nocapture`
- `cargo test -p shell --test world_gateway -- --nocapture`

**Done When**

- One named non-Codex backend is supported end to end
- The runtime handoff is no longer publishable only in theory
- No-fallback behavior remains an explicit proven contract

## WP5 - Publish THR-02 and Reconcile Governance

**Goal**

Make governance reflect the truth of the landed implementation and evidence.

**Files**

- [seam-2-closeout.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/governance/seam-2-closeout.md)
- [remediation-log.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/governance/remediation-log.md)

**Tasks**

1. Remove the current contradiction in `seam-2-closeout.md` where `open_remediations: []` coexists with unresolved blocker prose.
2. Reconcile `REM-003` and `REM-004` with the real blocker posture:
   - while implementation is incomplete: treat them as open blockers
   - once evidence lands: mark them resolved with concrete evidence
3. Update closeout to the actual publishable state:
   - `seam_exit_gate.status: passed`
   - `promotion_readiness: ready`
   - `gates.post_exec.landing: passed`
   - `THR-02` published
   - `Promotion blockers: none`
4. Cite exact landed evidence for:
   - binding lookup
   - capability gating
   - request/auth widening
   - runtime artifact semantics
   - lifecycle conformance
5. Keep `REM-005` deferred under `SEAM-3`.

**Validation**

- Manual governance review against:
  - [slice-99-seam-exit-gate.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threaded-seams/seam-2-runtime-realization-and-artifacts/slice-99-seam-exit-gate.md)
  - landed code
  - landed tests

**Done When**

- `THR-02` is honestly publishable
- Governance no longer contradicts code reality
- `SEAM-3` promotion can consume a single authoritative upstream handoff

## Recommended Execution Order

1. WP0 - lock target and guardrails
2. WP1 - runtime registry and capability ordering
3. WP2 - shared request/auth hardening
4. WP3 - shell auth emission
5. WP4 - supported non-Codex runtime proof matrix
6. WP5 - governance publication

## Stop Conditions

Do not mark `THR-02` published if any of these remain true:

- non-Codex runtime binding is still theoretical rather than proven
- non-Codex shell requests still rely on `integrated_auth == None`
- incomplete auth payloads can cross the request boundary silently
- unbound/unsupported backends fall back to Codex
- `seam-2-closeout.md` and `remediation-log.md` still disagree about blocker posture

## Recommended Command Sequence

Run these as each package lands:

```bash
cargo test -p agent-api-types -- --nocapture
cargo test -p world-agent --lib -- --nocapture
cargo test -p world-agent --test gateway_runtime_parity -- --nocapture
cargo test -p shell --test world_gateway -- --nocapture
```

Before final publication:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

## Final Exit Criteria

This step is complete only when all of the following are true:

- the lifecycle/auth request boundary is bounded and validated
- shell auth emission is backend-aware
- runtime binding lookup is no longer structurally Codex-only
- one real non-Codex backend path is proven through sync/status/restart and recovery
- Codex remains green as the regression floor
- unbound/unsupported backends still fail explicitly with no fallback
- `REM-003` and `REM-004` are resolved with evidence
- `THR-02` is recorded as published in `seam-2-closeout.md`
- `SEAM-3` can promote without relying on inference
