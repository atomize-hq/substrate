# Agent Hub Isolation Hardening (I0–I5) — Manual Testing Playbook

This playbook validates the I0–I5 policy schema and Linux isolation semantics end-to-end.

Authoritative docs:
- ADR: `docs/project_management/next/p0-agent-hub-isolation-hardening/ADR-0001-agent-hub-runtime-config-and-isolation.md`
- Specs: `docs/project_management/next/p0-agent-hub-isolation-hardening/I0-spec.md` through `docs/project_management/next/p0-agent-hub-isolation-hardening/I5-spec.md`
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Automated smoke scripts

Run the platform smoke script first:
- Linux: `bash docs/project_management/next/p0-agent-hub-isolation-hardening/smoke/linux-smoke.sh`
- macOS: `bash docs/project_management/next/p0-agent-hub-isolation-hardening/smoke/macos-smoke.sh`
- Windows: `pwsh -File docs/project_management/next/p0-agent-hub-isolation-hardening/smoke/windows-smoke.ps1`

## 0) Preconditions

1) Verify the CLI:
```bash
substrate --version
which substrate
```

2) Create a clean test workspace:
```bash
export IH_TEST_WS="$(mktemp -d)"
cd "$IH_TEST_WS"
echo "IH_TEST_WS=$IH_TEST_WS"
```

## 1) I0: strict `.substrate-profile` schema validation

1) Invalid profile (missing `world_fs`) fails fast:
```bash
cat > .substrate-profile <<'YAML'
version: 1
YAML

SUBSTRATE_WORLD=disabled substrate -c 'true'
echo "exit=$?"
```

Expected:
- Exit `2`.
- Error output mentions `.substrate-profile` and `world_fs`.

2) Valid minimal profile parses:
```bash
cat > .substrate-profile <<'YAML'
world_fs:
  require_world: false
  mode: writable
  cage: project
  read_allowlist:
    - "**"
  write_allowlist:
    - "**"
YAML

SUBSTRATE_WORLD=disabled substrate -c 'true'
echo "exit=$?"
```

Expected:
- Exit `0`.

## 2) I1: required world vs host fallback on backend unavailability

This section simulates a missing world backend socket via `SUBSTRATE_WORLD_SOCKET`.

1) Host fallback is allowed when `world_fs.require_world=false`:
```bash
cat > .substrate-profile <<'YAML'
world_fs:
  require_world: false
  mode: writable
  cage: project
  read_allowlist:
    - "**"
  write_allowlist:
    - "**"
YAML

SUBSTRATE_WORLD=enabled SUBSTRATE_WORLD_ENABLED=1 SUBSTRATE_WORLD_SOCKET=/tmp/substrate-test-missing.sock substrate -c 'echo host-fallback-ok'
echo "exit=$?"
```

Expected:
- Exit `0`.
- Output contains a single warning mentioning world backend unavailability and `SUBSTRATE_WORLD_SOCKET`.
- Command output includes `host-fallback-ok`.

2) Fail closed when `world_fs.require_world=true`:
```bash
cat > .substrate-profile <<'YAML'
world_fs:
  require_world: true
  mode: writable
  cage: project
  read_allowlist:
    - "**"
  write_allowlist:
    - "**"
YAML

SUBSTRATE_WORLD=enabled SUBSTRATE_WORLD_ENABLED=1 SUBSTRATE_WORLD_SOCKET=/tmp/substrate-test-missing.sock substrate -c 'echo must-not-run'
echo "exit=$?"
```

Expected:
- Exit `3`.
- Error output mentions that world execution is required and references `substrate world doctor --json`.
- Output does not include `must-not-run`.

## 3) I2/I3: full cage semantics (Linux)

This section validates full cage behavior when available and validates fail-closed behavior when it is not.

1) Request full cage:
```bash
cat > .substrate-profile <<'YAML'
world_fs:
  require_world: true
  mode: writable
  cage: full
  read_allowlist:
    - "**"
  write_allowlist:
    - "**"
YAML
```

2) Validate full cage allows `/tmp` and blocks `/etc` writes:
```bash
SUBSTRATE_WORLD=enabled substrate -c 'sh -c "touch /tmp/substrate-ih-tmp-write && echo tmp-ok"'
echo "exit=$?"

SUBSTRATE_WORLD=enabled substrate -c 'sh -c "touch /etc/substrate-ih-etc-write && echo etc-unexpected"'
echo "exit=$?"
```

Expected:
- First command exits `0` and prints `tmp-ok`.
- Second command exits non-zero.
- If full cage cannot be created, both commands exit non-zero and print an actionable error; the run must not fall back to host execution.

## 4) I4: Landlock detection is surfaced in `world doctor` (Linux)

```bash
SUBSTRATE_WORLD=enabled substrate world doctor --json | jq -e '.. | objects | select(has("landlock")) | .landlock' >/dev/null
echo "exit=$?"
```

Expected:
- Exit `0`.

## 5) Cleanup

```bash
cd /
rm -rf "$IH_TEST_WS"
unset IH_TEST_WS
```
