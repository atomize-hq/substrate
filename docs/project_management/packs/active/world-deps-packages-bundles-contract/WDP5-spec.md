# WDP5-spec — APT installs + full sync/install engine + replacement completeness

## Scope
- Implement `install.method=apt` execution (world image installs) and the full install/sync engine:
  - `substrate world deps current install ...`
  - `substrate world deps current sync ...`
- Enforce replacement completeness:
  - legacy world-deps paths do not influence behavior (tests fail if they do).

## Behavior (authoritative)
All behavior for commands in scope is defined by the contract doc:
- `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md`

Constraints enforced in this slice:
- Install ordering is apt-first, script-second.
- `manual` installs are never executed automatically; they are surfaced as blocked and exit `4`.
- Hardening conflicts surface as exit `5` with actionable remediation.

## Acceptance criteria
- `current sync` applies the effective enabled list and exits with contract-defined exit codes.
- `current install` applies the requested items and does not mutate enabled lists.
- Legacy selection/overlay/manifest paths do not affect any `world deps` behavior.

