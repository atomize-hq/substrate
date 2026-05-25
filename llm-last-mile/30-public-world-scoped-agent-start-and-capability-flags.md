# SOW: Public World-Scoped Agent Start And Capability Flags

Status: draft aligned to validated architecture. This slice is not implementation-ready yet. It depends on [28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md](28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md), [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](29-shared-agent-dispatch-envelope-and-capability-override-contract.md), and [29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md](29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md) landing first.

This slice no longer carries host-rooted versus standalone world-root as an open product decision. The validated architecture has already closed that question.

## Frozen Direction

The only valid forward meaning of public `substrate agent start --scope world ...` is:

1. create a host-rooted durable orchestration session,
2. persist the resolved host attach contract under that session,
3. launch a world worker/member under that session through the shared dispatch contract,
4. leave durable authority host-rooted,
5. avoid eager host execution-client startup.

This slice must not reopen standalone world-root continuity.

## Objective

Broaden the public `substrate agent start` surface so a human can explicitly launch a world-scoped worker under a host-rooted orchestration session using the shared dispatch contract from 29.

This slice is done only when all of the following are true:

1. `substrate agent start` accepts explicit scope selection through the validated shared dispatch contract.
2. `--scope world` always means host-rooted orchestration plus world worker launch.
3. The command persists the host attach contract at session birth.
4. The command does not eagerly start a host execution client just to create ownership theater.
5. The CLI surface and docs explain the resulting durable session truth clearly.

## What This Slice Assumes Is Already Landed

1. 28.5 has removed blank-prompt control semantics from Substrate architecture and split control-only attach from prompt-bearing launch.
2. 29 has landed one shared `DispatchRequestEnvelope`, one resolved launch contract, and one persisted host attach contract.
3. 29 has frozen the two baseline domains:
   - inventory-backed resolution for new dispatch,
   - persisted-attach-backed resolution for later host attach and detached follow-up recovery.

## What This Slice Leaves To 31

This slice does not finish lazy host attach behavior. 31 owns:

1. when lazy host attach is triggered,
2. fresh attach versus continuity attach behavior,
3. operator/status truth for born-unattached sessions with pending host-side work.

## 29.75 Contract Floor This Slice Must Reuse

Before this public surface is promoted, it must inherit the 29.75 closeout floor exactly:

1. inventory `policy_overlay` already merges into the resolved `effective_policy`;
2. the only dispatch-time capability narrowing family currently supported is `session_resume`, `session_fork`, `session_stop`, `status_snapshot`, and `event_stream`, and only from `true` to `false`;
3. `session_start`, `llm`, and `mcp_client` remain dispatch-time unsupported and must stay fail closed until a later slice deliberately broadens scope;
4. retained world-member follow-up turns already consume a shared-contract-derived parity subset, so this slice must not invent a second public world-start follow-up dialect.
5. host session birth now persists authoritative attach-relevant truth from resolved-contract semantics across both public start and REPL host cold start, so this slice must not repair durable attach truth itself.

## Draft Work Breakdown

### 1. Expose explicit scope selection on public `agent start`

Required direction:

1. `--scope host` preserves the current host-rooted start meaning.
2. `--scope world` routes through the same shared dispatch contract from 29; it does not invent a second CLI-only launch dialect.
3. invalid scope/backend combinations fail closed.

### 2. Define the public world-start contract in operator-visible terms

Required direction:

1. a durable host-rooted orchestration session exists immediately,
2. a world worker/member is launched under that session,
3. the host attach contract is already persisted,
4. no host execution client must be attached yet.

### 3. Reuse the same capability flag families exposed by 29

Required direction:

1. user-facing capability flags map onto the same resolved launch contract used by orchestrator-controlled dispatch,
2. policy-denied flags fail closed,
3. the command does not invent a CLI-only override model or alternate attach-truth vocabulary,
4. any public capability flags introduced in this slice must stay inside the already-supported narrowing family unless a later slice explicitly reopens the contract.

### 4. Preserve current lifecycle boundaries

Required direction:

1. no hidden bootstrap prompt reappears,
2. detached-world follow-up remains fail-closed until host ownership is attached through the sanctioned path,
3. `start` does not imply standalone world continuity.

## Draft Acceptance Shape

This slice should only be promoted out of draft once:

1. 28.5 has landed,
2. 29 has landed,
3. 29.75 has landed,
4. the command can create a host-rooted durable session plus world worker without eager host attach,
5. the public CLI and docs no longer imply standalone world-root as an option.

## Draft Validation Targets

When this slice is eventually promoted, validation must include:

1. host-scoped root start still working,
2. world-scoped root start creating a host-rooted durable session,
3. resolved capability flags matching the 29 contract,
4. truthful status output for the newly created session before any host attach,
5. detached-world fail-closed behavior before host attach.

## Sequencing

Current stack status:

1. [28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md](28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md): implementation-ready immediate slice.
2. [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](29-shared-agent-dispatch-envelope-and-capability-override-contract.md): implementation-ready next slice.
3. [29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md](29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md): final contract-authority closeout floor.
4. This SOW: draft pending those earlier landings.
5. [31-lazy-host-attach-for-host-rooted-world-start.md](31-lazy-host-attach-for-host-rooted-world-start.md): draft follow-on after this slice fixes the public entrypoint.
