# world-sync — platform parity spec

Template: `docs/project_management/standards/templates/spec/platform-parity-spec.md.tmpl`

## Scope
This spec defines the authoritative supported/unsupported contract for world-sync by platform.

## Platforms

### Linux
- `workspace sync`:
  - `--dry-run` is supported.
  - Non-PTY apply (`direction=from_world`) is supported by WS2.
  - Direction expansion (`from_host|both`) is supported by WS5.
  - PTY pending-diff discovery is supported by WS4; PTY apply is supported by WS5.
- `workspace checkpoint` / `workspace rollback`:
  - Supported by WS6/WS7.

### macOS (Lima-backed world)
- `workspace sync`:
  - Supported when the backend is available (world enabled) and can provide pending diffs.
  - If the backend cannot provide pending diffs or apply semantics, `workspace sync` exits `4` with an explicit unsupported message (no mutations).
- `workspace checkpoint` / `workspace rollback`:
  - Supported by WS6/WS7 (host-only; does not require the world backend).

### Windows (WSL-backed world)
- `workspace sync`:
  - This feature pack is explicit: sync apply is unsupported on Windows (DR-0006).
  - Behavior:
    - `workspace sync --dry-run` exits `4` with an explicit unsupported message.
    - `workspace sync` (apply) exits `4`.
  - Required message substring (case-insensitive) for all unsupported Windows sync paths:
    - `unsupported on windows`
- `workspace checkpoint` / `workspace rollback`:
  - Supported by WS6/WS7 (host-only; requires `git`).

## Validation evidence requirements (authoritative)
- Smoke scripts under `docs/project_management/next/world-sync/smoke/` MUST encode these platform guarantees.
- For checkpoint-boundary slices (WS2, WS5, WS7), the smoke workflow MUST set:
  - `SUBSTRATE_SMOKE_SLICE_ID=WS2` (or `WS5` / `WS7`; via `SMOKE_SLICE_ID` in `make feature-smoke`)
- Each smoke script MUST:
  - exit `0` only when the slice’s required behaviors are satisfied for that platform, and
  - assert exit codes and one or more key message substrings for unsupported paths.
