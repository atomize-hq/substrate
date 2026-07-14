# R6-GAP-TGG-TRUTH-PATH-ACTION — Truth-Path Action-Before-Read Gap

Status: **ACTIVE — OPTION-A PACKET-AMENDMENT REVIEW GATE**. The operator has resolved `R6-TGG-CROSS-CHECKPOINT-PROVENANCE-01 = A`: repair the internal, typed, session-local, path-scoped truth-grounding provenance seam. Candidate `49b2bbd7f` (`docs: authorize typed grounding provenance`) has landed and is pending fresh review; Task 2A remains unchecked until the packet-amendment series actually receives fresh built-in `default` `REVIEW CLEAN`. This decision and landed candidate do not authorize implementation. The packet-required next gate remains a separate ledger-only decision reconciliation before any source or test edit.

## Decision Record And Current Evidence

- Decision: **Option A**, explicitly selected by the operator on 2026-07-13.
- Decision receipt: `48f259d25` (`docs: record grounding provenance review gap`). This is the decision-required receipt, not production proof or closure.
- Landed packet-amendment candidate: `49b2bbd7f` (`docs: authorize typed grounding provenance`). It is pending fresh review and is neither review-clean authority nor implementation proof.
- Review-clean witness: `6409ae072` (`test: preserve cross-checkpoint grounding gaps`) changes only `crates/agent-drift-analyzer/tests/truth_grounding_gap.rs` and independently received fresh `REVIEW CLEAN`.
- Preserved incomplete series: production `52c9ab296` plus receipt `73132aead` received fresh `REVIEW FINDINGS` P1. The later receipt `48f259d25` records that disposition and keeps Task 3A unchecked.
- Historical entry gates remain complete: packet docs `03754a2de` and ledger reconciliation `a5380c04e` each received fresh built-in `default` `REVIEW CLEAN` for the earlier scorer-only boundary. They do not authorize this expanded Option-A boundary; this amendment must pass its own commit-and-review gate.

No fresh `REVIEW CLEAN` verdict is claimed for `49b2bbd7f` or its packet-only fix series, and no implementation command result or closure proof is claimed by this amendment.

## Objective And Preserved Controls

Close only `CTX-R6-12` and its review-clean cross-checkpoint provenance witnesses without weakening their committed expectations:

1. **Original `CTX-R6-12` remains exact.** `truth_grounding_gap_flags_truth_path_action_before_read` must remain `80 / High / Active`, flagged, with the declared truth-artifact authority evidence and `command family: apply_patch` action evidence when a truth-path-touching action occurs before any qualifying read.
2. **Historical-only is not grounding.** `truth_grounding_gap_reactivates_truth_path_action_after_historical_only_recovery` must become `80 / High / Active`, flagged, at the later same-path action. Historical score posture and historical evidence do not establish a read.
3. **A real same-path read crosses the next checkpoint boundary.** `truth_grounding_gap_carries_clean_read_to_next_checkpoint_truth_path_action` must become `0 / Medium / Cleared`, unflagged, at the next-checkpoint same-path action. This proves carry across one checkpoint boundary; it is not an expiry, TTL, freshness, or read-consumption rule.
4. **Path identity is load-bearing.** `truth_grounding_gap_does_not_ground_path_b_from_path_a_read` must assert `80 / High / Active`, flagged, for the path-B action: a qualifying read of path A never grounds path B.
5. **Removal is permanent for the carried observation.** `truth_grounding_gap_does_not_resurrect_read_after_path_redeclaration` must assert `80 / High / Active`, flagged, when a path is read, removed from an intervening task frame, later re-declared, and then acted on without a new qualifying read.
6. **Bundle sessions are isolated.** `truth_grounding_gap_does_not_carry_read_across_sessions` must assert `80 / High / Active`, flagged, for a later session's same-path action without its own qualifying read.
7. **Parent provenance is not inherited.** `truth_grounding_gap_does_not_inherit_parent_read_in_child_session` must assert `80 / High / Active`, flagged, for the child session's same-path action without its own qualifying read. Prove this only through the analyzer's existing bundle/session fixture boundary; do not add or validate R7 topology.
8. **Carry exceeds one checkpoint.** `truth_grounding_gap_carries_clean_read_across_multiple_checkpoints` must assert `0 / Medium / Cleared`, unflagged, when the same declared path is acted on more than one checkpoint after its qualifying read. This proves there is no packet-authorized checkpoint-count expiry or TTL.
9. **A prior action does not consume the read.** `truth_grounding_gap_retains_clean_read_after_prior_same_path_action` must assert `0 / Medium / Cleared`, unflagged, for a later same-path action after an earlier same-path action already used the same carried read. This proves there is no packet-authorized read-consumption rule.

