# Design Validation Memo: Stop Fallback, Host Ownership, and Retained Worker Semantics

Date: 2026-07-03  
Scope: design-validation research pass only; no code review or implementation changes  
Workspace: `/home/spenser/__Active_code/substrate`

## Purpose

Validate or invalidate the earlier Sartre-style concerns against the repo's current `llm-last-mile` design stack, prioritizing `DESIGN*`, `SPEC*`, `PLAN*`, `TASKS*`, and `PROMPTS*` materials.

No on-disk Sartre artifact was found, so this memo evaluates the four claims as independent propositions against the primary design/planning sources.

## Headline Conclusions

1. **The current stop-fallback direction is likely misaligned with intended design.**
   - The design stack treats `stop` as an explicit durable control-plane closeout, not as a transport-side heuristic or "best effort" delivery semantic.
   - The private stop transport is implementation posture, not the semantic owner of stop.
2. **Attached host participant ownership assumptions are wrong if they treat the attached host as the durable authority.**
   - Durable authority is the orchestration session plus persisted attach/session truth, not the currently attached host execution client.
   - The authoritative attached host participant may change across turns via sanctioned `reattach` or router-owned attach restoration.
3. **Transport-centric reasoning is too narrow for this area.**
   - The design stack repeatedly separates lifecycle truth, session authority, and obligation truth from transport liveness.
   - The repo's own later repair plan for Slice 63 explicitly calls out "transport accidents" and bootstrap-process liveness as the wrong continuity model.
4. **Fork/continue direction appears aligned with intended design.**
   - Continue remains exact-target, retained-only, same-session/same-world steering.
   - Fork remains explicit, host-mediated, exact-source, lineage-preserving, and intentionally distinct from public host-session `fork`.

## Primary Design Truth

### 1. Durable authority is the orchestration session, not the attached host client

- `llm-last-mile/SPEC-31-lazy-host-attach-for-host-rooted-world-start.md:46-53`
  - durable authority remains the host-rooted orchestration session whether or not a host client is attached
  - automatic attach is continuity-first / fresh-fallback
  - detached follow-up remains fail-closed until host ownership is restored
- `llm-last-mile/PLAN-31.md:80-85`
  - persisted `HostAttachContract` is authoritative baseline truth
  - later attach attempts may honor or narrow that truth, but may not silently broaden or re-derive it from live snapshots
- `llm-last-mile/29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md:46-70`
  - `HostAttachContract` remains the only durable attach object
  - missing durable truth must fail closed
- `docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md:20-29`
  - durable unit is the Substrate-owned orchestration session, not any one attached backend client process

Implication:

- The attached host participant is an **execution client**, not the durable owner.
- Any model that treats "the currently attached host participant" as the canonical authority is out of line with the design stack.

### 2. Attached-host changes across turns are expected, but only through sanctioned restoration

- `llm-last-mile/DESIGN-auto-attach-trigger-and-work-queue-contract.md:95-102`
  - auto-attach eligibility requires that there is **no authoritative attached host participant**
- `llm-last-mile/DESIGN-auto-attach-trigger-and-work-queue-contract.md:189-203`
  - if an authoritative attached host participant already exists, attach processing should not start
  - successful manual `reattach` converges with queued/claimed automatic work
- `llm-last-mile/DESIGN-router-daemon-attach-trigger-integration.md:248-257`
  - router attach boundary is: restore/launch sanctioned host execution client, restore attached-host ownership truth, stop
- `llm-last-mile/DESIGN-router-daemon-attach-trigger-integration.md:326-335`
  - manual `reattach` and router-owned auto-attach share the same authority model
- `llm-last-mile/PLAN-48.md:16-23`
  - successful attach restores host ownership truth and settles attach-state substates without resolving review state

Implication:

- The authoritative attached host participant can legitimately change over time.
- What must stay stable is the **session-owned attach contract and authority model**, not the specific attached participant instance.

### 3. Retained workers are authoritative session state; process liveness is not the continuity truth

- `llm-last-mile/DESIGN-world-worker-lifecycle-model.md:68-81`
  - retained means "become part of authoritative orchestration state"
- `llm-last-mile/DESIGN-world-worker-lifecycle-model.md:169-181`
  - `parked` means valid/resumable with no active turn executing
  - `stopped` is explicit durable closeout
  - `invalidated` means routing/authority truth no longer matches
- `llm-last-mile/DESIGN-world-worker-lifecycle-model.md:217-238`
  - bootstrap exit is not retained closeout
  - after retained identity + resumable session identity are surfaced, clean bootstrap exit means `running -> parked`
  - runtime must not delete/unregister retained workers just because bootstrap exited cleanly
- `llm-last-mile/DESIGN-world-worker-lifecycle-model.md:322-329`
  - only explicit terminal lifecycle transitions close retained workers
  - turn/process exit is not retained-worker closeout once the continuity tuple exists
