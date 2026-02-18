# Workspace Sync — Filesystem Model (Reality, Linux)

Scope: this document explains how `substrate workspace sync` behaves **today** on Linux given the
current world implementation (overlay-based “world” sessions). This is developer-facing and is
grounded in the codebase, not an aspirational contract.

If you are looking for the *operator-facing* CLI contract, see `docs/reference/cli/workspace_sync.md`.

## Mental model (one sentence)

On Linux, the “world filesystem” is the **live host workspace** (overlay lower layer) plus an
optional **world-only overlay upper layer** for pending changes; `workspace sync` controls when the
world-only layer is applied to the host and/or discarded.

## Glossary

- **Host workspace**: the real directory on the developer machine (e.g. `/home/<user>/repo`).
- **World session**: a reusable isolated context created by `world-agent` (one per compatible
  `WorldSpec` in the daemon process).
- **Overlay lower**: the host workspace directory (bind-mounted for overlay mounting).
- **Overlay upper/work**: where world-only writes and deletes are materialized.
- **Overlay merged**: what world processes see when reading/writing the project directory.
- **Pending diff**: the set of workspace-relative paths currently represented in overlay upper/work
  (writes/mods/deletes).

## Where the behavior lives (code pointers)

Shell (“workspace sync” CLI and apply logic):
- `crates/shell/src/execution/workspace_cmd.rs`
  - Fetch pending diff, filter excludes, detect conflicts, apply, then clear by `diff_id`.
  - Implements directions `from_world | from_host | both` and conflict policy.

World agent (pending diff record, conditional clear, reconciliation):
- `crates/world-agent/src/service.rs`
  - `pending_diff()` builds `PendingDiffRecordV1` (includes `session_started_at` + `diff_id`).
  - `pending_diff_clear()` clears only if the `diff_id` still matches.
  - `pending_diff_reconcile()` discards selected overlay paths (host-preferred reconciliation).

World backend and overlay tracking:
- `crates/world/src/lib.rs`: `LinuxLocalBackend::{pending_diff, clear_pending_diff, discard_pending_paths}`
- `crates/world/src/session.rs`: `SessionWorld` owns a persistent `OverlayFs` per session.
  - `SessionWorld::clear_pending_diff()` discards the overlay upper/work (resetting “world-only” state).
- `crates/world/src/overlayfs/mod.rs`: overlay mount/unmount + `discard_paths()`.
- `crates/world/src/overlayfs/utils.rs`: computes a pending diff by walking the overlay upper layer.

Specs / planning-pack references (authoritative intent, but not the source of truth for this doc):
- `docs/project_management/packs/active/world-sync/filesystem-semantics-spec.md`
- `docs/project_management/packs/active/world-sync/WS2-spec.md`
- `docs/project_management/packs/active/world-sync/WS5-spec.md`

## Key invariants (current reality)

### 1) Host changes are generally visible in world immediately

Because the overlay lower is the **live host workspace**, any host create/modify/delete is visible
in world reads as soon as it happens *unless* the path is shadowed by an overlay upper entry.

Shadowing happens when the world has a pending change for the same path. In that case, the overlay
merged view can continue to show the world version until the upper entry is discarded/cleared.

### 2) World changes accumulate as overlay upper entries (“pending diffs”)

When a world command creates/modifies/deletes a path under the project root, it is represented in
overlay upper/work until cleared:

- Creates/mods: materialized as real files/dirs under the overlay upper directory.
- Deletes: materialized as overlayfs whiteouts (`.wh.<basename>` files) in the upper directory.

Pending diff computation walks overlay upper and interprets whiteouts:
- `crates/world/src/overlayfs/utils.rs` (`compute_diff()` + `.wh.` handling).

### 3) `workspace sync` (from_world) applies pending world diffs to host, then clears the snapshot

The host apply path:
1) Fetch pending diff record from world-agent (includes `diff_id`).
2) Apply deletes then writes/mods (after preflight validation).
3) Attempt to clear the pending diff snapshot using `pending_diff_clear(diff_id=...)`.

