# Manual Testing Playbook — Workspace Config/Policy Unification (ADR-0008)

This playbook validates the operator-facing contract in:
- `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`

All steps are written to be run in a scratch workspace directory and must not mutate any real project.

## 0) Setup (all platforms)

1. Create a fresh directory and enter it:
   - `mkdir -p /tmp/substrate-wcu && cd /tmp/substrate-wcu`
2. Ensure no workspace exists:
   - `test ! -e .substrate/workspace.yaml`
3. Confirm no override env vars are set:
   - `env | rg '^SUBSTRATE_OVERRIDE_'` (expected: no output)

## 1) Global scope behavior (no workspace)

1. Reset global config and policy to empty patches:
   - `substrate config global reset`
   - `substrate policy global reset`
2. Show global patches:
   - `substrate config global show` (expected: `{}` or empty mapping)
   - `substrate policy global show` (expected: `{}` or empty mapping)
3. Show current/effective views:
   - `substrate config current show` (expected: prints a merged notice to stderr; prints effective config on stdout)
   - `substrate policy current show` (expected: prints a merged notice to stderr; prints effective policy on stdout)
4. Prove overrides affect `current` only when explicitly set:
   - `SUBSTRATE_OVERRIDE_CAGED=1 substrate config current show --json | jq -e '.world.caged==true' >/dev/null`

## 2) Workspace initialization and directory layout

1. Initialize workspace:
   - `substrate workspace init .`
2. Confirm the canonical `.substrate` directory and required files exist:
   - `test -f .substrate/workspace.yaml`
   - `test -f .substrate/policy.yaml`
   - `test -d .substrate/git/repo.git`
3. Confirm `.gitignore` contains the required lines:
   - `rg -n '^\\.substrate/$' .gitignore`
   - `rg -n '^!\\.substrate/workspace\\.yaml$' .gitignore`
   - `rg -n '^!\\.substrate/policy\\.yaml$' .gitignore`

## 3) Workspace scope config patches and reset semantics

1. Show workspace patch:
   - `substrate config workspace show`
2. Set a workspace override:
   - `substrate config workspace set world.caged=false`
3. Confirm current/effective view reflects the workspace override:
   - `substrate config current show --json | jq -e '.world.caged==false' >/dev/null`
4. Reset the workspace key and confirm inheritance:
   - `substrate config workspace reset world.caged`
   - `substrate config current show --json | jq -e '.world.caged==true' >/dev/null`

## 4) Workspace scope policy patches and reset semantics

1. Show workspace policy patch:
   - `substrate policy workspace show`
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
   - `test -d .substrate/git/repo.git`
3. Workspace remove deletes the entire `.substrate/` directory and leaves `.gitignore` unchanged:
   - `cp -a .gitignore .gitignore.before`
   - `substrate workspace remove .`
   - `test ! -d .substrate`
   - `diff -u .gitignore.before .gitignore` (expected: no differences)

