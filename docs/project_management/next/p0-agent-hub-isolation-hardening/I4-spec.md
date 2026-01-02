# I4-spec: Landlock (Additive Hardening)

## Scope
- Add Landlock-based allowlist enforcement when available:
  - Apply read/write allowlists from `world_fs.*_allowlist`.
  - Make Landlock additive with pivot_root:
  - Landlock runs only inside full isolation (`world_fs.isolation=full`).
  - Landlock never replaces the pivot_root full-isolation guarantee.
- Runtime detection:
  - Detect whether Landlock is supported and which ABI/features are available.
  - Surface in `substrate world doctor` output.

## Acceptance
- On Landlock-capable hosts, path allowlists are enforced for world commands.
- On non-capable hosts, Substrate runs full isolation without Landlock and `substrate world doctor` reports Landlock as unavailable.

## Out of Scope
- Using Landlock as a fallback for full isolation.
