# Design Lineage Audit: Retained World Worker Resume Contract

## Scope

Read-only lineage audit for the retained world-worker bug narrowed in:

- `handoffs/2026-06-23-retained-world-worker-lifecycle-debug.md`
- `handoffs/2026-06-22-211455-spec-62-world-worker-retained-debug.md`

Focus only: the contract for **bootstrap exits cleanly, but the retained worker remains resumable by surfaced session handle / `uaa_session_id`**.

## Decisive finding

The primary missing contract belongs in:

- `llm-last-mile/DESIGN-world-worker-lifecycle-model.md`

That file already claims all of the design truths that imply durable retained continuity:

- retained means authoritative orchestration state with future routing obligations
- `parked` means valid and resumable with no active turn executing
- retained workers have stable `participant_id`
- later `continue` targeting is part of the contract
- partial results do not terminate retained lifecycle

What it does **not** currently freeze is the specific transition:

> when bootstrap surfaces authoritative retained identity plus resumable session identity, clean bootstrap process exit is a `running -> parked` handoff, not implicit retained-worker destruction.

That omission is the seam the current runtime fell through.

## Supporting docs

These docs reinforce the durable-handle / receipt-oriented model, but do not own the missing transition:

- `llm-last-mile/DESIGN-host-orchestrator-world-dispatch-contract.md`
  - `spawn_world_worker` allocates a retained participant and future continuation is part of the contract.
- `llm-last-mile/DESIGN-retained-world-worker-messaging-and-steering-contract.md`
  - retained messaging is exact-identity and resumable; continuation requires exact retained identity.
- `llm-last-mile/DESIGN-host-orchestrator-tool-invocation-surface.md`
  - tool surface is receipt-oriented and resumable; `spawn_world_worker` returns authoritative retained-worker identity and later follow-up uses durable handles.
- `llm-last-mile/DESIGN-internal-toolbox-transport-and-session-binding.md`
  - `spawn_world_worker` is receipt-oriented and returns an authoritative retained-worker receipt rather than keeping the original transport open forever.

## Classification

Best classification: **design omission with resulting implementation drift**, not a sound-design implementation bug.

Why:

1. the surrounding doc stack already leans toward durable retained continuity,
2. the runtime instead models continuity as bootstrap process liveness,
3. but the lifecycle authority never explicitly froze the post-bootstrap handoff rule that would have made the implementation obligation unambiguous.

So the implementation is drifting from the broader design intent, but the immediate root is that the owning lifecycle doc omitted the exact transition contract.

## SPEC-62 consequence

`llm-last-mile/SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md` should be narrowed back to what it actually owns:

- direct `cli:codex-world` bootstrap compatibility inputs
- exact-backend-gated auth/config materialization
- bootstrap truthfulness / fail-closed startup

It should **not** implicitly own or close:

- retained worker parked/resume semantics after clean bootstrap exit
- the world-service retained registry contract after `spawn_world_worker`
- `continue_world_worker` success after bootstrap has exited

That lifecycle/resume seam is a separate slice.

## Recommended next artifacts

1. **Rewrite `llm-last-mile/DESIGN-world-worker-lifecycle-model.md`**
   - add an explicit retained bootstrap handoff rule under retained lifecycle / parked semantics:
     - if bootstrap surfaces authoritative retained identity and resumable session identity,
     - clean turn exit transitions the worker to `parked`,
     - resumability is broken only by explicit `stopped`, `invalidated`, or terminal lifecycle closure,
     - world-service must not equate bootstrap process exit with retained-worker deletion.
2. **Then write a new numbered SPEC**
   - recommended path: `llm-last-mile/SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md`
   - bounded scope:
     - `spawn_world_worker` receipt/registration semantics
     - retained registry persistence across clean bootstrap exit
     - `uaa_session_id` / surfaced-session-handle based resume handoff
     - `continue_world_worker` against parked retained workers
     - joined regression: registered retained worker -> bootstrap exits -> later continue succeeds

## Bottom line

The bug does **not** justify a new follow-on DESIGN doc. It justifies:

- tightening the existing lifecycle authority doc, and
- moving the runtime fix into a new lifecycle-focused SPEC instead of continuing to load that responsibility onto `SPEC-62`.
