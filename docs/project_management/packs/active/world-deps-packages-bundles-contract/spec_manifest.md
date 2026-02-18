# world-deps-packages-bundles-contract — spec manifest

This file enumerates every contract/protocol/schema/env-var surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/active/world-deps-packages-bundles-contract`
- ADR(s) / upstream contracts (inputs; not owned by this Planning Pack):
  - `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md`
  - `docs/project_management/next/world_deps_packages_bundles_contract.md`
  - `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
  - `docs/project_management/adrs/implemented/ADR-0012-config-schema-per-key-merge-and-provenance.md`
  - `docs/project_management/adrs/implemented/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`
  - `docs/project_management/adrs/implemented/ADR-0014-world-agent-policy-resolution-and-concurrency.md`
  - `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

## Required spec documents (authoritative)

List the exact spec documents that must exist. These documents are owned by this Planning Pack and are the authoritative execution surfaces for slicing, validation, and cross-platform cadence.

- `docs/project_management/packs/active/world-deps-packages-bundles-contract/spec_manifest.md` — spec selection + ownership map (this file)
- `docs/project_management/packs/active/world-deps-packages-bundles-contract/impact_map.md` — touch set + cascading implications + cross-queue conflicts
- `docs/project_management/packs/active/world-deps-packages-bundles-contract/plan.md` — execution runbook + sequencing overview
- `docs/project_management/packs/active/world-deps-packages-bundles-contract/tasks.json` — triad task graph + acceptance criteria
- `docs/project_management/packs/active/world-deps-packages-bundles-contract/session_log.md` — planning/execution evidence log
- `docs/project_management/packs/active/world-deps-packages-bundles-contract/decision_register.md` — decisions required to execute ADR-0011 without ambiguity
- `docs/project_management/packs/active/world-deps-packages-bundles-contract/platform-parity-spec.md` — platform guarantees + validation evidence requirements
- `docs/project_management/packs/active/world-deps-packages-bundles-contract/manual_testing_playbook.md` — human validation workflow (authoritative)
- `docs/project_management/packs/active/world-deps-packages-bundles-contract/ci_checkpoint_plan.md` — bounded CI cadence (authoritative)
- `docs/project_management/packs/active/world-deps-packages-bundles-contract/quality_gate_report.md` — planning pack quality gate (required before execution)
- `docs/project_management/packs/active/world-deps-packages-bundles-contract/execution_preflight_report.md` — execution preflight gate (required before WDP0)
- Slice specs:
  - `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP0-spec.md`
  - `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP1-spec.md`
  - `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP2-spec.md`
  - `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP3-spec.md`
  - `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP4-spec.md`
  - `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP5-spec.md`
- Slice closeout reports (required by tasks.json meta.execution_gates=true):
  - `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP0-closeout_report.md`
  - `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP1-closeout_report.md`
  - `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP2-closeout_report.md`
  - `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP3-closeout_report.md`
  - `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP4-closeout_report.md`
  - `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP5-closeout_report.md`

## Coverage matrix (surface → authoritative doc)

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| CLI commands/flags/defaults | `docs/project_management/next/world_deps_packages_bundles_contract.md` | names, defaults, examples, exit codes |
| Config file paths/precedence/keys | `docs/project_management/next/world_deps_packages_bundles_contract.md` | file paths, precedence, key set + defaults |
| Inventory directory layout + schema | `docs/project_management/next/world_deps_packages_bundles_contract.md` | file layout, item schema, merge semantics |
| World shell contract for “runnable” deps | `docs/project_management/next/world_deps_packages_bundles_contract.md` | interactive vs non-interactive evaluator, no rcfiles |
| World status semantics (`present|missing|blocked`) | `docs/project_management/next/world_deps_packages_bundles_contract.md` | probe/blocked rules, exit codes |
| Cross-platform support/unsupported contract | `docs/project_management/packs/active/world-deps-packages-bundles-contract/platform-parity-spec.md` | guaranteed behavior, fail-closed posture |
| Slicing + acceptance criteria per increment | `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP*-spec.md` | slice scope, explicit unsupported surfaces, acceptance |
| CI cadence + checkpoint boundaries | `docs/project_management/packs/active/world-deps-packages-bundles-contract/ci_checkpoint_plan.md` | checkpoint groups, gates, rationale |
| Manual validation workflow | `docs/project_management/packs/active/world-deps-packages-bundles-contract/manual_testing_playbook.md` | commands, assertions, exit codes |

## Determinism checklist (must be satisfied before quality gate)

For every selected spec document, confirm it explicitly defines:
- Inputs (all) + precedence order (if multiple inputs exist)
- Defaults (all) + absence semantics
- Data model (types/constraints) for every serialized boundary
- Error model (exit codes, error messages where applicable) and failure posture
- Ordering/atomicity/concurrency rules (if any)
- Security/redaction invariants (if any)
- Platform guarantees (Linux/macOS/WSL as applicable)
