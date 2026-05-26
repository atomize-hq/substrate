# PLAN-14: Replace Gateway Secret Env Injection With Read-Once Auth-Bundle Delivery

Source SOW: [14-secret-handoff-into-the-world-gateway.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/14-secret-handoff-into-the-world-gateway.md)  
Gap matrix anchor: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Branch: `feat/session-centric-state-store`  
Base branch: `main`  
Plan type: Linux-first gateway secret-carrier hardening plan, backend-only, cross-crate (`world-agent` + `substrate-gateway`) with contract-doc impact, no UI scope  
Review posture: `/autoplan` scope discipline with `/plan-eng-review` depth, rewritten as one cohesive execution plan  
Status: execution-ready planning pass on 2026-05-04  
Outside voice: not used for this document generation

## Objective

The host-side auth sourcing, policy gating, and typed lifecycle request work is already landed.

The remaining bug is narrower and more serious: the last hop still turns integrated auth into child-process environment variables, and the gateway still expects integrated auth from env-backed surfaces. That leaves the shell/request boundary honest and the world-runtime boundary dishonest.

This plan closes that last-mile gap without reopening the earlier slices:

1. keep shell-owned host auth sourcing and `GatewayLifecycleRequestV1.integrated_auth` exactly as they are,
2. replace secret-bearing child-env delivery with a read-once auth bundle written over an inherited FD/pipe,
3. expose only one non-secret pointer env var, `SUBSTRATE_LLM_AUTH_BUNDLE_FD`,
4. teach `substrate-gateway` integrated startup to consume that bundle once, in memory, before provider construction,
5. prove that the managed gateway process environment no longer contains `SUBSTRATE_LLM_BACKEND_AUTH_*`, `OPENAI_API_KEY`, or equivalent secret-bearing values by default,
6. preserve the current policy precedence, failure taxonomy, lifecycle semantics, and restart/rotation behavior.

The user-visible result is simple:

- `substrate world gateway sync` and `restart` still work,
- policy-denied, invalid-integration, unavailable, and transient-failure behavior stay distinct,
- but the in-world gateway no longer receives integrated auth secrets in its process environment by default.

## Locked Starting State

### What is already done

The following work is already landed and is not reopened here:

