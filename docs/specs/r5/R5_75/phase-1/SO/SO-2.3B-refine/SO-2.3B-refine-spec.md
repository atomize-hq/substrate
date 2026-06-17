# Spec: SO-2.3B-refine Structured Objective Bridge Honesty

Status: SPECIFY artifact created on 2026-06-17 via `spec-driven-development` after reconciling
`R5.75-1`, the live structured-objective code path, and the design-arch authorities. This packet is
bounded corrective work inside `R5.75-1`, not a new phase family and not a downstream migration.

## Assumptions I'm Making

1. `SO-2.3B-refine` is the next honest packet inside `R5.75-1`, and it must land before `SO-3`
   because compatibility rendering and `comparison_key` derivation should operate on preserved,
   honest structured state rather than on a checkpoint path that still erases or overclaims it.
2. This packet remains analyzer-local by default: `crates/agent-drift-analyzer/src/checkpoint/mod.rs`,
   `crates/agent-drift-analyzer/src/context/objective.rs`, and
   `crates/agent-drift-analyzer/tests/checkpoints.rs` are the main seams.
3. The objective-acceptance harness (`SO-4`) is still absent and is explicitly out of scope for
   this packet; this packet exists partly so that `SO-4` locks corrected semantics rather than the
   current stopgap behavior.
4. Weak evidence should stay unknown. This packet succeeds by making the current sidecar more
   honest, not by making it look more complete.
5. Each corrected behavior packet should land with its own proving regression instead of deferring
   proof to a later omnibus regression packet. The final audit packet should only fill gaps and add
   combined-case coverage.
6. No TaskFrame coexistence, working-set migration, checkpoint predicate migration, progress
   comparability migration, classifier/runtime work, or new dependency belongs in this packet.

If any of these assumptions drift, update this spec before implementation.

## Objective

Land a bounded corrective packet that makes the current structured-objective sidecar honest and
preserved through the checkpoint path before compatibility/comparison work (`SO-3`) and before the
objective-acceptance harness (`SO-4`).

Primary outcomes:

1. `checkpoint_analyses(...)` no longer erases the richer `ObjectiveSummary.structured` state when
   narrowing the checkpoint objective, and the same packet proves that behavior with a focused
   regression.
2. `target` is populated only when grounded target evidence exists, while vague review/analyze/fix
   asks leave `target` unknown, and the target packets prove both the unknown and explicit-target
   behaviors directly.
3. Verification-command clauses receive grounded verification-role evidence even when they are not
   under an explicit `Verification` heading, and the verification packets prove both role discovery
   and clause-grounded extraction directly.
4. A final audit packet may add any missing combined-case regressions, but the core behavior
   packets must already carry their own proof before review.

## Tech Stack

- Language: Rust 2021
- Primary crate: `agent-drift-analyzer`
- Design authorities:
  - `docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md`
  - `docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md`
  - `docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md`
- Packet authority set:
  - `docs/specs/r5/R5_75/MAP.md`
  - `docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md`
  - `docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md`
  - `docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md`
- Primary code seams:
  - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
  - `crates/agent-drift-analyzer/src/context/objective.rs`
- Primary regression surface:
  - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## Commands

Focused packet verification:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Optional formatting/lint gates if the packet widens beyond the narrow code seam:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Useful source inspection while implementing:

```bash
rg -n "narrowed_objective_summary|ObjectiveSummary::compatibility|assemble_structured_objective|verification_commands_from_decomposition|target_kind_for_text|role_candidates_for_clause"   crates/agent-drift-analyzer/src/checkpoint/mod.rs   crates/agent-drift-analyzer/src/context/objective.rs   crates/agent-drift-analyzer/tests/checkpoints.rs
```

## Project Structure

```text
docs/specs/r5/R5_75/MAP.md
  Root R5.75 routing authority; now records that `R5.75-1` remains active and that
  `SO-2.3B-refine` precedes `SO-3` and `SO-4`.

docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
  Phase-1 semantic contract and boundary rules.

docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
  Phase-1 landing-order authority; this packet tightens the still-open SO-2/SO-3 seam.

docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
  This packet-level spec.

docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
  Packet-level implementation plan.

docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
  Packet-level implementation ledger.

docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-packet-prompts.md
  Packet-scoped orchestration prompts; each behavior packet now includes its own proof before review.

crates/agent-drift-analyzer/src/checkpoint/mod.rs
  Current checkpoint narrowing path; this is where structured objective state is currently
  overwritten by `ObjectiveSummary::compatibility(...)`.

crates/agent-drift-analyzer/src/context/objective.rs
  Current structured-objective decomposition, assembly, target derivation, unknown generation,
  and verification-command extraction logic.

crates/agent-drift-analyzer/tests/checkpoints.rs
  Focused deterministic regressions for checkpoint objective behavior; each behavior packet should
  land its own targeted proof here, while the final B4 audit only fills any remaining combined-case gaps.
```

