# Execution Preflight Gate Report — workspace-config-policy-unification

Date (UTC): 2026-01-15

Standard:
- `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/_archived/workspace-config-policy-unification/`

## Recommendation

RECOMMENDATION: ACCEPT

## Inputs Reviewed

- [x] Planning quality gate is `ACCEPT` (`docs/project_management/_archived/workspace-config-policy-unification/quality_gate_report.md`)
- [x] ADR-0008 still matches intent:
  - `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- [x] ADR-0012 still matches intent:
  - `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`
- [x] Phase A/B gate file reviewed:
  - `docs/project_management/_archived/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`
- [x] Planning Pack complete (`plan.md`, `tasks.json`, `session_log.md`, specs, kickoff prompts)
- [x] Triad sizing is appropriate (each slice is one behavior delta; no “grab bag” slices)
- [x] Required planning artifacts exist: `integration_map.md`, `manual_testing_playbook.md`
- [x] Cross-platform plan is explicit (tasks.json meta: behavior platforms; smoke scripts exist)

## 0) Slice Sizing (one behavior delta each)
- Slices reviewed:
- WCU1: `.substrate/` directory layout + internal git + disable marker
- WCU2: config patch parsing + schema merge strategies + provenance (ADR-0012 Phase A)
- WCU3: config CLI scopes + set/reset + `--explain` (ADR-0012 Phase B)
- WCU4: installer/dev env cleanup (stop exporting `SUBSTRATE_OVERRIDE_*` by default)
- WCU5: docs + smoke + playbook alignment and parity validation
- Any required splits before starting execution:
- None required.

## 1) Cross-Platform Coverage (explicit and correct)
From `docs/project_management/_archived/workspace-config-policy-unification/tasks.json` meta:
- Declared behavior platforms (smoke required): `["linux", "macos", "windows"]`

## 2) Smoke Scripts Are Not “Toy” Checks
Manual playbook:
- `docs/project_management/_archived/workspace-config-policy-unification/manual_testing_playbook.md`

Smoke scripts:
- Linux: `docs/project_management/_archived/workspace-config-policy-unification/smoke/linux-smoke.sh`
- macOS: `docs/project_management/_archived/workspace-config-policy-unification/smoke/macos-smoke.sh`
- Windows: `docs/project_management/_archived/workspace-config-policy-unification/smoke/windows-smoke.ps1`

Smoke ↔ manual parity notes (map manual steps to smoke assertions):
- Manual step(s):
  - Smoke command(s):
  - Expected output/assertion(s):
- Workspace init `--examples` / `--force` behaviors:
  - Smoke command(s): `substrate workspace init . --examples`, `substrate workspace init . --force`
  - Assertions: example files exist and are not read; `--force` repairs missing internal git/policy and does not overwrite non-empty workspace patch bytes.
- ADR-0012 Phase A/B world-deps list merge + provenance:
  - Smoke command(s): `substrate config {global,workspace} set world.deps.enabled+=...`, `substrate config current show --json --explain`
  - Assertions: effective `world.deps.enabled` contains expected items in ordered-set order; `--explain` includes `merge_strategy=concat_dedupe_ordered_set` and deterministic `global_patch` then `workspace_patch` source ordering; repeat runs are byte-for-byte identical.
- Workspace disabled marker behavior for merge keys:
  - Smoke command(s): `substrate workspace disable .`, `substrate config current show --json --explain`
  - Assertions: workspace contribution is ignored (e.g., `deno` absent), `workspace_patch` is absent from `--explain`.
- World-deps enum keys (replace precedence + strict enum validation):
  - Smoke command(s): `substrate config {global,workspace} set world.deps.inventory_mode=... world.deps.builtins=...`, `substrate config current show --json --explain`
  - Assertions: effective values reflect workspace overrides when enabled; `--explain` reports `merge_strategy=replace` and exactly one contributing source layer; invalid enum values exit `2` and patch file bytes remain unchanged.
- List removal + reset semantics for merge keys:
  - Smoke command(s): `substrate config workspace set world.deps.enabled-=deno`, `substrate config workspace reset world.deps.enabled`
  - Assertions: `-=` is exact-match removal; `reset` removes the key from the patch mapping and global still contributes.

Gaps (must fix before execution begins):
- None.

## 3) CI Dispatch Path Is Runnable (if applicable)
- Feature smoke dispatch commands embedded in integration task end_checklists are runnable:
  - `make feature-smoke ...`

Run ids/URLs (if executed during preflight):
- Linux smoke:
- macOS smoke:
- Windows smoke:

## 4) Required Fixes Before Starting WCU1 (if any)
- None.
