**Warning: Pre-Planning Only. This document will be superseded by downstream FSE planning or decomposition artifacts.**

# gateway-backend-selection-runtime-integration minimal spec draft

## Scope and authority

This draft defines the pack-level alignment backbone for `gateway-backend-selection-runtime-integration`.

This draft is allowed to define:
- cross-cutting defaults shared by downstream feature-local specs
- precedence across existing CLI, config, policy, inventory, and derived env-var surfaces
- high-level invariants for backend selection, policy gating, adapter realization, auth handoff, and platform parity
- seam boundaries that downstream seam planning and decomposition will refine
- unresolved choices that block deterministic downstream planning

This draft does not define:
- execution tasks
- kickoff prompts
- ownership of runtime worktrees
- detailed implementation sequencing
- additive operator-schema fields outside the owners already named in `spec_manifest.md`

Authoritative upstream sources for this draft:
- `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/impact_map.md`

External source-of-truth docs reused by this feature:
- `docs/contracts/gateway/operator-contract.md`
- `docs/contracts/gateway/status-schema.md`
- `docs/contracts/gateway/policy-evaluation.md`
- `docs/contracts/gateway/runtime-parity.md`
- `docs/contracts/gateway/backend-adapter-selection.md`
- `docs/contracts/gateway/backend-adapter-protocol.md`
- `docs/contracts/gateway/backend-adapter-schema.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`

## Defaults and precedence

### Operator-facing precedence

1. Existing `substrate world gateway` CLI commands remain the entrypoints for lifecycle actions. This feature adds no new backend-selection CLI flag family.
2. Effective config from `$SUBSTRATE_HOME/config.yaml` and `<workspace_root>/.substrate/workspace.yaml` defines gateway enablement, gateway mode, and `llm.routing.default_backend` using the precedence already owned by ADR-0027 surfaces.
3. Effective policy from `$SUBSTRATE_HOME/policy.yaml` and `<workspace_root>/.substrate/policy.yaml` gates `llm.allowed_backends`, `llm.secrets.env_allowed`, and `agents.host_credentials.read.allowed_backends`.
4. Backend inventory lookup validates backend existence and filename-to-id consistency before integrated adapter realization.
5. Integrated adapter registry metadata resolves one binding for the selected backend after config, policy, and inventory all pass.
6. Internal env vars used between shell and world-service are derived transport outputs from the earlier steps. They do not override config, policy, or inventory truth.

### Source-of-truth posture

- `contract.md` will own the feature-local operator contract deltas for selected-backend realization and exit-code meanings.
- `policy-spec.md` will own ordered evaluation inputs, allowlist gating, and the trusted-input boundary.
- `env-vars-spec.md` will own internal env-var definitions, producers, consumers, defaults, redaction, and auth handoff precedence.
- `gateway-runtime-adapter-protocol-spec.md` will own adapter lookup order, capability gating, auth handoff resolution order, config rendering order, launch order, readiness confirmation, and restart order.
- `gateway-runtime-adapter-schema-spec.md` will own integrated adapter binding metadata, auth payload variants, runtime config payloads, and feature-local failure shapes.
- `filesystem-semantics-spec.md` will own backend inventory path rules, runtime artifact paths, host credential file rules, permissions, and inspectability expectations.
- `platform-parity-spec.md` will own Linux, macOS, and Windows guarantees plus hidden divergence boundaries.
- `compatibility-spec.md` will own the `cli:codex` regression baseline and explicit unsupported-backend failure posture.
- `manual_testing_playbook.md` will own the validation matrix and required smoke-script assertions.

## Failure posture and invariants

### Failure posture

- Integrated backend realization is fail-closed.
- The lifecycle fails before adapter dispatch when the selected backend id is malformed, missing from inventory, inventory-inconsistent, or disallowed by policy.
- The lifecycle fails before launch when no integrated adapter binding exists, required capabilities are unavailable, or required auth handoff material is unavailable.
- Unsupported integrated backends fail explicitly. The lifecycle does not collapse back to a Codex-specific path.

### Security and redaction invariants

- Stable backend ids in `<kind>:<name>` form remain the only backend selectors at the Substrate boundary.
- Gateway-local config, admin mutation, and token persistence do not authorize execution.
- `llm.secrets.env_allowed` and `agents.host_credentials.read.allowed_backends` remain the only policy gates for host-side auth material sourcing in this feature.
- Secret-bearing env vars and host credential material stay under shared redaction rules. Downstream docs must route every secret-bearing field through `env-vars-spec.md` and the existing redaction helpers rather than inventing feature-local logging rules.
- Operator-facing wiring outputs remain non-secret. This draft does not widen the stable operator-facing env output family.
- Tuple metadata, tuple-axis policy keys, and status-schema widening stay outside this feature-local doc set and remain owned by ADR-0042 or ADR-0043 follow-ons.

## Exit-code posture

