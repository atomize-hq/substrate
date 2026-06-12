# PLAN-56: Read-Side And Strict Control-Surface Hardening

Source spec: [SPEC-56-read-side-and-strict-control-surface-hardening.md](./SPEC-56-read-side-and-strict-control-surface-hardening.md)  
Source tracker note: [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)  
Source gap matrix: [AGENT_ORCHESTRATION_GAP_MATRIX.md](../AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Prior slice: [PLAN-55-broader-caller-surface-contract-freeze.md](./PLAN-55-broader-caller-surface-contract-freeze.md)  
Plan type: read-side/control-contract hardening above the landed caller-surface freeze  
Phase: `PLAN`  
Status: draft on `2026-06-12`

## Objective

Land the next narrow hardening seam after Slice `55` by finishing the read-side vs strict-control contract, participant-aware fallback behavior, and canonical session-handle naming without widening selectors or reopening caller-surface debates.

This slice is complete only when:

1. `agent status` remains readable for the targeted degraded/torn cases,
2. strict control surfaces remain fail-closed and clearly separate from degraded status logic,
3. participant-aware fallback/suppression/correlation is consistent when trace evidence provides participant identity,
4. participant-less fallback remains explicit and warning-bearing rather than overclaimed as exact,
5. `orchestration_session_id` is frozen as the forward public session handle,
6. `session_handle_id` / `active_session_handle_id` are either removed from forward-facing semantics or explicitly marked as compatibility-only temporary artifacts,
7. the slice lands without `--current`, fuzzy selectors, default-agent routing, or broader governance cleanup.

## Phase Gate

This plan assumes the `SPECIFY` artifact in [SPEC-56-read-side-and-strict-control-surface-hardening.md](./SPEC-56-read-side-and-strict-control-surface-hardening.md) has been reviewed and accepted before implementation starts.

## Tracker Update Rule

The running sequencing source remains:

- [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)

The broader product-truth tracker that should stay aligned with that note is:

- [AGENT_ORCHESTRATION_GAP_MATRIX.md](../AGENT_ORCHESTRATION_GAP_MATRIX.md)

During execution:

1. record any newly discovered degraded-status/control-split findings in the remaining-scope note only if they materially change follow-on order,
2. keep the gap matrix aligned to the same narrowed slice truth,
3. do **not** widen this slice into general governance cleanup just because legacy-handle names are still visible in storage.

## Repo-Truth Framing

What is already landed:

1. caller-surface contract freeze from Slice `55`,
2. warning-bearing degraded session discovery for several torn-root cases,
3. fail-closed strict live-session/control resolution,
4. status identity keys that already carry `participant_id` when available,
5. public `start|turn|reattach|fork|stop` surfaces that already use exact `orchestration_session_id` plus exact backend targeting where applicable.

What remains as the narrow hardening seam:

1. make the read-side vs strict-control split fully explicit and consistent,
2. finish participant-aware fallback/suppression/correlation where evidence exists,
3. keep participant-less fallback honest and warning-bearing,
4. de-canonicalize `session_handle_id` / `active_session_handle_id` from forward-facing semantics,
5. lock this truth with targeted regression coverage rather than broader selector ergonomics or storage-governance churn.

## Locked Decisions

### What this slice changes

1. It hardens and normalizes the degraded-read contract for `substrate agent status`.
2. It preserves strict fail-closed resolution for control surfaces and makes that split explicit.
3. It sharpens participant-aware fallback behavior and warnings.
4. It freezes forward public naming around `orchestration_session_id`.
5. It treats `session_handle_id` / `active_session_handle_id` as temporary compatibility/storage artifacts only if they remain at all.

### What this slice does not change

1. no `--current` or fuzzy selectors,
2. no default-agent routing,
3. no caller-surface redesign,
4. no toolbox mutation/public write expansion,
5. no Family-2 ingress/federation work,
6. no broad storage/schema/governance cleanup beyond de-canonicalizing legacy-handle semantics.

## Major Components And Dependencies

1. **status projection and warning surfaces**
   - `crates/shell/src/execution/agents_cmd.rs`
   - owns status JSON/text shape, fallback projection, nested correlation, and warning emission

2. **strict control/session resolution**
   - `crates/shell/src/execution/agent_runtime/state_store.rs`
   - owns strict live-session resolution and degraded/torn session-record construction

3. **participant/session naming and compatibility aliases**
   - `crates/shell/src/execution/agent_runtime/session.rs`
   - `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
   - own canonical vs legacy handle vocabulary

4. **status/control contract truth docs**
   - `AGENT_ORCHESTRATION_GAP_MATRIX.md`
   - `llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md`
   - `llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md`
   - `llm-last-mile/TASKS-56.md`

5. **regression floor**
   - `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
   - unit tests in `state_store.rs`
   - unit tests in `session.rs`

## Plan Summary

The narrowest honest Slice `56` is:

1. freeze the read-side vs strict-control contract and naming decisions first,
2. normalize degraded status rendering and warning behavior second,
3. finish participant-aware fallback/suppression/correlation and compatibility naming cleanup third,
4. finish with targeted validation and repo-truth closeout last.

## Implementation Order

### Packet 1: Freeze The Read-Side / Strict-Control Contract

Goal:

1. define exactly which degraded cases should still render on `agent status`,
2. define exactly which selectors/control surfaces remain fail-closed,
3. freeze the forward naming contract around `orchestration_session_id`,
4. record the greenfield posture that `session_handle_id` / `active_session_handle_id` are not part of the forward public contract.

Primary touch surface:

1. `llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md`
2. `AGENT_ORCHESTRATION_GAP_MATRIX.md`
3. bounded comments/docs only if the naming contract is currently contradicted

Why first:

1. later packets need one stable contract for degraded reads vs strict control,
2. this prevents implementation from accidentally weakening control selectors,
3. it keeps the legacy-handle decision bounded before code changes start.

Verification checkpoint:

1. the slice has one explicit rule for readable degraded status,
2. the slice has one explicit rule for fail-closed control surfaces,
3. the repo has one explicit statement that `orchestration_session_id` is the only forward public session handle.

### Packet 2: Normalize Degraded Status Rendering And Warning Truth

Goal:

1. make `agent status` consistently return warning-bearing output for the targeted torn/degraded cases,
2. keep status rows readable without pretending ambiguous state is authoritative,
3. ensure the read surface explains when it is degraded and why.

Primary touch surface:

1. `crates/shell/src/execution/agents_cmd.rs`
2. `crates/shell/src/execution/agent_runtime/state_store.rs`

Why second:

1. Packet `1` defines what is allowed on the read surface,
2. this is the first packet that changes operator-visible runtime behavior,
3. later fallback cleanup should build on a stable degraded-read contract rather than invent one implicitly.

Verification checkpoint:

1. `agent status --json` succeeds for the targeted torn/degraded cases,
2. warnings are explicit rather than silent,
3. strict control helpers remain unchanged in their fail-closed behavior.

### Packet 3: Finish Participant-Aware Fallback And Legacy-Handle De-Canonicalization

Goal:

1. make fallback suppression/correlation participant-aware wherever trace evidence permits,
2. keep participant-less fallback explicit and coarse,
3. de-canonicalize `session_handle_id` / `active_session_handle_id` in status/control/docs/comments,
4. preserve compatibility reads if full physical removal would widen scope.

Primary touch surface:

1. `crates/shell/src/execution/agents_cmd.rs`
2. `crates/shell/src/execution/agent_runtime/session.rs`
3. `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
4. bounded docs/comments if needed

Why third:

1. Packet `2` should first stabilize the degraded-read surface,
2. this packet is the semantic cleanup that keeps sibling participants and naming truth honest,
3. it is also the place to enforce the user’s greenfield preference without forcing a broad schema cutover.

Verification checkpoint:

1. sibling participants remain distinct when participant evidence exists,
2. participant-less fallback emits warnings and remains coarse,
3. forward-facing wording no longer treats legacy handle names as public contract.

### Packet 4: Validation Wall And Repo-Truth Closeout

Goal:

1. run the bounded validation wall,
2. confirm the slice stayed narrow,
3. update slice-local docs/tracker truth only as needed to reflect the landed contract,
4. make any retained legacy-handle temporary posture explicit in the closeout wording.

Primary touch surface:

1. `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
2. unit tests in `state_store.rs`
3. unit tests in `session.rs`
4. Slice `56` spec/plan/tasks docs
5. `AGENT_ORCHESTRATION_GAP_MATRIX.md` only if bounded truth alignment is needed

Why last:

1. repo truth should describe what actually landed,
2. this packet is where the slice proves it did not widen into ergonomics or governance cleanup,
3. the temporary compatibility posture for legacy handles should be documented only once the implementation truth is known.

Verification checkpoint:

1. targeted status/control/naming tests are green,
2. fmt/clippy are green for any touched Rust files,
3. the final docs say clearly whether legacy-handle names remain temporarily and in what limited role.

## Risks And Mitigations

1. **Risk: the slice weakens strict control resolution instead of separating read-side logic cleanly**
   - Mitigation: keep `resolve_single_live_session_for_agent(...)` strict and move permissive behavior into status-specific projection/read helpers only.

2. **Risk: participant-aware fallback is only partially fixed**
   - Mitigation: treat pure-session projection, suppression, and nested parent correlation as one packet with shared regression coverage.

3. **Risk: legacy-handle naming cleanup widens into storage/schema churn**
   - Mitigation: prefer de-canonicalization plus explicit temporary-compatibility notes; stop before broad field/storage rewrites unless they prove truly trivial.

4. **Risk: warnings become noisy without clarifying operator truth**
   - Mitigation: make warnings specific about whether the issue is missing parent metadata, missing participant linkage, or coarse trace fallback.

5. **Risk: the slice drifts into selector ergonomics**
   - Mitigation: keep `--current`, repair/reap flows, and richer operator UX explicitly out of scope.

## Parallelism Guidance

1. Packet `1` must happen first because it freezes the naming and read/control contract.
2. After Packet `1`, Packet `2` and Packet `3` can be explored in parallel, but final merge/reconciliation should still be centralized because both touch `agents_cmd.rs` and the public operator contract.
3. Packet `4` must happen last.

## Success Markers

1. The repo has one stable answer for when status degrades instead of failing.
2. The repo has one stable answer for which surfaces remain fail-closed.
3. The repo has one stable answer for how participant-aware fallback behaves.
4. The repo has one stable answer for which session identifier is public and canonical.
5. The slice lands as bounded status/control hardening rather than selector ergonomics or broad governance cleanup.
