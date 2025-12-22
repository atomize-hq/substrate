# I4-spec: Landlock (Optional Layer / Fallback)

## Scope
- Add Landlock-based allowlist enforcement when available:
  - Apply read/write allowlists from `world_fs.*_allowlist`.
  - Make Landlock additive with pivot_root:
    - If full cage is available, Landlock may further restrict within the cage.
    - If full cage is not available, Landlock may serve as a best-effort fallback only when policy allows (must be explicit).
- Runtime detection:
  - Detect whether Landlock is supported and which ABI/features are available.
  - Surface in `substrate world doctor` output.

## Acceptance
- On Landlock-capable hosts, path allowlists are enforced for world commands.
- On non-capable hosts, behavior matches spec (fail closed when required; otherwise warn and degrade).

## Out of Scope
- Replacing pivot_root full cage with Landlock-only as the default guarantee.

