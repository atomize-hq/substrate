# Final Alignment Report — `docs/project_management/next/*`

Status: Complete (0 unresolved misalignments)

Scope audited:
- `docs/project_management/next/p0-agent-hub-isolation-hardening/*`
- `docs/project_management/next/world-sync/*`
- `docs/project_management/next/world_deps_selection_layer/*`
- `docs/project_management/next/yaml-settings-migration/*`
- `docs/project_management/next/sequencing.json`
- `docs/project_management/next/p0-agent-hub-isolation-hardening/ADR-0001-agent-hub-runtime-config-and-isolation.md`
- `docs/project_management/next/ADR-0002-world-deps-install-classes-and-world-provisioning.md`

---

## Verified Alignment Checklist (pass)

### Command surface consistency
- [x] `substrate world deps` command surface is identical across ADR-0001, ADR-0002, and WDL specs (`status|sync|install|init|select|provision`) and uses one exit-code taxonomy (0/2/3/4/5).
- [x] `substrate sync` command flags are explicitly defined once (C1) and used consistently in specs and the world-sync playbook (`--direction`, `--conflict-policy`, `--exclude`, `--dry-run`, `--verbose`).
- [x] `substrate init` gating scope is singular and consistent: it gates world-sync commands (`sync|checkpoint|rollback`) and does not gate unrelated `substrate world …` commands.

### File path + naming consistency
- [x] YAML settings paths are consistent everywhere: `~/.substrate/config.yaml` and `.substrate/settings.yaml`; TOML paths are called out only as explicitly unsupported.
- [x] World-deps selection config is consistent everywhere:
  - filename: `world-deps.selection.yaml`
  - workspace path: `.substrate/world-deps.selection.yaml`
  - global path: `~/.substrate/world-deps.selection.yaml`
- [x] World-deps overlay/inventory paths are singular and consistent:
  - base inventory: `config/manager_hooks.yaml`
  - overlays: `~/.substrate/manager_hooks.local.yaml`, `scripts/substrate/world-deps.yaml`, `~/.substrate/world-deps.local.yaml`

### Prerequisites and dependency correctness
- [x] Y0 is treated as a hard prerequisite for any new runtime YAML surfaces (WDL slices).
- [x] WDL0–WDL2 are placed between C1 and C2 and tasks reflect that ordering.
- [x] No circular dependencies exist in `docs/project_management/next/sequencing.json` and scoped `tasks.json`.

### Cross-triad invariants (explicit + compatible)
- [x] YAML-only runtime settings is explicitly compatible across Y0, ADR-0001, ADR-0002, WDL specs, and world-sync specs.
- [x] `.substrate/` workspace model is explicit and consistent: created by `substrate init`, used for workspace selection (`world-deps.selection.yaml`) and workspace settings (`settings.yaml`), protected from sync mutation.
- [x] Full-cage constraints and required writable mounts are explicit:
  - `/var/lib/substrate/world-deps` must be mounted read-write inside a full cage (I2/I3) and is required by world-deps (S1/ADR-0002).
- [x] Selection-driven/no-op posture is explicit and consistent:
  - missing selection is a no-op with exit `0` and no world backend calls.

### Testability + playbooks
- [x] All triads in scope that affect UX/provisioning have `manual_testing_playbook.md` with runnable steps and explicit expected exit codes.
- [x] Specs no longer contain “stubbed/unimplemented/verify it works” acceptance statements; failure modes specify explicit behavior and exit codes.

### Tasks readiness
- [x] Every `tasks.json` in scope is valid JSON (`jq .` succeeds).
- [x] Triad task dependency chains reflect `docs/project_management/next/sequencing.json`, including the C1 → WDL → C2 insertion.
- [x] Kickoff prompts referenced by tasks are present and aligned to spec decisions for direction handling and `world_fs.require_world`.

---

## Misalignments Found (all corrected)

Each item: issue → impact → exact fix → files changed.

1) ADR-0002 contained open questions and a research prompt instead of final decisions
- Impact: created non-executable ambiguity and violated “no open questions” contract for the queued work.
- Exact fix: rewrote ADR-0002 as an Accepted ADR with final decisions, fixed command surface, fixed file paths, fixed exit codes, and explicit sequencing contract.
- Files changed: `docs/project_management/next/ADR-0002-world-deps-install-classes-and-world-provisioning.md`

