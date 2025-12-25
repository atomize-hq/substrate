# World Sync (C0–C9) — Manual Testing Playbook

This playbook validates the world-sync workspace model (`substrate init`) and the core sync workflow (`substrate sync`) end-to-end.

Authoritative docs:
- Plan: `docs/project_management/next/world-sync/plan.md`
- Specs: `docs/project_management/next/world-sync/C0-spec.md` through `docs/project_management/next/world-sync/C9-spec.md`
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Automated smoke scripts

Run the platform smoke script first:
- Linux: `bash docs/project_management/next/world-sync/smoke/linux-smoke.sh`
- macOS: `bash docs/project_management/next/world-sync/smoke/macos-smoke.sh`
- Windows: `pwsh -File docs/project_management/next/world-sync/smoke/windows-smoke.ps1`

## 0) Preconditions

1) Verify tools:
```bash
substrate --version
which substrate
git --version
jq --version
```

2) Create a clean test workspace:
```bash
export WS_TEST_WS="$(mktemp -d)"
cd "$WS_TEST_WS"
git init -q
echo "WS_TEST_WS=$WS_TEST_WS"
```

## 1) C0: init creates workspace directories and `.gitignore` entries

1) Run:
```bash
substrate init
echo "exit=$?"
```

Expected:
- Exit `0`.
- Directories exist:
  - `.substrate/`
  - `.substrate-git/`
  - `.substrate-git/repo.git/`

2) Verify paths:
```bash
test -d .substrate
echo "exit=$?"
test -d .substrate-git
echo "exit=$?"
test -d .substrate-git/repo.git
echo "exit=$?"
```

Expected:
- All commands exit `0`.

3) Verify `.gitignore` contains the workspace entries:
```bash
grep -q '^\\.substrate/$' .gitignore
echo "exit=$?"
grep -q '^\\.substrate-git/$' .gitignore
echo "exit=$?"
```

Expected:
- Both commands exit `0`.

4) Re-run init (idempotent):
```bash
substrate init
echo "exit=$?"
```

Expected:
- Exit `0`.

## 2) C0 gating: sync commands require init

1) Create a second directory without init:
```bash
export WS_TEST_NO_INIT="$(mktemp -d)"
cd "$WS_TEST_NO_INIT"
```

2) Run:
```bash
substrate sync
echo "exit=$?"
substrate checkpoint
echo "exit=$?"
substrate rollback last
echo "exit=$?"
```

Expected:
- Each command exits `2`.
- Each error message points to `substrate init`.

3) Return to the initialized workspace:
```bash
cd "$WS_TEST_WS"
```

## 3) C1: `substrate sync --dry-run` prints effective settings and makes no changes

```bash
substrate sync --dry-run
echo "exit=$?"
```

Expected:
- Exit `0`.
- Output includes the effective settings values for:
  - `sync.direction`
  - `sync.conflict_policy`
  - `sync.exclude`

## 4) C2: non-PTY world→host sync applies overlay changes

Preconditions:
- A world backend is available:
```bash
SUBSTRATE_WORLD=enabled substrate world doctor --json | jq . >/dev/null
echo "exit=$?"
```

Expected:
- Exit `0`.

1) Create a change inside the world (non-PTY):
```bash
SUBSTRATE_WORLD=enabled substrate -c 'sh -c "echo from-world > ws_world_file.txt"'
echo "exit=$?"
```

Expected:
- Exit `0`.

2) Apply world→host:
```bash
substrate sync --direction from_world --conflict-policy prefer_world
echo "exit=$?"
```

Expected:
- Exit `0`.
- File exists and matches content:
```bash
test -f ws_world_file.txt
echo "exit=$?"
grep -q '^from-world$' ws_world_file.txt
echo "exit=$?"
```

Expected:
- Both commands exit `0`.

## 5) Protected paths are never applied by sync

1) Attempt to create a protected-path change inside the world:
```bash
SUBSTRATE_WORLD=enabled substrate -c 'sh -c "echo blocked > .substrate/ws-protected.txt"'
echo "exit=$?"
```

Expected:
- Exit `0`.

2) Apply world→host:
```bash
substrate sync --direction from_world --conflict-policy prefer_world
echo "exit=$?"
```

Expected:
- Exit `5`.
- `.substrate/ws-protected.txt` does not exist on the host:
```bash
test ! -e .substrate/ws-protected.txt
echo "exit=$?"
```

Expected:
- Exit `0`.

## 6) C6/C7: internal git commits, checkpoint, and rollback

1) Record a checkpoint:
```bash
substrate checkpoint
echo "exit=$?"
```

Expected:
- Exit `0`.

2) Verify internal git has at least one commit:
```bash
git --git-dir .substrate-git/repo.git log -1 --oneline >/dev/null
echo "exit=$?"
```

Expected:
- Exit `0`.

3) Mutate a file and roll back:
```bash
printf 'local-change\n' > ws_world_file.txt
grep -q '^local-change$' ws_world_file.txt
echo "exit=$?"

substrate rollback last
echo "exit=$?"

grep -q '^from-world$' ws_world_file.txt
echo "exit=$?"
```

Expected:
- First grep exits `0`.
- `substrate rollback last` exits `0`.
- Final grep exits `0`.

## 7) Cleanup

```bash
cd /
rm -rf "$WS_TEST_WS" "$WS_TEST_NO_INIT"
unset WS_TEST_WS WS_TEST_NO_INIT
```
