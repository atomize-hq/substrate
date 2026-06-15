# Spec: Read-Side And Strict Control-Surface Hardening

Source tracker note: [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)  
Source gap matrix: [AGENT_ORCHESTRATION_GAP_MATRIX.md](../AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Prior numbered slice: [SPEC-55-broader-caller-surface-contract-freeze.md](./SPEC-55-broader-caller-surface-contract-freeze.md)  
Companion inputs:
- [18-status-surface-and-session-handle-hardening.md](./18-status-surface-and-session-handle-hardening.md)
- [19-public-agent-control-surfaces.md](./19-public-agent-control-surfaces.md)
- [`crates/shell/src/execution/agents_cmd.rs`](../crates/shell/src/execution/agents_cmd.rs)
- [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
- [`crates/shell/src/execution/agent_runtime/session.rs`](../crates/shell/src/execution/agent_runtime/session.rs)
- [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](../crates/shell/src/execution/agent_runtime/orchestration_session.rs)
- [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](../crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
Phase: `SPECIFY`  
Status: Packet `4` checkpoint-green closeout on `2026-06-12`

## Closeout Note

Slice `56` is now checkpoint-green in the current tree.

1. The read-side vs strict-control split is landed and verified.
2. Participant-aware fallback stays sibling-distinct when trace evidence exists, while participant-less fallback remains explicitly coarse and warning-bearing by design.
3. `orchestration_session_id` remains the only forward public session handle.
4. Retained `session_handle_id` / `active_session_handle_id` usage remains temporary compatibility/storage-only state rather than forward operator vocabulary.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `55` closed the caller-surface contract freeze, so the next honest execution-bearing seam is read-side and strict control-surface hardening rather than another caller-surface or runtime-family parity slice.
2. The current tree already lands part of this seam:
   - `substrate agent status` already returns warning-bearing degraded output in several torn-root cases,
   - `StatusIdentityKey` already carries `participant_id` when present,
   - strict control surfaces such as `toolbox status|env` already remain fail-closed.
3. The older [18-status-surface-and-session-handle-hardening.md](./18-status-surface-and-session-handle-hardening.md) SOW is still directionally useful, but parts of its framing are stale because public `substrate agent start|turn|reattach|fork|stop` now already exist in the current tree.
4. The user’s greenfield preference is that `session_handle_id` and `active_session_handle_id` do **not** remain part of the forward public contract. If removing them entirely would widen scope too much, Slice `56` may keep them as bounded compatibility/storage artifacts only, but must mark them as non-canonical and temporary.
5. This slice should not widen selectors, introduce `--current`, add default-agent routing, redesign `substrate -c`, or reopen Family-2 / platform-parity / governance work beyond the narrow naming clarifications needed here.
6. If implementation discovers that full removal of legacy handle names is entangled with a larger storage/schema cutover, Slice `56` should stop at de-canonicalization plus explicit repo-truth notes rather than forcing a broad rename slice implicitly.

If any of these are wrong, correct them before implementation.

## Objective

Harden the operator-facing read-side truth and strict-control contract so the repo has one clear answer for:

1. when `substrate agent status` should degrade with warnings instead of failing,
2. when strict control/read-projection surfaces must still fail closed,
3. how participant-aware trace fallback should behave when authoritative state is incomplete,
4. which identifiers are canonical in public/operator-facing status and control contracts,
5. and how to treat legacy `session_handle_id` / `active_session_handle_id` naming without letting them define the forward product surface.

## Tech Stack

- Rust workspace (`cargo`)
- `crates/shell` status, control-surface, and runtime-state code
- `llm-last-mile/` spec and design docs
- top-level sequencing and product-truth docs:
  - [`AGENT_ORCHESTRATION_GAP_MATRIX.md`](../AGENT_ORCHESTRATION_GAP_MATRIX.md)
  - [`REMAINING-overall-scope-2026-06-10.md`](./REMAINING-overall-scope-2026-06-10.md)
- targeted regression surfaces:
  - [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](../crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
  - unit tests in [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
  - unit tests in [`crates/shell/src/execution/agent_runtime/session.rs`](../crates/shell/src/execution/agent_runtime/session.rs)

This slice is complete only when:

1. `substrate agent status` remains readable for torn/ambiguous-but-reportable state and surfaces explicit warnings instead of aborting the whole read surface,
2. strict control-plane surfaces (`toolbox status|env`, selector resolution for control actions, and adjacent authorization paths) remain fail-closed on ambiguity or stale linkage,
3. participant-aware fallback/suppression/correlation behave consistently when `participant_id` / `parent_participant_id` evidence exists,
4. participant-less trace fallback stays explicitly marked as coarse and warning-worthy rather than pretending to be exact,
5. `orchestration_session_id` is the only forward public session handle in docs/status/control wording,
6. `session_handle_id` and `active_session_handle_id` are either removed from forward-facing semantics or explicitly marked as compatibility-only temporary artifacts,
7. the slice lands without widening selectors, adding new control affordances, or reopening unrelated governance cleanup.

## Commands

Build:

```bash
cargo build --workspace
```

Format:

```bash
cargo fmt --all -- --check
```

Lint:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Targeted status/control regression coverage:

```bash
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell session -- --nocapture
```

Targeted repo-truth greps:

```bash
rg -n "orchestration_session_id|participant_id|session_handle_id|active_session_handle_id|trace_fallback|toolbox status|toolbox env" \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution
```

## Project Structure

```text
crates/shell/src/execution/agents_cmd.rs
  -> operator-facing status/toolbox rendering and trace-fallback correlation

crates/shell/src/execution/agent_runtime/state_store.rs
  -> authoritative session/participant reads, strict live-session resolution, degraded session-record warnings

crates/shell/src/execution/agent_runtime/session.rs
  -> participant/session serialization, compatibility aliases, internal surfaced backend ids

crates/shell/src/execution/agent_runtime/orchestration_session.rs
  -> authoritative parent-session record, including active participant linkage

crates/shell/tests/agent_successor_contract_ahcsitc0.rs
  -> end-to-end shell contract coverage for status/fallback/selection truth

llm-last-mile/
  -> slice specs, plans, tasks, and repo-truth reasoning for the status/control contract
```

## Code Style

Use narrow, fail-closed helpers for control-plane selection and separate permissive, warning-bearing helpers for read-only status projection.

```rust
let live_selection = store.resolve_single_live_session_for_agent(&orchestrator_agent_id)?;
let status_sessions = store.list_status_sessions_for_agent(&orchestrator_agent_id)?;

if live_selection.is_none() && !status_sessions.is_empty() {
    warnings.push(
        "status is degraded because authoritative live-session selection is incomplete".to_string(),
    );
}
```

Key conventions:

1. keep public/operator naming canonical around `orchestration_session_id`,
2. keep `participant_id` explicit when status evidence is participant-scoped,
3. preserve `anyhow::Context` / `config_model::user_error(...)` patterns for clear fail-closed control errors,
4. do not repurpose compatibility field names as forward product vocabulary in new code or docs.

## Testing Strategy

Framework:

- Rust unit and integration tests via `cargo test`

Test locations:

1. `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
   - status rendering, trace fallback, invalidation suppression, selector truth
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
   - degraded/torn-root session discovery vs strict live-session resolution
3. `crates/shell/src/execution/agent_runtime/session.rs`
   - compatibility alias reads and canonical naming expectations

Coverage expectations:

1. prove readable degraded status survives torn parent/participant linkage,
2. prove strict selectors still fail closed on ambiguity or stale linkage,
3. prove participant-aware fallback stays sibling-distinct when evidence exists,
4. prove participant-less fallback emits warnings and remains coarse,
5. prove canonical public naming stays `orchestration_session_id`,
6. prove any retained `session_handle_id` / `active_session_handle_id` usage is compatibility-only rather than forward public contract.

## Boundaries

- Always:
  - keep `orchestration_session_id` as the only forward public session selector
  - keep control-plane authorization helpers fail-closed
  - preserve live authoritative state over trace fallback
  - document any retained legacy-handle naming as compatibility-only
- Ask first:
  - removing or renaming on-disk storage fields if that requires a broad compatibility/schema cutover
  - widening any public selector ergonomics (`--current`, fuzzy lookup, latest-session heuristics)
  - changing gateway-status schema ownership as part of this slice
- Never:
  - use trace-only history to authorize control actions
  - make `participant_id`, `session_handle_id`, `active_session_handle_id`, or upstream backend ids into public control selectors
  - widen this slice into Family-2, platform-parity, toolbox mutation, or caller-surface redesign

## Current Repo-Truth Gut Check

### 1. Status degradation already exists, but the contract is still uneven

Current code already shows:

1. session-record construction can preserve warnings for missing parent/linkage truth,
2. `agent status` already returns top-level `warnings`,
3. torn roots can remain visible on the read surface,
4. but the remaining read/control split still needs one explicit contract-cleanup pass.

Repo-truth consequence:

1. Slice `56` should finish and normalize this behavior, not pretend it is still absent,
2. the slice should focus on clearer degraded operator truth rather than a fresh status redesign.

### 2. Strict control selection is correct and must remain separate

Current strict helpers intentionally fail closed for:

1. multiple active parents,
2. missing or stale active participant linkage,
3. inactive selected participants,
4. live-child-without-live-parent situations.

Repo-truth consequence:

1. Slice `56` must preserve this strictness for control paths,
2. the slice should **not** weaken `resolve_single_live_session_for_agent(...)`,
3. instead it should sharpen the distinction between permissive status reads and strict control resolution.

### 3. Participant-aware fallback is partially landed, not fully closed

Current status code already includes:

1. `participant_id` in `StatusIdentityKey`,
2. exact and coarse suppression sets,
3. warnings for participant-less coarse fallback in some cases.

Repo-truth consequence:

1. Slice `56` should finish the remaining participant-aware correlation/suppression cleanup,
2. participant-less fallback should remain explicitly coarse instead of being overclaimed as exact.

### 4. Naming is still muddied by legacy handle vocabulary

Current runtime/state records still mix:

1. canonical `orchestration_session_id`,
2. subordinate `participant_id`,
3. compatibility/storage names such as `session_handle_id` / `active_session_handle_id`,
4. internal backend-native ids such as `internal.uaa_session_id`.

Repo-truth consequence:

1. Slice `56` must freeze the forward contract around `orchestration_session_id`,
2. the slice must explicitly de-canonicalize `session_handle_id` / `active_session_handle_id`,
3. if full removal is too broad, the repo must say clearly that these are temporary compatibility/storage artifacts only.

## Frozen Decision Ledger

| Contract area | Allowed in Slice `56` | Not allowed in Slice `56` | Deferred follow-on |
|---|---|---|---|
| Read-side degradation | warning-bearing readable `agent status` for torn/ambiguous-but-reportable state | whole-surface failure when valid degraded reporting is possible | richer repair/reap/remediation UX |
| Strict control resolution | fail-closed `toolbox status|env` and control selectors | using degraded status or trace-only history to authorize control | bounded later selector ergonomics |
| Trace fallback | participant-aware selection/suppression/correlation when evidence exists; explicit warnings when it does not | silently collapsing sibling participants or pretending coarse fallback is exact | deeper producer-side trace cleanup if new gaps remain |
| Public naming | `orchestration_session_id` as public session handle; `participant_id` as subordinate diagnostics | treating `session_handle_id`, `active_session_handle_id`, or upstream backend ids as forward public selectors | later physical field/storage cleanup |
| Legacy-handle posture | compatibility-only retention if needed for bounded landing | preserving legacy names as meaningful public contract | governance slice to remove or rename them fully |
| Selector ergonomics | exact current selectors only | `--current`, fuzzy lookup, latest-session heuristics, or caller-surface widening | separate ergonomics slice |

### Decision 1: readable degraded status is allowed; degraded control is not

Allowed:

1. `substrate agent status` returns partial-but-valid rows with warnings,
2. status output distinguishes degraded read truth from authoritative live truth where possible.

Disallowed:

1. using degraded status output as a control target,
2. letting trace-only history back-authorize `toolbox` or control paths.

### Decision 2: participant-aware fallback is the target; coarse fallback must stay honest

Allowed:

1. participant-aware identity and suppression when participant evidence is present,
2. coarse fallback only when that evidence is absent,
3. explicit warnings whenever correlation had to remain coarse.

Disallowed:

1. collapsing sibling participants without warning,
2. silently suppressing unrelated siblings because the key stayed too coarse.

### Decision 3: `orchestration_session_id` is the only forward public session handle

Allowed:

1. public/operator-facing status and control wording centered on `orchestration_session_id`,
2. `participant_id` shown only as subordinate runtime lineage/diagnostic truth.

Disallowed:

1. documenting `session_handle_id` or `active_session_handle_id` as meaningful public handles,
2. asking operators to use backend-native/upstream ids directly.

### Decision 4: legacy handle names may remain only as temporary compatibility/storage artifacts

Allowed:

1. retaining `session_handle_id` / `active_session_handle_id` temporarily if removing them would force a larger compatibility/storage migration,
2. comments/tests/docs that explicitly mark them as non-canonical and temporary.

Disallowed:

1. new public docs or new status/control wording that present them as forward contract,
2. broad governance cleanup beyond this de-canonicalization unless a new slice is opened for it.

## Success Criteria

1. `substrate agent status` produces useful output plus warnings for the degraded cases this slice targets.
2. Strict control surfaces stay fail-closed and do not inherit permissive status logic.
3. Participant-aware fallback/suppression behaves consistently when trace identity evidence exists.
4. Coarse fallback is warning-bearing and explicitly non-authoritative when participant evidence is missing.
5. Repo docs/specs/comments/tests consistently treat `orchestration_session_id` as the canonical public session handle.
6. Any retained `session_handle_id` / `active_session_handle_id` usage is explicitly compatibility-only and called out as temporary.
7. No new selector affordances, caller-surface widenings, or unrelated governance work sneak into the slice.

## Open Questions

1. No blocking product question remains for spec creation; default assumption is that full physical removal/rename of legacy handle fields is **out of scope** for Slice `56` unless it proves surprisingly trivial.