2) ADR-0001 had naming drift and an open runtime-format decision
- Impact: conflicting selection filename/paths and TOML/YAML uncertainty would block implementation and cross-triad composition.
- Exact fix: marked selection naming/paths as Accepted and singular; marked runtime settings format as YAML-only (Y0); removed “Open Questions”; aligned Landlock posture to “additive only”; removed ambiguous proposal sections that diverged from WDL specs.
- Files changed: `docs/project_management/next/p0-agent-hub-isolation-hardening/ADR-0001-agent-hub-runtime-config-and-isolation.md`

3) Agent-hub hardening specs had ambiguous “optional world”, “optional Landlock”, and non-singular derivation rules
- Impact: fail-closed semantics and enforcement expectations were not implementable without interpretation.
- Exact fix:
  - Introduced `world_fs.require_world` as the single required-world knob (I0/I1).
  - Removed “optional world” language; replaced with explicit `world_fs.require_world=false` fallback semantics.
  - Made Landlock additive-only and non-fallback.
  - Added explicit full-cage mount requirement for `/var/lib/substrate/world-deps`.
- Files changed:
  - `docs/project_management/next/p0-agent-hub-isolation-hardening/I0-spec.md`
  - `docs/project_management/next/p0-agent-hub-isolation-hardening/I1-spec.md`
  - `docs/project_management/next/p0-agent-hub-isolation-hardening/I2-spec.md`
  - `docs/project_management/next/p0-agent-hub-isolation-hardening/I3-spec.md`
  - `docs/project_management/next/p0-agent-hub-isolation-hardening/I4-spec.md`
  - `docs/project_management/next/p0-agent-hub-isolation-hardening/plan.md`
  - `docs/project_management/next/p0-agent-hub-isolation-hardening/tasks.json`
  - `docs/project_management/next/p0-agent-hub-isolation-hardening/kickoff_prompts/I1-code.md`
  - `docs/project_management/next/p0-agent-hub-isolation-hardening/kickoff_prompts/I4-code.md`
  - `docs/project_management/next/p0-agent-hub-isolation-hardening/kickoff_prompts/I4-test.md`
  - `docs/project_management/next/p0-agent-hub-isolation-hardening/kickoff_prompts/I4-integ.md`

4) World-sync had a draft document with TOML-era references and ambiguous direction/filter language
- Impact: path drift against Y0 and non-final behavior statements (“stubbed”, “optional include list”, “degrade gracefully”).
- Exact fix:
  - Deleted the unused draft file.
  - Made direction handling singular per slice (C2 supports only `from_world`; `from_host|both` are implemented in C5).
  - Defined stable exit codes in the plan and made protected-path and size-guard behavior explicit (exit `5`).
  - Fixed internal git path to a single git directory (`.substrate-git/repo.git`) across C0/C6 and aligned C8 to “host-only internal git”.
- Files changed:
  - `docs/project_management/next/world-sync/DRAFT_WORLD_SYNC.md` (deleted)
  - `docs/project_management/next/world-sync/plan.md`
  - `docs/project_management/next/world-sync/C0-spec.md`
  - `docs/project_management/next/world-sync/C1-spec.md`
  - `docs/project_management/next/world-sync/C2-spec.md`
  - `docs/project_management/next/world-sync/C3-spec.md`
  - `docs/project_management/next/world-sync/C4-spec.md`
  - `docs/project_management/next/world-sync/C5-spec.md`
  - `docs/project_management/next/world-sync/C6-spec.md`
  - `docs/project_management/next/world-sync/C7-spec.md`
  - `docs/project_management/next/world-sync/C8-spec.md`
  - `docs/project_management/next/world-sync/C9-spec.md`
  - `docs/project_management/next/world-sync/kickoff_prompts/C2-code.md`

5) World-deps selection layer contained non-final/future language and minor naming ambiguity
- Impact: implementation ambiguity for provisioning output surface and overlay location; rubric violations in behavioral statements.
- Exact fix:
  - Removed future `provision --json` language; decided `provision` is human-only for this track.
  - Removed ambiguous installed-overlay path variants; fixed to `scripts/substrate/world-deps.yaml`.
  - Tightened “worlds must feel the same” wording and removed remaining ambiguity language in specs.
  - Made the manual playbook runnable without “depends on … once it lands” placeholders by explicitly injecting a manual tool via `~/.substrate/manager_hooks.local.yaml`.
