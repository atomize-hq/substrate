# world-deps-packages-bundles-contract — plan (v4)

## Scope
- Feature directory: `docs/project_management/next/world-deps-packages-bundles-contract`
- Orchestration branch: `feat/world-deps-packages-bundles-contract`
- Authoritative contract inputs (source of truth):
  - `docs/project_management/next/ADR-0011-world-deps-packages-bundles-contract.md`
  - `docs/project_management/next/world_deps_packages_bundles_contract.md`
- Upstream dependency ADRs (constraints; incorporated into slice acceptance criteria):
  - `docs/project_management/next/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md` (patch-only format + broker-canonical effective resolution)
  - `docs/project_management/next/ADR-0014-world-agent-policy-resolution-and-concurrency.md` (host-resolved policy snapshot input to world-agent)
  - `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md` (interactive REPL semantics + evaluator shell contract)
  - `docs/project_management/next/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md` (PTY bytes vs structured output routing in REPL)
  - `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md` (full isolation world-fs deny semantics + policy schema invariants)
- Authoritative spec ownership map: `docs/project_management/next/world-deps-packages-bundles-contract/spec_manifest.md`
- Authoritative impact map: `docs/project_management/next/world-deps-packages-bundles-contract/impact_map.md`
- Authoritative CI cadence: `docs/project_management/next/world-deps-packages-bundles-contract/ci_checkpoint_plan.md`

## Goal (operator-facing)
Ship the inventory + enabled-patch contract for `substrate world deps`:
- Inventory is a directory model (built-ins + `$SUBSTRATE_HOME/deps/` + `<workspace_root>/.substrate/deps/`).
- Enabled deps are YAML patch keys in:
  - `$SUBSTRATE_HOME/config.yaml` (global scope)
  - `<workspace_root>/.substrate/workspace.yaml` (workspace scope)
- CLI uses explicit scopes and current/effective views:
  - `substrate world deps current ...`
  - `substrate world deps global ...`
  - `substrate world deps workspace ...`

The authoritative UX/exit-code contract is the contract doc above; this plan defines the execution slicing and cross-platform cadence.

## Shell contract invariant (non-negotiable)
World execution is not an interactive login shell:
- Non-interactive world execution pathways run under `/bin/sh -c` with no user shell rc sourcing.
- Interactive `substrate>` REPL runs under the world-first persistent-session model and evaluates submissions under `/bin/bash --noprofile --norc -c` (no rcfiles).
- Therefore, runnable deps MUST expose real executable entrypoints (files) and MUST NOT rely on shell functions, aliases, or rcfile initialization.

## Legacy removal invariant (non-negotiable)
The new `substrate world deps` contract MUST NOT read (or be influenced by) any legacy world-deps paths listed in the contract doc (manager hooks, overlay, selection).

## Cross-platform execution model (v4; bounded checkpoints)
- Behavior platforms (feature smoke required): `linux, macos`
- CI parity platforms (compile parity required): `linux, macos`
- WSL coverage required: `true` (bundled into Linux smoke via `RUN_WSL=1`)
- Cross-platform CI dispatch occurs only at checkpoint boundaries in `ci_checkpoint_plan.md`.

## Execution gates (enabled)
- Planning quality gate must exist and be `ACCEPT` before any triad starts:
  - `docs/project_management/next/world-deps-packages-bundles-contract/quality_gate_report.md`
- Execution preflight task must be completed before any code/test triad starts:
  - Task: `F0-exec-preflight`
  - Report: `docs/project_management/next/world-deps-packages-bundles-contract/execution_preflight_report.md`

## Triads (authoritative slice list)
- Checkpoint group CP1 (boundary slice: `WDP2`):
  - `WDP0` — Inventory parsing + merged available views + non-world `show`
  - `WDP1` — Enabled patch editing + effective enabled view
  - `WDP2` — World-backed status (`applied`) + `show --explain` + backend-unavailable posture

- Checkpoint group CP2 (boundary slice: `WDP5`):
  - `WDP3` — Install/sync planning + `--dry-run` behavior
  - `WDP4` — Script installs + wrapper generation + probes (world-deps prefix)
  - `WDP5` — APT installs + full sync/install engine + legacy-path replacement completeness

Authoritative task graph:
- `docs/project_management/next/world-deps-packages-bundles-contract/tasks.json`
