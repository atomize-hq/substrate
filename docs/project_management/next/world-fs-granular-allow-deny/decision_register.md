# Decision Register — World FS Granular Allow/Deny (V2) + Strict Deny

This decision register supports:
- `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

Each decision is recorded as A/B with explicit selection.

## DR-0001 — Policy schema versioning
- Option A: Introduce `world_fs.read|write|discover.{allow_list,deny_list}` as a breaking V2 schema (old keys hard error).
- Option B: Preserve old keys and add new optional keys with compatibility shims.
- Selected: A
- Rationale: “accepted but not enforced” must be structurally impossible; user requested no backwards compatibility.

## DR-0002 — Deny-overrides-allow implementation
- Option A: Implement denies via mount masking inside the per-command mount namespace.
- Option B: Implement denies via an LSM deny policy (AppArmor/SELinux) or syscall interposition.
- Selected: A
- Rationale: Landlock is allowlist-only; mount masking is the only practical per-session subtraction mechanism in this architecture.

## DR-0003 — Deny enforcement posture
- Option A: `strict` deny: denies are a hard security boundary; workload cannot undo masks.
- Option B: `best_effort` deny: apply masks at startup only; workload can undo them.
- Selected: Both, as a policy lever (strict recommended default when denies exist).

## DR-0004 — Isolation support
- Option A: Support deny lists only in `world_fs.isolation=full`.
- Option B: Support deny lists in both `full` and `workspace`.
- Selected: A
- Rationale: `workspace` is explicitly not a full pivot-root view; denying reads/visibility there is not reliable and must not be silently ignored.

## DR-0005 — `discover` dimension (directory visibility)
- Option A: Add `discover` as an optional dimension (defaults to mirror `read` when omitted).
- Option B: Keep `read` as a single dimension bundling directory listing and file reads.
- Selected: A
- Rationale: “visible but not readable” and “can access known child but not list parent” must be expressible without side effects.

## DR-0006 — Wildcard denies (`**/*.pem`) semantics
- Option A: Snapshot-at-exec-start semantics (scan and mask existing matches each exec).
- Option B: Promise “always denied” semantics via inotify/fanotify or deep kernel features.
- Selected: A
- Rationale: robust “always denied” cannot be guaranteed for within-process creation/rename; do not overpromise.

## DR-0007 — Where glob resolution happens
- Option A: Resolve wildcard deny matches inside the helper (after mounts exist), per exec.
- Option B: Resolve wildcard deny matches on the host or in world-agent service before entering the mount namespace.
- Selected: A
- Rationale: helper sees the authoritative in-namespace filesystem view and can fail closed consistently at the final chokepoint.

## DR-0008 — Strict-mode bypass prevention mechanism
- Option A: Enforce strict mode by dropping mount authority for the workload (cap drop) and blocking mount-family syscalls (seccomp) before exec.
- Option B: Rely on convention only (document “don’t mount/umount”) or on Landlock alone.
- Selected: A
- Rationale: without this, mount-based denies are bypassable; Landlock cannot express deny exceptions under broad allows.
