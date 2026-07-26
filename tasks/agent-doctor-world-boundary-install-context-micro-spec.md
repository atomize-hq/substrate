# Micro-spec: agent doctor world-boundary install-context propagation

## Objective

Fix the dev-install regression where `substrate agent doctor --json` fails the
`world_boundary` check because its nested `world doctor --json` respawn loses
the authenticated install context.

## Why

The installed-path invocation is valid, but the nested world-boundary check
respawns from `current_exe()` and currently does not preserve the parent
invocation's install identity. In a dev install, that drops the installed
product witness and fail-closes with:

- `substrate: an installed-product invocation witness or --install-prefix is required`

## Scope

In scope:

- `crates/shell/src/execution/agents_cmd.rs`
- focused tests that prove nested respawn context propagation

Out of scope:

- trusted-fs mode/ownership requirements
- install bootstrap carrier format
- world doctor semantics outside this nested respawn path
- unrelated installer or smoke-helper behavior

## Constraints

- Reuse the repo's existing install-context mechanism.
- Do not weaken fail-closed behavior.
- Do not special-case the dev-install path by bypassing validation.
- Keep the change minimal and local to the world-boundary respawn path.

## Success criteria

- The nested world-boundary command inherits the parent install selection and/or
  authenticated install bootstrap context.
- Focused tests prove the nested command carries the required install context.
- Manual installed-path repro no longer fails `world_boundary` with the missing
  install-witness error.

## Verification

- Focused Rust unit/integration tests for nested respawn argument propagation.
- Real installed-path repro:
  - `~/.substrate/bin/substrate agent doctor --json`

