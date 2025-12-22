# I5-spec: Docs Alignment + Verification Tooling

## Scope
- Align user-facing docs with actual enforced guarantees:
  - `docs/WORLD.md` (what is enforced today; what requires full cage; fallback behavior)
  - `docs/VISION.md` (policy examples must match real enforcement; remove implied guarantees if not implemented)
  - `docs/CONFIGURATION.md` (policy schema examples and toggles)
- Add/update a minimal “verification script” or manual checklist:
  - Demonstrate that project absolute-path writes are blocked when read-only.
  - Demonstrate full cage prevents reading/writing outside the project (when enabled).
  - Include “expected failure modes” and how to interpret them.

## Acceptance
- Docs explicitly state current guarantees and limitations, with clear language around fail-closed vs best-effort.
- Verification steps are reproducible on Linux and note macOS/Windows differences (Lima/WSL).

## Out of Scope
- Implementing new isolation mechanics — handled in I2/I3/I4.

