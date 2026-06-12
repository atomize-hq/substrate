# Spec: Agent Drift Analyzer Session Progress R5.75-0

Status: landed/closed on 2026-06-12. Original spec created from
`docs/specs/r5/R5_75/MAP.md` for the first sequential pre-`R6` landing only; see the packet
plan/tasks closeout notes for the preserved verification record.

## Assumptions I'm Making

1. `R5.75-0` is a docs/authority reconciliation packet, not an analyzer-semantic code-change
   packet.
2. The new `R5.75` map is now the authority for adopted pre-`R6` required fixes, while the older
   `R5.5` docs must be reconciled so they stop presenting already-landed work as still-open
   implementation debt.
3. The primary authority docs to update are
   `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md`,
   `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md`, and
   `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`; if a matching root-order doc still names
   the next family, it should be kept in sync as part of this packet.
4. This packet should not rewrite historical facts about what `R5.5` originally planned; it should
   add honest landed-status and remaining-scope language that reflects current repo truth.
5. `R6` must remain closed until `R5.75` is complete, so no authority doc touched by this packet
   should present `R6` as current or immediately open.

If any of these assumptions drift, update this spec before implementation.

## Objective

Reconcile the stale `R5.5` planning and landing-order docs so the repo’s written authority matches
what actually landed, what remains, and what must happen before `R6`.

Primary users:

1. maintainers deciding what packet is truly next,
2. future agents reading the docs to avoid reopening landed `R5.5` work,
3. reviewers checking whether the code/docs state is honestly represented,
4. the later `R5.75-1` through `R5.75-5` packets, which need a clean authority base.

`R5.75-0` succeeds when:

1. landed `R5.5` work is clearly called out as landed/history instead of open implementation work,
2. remaining pre-`R6` work is explicitly redirected into `R5.75`,
3. root landing-order authority names `R5.75` as the active pre-`R6` gate,
4. no touched authority doc still implies `R6` is ready to open now,
5. the docs remain internally consistent after manual review and baseline analyzer validation.

## Tech Stack

- Documentation format: Markdown
- Primary codebase under validation: Rust workspace
- Primary crate for sanity validation: `agent-drift-analyzer`
- Authority stack touched by this packet:
  - `docs/specs/r5/R5_75/MAP.md`
  - `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md`
  - `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md`
  - `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
  - any matching root-order doc that also declares the next packet family

No new code dependency, fixture family, or schema change is expected in this packet.

## Commands

Authority audit commands:

```bash
rg -n "R5\\.5|R5\\.75|R6" \
  docs/specs/r5/R5_75/MAP.md \
  docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md \
  docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md \
  HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md \
  docs/specs/hybrid-drift-sentinel-implementation-order.md
```

Baseline analyzer validation:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

Optional control smoke using the existing Hybrid Drift manual path:

```bash
export CODEX_HOME="$HOME/.codex"
export SESSION_ID="019eb430-6f9a-7a03-9a63-cb451b654795"
export SMOKE_ROOT="target/r5_75-smoke/r5_75-0/$SESSION_ID"
export COMPACTOR_OUT="$SMOKE_ROOT/compactor"
export ANALYZER_OUT="$SMOKE_ROOT/analyzer"

rm -rf "$SMOKE_ROOT"

cargo run -p agent-session-compactor -- \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --output-dir "$COMPACTOR_OUT"

cargo run -p agent-drift-analyzer -- \
  --input-dir "$COMPACTOR_OUT" \
  --output-dir "$ANALYZER_OUT"

cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir "$ANALYZER_OUT"
```

## Project Structure

```text
docs/specs/r5/R5_75/MAP.md
  The packet-family map and sequence authority for R5.75.

docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-0-spec.md
  This packet’s spec authority.

docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-0-plan.md
  Technical landing plan for the docs/authority reconciliation.

docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-0-tasks.md
  Discrete implementation checklist for this packet only.

docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md
  Main R5.5 planning authority preserved as the landed closeout record for that packet family.

docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md
  Task ledger preserved as the landed closeout checklist for the completed R5.5 packet family.

HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md
  Root landing-order authority that must stay synchronized with the active packet family.

docs/specs/hybrid-drift-sentinel-implementation-order.md
  Companion packet-order authority touched and audited in this packet for final R5.75/R6 routing.
```

## Code Style

This packet is docs-only, so the style requirement is about durable authority writing: explicit
status, clear historical separation, and precise remaining-scope language.

```markdown
## Landed Status

Landed in the current crate snapshot:

- parent-visible finalization hygiene
- delegation limiting counter-evidence visibility
- diagnostics/task-doc cleanup

Remaining pre-R6 work moved to `R5.75`:

- `R5.75-1` objective condensation / target extraction
- `R5.75-2` sparse readable fail-open
```

Conventions:

- Preserve historical facts; do not rewrite old packet intent as if it never existed.
- Prefer “landed”, “remaining”, and “deferred” labels over vague “done enough” wording.
- Name the active next family exactly once in each authority section instead of scattering
  conflicting next-step language.
- Keep `R6` gate wording explicit and conservative.

## Testing Strategy

This packet uses three validation layers:

1. **Manual authority review**
   - confirm touched docs agree on what landed and what remains
   - confirm `R5.75` is the current pre-`R6` family
   - confirm `R6` is not prematurely opened
   - confirm `docs/specs/hybrid-drift-sentinel-implementation-order.md` matches the final
     `R5.75`/`R6` routing because it was touched in this packet
2. **Baseline analyzer sanity**
   - `cargo test -p agent-drift-analyzer -- --nocapture`
   - proves doc-only edits did not accidentally rely on broken current code/test state
3. **Optional native control smoke**
   - rerun `019eb430-6f9a-7a03-9a63-cb451b654795`
   - use only as a confidence check that the packet stayed doc-only and did not distort the stated
     validation surface

This packet does not require new fixtures or replay-contract changes.

## Boundaries

- Always:
  - keep the packet scoped to doc/authority reconciliation
  - preserve honest history about what `R5.5` planned vs. what later landed
  - keep root landing-order docs synchronized with the new `R5.75` family
  - run baseline analyzer validation before closing the packet
- Ask first:
  - widening this packet into analyzer code changes
  - adding new fixture families or new design docs outside the packet scope
  - changing `R6` readiness criteria beyond what `R5.75/MAP.md` already locked
- Never:
  - present `R6` as open before `R5.75` is complete
  - mark still-required pre-`R6` fixes as landed just to simplify the docs
  - silently drop historical `R5.5` context that later sessions still need for auditability

## Success Criteria

1. `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md` contains an explicit
   landed-status section or equivalent honest reconciliation language.
2. `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md` no longer reads like all
   remaining `R5.5` packets are still open implementation work if they already landed.
3. `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md` names `R5.75` as the active pre-`R6`
   family.
4. Touched authority docs consistently route remaining work into the `R5.75` sequence.
5. `cargo test -p agent-drift-analyzer -- --nocapture` is green after the edits.

## Historical Resolution Notes

1. Resolved on 2026-06-12: `docs/specs/hybrid-drift-sentinel-implementation-order.md` was
   touched in this packet and is part of the preserved companion authority set, so its
   `R5.75`-before-`R6` routing must stay synchronized with the root landing-order doc.
2. Resolved on 2026-06-12: a narrower landed-vs-remaining reconciliation was sufficient for this
   packet; the authority docs preserve honest landed examples without expanding `R5.75-0` into a
   packet-by-packet historical rewrite.