- host-side auth sourcing and policy enforcement in [crates/shell/src/builtins/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs)
- typed `integrated_auth` payload validation in [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- backend selection and integrated gateway runtime binding in [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs)
- operator command family and exit-code contract in [docs/contracts/gateway/operator-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/gateway/operator-contract.md)
- policy precedence and fail-closed auth sourcing rules in [docs/contracts/gateway/policy-evaluation.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/gateway/policy-evaluation.md)

### Exact remaining gap

The remaining gap is concrete:

1. the shell builds `GatewayLifecycleRequestV1.integrated_auth` correctly in [crates/shell/src/builtins/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs:339)
2. `world-agent` validates the selected backend and request payload correctly in [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1528)
3. `world-agent` still resolves that payload into `ResolvedGatewayAuthHandoff { env_vars }` in [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:1097)
4. `start_runtime(...)` still injects those secret values into the child process environment in [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:706)
5. `substrate-gateway` still expects integrated auth from env-backed seams:
   - Codex integrated auth reads `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_*` from env in [crates/gateway/src/auth/codex_auth_context.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/auth/codex_auth_context.rs:43)
   - API-key providers still resolve `$OPENAI_API_KEY` style placeholders from process env during config load in [crates/gateway/src/cli/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/cli/mod.rs:350)
6. runtime parity and fake gateways still encode the old posture in:
   - [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs)
   - [crates/world-agent/tests/gateway_runtime_parity.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/gateway_runtime_parity.rs)
   - [crates/gateway/tests/openai_shared_parity.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/tests/openai_shared_parity.rs)

This means the trust boundary is still technically accurate at the shell and request layer, but false at the last mile.

## Frozen Execution Contract

This section is not interpretation space.

### Non-negotiable invariants

1. Host-side secret sourcing and policy precedence do not change.
2. `GatewayLifecycleRequestV1.integrated_auth` remains the only host-to-world auth seam.
3. The production integrated path stops placing secret-bearing values in the managed gateway process env by default.
4. The only new stable env surface for this slice is the non-secret pointer env var `SUBSTRATE_LLM_AUTH_BUNDLE_FD`.
5. The auth bundle is written once, read once, closed promptly, and never persisted to disk.
6. Canonical bundle field names stay in the `SUBSTRATE_LLM_BACKEND_AUTH_*` family even when values are no longer env vars.
7. Integrated runtime startup fails closed if the bundle is missing, malformed, unreadable, incomplete, or not consumed.
8. `sync` and `restart` always deliver a fresh bundle to the replacement process instance.
9. `host_only` gateway mode remains a bounded compatibility path and is not redefined by this slice.
10. No new public control plane, request family, or host credential source is introduced.

### Shared schema ownership

This plan freezes the bundle schema location so the implementation does not drift.

- `crates/common/src/gateway_auth_bundle.rs` owns:
  - `SUBSTRATE_LLM_AUTH_BUNDLE_FD`
  - `GatewayAuthBundleV1`
  - canonical field-name constants for `cli:codex` and `api:openai`
- `world-agent` serializes that schema
- `substrate-gateway` deserializes that schema
- `agent-api-types` does not own this bundle, because the bundle is not a new public transport contract

This is the smallest DRY answer. Duplicating string keys across `world-agent`, `gateway`, and tests is a drift trap. Moving the schema into `agent-api-types` would incorrectly turn a private last-mile carrier into a public host/world API.

### Chosen bundle contract

The inherited FD/pipe carries one JSON document, written by `world-agent` and read to EOF by `substrate-gateway`:

```json
{
  "schema_version": 1,
  "backend_id": "cli:codex",
  "fields": {
    "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID": "acct_...",
    "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN": "header.payload.signature"
  }
}
```

For `api:openai`, the canonical field key is:

- `SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY`

Required rules:

1. `fields` keys are canonical auth field names, not raw host env names.
2. The host env name `OPENAI_API_KEY` remains only the shell-side source of truth, never the in-world delivery key.
3. The gateway reads the bundle exactly once during integrated startup, closes the FD, and retains only in-memory auth state.
4. Any missing required field for the selected backend is invalid integration at request-mapping time and a startup failure if a malformed bundle somehow reaches the child.
5. Re-reading after the startup read is not part of the contract. Restart gets a fresh bundle instead.

### Chosen startup sequencing

This plan also freezes the gateway startup order. The current order is wrong for integrated mode because config env interpolation runs before any integrated auth can be injected.

The new required order is:

1. resolve `GatewayLaunchContract`
2. parse config file without resolving provider secret placeholders yet
3. if `GatewayMode::InWorld`, read and validate `GatewayAuthBundleV1`
4. overlay integrated auth into in-memory gateway state:
   - populate integrated Codex auth context
   - patch API-key provider config in memory
5. resolve remaining non-secret env placeholders
6. construct `ProviderRegistry`
7. start serving traffic

That means `crates/gateway/src/main.rs` and [crates/gateway/src/cli/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/cli/mod.rs) must move from the current one-step `AppConfig::from_file(...) -> resolve_env_vars()` flow to a two-phase load for integrated mode.

### Failure taxonomy freeze

The plan does not leave error classification fuzzy.

Request-layer failures remain what they already are:

- policy-denied host secret reads stay policy denials
- selected-backend / payload mismatches stay invalid integration
- incomplete request-provided auth payloads stay invalid integration

Bundle-transport failures are runtime failures, not auth-policy failures:

- pipe creation failure
- bundle serialization failure
- partial write / write failure
- child spawn failure
- missing pointer env in integrated mode
- unreadable inherited FD
- malformed JSON in the bundle
- child startup failure after bundle read

Those failures surface as transient integrated-startup failures. They are real bugs or runtime faults after the request already passed validation. They are not user policy mistakes.

## Step 0: Scope Challenge

### 0A. What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| host-side auth precedence and policy gates | [crates/shell/src/builtins/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs), [crates/shell/tests/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/world_gateway.rs) | Reuse exactly. No precedence or policy behavior changes. |
| typed gateway lifecycle request contract | [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs) | Reuse exactly. No new request family. |
| managed gateway runtime root, config, logs, manifests, readiness loop | [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs) | Reuse. Swap only the secret carrier and startup contract. |
| gateway launch-mode and token-store contract | [crates/gateway/src/launch.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/launch.rs) | Reuse. Extend integrated mode with auth-bundle consumption. |
| Codex integrated auth resolution | [crates/gateway/src/auth/codex_auth_context.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/auth/codex_auth_context.rs) | Change consumer from env to startup-owned in-memory context for integrated mode only. |
| provider registry construction | [crates/gateway/src/providers/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/providers/registry.rs), [crates/gateway/src/server/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/server/mod.rs) | Reuse. Feed resolved integrated auth into this seam before providers are built. |
| canonical auth field-name family | [docs/project_management/packs/PHASE_8_CROSS_CUTTING_DECISION_REGISTRY.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/PHASE_8_CROSS_CUTTING_DECISION_REGISTRY.md:250) | Reuse. Make code finally match the documented canonical names. |

### 0B. Minimum honest diff

The minimum honest implementation is:

1. keep shell request shape and policy logic unchanged
2. replace `ResolvedGatewayAuthHandoff.env_vars` with a canonical auth-bundle model
3. pass the bundle to the child via inherited FD plus `SUBSTRATE_LLM_AUTH_BUNDLE_FD`
4. split gateway startup into parse -> integrated overlay -> remaining env resolution -> provider construction
5. remove test and doc assumptions that the gateway receives secret env vars

Anything smaller is fake progress.

In particular:

- changing only the Codex auth helper is insufficient because `api:openai` still fails earlier in config env resolution
- changing only provider config interpolation is insufficient because integrated Codex auth still reads env
- writing bundle contents back into env inside the gateway just recreates the original bug one layer later

### 0C. Complexity check

This slice touches more than 8 files and multiple crates. That is justified and still minimal.

Expected production surfaces:

1. `crates/common/src/gateway_auth_bundle.rs`
2. `crates/common/Cargo.toml`
3. `crates/gateway/Cargo.toml`
4. `crates/world-agent/src/gateway_runtime.rs`
5. `crates/world-agent/tests/gateway_runtime_parity.rs`
6. `crates/gateway/src/main.rs`
7. `crates/gateway/src/cli/mod.rs`
8. `crates/gateway/src/auth/codex_auth_context.rs`
9. `crates/gateway/src/launch.rs`
10. `crates/gateway/src/server/mod.rs`
11. `crates/gateway/src/providers/registry.rs`
12. `crates/gateway/tests/openai_shared_parity.rs`
13. `crates/shell/tests/world_gateway.rs`
14. `docs/contracts/gateway/policy-evaluation.md`
15. `crates/gateway/docs/contracts/chatgpt-codex-auth-handoff-contract.md`
16. `AGENT_ORCHESTRATION_GAP_MATRIX.md`

This is the minimal real blast radius because the change spans:

- producer contract (`world-agent`)
- consumer contract (`substrate-gateway`)
- regression floor
- operator/documentation truth

### 0D. Search and completeness check

Search-before-building result:

- **[Layer 1]** reuse the existing typed `integrated_auth` request, do not invent a second request family
- **[Layer 1]** reuse the documented canonical `SUBSTRATE_LLM_BACKEND_AUTH_*` field-name family instead of raw provider env names
- **[Layer 1]** reuse inherited FD/pipe delivery, not temp files, named pipes, or persisted bundle artifacts
- **[Layer 1]** reuse the gateway launch contract and provider registry seams, inject integrated auth there instead of scattering startup reads across providers
- **[EUREKA]** the real seam is not only `CodexIntegratedAuthHandoff::from_env()`. The earlier failure is config load in `AppConfig::from_file()`. If that order stays intact, `api:openai` still dies before the bundle can help

Shortcut options explicitly rejected:

- keep `OPENAI_API_KEY` in the child env and only remove Codex env vars
- read bundle contents into env vars inside the gateway before provider init
- persist auth bundles under the runtime dir and point the gateway at files
- keep env injection as an automatic fallback if bundle consumption fails

### 0E. Distribution and runtime contract check

No new distributable artifact type is introduced. This is runtime-contract work.

That means this plan must cover:

- `world-agent` runtime launch behavior
- `substrate-gateway` integrated startup behavior
- Linux parity tests and shell contract tests
- canonical docs and gap-matrix closeout

No new installer or release-pipeline work is required beyond keeping existing binaries buildable and testable on Linux.

### 0F. NOT in scope

- redesigning `GatewayLifecycleRequestV1`
- changing host-side policy precedence or expanding auth sources
- generic token-store redesign in `substrate-gateway`
- adding support for new `api:*` backends beyond the current `api:openai` floor
- cross-platform parity beyond explicit Linux-first validation
- new operator commands or status schema families
- a compatibility env-fallback flag unless a real blocker appears during implementation

## Architecture Review

### Locked architecture decisions

`PLAN-14` does not leave the core implementation path open-ended.

1. `world-agent` owns bundle creation, serialization, FD inheritance, and child-env cleanup.
2. `substrate-gateway` owns one-time bundle consumption and in-memory auth hydration.
3. `substrate-common` owns the shared bundle schema and canonical field constants.
4. integrated-mode startup owns auth overlay before provider construction.
5. provider code consumes already-resolved auth context. Provider code does not become the owner of host/world secret delivery.

This is the smallest honest architecture. The request seam is already good. The bug is the last-mile carrier and the gateway startup order.

### Architecture findings resolved in-plan

[P1] (confidence: 10/10) [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:1097) still models the handoff as env assignments, which guarantees the carrier stays env-based no matter how correct the request contract is.

Resolution in this plan:

- replace `ResolvedGatewayAuthHandoff { env_vars }` with a bundle payload model
- make secret-bearing child env injection impossible on the default integrated path

[P1] (confidence: 10/10) [crates/gateway/src/cli/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/cli/mod.rs:350) resolves `$OPENAI_API_KEY` during config load, before integrated startup has any chance to supply API-key auth another way.

Resolution in this plan:

- split config load into parse-first and resolve-later phases
- overlay integrated API-key auth before generic env interpolation runs

[P1] (confidence: 9/10) [crates/gateway/src/auth/codex_auth_context.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/auth/codex_auth_context.rs:43) still treats env as the source of truth for integrated Codex auth.

Resolution in this plan:

- change integrated Codex auth to resolve from a startup-owned in-memory context
- keep standalone local auth-file behavior for `host_only`
- preserve explicit-account-id-over-JWT precedence exactly as today

[P1] (confidence: 9/10) [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:706) is the lifecycle seam that must guarantee fresh delivery and prompt cleanup. If bundle creation is bolted on somewhere else, restart correctness will drift.

Resolution in this plan:

- make bundle creation part of `start_runtime(...)`
- make `sync` and `restart` pass through the same fresh-bundle creation path
- fail before readiness probing if bundle creation or write fails

### Architecture ASCII diagrams

```text
CURRENT STATE
=============
shell
  ├── resolve_integrated_auth_payload(...)
  └── GatewayLifecycleRequestV1 { integrated_auth }
         │
         ▼
world-agent
  ├── resolve_integrated_auth_handoff(...)
  │     └── Vec<(env_key, secret_value)>
  └── start_runtime(...)
         └── child.env(secret_key, secret_value)
                │
                ▼
substrate-gateway
  ├── AppConfig::from_file(...)
  │     └── resolve_env_vars() demands $OPENAI_API_KEY
  └── CodexIntegratedAuthHandoff::from_env()

Result:
  integrated auth still lives in the gateway process environment
```

```text
TARGET STATE
============
shell
  ├── host policy-gated secret sourcing
  └── GatewayLifecycleRequestV1 { integrated_auth }
         │
         ▼
world-agent
  ├── validate selected backend + request payload
  ├── map request payload -> canonical auth fields
  ├── serialize GatewayAuthBundleV1
  ├── create one-time pipe
  ├── write JSON bundle to write end
  ├── pass read end to child
  └── child env:
        SUBSTRATE_LLM_AUTH_BUNDLE_FD=<n>
                │
                ▼
substrate-gateway startup
  ├── parse config without secret interpolation
  ├── read bundle once from FD <n>
  ├── close FD
  ├── hydrate in-memory integrated auth context
  ├── patch integrated API-key config in memory
  └── build ProviderRegistry from resolved in-memory state

Result:
  secret values live in memory only, not child env
```

```text
INTEGRATED STARTUP ORDER
========================
main.rs
  ├── GatewayLaunchContract::resolve(...)
  ├── AppConfig::parse_file_without_secret_resolution(...)
  ├── if InWorld:
  │     ├── read GatewayAuthBundleV1 from SUBSTRATE_LLM_AUTH_BUNDLE_FD
  │     ├── build IntegratedGatewayAuthContext
  │     └── overlay provider/auth state in memory
  ├── resolve remaining non-secret env placeholders
  └── start_server(config, launch, integrated_auth_ctx)
```

### Required field mapping

| Selected backend | Host-side source name(s) | Canonical bundle field(s) | Consumer |
| --- | --- | --- | --- |
| `cli:codex` | `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID`, `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN`, or `~/.codex/auth.json` fallback allowed by policy | same canonical `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_*` names | startup-owned `IntegratedGatewayAuthContext`, then [crates/gateway/src/auth/codex_auth_context.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/auth/codex_auth_context.rs) |
| `api:openai` | host `OPENAI_API_KEY` | `SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY` | integrated provider-config overlay before [ProviderRegistry::from_configs_with_models_and_mode(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/providers/registry.rs) |

## Code Quality Review

### Findings resolved in-plan

[P2] (confidence: 9/10) `world-agent` and `gateway` need the same bundle schema. Hand-rolled JSON keys in both crates plus tests would be a drift trap.

Resolution:

- put the serde schema and canonical field constants in `crates/common/src/gateway_auth_bundle.rs`
- do not duplicate free-form string literals across three modules

[P2] (confidence: 9/10) integrated auth and generic config env interpolation are different concerns. Forcing one helper to do both keeps the secret path implicit and hard to audit.

Resolution:

- keep generic env interpolation for `host_only` and non-secret config
- add one explicit integrated-auth overlay step before provider construction
- make the integrated secret path obvious in startup code

[P2] (confidence: 8/10) runtime parity fixtures currently encode the wrong security posture. Repeating slightly different shell fragments across tests will make the rewrite brittle.

Resolution:

- allow one test-only fake-gateway helper that reads `SUBSTRATE_LLM_AUTH_BUNDLE_FD`
- centralize assertions for pointer-env presence, secret-env absence, and read-once behavior

### Allowed code shape

1. one shared bundle-schema module in `substrate-common`
2. one startup-owned `IntegratedGatewayAuthContext` inside `substrate-gateway`
3. one explicit integrated provider-overlay step before provider construction
4. no public request-shape changes
5. no second integrated auth source inside the gateway
6. no temp-file bundle staging
7. no silent env fallback on the integrated path

## Test Review

### Test framework detection

- Runtime: Rust
- Framework: `cargo test`
- Primary packages: `world-agent`, `substrate-gateway`, `shell`
- No LLM eval suite is required for this slice

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] world-agent request -> bundle mapping
    │
    ├── [GAP]         typed cli:codex payload -> canonical bundle fields
    ├── [GAP]         typed api:openai payload -> canonical bundle field
    ├── [GAP]         selected-backend mismatch fails invalid_integration
    └── [GAP]         empty/partial payload fails invalid_integration

[+] world-agent spawn plumbing
    │
    ├── [GAP] [->E2E] read end inherited by child, write end stays parent-only
    ├── [GAP]         secret env vars removed from child
    ├── [GAP]         bundle write failure fails before readiness loop
    └── [GAP]         restart delivers a fresh bundle and closes the old one

[+] gateway integrated startup
    │
    ├── [GAP]         config parses before integrated auth overlay
    ├── [GAP]         pointer env present and readable
    ├── [GAP]         missing pointer env fails integrated startup
    ├── [GAP]         malformed JSON fails integrated startup
    ├── [GAP]         codex integrated auth resolves from bundle, not env
    └── [GAP]         api:openai provider config resolves from bundle, not env interpolation

[+] contract and parity protection
    │
    ├── [GAP]         shell policy precedence unchanged
    ├── [GAP]         gateway env no longer contains secret-bearing auth fields
    ├── [GAP]         raw host env name OPENAI_API_KEY absent from child env
    └── [GAP]         operator-facing failure buckets stay distinct

─────────────────────────────────
COVERAGE: 0/14 auth-carrier paths proven
GAPS: 14 paths need coverage before closeout
─────────────────────────────────
```

### User flow coverage

```text
USER FLOW COVERAGE
===========================
[+] cli:codex integrated sync
    ├── [GAP] [->E2E] sync -> bundle read -> gateway ready
    ├── [GAP]         explicit account_id survives through bundle
    └── [GAP]         JWT fallback still works when explicit account_id is absent

[+] api:openai integrated sync
    ├── [GAP] [->E2E] sync -> config parse -> bundle overlay -> provider ready
    ├── [GAP]         no OPENAI_API_KEY in child env
    └── [GAP]         missing API-key field fails startup cleanly

[+] restart / rotation
    ├── [GAP] [->E2E] restart launches replacement process with fresh bundle
    ├── [GAP]         old bundle FD is closed and cannot be reused
    └── [GAP]         lifecycle status remains available after replacement

[+] failure posture
    ├── [GAP]         malformed bundle -> transient startup failure
    ├── [GAP]         pointer missing or unreadable -> transient startup failure
    └── [GAP]         no silent fallback to secret env injection
```

### Required tests to add or extend

1. `crates/world-agent/src/gateway_runtime.rs`
   - replace env-oriented unit tests around `resolve_integrated_auth_handoff(...)`
   - add canonical field-mapping tests for `cli:codex` and `api:openai`
   - add bundle serialization tests against `GatewayAuthBundleV1`

2. `crates/world-agent/tests/gateway_runtime_parity.rs`
   - rewrite the fake gateway startup to read `SUBSTRATE_LLM_AUTH_BUNDLE_FD`
   - prove pointer-env presence, secret-env absence, read-once behavior, restart rotation, and fail-closed startup

3. `crates/gateway/src/main.rs` and `crates/gateway/src/cli/mod.rs`
   - add tests proving integrated mode parses config before secret interpolation
   - add tests proving host-only mode keeps the existing env-resolution behavior

4. `crates/gateway/src/auth/codex_auth_context.rs`
   - add integrated bundle-backed auth-resolution tests
   - keep standalone local-path tests intact
   - verify explicit account-id wins over JWT fallback exactly as before

5. `crates/gateway/src/server/mod.rs` and `crates/gateway/src/providers/registry.rs`
   - add startup tests that integrated mode fails without pointer env or with malformed bundle
   - add integrated startup tests that patch API-key providers in memory

6. `crates/gateway/tests/openai_shared_parity.rs`
   - replace integrated env-assumption tests with bundle-backed parity coverage
   - verify `ChatGPT-Account-ID` behavior still works without env-based secret delivery

7. `crates/shell/tests/world_gateway.rs`
   - keep existing host-side precedence, allowlist, partial-auth, and exit-code tests green
   - add assertions only if response wording changes, not for carrier mechanics

### Regression rule for this slice

Any path that can satisfy all three conditions below is a critical gap and must get a regression test:

1. request validation has already succeeded
2. integrated auth delivery can still be wrong or absent
3. operator-visible status could still suggest the runtime is healthy

## Failure Modes Registry

| New codepath | Real production failure | Test covers it? | Error handling exists? | User sees clear error? | Critical gap? |
| --- | --- | --- | --- | --- | --- |
| request -> canonical mapping | `api:openai` still maps to raw `OPENAI_API_KEY` instead of canonical bundle field | no | no | no | yes |
| bundle creation | pipe/FD creation fails before spawn | no | partial | partial | yes |
| bundle write | parent writes partial JSON then child starts | no | no | no | yes |
| child env cleanup | child still receives `SUBSTRATE_LLM_BACKEND_AUTH_*` or `OPENAI_API_KEY` | no | no | no | yes |
| config parse / overlay order | config still resolves `$OPENAI_API_KEY` before the bundle can be read | no | no | no | yes |
| gateway startup read | pointer env missing or unreadable | no | partial | partial | yes |
| gateway JSON parse | malformed bundle becomes generic startup crash | no | partial | partial | yes |
| codex integrated auth | gateway falls back to env in integrated mode | no | no | no | yes |
| api-key provider init | config interpolation still demands secret env in integrated mode | no | no | no | yes |
| restart rotation | replacement process reuses stale bundle or stale in-memory auth | no | no | no | yes |

## Performance Review

This is a correctness-first slice. The performance rules are still simple and important:

1. read the auth bundle once at startup, not per request
2. keep the bundle small, one JSON document with the selected backend only
3. avoid any on-disk bundle persistence or retry loops that turn startup into file IO
4. keep provider request paths unchanged after startup hydration

There is no throughput feature here. The risk is startup complexity, not steady-state QPS.

## DX Guardrails

This is a developer tool. Failure messages need to be precise.

Required error-message posture:

1. `world-agent` must say whether the failure was request validation, bundle creation, bundle write, child spawn, or readiness failure
2. integrated gateway startup must say whether the problem was missing pointer env, unreadable FD, malformed bundle, or missing required field for the selected backend
3. operator-visible diagnostics may name:
   - selected backend id
   - runtime log paths
   - manifest path
   - pointer env var name
4. operator-visible diagnostics must not print:
   - bundle contents
   - canonical secret field values
   - raw host env names plus resolved secret values

## Worktree Parallelization Strategy

There is one safe parallel window here, but only after the parent freezes the bundle contract. Before that point, parallel work would just create schema drift.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| P0. Freeze bundle schema, canonical field mapping, pointer env name, startup order, and failure taxonomy | `llm-last-mile/`, `crates/common/`, `crates/world-agent/`, `crates/gateway/src/main.rs`, `crates/gateway/src/cli/`, `docs/contracts/` | - |
| L1. Producer lane, world-agent bundle generation and spawn plumbing | `crates/world-agent/` | P0 |
| L2. Consumer lane, gateway startup refactor and in-memory auth overlay | `crates/gateway/` | P0 |
| L3. Regression/docs lane, parity tests and contract-doc closeout | `crates/world-agent/tests/`, `crates/gateway/tests/`, `crates/shell/tests/`, `docs/contracts/`, `AGENT_ORCHESTRATION_GAP_MATRIX.md` | L1, L2 |
| P1. Parent proof wall and closeout | `crates/world-agent/`, `crates/gateway/`, `crates/shell/`, `docs/` | L3 |

### Parallel lanes

- Lane A: `P0`, sequential, parent-owned
  - write scope: `PLAN-14`, `crates/common/src/gateway_auth_bundle.rs`, any shared constants needed by both code lanes
- Lane B: `L1`, world-agent producer lane
  - write scope: `crates/world-agent/**`
- Lane C: `L2`, gateway consumer lane
  - write scope: `crates/gateway/**`
- Lane D: `L3`, regression/docs lane after B and C merge
  - write scope: tests + docs only
- Lane E: `P1`, sequential proof wall and closeout

### Execution order

1. Parent freezes the shared bundle schema, pointer env name, canonical field names, startup order, and error taxonomy.
2. Launch Lane B and Lane C in parallel worktrees.
3. Merge Lane B first only if it changes shared constants or serialization details Lane C consumes. Otherwise merge whichever lane stabilizes first.
4. Run Lane D only after both code lanes merge. The regression floor must lock the final contract, not an intermediate one.
5. Run the parent proof wall and closeout.

### Conflict flags

- Lane B and Lane C must not invent different `GatewayAuthBundleV1` schemas
- only one place may define `SUBSTRATE_LLM_AUTH_BUNDLE_FD`
- only one place may define `SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY`
- Lane C must not preserve secret-env fallback as an invisible compatibility path
- Lane D must not encode fake gateway fixtures against intermediate env behavior
- if the shared schema starts spreading into `agent-api-types`, stop and re-evaluate. That is over-scoping the slice

### Parallelization verdict

Two code lanes can run in parallel after one parent-owned contract freeze. Worker cap should stay `2`.

## Implementation Sequence

### Step 1. Freeze the shared auth-bundle contract

Files:

- [crates/common/src/gateway_auth_bundle.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/gateway_auth_bundle.rs)
- [crates/common/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/Cargo.toml)
- [crates/gateway/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/Cargo.toml)
- [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs)
- [crates/gateway/src/main.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/main.rs)
- [crates/gateway/src/cli/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/cli/mod.rs)

Work:

1. add `GatewayAuthBundleV1` and the canonical field constants to `substrate-common`
2. freeze `SUBSTRATE_LLM_AUTH_BUNDLE_FD` there too
3. freeze the `cli:codex` and `api:openai` mapping rules
4. freeze the integrated startup order: parse -> read bundle -> overlay -> resolve remaining env -> build registry
5. freeze the transient-vs-invalid failure split for the carrier

Validation gate:

- one schema
- one pointer env name
- one canonical field mapping
- one startup order
- one failure taxonomy

### Step 2. Replace world-agent env injection with bundle delivery

Files:

- [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs)
- [crates/world-agent/tests/gateway_runtime_parity.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/gateway_runtime_parity.rs)

Work:

1. replace `ResolvedGatewayAuthHandoff { env_vars }` with a canonical bundle model
2. map `cli:codex` and `api:openai` request payloads into canonical bundle fields
3. create the one-time pipe/FD channel during `start_runtime(...)`
4. write the JSON bundle, pass the read end to the child, export only `SUBSTRATE_LLM_AUTH_BUNDLE_FD`
5. remove secret-bearing child env assignments from the default integrated path
6. ensure spawn failure, write failure, and restart replacement all clean up FDs correctly

Validation gate:

- child receives only the pointer env var
- bundle write and close discipline is correct
- restart always re-delivers a fresh bundle

### Step 3. Refactor gateway startup to consume the bundle before provider construction

Files:

- [crates/gateway/src/main.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/main.rs)
- [crates/gateway/src/cli/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/cli/mod.rs)
- [crates/gateway/src/launch.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/launch.rs)
- [crates/gateway/src/server/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/server/mod.rs)
- [crates/gateway/src/providers/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/providers/registry.rs)
- [crates/gateway/src/auth/codex_auth_context.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/auth/codex_auth_context.rs)

Work:

1. split config load so integrated mode can parse config without resolving secret env placeholders first
2. add a startup-owned `IntegratedGatewayAuthContext` built from `GatewayAuthBundleV1`
3. read the bundle exactly once during integrated startup and close the FD immediately on success or failure
4. patch API-key bundle fields into in-memory provider config before `ProviderRegistry::from_configs_with_models_and_mode(...)`
5. change integrated Codex auth resolution from env-based lookup to the injected in-memory context
6. preserve standalone local token-store and auth-file behavior for `host_only`
7. fail startup before serving traffic if the bundle is missing or malformed

Validation gate:

- `api:openai` no longer depends on process-env interpolation in integrated mode
- `cli:codex` no longer depends on process env in integrated mode
- host-only behavior remains unchanged

### Step 4. Rewrite the regression floor and close the docs gap

Files:

- [crates/world-agent/tests/gateway_runtime_parity.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/gateway_runtime_parity.rs)
- [crates/gateway/tests/openai_shared_parity.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/tests/openai_shared_parity.rs)
- [crates/shell/tests/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/world_gateway.rs)
- [docs/contracts/gateway/policy-evaluation.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/gateway/policy-evaluation.md)
- [crates/gateway/docs/contracts/chatgpt-codex-auth-handoff-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/docs/contracts/chatgpt-codex-auth-handoff-contract.md)
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)

Work:

1. invert env-oriented parity tests into bundle-oriented parity tests
2. prove pointer-env presence, secret-env absence, read-once behavior, and restart rotation
3. keep shell precedence and failure-taxonomy tests green
4. update docs so the old "preferred direction" becomes the landed default integrated path
5. close the gap-matrix row only after tests and docs match the code

Validation gate:

- regression floor protects the new posture
- docs and matrix no longer describe env injection as current behavior

### Step 5. Parent proof wall and closeout

Work:

1. run the Rust unit and integration test wall
2. run one Linux manual proof that the gateway env no longer contains secret-bearing auth values
3. run `sync`, `status --json`, and `restart` against the integrated path
4. capture closeout evidence for the gap-matrix row

Closeout is not done until the code, tests, and docs all agree that the default integrated path is bundle-based.

## Recommended Verification Commands

Run in this order.

```bash
cargo test -p world-agent gateway_runtime -- --nocapture
cargo test -p world-agent --test gateway_runtime_parity -- --nocapture
cargo test -p substrate-gateway codex_auth_context -- --nocapture
cargo test -p substrate-gateway openai_shared_parity -- --nocapture
cargo test -p shell --test world_gateway -- --nocapture
```

Linux manual proof after the test wall:

```bash
substrate world gateway sync
substrate world gateway status --json
substrate world gateway restart
```

Required manual assertions for the integrated runtime:

1. status returns `available`
2. the managed gateway process environment contains `SUBSTRATE_LLM_AUTH_BUNDLE_FD`
3. the managed gateway process environment does not contain:
   - `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID`
   - `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN`
   - `SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY`
   - `OPENAI_API_KEY`
4. restart produces a fresh process instance and still reaches `available`

## Definition of Done

1. the shell still resolves and sends typed `integrated_auth` under existing policy gates
2. `world-agent` no longer models the default integrated handoff as child env assignments
3. `world-agent` launches the managed gateway with a read-once auth bundle and only the non-secret pointer env var
4. `substrate-gateway` integrated startup parses config before secret interpolation, reads the bundle once, and closes the FD promptly
5. `substrate-gateway` integrated `cli:codex` auth no longer depends on process env
6. `substrate-gateway` integrated `api:openai` startup no longer depends on process-env interpolation
7. the managed gateway process environment no longer contains secret-bearing auth values by default
8. `sync` and `restart` both re-deliver fresh auth without persistence
9. shell policy tests still prove precedence and failure taxonomy are unchanged
10. docs and gap matrix describe bundle delivery as the landed integrated default path

## Deferred Work

- additional `api:*` canonical field expansions beyond `api:openai`
- optional explicit compatibility env-fallback mode, only if a real blocker appears later
- cross-platform parity beyond Linux-first validation
- broader gateway auth-source refactors unrelated to this carrier swap

No new `TODOS.md` entry is required yet. These are explicit deferrals, not forgotten work.

## Completion Summary

- Step 0: scope accepted as a narrow last-mile secret-carrier hardening slice
- Architecture Review: 4 core architecture seams frozen, no remaining contract ambiguity
- Code Quality Review: 3 structural drift risks identified and resolved in-plan
- Test Review: diagrams produced, 14 codepath gaps identified
- Performance Review: startup-only overhead accepted, 0 throughput redesign
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0 durable TODOs proposed
- Failure modes: 10 critical gaps identified for implementation proof
- Outside voice: not used for this document generation
- Parallelization: 5 execution phases, 1 real parallel window, worker cap stays `2`
- Lake Score: complete option chosen over partial carrier swaps, temp-file bundles, or env fallback

<!-- AUTONOMOUS DECISION LOG -->
## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Scope | Keep shell request shape and host policy logic unchanged | Mechanical | Minimal diff | The request contract is already correct; the lie is in the last-mile carrier | New request family or shell-side redesign |
| 2 | Shared schema | Put `GatewayAuthBundleV1` and canonical field constants in `substrate-common` | Mechanical | DRY | Two runtime crates and the regression floor need one shared internal truth | Duplicated JSON builders, `agent-api-types`, or a new micro-crate |
| 3 | Bundle format | Use one read-once JSON bundle over inherited FD/pipe | Mechanical | Explicit over clever | JSON is auditable, small, and fits the closed auth field set | Temp files, sockets, or opaque binary blobs |
| 4 | Canonical names | Map `api:openai` to `SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY` | Mechanical | Completeness | Docs already define the canonical family, and raw host env names should stay host-only | Keeping raw `OPENAI_API_KEY` as the in-world delivery key |
| 5 | Gateway startup | Split startup into parse -> bundle overlay -> remaining env resolution -> provider construction | Mechanical | Pragmatic | This is the earliest honest place to break env dependence for both API-key and Codex integrated auth | Leaving `AppConfig::from_file()` env resolution first |
| 6 | Auth source | Make integrated Codex auth resolve from startup-owned in-memory context | Mechanical | Explicit over clever | Integrated auth should be injected once, not rediscovered from process env later | `from_env()` on the integrated path |
| 7 | Fallback posture | No silent fallback to env injection on the integrated path | Mechanical | Completeness | Silent fallback recreates the original security gap | Warning-only or automatic compatibility fallback |
| 8 | Parallelization | Run `world-agent` producer and gateway consumer lanes in parallel after the parent contract freeze | Taste | Pragmatic | That is the widest safe parallel window without schema drift | Fully serial execution or three overlapping code lanes |
