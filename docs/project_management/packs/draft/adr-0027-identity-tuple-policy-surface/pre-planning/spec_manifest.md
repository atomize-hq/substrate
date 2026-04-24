# adr-0027-identity-tuple-policy-surface — spec manifest (pre-planning)

This file enumerates every contract, schema, policy-evaluation, telemetry, compatibility, validation, and slice-planning surface required for ADR-0043 and assigns each touched surface to exactly one authoritative document.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `docs/project_management/system/standards/adr/ADR_STANDARD_AND_TEMPLATE.md`
- `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/`
- ADRs:
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- External authoritative docs reused by this feature:
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

## Slice IDs (canonical)

Canonical slice ids selected for this feature:
- Slice prefix: `ITPS`
- `ITPS0` — publish the additive contract and schema lock for `llm.constraints.*`
- `ITPS1` — publish policy-evaluation ordering, deny taxonomy, and explain-surface closure
- `ITPS2` — publish telemetry publication and compatibility closure
- `ITPS3` — publish manual validation, CI checkpoint alignment, and promotion closure

## Required spec documents (authoritative)

### Current pre-planning artifact produced in this lane

- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/spec_manifest.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the required document set for ADR-0043
    - the one-owner-per-surface matrix
    - the explicit list of unselected doc classes
  - Must define:
    - every feature-local document required by this body of work
    - every reused external owner that remains authoritative for unchanged surfaces
    - every follow-up required to remove residual ambiguity before full planning promotion

### Topic-specific specs required by ADR-0043

- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - the additive operator-facing contract for existing config and policy inspection commands
    - the rendered visibility rules for the new tuple-axis keys on effective views and explain output
    - the feature-local exit-code mapping for tuple-axis schema errors and policy denials
    - the feature-local platform guarantee that Linux, macOS, and Windows expose the same tuple-axis policy semantics
  - Must define:
    - the exact commands and output surfaces changed by this feature
    - the exact stdout and stderr visibility rules for effective values and provenance
    - the exact exit-code meanings reused from the canonical taxonomy
    - the exact links back to the unchanged file-family and precedence owners in the implemented ADR-0027 pack

- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tuple-policy-schema-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - the YAML schema additions for `llm.constraints.routers`
    - the YAML schema additions for `llm.constraints.providers`
    - the YAML schema additions for `llm.constraints.protocols`
    - the YAML schema additions for `llm.constraints.auth_authorities`
    - the canonical token grammar for values carried by those keys
    - the rule that `client` remains outside the standalone policy-key surface in v1
  - Must define:
    - every key path, type, default, and absence rule
    - the empty-list semantics for each key
    - normalization and rejection rules for valid and invalid values
    - illustrative YAML payloads that show accepted and rejected shapes

- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/policy-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - the ordered evaluation flow that combines backend allowlists with the new tuple-axis constraints
    - the interaction between tuple-axis constraints and `llm.fail_closed.routing`
    - the interaction between tuple-axis constraints and the existing host secret-read gates
    - the interaction between tuple-axis constraints, `net_allowed`, and the transport-only `host_to_world_bridge`
    - the deny taxonomy for router, provider, protocol, and auth-authority mismatches
  - Must define:
    - the exact ordered inputs used during policy evaluation
    - the exact precedence between unchanged backend gates and the new tuple-axis narrowing gates
    - the exact failure posture for schema-invalid input, policy denial, missing world boundary, and disallowed host credential reads
    - the exact operator-explanation requirements for every deny path

- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/telemetry-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - the trace and log publication rules for tuple-axis policy evaluation outcomes
    - the field-level publication rules for router, provider, protocol, and auth-authority metadata on allow and deny records
    - the redaction and no-secret rules for those records
    - the augmentation boundary against the existing ADR-0028 trace vocabulary
  - Must define:
    - the exact field names or reused field families that appear on policy-evaluation records
    - the exact allow-versus-deny publication rules
    - the exact redaction and omission rules for secret-adjacent data
    - the exact collision-avoidance boundary against existing correlation keys

- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/compatibility-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - the additive rollout rule for existing operators that do not set `llm.constraints.*`
    - the rule that backend ids remain adapter gates and do not become tuple surrogates
    - the promotion rule that the new keys extend the implemented ADR-0027 contract and schema without creating a second config system
  - Must define:
    - the exact no-change behavior when the new keys are absent
    - the exact migration and promotion boundary into the implemented ADR-0027 pack
    - the exact invariants that future tuple-axis additions must preserve

- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md`
  - Role: topic-specific validation spec required by the ADR
  - Owns:
    - the deterministic manual review procedure for doc and contract alignment
    - the concrete inspection cases named by ADR-0043
    - the one-owner-per-surface validation checklist
  - Must define:
    - the exact commands and expected results for `substrate policy current show --explain` and `substrate policy current show --json --explain`
    - the exact commands and expected results for `smoke/linux-smoke.sh`, `smoke/macos-smoke.sh`, and `smoke/windows-smoke.ps1`
    - the exact deny-case assertions for disallowed router, provider, protocol, and auth-authority combinations
    - the exact review posture for the Codex example path `~/.codex/auth.json` as validation input only, not as a new Substrate-owned path contract

### Planning Pack artifacts required before execution

- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/plan.md`
  - Role: required Planning Pack artifact
  - Must define:
    - sequencing for `ITPS0`, `ITPS1`, `ITPS2`, and `ITPS3`
    - validation gates and doc dependencies
    - the rule that this feature remains documentation-driven until promotion completes

- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json`
  - Role: required Planning Pack artifact
  - Must define:
    - triad tasks aligned to `ITPS0`, `ITPS1`, `ITPS2`, and `ITPS3`
    - acceptance criteria tied to the selected local specs
    - deterministic branch, worktree, and validation wiring for downstream execution

- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS0/ITPS0-spec.md`
  - Role: canonical slice spec required by the standard
  - Must define:
    - the contract and schema publication work for the new tuple-axis keys
    - the acceptance criteria for operator-facing inspection output
    - the out-of-scope boundary that keeps unchanged ADR-0027 surfaces external

- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS1/ITPS1-spec.md`
  - Role: canonical slice spec required by the standard
  - Must define:
    - the policy-evaluation ordering, deny taxonomy, and explain-surface closure for the feature
    - the acceptance criteria for deny semantics and authoritative explain output
    - the out-of-scope boundary that keeps tuple semantics external to ADR-0042

- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS2/ITPS2-spec.md`
  - Role: canonical slice spec required by the standard
  - Must define:
    - the telemetry-publication and compatibility-closure work for the feature
    - the acceptance criteria for tuple-aware trace publication and additive rollout guarantees
    - the out-of-scope boundary that keeps policy-ordering semantics in `ITPS1`

- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS3/ITPS3-spec.md`
  - Role: canonical slice spec required by the standard
  - Must define:
    - the manual-validation, CI-checkpoint-alignment, and promotion-closure work for the feature
    - the acceptance criteria for cross-platform validation coverage and checkpoint wiring
    - the out-of-scope boundary that keeps telemetry-field ownership in `ITPS2`

## Coverage matrix (surface → authoritative doc)

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| Existing config patch file families `$SUBSTRATE_HOME/config.yaml` and `<workspace_root>/.substrate/workspace.yaml` | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md` | file locations, patch role, and precedence posture that remain unchanged |
| Existing policy patch file families `$SUBSTRATE_HOME/policy.yaml` and `<workspace_root>/.substrate/policy.yaml` | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md` | file locations, patch role, and precedence posture that remain unchanged |
| Existing config inspection command family `substrate config ...` | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md` | command family and baseline effective-view contract reused by this feature |
| Existing policy inspection command family `substrate policy ...` | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md` | command family and baseline effective-view contract reused by this feature |
| Existing key paths `llm.allowed_backends`, `agents.allowed_backends`, `llm.fail_closed.routing`, `agents.fail_closed.routing`, `llm.secrets.env_allowed`, `agents.host_credentials.read.allowed_backends`, and `net_allowed` | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md` | key paths, types, defaults, and unchanged storage surface |
| Semantic meaning of `router`, `provider`, `protocol`, and `auth_authority` | `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md` | normalized operator meaning of each tuple field |
| Semantic meaning of `host_to_world_bridge` as transport-only | `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md` | transport-only rule and no-second-control-plane invariant |
| New key paths `llm.constraints.routers`, `llm.constraints.providers`, `llm.constraints.protocols`, and `llm.constraints.auth_authorities` | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tuple-policy-schema-spec.md` | path names, types, examples, and YAML shape |
| Default `[]` values for the new key paths | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tuple-policy-schema-spec.md` | per-key default and absence semantics |
| Empty-list meaning for the new key paths | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tuple-policy-schema-spec.md` | unconstrained behavior when a list is empty |
| Token grammar for `router`, `provider`, and `auth_authority` values | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tuple-policy-schema-spec.md` | lowercase snake_case rules and rejection criteria |
| Token grammar for `protocol` values | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tuple-policy-schema-spec.md` | lowercase dotted-id rules and rejection criteria |
| Omission rule that `client` is not a standalone policy key in v1 | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tuple-policy-schema-spec.md` | explicit omission boundary and reserved-key posture |
| Ordered evaluation of backend allowlists and tuple-axis constraints | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/policy-spec.md` | ordered inputs, precedence, and failure classification |
| Interaction between tuple-axis constraints and `llm.fail_closed.routing` | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/policy-spec.md` | routing posture when the world boundary is unavailable |
| Interaction between tuple-axis constraints and `llm.secrets.env_allowed` | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/policy-spec.md` | host env-read gating during tuple-aware routing |
| Interaction between tuple-axis constraints and `agents.host_credentials.read.allowed_backends` | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/policy-spec.md` | host credential-read gating during tuple-aware routing |
| Interaction between tuple-axis constraints, `net_allowed`, and transport bridge usage | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/policy-spec.md` | egress-boundary invariants and transport-only bridge rule in policy evaluation |
| Deny taxonomy for router mismatch, provider mismatch, protocol mismatch, and auth-authority mismatch | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/policy-spec.md` | deny classes, error posture, and explanation requirements |
| Existing config inspection command family `substrate config ...` remains config-root inspection only on `substrate config show` and `substrate config show --explain` | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md` | config-root inspection remains available, but tuple-policy ownership and provenance do not move off the policy view |
| Effective merged policy view on `substrate policy current show` and `substrate policy current show --explain` is the sole authoritative tuple-policy surface | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md` | exact rendered output obligations and provenance visibility for `llm.constraints.*` |
| Exit-code reuse for invalid tuple-axis schema input | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md` | exact mapping to the canonical taxonomy for schema-invalid input |
| Exit-code reuse for tuple-axis policy denial | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md` | exact mapping to the canonical taxonomy for policy-denied execution |
| Cross-platform guarantee for identical tuple-axis policy semantics on Linux, macOS, and Windows | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md` | parity guarantee and validation reference |
| Trace publication of tuple-axis policy metadata on allow records | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/telemetry-spec.md` | exact field publication rules for allow outcomes |
| Trace publication of tuple-axis policy metadata on deny records | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/telemetry-spec.md` | exact field publication rules for deny outcomes |
| Log and trace redaction rules for auth-authority-adjacent data | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/telemetry-spec.md` | no-secret publication rule and omission behavior |
| Collision boundary against existing correlation keys and trace vocabulary | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/telemetry-spec.md` | exact augmentation boundary against existing trace fields |
| Existing trace vocabulary and correlation keys reused by this feature | `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md` | trace envelope, field vocabulary, and correlation semantics that remain unchanged |
| Additive rollout rule when operators do not set `llm.constraints.*` | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/compatibility-spec.md` | exact no-change posture when the new keys are absent |
| Rule that backend ids remain adapter gates rather than tuple surrogates | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/compatibility-spec.md` | exact compatibility boundary between existing backend ids and the new tuple-axis keys |
| Promotion boundary into the implemented ADR-0027 pack | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/compatibility-spec.md` | exact promotion target and anti-duplication rule |
| Manual validation case for Claude Code routed through `substrate_gateway` to multiple providers under policy control | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md` | exact commands, expected outcomes, and review assertions |
| Manual validation case for Codex plus Responses API with `~/.codex/auth.json` | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md` | exact commands, expected outcomes, and validation-only path handling |
| One-owner-per-surface proof for all selected local and external docs | `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md` | deterministic cross-doc review procedure and expected pass criteria |

## Determinism checklist

### `contract.md`
- Define the exact existing commands that surface the new tuple-axis keys.
- Define the exact output placement for effective values and provenance.
- Define the exact exit-code mapping reused from the canonical taxonomy.
- Define the exact platform guarantee and external links for unchanged file families and precedence.

