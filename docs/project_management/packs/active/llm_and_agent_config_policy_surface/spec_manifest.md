# llm_and_agent_config_policy_surface — spec manifest

This file enumerates every contract/protocol/schema/file-format surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:

- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs

- Feature directory: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
- Related/upstream contracts (inputs; not owned by this feature pack):
  - `docs/project_management/adrs/implemented/ADR-0005-workspace-config-precedence-over-env.md`
  - `docs/project_management/adrs/implemented/ADR-0006-env-var-taxonomy-and-override-split.md`
  - `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
  - `docs/project_management/adrs/implemented/ADR-0012-config-schema-per-key-merge-and-provenance.md`
  - `docs/project_management/adrs/implemented/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`
  - `docs/project_management/adrs/draft/ADR-0020-profiles-config-policy-snapshots.md`
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md` (operator-facing tuple semantics)
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md` (additive `llm.constraints.*` tuple-axis policy surface)
  - `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md` (Phase 8 additive keys: `workflow.router.*`)

## Required spec documents (authoritative)

Spec templates:

- `docs/project_management/system/templates/spec/`

- `docs/project_management/packs/active/llm_and_agent_config_policy_surface/spec_manifest.md` — spec selection + ownership map (this file)
- `docs/project_management/packs/active/llm_and_agent_config_policy_surface/impact_map.md` — touch set + cascading implications + cross-queue scan
- `docs/project_management/packs/active/llm_and_agent_config_policy_surface/contract.md` — operator-facing contract (files, precedence summary, invariants, examples)
- `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md` — authoritative schema for:
  - config/policy key paths + types + defaults + constraints, and
  - agent inventory file format (agents/<agent_id>.yaml) + policy_overlay rules
- `docs/project_management/packs/active/llm_and_agent_config_policy_surface/decision_register.md` — A/B decisions and selections
- `docs/project_management/packs/active/llm_and_agent_config_policy_surface/LACP0-spec.md` — Phase 3a slice spec (config/policy strict schema + dotted updates + explain)
- `docs/project_management/packs/active/llm_and_agent_config_policy_surface/LACP1-spec.md` — Phase 3b slice spec (agent inventory strict parsing + restriction-only overlays)
- Validation artifacts (authoritative; required by ADR-0027):
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/manual_testing_playbook.md`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/smoke/linux-smoke.sh`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/smoke/macos-smoke.sh`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/smoke/windows-smoke.ps1`
- Planning Pack artifacts (required before execution triads begin; created by `docs/project_management/system/standards/planning/PLANNING_README.md`):
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/plan.md`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/ci_checkpoint_plan.md`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/tasks.json`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/kickoff_prompts/`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/session_log.md`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/quality_gate_report.md`

## Coverage matrix (surface → authoritative doc)

| Surface                                                                                           | Authoritative doc                                                                                     | What is explicitly defined                                                                            |
| ------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| Config patch file locations + effective precedence                                                | `docs/project_management/packs/active/llm_and_agent_config_policy_surface/contract.md`                | paths, precedence summary, workspace-vs-env override rules                                            |
| Policy patch file locations + effective precedence                                                | `docs/project_management/packs/active/llm_and_agent_config_policy_surface/contract.md`                | paths, precedence summary, workspace-vs-global rules                                                  |
| Exit code posture for schema violations                                                           | `docs/project_management/packs/active/llm_and_agent_config_policy_surface/contract.md`                | mapping to taxonomy; exit `2` for unknown keys / invalid values                                       |
| New config key families (`llm.*`, `agents.*`)                                                     | `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md`                  | key paths, types/enums, defaults, constraints (`llm.gateway.mode` vs policy)                          |
| New policy key families (`llm.*`, `agents.*`, `workflow.router.*`)                                | `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md`                  | key paths, types/enums, defaults, deny-by-default allowlists, constraints                             |
| Backend id format (`<kind>:<name>`)                                                               | `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md`                  | grammar, examples, where it applies                                                                   |
| Tuple semantics (`client`, `router`, `provider`, `auth_authority`, `protocol`)                   | `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md` | operator-facing meaning of tuple fields; deployment posture; tuple-vs-backend separation              |
| Tuple-axis policy constraints (`llm.constraints.*`)                                               | `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`              | additive policy keys for `router`, `provider`, `protocol`, and `auth_authority`                       |
| Agent inventory directory model                                                                   | `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md`                  | file locations, precedence rules, strictness rules, “no secrets” invariant                            |
| Embedded per-agent `policy_overlay`                                                               | `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md`                  | allowed key subset + restriction-only composition rules and error posture                             |
| Operator-facing invariants (fail-closed defaults; deny-by-default allowlists; no secrets in YAML) | `docs/project_management/packs/active/llm_and_agent_config_policy_surface/contract.md`                | enable/allowlist interplay, fail-closed routing invariants, examples                                  |
| Phase 3a acceptance criteria (config/policy strict schema)                                        | `docs/project_management/packs/active/llm_and_agent_config_policy_surface/LACP0-spec.md`              | testable behavior requirements for patch parsing, dotted updates, explain surfaces                    |
| Phase 3b acceptance criteria (agent inventory + overlays)                                         | `docs/project_management/packs/active/llm_and_agent_config_policy_surface/LACP1-spec.md`              | testable behavior requirements for inventory strictness, overlay broadening rejection, validation CLI |
| Manual validation                                                                                 | `docs/project_management/packs/active/llm_and_agent_config_policy_surface/manual_testing_playbook.md` | deterministic manual cases and expected outcomes                                                      |
| Automation smoke validation                                                                       | `docs/project_management/packs/active/llm_and_agent_config_policy_surface/smoke/*`                    | cross-platform smoke mirrors key schema/strictness cases                                              |
| Cross-platform CI cadence + checkpoint boundaries                                                 | `docs/project_management/packs/active/llm_and_agent_config_policy_surface/ci_checkpoint_plan.md`      | which gates run at checkpoints, slice grouping, and task id wiring                                    |

## Determinism checklist (must be satisfied before quality gate)

For the docs above, confirm they explicitly define:

- Inputs and precedence (config: CLI/workspace/env/global/default; policy: workspace/global/default).
- Defaults and absence semantics (disabled-by-default config; deny-by-default allowlists; empty strings).
- Data model and strictness for every serialized boundary (config patch, policy patch, agent inventory files).
- Error model and failure posture (exit `2` for schema violations; fail-closed routing invariants).
- Security/redaction invariants (no secret values in YAML; names-only allowlists for secret sourcing).
- Platform guarantees (Linux/macOS parity for file shapes and key paths for this Planning Pack).
- Ownership boundaries versus additive follow-ons (backend ids here; tuple semantics in ADR-0042; tuple-axis policy in ADR-0043).
