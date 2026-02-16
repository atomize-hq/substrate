# Kickoff: WDH1-test (test)

## Scope
- Tests only.
- Spec: `docs/project_management/next/world-deps-host-visible-hardening/WDH1-spec.md`

## Requirements
- Add tests that `command -v <entrypoint>` resolves to `/var/lib/substrate/world-deps/bin/<entrypoint>` only when enabled/applied.

