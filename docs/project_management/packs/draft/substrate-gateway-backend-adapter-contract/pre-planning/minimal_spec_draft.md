**Warning: Pre-Planning Only. This document will be superseded by downstream FSE planning or decomposition.**

# substrate-gateway-backend-adapter-contract minimal spec draft

## Scope and authority

This draft defines only the pack-level alignment backbone for ADR-0041 and the pre-planning inputs under `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/`.

This draft defines:

- cross-cutting defaults
- precedence and authority boundaries
- security and failure invariants
- seam boundaries for downstream planning
- unresolved choices that block deterministic downstream planning

This draft does not define:

- execution tasks
- kickoff prompts
- runtime worktree ownership
- detailed implementation sequencing
- implementation checklists

Authority boundaries for this feature:

- `contract.md` is the downstream single owner for the user-facing adapter contract inside this pack.
- `policy-spec.md` is the downstream single owner for adapter selection, allowlist gating, and trusted-input boundaries.
- `gateway-backend-adapter-protocol-spec.md` and `gateway-backend-adapter-schema-spec.md` are the downstream single owners for adapter lifecycle, payload, capability, error, and session-handle boundary details.
- `platform-parity-spec.md` and `compatibility-spec.md` are the downstream single owners for parity and rollout guarantees.
- `docs/contracts/substrate-gateway-operator-contract.md` remains the owner for the `substrate world gateway status`, `sync`, and `restart` command family.
- `docs/contracts/substrate-gateway-status-schema.md` remains the owner for the existing `status --json` envelope and existing `client_wiring.*` family.
- `docs/contracts/substrate-gateway-policy-evaluation.md` remains the owner for gateway/world placement evaluation and host-to-world secret delivery posture.
- `docs/contracts/substrate-gateway-runtime-parity.md` remains the owner for the general gateway lifecycle runtime-parity surface.
- ADR-0027 and the implemented `llm_and_agent_config_policy_surface` pack remain the owners for backend-id grammar, inventory locations, config roots, policy roots, and allowlist storage.
- ADR-0017 remains the owner for structured event envelope semantics.
- ADR-0028 remains the owner for canonical trace vocabulary and correlation semantics.

## Defaults and precedence

Source-of-truth inputs for this feature:

- ADR basis: `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- Pre-planning authorities: `pre-planning/spec_manifest.md` and `pre-planning/impact_map.md`
- Config and policy authorities:
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`

Precedence posture for this draft:

1. Existing `substrate world gateway ...` CLI invocation selects the operator action only. ADR-0041 adds no new CLI flags for adapter selection.
2. Backend selection reads the existing ADR-0027 config surfaces:
   - `$SUBSTRATE_HOME/config.yaml`
   - `<workspace_root>/.substrate/workspace.yaml`
3. Adapter authorization and execution gating read the existing ADR-0027 policy surfaces:
   - `$SUBSTRATE_HOME/policy.yaml`
   - `<workspace_root>/.substrate/policy.yaml`
4. Backend inventory lookup uses the existing file-based one-file-per-backend inventory model and the existing `<kind>:<name>` filename-to-id matching rule.
5. Existing wiring env vars remain output surfaces only:
   - `SUBSTRATE_LLM_OPENAI_BASE_URL`
   - `SUBSTRATE_LLM_ANTHROPIC_BASE_URL`

Precedence rules that downstream docs must preserve:

- No new adapter-selection env var exists in this feature.
- No gateway-local admin, config, persistence, or session state becomes a new precedence layer for authorization.
- Stable backend selection stays expressed as one `<kind>:<name>` id.
- Policy gating executes before adapter dispatch.
- Local adapter docs define adapter-specific behavior only after the external config, policy, operator, event, and trace owners are acknowledged.

## Failure posture and invariants

Failure posture:

- Adapter-backed execution is fail-closed.
- Selection fails with a contract error when the selected backend id is invalid.
- Selection fails with a dependency-unavailable posture when the required adapter component is unavailable.
- Selection fails with a safety or policy posture when the backend id is denied by policy.
- Capability validation fails closed when the adapter cannot satisfy a required capability or required extension-key contract.
- World-required execution remains fail-closed when policy demands in-world routing.

Security and boundary invariants:

- Stable backend ids remain the only Substrate-facing backend identity.
- One backend id maps to one adapter contract identity.
- Substrate-owned surfaces do not split backend identity into planner, executor, provider, router, wrapper, or auth-authority sub-identities.
- Secrets do not appear in backend identity fields.
- Secrets do not appear in operator-visible status output.
- Session handles, adapter-private mechanics, provider quirks, and prompt-shaping internals stay inside `substrate-gateway`.
- Gateway-local admin mutation, token persistence, config mutation, and session storage do not authorize execution.
- Structured event envelopes remain owned by ADR-0017.
- Canonical trace vocabulary remains owned by ADR-0028.
- This feature does not introduce a second Substrate control plane.

