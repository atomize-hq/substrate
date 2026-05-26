**Warning: Pre-Planning Only. This document will be superseded by full planning.**

# llm-and-agent-identity-tuple-and-deployment-posture — minimal spec draft

## Scope and authority

This draft defines the pack-level defaults, invariants, and draft slice skeleton for ADR-0042 under `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/`.

This draft defines:
- cross-cutting defaults for identity-tuple vocabulary and placement-posture vocabulary
- reused precedence and authority boundaries from ADR-0027 and the existing gateway contract docs
- fail-closed, security, and redaction invariants that every later planning artifact preserves
- the draft slice skeleton that full planning expands into plan, task, and slice-spec artifacts
- explicit follow-ups that block deterministic full planning

This draft does not define:
- execution tasks
- kickoff prompts
- worktree ownership
- implementation patch order
- closeout checklists

Authority boundaries for this feature:
- `contract.md` owns the operator-visible meaning of `client`, `router`, `provider`, `auth_authority`, `protocol`, `in_world`, `host_only`, and `host_to_world_bridge`, plus additive human-readable status wording and the illustrative-only rule for example auth paths.
- `identity-tuple-schema-spec.md` owns the machine-readable tuple object, placement-posture object, canonical token grammar, and absence semantics.
- `policy-spec.md` owns routing-hint evaluation, direct-provider permission boundaries, reuse of existing ADR-0027 keys, and the bridge transport-only rule at the policy layer.
- `telemetry-spec.md` owns additive tuple and posture publication in status, diagnostics, and trace, plus redaction rules and consumer-impact notes.
- `platform-parity-spec.md` owns Linux, macOS, and Windows parity guarantees and the rule that `host_to_world_bridge` does not change in-world `net_allowed` governance.
- `compatibility-spec.md` owns overloaded-backend-label retirement posture and rollout proof for new docs and diagnostics.
- `manual_testing_playbook.md` owns deterministic doc-alignment validation and example-based review.
- `docs/contracts/gateway/operator-contract.md` remains the owner for the existing gateway command family.
- `docs/contracts/gateway/status-schema.md` remains the owner for the published `status --json` envelope and `client_wiring.*` field family.
- `docs/contracts/gateway/policy-evaluation.md` remains the owner for gateway placement evaluation, fail-closed routing semantics, and host-to-world secret-delivery posture.
- `docs/contracts/gateway/runtime-parity.md` remains the owner for general gateway lifecycle parity.
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md` and `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md` remain the owners for config roots, policy roots, key paths, precedence, and backend-id grammar.
- `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md` remains the owner for canonical trace vocabulary and correlation keys.
- `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md` remains the owner for tuple-axis policy keys under `llm.constraints.*`.

## Defaults and precedence

Source-of-truth inputs for this feature:
- ADR basis: `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- Pre-planning authorities:
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/impact_map.md`
- Config and policy authorities:
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`

Adopted default vocabulary:
- identity tuple fields: `client`, `router`, `provider`, `auth_authority`, `protocol`
- placement-posture tokens: `in_world`, `host_only`, `host_to_world_bridge`
- token grammar baseline:
  - `client`, `router`, `provider`, and `auth_authority` use normalized lowercase snake_case ids
  - `protocol` uses normalized lowercase dotted ids
- `backend_id` remains an adapter/runtime selector and does not substitute for tuple fields

Effective config precedence reused from the implemented ADR-0027 contract:
1. Existing CLI flags apply only to the underlying world-related command surface. ADR-0042 adds no new CLI flags for tuple or posture semantics.
2. Workspace config patch at `<workspace_root>/.substrate/workspace.yaml`
3. `SUBSTRATE_OVERRIDE_*` environment overrides when the command is not inside an enabled workspace
4. Global config patch at `$SUBSTRATE_HOME/config.yaml`
5. Built-in defaults

Effective policy precedence reused from the implemented ADR-0027 contract:
1. Workspace policy patch at `<workspace_root>/.substrate/policy.yaml`
2. Global policy patch at `$SUBSTRATE_HOME/policy.yaml`
3. Built-in defaults

Precedence rules downstream docs preserve:
- Tuple and posture semantics are interpreted from the effective ADR-0027 config and policy result.
- This feature adds no new config files, policy files, env vars, or CLI override channels.
- Existing wiring env vars and example credential paths remain non-authoritative inputs unless an external owner already defines them.
- `llm.gateway.mode`, `llm.fail_closed.routing`, `llm.secrets.env_allowed`, and `agents.host_credentials.read.allowed_backends` remain the only relevant existing key paths in this lane.
- `status --json`, diagnostics, and trace publication remain additive layers over existing owners rather than replacement authorities.

## Failure posture and invariants

Failure posture:
- This lane remains fail-closed.
- World-first placement remains the default posture.
- `host_only` execution exists only when effective policy permits it.
- Rejected routing hints do not rewrite `client`, do not create implicit provider authority, and do not bypass policy.
- `host_to_world_bridge` remains transport-only and does not create a second standing router or control plane.
- Status and trace publication stay additive. Existing owners for `client_wiring.*`, the top-level status envelope, and canonical trace correlation keys remain intact.

