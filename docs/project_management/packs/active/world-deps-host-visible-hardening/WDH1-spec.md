# WDH1 (CP1) — Runnable `apt` wrappers + host-path-independent “present”

## Goal
Make runnable `apt` packages behave like runnable script packages by anchoring entrypoint resolution in `/var/lib/substrate/world-deps/bin`, and redefine “present” to be wrapper/probe-based under the sanitized env.

## Inputs (authoritative)
- `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md` (Appendix A + inventory schema)
- `docs/project_management/packs/active/world-deps-host-visible-hardening/decision_register.md` (DR-0003, DR-0004, DR-0009)
- `docs/project_management/next/world_deps_packages_bundles_contract.md` (status semantics)

## Contract

### Wrapper requirement for runnable packages
For every runnable package, for every `entrypoints[]` name, `sync` MUST ensure there is an executable at:
- `/var/lib/substrate/world-deps/bin/<entrypoint>`

Rules:
- Script packages: the installer must already create the wrapper/entrypoint.
- Apt packages: `sync` MUST create a POSIX `sh` wrapper script after apt install.
- Wrapper creation MUST be idempotent.
- Wrapper collisions MUST fail-closed with exit code `5` (two packages claim the same entrypoint).

### Wrapper form (apt)
For apt packages, wrappers MUST:
- be a file at `/var/lib/substrate/world-deps/bin/<entrypoint>` that is executable
- use `#!/bin/sh` (not bash)
- `exec /usr/bin/<entrypoint> "$@"`

The wrapper MUST NOT resolve targets via PATH (to avoid recursion back into `/var/lib/substrate/world-deps/bin`).

### “present” semantics (runnable)
Default rule:
- If `probe.command` exists: present iff the probe succeeds under the sanitized env.
- Else: present iff for every `entrypoints[]` name, `command -v <entrypoint>` under the sanitized env resolves to `/var/lib/substrate/world-deps/bin/<entrypoint>`.

“present” MUST NOT be computed by searching an inherited PATH.

## Exit codes
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- 5: wrapper collision, wrapper creation denied, or other safety/policy violation

## Acceptance criteria
- With `world_fs.host_visible=true` and `npm` enabled/applied:
  - `substrate --world -c 'command -v npm'` returns `/var/lib/substrate/world-deps/bin/npm`
- With `npm` not enabled/applied:
  - `substrate --world -c 'command -v npm >/dev/null'` exits `1`
- `world deps current list applied` reports runnable package status based on wrapper/probe under the sanitized env, independent of host PATH.

## Out of scope
- Exec-time host binary guard (WDH2)
- Installer scaffolding (WDH3)