### `tuple-policy-schema-spec.md`
- Define the exact YAML paths, types, defaults, and empty-list semantics.
- Define the exact normalized token grammar and rejection rules.
- Define the exact omission rule for `client`.
- Define accepted and rejected examples with no placeholder behavior.

### `policy-spec.md`
- Define the exact ordered evaluation inputs and precedence.
- Define the exact interaction with unchanged backend gates, fail-closed routing, host credential gates, and `net_allowed`.
- Define the exact deny classes and operator explanation requirements.
- Define the exact no-second-control-plane rule during tuple-aware evaluation.

### `telemetry-spec.md`
- Define the exact field families published on allow and deny records.
- Define the exact redaction and omission rules.
- Define the exact reuse boundary against ADR-0028 correlation keys.
- Define the exact publication scope for operator-visible tuple metadata.

### `compatibility-spec.md`
- Define the exact no-change behavior when the new keys are absent.
- Define the exact backend-id compatibility rule.
- Define the exact promotion target into the implemented ADR-0027 pack.
- Define the exact future-extension invariants for added tuple-axis keys.

### `manual_testing_playbook.md`
- Define the exact command matrix and expected outputs.
- Define the exact deny cases that must be exercised.
- Define the exact review posture for validation-only example paths.
- Define the exact one-owner-per-surface review checklist.

### `plan.md`
- Define the exact slice order and validation gates.
- Define the exact document dependency order.
- Define the exact promotion and closeout steps.

### `tasks.json`
- Define exact triad tasks for `ITPS0`, `ITPS1`, `ITPS2`, and `ITPS3`.
- Define exact acceptance criteria linked to the selected local specs.
- Define exact validation commands and task wiring.

### `slices/ITPS0/ITPS0-spec.md`
- Define the exact contract and schema publication scope.
- Define the exact acceptance criteria for explain-surface updates.
- Define the exact out-of-scope boundary for unchanged ADR-0027 surfaces.

### `slices/ITPS1/ITPS1-spec.md`
- Define the exact policy-evaluation ordering, deny taxonomy, and explain-surface scope.
- Define the exact acceptance criteria for deny semantics and authoritative explain output.
- Define the exact out-of-scope boundary for tuple semantics owned by ADR-0042.

### `slices/ITPS2/ITPS2-spec.md`
- Define the exact telemetry-publication and compatibility-closure scope.
- Define the exact acceptance criteria for tuple-aware trace publication and additive rollout behavior.
- Define the exact out-of-scope boundary for policy-ordering work owned by `ITPS1`.

### `slices/ITPS3/ITPS3-spec.md`
- Define the exact manual-validation, CI-checkpoint-alignment, and promotion-closure scope.
- Define the exact acceptance criteria for cross-platform validation coverage and checkpoint wiring.
- Define the exact out-of-scope boundary for telemetry-field ownership held by `ITPS2`.

## Explicitly unselected doc classes

No feature-local doc is selected for these classes:

- `env-vars-spec.md`
  - Reason: ADR-0043 does not add or change any environment-variable names. The feature only constrains the existing YAML key `llm.secrets.env_allowed`, which remains externally owned.

- `<topic>-protocol-spec.md`
  - Reason: ADR-0043 does not define a new host-to-agent, HTTP, WebSocket, named-pipe, or shim protocol surface. The ADR states that exact per-request hint headers and env-var names remain out of scope.

- `filesystem-semantics-spec.md`
  - Reason: ADR-0043 does not create a new filesystem contract. The only concrete path named by the ADR is `~/.codex/auth.json`, and this manifest assigns it to validation-only review in `manual_testing_playbook.md`.

- `platform-parity-spec.md`
  - Reason: ADR-0043 states one parity rule: Linux, macOS, and Windows expose the same tuple-axis policy semantics. `contract.md` owns that guarantee, and `manual_testing_playbook.md` owns the validation procedure.

## Follow-ups

- `telemetry-spec.md` must pin the exact field names or reused field families used for tuple-axis policy evaluation records. ADR-0043 names the metadata categories but does not name the record-level field shape.
- `contract.md` must pin the exact exit-code mapping and rendered explain output for tuple-axis policy denials. ADR-0043 reuses the canonical taxonomy but does not name the final rendered contract text.
