# M6 Spec – World deps failure safety (macOS)

## Goal
On macOS, `substrate world deps` must not report “synced”/“installed” when the world backend is unreachable and the implementation falls back to host execution.

This triad makes failure modes **safe and honest** so users don’t end up with a false sense of parity.

## Context / Problem
World deps currently has a fallback path that can run “guest” detection/install commands on the host when the world backend/transport is unavailable. On macOS, this can be misleading and can also produce incorrect host-side side effects (Linux recipes executed on macOS).

## Scope
### Required behavior
1. **No silent host fallback for install/sync on macOS**
   - `substrate world deps install ...` and `substrate world deps sync ...` must fail with an actionable error when the world backend is unavailable.
   - The error should guide users to `substrate world doctor --json` and forwarding diagnostics.
2. **Status remains diagnostic**
   - `substrate world deps status` may still return a report, but guest status must clearly indicate “unavailable” with a reason when the backend cannot be reached.
3. **Observability**
   - If any fallback behavior remains for other platforms or specific subcommands, it must be explicit in output and/or JSON so support can diagnose it.

### Out of scope
- Changing forwarding transport selection (vsock vs ssh) beyond what is needed to surface accurate errors.
- Expanding the tool list or changing recipes.

## Acceptance criteria
- On macOS, an unreachable world backend causes `world deps sync/install` to exit non-zero with a clear remediation message.
- `world deps status` clearly distinguishes “missing in guest” from “guest unavailable”.
- No macOS run can “successfully install” guest tools by executing Linux recipes on the host.

