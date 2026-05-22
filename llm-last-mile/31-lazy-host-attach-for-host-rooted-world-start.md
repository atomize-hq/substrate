# SOW: Lazy Host Attach For Host-Rooted World Start

Status: remaining-work draft. This SOW is the lifecycle refinement follow-on after [30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md). It is anchored to the shared dispatch model from [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md), the public world-scoped start expansion from [30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md), and the durable host-session truth in [ADR-0047 — Host Orchestrator Durable Session and Parked-Resumable Ownership](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md) and [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md).

This slice does not broaden the architecture to standalone world-root continuity. It refines the host-rooted model so a public world-scoped root start can create the durable host orchestration session and launch a world worker without eagerly starting a host orchestration agent process just to establish ownership theater.

## Objective

Allow `substrate agent start --scope world ...` to create a real host-rooted orchestration session and launch a world worker/member immediately, while deferring host orchestration agent process startup until there is actual host-side work to perform.

This slice is done only when all of the following are true:

1. A host-rooted orchestration session can exist from birth with no attached host execution client.
2. Public world-scoped root start no longer requires an eager host orchestration agent prompt or warm-up binary launch just to create/claim the session.
3. If the world worker later needs approval, guidance, reply handling, or other host-side orchestration work, that need lands in durable session-owned state and can trigger sanctioned host attach.
4. Lazy host attach uses real pending context rather than a synthetic bootstrap prompt.
5. The host-rooted durability model, detached-world fail-closed rule, and `reattach` semantics remain coherent and explicit.

## Sequencing And Landed Prerequisites

This slice assumes all of the following are already landed or intentionally chosen and must be treated as fixed floor rather than reopened here:

1. [28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md) handles gateway-mediated fulfillment rather than lifecycle expansion.
2. [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md) provides the shared dispatch envelope used by human and orchestrator-driven launch requests.
3. [30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md) has already chosen the host-rooted meaning of public `--scope world` root start rather than standalone world-root continuity.
4. [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md) remains the durability and recovery floor.
5. The current detached-world fail-closed rule remains live until a valid host owner is attached through the sanctioned path.

This slice does not absorb:

1. standalone world-root session design,
2. new gateway/backend adapter work,
3. dispatch-envelope redesign,
4. broader inbox workflow productization,
5. fuzzy routing or implicit world continuity rules.

## Frozen Contract To Preserve

This slice must preserve all of the following:

1. The durable authority remains the host-rooted orchestration session, not the world worker.
2. The host orchestration agent process remains an attachable execution client rather than the identity of the durable session.
3. `substrate agent reattach --session <orchestration_session_id>` remains attached-owner recovery only. It does not become a generic “wake any world session” verb.
4. Detached-world follow-up remains fail-closed until host ownership is actually attached or reattached through the sanctioned path.
5. No hidden bootstrap prompt, synthetic warm-up turn, or fake “this session is starting” host prompt may be introduced.
6. World-originated approvals, replies, and pending orchestration work must still land in durable Substrate-owned state rather than depending on a continuously live host agent process.

Primary truth anchors:

