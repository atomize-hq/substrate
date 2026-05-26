# World Deps Provisioning Contract

This document is the stable operator-facing contract for system-package provisioning and runtime
fail-early behavior in `substrate world deps`.

## Commands in scope

- Provisioning:
  - `substrate world enable --provision-deps [--dry-run] [--verbose]`
- Runtime world deps:
  - `substrate world deps current sync [--dry-run] [--verbose] [--all]`
  - `substrate world deps current install <ITEM...> [--dry-run] [--verbose]`

## Core invariants

- Runtime `substrate world deps current sync` and `substrate world deps current install` never
  invoke `apt`, `apt-get`, mutating `dpkg`, or `pacman`.
- Runtime system-package checks are probe-only and use read-only presence checks.
- `substrate world enable --provision-deps` is the only operator-facing Substrate workflow that
  performs provisioning-time system-package mutation for world deps on supported guest backends.
- Linux host-native must not mutate the host OS.
- Missing-package remediation includes the exact command:

```text
substrate world enable --provision-deps
```

## Platform and backend guarantees

| Platform/backend | `substrate world enable --provision-deps` | Runtime `substrate world deps current sync/install` for system-package-backed items |
| --- | --- | --- |
| Linux host-native world backend | Unsupported (exit `4`); Substrate will not mutate the host OS | Probe-only; exits `4` when required system packages are missing |
| macOS Lima guest world backend | Supported | Probe-only; exits `4` when required system packages are missing |
| Windows | Unsupported (exit `4`); unsupported on Windows | Probe-only; exits `4` when required system packages are missing |

## Provisioning contract

`substrate world enable --provision-deps` derives requirements from the effective enabled
world-deps set for the current directory.

Behavior:
- If the derived requirement set is empty, the command is a no-op and exits `0`.
- If all required system packages are already present, the command is a no-op and exits `0`.
- `--dry-run` prints the derived requirement set without mutating the world.
- `--verbose` additionally reports the provisioning request posture used for the world-service call.
- On Linux host-native, the command exits `4` and states that Substrate will not mutate the host OS.
- On Windows, the command exits `4` and remains unsupported on Windows.

## Runtime fail-early contract

For system-package-backed items, runtime application remains probe-only:
- `substrate world deps current sync`
- `substrate world deps current install <ITEM...>`

Behavior:
- Requirements are derived from the in-scope item set.
- Requirements are probed read-only inside the world.
- If required system packages are missing, the command exits `4` with remediation.
- If required system packages are already present, system-package items are treated as satisfied and
  Substrate continues with non-system-package work.
- `--dry-run` still enforces the fail-early rule and still exits `4` when required system packages
  are missing.

## Request profile note

Built-in world-deps request profiles such as `world-deps-provision` and `world-deps-probe` are
reserved for Substrate’s own world-deps flows. `SUBSTRATE_WORLD_REQUEST_PROFILE` does not select
them.
