# I6-spec: `substrate world verify` (Enforcement Verification Command)

## Scope
- Add a new CLI subcommand: `substrate world verify`.
- The command performs **end-to-end enforcement checks** (not just readiness) for the Agent Hub Isolation
  Hardening track, in a way that works for **prefix installs** (i.e., without requiring this repo checkout).
- Checks must be reproducible and safe:
  - Uses temporary directories under the OS temp directory (or an explicitly provided log/temp root).
  - Never writes outside the temporary working directories except for logs/artifacts.
  - Uses policy files written into the temporary directories (e.g., `.substrate-profile`) so it does not
    mutate the user’s real projects.

### Minimum checks (v1)
1) **Read-only project**: prove `world_fs.mode=read_only` blocks both:
   - A relative write in the project
   - An absolute-path write targeting a file under the project’s absolute path

2) **Full cage host-path isolation**: when `world_fs.cage=full` is enabled, prove:
   - Writing to an allowlisted project prefix succeeds (e.g., `./writable/*`)
   - Writing to a non-allowlisted project path fails
   - Reading a host path outside the project fails
   - Writing a host path outside the project fails

### Platform expectations
- Linux/macOS: **required parity** when the world backend is available (`substrate world doctor --json` ok=true).
  - The implementation must avoid Linux-host assumptions (e.g., `/run/substrate.sock`) and must rely on
    normal world routing so it works on macOS (Lima) as well.
  - Full cage may require privileges/user namespaces; failures must be clearly explained.
- Windows: allowed to be `SKIP` or “not yet supported” initially (explicit message + non-zero only if the
  user requested strict mode).

### Output and exit behavior
- Human output:
  - Prints the individual checks and a final `PASS`/`FAIL` line.
  - On failure, prints the minimal next-action hint (e.g., “run `substrate world doctor --json`” or
    “full cage requires CAP_SYS_ADMIN or unprivileged user namespaces”).
- Machine output (recommended):
  - Add `--json` to emit a structured report (per-check status + error strings + artifact paths).
- Exit codes:
  - `0` on success.
  - Non-zero on any failed check (including a missing world backend when checks require it).
  - If a check is skipped due to platform/privileges, the default behavior must be to `SKIP` that check
    and continue, but still return non-zero if *no* checks ran or if the user requested strict mode.

## Acceptance
- `substrate world verify` exists and is documented in `--help`.
- The command can be run from any directory and does not require a repo checkout.
- On a correctly provisioned Linux host, it can demonstrate:
  - `world_fs.mode=read_only` blocks relative + absolute project writes.
  - `world_fs.cage=full` blocks host paths outside the project and enforces project allowlists.
- On a correctly provisioned macOS host (Lima backend), it demonstrates the same outcomes (parity), or
  clearly reports any missing capability and how to fix it.
- Failures are actionable (clear reasons + next steps).
- Tests cover:
  - CLI wiring and `--json` shape/stability.
  - “Skip” behavior when world backend is unavailable or platform doesn’t support the check.

## Out of Scope
- Adding new isolation mechanisms (mount namespace, pivot_root, Landlock behavior) beyond what exists.
- Making the installer run verification automatically.
- Windows verification parity (beyond explicit “not supported/skip” behavior).
