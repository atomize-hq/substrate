# TASKS-56: Read-Side And Strict Control-Surface Hardening

Source spec: [SPEC-56-read-side-and-strict-control-surface-hardening.md](./SPEC-56-read-side-and-strict-control-surface-hardening.md)  
Source plan: [PLAN-56-read-side-and-strict-control-surface-hardening.md](./PLAN-56-read-side-and-strict-control-surface-hardening.md)  
Execution model: four sequential implementation packets  
Phase: `TASKS`  
Status: Packet `4` checkpoint-green closeout on `2026-06-12`

## Current tree note

1. Slice `55` landed the caller-surface contract freeze and is no longer the active seam.
2. The next honest seam is status/session-handle/read-side hardening rather than another caller-surface or runtime-family slice.
3. The current tree already lands the full narrow seam:
   - `agent status` can already degrade with warnings for some torn-root cases,
   - strict control surfaces already remain fail-closed,
   - participant-aware fallback identity is already present where trace evidence permits it,
   - retained legacy handle names are already bounded to compatibility/storage-only posture.
4. Slice `56` therefore closes by validating and documenting the landed contract rather than inventing it from scratch.
5. The user’s greenfield preference is that `session_handle_id` and `active_session_handle_id` should not survive as forward public contract; if they remain temporarily, Slice `56` must say so explicitly.

## Task List

- [x] Task 56.1: Freeze the read-side vs strict-control contract and canonical naming
  - Acceptance:
    - the slice docs explicitly distinguish warning-bearing readable `agent status` from fail-closed control surfaces
    - the slice docs explicitly freeze `orchestration_session_id` as the only forward public session handle
    - the slice docs explicitly state that `session_handle_id` and `active_session_handle_id` are not forward public contract and, if retained, are compatibility/storage artifacts only
    - no updated doc implies selector widening, fuzzy lookup, or control authorization from degraded status
  - Checkpoint note:
    - Packet `1` landed as a docs-only contract freeze on `2026-06-12`; no Rust production-symbol edits or GitNexus impact run were required
  - Verify:
    - `rg -n "orchestration_session_id|session_handle_id|active_session_handle_id|fail-closed|degraded|warning" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md`
  - Files:
    - `llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md`
    - `llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md`
    - `llm-last-mile/TASKS-56.md`
    - `AGENT_ORCHESTRATION_GAP_MATRIX.md` (only if bounded truth alignment is needed)

- [x] Task 56.2: Normalize degraded `agent status` rendering without weakening strict control helpers
  - Acceptance:
    - `substrate agent status --json` returns warning-bearing output for the targeted torn/degraded cases rather than aborting the whole read surface
    - the degraded warnings explain the specific missing/ambiguous condition instead of silently dropping truth
    - strict control-plane helpers remain fail-closed and are not relaxed to share status permissiveness
    - no touched code turns degraded status into a control selector source
  - Checkpoint note:
    - Packet `2` verified green on `2026-06-12` in the current workspace: the targeted degraded/torn `agent status` cases already render warning-bearing output, the paired fail-closed toolbox/control assertions already hold, GitNexus impact review stayed low-risk and bounded to the status path, and no additional production-symbol edits were required
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Files:
    - `crates/shell/src/execution/agents_cmd.rs`
    - `crates/shell/src/execution/agent_runtime/state_store.rs`
    - `crates/shell/tests/agent_successor_contract_ahcsitc0.rs` (only for bounded regression updates)

- [x] Task 56.3: Finish participant-aware fallback/suppression/correlation and keep coarse fallback honest
  - Acceptance:
    - sibling participants remain distinct when `participant_id` / `parent_participant_id` evidence exists
    - participant-aware suppression only suppresses matching fallback rows rather than unrelated siblings
    - participant-less fallback emits explicit warnings and remains coarse rather than pretending to be exact
    - nested correlation stays fail-closed on malformed selected rows
  - Checkpoint note:
    - Packet `3` was already present in the current workspace by the time Packet `4` closeout began; the targeted `agent_successor_contract_ahcsitc0` regression floor verified that sibling-specific suppression, participant-aware nested correlation, and explicit coarse fallback warnings are green without reopening Packets `1`-`2`
  - Verify:
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Files:
    - `crates/shell/src/execution/agents_cmd.rs`
    - `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`

- [x] Task 56.4: De-canonicalize legacy handle naming while preserving bounded compatibility if needed
  - Acceptance:
    - forward-facing status/control/docs/comments treat `orchestration_session_id` as canonical and `participant_id` as subordinate runtime identity
    - `session_handle_id` / `active_session_handle_id` are no longer described as meaningful public handles
    - if legacy handle names remain in storage or serde aliases, the code/docs/comments explicitly mark them as compatibility-only temporary artifacts
    - the slice does not force a broad storage/schema/governance migration unless it proves truly trivial
  - Checkpoint note:
    - Packet `3` also landed the bounded legacy-handle posture already visible in the current tree: canonical writes omit the legacy field names, compatibility reads still deserialize them, and runtime comments/docs keep `orchestration_session_id` as the only forward public session handle
  - Verify:
    - `cargo test -p shell session -- --nocapture`
    - `rg -n "session_handle_id|active_session_handle_id" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile`
  - Files:
    - `crates/shell/src/execution/agent_runtime/session.rs`
    - `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
    - bounded status/docs/comments files only if needed

- [x] Task 56.5: Close the Slice 56 validation wall and document any temporary compatibility posture explicitly
  - Acceptance:
    - fmt/clippy and targeted status/control/naming regression coverage are green for touched Rust files
    - the final spec/plan/tasks/docs wording is internally consistent
    - the slice clearly states whether `session_handle_id` / `active_session_handle_id` remain temporarily and in what limited compatibility role
    - the slice remains a bounded status/control hardening slice rather than selector ergonomics or broad governance cleanup
  - Checkpoint note:
    - Packet `4` closed green on `2026-06-12` after reconciling the slice-local docs/tracker truth to the already-landed Packet `3` runtime state and fixing one test-only `rustfmt` wrap in `session.rs`; no production-symbol edits or selector-surface widening were required, and retained `session_handle_id` / `active_session_handle_id` posture remains explicitly compatibility/storage-only temporary state
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
