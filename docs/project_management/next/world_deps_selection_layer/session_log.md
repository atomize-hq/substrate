# World Deps Selection Layer — Session Log

This log records planning and execution sessions for the `world_deps_selection_layer` triad(s).

Rules:
- Log entries are append-only.
- Record commands run (for verification) and any deviations from the spec (must be rare; prefer updating the spec instead).

---

## 2025-12-24 — RESEARCH — ADR-0002 planning + triad setup

**Owner:** Codex CLI (planning/research only; no production code)

**Inputs reviewed (required)**
- `docs/project_management/next/ADR-0002-world-deps-install-classes-and-world-provisioning.md`
- `docs/project_management/next/world_deps_selection_layer/decision_register.md` (replaced with final decisions)
- `docs/project_management/next/world_deps_selection_layer/integration_map.md` (updated)
- `docs/project_management/next/world_deps_selection_layer/manual_testing_playbook.md` (updated)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/ADR-0001-agent-hub-runtime-config-and-isolation.md`
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I0-spec.md` through `I5-spec.md`
- `docs/project_management/next/yaml-settings-migration/Y0-spec.md`, `plan.md`, `tasks.json`
- `docs/project_management/next/world-sync/plan.md`, `C0-spec.md`, `C1-spec.md` (skimmed C2–C5 patterns)
- Current touchpoints:
  - `config/manager_hooks.yaml`
  - `scripts/substrate/world-deps.yaml`
  - `crates/shell/src/builtins/world_deps/*`
  - `crates/common/src/world_deps_manifest.rs`, `crates/common/src/manager_manifest/schema.rs`
  - `crates/shell/src/execution/routing/dispatch/world_ops.rs` (`normalize_env_for_linux_guest`)
  - `docs/WORLD.md`

**Outputs produced**
- New specs: `S0`, `S1`, `S2`
- Triad scaffolding: `plan.md`, `tasks.json`, `session_log.md`
- Sequencing update plan captured in `integration_map.md` and applied in `docs/project_management/next/sequencing.json` (see commit for exact diff)

**Key decisions recorded**
- Selection config is YAML-only and stored in `world-deps.selection.yaml` at workspace + global paths (workspace overrides).
- Install classes are embedded into the layered manager manifest schema (no secondary mapping file).
- `system_packages` are represented as structured package lists (apt-first), installed only via explicit `world deps provision` on Lima/WSL; Linux host is manual-only.
- User-space installs live under `/var/lib/substrate/world-deps` and must remain compatible with full cage.