At `6409ae072`, the original control passes while the two preserved cross-checkpoint controls are red; the owning family is historically recorded as `10 passed; 2 failed`. That is witness evidence, not an implementation result.

## Option-A Internal Contract

The repair may introduce only an internal provenance value threaded through the existing analyzer call chain:

`analyze_loaded_bundle` → `score_session` → `score_truth_grounding_gap`

The implementation contract is:

- Provenance is derived only from typed qualifying read observations and their event order.
- `DriftScore`, `raw_score`, `state`, `flagged`, historical score posture, evidence presence, and evidence `reason` strings are forbidden provenance sources.
- Provenance is keyed to declared truth-path identity, not to a session-wide grounded boolean.
- A carried entry may affect an action only when its matching path is still declared in the current `TaskFrame.truth_artifacts`.
- Paths no longer declared in the current task frame are removed from carried eligibility. A later re-declaration must not resurrect provenance that was dropped while out of frame.
- A same-interval read qualifies only when event order places it before the relevant action. A read observed in an earlier checkpoint of the same session is earlier by construction and may carry for that matching path.
- Provenance is initialized inside the per-session loop in `analyze_loaded_bundle`; it resets for each session and never crosses a parent/child or sibling trajectory boundary.
- History/evidence recovery remains a disposition and presentation concern only. It cannot create, restore, or infer provenance.
- A carried read remains eligible beyond one checkpoint and after a prior same-path action while the path remains declared. This packet establishes no freshness, TTL, maximum checkpoint count, expiry, or read-consumption behavior. Such semantics require separate evidence and a separate decision.

Every new helper must be private. The provenance type must also be private unless cross-module signature threading makes `pub(crate)` visibility necessary; no helper and no other new item may use `pub(crate)`. The representation must remain housed in the three bounded source files below. No public signature or exported type may change.

## Authorized Files By Atomic Batch

### Packet-amendment gate — current batch

Candidate `49b2bbd7f` has landed and is pending fresh review. Any review-fix batch remains exactly:

- this SPEC;
- this packet's PLAN;
- this packet's TASKS.

No source, test, ledger, staging, or implementation belongs in the current batch.

### Ledger-only decision reconciliation — next required batch

Exactly:

- `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md`.

Record `R6-TGG-CROSS-CHECKPOINT-PROVENANCE-01 = A`, the review-clean packet-amendment commit, the expanded bounded source/test authority, and Task 3A as the next action. Preserve the gap as active and make no proof or closure claim. Commit separately and obtain fresh built-in `default` `REVIEW CLEAN` before implementation.

### Option-A implementation/proof batch — only after both gates are review-clean

Only:

- `crates/agent-drift-analyzer/src/scoring/truth_grounding_gap.rs` — `score_truth_grounding_gap`, the provenance type (private unless cross-module signature threading requires `pub(crate)`), and necessary private helpers;
- `crates/agent-drift-analyzer/src/scoring/mod.rs` — `score_session` and necessary private threading helpers;
- `crates/agent-drift-analyzer/src/lib.rs` — `analyze_loaded_bundle` and necessary private per-session initialization/threading helpers;
- `crates/agent-drift-analyzer/tests/truth_grounding_gap.rs` — focused provenance behavior tests only;
- this TASKS — exact results and review state;
- `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md` — actual `CTX-R6-12` / named-gap evidence only.

