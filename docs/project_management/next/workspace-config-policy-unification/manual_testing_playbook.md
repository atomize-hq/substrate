# Manual Testing Playbook — Workspace Config/Policy Unification (ADR-0008)

This playbook validates the operator-facing contract in:
- `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md` (Phase A/B gates)

All steps are written to be run in a scratch workspace directory and must not mutate any real project.

Smoke scripts (automated counterpart; must remain in parity with this playbook):
- Linux: `docs/project_management/next/workspace-config-policy-unification/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/workspace-config-policy-unification/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/workspace-config-policy-unification/smoke/windows-smoke.ps1`

## 0) Setup (all platforms)

1. Create a fresh scratch workspace and scratch Substrate home (so you do not touch your real `~/.substrate`):
   - `rm -rf /tmp/substrate-wcu /tmp/substrate-wcu-home`
   - `mkdir -p /tmp/substrate-wcu /tmp/substrate-wcu-home`
   - `export SUBSTRATE_HOME=/tmp/substrate-wcu-home`
2. Enter the scratch workspace directory:
   - `cd /tmp/substrate-wcu`
3. Ensure no workspace exists:
   - `test ! -e .substrate/workspace.yaml`
4. Confirm no override env vars are set:
   - `env | rg '^SUBSTRATE_OVERRIDE_'` (expected: no output)
5. Confirm no global patch files exist yet:
   - `test ! -e "$SUBSTRATE_HOME/config.yaml"`
   - `test ! -e "$SUBSTRATE_HOME/policy.yaml"`

## 1) Global scope behavior (no workspace)

1. Show does not create patch files:
   - `substrate config global show >/dev/null` and `test ! -e "$SUBSTRATE_HOME/config.yaml"`
   - `substrate policy global show >/dev/null` and `test ! -e "$SUBSTRATE_HOME/policy.yaml"`
2. Initialize global patch files (empty overrides + comment headers):
   - `substrate config global init --force`
   - `substrate policy global init --force`
3. Confirm global patch file headers exist and the patch is empty:
   - `rg -n '^# Substrate config patch' "$SUBSTRATE_HOME/config.yaml"`
   - `rg -n '^# Substrate policy patch' "$SUBSTRATE_HOME/policy.yaml"`
   - `tail -n 1 "$SUBSTRATE_HOME/config.yaml" | rg -x '\\{\\}'`
   - `tail -n 1 "$SUBSTRATE_HOME/policy.yaml" | rg -x '\\{\\}'`
4. Show global patches:
   - `substrate config global show` (expected stdout: `{}`; expected stderr: “global config patch is empty …” note)
   - `substrate policy global show` (expected stdout: `{}`; expected stderr: “global policy patch is empty …” note)
5. Show current/effective views:
   - `substrate config current show` (expected: prints a merged notice to stderr; prints effective config on stdout)
   - `substrate policy current show` (expected: prints a merged notice to stderr; prints effective policy on stdout)
6. Prove overrides affect `current` only when explicitly set:
   - `SUBSTRATE_OVERRIDE_CAGED=1 substrate config current show --json | jq -e '.world.caged==true' >/dev/null`

## 2) Workspace initialization and directory layout

1. Initialize workspace:
   - `substrate workspace init .`
2. Confirm the canonical `.substrate` directory and required files exist:
   - `test -f .substrate/workspace.yaml`
   - `test -f .substrate/policy.yaml`
   - `test -d .substrate/git/repo.git`
3. Confirm the workspace patch file headers exist and the patch is empty:
   - `rg -n '^# Substrate config patch' .substrate/workspace.yaml`
   - `rg -n '^# Substrate policy patch' .substrate/policy.yaml`
   - `tail -n 1 .substrate/workspace.yaml | rg -x '\\{\\}'`
   - `tail -n 1 .substrate/policy.yaml | rg -x '\\{\\}'`
4. Confirm `.gitignore` contains the required lines:
   - `rg -n '^\\.substrate/$' .gitignore`
   - `rg -n '^!\\.substrate/workspace\\.yaml$' .gitignore`
   - `rg -n '^!\\.substrate/policy\\.yaml$' .gitignore`
