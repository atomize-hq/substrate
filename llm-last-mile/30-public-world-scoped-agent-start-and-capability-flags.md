# SOW: Public World-Scoped Agent Start And Capability Flags

Status: draft realigned to host-first product intent. This slice is not implementation-ready yet. It depends on [28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md](28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md), [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](29-shared-agent-dispatch-envelope-and-capability-override-contract.md), and the landed [29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md](29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md) floor.

This slice no longer carries host-rooted versus standalone world-root as an open product decision. The validated architecture has already closed that question.

## Frozen Direction

The thin-slice meaning of public `substrate agent start` is now:

1. create a host-rooted durable orchestration session,
2. submit the inaugural operator prompt to the host orchestration agent,
3. persist the resolved host attach contract under that session,
4. when scope resolves to world, also create or bind the authoritative world session/binding that later host-dispatched world agents will use,
5. keep the host agent as the primary operator-facing control surface.

Scope meaning is now:

1. omitting `--scope` resolves requested execution substrate through workspace-local config/profile/policy first, then global config/policy;
2. `--scope world` explicitly requests the world-backed default path;
3. `--scope host` explicitly bypasses world and keeps orchestration plus later dispatch host-scoped.

This slice must not reopen standalone world-root continuity.

## Objective

Broaden the public `substrate agent start` surface so a human launches a host orchestration session first, with world-backed execution as the default substrate when scope resolution selects it.

This slice is done only when all of the following are true:

1. `substrate agent start` accepts explicit scope selection through the validated shared dispatch contract.
2. omitting `--scope` resolves through workspace-local config/profile/policy first, then global config/policy, instead of hardcoding host.
3. `--scope world` means host-rooted orchestration with a world-backed session/binding available for later host-dispatched world work.
4. `--scope host` means bypass world and keep orchestration plus later dispatch host-scoped.
5. the inaugural operator prompt goes to the host orchestration agent, not directly to a first world worker/member.
6. the CLI surface and docs explain the resulting durable session truth clearly.

## What This Slice Assumes Is Already Landed

1. 28.5 has removed blank-prompt control semantics from Substrate architecture and split control-only attach from prompt-bearing launch.
2. 29 has landed one shared `DispatchRequestEnvelope`, one resolved launch contract, and one persisted host attach contract.
3. 29 has frozen the two baseline domains:
   - inventory-backed resolution for new dispatch,
   - persisted-attach-backed resolution for later host attach and detached follow-up recovery.

## What This Slice Leaves To 31

This slice does not finish automatic world-agent dispatch policy or any specialized born-unattached flow. 31 owns any later work on:

1. automatic dispatch or attach triggers beyond the inaugural host-start path,
2. any future explicit world-first/headless path that truly starts unattached,
3. broader pending-work or inbox-driven trigger policy for detached or unattached sessions.

## 29.75 Contract Floor This Slice Must Reuse

Before this public surface is promoted, it must inherit the 29.75 closeout floor exactly:

1. inventory `policy_overlay` already merges into the resolved `effective_policy`;
2. the only dispatch-time capability narrowing family currently supported is `session_resume`, `session_fork`, `session_stop`, `status_snapshot`, and `event_stream`, and only from `true` to `false`;
3. `session_start`, `llm`, and `mcp_client` remain dispatch-time unsupported and must stay fail closed until a later slice deliberately broadens scope;
4. retained world-member follow-up turns already consume a shared-contract-derived parity subset, so this slice must not invent a second public world-dispatch dialect.
5. host session birth now persists authoritative attach-relevant truth from resolved-contract semantics across both public start and REPL host cold start, so this slice must not repair durable attach truth itself.
6. missing or invalid persisted durable attach truth now fails closed with no repair/backfill branch in 29.75, so this slice must not reintroduce one.

## Draft Work Breakdown

### 1. Expose explicit scope selection on public `agent start`

Required direction:

1. omitting `--scope` resolves requested execution substrate through workspace-local config/profile/policy first, then global config/policy.
2. `--scope world` explicitly requests the world-backed path.
3. `--scope host` explicitly bypasses world.
4. invalid scope/backend combinations fail closed.

### 2. Define the public host-first start contract in operator-visible terms

Required direction:

1. a durable host-rooted orchestration session exists immediately,
2. the inaugural operator prompt is submitted to the host orchestration agent,
3. the host attach contract is already persisted,
4. when scope resolves to world, authoritative world binding/session truth is established for later host-dispatched world work,
5. the default world-backed path must still feel like starting a host conversation rather than a specialized unattached world-first mode.

### 3. Reuse the same capability flag families exposed by 29

Required direction:

1. user-facing capability flags map onto the same resolved launch contract used by orchestrator-controlled dispatch,
2. policy-denied flags fail closed,
3. the command does not invent a CLI-only override model or alternate attach-truth vocabulary,
4. any public capability flags introduced in this slice must stay inside the already-supported narrowing family unless a later slice explicitly reopens the contract.

### 4. Preserve current lifecycle boundaries

Required direction:

1. no hidden bootstrap prompt reappears,
2. `start` does not imply standalone world continuity,
3. this slice does not introduce a second inaugural prompt or immediate world-agent bootstrap conversation.

## Draft Acceptance Shape

This slice should only be promoted out of draft once:

1. 28.5 has landed,
2. 29 has landed,
3. 29.75 has landed,
4. the command can create a host-rooted durable session with host-first prompt handling and world-backed default substrate resolution,
5. the public CLI and docs no longer imply standalone world-root as an option or a born-unattached default.

## Draft Validation Targets

When this slice is eventually promoted, validation must include:

1. host-scoped root start still working,
2. bare root start resolving scope through workspace/global config and policy in the documented order,
3. world-backed root start creating a host-rooted durable session plus world binding,
4. inaugural prompt flowing through the host orchestration path,
5. resolved capability flags matching the 29 contract,
6. truthful host lifecycle/status output for the newly created session,
7. no regression to explicit `--scope host` bypass behavior.

## Sequencing

Current stack status:

1. [28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md](28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md): implementation-ready immediate slice.
2. [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](29-shared-agent-dispatch-envelope-and-capability-override-contract.md): implementation-ready next slice.
3. [29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md](29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md): final contract-authority closeout floor.
4. This SOW: draft pending those earlier landings.
5. [31-lazy-host-attach-for-host-rooted-world-start.md](31-lazy-host-attach-for-host-rooted-world-start.md): later follow-on only if the product reopens explicit unattached or automatic-trigger flows.
