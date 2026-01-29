# Manual Testing Playbook — World FS Granular Allow/Deny (V2)

This playbook is authoritative for ADR-0018 manual validation.

## Preconditions
- Linux host with world-agent running and `world_fs.isolation=full` supported.
- A test workspace with:
  - `./secrets/secret.txt`
  - `./docs/public.txt`
  - optionally `./certs/a.pem`

## Cases

### Case 1 — Deny overrides allow (directory deny)
Policy (global or workspace):
- `world_fs.isolation=full`
- `world_fs.enforcement=strict`
- `world_fs.read.allow_list=['.']`
- `world_fs.read.deny_list=['./secrets/**']`

Expected:
- `ls ./secrets` fails with `Permission denied`
- `cat ./secrets/secret.txt` fails with `Permission denied`
- `cat ./docs/public.txt` succeeds

### Case 2 — Attempted bypass (strict)
With Case 1 policy, attempt:
- `umount /project/secrets` (or `umount $SUBSTRATE_MOUNT_PROJECT_DIR/secrets`)

Expected:
- The command fails (blocked by strict lockdown).
- Subsequent `cat ./secrets/secret.txt` remains denied.

### Case 3 — Discover vs read (visible but not readable)
Policy:
- `world_fs.isolation=full`
- `world_fs.enforcement=strict`
- `world_fs.discover.allow_list=['.']`
- `world_fs.read.allow_list=['.']`
- `world_fs.read.deny_list=['./secrets/secret.txt']`

Expected:
- `ls ./secrets` shows `secret.txt`
- `cat ./secrets/secret.txt` fails with `Permission denied`

### Case 4 — Filename glob deny (snapshot at exec start)
Policy:
- `world_fs.isolation=full`
- `world_fs.enforcement=strict`
- `world_fs.read.allow_list=['.']`
- `world_fs.read.deny_list=['**/*.pem']`

Expected:
- `cat ./certs/a.pem` fails with `Permission denied` (if present)
- Clarify limitation: creating a `.pem` and reading it within the same long-running command is not guaranteed to be blocked.