5. Confirm nested workspace creation is refused:
   - `mkdir -p nested_ws`
   - `(cd nested_ws && substrate workspace init .); test $? -eq 2`

## 3) Workspace scope config patches and reset semantics

1. Show workspace patch:
   - `substrate config workspace show` (expected stdout: `{}`; expected stderr: “workspace config patch is empty …” note)
2. Set a workspace override:
   - `substrate config workspace set world.caged=false`
3. Confirm current/effective view reflects the workspace override:
   - `substrate config current show --json | jq -e '.world.caged==false' >/dev/null`
4. Reset the workspace key and confirm inheritance:
   - `substrate config workspace reset world.caged`
   - `substrate config current show --json | jq -e '.world.caged==true' >/dev/null`
5. Confirm workspace-scoped commands work from a nested directory (workspace discovery from `cwd`):
   - `mkdir -p subdir && cd subdir`
   - `substrate config workspace show >/dev/null`
   - `substrate config workspace set world.caged=false >/dev/null`
   - `substrate config current show --json | jq -e '.world.caged==false' >/dev/null`
   - `cd ..`

## 4) Workspace scope policy patches and reset semantics

1. Show workspace policy patch:
   - `substrate policy workspace show` (expected stdout: `{}`; expected stderr: “workspace policy patch is empty …” note)
2. Set a workspace policy override (example: require world when read-only):
   - `substrate policy workspace set world_fs.mode=read_only world_fs.require_world=true`
3. Confirm current/effective view reflects the workspace override:
   - `substrate policy current show --json | jq -e '.world_fs.mode==\"read_only\" and .world_fs.require_world==true' >/dev/null`
4. Reset workspace policy keys:
   - `substrate policy workspace reset world_fs.mode world_fs.require_world`

## 5) Workspace disable/enable behavior

1. Disable the workspace:
   - `substrate workspace disable .`
2. Confirm current/effective views no longer use workspace patches:
   - `substrate config current show --json | jq -e '.world.caged==true' >/dev/null`
3. Re-enable:
   - `substrate workspace enable .`

## 6) Workspace reset vs remove

1. Ensure workspace has overrides:
   - `substrate config workspace set world.caged=false`
2. Workspace reset clears overrides but preserves internal git:
   - `substrate workspace reset .`
   - `substrate config workspace show` (expected: empty mapping)
   - `rg -n '^# Substrate config patch' .substrate/workspace.yaml` (expected: header preserved)
   - `rg -n '^# Substrate policy patch' .substrate/policy.yaml` (expected: header preserved)
   - `test -d .substrate/git/repo.git`
3. Workspace remove deletes the entire `.substrate/` directory and leaves `.gitignore` unchanged:
   - `cp -a .gitignore .gitignore.before`
   - `substrate workspace remove .`
   - `test ! -d .substrate`
   - `diff -u .gitignore.before .gitignore` (expected: no differences)

## 7) ADR-0012 Phase A/B — `world.deps.enabled` merge + multi-source provenance

This section validates the ADR-0012 requirements that are intentionally implemented as part of ADR-0008.

1. Ensure global + workspace patches exist:
   - `substrate config global init --force`
   - `substrate workspace init .` (if not already initialized)
2. Add one enabled dep at global scope and one at workspace scope (via the config editor):
   - `substrate config global set world.deps.enabled+=bun`
   - `substrate config workspace set world.deps.enabled+=node-runtime`
3. Verify effective merged value contains both items:
   - `substrate config current show --json | jq -e '.world.deps.enabled | index(\"bun\")!=null and index(\"node-runtime\")!=null' >/dev/null`
4. Verify `--explain` indicates a merge strategy and multi-source provenance (stderr):
   - `substrate config current show --json --explain >/dev/null 2> /tmp/wcu-explain.txt`
   - `rg -n 'concat_dedupe_ordered_set' /tmp/wcu-explain.txt`
   - `rg -n 'global_patch' /tmp/wcu-explain.txt`
   - `rg -n 'workspace_patch' /tmp/wcu-explain.txt`
5. Reset key at workspace scope and confirm it remains enabled via global:
   - `substrate config workspace reset world.deps.enabled`
   - `substrate config current show --json | jq -e '.world.deps.enabled | index(\"bun\")!=null' >/dev/null`
