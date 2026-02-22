# World Sync — Manual Testing Playbook (WS0–WS7)

This playbook is runnable by a human and is aligned to:
- `docs/project_management/packs/active/world-sync/contract.md`
- `docs/project_management/packs/active/world-sync/filesystem-semantics-spec.md`
- `docs/project_management/packs/active/world-sync/internal-git-spec.md`
- `docs/project_management/packs/active/world-sync/platform-parity-spec.md`

Exit codes:
- Taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Smoke scripts (behavior platforms)

Run your platform smoke script first:
- Linux: `bash smoke/linux-smoke.sh` (expected exit: `0`)
- macOS: `bash smoke/macos-smoke.sh` (expected exit: `0`)

CI smoke dispatch (preferred; self-hosted):
- Choose the exact commit to test:
  - `export SMOKE_CHECKOUT_REF="$(git rev-parse HEAD)"`
- Dispatch the smoke for a checkpoint slice:
  - WS2: `make feature-smoke FEATURE_DIR="docs/project_management/packs/active/world-sync" PLATFORM=behavior WORKFLOW_REF="feat/world-sync" SMOKE_CHECKOUT_REF="$SMOKE_CHECKOUT_REF" SMOKE_SLICE_ID="WS2"`
  - WS5: `make feature-smoke FEATURE_DIR="docs/project_management/packs/active/world-sync" PLATFORM=behavior WORKFLOW_REF="feat/world-sync" SMOKE_CHECKOUT_REF="$SMOKE_CHECKOUT_REF" SMOKE_SLICE_ID="WS5"`
  - WS7: `make feature-smoke FEATURE_DIR="docs/project_management/packs/active/world-sync" PLATFORM=behavior WORKFLOW_REF="feat/world-sync" SMOKE_CHECKOUT_REF="$SMOKE_CHECKOUT_REF" SMOKE_SLICE_ID="WS7"`

Notes:
- Smoke scripts must branch on `SUBSTRATE_SMOKE_SLICE_ID` (exported by the workflow from `SMOKE_SLICE_ID`) so earlier checkpoints do not require later-slice functionality.

## 0) Preconditions (all platforms)

1) Verify `substrate` is on PATH:
```bash
substrate --version
which substrate
```
Expected:
- Exit `0`.

2) Verify `git` is available (required for checkpoint/rollback):
```bash
git --version
```
Expected:
- Exit `0`.

3) Create a clean test workspace:
```bash
export WS_TEST_WS="$(mktemp -d)"
cd "$WS_TEST_WS"
git init -q
echo "WS_TEST_WS=$WS_TEST_WS"
```
Expected:
- Exit `0`.

4) Initialize workspace:
```bash
substrate workspace init .
echo "exit=$?"
```
Expected:
- Exit `0`.
- `.substrate/workspace.yaml` exists.
- `.substrate/policy.yaml` exists.
- `.substrate/git/repo.git/` exists (directory).

## 1) WS0 — Gating + dry-run baseline

1) `workspace sync` must refuse outside a workspace (exit `2`):
```bash
export WS_NO_WS="$(mktemp -d)"
cd "$WS_NO_WS"
substrate workspace sync --dry-run
echo "exit=$?"
```
Expected:
- Exit `2`.
- Stderr contains: `not in a workspace` and `substrate workspace init`.

2) Return to workspace:
```bash
cd "$WS_TEST_WS"
```

## 2) WS2 — Non-PTY from_world apply seam (checkpoint CP1)

This section assumes you can run at least one command in the world backend that produces pending diffs for the session.

### WS2 — Linux/macOS (supported contract)

1) Run a world command that writes inside the workspace (example; adjust as needed):
```bash
substrate --world -c "sh -lc 'echo hello > hello-from-world.txt'"
echo "exit=$?"
```
Expected:
- Exit `0`.

2) Preview pending diffs:
```bash
substrate workspace sync --dry-run --direction from_world --verbose
echo "exit=$?"
```
Expected:
- Exit `0`.
- Output includes a deterministic summary and mentions `hello-from-world.txt` as pending.

3) Apply pending diffs:
```bash
substrate workspace sync --direction from_world --verbose
echo "exit=$?"
```
Expected:
- Exit `0`.
- File exists on host:
```bash
test -f hello-from-world.txt
```
Expected:
- Exit `0`.

4) Re-run sync (no-op):
```bash
substrate workspace sync --direction from_world
echo "exit=$?"
```
Expected:
- Exit `0`.
- Output indicates no pending diffs (no-op).

## 3) WS5 — Direction expansion seam (checkpoint CP2)

This section validates that `from_host` and `both` are accepted and that unsupported backends/platforms are explicit.

### WS5 — Linux/macOS (supported contract)

1) Create/modify a host file:
```bash
echo "host" > host-only.txt
```

2) Run host→world pre-sync (does not mutate host; exit `0` on supported backends):
```bash
substrate workspace sync --dry-run --direction from_host --verbose
echo "exit=$?"
```
Expected:
- Exit `0` on supported backends, else exit `4` with an explicit unsupported message.

3) Run both-direction dry-run:
```bash
substrate workspace sync --dry-run --direction both --verbose
echo "exit=$?"
```
Expected:
- Exit `0` on supported backends, else exit `4` with an explicit unsupported message.

## 4) WS6/WS7 — Internal checkpoint + rollback seam (checkpoint CP3)

1) Create a checkpoint:
```bash
substrate workspace checkpoint --message "cp1"
echo "exit=$?"
```
Expected:
- Exit `0`.

2) Mutate the workspace:
```bash
echo "mutated" > mutation.txt
```

3) Roll back to last checkpoint without `--force` (safety rail refusal; no mutations):
```bash
substrate workspace rollback last
echo "exit=$?"
```
Expected:
- Exit `5`.
- `mutation.txt` still exists:
```bash
test -f mutation.txt
```
Expected:
- Exit `0`.

4) Roll back to last checkpoint with `--force`:
```bash
substrate workspace rollback last --force
echo "exit=$?"
```
Expected:
- Exit `0`.
- `mutation.txt` does not exist:
```bash
test ! -f mutation.txt
```
Expected:
- Exit `0`.

## 5) Cleanup

```bash
rm -rf "$WS_TEST_WS" "$WS_NO_WS"
```
