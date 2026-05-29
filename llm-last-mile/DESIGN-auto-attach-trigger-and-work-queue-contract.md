# Design: Auto-Attach Trigger and Attach-Processing Projection

Status: draft design input. This document no longer defines a second canonical durable work-queue ledger. In the greenfield architecture, the canonical durable truth is the obligation ledger defined in [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md). This document freezes the auto-attach projection over those obligations: eligibility, attach-processing state, claiming semantics, manual-reattach interaction, and fail-closed behavior.

## Why This Doc Exists

The obligation-ledger design already answers what durable obligation exists.

What still needs a dedicated contract is:

1. when an obligation becomes eligible for host attach processing,
2. how attach-processing state is tracked,
3. how manual `reattach` and automatic attach coexist,
4. how processing remains separate from review resolution.

Without this projection contract:

1. router behavior will invent ad hoc trigger rules,
2. attach work will drift back into hidden notification consumption,
3. manual and automatic attach will race without explicit state semantics.

## Relationship To Existing Decisions

This design composes with:

1. [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md): canonical durable artifact and obligation substate model.
2. [DESIGN-durable-orchestration-notification-inbox-contract.md](./DESIGN-durable-orchestration-notification-inbox-contract.md): review/inbox projection and `awaiting_attention` derivation.
   - A future host-global inbox under `SUBSTRATE_HOME` may feed this layer, but it must not replace the local obligation as the canonical attach-processing source.
3. [ADR-0047](../docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md): exact-session durable authority and `reattach`.
4. [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md): retained workers may enter `attention_pending`, but attach processing belongs to the host-session side.
5. [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md): auto-attach remains deny-by-default and policy-gated by obligation kind and boundary truth.

## Problem Statement

How might Substrate process auto-attach from unresolved obligations so that:

1. eligibility is deterministic,
2. attach-processing state is durable,
3. attach-processing does not become a second source of truth,
4. manual `reattach` and automatic attach share one canonical record,
5. review state and attach state remain separate.

## Frozen Direction

This design freezes the following:

1. auto-attach evaluates obligations, not raw stream events,
2. attach-processing state lives inside the obligation record,
3. there is no second canonical `work_queue` artifact,
4. attach-processing requests sanctioned host attach handling; it is not a prompt and not implicit worker continuation,
5. manual `reattach` remains canonical and may supersede automatic processing,
6. attach-processing completion does not resolve the obligation automatically.

## Non-Goals

This design does not:

1. define router/daemon implementation ownership,
2. define continuity attach versus fresh attach execution in detail,
3. define backoff timing or retry policy in full,
4. define a public queue/status CLI surface.

## Core Principle

Auto-attach is a processing projection over obligations.

The obligation says:

1. what is owed,
2. whether host attention is needed,
3. whether it is resolved.

The attach projection says:

1. whether automatic attach should be attempted,
2. whether processing is queued or claimed,
3. whether attach processing completed, was cancelled, or dead-lettered.

## Trigger Source Rule

The authoritative source for auto-attach processing is the unresolved obligation itself.

### Hard rule

No direct `stream event -> attach processing` shortcut is allowed.

The upstream sequence is:

1. worker event or runtime alert occurs,
2. an obligation is created if policy and event class require one,
3. auto-attach eligibility is then evaluated from that durable obligation plus current session truth.

## Eligibility Rule

An obligation may enter attach processing only when all of the following are true:

1. the parent orchestration session exists and is non-terminal,
2. the obligation remains unresolved,
3. there is no authoritative attached host participant,
4. policy allows auto-attach for this obligation kind,
5. exact session/world/attach truth needed for later attach is available or at least not disproven.

### Conservative v1 defaults

Eligible by default:

1. `follow_up_required`
2. `approval_required`
3. `blocked`
4. `fork_request`

Not eligible by default:

1. `task_completed`
2. `result_available`
3. `fork_recommendation`

Deferred unless policy explicitly opts in:

1. `task_failed`
2. `runtime_alert`
3. `escalation_recommended`

## Attach-Processing State

The attach-processing lifecycle is the obligation `attach_state`:

1. `not_requested`
2. `queued`
3. `claimed`
4. `completed`
5. `cancelled`
6. `dead_letter`

### Allowed transitions

```text
not_requested -> queued
not_requested -> dead_letter

queued -> claimed
queued -> cancelled
queued -> dead_letter

claimed -> completed
claimed -> cancelled
claimed -> dead_letter
```

No transition back to `queued` is required by this design.

Retry mechanics belong to the router/daemon integration doc.

## Meaning Of Attach States

1. `not_requested`
   - no automatic attach work is currently active or permitted for this obligation.
2. `queued`
   - the obligation is eligible and awaits a sanctioned processor.
3. `claimed`
   - one sanctioned processor is actively handling attach for this obligation.
4. `completed`
   - attach processing succeeded for this obligation.
   - this does not imply review resolution.
5. `cancelled`
   - attach processing became unnecessary or was superseded.
6. `dead_letter`
   - attach processing must stop under current truth or policy.

## Claiming And Dedupe

Because attach processing lives on the obligation itself:

1. `obligation_id` is the canonical join key,
2. one obligation can have at most one active attach-processing state at a time,
3. duplicate active queue rows are impossible by design if processors honor obligation state transitions.

### Required metadata

Any later implementation should preserve:

1. `attach_claim_owner`
2. `attach_attempt_count`
3. `attach_last_attempt_at`
4. `attach_completion_reason`

## No-Attached-Host Rule

If an authoritative attached host participant already exists, attach processing should not begin by default.

The obligation may still remain unresolved and attention-driving, but no host attach is needed because the host is already attached.

## Manual Reattach Coexistence

Manual exact-session `reattach` remains canonical and valid.

### Required coexistence rules

1. if manual `reattach` succeeds while `attach_state=queued`, the obligation should move to `cancelled` or `completed`,
2. if manual `reattach` succeeds while `attach_state=claimed`, the processor must converge without starting duplicate attach work,
3. manual `reattach` does not itself resolve the obligation,
4. automatic attach must not bypass exact session identity just because the manual path exists too.

## Review State Interaction

Review state and attach state are separate substates on the same obligation.

### Hard rules

1. `attach_state=completed` does not imply `review_state=resolved`,
2. `review_state=resolved|dismissed` may transition `attach_state` to `cancelled` if attach work is no longer needed,
3. host posture derives from review state and `attention_required`, not from attach state.

## Fail-Closed Rules

Attach processing must fail closed when:

1. the source orchestration session is missing,
2. the session is terminal,
3. the obligation is already resolved or dismissed,
4. required attach truth is missing,
5. authoritative world/session boundary truth is disproven,
6. an attached host participant already exists,
7. policy denies auto-attach for the obligation kind.

### Conservative v1 rule

When in doubt, preserve the obligation and keep `attach_state=not_requested`.

It is better to require manual `reattach` than to create unsound automatic attach behavior.

## Deferred To The Router Integration Doc

This document intentionally leaves the following to the next design:

1. who watches obligations for state changes,
2. how claiming is coordinated,
3. how continuity attach versus fresh attach is chosen,
4. how loops, retries, and backoff are handled,
5. what processor identity owns claims.

## Summary

The greenfield auto-attach model frozen here is:

1. obligations are the only durable trigger source,
2. attach-processing is a projection over obligations, not a sibling queue ledger,
3. manual and automatic attach share one canonical `obligation_id`,
4. attach-processing completion and obligation resolution are separate truths,
5. future host-global inbox or remote ingress layers may materialize local obligations, but they do not own local attach state once the obligation exists.