- Exit-code taxonomy reference: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- Current evidence does not require new exit codes for this feature.
- Downstream docs must stay inside the existing buckets already stated by ADR-0046:
  - `0` for success
  - `2` for invalid configuration, invalid inventory state, malformed backend selection, or unsupported integrated adapter selection
  - `3` for transient runtime failures during config rendering, launch, readiness probing, or restart
  - `4` for required gateway or adapter dependency unavailable
  - `5` for policy or safety failures, including backend denial and blocked env-read or host-credential-read paths

## Cross-cutting seams and constraints

### Naming and vocabulary

- Use `selected backend` for the backend id produced by effective config and policy.
- Use `inventory-backed backend realization` for the path from selected backend id through inventory validation into one integrated adapter binding.
- Use `integrated adapter binding` for the single adapter binding selected for the chosen backend id.
- Use `auth handoff` for bounded host-to-world auth material delivery. Do not fold auth semantics into the backend id.
- Use `supported integrated backend` for a backend id with valid inventory, policy approval, one integrated adapter binding, required capabilities, and satisfiable auth handoff material.

### Shared constraints for downstream specs

- `status`, `sync`, and `restart` remain the only operator lifecycle commands for this feature seam.
- `status --json` remains under the existing status schema owner unless that owner widens the schema explicitly.
- Effective config and policy remain the only operator-controlled inputs. Internal env vars stay derived.
- Inventory stays one-file-per-backend with filename-to-id consistency enforced before adapter dispatch.
- One selected backend id resolves to one integrated adapter binding.
- Capability gating happens before runtime config render and process launch.
- Auth handoff resolution completes before runtime launch.
- Runtime config rendering completes before readiness probing.
- Restart reuses the same selected-backend realization path as sync.
- Linux, macOS, and Windows share one operator-facing command family and one selected-backend contract even when transport details diverge under the hood.

## Follow-ups for downstream seam planning and decomposition

- Pin the first supported non-`cli:codex` integrated backend id used for fixtures, schema examples, smoke scripts, and manual validation.
- Pin one deterministic classification for a missing integrated adapter binding. Current inputs reference both unsupported selection and dependency unavailable.
- Pin one deterministic classification for missing auth handoff material after policy permits the read path.
- Pin the auth handoff delivery rule into the integrated runtime: env-only, file-only, or a fixed mixed model with explicit precedence.
- Pin the exact backend inventory roots and filename rules in `filesystem-semantics-spec.md`, then mirror the same wording in `docs/CONFIGURATION.md`.
- Confirm whether any feature-local contract wording must restate the existing non-secret operator-facing wiring env outputs, or whether linking to the existing operator contract is sufficient.

## Draft downstream seam skeleton (pre-planning only)

draft; may split/merge during downstream FSE planning or decomposition

Draft seam prefix: GBSRI

### Seam 1
- `draft_seam_id`: `GBSRI-01`
- `name`: `backend-selection-and-policy`
- `intent`: Freeze the feature-local contract, policy flow, and env-var boundary for selected-backend realization without changing the stable command family.
- `likely owned or touched surfaces`: `contract.md`, `policy-spec.md`, `env-vars-spec.md`, `crates/shell/src/builtins/world_gateway.rs`, `crates/shell/tests/world_gateway.rs`, `crates/shell/src/execution/config_model.rs`, `crates/shell/src/execution/policy_model.rs`, `crates/broker`

### Seam 2
- `draft_seam_id`: `GBSRI-02`
- `name`: `runtime-realization-and-artifacts`
- `intent`: Freeze the integrated adapter binding flow, capability gating, auth handoff resolution, runtime config rendering, and filesystem artifact rules for the selected backend.
- `likely owned or touched surfaces`: `gateway-runtime-adapter-protocol-spec.md`, `gateway-runtime-adapter-schema-spec.md`, `filesystem-semantics-spec.md`, `crates/world-service/src/gateway_runtime.rs`, `crates/world-service/src/service.rs`, `crates/transport-api-types/src/lib.rs`, `crates/gateway/src/auth/`, `crates/gateway/src/providers/`, `crates/gateway/tests/`

### Seam 3
- `draft_seam_id`: `GBSRI-03`
- `name`: `parity-validation-and-rollout`
- `intent`: Freeze parity guarantees, compatibility promises, validation evidence, smoke assertions, and rollout proof for `cli:codex` plus one additional integrated backend.
- `likely owned or touched surfaces`: `platform-parity-spec.md`, `compatibility-spec.md`, `manual_testing_playbook.md`, `docs/USAGE.md`, `docs/CONFIGURATION.md`, `crates/world-service/tests/gateway_runtime_parity.rs`, `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/smoke/linux-smoke.sh`, `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/smoke/macos-smoke.sh`, `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/smoke/windows-smoke.ps1`

`ci_checkpoint_plan.md` may use this draft seam list when proposing checkpoint groups.

`workstream_triage.md` may recommend edits to this skeleton, but it does not own this file.