- Files changed:
  - `docs/project_management/next/world_deps_selection_layer/S0-spec-selection-config-and-ux.md`
  - `docs/project_management/next/world_deps_selection_layer/S1-spec-install-classes.md`
  - `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`
  - `docs/project_management/next/world_deps_selection_layer/decision_register.md`
  - `docs/project_management/next/world_deps_selection_layer/integration_map.md`
  - `docs/project_management/next/world_deps_selection_layer/manual_testing_playbook.md`
  - `docs/project_management/next/world_deps_selection_layer/plan.md`
  - `docs/project_management/next/world_deps_selection_layer/session_log.md`

6) Cross-triad task dependency drift vs sequencing.json (C1 → WDL → C2)
- Impact: executing tasks in dependency order would violate sequencing and create merge/conflict risk.
- Exact fix:
  - Updated world-sync `C2-*` tasks to depend on `WDL2-integ`.
  - Updated `WDL0-*` tasks to depend on `Y0-integ` and `C1-integ`.
- Files changed:
  - `docs/project_management/next/world-sync/tasks.json`
  - `docs/project_management/next/world_deps_selection_layer/tasks.json`

7) Missing required manual playbooks
- Impact: UX/provisioning changes were not runnable/validatable by humans, violating the required deliverable.
- Exact fix: created the missing playbooks with runnable steps and explicit expected exit codes.
- Files changed:
  - `docs/project_management/next/yaml-settings-migration/manual_testing_playbook.md` (added)
  - `docs/project_management/next/p0-agent-hub-isolation-hardening/manual_testing_playbook.md` (added)
  - `docs/project_management/next/world-sync/manual_testing_playbook.md` (added)

---

## Rubric Violations (lint-like) and Corrections

### A) Ambiguity bans (TBD/TODO/WIP/TBA, “should”, “could”, “maybe”, “etc.”, behavior-level “optional”)
- Violations found:
  - “Draft + open questions” structures in ADR-0002 and ADR-0001.
  - “should/could/etc.” in behavioral and acceptance statements across specs/playbooks.
  - Future/placeholder language (e.g., “provision should support --json once … lands”, “stubbed/unimplemented”, “depends on … once it lands”).
- Corrections applied:
  - Rewrote ADR-0002 as final decisions and removed open-question prompt structure.
  - Removed ambiguity words from specs/playbooks and replaced with singular must-level contracts and explicit exit codes.
  - Removed future placeholders by making a single decision (e.g., `provision` has no `--json` in this track).

### B) Naming and path drift bans
- Violations found:
  - Selection config naming drift in ADR-0001.
  - Mixed TOML/YAML references in world-sync draft material.
  - Installed-overlay path ambiguity in world-deps docs.
- Corrections applied:
  - Fixed selection config naming/paths to `world-deps.selection.yaml` at workspace+global locations everywhere.
  - Deleted the TOML-era world-sync draft and standardized world-sync settings to `.substrate/settings.yaml`.
  - Fixed installed overlay to `scripts/substrate/world-deps.yaml` everywhere.

### C) Command/flag/exit-code drift bans
- Violations found:
  - World-sync flags were underspecified and direction handling was described inconsistently.
  - Protected-path handling and size guard lacked explicit exit codes.
- Corrections applied:
  - Defined `substrate sync` flags and settings schema in `C1-spec.md`.
  - Defined stable world-sync exit codes in `world-sync/plan.md`.
  - Set protected-path-only diffs and size-guard failures to exit `5` in `C2-spec.md`.

### D) Dependency/sequencing drift bans
- Violations found:
  - `tasks.json` dependency chains did not enforce the WDL insertion between C1 and C2.
- Corrections applied:
  - Updated `depends_on` for C2 and WDL0 tasks to reflect the final sequencing order.

### E) Testability bans
- Violations found:
  - Playbook steps with missing commands/expected outputs and placeholder “depends on …” statements.
- Corrections applied:
  - Added three missing playbooks and made the existing WDL playbook fully runnable by creating explicit test overlays and explicit exit-code checks.

---

## Sequencing.json Updates

- No change required to `docs/project_management/next/sequencing.json`.
- Conflict-free execution was achieved by aligning `depends_on` in scoped `tasks.json` to the already-final WDL insertion in sequencing.

---

## Allowed Phrase Exceptions (explicit)

- The word “optional” appears only inside an explicit `Option A`/`Option B` comparison in `docs/project_management/next/world_deps_selection_layer/decision_register.md`.