If the clear step fails, Substrate must not clear “whatever is current”:
- `crates/shell/src/execution/workspace_cmd.rs` (clear refusal path).
- `docs/project_management/packs/active/world-sync/filesystem-semantics-spec.md` (“Clear/ack semantics”).

### 4) Clearing pending diffs resets the world-only overlay state

Clearing discards overlay upper/work (and unmounts the overlay) for that session:
- `crates/world/src/session.rs` (`clear_pending_diff()` → `OverlayFs::cleanup()`).

After a clear, subsequent world commands see only the live host baseline until a new world change
recreates overlay upper entries again.

### 5) `from_host` is reconciliation of shadowed paths, not “copy host into world”

Because host is already the baseline, “host→world” sync is implemented as:
- identify paths shadowed by pending overlay entries (union of writes/mods/deletes),
- for conflicting paths, decide whether to discard (prefer host) or keep (prefer world),
- perform the decision by deleting overlay upper entries / whiteouts for selected paths.

This is implemented by `pending_diff_reconcile_v1`:
- shell: `crates/shell/src/execution/workspace_cmd.rs`
- agent: `crates/world-agent/src/service.rs` (`pending_diff_reconcile()`)
- backend: `crates/world/src/lib.rs` (`discard_pending_paths()`)
- overlay: `crates/world/src/overlayfs/mod.rs` (`discard_paths()`)

### 6) Conflict detection uses `session_started_at` (world session start)

On apply/reconciliation, Substrate considers a host path “in conflict” if:
- host path exists, and
- host mtime is strictly greater than `session_started_at`.

Implications:
- This is a coarse-grained “anything touched after the world session began” signal.
- It is not a 3-way merge; it is a policy gate for deciding whether to apply/discard world changes.

## Common “surprise” scenarios (expected today)

### Scenario: “I synced a world file to host; deleting it on host also deletes it in world”

This is expected.

Once a world-created file is applied to host and the pending diff is cleared, the file becomes part
of the shared baseline. Removing it on host removes it from what the world sees (overlay lower).

### Scenario: “`from_host` doesn’t upload host-only new files to world”

Also expected.

Host-only files are already visible in world via the baseline. `from_host` exists for the case
where the world has a pending upper entry that is hiding the host’s current version.

## Auto-sync (`sync.auto_sync=true`) — current reality

When `sync.auto_sync=true`, the shell will opportunistically run a workspace sync automatically
after successful commands, using the same engine as `substrate workspace sync`:

- One-shot execution path (`substrate -c "..."`):
  - Hook lives in `crates/shell/src/execution/routing/dispatch/exec.rs` (post-success hook).
- Async REPL path (`substrate` interactive session):
  - Hook runs on REPL exit and lives in `crates/shell/src/repl/async_repl.rs`.

Implementation details:
- Auto-sync is wired through `crates/shell/src/execution/auto_sync.rs`.
- Effective direction handling:
  - `sync.direction=from_host` → no-op (auto-sync does not run).
  - `sync.direction=from_world|both` → calls the `workspace sync` engine with that direction.
- Failures are surfaced as `auto-sync failed: ...` and propagate a non-zero exit code.

## Debugging workflows

### Inspect what sync would do

- `substrate workspace sync --dry-run --verbose`
  - shows pending diff counts and (with verbose) the `diff_id` and decisions.

### Inspect overlay storage on Linux

The overlay base directory is selected by uid/runtime dirs:
- `crates/world/src/overlayfs/layering.rs` (`choose_base_dir()`).

Within that base directory, overlay state is stored per world id (`wld_<uuid>`):
- `crates/world/src/session.rs` (world ids).

## Notes on future evolution (non-authoritative)

The current model is intentionally “live host lower + world upper”.

If we want a UX where “host changes do not enter world until explicitly synced”, the world lower
cannot be the live host workspace. That would require introducing a snapshot baseline for the world
(e.g., a materialized snapshot dir) and explicit host→world apply semantics. Internal git
(`workspace checkpoint`/`rollback`) is a plausible building block for snapshot materialization, but
it is not currently wired into world mounts.
