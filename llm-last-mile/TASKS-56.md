# TASKS-56: Read-Side And Strict Control-Surface Hardening

Source spec: [SPEC-56-read-side-and-strict-control-surface-hardening.md](./SPEC-56-read-side-and-strict-control-surface-hardening.md)  
Source plan: [PLAN-56-read-side-and-strict-control-surface-hardening.md](./PLAN-56-read-side-and-strict-control-surface-hardening.md)  
Execution model: four sequential implementation packets  
Phase: `TASKS`  
Status: draft on `2026-06-12`

## Current tree note

1. Slice `55` landed the caller-surface contract freeze and is no longer the active seam.
2. The next honest seam is status/session-handle/read-side hardening rather than another caller-surface or runtime-family slice.
3. The current tree already lands part of this seam:
   - `agent status` can already degrade with warnings for some torn-root cases,
   - strict control surfaces already remain fail-closed,
   - participant-aware identity is already partly present in fallback code.
4. Slice `56` therefore finishes and normalizes this contract rather than inventing it from scratch.
5. The user’s greenfield preference is that `session_handle_id` and `active_session_handle_id` should not survive as forward public contract; if they remain temporarily, Slice `56` must say so explicitly.

## Task List

- [ ] Task 56.1: Freeze the read-side vs strict-control contract and canonical naming
  - Acceptance:
    - the slice docs explicitly distinguish warning-bearing readable `agent status` from fail-closed control surfaces
    - the slice docs explicitly freeze `orchestration_session_id` as the only forward public session handle
    - the slice docs explicitly state that `session_handle_id` and `active_session_handle_id` are not forward public contract and, if retained, are compatibility/storage artifacts only
    - no updated doc implies selector widening, fuzzy lookup, or control authorization from degraded status
  - Verify:
    - `rg -n "orchestration_session_id|session_handle_id|active_session_handle_id|fail-closed|degraded|warning" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md`
  - Files:
    - `llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md`
    - `llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md`
    - `llm-last-mile/TASKS-56.md`
    - `AGENT_ORCHESTRATION_GAP_MATRIX.md` (only if bounded truth alignment is needed)

- [ ] Task 56.2: Normalize degraded `agent status` rendering without weakening strict control helpers
  - Acceptance:
    - `substrate agent status --json` returns warning-bearing output for the targeted torn/degraded cases rather than aborting the whole read surface
    - the degraded warnings explain the specific missing/ambiguous condition instead of silently dropping truth
    - strict control-plane helpers remain fail-closed and are not relaxed to share status permissiveness
    - no touched code turns degraded status into a control selector source
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Files:
    - `crates/shell/src/execution/agents_cmd.rs`
    - `crates/shell/src/execution/agent_runtime/state_store.rs`
    - `crates/shell/tests/agent_successor_contract_ahcsitc0.rs` (only for bounded regression updates)

- [ ] Task 56.3: Finish participant-aware fallback/suppression/correlation and keep coarse fallback honest
  - Acceptance:
    - sibling participants remain distinct when `participant_id` / `parent_participant_id` evidence exists
    - participant-aware suppression only suppresses matching fallback rows rather than unrelated siblings
    - participant-less fallback emits explicit warnings and remains coarse rather than pretending to be exact
    - nested correlation stays fail-closed on malformed selected rows
  - Verify:
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Files:
    - `crates/shell/src/execution/agents_cmd.rs`
    - `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`

- [ ] Task 56.4: De-canonicalize legacy handle naming while preserving bounded compatibility if needed
  - Acceptance:
    - forward-facing status/control/docs/comments treat `orchestration_session_id` as canonical and `participant_id` as subordinate runtime identity
    - `session_handle_id` / `active_session_handle_id` are no longer described as meaningful public handles
    - if legacy handle names remain in storage or serde aliases, the code/docs/comments explicitly mark them as compatibility-only temporary artifacts
    - the slice does not force a broad storage/schema/governance migration unless it proves truly trivial
  - Verify:
    - `cargo test -p shell session -- --nocapture`
    - `rg -n "session_handle_id|active_session_handle_id" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile`
  - Files:
    - `crates/shell/src/execution/agent_runtime/session.rs`
    - `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
    - bounded status/docs/comments files only if needed

- [ ] Task 56.5: Close the Slice 56 validation wall and document any temporary compatibility posture explicitly
  - Acceptance:
    - fmt/clippy and targeted status/control/naming regression coverage are green for touched Rust files
    - the final spec/plan/tasks/docs wording is internally consistent
    - the slice clearly states whether `session_handle_id` / `active_session_handle_id` remain temporarily and in what limited compatibility role
    - the slice remains a bounded status/control hardening slice rather than selector ergonomics or broad governance cleanup
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell session -- --nocapture`
  - Files:
    - files touched by Tasks `56.1` through `56.4`
    - `llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md`
    - `llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md`
    - `llm-last-mile/TASKS-56.md`