## Code Style

Prefer preservation-first bridge logic that keeps evidence-bearing state intact instead of
defaulting to re-extraction or another broad string fallback.

Preferred `B1.1` implementation shape:

- Preserve the `ObjectiveSummary` already produced by `assemble_context(&window)` when it already
  contains `structured`.
- Do not replace that richer state with `ObjectiveSummary::compatibility(...)`.
- Do not re-run objective extraction over a different row slice unless the implementation proves
  the slice is equivalent and preserves sidecar state, `verification_commands`, `unknowns`, and
  evidence spans.
- If legacy narrowing is still needed for checkpoint display compatibility, layer only the display
  text / comparison fallback in a helper that preserves `structured`, `verification_commands`,
  `unknowns`, and evidence spans.

Simplified reduced sketch only (acceptable as an example, but not the default implementation
guidance):

```rust
fn narrowed_objective_summary(rows: &[CompactionRow]) -> Option<ObjectiveSummary> {
    let structured = extract_objective(rows);
    if structured.structured.is_some() {
        return Some(structured);
    }
    legacy_narrowed_objective_summary(rows)
}
```

Packet-specific conventions:

- Treat the bridge bug as overwrite, not extraction absence: `assemble_context(...)` already
  produces richer objective state and checkpoint narrowing must preserve it whenever that state is
  present.
- Preserve structured/evidence/unknown state when narrowing; do not collapse back to
  `ObjectiveSummary::compatibility(...)` if structured extraction already succeeded.
- Separate goal selection from target extraction; a grounded goal may exist while `target` stays
  unknown.
- Prefer helpers like `has_role(clause, ObjectiveRole::Verification)` over single top-role checks
  when verification grounding is semantically explicit.
- When `has_explicit_verification_cue(...)` is true, the clause must receive an
  `ObjectiveRole::Verification` role candidate with evidence; it must not merely suppress
  `ObjectiveRole::Goal`.
- `verification_commands` should be collected from any clause carrying a verification role
  candidate, not only from clauses where `Verification` is the top role.
- Add focused regressions for every corrected heuristic in the same behavior packet so the packet
  does not depend on prose-only intent during review.
- Use the final audit packet only to cover missing combined cases or proof gaps that the earlier
  behavior packets did not already lock down.

Explicit target anchor contract for this packet:

- Accepted explicit target anchors:
  - repo-relative file or directory paths
  - crate/package names with crate/package cues
  - spec/design/doc names or markdown/doc paths
  - test/verifier target names when the task is about the test/verifier itself
  - instruction surfaces such as `AGENTS.md`, `<skill>`, `Available skills`, or profile/plugin
    instructions
  - workspace refs such as `@shared-cab-app`
  - named packet/work item identifiers only when directly tied to the requested task
- Not enough by itself:
  - `this`
  - `it`
  - `the above`
  - `what landed`
  - `the current issue`
  - the entire goal sentence copied as target
- If the packet cannot ground `target` to one of the accepted anchors above, conservative
  omission/unknown is the correct outcome and `ObjectiveUnknown` should remain present.

## Testing Strategy

This packet uses four validation layers.

1. **Behavior-packet checkpoint regressions**
   - extend `crates/agent-drift-analyzer/tests/checkpoints.rs`
   - `B1.1` proves structured state survives checkpoint narrowing
   - `B2.1` proves vague targets remain unknown while at least one obvious explicit target case
     still survives
   - `B2.2` proves the broader explicit-target preservation matrix across file/directory,
     instruction-surface, spec/doc, test/verifier, crate/package, workspace-ref, and directly tied
     packet/work-item targets
   - `B3.1` proves bullet-only or unheaded verifier clauses receive a verification role candidate
     with evidence, not just suppressed goal behavior
   - `B3.2` proves clause-grounded extraction is preferred when grounded verifier clauses exist,
     and that `verification_commands` are collected from any clause carrying that verification role
     candidate

