# world-deps-host-visible-hardening — manual testing playbook (Authoritative)

Standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (section 6)

Exit code taxonomy:
- `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Scope
Validates the host-visible hardening contract described in ADR-0011 Appendix A and this Planning Pack:
- `--world` environment is sanitized by default (PATH/HOME/XDG).
- Host toolchains do not satisfy runnable deps via PATH resolution.
- Runnable `apt` packages produce deterministic wrappers under `/var/lib/substrate/world-deps/bin`.
- “present” is computed under the sanitized env and is host-path-independent.
- Exec-time guardrails deny explicit execution of host-mounted toolchain binaries (override-input controlled).
- Installer scaffolds `$SUBSTRATE_HOME/deps/` with examples (non-enabling).

## Prerequisites
- `substrate` available on PATH, or set `SUBSTRATE_BIN=/path/to/substrate`.
- World backend healthy for the platform under test:
  - Run `substrate world doctor` and fix any reported issues before proceeding.
- Tools:
  - `bash` (Linux/macOS/WSL)

## Smoke scripts (required)
- Linux: `docs/project_management/next/world-deps-host-visible-hardening/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/world-deps-host-visible-hardening/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/world-deps-host-visible-hardening/smoke/windows-smoke.ps1` (WSL-backed where applicable)

Smoke slice selection:
- `SUBSTRATE_SMOKE_SLICE_ID=WDH0|WDH1|WDH2|WDH3` (default: `WDH3`)

## Case 1 — Host-visible + no enabled deps: host toolchains are not discoverable via PATH

Precondition:
- Policy: `world_fs.host_visible=true`
- Enabled deps list is empty (or does not include `node`/`npm`).

Command:
```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
"$SUBSTRATE_BIN" policy global set world_fs.host_visible=true >/dev/null

set +e
"$SUBSTRATE_BIN" --world -c 'command -v npm >/dev/null'
status=$?
set -e
test "$status" -eq 1
```

Expected:
- Exit code `1` (from `command -v` not finding `npm`).

Also assert PATH is sanitized (no host toolchain segments):
```bash
path="$("$SUBSTRATE_BIN" --world -c 'printf "%s" "$PATH"')"
printf "%s\n" "$path" | grep -q '^/var/lib/substrate/world-deps/bin:'
printf "%s\n" "$path" | grep -qv '/\\.config/nvm/'
printf "%s\n" "$path" | grep -qv '/\\.pyenv/'
printf "%s\n" "$path" | grep -qv '/\\.cargo/bin'
```

## Case 2 — Enable `npm`: PATH resolution returns the world-deps wrapper

Commands:
```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"

"$SUBSTRATE_BIN" world deps global reset >/dev/null 2>&1 || true
"$SUBSTRATE_BIN" world deps workspace reset >/dev/null 2>&1 || true

"$SUBSTRATE_BIN" world deps global add npm >/dev/null
"$SUBSTRATE_BIN" world deps current sync >/dev/null

resolved="$("$SUBSTRATE_BIN" --world -c 'command -v npm')"
test "$resolved" = "/var/lib/substrate/world-deps/bin/npm"
```

Expected:
- `command -v npm` prints `/var/lib/substrate/world-deps/bin/npm`.

## Case 3 — Exec-time guard: explicit host binary path is denied (exit 5)

Precondition:
- `world_fs.host_visible=true` (the guard defaults to enabled in this mode; see `WDH2-spec.md`).

Commands (example; actual host path varies):
```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
host_npm="$(command -v npm || true)"
test -n "$host_npm"

set +e
"$SUBSTRATE_BIN" --world -c "$host_npm --version" >/dev/null 2>&1
status=$?
set -e
test "$status" -eq 5
```

Expected:
- Exit code `5` with an actionable message indicating host-mounted binary execution is denied and recommending enabling a world-deps package instead.

## Case 4 — Installer scaffolding exists (non-enabling)

Commands:
```bash
test -d "${SUBSTRATE_HOME:-$HOME/.substrate}/deps/packages"
test -d "${SUBSTRATE_HOME:-$HOME/.substrate}/deps/bundles"
test -d "${SUBSTRATE_HOME:-$HOME/.substrate}/deps/scripts"
test -f "${SUBSTRATE_HOME:-$HOME/.substrate}/deps/README.md"
```

Expected:
- Scaffold directories exist.
- No example deps are enabled automatically (enabled lists remain empty unless the operator explicitly adds them).