Security and redaction invariants:
- No secrets appear in tuple fields, placement-posture fields, status output, diagnostics, or trace by default.
- `auth_authority` remains distinct from `client` and `provider`.
- `protocol` remains capability metadata and does not carry routing authority.
- `backend_id` remains separate from tuple meaning.
- Additive status fields for tuple metadata stay outside `client_wiring.*`.
- Additive trace fields augment canonical ADR-0028 correlation vocabulary and do not replace it.
- `client` is not inferred from `provider`.
- `provider` is not inferred from `auth_authority`.
- `router` is not inferred from placement posture alone.
- Platform-specific transport details stay outside the public tuple and posture contract.
- Example auth paths such as `~/.codex/auth.json` remain illustrative and do not become new Substrate-owned filesystem contracts.

## Exit-code posture

- Canonical taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- Existing gateway commands keep the existing exit-taxonomy family. ADR-0042 introduces no new commands and no new exit codes.
- Current posture remains:
  - `0`: success or explicit no-op on the existing gateway command family
  - `2`: invalid usage or invalid config or policy on the existing gateway command family
  - `3`: required dependency unavailable on the existing gateway command family
  - `4`: unsupported or prerequisite-missing platform posture on the existing gateway command family
  - `5`: safety or policy violation on the existing gateway command family
- This work does not require new exit codes.

## Cross-cutting constraints

Shared constraints for every downstream doc:
- Preserve the adopted tuple vocabulary and placement-posture vocabulary from ADR-0042.
- Preserve the one-owner-per-surface rule from `pre-planning/spec_manifest.md`.
- Keep `backend_id` distinct from `client`, `router`, `provider`, `auth_authority`, and `protocol`.
- Keep additive machine-readable tuple and posture publication outside `client_wiring.*`.
- Keep `host_to_world_bridge` described as transport-only on every surface.
- Keep the existing gateway command family unchanged.
- Keep config-root, policy-root, key-path, and precedence ownership in the implemented ADR-0027 pack.
- Keep tuple-axis policy keys in ADR-0043 rather than this pack.
- Keep pure-agent tuple-publication follow-on work in ADR-0044 and toolbox-specific tuple-publication follow-on work in ADR-0045.
- Keep backend-selection realization work in ADR-0046 while this pack owns tuple and posture semantics.
- Keep Linux, macOS, and Windows operator-visible tuple semantics identical.

## Follow-ups for full planning

- Resolve the canonical router identity for host-only direct-provider fulfillment.
- Freeze the exact top-level status and diagnostic field families that carry tuple and posture metadata outside `client_wiring.*`.
- Freeze the exact trace field family and field placement that augment ADR-0028 correlation keys without replacing them.
- Resolve absence semantics for `provider` and `auth_authority` when routing-hint validation ends before provider selection.
- Resolve absence semantics for `provider` and `auth_authority` when agent-only or toolbox-adjacent flows reuse the same tuple vocabulary.
- Confirm the exact additive human-readable wording for tuple and posture display on the existing gateway status surfaces.
- Confirm the exact parity proof and validation evidence that show `host_to_world_bridge` leaves `net_allowed` governance unchanged across Linux, macOS, and Windows.

## Draft slice skeleton (pre-planning only)

Slice prefix (draft): `LAITDP`

Accepted draft slice count: `3`

- slice_id: `LAITDP0`
  - name: `identity_contract_and_schema`
  - intent: lock operator wording, tuple vocabulary, placement-posture vocabulary, machine-readable tuple-shape boundaries, and illustrative-path rules without reopening existing gateway status-envelope ownership
  - likely owned or touched surfaces:
    - `contract.md`
    - `identity-tuple-schema-spec.md`
    - `docs/contracts/gateway/operator-contract.md`
    - `docs/contracts/gateway/status-schema.md`
    - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
    - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`

- slice_id: `LAITDP1`
  - name: `policy_and_observability_alignment`
  - intent: align routing-hint evaluation, direct-provider permission boundaries, tuple-publication field families, redaction rules, and trace and status owner boundaries without reopening ADR-0027 key ownership or ADR-0043 tuple-axis policy ownership
  - likely owned or touched surfaces:
    - `policy-spec.md`
    - `telemetry-spec.md`
    - `docs/contracts/gateway/policy-evaluation.md`
    - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
    - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
    - `docs/contracts/gateway/status-schema.md`
    - `docs/TRACE.md`

- slice_id: `LAITDP2`
  - name: `platform_rollout_and_validation`
  - intent: lock platform parity, terminology rollout, compatibility proof, validation evidence, and bridge transport invariants across Linux, macOS, and Windows
  - likely owned or touched surfaces:
    - `platform-parity-spec.md`
    - `compatibility-spec.md`
    - `manual_testing_playbook.md`
    - `pre-planning/ci_checkpoint_plan.md`
    - `docs/contracts/gateway/runtime-parity.md`
    - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
    - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`

`ci_checkpoint_plan.md` groups checkpoints around this draft slice spine.

`workstream_triage.md` may recommend edits to this skeleton, but it does not own this file.