## Exit-code posture

- Canonical taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- ADR-0041 already maps the needed buckets:
  - `0`: success
  - `2`: invalid configuration, invalid policy, or invalid adapter selection
  - `3`: transient runtime failure
  - `4`: required gateway or backend component unavailable
  - `5`: policy or safety failure
- This work does not require new exit codes.

## Cross-cutting seams and constraints

Shared constraints for every downstream doc:

- Preserve the stable backend-id grammar: `<kind>:<name>`.
- Preserve the deny-by-default posture of `llm.allowed_backends`.
- Preserve the one-file-per-backend inventory rule and filename-to-id matching rule from ADR-0027 surfaces.
- Preserve the command family ordering `substrate world gateway status`, `substrate world gateway sync`, and `substrate world gateway restart`.
- Keep `cli:codex` as the first required backend adapter.
- Keep future `cli:*` and `api:*` adapters additive under the same backend-id contract.
- Keep tuple fields such as `client`, `router`, `provider`, `auth_authority`, and `protocol` outside new local schema ownership in this pack.
- Keep additive `status --json` changes blocked until one owner is assigned for each new field family.
- Keep local event-translation rules subordinate to ADR-0017 envelope ownership.
- Keep local telemetry and correlation references subordinate to ADR-0028 trace ownership.
- Keep the implemented ADR-0027 pack path authoritative in this checkout.

Downstream seam boundaries:

- Selection boundary:
  - covers stable backend identity, config and policy input usage, allowlist gating, invalid-selection classification, unavailable-adapter classification, and the ban on gateway-local authorization inputs
- Protocol and schema boundary:
  - covers adapter registry lookup, request normalization, response translation, capability advertisement, error shapes, extension-key subset, and the session-handle boundary
- Parity and validation boundary:
  - covers Linux, macOS, and Windows parity guarantees, ADR-0024 supersession posture, compatibility proof, and validation evidence

## Follow-ups for downstream seam planning and decomposition

- Assign the owning document for any additive adapter metadata on `substrate world gateway status --json` before local schema or runtime surfaces widen.
- Pin the adopted Unified Agent API subset in `gateway-backend-adapter-schema-spec.md`: capability ids, extension keys, session-handle facet fields, and bounded error detail.
- Pin the exact capability-validation order and dispatch ordering in `gateway-backend-adapter-protocol-spec.md`.
- Record the exact boundary line between local adapter event translation and ADR-0017 event-envelope ownership.
- Record the exact boundary line between local adapter telemetry references and ADR-0028 trace ownership.
- Record the ADR path drift from `packs/active/llm_and_agent_config_policy_surface/*` to `packs/implemented/llm_and_agent_config_policy_surface/*` in downstream authored docs and ADR cleanup work.
- Confirm the exact compatibility proof that demonstrates ADR-0024 is historical evidence only and no second Substrate control plane exists.
- Confirm whether adjacent ADR-0040 planning-pack docs remain evidence-only or enter the downstream touch set as direct alignment edits.

## Draft downstream seam skeleton (pre-planning only)

Draft seam prefix: `SGBA`

Disclaimer: `draft; may split/merge during downstream FSE planning or decomposition`

Baseline seam count from `spec_manifest.md`: 3 draft seams. This draft keeps that baseline unchanged.

### Seam 1

- `draft_seam_id`: `SGBA-01`
- `name`: `adapter-selection-boundary`
- `intent`: Define the stable backend-identity boundary, reuse ADR-0027 config and policy inputs, enforce allowlist gating before dispatch, and keep adapter-private control surfaces outside Substrate-owned contract space.
- `likely owned or touched surfaces`:
  - `contract.md`
  - `policy-spec.md`
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
  - `docs/contracts/substrate-gateway-operator-contract.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`

### Seam 2

- `draft_seam_id`: `SGBA-02`
- `name`: `adapter-protocol-and-schema`
- `intent`: Define adapter lookup, request and response translation order, capability and extension-key contract, adapter error shape, and the session-handle boundary while preserving ADR-0017 and ADR-0028 ownership lines.
- `likely owned or touched surfaces`:
  - `gateway-backend-adapter-protocol-spec.md`
  - `gateway-backend-adapter-schema-spec.md`
  - `contract.md`
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - `docs/contracts/substrate-gateway-status-schema.md`

### Seam 3

- `draft_seam_id`: `SGBA-03`
- `name`: `parity-and-validation`
- `intent`: Define cross-platform invariants, compatibility and supersession proof, checkpoint intent, and the validation evidence for Linux, macOS, and Windows adapter-backed execution.
- `likely owned or touched surfaces`:
  - `platform-parity-spec.md`
  - `compatibility-spec.md`
  - `manual_testing_playbook.md`
  - `pre-planning/ci_checkpoint_plan.md`
  - `docs/contracts/substrate-gateway-runtime-parity.md`
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
