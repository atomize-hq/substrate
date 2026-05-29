# SOW: Born-Unattached Host Sessions and Obligation-Driven Auto-Attach

Status: draft realigned to the obligation-ledger architecture. This slice is not implementation-ready yet. It depends on [28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md](28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md), [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](29-shared-agent-dispatch-envelope-and-capability-override-contract.md), the landed [29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md](29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md) floor, the public host-rooted start surface in [30-public-world-scoped-agent-start-and-capability-flags.md](30-public-world-scoped-agent-start-and-capability-flags.md), and the new design stack around:

1. [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md)
2. [DESIGN-durable-orchestration-notification-inbox-contract.md](./DESIGN-durable-orchestration-notification-inbox-contract.md)
3. [DESIGN-auto-attach-trigger-and-work-queue-contract.md](./DESIGN-auto-attach-trigger-and-work-queue-contract.md)
4. [DESIGN-router-daemon-attach-trigger-integration.md](./DESIGN-router-daemon-attach-trigger-integration.md)

This slice no longer treats “manual-only or automatic attach?” as open. That question is now closed.

## Objective

Finish the specialized host-rooted session path where no host execution client is attached yet, while preserving truthful durable session authority and making obligation-driven auto-attach the sanctioned recovery path.

This slice exists to define and later implement:

1. born-unattached host-rooted session taxonomy,
2. obligation persistence while no host client is attached,
3. router-driven auto-trigger attach from eligible obligations,
4. continuity-first and fresh-attach fallback using persisted attach truth,
5. fail-closed world follow-up before host ownership is restored.

## Frozen Direction

The validated direction for this slice is now:

1. durable authority remains the host-rooted orchestration session,
2. the host attach contract is persisted at session birth,
3. the attached host execution client may be absent at birth in this specialized path,
4. deferred host work is modeled as durable obligations, not prompts and not a separate notification-plus-queue pair,
5. auto-trigger attach from eligible obligations is required,
6. the router/daemon owns attach-trigger evaluation and attach coalescing,
7. continuity attach is preferred when a valid continuity selector exists,
8. fresh attach is the fallback when continuity is unavailable but persisted attach truth remains valid,
9. detached or born-unattached public world follow-up remains fail-closed until host ownership is actually restored.

## What This Slice Assumes Is Already Frozen

### Dispatch and attach truth floor

The 29 / 29.75 floor this slice inherits is fixed:

1. persisted host attach resolution already reuses durable backend, protocol, scope, capabilities, effective policy, continuity selector truth, and attach policy defaults from `HostAttachContract`,
2. later attach-time requests are bounded overlays on that baseline: they may honor it, narrow it where permitted, and may not silently broaden or replace it,
3. missing or invalid persisted durable attach truth fails closed with no repair/backfill branch,
4. successor allocation already copies generalized attach truth forward while clearing only continuity-specific state,
5. retained world-member follow-up already avoids hidden baseline re-resolution and must stay orthogonal to born-unattached attach recovery.

### Slice-30 floor

Slice 30 already owns the public host-first happy path.

This slice must not reopen it.

That means:

1. the normal public world-backed path remains host-attached at successful `start` return,
2. this slice covers specialized born-unattached or later-detached host-rooted sessions,
3. any operator-visible born-unattached posture must not be misrepresented as the default slice-30 success path.

## What Must Stay True

1. The durable orchestration session remains the authority whether or not a host client is attached.
2. No hidden bootstrap prompt, fake inbox-consumption prompt, or synthetic warm-up turn may be introduced.
3. Pending host-side work must remain durable as obligations while no host client is attached.
4. Auto-attach restores a sanctioned host execution client; it does not resolve obligations automatically.
5. Detached-world follow-up stays fail-closed until host ownership is actually attached through the sanctioned path.
6. Manual `reattach` remains canonical and must interoperate cleanly with auto-attach.

## Scope Boundary

This slice is specialized branch work, not the mainline host-first start path.

It owns:

1. exact posture/status truth for born-unattached host-rooted sessions,
2. distinction between never-attached and previously-attached durable sessions,
3. obligation-driven auto-attach behavior for those sessions,
4. attach-mode selection in that specialized posture.

It does not own:

1. the normal public host-attached slice-30 happy path,
2. generic broader caller-surface expansion,
3. direct world-worker continuation before host attach,
4. a second durable attach-truth dialect.

## Required Runtime Taxonomy

This slice must freeze truthful host-session taxonomy for the specialized path.

### Minimum distinction

The implementation must distinguish at least:

1. `active_attached`
   - host execution client is attached.
2. `parked_resumable`
   - no host client attached, no unresolved attention-driving obligations, later attach is valid.
3. `awaiting_attention`
   - no host client attached, unresolved attention-driving obligations exist.
4. born-unattached equivalent posture
   - no host client has ever attached for this host-rooted session yet, but the session is valid and attach-capable.
5. `terminal`
   - non-routable closeout state.

### Important rule

This slice may choose final naming, but it must not collapse:

1. never-attached-yet,
2. previously attached then parked,
3. unresolved-attention detached states.

Those distinctions are operationally real now that obligations and auto-attach exist.

## Required Obligation Behavior

The specialized born-unattached or detached path must use the obligation-ledger model.

Required direction:

1. unresolved host work persists as obligations under the session,
2. inbox/review is a projection over those obligations,
3. auto-attach is a projection over those obligations,
4. `awaiting_attention` derives from unresolved attention-driving obligations,
5. no second canonical queue ledger is introduced.

