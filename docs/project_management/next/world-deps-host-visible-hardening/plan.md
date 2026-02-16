# world-deps-host-visible-hardening — plan (v1)

## Scope
- Feature directory: `docs/project_management/next/world-deps-host-visible-hardening`
- Orchestration branch: `feat/world-deps-host-visible-hardening`
- Authoritative contract inputs (source of truth):
  - `docs/project_management/next/ADR-0011-world-deps-packages-bundles-contract.md` (Appendix A)
  - `docs/project_management/next/world_deps_packages_bundles_contract.md`
  - `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md` (host_visible semantics)
- Planning standard:
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`

## Goal (operator-facing)
Make `--world` execution deterministic and hardened in **host-visible worlds** (`world_fs.host_visible=true`) by ensuring:
- Host toolchains do not satisfy runnable deps via PATH resolution.
- Enabled/applied runnable deps resolve via `/var/lib/substrate/world-deps/bin` entrypoints/wrappers.
- “present/missing/blocked” probes are host-path-independent.
- (Optional hardening) Explicit execution of host-mounted toolchain binaries is denied by policy.
- The installer scaffolds `$SUBSTRATE_HOME/deps/` with example inventory shape to reduce confusion about “available” vs “enabled/applied”.

## Non-negotiable invariants
- `--world` MUST NOT inherit host user toolchain PATH segments by default.
- `/var/lib/substrate/world-deps/bin` MUST be first in PATH for all `--world` execution pathways (PTY + non-PTY).
- Runnable `apt` packages MUST expose entrypoints under `/var/lib/substrate/world-deps/bin` (wrapper/symlink), just like script packages do.
- “present” MUST be computed under a sanitized world environment and MUST NOT accept “some host PATH entry had it”.

## Cross-platform posture
- Behavior platforms (smoke required): `linux`, `macos`
- CI parity platforms (compile parity required): `linux`, `macos`
- WSL coverage: required (bundled via Linux smoke where applicable)

## Execution gates
- A planning quality gate report MUST exist and be `ACCEPT` before any execution triads begin:
  - `docs/project_management/next/world-deps-host-visible-hardening/quality_gate_report.md`

## Triads (authoritative slice list)

Checkpoint boundaries (cross-platform parity gates):
- CP1: `WDH1`
- CP2: `WDH3`

Slices:
- `WDH0` — World env normalization contract (PATH/HOME/XDG) + request/PTY env construction
- `WDH1` (CP1) — Deterministic wrappers for runnable `apt` packages + host-path-independent “present” semantics
- `WDH2` — Execution-time guardrails against explicit host-binary execution (policy surface + enforcement)
- `WDH3` (CP2) — Installer scaffolding for `$SUBSTRATE_HOME/deps/` + examples + UX docs alignment

Authoritative task graph:
- `docs/project_management/next/world-deps-host-visible-hardening/tasks.json`

