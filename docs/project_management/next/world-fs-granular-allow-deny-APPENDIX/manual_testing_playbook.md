# Manual Testing Playbook — world-fs-granular-allow-deny-appendix (Authoritative)

## Scope
- This playbook validates Appendix A + B behavior after implementation.
- Smoke scripts:
  - `smoke/linux-smoke.sh`

## Preconditions
- `substrate` is installed on PATH.
- World backend is healthy when fail-closed routing cases are executed.

Recommended: run all cases in a clean sandbox (no writes to a real `$HOME`):

```bash
export AX_TEST_ROOT="$(mktemp -d)"
export SUBSTRATE_HOME="$AX_TEST_ROOT/substrate-home"
mkdir -p "$AX_TEST_ROOT/workspace"
cd "$AX_TEST_ROOT/workspace"
echo "AX_TEST_ROOT=$AX_TEST_ROOT"
```

## Cases

### Case 1 — Schema V3 hard errors
Run:
- `bash docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/smoke/linux-smoke.sh`

Expected:
- Exit `0`.

### Case 2 — Routing fail-closed hard errors (pre-exec)
Run:

```bash
cd "$AX_TEST_ROOT/workspace"
substrate workspace init --force >/dev/null
substrate policy init --force >/dev/null

substrate policy set 'world_fs.fail_closed.routing=true' >/dev/null

SUBSTRATE_WORLD=disabled substrate -c 'echo must-not-run'
echo "exit=$?"
```

Expected:
- Hard error before execution (exit `2`).
- Output does not contain `must-not-run`.

### Case 3 — Routing fail-closed runtime failure mapping
Run:

```bash
cd "$AX_TEST_ROOT/workspace"
substrate workspace init --force >/dev/null
substrate policy init --force >/dev/null

substrate policy set 'world_fs.fail_closed.routing=true' >/dev/null

SUBSTRATE_WORLD=enabled SUBSTRATE_WORLD_ENABLED=1 \
  SUBSTRATE_WORLD_SOCKET=/tmp/substrate-test-missing.sock \
  substrate -c 'echo must-not-run'
echo "exit=$?"
```

Expected:
- Exit `3`.
- Output does not contain `must-not-run`.

### Case 3b — Routing fallback warning when `host_visible=false` + fail-closed routing disabled
Run:

```bash
cd "$AX_TEST_ROOT/workspace"
substrate workspace init --force >/dev/null
substrate policy init --force >/dev/null

substrate policy set 'world_fs.host_visible=false' 'world_fs.fail_closed.routing=false' >/dev/null

rm -f stdout.log stderr.log
SUBSTRATE_WORLD=enabled SUBSTRATE_WORLD_ENABLED=1 \
  SUBSTRATE_WORLD_SOCKET=/tmp/substrate-test-missing.sock \
  substrate -c 'echo must-run' >stdout.log 2>stderr.log
echo "exit=$?"

cat stdout.log
cat stderr.log

grep -F 'world routing failed; falling back to host' stderr.log
grep -F 'world_fs.host_visible=false was requested' stderr.log
grep -F 'world_fs.fail_closed.routing=false allows fallback' stderr.log
```

Expected:
- `stdout.log` contains `must-run`.
- Each `grep` command exits `0`.

### Case 4 — Caging required
Run:

```bash
cd "$AX_TEST_ROOT/workspace"
mkdir -p subdir

substrate workspace init --force >/dev/null
substrate policy init --force >/dev/null
substrate config init --force >/dev/null

substrate config set world.caged=true world.anchor_mode=project >/dev/null
substrate policy set 'world_fs.caged_required=true' >/dev/null

SUBSTRATE_WORLD=disabled substrate
```

In the REPL:
1. Run: `pwd` (record it).
2. Run: `cd ..`
3. Run: `pwd` again.

Expected:
- The attempted escape is blocked (the second `pwd` remains inside the workspace root).
- A human-readable note is printed indicating the rejected destination and the caging boundary.

### Case 5 — REPL exit note
Run:

```bash
cd "$AX_TEST_ROOT/workspace"
mkdir -p exit-target

substrate workspace init --force >/dev/null
substrate policy init --force >/dev/null
substrate config init --force >/dev/null

substrate config set repl.exit_cwd=last_world >/dev/null

SUBSTRATE_WORLD=disabled substrate
```

In the REPL:
1. Run: `cd exit-target`
2. Run: `exit`

Expected:
- A note line prints:
  - `substrate: note: returning to host cwd: <path>`
- The printed `<path>` equals `$AX_TEST_ROOT/workspace/exit-target`.
