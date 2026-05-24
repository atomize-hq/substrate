# SOW: Lazy Host Attach For Host-Rooted World Start

Status: draft aligned to validated architecture. This slice is not implementation-ready yet. It depends on [28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md](28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md), [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](29-shared-agent-dispatch-envelope-and-capability-override-contract.md), and the public host-rooted world-start entrypoint from [30-public-world-scoped-agent-start-and-capability-flags.md](30-public-world-scoped-agent-start-and-capability-flags.md).

This slice no longer tries to discover the architecture. The validated architecture is already fixed:

1. durable authority is the host-rooted orchestration session,
2. the host attach contract is persisted at session birth,
3. the attached host execution client is optional at birth,
4. lazy host attach is a Substrate attach-worker concern, not a prompt trick,
5. 29 owns the only shared dispatch contract and the only durable host-attach truth.

## Objective

Allow a host-rooted world session that was born without an attached host execution client to attach one later using real persisted launch truth and real pending context instead of synthetic bootstrap prompts.

## Frozen Direction

The attach worker in this slice must support two explicit modes:

1. continuity attach
   - resume an existing backend-native session when a valid continuity selector exists;
2. fresh attach
   - start a new attached host execution client from the persisted host attach contract when no backend-native session exists yet.

This slice must not treat blank prompt as the meaning of either mode.

## What Must Stay True

1. The durable orchestration session remains the authority whether or not a host client is attached.
2. Detached-world follow-up stays fail-closed until host ownership is actually attached through the sanctioned path.
3. No hidden bootstrap prompt, fake inbox-consumption prompt, or synthetic warm-up turn may be introduced.
4. Pending host-side work must remain durable under Substrate session state while no host client is attached.

## Draft Work Breakdown

### 1. Define born-unattached host-rooted session behavior

Required direction:

1. session birth without an attached host client is an intentional steady state,
2. the persisted host attach contract is authoritative from birth,
3. status/posture truth distinguishes this case from a previously attached-and-parked session.

### 2. Implement attach-worker mode selection

Required direction:

1. if a valid continuity selector exists, the attach worker may choose continuity attach;
2. if no continuity selector exists, the attach worker must be able to perform fresh attach from the persisted host attach contract;
3. attach-mode choice must be explicit and auditable;
4. neither mode may invent a second durable attach object or re-derive launch truth from the last live participant snapshot.

### 3. Define how lazy attach is triggered

Open draft question that still needs a later freeze:

1. require explicit operator `reattach`, or
2. allow sanctioned automatic launch when durable pending work exists.

Either direction must use real pending context and the persisted host attach contract.

### 4. Keep fail-closed world follow-up semantics intact

Required direction:

1. a detached or unattached host-rooted session does not authorize public world-to-world continuity,
2. error messaging still points operators back to the sanctioned attach path.

## Draft Acceptance Shape

This slice should only be promoted out of draft once:

1. 28.5 and 29 have landed,
2. 30 has landed the public host-rooted world-start entrypoint,
3. the repo has frozen whether lazy attach is manual-only or may be auto-launched from pending work,
4. attach-worker mode selection is explicit and testable,
5. born-unattached versus parked versus awaiting-attention status truth is operator-visible.

## Draft Validation Targets

When this slice is eventually promoted, validation must include:

1. world-scoped root start creating a host-rooted session with no eager host client,
2. pending host-side work persisting durably before attach,
3. continuity attach when a valid backend-native session exists,
4. fresh attach when no backend-native session exists,
5. no synthetic prompt submission during either attach mode,
6. detached-world fail-closed behavior before host attach.

## Sequencing

Current stack status:

1. [28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md](28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md): implementation-ready immediate slice.
2. [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](29-shared-agent-dispatch-envelope-and-capability-override-contract.md): implementation-ready next slice.
3. [30-public-world-scoped-agent-start-and-capability-flags.md](30-public-world-scoped-agent-start-and-capability-flags.md): draft public entrypoint slice.
4. This SOW: draft lazy-attach realization slice after the earlier landings.
