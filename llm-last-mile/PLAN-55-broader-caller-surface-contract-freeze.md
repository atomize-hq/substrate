# PLAN-55: Broader Caller-Surface Contract Freeze

Source spec: [SPEC-55-broader-caller-surface-contract-freeze.md](./SPEC-55-broader-caller-surface-contract-freeze.md)  
Source tracker note: [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)  
Source gap matrix: [AGENT_ORCHESTRATION_GAP_MATRIX.md](../AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Prior slice: [PLAN-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md](./PLAN-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md)  
Plan type: product-contract freeze above the already-landed public caller/runtime floor  
Phase: `PLAN`  
Status: landed and validated on `2026-06-11`

## Closeout outcome

Slice `55` is now the landed caller-surface contract baseline for the current tree:

1. repo truth now distinguishes config/runtime default backend selection from implicit default-agent routing,
2. operator-facing docs and regression-floor wording now agree on the narrow prompt-taking surfaces, shell-wrap-only boundaries, exact follow-up selectors, and host-rooted world-backed `start` wording without widening runtime behavior,
3. remaining deferred work stays explicit: default-agent UX, broader non-REPL targeting, public member-root or member-level selector ergonomics, later read-side/control hardening, cross-platform posture decisions, and Family-2/global-ingress follow-ons.

## Objective

Turn the already-discussed caller-surface posture into one explicit, implementation-ready repo contract without widening runtime behavior.

This slice is complete only when:

1. the repo clearly distinguishes config/runtime default backend selection from implicit caller-surface default-agent routing,
2. the repo clearly freezes the sanctioned prompt-taking surfaces to the already-landed explicit REPL and `substrate agent` surfaces,
3. shell command mode remains frozen to shell command mode,
4. public follow-up remains exact-session plus exact-backend only,
5. world-backed start wording is reconciled to “host-rooted world-backed start exists; standalone member-root public start/continuity does not,”
6. docs/tests/help text align to that truth without reopening runtime semantics.

## Phase Gate

This plan assumes the `SPECIFY` artifact in [SPEC-55-broader-caller-surface-contract-freeze.md](./SPEC-55-broader-caller-surface-contract-freeze.md) has been reviewed and accepted before implementation starts.

## Tracker Update Rule

The running repo-truth sources for this slice remain:

- [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](../AGENT_ORCHESTRATION_GAP_MATRIX.md)

During implementation:

1. use the gap matrix as the product-truth wording source for the six remaining v1 buckets,
2. use the remaining-scope note as the sequencing source so Slice `55` does not accidentally re-open host-tool parity work,
3. keep wording updates bounded to the caller-surface freeze rather than broad repo cleanup.

## Locked Decisions

Slice `55` should implement the following decisions exactly:

1. **Default backend vs default-agent routing**
   - config/runtime default backend selection remains real,
   - implicit caller-surface default-agent prompt routing remains disallowed.
2. **Non-REPL prompt-taking scope**
   - keep the sanctioned non-REPL prompt-taking surface to the existing `substrate agent` namespace,
   - defer broader ambient or fuzzy targeting.
3. **Shell mode boundary**
   - keep `substrate -c`, `--command`, and piped stdin as shell-wrap-only.
4. **Public follow-up selector contract**
   - keep exact `(--session <orchestration_session_id>, --backend <backend_id>)`,
   - reject noncanonical public selectors and fuzzy inference.
5. **World-backed start wording**
   - host-rooted world-backed start exists,
   - standalone member-root public start/continuity does not.
6. **Deferred follow-ons**
   - keep default-agent UX, broader non-REPL targeting, public member-root lifecycle, and richer write-side selector ergonomics out of scope.

## Major Components And Dependencies

1. **caller-surface decision ledger**
   - `llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md`
   - the decision ledger is the authority; implementation should align to it rather than ad hoc doc phrasing.

2. **repo-wide product truth docs**
   - `AGENT_ORCHESTRATION_GAP_MATRIX.md`
   - `docs/USAGE.md`
   - `llm-last-mile/README.md`
   - these files currently carry most of the user-visible or planner-visible wording drift.

3. **historical slice authorities to preserve**
   - `llm-last-mile/20-public-non-interactive-agent-caller-surface.md`
   - `llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md`
   - `llm-last-mile/PLAN-30.md`
   - these should be referenced, not contradicted.

4. **public regression floor**
   - `crates/shell/tests/agent_public_control_surface_v1.rs`
   - `crates/shell/tests/repl_world_first_routing_v1.rs`
   - these verify that the frozen contract is not just doc language.

5. **bounded CLI/help-text surfaces only if needed**
   - `crates/shell/src/execution/cli.rs`
   - `crates/shell/src/execution/agents_cmd.rs`
   - only touch if implementation discovers current user-facing text that contradicts the frozen contract.

## Plan Summary

The narrowest honest Slice `55` is:

1. freeze the six caller-surface decisions first,
2. reconcile world-backed start terminology second,
3. align repo-truth docs and regression-floor wording third,
4. finish with validation and closeout last.

This is a contract/productization slice, not a runtime-expansion slice.

## Implementation Order

### Packet 1: Freeze The Decision Ledger

Goal:

1. codify the six caller-surface decisions from `SPEC-55`,
2. explicitly distinguish default backend selection from default-agent routing,
3. freeze the allowed / disallowed / deferred boundaries for v1.

Primary touch surface:

1. `llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md`
2. `AGENT_ORCHESTRATION_GAP_MATRIX.md`
3. `llm-last-mile/README.md` only if a bounded summary alignment is needed

Why first:

1. later packets need one authoritative wording source,
2. this prevents doc updates from “improvising” caller semantics mid-slice.

Verification checkpoint:

1. the repo has one explicit decision table for default backend vs default-agent routing,
2. the slice still forbids fuzzy or implicit prompt routing.

### Packet 2: Reconcile Root/Follow-Up/World-Start Terminology

Goal:

1. align root start, follow-up, reattach, and shell-wrap terminology,
2. resolve the current wording drift between host-rooted world-backed start and “public world-root start,”
3. keep member-root public lifecycle explicitly deferred.

Primary touch surface:

1. `docs/USAGE.md`
2. `AGENT_ORCHESTRATION_GAP_MATRIX.md`
3. `llm-last-mile/20-public-non-interactive-agent-caller-surface.md`
4. `llm-last-mile/README.md`

Why second:

1. Packet `1` provides the contract vocabulary,
2. Packet `2` is the first place that vocabulary becomes operator-facing repo truth.

Verification checkpoint:

1. repo wording now consistently says host-rooted world-backed start exists,
2. repo wording no longer implies standalone member-root public start/continuity is already shipped.

### Packet 3: Align Regression Floors And Bounded Help Text

Goal:

1. keep the regression wall aligned to the frozen contract,
2. preserve shell-wrap-only behavior and exact selector behavior,
3. touch help text only if current user-visible text contradicts the contract.

Primary touch surface:

1. `crates/shell/tests/agent_public_control_surface_v1.rs`
2. `crates/shell/tests/repl_world_first_routing_v1.rs`
3. `crates/shell/src/execution/cli.rs` only if needed
4. `crates/shell/src/execution/agents_cmd.rs` only if needed

Why third:

1. docs alone are not enough if tests or help text still imply a different contract,
2. this packet keeps the slice honest without widening into runtime redesign.

Verification checkpoint:

1. shell-wrap-only tests still pass,
2. exact public selector tests still pass,
3. no test/help text suggests implicit default-agent routing or public member-root lifecycle.

### Packet 4: Validation Wall And Closeout

Goal:

1. run the bounded validation wall,
2. confirm the slice stayed contract-first,
3. close the spec/plan/tasks/docs loop without overclaiming follow-on work.

Primary touch surface:

1. `llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md`
2. `llm-last-mile/PLAN-55-broader-caller-surface-contract-freeze.md`
3. `llm-last-mile/TASKS-55.md`
4. any docs/tests touched by Packets `1`-`3`

Why last:

1. contract truth should be finalized only after docs/tests/help text agree,
2. closeout is where the slice should explicitly say what remains deferred.

Verification checkpoint:

1. docs/tests/help text all agree on the frozen contract,
2. the slice did not widen into default-agent UX, fuzzy selectors, or runtime expansion.

## Risks And Mitigations

1. **Risk: “default backend” gets conflated with implicit default-agent routing**
   - Mitigation: make the distinction explicit in every decision summary and user-facing doc touched by the slice.

2. **Risk: world-backed start wording reopens old `born_unattached` or standalone world-root debates**
   - Mitigation: treat Slice `30` as authority and normalize wording to host-rooted world-backed start vs deferred member-root lifecycle.

3. **Risk: the slice widens into broader caller ergonomics**
   - Mitigation: keep `--current`, fuzzy selectors, public member-root selectors, and broader non-REPL targeting explicitly out of scope.

4. **Risk: doc truth and test truth diverge**
   - Mitigation: use Packet `3` to align the regression floor after the wording freeze.

5. **Risk: implementation touches runtime semantics unnecessarily**
   - Mitigation: prefer docs/tests/help-text-only changes unless a current user-facing surface plainly contradicts the frozen contract.

## Parallelism Guidance

1. Packet `1` must happen first because it defines the vocabulary.
2. After Packet `1`, Packet `2` docs alignment and Packet `3` regression-floor review can be explored in parallel, but final merges should still be reconciled centrally.
3. Packet `4` must happen last.

## Success Markers

1. The repo has one stable answer for “what is a prompt-taking surface?”
2. The repo has one stable answer for “what remains ordinary shell execution?”
3. The repo has one stable answer for “what selectors are public and canonical?”
4. The repo has one stable answer for “what does world-backed start mean in public lifecycle terms?”
5. The slice can be implemented as bounded docs/tests/help-text alignment rather than as a runtime redesign.
