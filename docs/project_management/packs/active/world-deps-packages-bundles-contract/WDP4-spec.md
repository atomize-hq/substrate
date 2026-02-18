# WDP4-spec — Script installs + wrapper generation (world-deps prefix)

## Scope
- Implement `install.method=script` execution and wrapper generation:
  - world-deps prefix installs under `/var/lib/substrate/world-deps`
  - wrapper entrypoints under `/var/lib/substrate/world-deps/bin`
- Implement `current install` for script-only plans (no apt in this slice).

## Behavior (authoritative)
All behavior for commands in scope is defined by the contract doc:
- `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md`

Constraints enforced in this slice:
- Wrapper generation is deterministic and idempotent.
- After a successful script install, runnable entrypoints are invokable under both:
  - non-interactive `/bin/sh -c` world execution
  - interactive REPL evaluator semantics (`bash --noprofile --norc -c`)

## Acceptance criteria
- Script installs place entrypoints under `/var/lib/substrate/world-deps/bin` and are invokable via `probe.command` or `entrypoints[]`.
- Wrapper generation kinds behave per contract and produce actionable stderr on failure.

## Out of scope
- APT installs (WDP5).

