# Manual Testing Playbook — World FS Granular Allow/Deny (V2)

This playbook is authoritative for ADR-0018 manual validation.

## Preferred automation (repeatable)

Run the feature-local smoke script (Linux only; expected exit code `0` on success):
- `bash docs/project_management/next/world-fs-granular-allow-deny/smoke/linux-smoke.sh`

If you need to debug a specific case, follow the manual cases below.

## Preconditions
- Linux host with world-agent running and `world_fs.isolation=full` supported.
- A test workspace with:
  - `./secrets/secret.txt`
  - `./docs/public.txt`
  - optionally `./certs/a.pem`
  - `./outputs/private/` (directory)

## Setup (workspace + policy patch)

In the test workspace root:

1) Ensure the directory is an enabled workspace (exit `0`):
- `substrate workspace init --force`

2) Initialize/reset the workspace policy patch (exit `0`):
- `substrate policy init --force`

3) Confirm you can see the effective policy (exit `0`):
- `substrate policy current show`

Notes:
- This playbook runs commands inside the world using `substrate --world --command '<cmd>'`.
- When a command fails due to access denial, it MUST exit non-zero, and stderr/stdout MUST contain the specified diagnostic substring.

## Cases

### Case 1 — Deny overrides allow (directory deny)
Policy patch (apply via `substrate policy set ...`; expected exit `0`):
- `substrate policy init --force`
- `substrate policy set 'world_fs.mode=read_only' 'world_fs.isolation=full' 'world_fs.require_world=true' 'world_fs.enforcement=strict' 'world_fs.read.allow_list+=.' 'world_fs.read.deny_list+=./secrets/**'`

Expected:
- `substrate --world --command 'ls ./secrets'` exits non-zero; output contains `Permission denied`
- `substrate --world --command 'cat ./secrets/secret.txt'` exits non-zero; output contains `Permission denied`
- `substrate --world --command 'test -n \"$SUBSTRATE_MOUNT_PROJECT_DIR\"'` exits `0`
- `substrate --world --command 'cat \"$SUBSTRATE_MOUNT_PROJECT_DIR/secrets/secret.txt\"'` exits non-zero; output contains `Permission denied`
- `substrate --world --command 'cat ./docs/public.txt'` exits `0`

### Case 2 — Attempted bypass (strict)
With Case 1 policy, attempt:
- `substrate --world --command 'umount /project/secrets'`

Expected:
- The `umount` command exits non-zero; output contains `Operation not permitted` (strict syscall blocking).
- Subsequent `substrate --world --command 'cat ./secrets/secret.txt'` exits non-zero; output contains `Permission denied`.
- Subsequent `substrate --world --command 'cat \"$SUBSTRATE_MOUNT_PROJECT_DIR/secrets/secret.txt\"'` exits non-zero; output contains `Permission denied`.

### Case 3 — Discover vs read (visible but not readable)
Policy:
- `substrate policy init --force`
- `substrate policy set 'world_fs.mode=read_only' 'world_fs.isolation=full' 'world_fs.require_world=true' 'world_fs.enforcement=strict' 'world_fs.discover.allow_list+=.' 'world_fs.read.allow_list+=.' 'world_fs.read.deny_list+=./secrets/secret.txt'`

Expected:
- `substrate --world --command 'ls ./secrets | grep -qx \"secret.txt\"'` exits `0`
- `substrate --world --command 'cat ./secrets/secret.txt'` exits non-zero; output contains `Permission denied`

### Case 4 — Wildcard deny (snapshot at exec start)
Policy:
- `substrate policy init --force`
- `substrate policy set 'world_fs.mode=read_only' 'world_fs.isolation=full' 'world_fs.require_world=true' 'world_fs.enforcement=strict' 'world_fs.read.allow_list+=.' 'world_fs.read.deny_list+=**/*.pem'`

Expected:
- `substrate --world --command 'cat ./certs/a.pem'` exits non-zero; output contains `Permission denied` (if `./certs/a.pem` exists)
- Clarify limitation: creating a `.pem` and reading it within the same long-running command is not guaranteed to be blocked.

### Case 5 — Write deny (read-only failure)
Policy:
- `substrate policy init --force`
- `substrate policy set 'world_fs.mode=writable' 'world_fs.isolation=full' 'world_fs.require_world=true' 'world_fs.enforcement=strict' 'world_fs.write.allow_list+=.' 'world_fs.write.deny_list+=./outputs/private/**'`

Expected:
- `substrate --world --command 'mkdir -p ./outputs/private/x'` exits non-zero; output contains `Read-only file system`

### Case 6 — Discover deny (invisible subtree)
Policy:
- `substrate policy init --force`
- `substrate policy set 'world_fs.mode=read_only' 'world_fs.isolation=full' 'world_fs.require_world=true' 'world_fs.enforcement=strict' 'world_fs.discover.allow_list+=.' 'world_fs.discover.deny_list+=./secrets/**' 'world_fs.read.allow_list+=.'`

Expected:
- `substrate --world --command 'ls ./secrets'` exits non-zero; output contains `Permission denied`
- `substrate --world --command 'cat ./secrets/secret.txt'` exits non-zero; output contains `Permission denied`
