# Agent Hub Isolation Hardening (I0–I9) — Manual Testing Playbook

This playbook validates the I0–I9 policy schema and Linux isolation semantics end-to-end.

Authoritative docs:
- ADR: `docs/project_management/next/p0-agent-hub-isolation-hardening/ADR-0001-agent-hub-runtime-config-and-isolation.md`
- Specs: `docs/project_management/next/p0-agent-hub-isolation-hardening/I0-spec.md` through `docs/project_management/next/p0-agent-hub-isolation-hardening/I9-spec.md`
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
case "$IH_TEST_WS" in /tmp/*) ;; *) echo "NOTE: IH_TEST_WS is not under /tmp; set TMPDIR=/tmp for I9 /tmp-rooted coverage." ;; esac
```

## 1) I0: strict `.substrate-profile` schema validation

1) Invalid profile (missing `world_fs`) fails fast:
```bash
cat > .substrate-profile <<'YAML'
id: ih-test
name: IH Test Policy
YAML

SUBSTRATE_WORLD=disabled substrate -c 'true'
echo "exit=$?"
```

Expected:
- Exit is non-zero (exact numeric mapping is not specified by I0–I5; refer to `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`).
- Error output mentions missing `world_fs` and provides an example fix.

2) Valid minimal profile parses:
```bash
cat > .substrate-profile <<'YAML'
id: ih-test
name: IH Test Policy
world_fs:
  require_world: false
  mode: writable
  cage: project
  read_allowlist:
    - "*"
  write_allowlist: []
YAML

SUBSTRATE_WORLD=disabled substrate -c 'true'
echo "exit=$?"
```

Expected:
- Command succeeds.

## 2) I1: required world vs host fallback on backend unavailability

This section simulates a missing world backend socket via `SUBSTRATE_WORLD_SOCKET`.

1) Host fallback is allowed when `world_fs.require_world=false`:
```bash
cat > .substrate-profile <<'YAML'
id: ih-test
name: IH Test Policy
world_fs:
  require_world: false
  mode: writable
  cage: project
  read_allowlist:
    - "*"
  write_allowlist: []
YAML

SUBSTRATE_WORLD=enabled SUBSTRATE_WORLD_ENABLED=1 SUBSTRATE_WORLD_SOCKET=/tmp/substrate-test-missing.sock substrate -c 'echo host-fallback-ok'
echo "exit=$?"
```

Expected:
- Command succeeds.
- Output contains a single warning mentioning world backend unavailability.
- Command output includes `host-fallback-ok`.

2) Fail closed when `world_fs.require_world=true`:
```bash
cat > .substrate-profile <<'YAML'
id: ih-test
name: IH Test Policy
world_fs:
  require_world: true
  mode: writable
  cage: project
  read_allowlist:
    - "*"
  write_allowlist: []
YAML

SUBSTRATE_WORLD=enabled SUBSTRATE_WORLD_ENABLED=1 SUBSTRATE_WORLD_SOCKET=/tmp/substrate-test-missing.sock substrate -c 'echo must-not-run'
echo "exit=$?"
```

Expected:
- Exit is non-zero (exact numeric mapping is not specified by I0–I5; refer to `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`).
- Error output mentions that world execution is required and that the world backend is unavailable (typically includes a hint to run `substrate world doctor --json` and/or to check `systemctl status substrate-world-agent.socket`).
- Output does not include `must-not-run`.

## 3) I2/I3: full cage semantics (Linux)

This section validates full cage behavior when available and validates fail-closed behavior when it is not.

1) Request full cage:
```bash
cat > .substrate-profile <<'YAML'
id: ih-test
name: IH Test Policy
world_fs:
  require_world: true
  mode: writable
  cage: full
  read_allowlist:
    - "*"
  write_allowlist: []
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
- First command succeeds and prints `tmp-ok`.
- Second command exits non-zero.
- If full cage cannot be created, both commands exit non-zero and print an actionable error; the run must not fall back to host execution.

## 4) I4: Landlock detection is surfaced in `world doctor` (Linux)

```bash
SUBSTRATE_WORLD=enabled substrate world doctor --json | jq -e '.landlock | type == "object" and (.supported | type == "boolean") and (.abi | type == "number")' >/dev/null
echo "exit=$?"
```

Expected:
- Command succeeds (the report includes a structured `.landlock` object).
- On Landlock-capable hosts, `.landlock.supported` should be `true` (inspect via `substrate world doctor --json | jq '.landlock'`).

## 5) I5: verification tooling (optional)

I5 is documentation alignment plus a minimal verification script/checklist.

On Linux (repo checkout), you can run the verification script:

```bash
bash scripts/linux/agent-hub-isolation-verify.sh
```

Expected:
- Script exits `0` and prints `Verification complete. Logs saved under: ...`.
- If full cage is unavailable, the script prints actionable hints (unprivileged user namespaces / CAP_SYS_ADMIN).

## 6) I6: `substrate world verify` (enforcement verification)

Run the end-to-end verifier (works from any directory; does not require a repo checkout):

```bash
SUBSTRATE_WORLD=enabled substrate world verify
echo "exit=$?"
```

Expected:
- Exit is `0`.
- Output shows all checks and ends with `PASS`.

Optional: validate the machine report shape:

```bash
SUBSTRATE_WORLD=enabled substrate world verify --json | jq -e '
  .ok == true
  and (.checks | type == "array")
  and ([.checks[].id] | index("world_backend") != null)
  and ([.checks[].id] | index("read_only_relative_write") != null)
  and ([.checks[].id] | index("read_only_absolute_write") != null)
  and ([.checks[].id] | index("full_cage_host_isolation") != null)
' >/dev/null
echo "exit=$?"
```

## 7) I7: Playbook/spec alignment sanity (docs)

This triad is documentation-only. Spot-check:
- Every `.substrate-profile` snippet includes top-level `id` and `name`.
- Expected outputs do not claim strict numeric exit codes unless a spec defines them.
- Any behavior differences are described as “typical” and include actionable next steps.

## 8) I8: I1 noise reduction (single warning / single error)

Re-run the I1 “missing world socket” scenarios and verify warning/error line counts.

1) When fallback is allowed (`world_fs.require_world=false`): exactly one warning.

```bash
SUBSTRATE_WORLD=enabled SUBSTRATE_WORLD_ENABLED=1 SUBSTRATE_WORLD_SOCKET=/tmp/substrate-test-missing.sock \
  substrate -c 'echo host-fallback-ok' 2>"$IH_TEST_WS/fallback.stderr"
grep -c '^substrate: warn:' "$IH_TEST_WS/fallback.stderr"
```

Expected:
- Command succeeds and prints `host-fallback-ok`.
- The grep count is `1`.

2) When world is required (`world_fs.require_world=true`): exactly one error (and no warning).

```bash
SUBSTRATE_WORLD=enabled SUBSTRATE_WORLD_ENABLED=1 SUBSTRATE_WORLD_SOCKET=/tmp/substrate-test-missing.sock \
  substrate -c 'echo must-not-run' 2>"$IH_TEST_WS/required.stderr" || true
grep -c '^Error:' "$IH_TEST_WS/required.stderr"
grep -c '^substrate: warn:' "$IH_TEST_WS/required.stderr"
```

Expected:
- The command fails.
- `grep -c '^Error:'` prints `1`.
- `grep -c '^substrate: warn:'` prints `0`.
- Output does not include `must-not-run`.

## 9) I9: Full cage robustness (`/tmp` projects + `world verify` full cage)

If `IH_TEST_WS` is `/tmp`-rooted (the default on Linux), this playbook exercises the I9 regression path, and
`substrate world verify` runs its temporary projects under the OS temp directory.

Validate the full-cage verifier check explicitly:

```bash
SUBSTRATE_WORLD=enabled substrate world verify --json | jq -e '
  .checks[] | select(.id == "full_cage_host_isolation") | .status == "pass"
' >/dev/null
echo "exit=$?"
```

Expected:
- Exit is `0` and the full-cage check status is `pass`.
- If it is `skip`, follow the hint in the report (typically: enable unprivileged user namespaces or run
  with CAP_SYS_ADMIN).

## 10) Cleanup

```bash
cd /
rm -rf "$IH_TEST_WS"
unset IH_TEST_WS
```
