# world-deps-host-visible-hardening — impact map

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/active/world-deps-host-visible-hardening`
- ADR(s):
  - `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md` (Appendix A)
  - `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
- Contract:
  - `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md`
- Spec manifest:
  - `docs/project_management/packs/active/world-deps-host-visible-hardening/spec_manifest.md`

## Touch set (explicit)

### Create (planning pack)
- `docs/project_management/packs/active/world-deps-host-visible-hardening/plan.md`
- `docs/project_management/packs/active/world-deps-host-visible-hardening/spec_manifest.md`
- `docs/project_management/packs/active/world-deps-host-visible-hardening/tasks.json`
- `docs/project_management/packs/active/world-deps-host-visible-hardening/session_log.md`
- `docs/project_management/packs/active/world-deps-host-visible-hardening/decision_register.md`
- `docs/project_management/packs/active/world-deps-host-visible-hardening/impact_map.md`
- `docs/project_management/packs/active/world-deps-host-visible-hardening/manual_testing_playbook.md`
- `docs/project_management/packs/active/world-deps-host-visible-hardening/quality_gate_report.md`
- `docs/project_management/packs/active/world-deps-host-visible-hardening/smoke/*`
- `docs/project_management/packs/active/world-deps-host-visible-hardening/kickoff_prompts/*`
- `docs/project_management/packs/active/world-deps-host-visible-hardening/WDH*-spec.md`

### Edit (execution scope; non-doc)
Execution triads under this Planning Pack are expected to edit:
- Host-side world execution env construction:
  - `crates/shell/src/execution/routing/dispatch/world_ops.rs` (PTY + non-PTY env maps)
  - any shared env-building helpers used by macOS/Windows request builders
- World-deps apply/probe logic:
  - `crates/shell/src/builtins/world_deps/*` (presence semantics and wrapper reconciliation)
  - `crates/world-agent/*` (probe execution environment; wrapper install steps if in-world)
- Policy/config surfaces for env mode + exec guard:
  - `crates/broker/*` (policy schema + snapshot) and/or `crates/common/*` models, as required by existing policy plumbing
- Installer / first-run initialization:
  - the install/init path that creates `$SUBSTRATE_HOME` (to scaffold `$SUBSTRATE_HOME/deps/`)

### Edit (docs scope; expected follow-up)
- `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md` (link this Planning Pack; tighten Appendix A to MUST-level language where implemented)
- User-facing docs that explain inventory vs enabled/applied and the scaffolded deps directory (if those docs are considered authoritative outside `docs/project_management/packs/`)
- `docs/CONFIGURATION.md` and `docs/reference/config/world.md` — document the new config key `world.env.inherit_from_host` and its default posture

## Cascading implications (behavior/UX)

### World execution (PTY + non-PTY)
- Change:
  - `--world` no longer inherits host user toolchain env by default; it constructs a sanitized env.
- Direct impact:
  - Host-visible worlds no longer “accidentally” have host toolchains via PATH.
- Contradiction risks:
  - Some workflows currently rely on inherited env vars; mitigated via explicit opt-in env mode (DR-0007).

### World-deps applied/present semantics
- Change:
  - Runnable `apt` packages gain deterministic wrappers under `/var/lib/substrate/world-deps/bin`.
  - `present` is wrapper/probe-based under the sanitized env.
- Direct impact:
  - `command -v npm` resolves to the wrapper only when enabled/applied.

### Hardened posture vs explicit host path execution
- Change:
  - Optional exec-time guard denies explicit execution of host-mounted binaries (exit `5`) unless allowed by override inputs.
- Direct impact:
  - “Host-visible” stays filesystem-only; toolchains remain world-deps-controlled.

### Installer scaffolding
- Change:
  - `$SUBSTRATE_HOME/deps/` is created on install/first-run init with examples.
- Direct impact:
  - Operators have a canonical place to add global inventory and can see the intended directory shape immediately.

## Cross-queue scan (ADRs + Planning Packs)

### ADR-0011
- Overlap: world-deps runnable contract, world-deps prefix rules.
- Resolution: This Planning Pack operationalizes ADR-0011 Appendix A into deterministic implementation slices.

### ADR-0018
- Overlap: `world_fs.host_visible` semantics.
- Resolution: Treat host-visible as filesystem visibility only; do not inherit host toolchain env by default.

## Sequencing alignment
- `docs/project_management/packs/sequencing.json` reviewed: YES
- Sequencing entry: `world_deps_host_visible_hardening` (WDH0..WDH3)