- `llm-last-mile/SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md:26-31`
  - explicit terminal transitions such as `stopped` and `invalidated` are the only valid reasons to destroy retained continuity after the authoritative identity tuple exists
- `llm-last-mile/PLAN-63-retained-world-worker-parked-resume-session-handle-contract.md:35-48`
  - explicit implementation drift statement: `world-service` currently models continuity as bootstrap-process liveness and unregisters too early

Implication:

- "Worker truth says live" is not equivalent to "transport/process is still live."
- The intended model is continuity by authoritative identity tuple, not by bootstrap/transport liveness.

### 4. Stop is a durable control-plane closeout, not a transport-mediated semantic

- `llm-last-mile/DESIGN-world-worker-lifecycle-model.md:343-356`
  - stop means explicit durable closeout of a retained worker
  - no further continuation should be allowed
  - stop is a control-plane lifecycle action, not merely a transport interruption
- `llm-last-mile/DESIGN-host-orchestrator-world-dispatch-contract.md:141-145`
  - `cancel_world_work` stops active work in flight
  - `stop_world_worker` closes a retained world participant as a durable lifecycle action
- `llm-last-mile/SPEC-36-internal-retained-world-worker-stop-closeout.md:34-49`
  - objective is durable retained-worker closeout through internal dispatch
  - Linux v1 routes through the existing private owner stop surface
  - non-Linux fails closed rather than approximating stop via public stop/cancel behavior
- `llm-last-mile/SPEC-36-internal-retained-world-worker-stop-closeout.md:60-73`
  - stop is durable lifecycle closeout, not snapshot, not generic cancel
  - exact identity is mandatory
  - may reuse stop transport posture, but must not widen into a new public CLI contract
- `llm-last-mile/SPEC-36-internal-retained-world-worker-stop-closeout.md:241-245`
  - never introduce a second stop transport or second lifecycle model just for the internal verb
- `llm-last-mile/PLAN-36.md:191-192`
  - explicit risk: transport and detached closeout paths may diverge semantically
  - mitigation: hold success on authoritative stopped state, not transport implementation details alone
- `llm-last-mile/TASKS-36.md:197-200`
  - if honest stop would require a second stop transport or second lifecycle model, spec/plan/task work must be reopened rather than silently widened

Implication:

- The design does **not** define stop as "whatever the private stop transport can deliver."
- The transport is one routed mechanism. The semantic owner is the durable closeout contract.

### 5. Transport failure is fail-closed transport truth, not durable lifecycle truth

- `llm-last-mile/DESIGN-internal-toolbox-transport-and-session-binding.md:176-180`
  - stream is incomplete until terminal `result`
  - if owner disappears before terminal response, transport must fail closed
- `llm-last-mile/DESIGN-internal-toolbox-transport-and-session-binding.md:208-208`
  - `stop_world_worker` reuses the same transport family and terminal framing rules as other verbs
- `llm-last-mile/DESIGN-internal-toolbox-transport-and-session-binding.md:248-257`
  - transport fails closed when no live orchestrator runtime owns the session or owner closes before terminal response

Implication:

- Transport loss means **the transport action failed closed**.
- It does not, by itself, mean the retained worker is durably stopped.

## Evaluation Of The Four Sartre Claims

### Claim 1: Current stop-fallback direction is misaligned

Assessment: **Validated**

Why:

- The design stack freezes stop as durable control-plane closeout, not delivery semantics.
- The stop slice explicitly warns against inventing a second stop plane or letting transport details redefine the contract.
- The transport design says owner disappearance yields fail-closed transport behavior, not success.

Strongest refs:

- `llm-last-mile/DESIGN-world-worker-lifecycle-model.md:343-356`
- `llm-last-mile/SPEC-36-internal-retained-world-worker-stop-closeout.md:34-49`
- `llm-last-mile/SPEC-36-internal-retained-world-worker-stop-closeout.md:60-73`
- `llm-last-mile/PLAN-36.md:191-192`
- `llm-last-mile/DESIGN-internal-toolbox-transport-and-session-binding.md:176-180`

Design reading for "worker truth says live but private stop transport is dead":

- **Inference from the cited sources:** transport death alone should not be treated as durable stop.
- The honest outcomes are:
  - fail closed on delivery,
  - or invalidate/mark unroutable if authority/binding truth is disproven,
  - but not silently convert transport failure into successful durable closeout.

### Claim 2: Attached host participant ownership assumptions are wrong

Assessment: **Validated**

Why:

- The durable owner is the orchestration session plus persisted attach/session truth.
- Manual `reattach` and router attach both restore attached-host ownership truth; neither becomes a new authority root.
- Follow-up remains fail-closed until sanctioned host ownership is actually restored.

Strongest refs:

