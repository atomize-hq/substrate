# world-deps-host-visible-hardening — spec manifest

Authoring standard:
- `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/next/world-deps-host-visible-hardening`
- ADR(s) / upstream contracts (inputs; not owned by this Planning Pack):
  - `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md` (Appendix A)
  - `docs/project_management/next/world_deps_packages_bundles_contract.md`
  - `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

## Required spec documents (authoritative)

- `docs/project_management/next/world-deps-host-visible-hardening/spec_manifest.md` — spec selection + ownership map (this file)
- `docs/project_management/next/world-deps-host-visible-hardening/impact_map.md` — touch set + cascading implications + cross-queue conflicts
- `docs/project_management/next/world-deps-host-visible-hardening/plan.md` — execution runbook + slicing
- `docs/project_management/next/world-deps-host-visible-hardening/tasks.json` — triad task graph + checkpoints
- `docs/project_management/next/world-deps-host-visible-hardening/ci_checkpoint_plan.md` — bounded CI checkpoint groups + wiring (schema v4+)
- `docs/project_management/next/world-deps-host-visible-hardening/session_log.md` — planning/execution evidence log
- `docs/project_management/next/world-deps-host-visible-hardening/decision_register.md` — decisions required to execute Appendix A without ambiguity
- `docs/project_management/next/world-deps-host-visible-hardening/manual_testing_playbook.md` — human validation workflow (authoritative)
- `docs/project_management/next/world-deps-host-visible-hardening/quality_gate_report.md` — planning pack quality gate artifact (required before execution)
- Feature smoke scripts:
  - `docs/project_management/next/world-deps-host-visible-hardening/smoke/linux-smoke.sh`
  - `docs/project_management/next/world-deps-host-visible-hardening/smoke/macos-smoke.sh`
  - `docs/project_management/next/world-deps-host-visible-hardening/smoke/windows-smoke.ps1`
  - `docs/project_management/next/world-deps-host-visible-hardening/smoke/_core.sh`
- Slice specs:
  - `docs/project_management/next/world-deps-host-visible-hardening/WDH0-spec.md`
  - `docs/project_management/next/world-deps-host-visible-hardening/WDH1-spec.md`
  - `docs/project_management/next/world-deps-host-visible-hardening/WDH2-spec.md`
  - `docs/project_management/next/world-deps-host-visible-hardening/WDH3-spec.md`

## Coverage matrix (surface → authoritative doc)

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| In-world env construction for `--world` | `docs/project_management/next/world-deps-host-visible-hardening/WDH0-spec.md` | baseline PATH, sanitization rules, env var allow/deny list |
| Config key `world.env.inherit_from_host` | `docs/project_management/next/world-deps-host-visible-hardening/WDH0-spec.md` | schema, default, precedence, forwarded host env keys, reserved keys, warning emission |
| Env var `SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR` | `docs/project_management/next/world-deps-host-visible-hardening/WDH0-spec.md` | exact value, relationship to PATH construction, propagation rules |
| Runnable apt wrapper requirement | `docs/project_management/next/world-deps-host-visible-hardening/WDH1-spec.md` | wrapper form, creation/update/remove rules, collision semantics |
| “present/missing/blocked” semantics (host-visible) | `docs/project_management/next/world-deps-host-visible-hardening/WDH1-spec.md` | default probe strategy, wrapper-based presence, no inherited PATH dependency |
| Override env var `SUBSTRATE_OVERRIDE_WORLD_EXEC_GUARD` | `docs/project_management/next/world-deps-host-visible-hardening/WDH2-spec.md` | allowed values, default, effect on deny behavior when `world_fs.host_visible=true` |
| Override env var `SUBSTRATE_OVERRIDE_WORLD_EXEC_GUARD_DENY_CONTAINS` | `docs/project_management/next/world-deps-host-visible-hardening/WDH2-spec.md` | format, precedence, replacement semantics for the denylist |
| Host-binary execution guardrails | `docs/project_management/next/world-deps-host-visible-hardening/WDH2-spec.md` | default posture, default denylist, override env vars, exit codes, error messages |
| Installer scaffolding for `$SUBSTRATE_HOME/deps/` | `docs/project_management/next/world-deps-host-visible-hardening/WDH3-spec.md` | when created, exact layout, example contents, non-enabling rule |
| Manual validation workflow | `docs/project_management/next/world-deps-host-visible-hardening/manual_testing_playbook.md` | commands + expected outputs/exit codes |

## Determinism checklist (must be satisfied before quality gate)

For every selected spec document, confirm it explicitly defines:
- Inputs (all) + precedence (if multiple inputs exist)
- Defaults (all) + absence semantics
- Data model boundaries (types/constraints) for any serialized surface added/changed
- Error model (exit codes; messages where applicable) and failure posture
- Ordering/atomicity/concurrency rules (wrapper updates; install/sync)
- Platform guarantees (Linux/macOS/WSL)