2. **Final coverage audit / combined-case regressions**
   - `B4.1` audits the remaining proof surface after the behavior packets land
   - add any missing combined-case regressions only if the earlier packets did not already provide
     sufficient proof

3. **Focused analyzer wall**
   - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
   - this is the main gate for the packet because all changes are analyzer-local and checkpoint-facing

4. **Full analyzer regression run**
   - `cargo test -p agent-drift-analyzer -- --nocapture`
   - confirms the packet did not destabilize unrelated analyzer behavior

`objective_acceptance` remains out of scope for this packet; its job is to validate the corrected
semantics in `SO-4`, not to be partially bootstrapped here.

## Boundaries

- **Always do:**
  - verify any prior packet tasks named as prerequisites are already landed in live repo state and tests before editing; if one is missing, stop and report it instead of compensating inside the later packet
  - preserve structured/evidence/unknown state when narrowing if structured extraction already exists
  - leave weak target evidence unknown rather than guessing
  - keep the packet analyzer-local and reviewable
  - land each corrected behavior packet with its own targeted regression before review

- **Ask first:**
  - touching `context/working_set.rs`, `checkpoint/progress.rs`, or `TaskFrame`
  - adding `objective_acceptance` harness files or fixture families
  - changing public schema/export contracts beyond additive packet-local needs
  - widening into `comparison_key` derivation or compatibility rendering policy beyond what is
    required to keep this packet honest

- **Never do:**
  - fabricate `target` from a weak goal clause just to avoid `unknowns`
  - treat the current `comparison_key == text` stopgap as downstream-ready
  - let compatibility-only narrowing erase richer structured state
  - defer the core proof for a behavior change to a later omnibus packet when the behavior packet
    itself is the thing being committed and reviewed
  - widen into classifier/runtime work, Phase 2/3 migration, or unrelated objective refactors

## Success Criteria

This packet is done only when all of the following are true:

1. `checkpoint_analyses(...)` no longer overwrites a richer structured objective with a
   compatibility-only summary; it preserves the richer `assemble_context(&window).objective` when
   `structured` is already present, and the same packet proves the behavior with a focused
   regression.
2. `target` is emitted only when explicit target evidence exists; vague review/analyze/fix prompts
   keep `target == None` and record `ObjectiveUnknown { field_name: "target", ... }`; `B2.1`
   proves that unknown behavior without nuking all explicit-target extraction by preserving at
   least one obvious explicit target case. Accepted explicit target evidence is limited to
   repo-relative file/directory paths, crate/package names with cues, spec/design/doc names or
   doc paths, test/verifier targets when the task is about the test/verifier itself, instruction
   surfaces, workspace refs, and directly tied packet/work-item identifiers; pronoun-only or
   copied-whole-goal fallbacks are not enough by themselves. `B2.2` proves the broader
   explicit-target preservation matrix.
3. Verification-command clauses produce verification-role evidence spans even when there is no
   dedicated `Verification` heading. When `has_explicit_verification_cue(...)` is true, the clause
   receives an `ObjectiveRole::Verification` role candidate with evidence rather than merely
   suppressing `ObjectiveRole::Goal`; clause-grounded extraction is preferred when grounded
   verifier clauses exist; and `verification_commands` are collected from any clause carrying a
   verification role candidate, not only from clauses where `Verification` is the top role.
4. `B4.1` closes any remaining combined-case proof gaps rather than carrying the core proof load for
   the earlier behavior packets.
5. `cargo test -p agent-drift-analyzer checkpoints -- --nocapture` and
   `cargo test -p agent-drift-analyzer -- --nocapture` are green.
6. The packet stays bounded enough that `SO-3` remains the next packet after closeout.

## Open Questions

1. If a display-only legacy narrowing is still needed, should it layer only `text` /
   `comparison_key` while leaving `structured`, `verification_commands`, `unknowns`, and evidence
   untouched? The preferred direction for this packet is yes; only a concrete regression should
   justify anything broader.
2. If a narrowed compatibility string and the richer structured evidence disagree, should the packet
   always prefer structured state and force a regression to explain the mismatch? This packet
   assumes yes.
3. If a clause meaningfully supports both `Goal` and `Verification`, should the implementation emit
   multiple evidence spans or just ensure `Verification` is discoverable without top-role loss?
   This packet assumes at least verification discoverability is mandatory; multi-span emission is
   acceptable if it stays additive and deterministic.