An allowed file is not permission to make unrelated edits within it. Unrelated dirty files remain unstaged.

## Explicit Non-Goals

No public API, schema, export, replay, sentinel, or presentation change is authorized. Do not change `Checkpoint`, `DriftScore`, serialized JSON, summary/checkpoint export, inference/context extraction, command classification, another scorer, R7, successor packets, or replay fixtures. Do not use evidence reason parsing as hidden state. Do not add freshness, TTL, or read consumption absent separate evidence and a separate packet decision.

## Mandatory GitNexus Pre-Edit Gate

After both docs gates are review-clean and before any source edit, run these three exact commands individually and record each risk, direct callers, affected processes, and affected modules in TASKS:

```bash
npx gitnexus impact score_truth_grounding_gap -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact score_session -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact analyze_loaded_bundle -r 97a0-substrate --direction upstream --depth 3
```

Before editing any existing helper, run the same exact gate with that helper's actual symbol name as a separate command. The currently bounded helpers in `truth_grounding_gap.rs` therefore require these exact commands if any is edited:

```bash
npx gitnexus impact historical_truth_grounding_gap_evidence -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact Function:crates/agent-drift-analyzer/src/scoring/truth_grounding_gap.rs:dedupe_evidence -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact first_event_index -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact is_historical_truth_grounding_gap_reason -r 97a0-substrate --direction upstream --depth 3
```

Every other existing helper proposed for edit gets its own literal command line and recorded result before editing; a wildcard, file-level result, or one anchor's result cannot stand in for another symbol. A genuinely new private helper has no pre-edit indexed symbol: record that fact rather than inventing impact output, keep it inside the bounded files, and rely on the required staged `detect-changes` gate to inspect its actual graph effect. Warn and stop before editing on any HIGH or CRITICAL result.

## Exact Verification And Acceptance

Run focused controls first, then the full owning target, checkpoint family, and static gates:

```bash
cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_flags_truth_path_action_before_read -- --exact --nocapture
cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_reactivates_truth_path_action_after_historical_only_recovery -- --exact --nocapture
cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_carries_clean_read_to_next_checkpoint_truth_path_action -- --exact --nocapture
cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_does_not_ground_path_b_from_path_a_read -- --exact --nocapture
cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_does_not_resurrect_read_after_path_redeclaration -- --exact --nocapture
cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_does_not_carry_read_across_sessions -- --exact --nocapture
cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_does_not_inherit_parent_read_in_child_session -- --exact --nocapture
cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_carries_clean_read_across_multiple_checkpoints -- --exact --nocapture
cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_retains_clean_read_after_prior_same_path_action -- --exact --nocapture
cargo test -p agent-drift-analyzer --test truth_grounding_gap -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo fmt --all -- --check
cargo check -p agent-drift-analyzer
cargo clippy -p agent-drift-analyzer --all-targets -- -D warnings
```

Acceptance requires every focused control, the full `truth_grounding_gap` integration target, all matching checkpoint tests, format, check, and clippy to be green. Record exact counts and dispositions only after running them; this packet amendment claims none of those future results.

## Commit, Review, And Exit

Before every implementation or proof commit:

```bash
git add -- <intended-files-only>
npx gitnexus detect-changes --scope staged -r 97a0-substrate
git diff --cached --check
git diff --cached
```

Commit atomically and dispatch a fresh built-in `default` reviewer. Apply findings in a new bounded commit and repeat with a fresh built-in `default` reviewer until `REVIEW CLEAN`. Task 3A, Task 4, the authority transition, and successor work remain incomplete. Only a later review-clean implementation/proof series may unlock the separate authority-only transition already listed in TASKS.
