# world-deps-packages-bundles-contract — impact map

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/next/world-deps-packages-bundles-contract`
- ADR(s):
  - `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md`
- Contract:
  - `docs/project_management/next/world_deps_packages_bundles_contract.md`
- Spec manifest:
  - `docs/project_management/next/world-deps-packages-bundles-contract/spec_manifest.md`

## Touch set (explicit)

### Create
- `docs/project_management/next/world-deps-packages-bundles-contract/plan.md` — v4 plan and slice model
- `docs/project_management/next/world-deps-packages-bundles-contract/tasks.json` — v4 task graph (automation + checkpoints)
- `docs/project_management/next/world-deps-packages-bundles-contract/spec_manifest.md` — spec ownership map
- `docs/project_management/next/world-deps-packages-bundles-contract/impact_map.md` — touch set and cross-queue scan
- `docs/project_management/next/world-deps-packages-bundles-contract/decision_register.md` — decision capture
- `docs/project_management/next/world-deps-packages-bundles-contract/platform-parity-spec.md` — platform guarantees
- `docs/project_management/next/world-deps-packages-bundles-contract/manual_testing_playbook.md` — authoritative validation workflow
- `docs/project_management/next/world-deps-packages-bundles-contract/ci_checkpoint_plan.md` — bounded CI cadence
- `docs/project_management/next/world-deps-packages-bundles-contract/session_log.md` — evidence ledger
- `docs/project_management/next/world-deps-packages-bundles-contract/quality_gate_report.md` — planning quality gate artifact
- `docs/project_management/next/world-deps-packages-bundles-contract/execution_preflight_report.md` — execution preflight gate artifact
- `docs/project_management/next/world-deps-packages-bundles-contract/smoke/*` — Linux/macOS smoke scripts (plus WSL via Linux smoke dispatch with `RUN_WSL=1`)
- `docs/project_management/next/world-deps-packages-bundles-contract/kickoff_prompts/*` — triad kickoff prompts
- `docs/project_management/next/world-deps-packages-bundles-contract/WDP*-spec.md` — slice specs
- `docs/project_management/next/world-deps-packages-bundles-contract/WDP*-closeout_report.md` — slice closeout gate reports

### Edit
- `docs/project_management/next/sequencing.json` — add this feature directory as a sprint entry
- `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md` — link to this Planning Pack (spec manifest + tasks + decision register)
- `docs/project_management/next/world_deps_packages_bundles_contract.md` — align “World Shell Contract” wording with ADR-0016 and implemented REPL evaluator behavior

### Edit (execution scope; non-doc)
Execution triads under this Planning Pack are expected to edit:
- `crates/shell/src/builtins/world_deps/*` — replace legacy selection/manifest plumbing with inventory+enabled-patch contract
- `crates/common/src/world_deps_manifest.rs` — replace/extend manifest model to parse package/bundle inventory definitions
- `crates/world-agent/*` — implement in-world probe + install execution for packages/bundles (apt + script + manual blocked)
- Installer scripts that stage legacy world-deps files — stop copying/reading legacy paths

### Delete (execution scope; end state)
When execution completes, delete legacy world-deps selection/overlay plumbing so `world deps` cannot be influenced by old paths:
- legacy selection file semantics and paths
- legacy manager hooks / overlay file semantics for world-deps

## Cascading implications (behavior/UX)

### CLI / UX
- Change:
  - Replace legacy `substrate world deps status|init|select|provision` selection-file UX with `current|global|workspace` scoped inventory/enabled UX.
- Direct impact:
  - Operators can answer three questions deterministically:
    - what is available (inventory),
    - what is enabled (patch keys),
    - what is applied/present (world-backed status).
- Cascading impact:
  - Help output, docs, and smoke scripts must match the new subcommand tree.
  - Exit code meanings must remain aligned to `EXIT_CODE_TAXONOMY.md`.
- Contradiction risks:
  - Existing selection-file semantics and “tool” naming conflicts with packages/bundles naming; legacy must be removed from plumbing (tests enforce).

### Config / env vars / paths
- Change:
  - Enabled deps move to patch keys (`world.deps.*`) under `$SUBSTRATE_HOME/config.yaml` and `<workspace_root>/.substrate/workspace.yaml`.
- Direct impact:
  - Operators can use patch-file scopes consistently with `ADR-0008`.
- Cascading impact:
  - Shared config editor/merge engine must be used (no bespoke YAML mutation).
- Contradiction risks:
  - Any remaining legacy file reads create silent drift; replacement completeness tests are required.

### Policy / isolation / security posture
- Change:
  - World-backed operations (`applied`, `install`, `sync`) fail closed when the world backend is unavailable (exit `3`).
- Direct impact:
  - No silent host fallback for world-backed operations.
- Cascading impact:
  - Smoke scripts and integration tasks must validate backend-unavailable posture and messages.
- Contradiction risks:
  - Legacy world-deps flows that allow host fallback must not be reachable via the new CLI.

## Cross-queue scan (ADRs + Planning Packs)

### Relevant ADRs (cross-cutting constraints)
- ADR: `docs/project_management/adrs/implemented/ADR-0002-world-deps-install-classes-and-world-provisioning.md`
  - Overlap surfaces: world-deps install/provision semantics, install classes
  - Conflict: yes
  - Resolution (explicit): ADR-0011 is the authoritative end-state contract; execution removes legacy selection/manifest paths and replaces the CLI surface

- ADR: `docs/project_management/adrs/implemented/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`
  - Overlap surfaces: patch-only YAML semantics, broker-canonical effective resolution, workspace disable marker
  - Conflict: no
  - Resolution (explicit): `world deps` patch editing and effective views use the broker-owned patch merge rules and do not implement a bespoke merge/precedence engine

- ADR: `docs/project_management/adrs/implemented/ADR-0014-world-agent-policy-resolution-and-concurrency.md`
  - Overlap surfaces: world-agent concurrency safety, policy snapshot inputs to world-agent enforcement
  - Conflict: no
  - Resolution (explicit): world-backed `world deps` operations execute via host-provided policy snapshots (`PolicySnapshotV3`); legacy local policy resolution inside world-agent is not in scope for this Planning Pack

- ADR: `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`
  - Overlap surfaces: interactive REPL evaluator shell contract
  - Conflict: no
  - Resolution (explicit): world-deps runnable entrypoints must remain compatible with REPL evaluator semantics (no rcfiles)

- ADR: `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - Overlap surfaces: REPL output routing, PTY passthrough vs structured output
  - Conflict: no
  - Resolution (explicit): `world deps` user-visible output respects the REPL output routing contract and does not inject structured host output into PTY byte streams during passthrough

- ADR: `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
  - Overlap surfaces: policy schema invariants for full isolation, fail-closed posture on invalid policy inputs
  - Conflict: no
  - Resolution (explicit): `world deps` world-backed behavior remains compatible with policy snapshot + full-isolation enforcement posture and does not introduce any policy-bypass execution path

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/_archived/world_deps_selection_layer`
  - Overlap surfaces: world-deps selection model, file path semantics
  - Conflict: yes
  - Resolution (explicit): do not edit the archived Planning Pack; implement ADR-0011 under this new Planning Pack and enforce “legacy paths removed” via tests

## Follow-ups (explicit)
- ADR cross-link update required:
  - `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md` related docs list includes this Planning Pack entrypoints
