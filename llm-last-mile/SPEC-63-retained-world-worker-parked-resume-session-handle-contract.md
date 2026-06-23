# Spec: Retained World Worker Parked-Resume Session-Handle Contract

Source authorities:
- [handoffs/2026-06-23-retained-world-worker-lifecycle-debug.md](../handoffs/2026-06-23-retained-world-worker-lifecycle-debug.md)
- [handoffs/2026-06-23-design-lineage-audit-retained-worker-resume.md](../handoffs/2026-06-23-design-lineage-audit-retained-worker-resume.md)
- [handoffs/2026-06-22-211455-spec-62-world-worker-retained-debug.md](../handoffs/2026-06-22-211455-spec-62-world-worker-retained-debug.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [DESIGN-host-orchestrator-tool-invocation-surface.md](./DESIGN-host-orchestrator-tool-invocation-surface.md)
- [DESIGN-internal-toolbox-transport-and-session-binding.md](./DESIGN-internal-toolbox-transport-and-session-binding.md)
- [docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md](../docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
- [SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md](./SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md)
- [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)
- [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
- [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
- [`crates/shell/src/execution/agent_runtime/session.rs`](../crates/shell/src/execution/agent_runtime/session.rs)

Phase: `SPECIFY`
Status: draft for review

## Assumptions

ASSUMPTIONS I'M MAKING:

1. `SPEC-62` remains the authority for direct `cli:codex-world` bootstrap compatibility only; this spec must not reopen auth/config bootstrap projection, exact-backend allowlists, or broader compatibility-bridge retirement work.
2. `DESIGN-world-worker-lifecycle-model.md` is now the current lifecycle authority, including the rule that once authoritative retained identity plus resumable session identity are surfaced, clean bootstrap exit is `running -> parked`, not retained-worker closeout.
3. For the active bug seam, the authoritative retained continuity tuple is: exact retained worker identity (`participant_id`) plus exact orchestration/world binding plus surfaced resumable session identity (`uaa_session_id` or equivalent surfaced backend session handle needed for resume).
4. `continue_world_worker` remains an exact-target retained action; this slice does not introduce fuzzy worker lookup, alternate selectors, or new public tool vocabulary.
5. A parked retained worker may have no active bootstrap or submitted-turn process in flight while still remaining valid for later `submit_turn` / resume.
6. Explicit terminal lifecycle transitions (`stopped`, `invalidated`, or other explicit retained closeout already allowed by lifecycle authority) remain the only valid reasons to destroy retained continuity after the authoritative identity tuple has been surfaced.

If any of these are wrong, correct them before implementation.

## Objective

Fix the bounded retained-world-worker lifecycle seam where shell/control-plane truth continues to route later `continue_world_worker` turns to a retained participant with surfaced resumable identity, but `world-service` currently deletes that retained participant from its in-memory registry as soon as the bootstrap process exits cleanly.

This spec freezes the minimum contract needed so that:

1. `spawn_world_worker` registers a retained worker receipt honestly,
2. surfaced resumable session identity becomes part of retained continuity authority,
3. clean bootstrap exit after that surfacing transitions the worker to `parked` instead of unregistering it,
4. later `continue_world_worker` targets that parked retained worker by exact participant identity and exact binding,
5. and the regression boundary proves: `registered retained worker -> bootstrap exits cleanly -> later continue succeeds`.

This spec is intentionally surgical. It owns the retained parked/resume lifecycle seam only. It does **not** own bootstrap compatibility inputs, config projection, workspace reconciliation, broad host auto-attach redesign, MCP/app-runtime concerns, or unrelated retained-worker behavior.

## Tech Stack

- Rust workspace (`cargo`)
- `crates/world-service` retained member runtime / submit-turn seam
- `crates/shell` orchestration routing, retained participant state, and session-handle persistence
- existing world-member streaming/request contracts already used by `spawn_world_worker` and `continue_world_worker`
- `llm-last-mile/` spec authority for bounded lifecycle slices

## Contract Slice

### In scope

1. `spawn_world_worker` receipt and registration semantics for retained workers.
2. Preservation of retained registry membership across clean bootstrap exit **when** authoritative retained identity and resumable session identity have already been surfaced.
3. The resume handoff contract between surfaced `uaa_session_id` (or equivalent surfaced session handle) and later `submit_turn` / `continue_world_worker` execution.
4. `continue_world_worker` against a parked retained worker with no active bootstrap process still running.
5. Regression coverage proving the joined lifecycle seam end-to-end at the retained-worker contract layer.

### Explicitly out of scope

1. Auth/config bootstrap compatibility or any widening of `SPEC-62` bridge inputs.
2. General config projection, profile overlays, project `.codex` overlays, or runtime-family config parity work.
3. Workspace sync, reconciliation, or host-visible file materialization semantics.
4. Broad host auto-attach, inbox, router, or review-projection redesign.
5. MCP, apps/connectors, plugin, skill, or app-runtime concerns.
6. New worker-selection semantics beyond exact retained target identity.
7. Unrelated retained-worker verbs or policies unless they are required to keep this lifecycle seam coherent.

## Commands

Build:

```bash
cargo build --workspace
```

Format:

```bash
cargo fmt --all -- --check
```

Lint:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Current targeted repo-truth checks:

```bash
rg -n "unregister_member|remember_uaa_session_id|submit_turn|find_submit_target|validate_submit_target_slot" \
  crates/world-service/src/member_runtime.rs

rg -n "resolve_internal_continue_world_dispatch_target|set_uaa_session_id|resume_eligible|continue_world_worker" \
  crates/shell/src/execution/agent_runtime/state_store.rs \
  crates/shell/src/execution/agent_runtime/session.rs \
  crates/shell/src/execution/orchestrator_world_dispatch.rs
```

Current relevant tests to keep green while landing this slice:

```bash
cargo test -p world-service bootstrap_completion_with_session_handle_emits_registered_then_exit -- --nocapture
cargo test -p world-service find_submit_target_rejects_participant_id_drift_for_retained_slot -- --nocapture
cargo test -p shell resolve_internal_continue_world_dispatch_target_returns_exact_retained_worker -- --nocapture
cargo test -p shell public_start_persists_detached_session_when_hidden_owner_helper_exits -- --nocapture
cargo test -p shell public_turn_routes_linux_world_member_follow_up_through_typed_submit_path -- --nocapture
```

Expected new regression command floor after implementation:

```bash
cargo test -p world-service member_runtime -- --nocapture
cargo test -p shell public_turn_routes_linux_world_member_follow_up_through_typed_submit_path -- --nocapture
```

## Project Structure

```text
crates/world-service/src/member_runtime.rs
  Owns retained-member registration, surfaced session-handle capture,
  bootstrap exit handling, and submitted-turn resume execution.

crates/shell/src/execution/orchestrator_world_dispatch.rs
  Owns host-side continue dispatch, exact-target submit request building,
  and delivery over the world member-turn seam.

crates/shell/src/execution/agent_runtime/state_store.rs
  Owns authoritative retained-target resolution and exact session/world/backend checks.

crates/shell/src/execution/agent_runtime/session.rs
  Owns persisted participant session-handle truth (`uaa_session_id`, `resume_eligible`).

crates/world-service/tests/
  Candidate home for world-service lifecycle regressions when a black-box seam is needed.

crates/shell/tests/agent_public_control_surface_v1.rs
  Public retained-world-worker regression coverage, including shell-owned parked/resumable truth.

llm-last-mile/DESIGN-world-worker-lifecycle-model.md
  Lifecycle authority that now freezes `running -> parked` on clean bootstrap exit after surfaced continuity identity.

llm-last-mile/SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md
  Adjacent but separate bootstrap-compatibility scope that this slice must not absorb.
```

## Code Style

Prefer small lifecycle helpers and explicit state predicates over ad hoc cleanup branching.

Rust style to preserve in this slice:

```rust
if surfaced_retained_identity && surfaced_resumable_session_identity {
    transition_retained_worker_to_parked(participant_id)?;
    preserve_retained_registry_slot(participant_id)?;
} else {
    fail_closed_or_closeout_without_promising_resume(participant_id)?;
}
```

Conventions for this slice:

- keep exact identity validation fail-closed and specific
- keep lifecycle transitions explicit (`running -> parked`, not “implicitly still active”)
- prefer helper names that describe lifecycle intent, not transport accidents
- do not hide retained-worker deletion behind generic process cleanup paths
- keep launcher/bootstrap cleanup separate from retained-registry closeout when those meanings differ

## Testing Strategy

### Test levels

1. **World-service unit / focused runtime tests** in `crates/world-service/src/member_runtime.rs`
   - prove registration survives clean bootstrap exit when surfaced session handle exists,
   - prove the retained slot remains exact-identity validated,
   - prove submitted-turn resume still requires matching orchestration/session/world/backend identity,
   - prove missing surfaced session handle still fails closed instead of pretending resumability.

2. **World-service black-box seam coverage** in `crates/world-service/tests/` if unit-only coverage cannot honestly express the lifecycle handoff
   - prove a retained member can register, exit bootstrap, and later accept a submitted turn without an always-live bootstrap process.

3. **Shell/public retained-worker regression coverage** in `crates/shell/tests/agent_public_control_surface_v1.rs`
   - keep exact-target public follow-up routing green,
   - extend or add coverage so the retained worker can be followed up after bootstrap has exited, not only while a test stub artificially stays alive.

### Required verification boundary

This slice is not done until the repo has regression coverage for the joined contract:

1. `spawn_world_worker` returns a retained receipt,
2. bootstrap surfaces authoritative retained registration,
3. bootstrap exits cleanly,
4. retained worker remains parked/resumable rather than being unregistered,
5. later `continue_world_worker` succeeds through the submitted-turn seam,
6. explicit stop/invalidation/terminal closeout still removes resumability.

### Coverage expectations

- preserve current exact-identity mismatch failures
- add positive coverage for parked retained resume after clean bootstrap exit
- add negative coverage that clean bootstrap exit **without** surfaced resumable session identity does **not** claim resumability
- do not let new tests depend on bootstrap process liveness as a proxy for retained continuity

## Boundaries

- **Always:**
  - keep the lifecycle authority aligned with `DESIGN-world-worker-lifecycle-model.md`
  - keep retained continuity owned by surfaced retained identity plus surfaced resumable session identity, not bootstrap-process liveness alone
  - keep `continue_world_worker` exact-target, exact-binding, and fail-closed
  - keep explicit terminal lifecycle transitions as the only closeout path for retained continuity once the authoritative identity tuple exists
  - keep the slice narrow to the active retained parked/resume bug seam

- **Ask first:**
  - adding new durable persisted lifecycle fields outside the minimum needed to keep runtime truth coherent
  - changing shell-side public control vocabulary or tool argument shapes
  - changing routing authority beyond the retained parked/resume seam
  - changing router/auto-attach/inbox contracts rather than consuming them as already-adjacent authority

- **Never:**
  - silently broaden this slice into bootstrap config/auth projection or general world bootstrap redesign
  - equate clean bootstrap process exit with retained-worker deletion once authoritative retained + resumable identities have been surfaced
  - introduce fuzzy retained-worker lookup or alternate selectors in place of exact `participant_id`
  - use workspace reconciliation, host visibility, or unrelated file-sync behavior as proxy proof for this lifecycle contract
  - treat MCP/app-runtime/apps/connector concerns as part of this lifecycle fix

## Success Criteria

1. The spec freezes that retained continuity is owned by surfaced retained identity plus surfaced resumable session identity, not by bootstrap-process liveness alone.
2. `spawn_world_worker` receipt semantics remain truthful: a worker is not represented as resumable unless the authoritative retained identity and resumable session identity have actually been surfaced.
3. After those identities are surfaced, clean bootstrap exit means `running -> parked`; it does **not** unregister or delete the retained worker.
4. `continue_world_worker` against that parked retained worker remains in-contract and resumes through the submitted-turn seam using the surfaced session handle.
5. Runtime realization keeps exact `participant_id`, exact orchestration session, exact backend, exact `world_id`, and exact `world_generation` validation intact.
6. Retained continuity is destroyed only by explicit terminal lifecycle transitions such as `stopped`, `invalidated`, or other explicit retained closeout already allowed by lifecycle authority.
7. Regression coverage exists for: `registered retained worker -> bootstrap exits cleanly -> later continue succeeds`.
8. Regression coverage also proves the fail-closed inverse: no surfaced resumable session identity means no parked-resumable promise.
9. `SPEC-62` remains bootstrap-compatibility-only and does not reabsorb this lifecycle seam.

## Open Questions

1. What is the minimum honest runtime representation for a parked retained worker inside `world-service` after bootstrap exit: preserve the existing registry entry in-place, or split active-turn/process bookkeeping from retained-registry ownership more explicitly?
2. When a retained submitted turn exits non-zero **after** a surfaced resumable session handle already exists, should the default post-turn lifecycle be `failed`, `parked`, or policy-driven by event/exit classification? This slice should stay bounded, but the implementation may need one explicit rule.
3. Does `world-service` need an explicit internal parked/not-running marker for retained members, or is “registered retained worker with no active turn slot” sufficient for v1 as long as exact routing and terminal closeout remain correct?
4. Which existing test harness is the narrowest honest place for the joined regression: `member_runtime.rs` unit-style coverage, a `crates/world-service/tests/` black-box harness, or an extension of the Linux public control surface test with a bootstrap-exits-then-resume script?