- [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
- [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)
- [25-host-durable-session-closeout-and-qa-hardening.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md)

## Current Tension In The Model

The repo already treats the host execution client as attachable and the durable orchestration session as the real authority. That is explicit in [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md:46) and [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md:31).

What is not yet modeled cleanly for the new public `--scope world` shape is:

1. a host-rooted orchestration session that exists from birth with no attached host client yet,
2. world worker launch under that session without an eager host agent binary startup,
3. later host-side attach only when the world worker actually needs approvals, replies, or orchestration help,
4. posture/status truth that distinguishes “valid host-rooted session with no attached host client yet” from “previously attached session that later parked cleanly.”

This SOW exists to close that lifecycle gap explicitly rather than faking it with an unnecessary eager host launch.

## In Scope

1. Define host-rooted world start semantics where the durable orchestration session is created before any host agent process is attached.
2. Allow immediate world worker/member launch under that session without an initial host orchestration agent prompt.
3. Persist enough launch-resolution and recovery metadata at session creation time to allow later sanctioned host attach.
4. Define when and how lazy host attach is triggered or requested.
5. Keep world-originated pending work durable and visible until host ownership is attached.
6. Make status/posture truth explicit for this lifecycle shape.

## Out Of Scope

1. Standalone world-root sessions with no host-rooted orchestration session.
2. Automatic inbox-driven product workflows beyond narrow lifecycle correctness.
3. Replacing `reattach` with a different public recovery verb unless another SOW explicitly does so.
4. Reopening gateway-mediated fulfillment, backend selection, or dispatch-merge semantics.
5. Allowing detached world members to continue independently with no host-owned authority above them.

## Required Contract Decision

This slice assumes and hardens the following exact meaning:

### Public `substrate agent start --scope world ...`

The command:

1. creates a host-rooted orchestration session,
2. persists the effective host and world launch contract for that session,
3. launches the world worker/member under that session,
4. does not require an eager host orchestration agent process launch,
5. leaves host attach lazy until real host-side work exists.

This is not a standalone world-root session. It is a host-rooted durable session with deferred host execution-client attachment.

## Concrete Work Breakdown

### 1. Define session-from-birth semantics for unattached host-rooted world starts

Primary anchors:

- [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
- [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)

Required outcome:

1. The repo explicitly defines that a host-rooted orchestration session may exist before any host execution client is attached.
2. The repo explicitly defines whether this state is represented as:
   - a refined existing posture such as `parked_resumable`, or
   - a new explicit posture for “not yet attached but valid.”
3. That truth is persisted and observable rather than reconstructed heuristically.

### 2. Persist the future host-attach contract at world-scoped root start

Primary anchors:

- [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md)
- [30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md)

Required outcome:

1. The resolved launch contract for the host orchestrator is persisted when the host-rooted session is created.
2. Later host attach does not depend on guessing backend, scope, capability flags, or policy posture after the fact.
3. World start and later host attach consume the same effective launch truth.

### 3. Launch the world worker without eager host-agent startup theater

Primary anchors:

- [30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md)
- [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)

Required outcome:

1. Public world-scoped root start can launch the world worker under a valid host-rooted orchestration session without first spinning up a host orchestration process just to announce responsibility.
2. No hidden bootstrap prompt or synthetic orchestration claim prompt is emitted.
3. The world worker still has a durable orchestration session above it for later approvals, replies, and orchestration-directed work.

### 4. Define the lazy host-attach triggers and sanctioned recovery path

Primary anchors:

- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)
- [25-host-durable-session-closeout-and-qa-hardening.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md)

Required outcome:

1. The repo explicitly defines the classes of host-side need that require host attach:
   - approval requests,
   - replies or follow-up orchestration messages,
   - human-directed orchestration work,
   - or other bounded control-plane needs.
2. Those needs land durably when no host client is attached.
3. The repo explicitly defines whether the product behavior is:
   - require the operator to run `substrate agent reattach --session ...`, or
   - allow a sanctioned lazy attach path that launches the host execution client automatically against real pending work.
4. In either case, the attach path must use real pending context rather than a fake bootstrap prompt.

### 5. Keep detached-world fail-closed semantics coherent under the new model

Primary anchors:

- [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
- [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)

Required outcome:

1. The existing rule does not soften into “world members are independently resumable.”
2. If host ownership is required and not attached, world follow-up still fails closed in a way that points back to the sanctioned host attach path.
3. The new lazy attach semantics must not accidentally create direct world-to-world public continuity.

### 6. Make status and operator truth explicit

Primary anchors:

- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)
- [crates/shell/tests/agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)

Required outcome:

1. `status --json` and human-readable status make it clear when:
   - the host-rooted session exists,
   - no host client is currently attached,
   - the world worker is active,
   - pending host-side work exists.
2. The operator can tell the difference between:
   - “valid host-rooted session that has not attached a host client yet,”
   - “previously attached session that is now parked,”
   - and “attention-needed host-rooted session with pending work.”

## Required Test Additions Or Tightening

### Public world-start coverage

Required scenarios:

1. `substrate agent start --scope world ...` creates a valid host-rooted orchestration session.
2. The world worker launches successfully without eager host orchestration process startup.
3. No hidden bootstrap or fake responsibility-claim prompt is emitted.

### Status and posture coverage

Required scenarios:

1. A newly created host-rooted world session with no attached host client surfaces coherent status truth.
2. Pending world-originated host-side work transitions the session into the correct visible state.
3. Operators can distinguish unattached-from-birth, parked, and attention-needed cases.

### Host attach / recovery coverage

Required scenarios:

1. A sanctioned host attach path can attach later using the persisted launch contract.
2. `reattach` remains non-prompt-taking and restores real attached host ownership if it reports success.
3. Later host attach uses real pending context rather than synthetic bootstrap text.

### Detached-world non-regression coverage

Required scenarios:

1. Detached-world follow-up remains fail-closed when host ownership is required and not attached.
2. Error messaging points to the correct sanctioned host attach path.
3. No new path allows independent world continuity with no host-rooted authority above it.

## Acceptance Criteria

1. Public world-scoped root start can create a host-rooted orchestration session and launch a world worker without eager host agent process startup.
2. The durable host-rooted session remains the authority even before any host client is attached.
3. The effective host attach contract is persisted at session creation time and can be reused later.
4. Host-side approvals, replies, and orchestration needs remain durable when no host client is attached.
5. Later host attach uses real pending context rather than synthetic bootstrap prompts.
6. Detached-world fail-closed semantics do not regress into standalone world continuity.

## Validation Expectations

- run targeted CLI, runtime-state, status, and lifecycle tests,
- run full touched package coverage and then full workspace tests:
  - `cargo test --workspace -- --nocapture`
- manual validation for this slice must explicitly exercise:
  - public `start --scope world ...` with no eager host process startup,
  - a world worker producing host-side pending work,
  - later sanctioned host attach,
  - and detached-world fail-closed behavior before host attach.

## Docs And Truth Sync

When this slice closes:

1. update [30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md) so its chosen host-rooted public-start contract explicitly points to lazy host attach rather than implying eager host startup,
2. update [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md),
3. update [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md) or its successor truth surface so “attachable execution client” and world-root public start no longer leave this case ambiguous,
4. update [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md),
5. and align any status/lifecycle docs so operators can understand “host-rooted session exists, host client not yet attached” as an intentional steady-state rather than an error.
