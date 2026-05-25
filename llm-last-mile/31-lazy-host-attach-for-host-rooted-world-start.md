# SOW: Lazy Host Attach For Host-Rooted World Start

Status: draft aligned to validated architecture. This slice is not implementation-ready yet. It depends on [28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md](28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md), [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](29-shared-agent-dispatch-envelope-and-capability-override-contract.md), the landed [29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md](29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md) floor, and the public host-rooted world-start entrypoint from [30-public-world-scoped-agent-start-and-capability-flags.md](30-public-world-scoped-agent-start-and-capability-flags.md).

This slice no longer tries to discover the architecture. The validated architecture is already fixed:

1. durable authority is the host-rooted orchestration session,
2. the host attach contract is persisted at session birth,
3. the attached host execution client is optional at birth,
4. lazy host attach is a Substrate attach-worker concern, not a prompt trick,
5. 29 owns the only shared dispatch contract and the only durable host-attach truth.

The 29.75 closeout floor this slice inherits is also fixed:

1. persisted host attach resolution already reuses durable backend, protocol, scope, capabilities, effective policy, continuity selector truth, and attach policy defaults from `HostAttachContract`,
2. later attach-time requests are bounded overlays on that baseline: they may honor it, narrow it where permitted, and may not silently broaden or replace it,
3. missing or invalid persisted durable attach truth fails closed with no repair/backfill branch in 29.75,
4. successor allocation already copies generalized attach truth forward while clearing only continuity-specific state,
5. retained world-member follow-up turns already avoid hidden baseline re-resolution and therefore must stay orthogonal to lazy host attach.

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

## Open Architectural Split: Born-Unattached Status Truth

The repo already has landed posture truth for attached-then-detached durable host sessions, but it has not yet fully frozen or implemented the distinct born-unattached host-rooted taxonomy that this slice needs.

Ownership is intentionally split between 30 and 31:

1. 30 owns the minimum pre-attach status floor needed for truthful public world-scoped root start:
   - a born-unattached host-rooted session must be visible as a valid non-terminal state;
   - public world follow-up remains fail-closed before host attach;
   - the public entrypoint must not imply attached ownership when none exists.
2. 31 owns the full taxonomy and lifecycle semantics:
   - exact posture/status naming for born-unattached sessions,
   - the distinction between never-attached and previously-attached-and-parked states,
   - how pending host-side work affects posture/status,
   - how continuity attach versus fresh attach and manual versus automatic trigger policy interact with those states.

This slice should therefore consume the minimal truthful floor that 30 establishes and then finish the deeper taxonomy and attach-lifecycle semantics, rather than assuming those details were already fully frozen earlier in the stack.

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
4. neither mode may invent a second durable attach object or re-derive launch truth from the last live participant snapshot,
5. both modes must trust the persisted effective policy, attach-relevant capability truth, and attach policy defaults that 29.75 freezes durably.

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
2. 29.75 has landed the final authoritative attach-truth floor,
3. 30 has landed the public host-rooted world-start entrypoint,
4. the repo has frozen whether lazy attach is manual-only or may be auto-launched from pending work,
5. attach-worker mode selection is explicit and testable,
6. born-unattached versus parked versus awaiting-attention status truth is operator-visible.

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
3. [29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md](29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md): final contract-authority closeout floor.
4. [30-public-world-scoped-agent-start-and-capability-flags.md](30-public-world-scoped-agent-start-and-capability-flags.md): draft public entrypoint slice.
5. This SOW: draft lazy-attach realization slice after the earlier landings.