- `docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md:20-29`
- `llm-last-mile/SPEC-31-lazy-host-attach-for-host-rooted-world-start.md:31-40`
- `llm-last-mile/SPEC-31-lazy-host-attach-for-host-rooted-world-start.md:46-53`
- `llm-last-mile/29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md:46-70`
- `llm-last-mile/DESIGN-router-daemon-attach-trigger-integration.md:248-257`
- `llm-last-mile/DESIGN-router-daemon-attach-trigger-integration.md:326-335`

### Claim 3: Current reasoning is too transport-centric

Assessment: **Validated**

Why:

- Transport docs define framing and fail-closed behavior, but they do not own lifecycle truth.
- Lifecycle docs define retained continuity independently of process/bootstrap liveness.
- Slice 63's spec/plan/tasks/prompts repeatedly frame the live bug as exactly this wrong coupling.

Strongest refs:

- `llm-last-mile/DESIGN-internal-toolbox-transport-and-session-binding.md:210-257`
- `llm-last-mile/DESIGN-world-worker-lifecycle-model.md:217-238`
- `llm-last-mile/PLAN-63-retained-world-worker-parked-resume-session-handle-contract.md:35-48`
- `llm-last-mile/TASKS-63.md:12-18`
- `llm-last-mile/PROMPTS-63.md:37-38`
- `llm-last-mile/PROMPTS-63.md:190-205`

### Claim 4: Fork/continue direction is aligned

Assessment: **Mostly validated**

Why:

- Continue is consistently frozen as exact-target retained follow-up over the existing retained seam.
- Fork is consistently frozen as explicit, exact-source, same-session/same-world, lineage-preserving, and host-mediated.
- I did not find parallel evidence of design drift in those areas comparable to the stop/lifecycle drift called out by Slice 63.

Strongest refs:

- `llm-last-mile/SPEC-33-internal-retained-world-worker-continue-and-event-bootstrap.md:34-39`
- `llm-last-mile/SPEC-33-internal-retained-world-worker-continue-and-event-bootstrap.md:45-58`
- `llm-last-mile/DESIGN-host-to-world-steering-policy-matrix.md:331-340`
- `llm-last-mile/SPEC-38-internal-retained-world-worker-fork.md:42-52`
- `llm-last-mile/SPEC-38-internal-retained-world-worker-fork.md:158-167`
- `llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md:57-67`
- `llm-last-mile/PLAN-45.md:10-19`

## Where Planning Material Explicitly Suggests Implementation Drift

1. **Retained continuity drift is explicit.**
   - `llm-last-mile/PLAN-63-retained-world-worker-parked-resume-session-handle-contract.md:35-48`
   - The plan states the implementation currently treats continuity as bootstrap-process liveness and unregisters too early.

2. **Stop-vs-transport semantic drift was anticipated by the plan itself.**
   - `llm-last-mile/PLAN-36.md:191-192`
   - The plan explicitly warns that stop transport and detached closeout paths may diverge semantically, and says success must be judged by authoritative stopped state, not transport details alone.

3. **Host authority drift was already hardened against fallback/default reconstruction.**
   - `llm-last-mile/29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md:68-70`
   - `llm-last-mile/29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md:120-123`
   - The stack had already decided missing attach truth must fail closed and must not be reconstructed from permissive/default sources.

4. **Router/attach work was deliberately kept from continuing worker work or silently resolving obligations.**
   - `llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md:72-75`
   - `llm-last-mile/PLAN-48.md:64-66`
   - `llm-last-mile/DESIGN-router-daemon-attach-trigger-integration.md:75-92`
   - This matters because any "attached host changed, therefore continuation/stop should just proceed through router-ish fallback" would exceed the stated Family-2 boundary.

## Recommended Next Move

Recommendation: **design-driven patching**

Why:

1. The design stack is already specific enough on the core questions:
   - session authority vs attached host participant
   - retained continuity vs process/transport liveness
   - stop as durable closeout vs transport event
2. There is already an explicit drift memo in the planning stack for the retained lifecycle seam.
3. The unresolved question is not "what was intended?" so much as "how to bring the implementation back to the intended contract without silently widening scope."

Recommended patching guardrails:

1. Treat stop fallback as a **contract alignment** problem, not a transport workaround.
2. Do not let dead private stop transport imply successful durable stop.
3. Keep attached-host restoration explicit and session-owned.
4. Reuse the same exact-target/same-session/same-world discipline already frozen for continue/fork.
5. If honest stop behavior would require a second stop plane or a revised invalidation/closeout taxonomy, reopen the spec/plan instead of patching ad hoc.

## Bottom Line

The strongest reading of the design/planning stack is:

- **Sartre's stop-direction concern is substantially correct.**
- **Sartre's host-ownership concern is substantially correct.**
- **Sartre's transport-centricity concern is substantially correct.**
- **Sartre's fork/continue alignment claim is also substantially correct.**

The next engineering step should be **design-driven patching**, not a blind code probe and not additional high-level design clarification unless the team intends to change the frozen meaning of `stop` or the durable session authority model.