## Required Auto-Attach Behavior

Auto-trigger attach is required in this slice.

That is no longer optional.

### Trigger source

1. the router evaluates durable obligations,
2. not raw stream events,
3. not trace-only history,
4. not synthetic prompt reconstruction.

### Trigger eligibility

At minimum, the slice must support router-driven attach evaluation for eligible unresolved obligations such as:

1. `follow_up_required`
2. `approval_required`
3. `blocked`
4. `fork_request`

### Session coalescing

Because attach is session-scoped:

1. multiple eligible obligations in the same session must coalesce into one attach episode,
2. at most one obligation may be `attach_state=claimed` in one session at a time,
3. sibling obligations must not cause duplicate attach launches.

## Required Attach-Mode Selection

Attach mode selection must be explicit and auditable.

### Continuity attach

Required direction:

1. if a valid continuity selector exists, the attach worker should prefer continuity attach,
2. continuity validity must be checked against persisted attach truth,
3. continuity must not broaden persisted policy or capability truth.

### Fresh attach

Required direction:

1. if no valid continuity selector exists, the attach worker must be able to perform fresh attach from the persisted host attach contract,
2. fresh attach must use the same persisted attach truth floor rather than a new ad hoc launch dialect,
3. fresh attach must not invent a second durable attach object or re-derive launch truth from the last live participant snapshot.

## Manual And Automatic Attach Interaction

This slice must preserve one coherent authority model for:

1. explicit operator `reattach`,
2. router-driven automatic attach.

Required direction:

1. manual `reattach` may satisfy or supersede an automatic attach episode,
2. automatic attach must not start duplicate work if manual `reattach` already succeeded,
3. manual `reattach` does not itself resolve obligations,
4. successful attach restores host ownership only; later host actions resolve obligations explicitly.

## Fail-Closed World Follow-Up

Required direction:

1. a born-unattached or detached host-rooted session does not authorize public world-to-world continuity,
2. exact world follow-up attempts remain fail-closed before host attach is restored,
3. operator-visible errors must continue to point back to sanctioned attach recovery paths.

## Draft Work Breakdown

### 1. Freeze born-unattached host-rooted session posture truth

Required work:

1. define the exact posture/status naming for never-attached-yet host-rooted sessions,
2. distinguish it from `parked_resumable`,
3. distinguish it from `awaiting_attention`,
4. keep it truthful in status/read surfaces without treating it as the slice-30 default happy path.

### 2. Realize obligation persistence for unattached host work

Required work:

1. persist unresolved host work as obligations,
2. derive review and attach substates from the same obligation,
3. keep host posture derived from unresolved attention-driving obligations,
4. keep obligations durable while no host client is attached.

### 3. Implement router-driven attach-trigger evaluation

Required work:

1. watch obligation and session-state changes,
2. evaluate policy and boundary truth,
3. coalesce eligible obligations by session,
4. claim one attach episode per session,
5. record explanation-ready attach outcomes.

### 4. Implement continuity-first / fresh-fallback attach execution

Required work:

1. prefer continuity when valid,
2. fall back to fresh attach when continuity is unavailable but persisted attach truth remains valid,
3. keep attach-mode choice explicit and auditable,
4. preserve the 29.75 narrowing-only attach overlay rules.

### 5. Preserve fail-closed public world follow-up before attach

Required work:

1. keep detached or born-unattached world follow-up non-routable,
2. ensure errors point to sanctioned attach recovery,
3. avoid any implicit world continuation before host ownership is restored.

## Draft Acceptance Shape

This slice should only be promoted out of draft once:

1. 28.5 and 29 have landed,
2. 29.75 has landed the final authoritative attach-truth floor,
3. 30 has landed the public host-rooted world-start entrypoint,
4. born-unattached versus parked versus awaiting-attention status truth is operator-visible,
5. unresolved host work persists as obligations,
6. auto-trigger attach from eligible obligations is implemented and session-coalesced,
7. attach-worker mode selection is explicit and testable,
8. detached-world follow-up remains fail-closed before host attach.

## Draft Validation Targets

When this slice is eventually promoted, validation must include:

1. creating or surfacing a valid born-unattached host-rooted session posture,
2. persisting pending host-side obligations durably before attach,
3. continuity attach when a valid backend-native session exists,
4. fresh attach when no backend-native session exists,
5. one attach episode for multiple eligible obligations in the same session,
6. no synthetic prompt submission during either attach mode,
7. successful manual `reattach` superseding auto-attach cleanly,
8. detached-world fail-closed behavior before host attach,
9. obligation review state remaining unresolved after attach until the host explicitly works it.

## Sequencing

Current stack status:

1. [28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md](28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md): implementation-ready earlier slice.
2. [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](29-shared-agent-dispatch-envelope-and-capability-override-contract.md): implementation-ready earlier slice.
3. [29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md](29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md): final attach-truth authority floor.
4. [30-public-world-scoped-agent-start-and-capability-flags.md](30-public-world-scoped-agent-start-and-capability-flags.md): public host-first path floor.
5. [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md): canonical durable deferred-work model.
6. [DESIGN-auto-attach-trigger-and-work-queue-contract.md](./DESIGN-auto-attach-trigger-and-work-queue-contract.md): attach-processing projection.
7. [DESIGN-router-daemon-attach-trigger-integration.md](./DESIGN-router-daemon-attach-trigger-integration.md): router coalescing and attach-mode integration.
8. This SOW: specialized born-unattached and auto-trigger-attach realization slice after those floors.
