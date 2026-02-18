# WDP2-spec — World-backed status (`applied`) + `show --explain`

## Scope
- Implement world-backed status surfaces:
  - `substrate world deps current list applied`
  - `substrate world deps current show <item_name> --explain`
- Enforce fail-closed backend posture:
  - World-backed operations exit `3` when the backend is unavailable.

## Behavior (authoritative)
All behavior for commands in scope is defined by the contract doc:
- `docs/project_management/next/world_deps_packages_bundles_contract.md`

Constraints enforced in this slice:
- Default `applied` scope is the current effective enabled set.
- `current list applied --all` is valid only for `applied` and includes every visible inventory item.
- `show --explain` prints enabled provenance and a single-line remediation command when world status is not `present`.
- World-agent enforcement inputs remain concurrency-safe and policy-home agnostic:
  - host-resolved policy snapshot input to world-agent: `docs/project_management/adrs/implemented/ADR-0014-world-agent-policy-resolution-and-concurrency.md`
  - policy schema invariants (full isolation): `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

## Acceptance criteria
- With a healthy world backend, `current list applied` exits `0` and prints `world=present|missing|blocked` for each item.
- With the world backend unavailable, `current list applied` exits `3` with actionable remediation.
- `current show <item> --explain` exits `3` only when world status is required and backend is unavailable.

## Out of scope
- Install/sync mutation surfaces (WDP3+).
