# Spec: Broader Caller-Surface Contract Freeze

Source tracker note: [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)  
Source gap matrix: [AGENT_ORCHESTRATION_GAP_MATRIX.md](../AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Prior numbered slice: [SPEC-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md](./SPEC-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md)  
Companion inputs:
- [PLAN-20.md](./PLAN-20.md)
- [20-public-non-interactive-agent-caller-surface.md](./20-public-non-interactive-agent-caller-surface.md)
- [SPEC-30-public-world-scoped-agent-start-and-capability-flags.md](./SPEC-30-public-world-scoped-agent-start-and-capability-flags.md)
- [PLAN-30.md](./PLAN-30.md)
- [`docs/USAGE.md`](../docs/USAGE.md)
- [`crates/shell/tests/agent_public_control_surface_v1.rs`](../crates/shell/tests/agent_public_control_surface_v1.rs)
Phase: `SPECIFY`  
Status: landed and validated on `2026-06-11`

## Implementation closeout

Historical note: the assumptions and repo-truth gut-check below capture the pre-landing problem statement. Current repo truth after Slice `55` validation is:

1. the repo now freezes prompt-taking to exact REPL `::<backend_id> <prompt>` plus explicit public `substrate agent start|turn`, with `reattach|fork|stop` preserved as lifecycle controls rather than ambient prompt surfaces,
2. `substrate -c`, `--command`, and piped stdin remain shell-wrap-only, and public follow-up remains exact `(--session <orchestration_session_id>, --backend <backend_id>)`,
3. host-rooted world-backed `start` wording is reconciled across repo truth without promoting standalone member-root lifecycle, and the deferred follow-ons remain default-agent UX, broader non-REPL targeting, public member-level selectors, and richer write-side selector ergonomics.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `54` closed the selected-host runtime-family parity seam, so the next honest slice is a caller-surface contract freeze rather than another host-tool parity slice.
2. The repo already has the runtime and control-plane capability needed for the narrow public surfaces; the remaining gap is product-contract clarity, terminology alignment, and regression-floor alignment.
3. The user has now explicitly chosen the following product posture for v1:
   - no implicit default-agent routing,
   - no caller-surface reinterpretation of `substrate -c` / `--command`,
   - exact public follow-up selectors remain required,
   - broader non-REPL targeting stays deferred,
   - host-rooted world-backed start exists,
   - standalone member-root public start/continuity does not.
4. Existing repo docs contain some wording drift on world-start terminology, especially around `public world-root start`, `host-rooted world start`, and older `born_unattached` wording, and Slice `55` should reconcile that wording to the landed Slice `30` truth rather than reopen the runtime design.
5. This slice should freeze the product contract above the landed runtime and tests; it should not widen into default-agent UX, fuzzy selectors, public member-level selectors, public toolbox mutation, or Family-2 host-global ingress work.
6. If implementation discovers a current user-visible surface that truly contradicts the frozen contract, the implementation may make bounded help-text, docs, or test updates, but it should not redesign runtime semantics without a new spec.

If any of these are wrong, correct them before implementation.

## Objective

Freeze the broader caller-surface contract for v1 so the repo has one explicit, exact, fail-closed answer for:

1. what counts as a prompt-taking agent surface,
2. what remains ordinary shell execution,
3. which public selectors are canonical,
4. whether any implicit default-agent routing exists,
5. and how to describe world-backed start without implying standalone member-root public lifecycle.

## Tech Stack

- Rust workspace (`cargo`)
- `crates/shell` public CLI and integration tests
- top-level product truth docs:
  - [`AGENT_ORCHESTRATION_GAP_MATRIX.md`](../AGENT_ORCHESTRATION_GAP_MATRIX.md)
  - [`docs/USAGE.md`](../docs/USAGE.md)
  - `llm-last-mile/` slice docs and README
- existing public control-surface regression suite:
  - [`crates/shell/tests/agent_public_control_surface_v1.rs`](../crates/shell/tests/agent_public_control_surface_v1.rs)
  - [`crates/shell/tests/repl_world_first_routing_v1.rs`](../crates/shell/tests/repl_world_first_routing_v1.rs)

This slice is complete only when:

1. the repo explicitly distinguishes config/runtime default backend selection from implicit caller-surface default-agent routing,
2. the repo explicitly freezes the sanctioned non-REPL prompt-taking surface to the existing `substrate agent` namespace rather than ambient shell surfaces,
3. `substrate -c`, `--command`, and piped stdin remain frozen as shell-wrap-only surfaces,
4. public follow-up keeps exact `(--session <orchestration_session_id>, --backend <backend_id>)` targeting and rejects noncanonical selectors,
5. repo docs consistently say that host-rooted world-backed start exists while standalone member-root public start/continuity does not,
6. docs/tests/help text do not overclaim broader caller breadth or world-root lifecycle that the runtime does not actually ship.

## Current Repo-Truth Gut Check

### 1. The repo already rejects implicit caller-surface defaults

Current public docs/tests already say:

1. no default-agent routing exists,
2. REPL targeted turns require exact `::<backend_id> <prompt>`,
3. public prompt-taking follow-up requires exact `(--session, --backend)`,
4. public selector fallback to `participant_id`, `session_handle_id`, `active_session_handle_id`, or `internal.uaa_session_id` fails closed.

Repo-truth consequence:

1. Slice `55` should freeze and restate this posture,
2. Slice `55` should not reinterpret missing backend/session input as a prompt-routing convenience feature.

### 2. Config/runtime default backend truth is real, but it is not the same as default-agent routing

The gap matrix and runtime stack already allow config/inventory/runtime-family selection to resolve a backend. That is different from allowing caller-visible prompt surfaces to omit explicit targeting and rely on hidden routing.

Repo-truth consequence:

1. Slice `55` should allow config/runtime default backend selection to remain real,
2. Slice `55` should explicitly disallow caller-surface implicit prompt routing based on that resolved default.

### 3. `substrate -c` is still shell wrap mode and already has regression coverage

The current repo already documents and tests that:

1. `substrate -c` is ordinary shell execution,
2. it must not be reinterpreted as an agent prompt surface,
3. piped stdin and `--command` remain in that same shell-oriented family.

Repo-truth consequence:

1. Slice `55` should freeze this as a product rule,
2. any future one-shot non-interactive agent prompting must arrive through a new explicit surface, not by overloading `-c`.

### 4. Public prompt-taking is already explicit and exact

The repo already has:

1. explicit REPL prompt-taking via `::<backend_id> <prompt>`,
2. explicit public prompt-taking via `substrate agent start|turn`,
3. exact `(orchestration_session_id, backend_id)` follow-up targeting,
4. explicit `reattach` as lifecycle recovery rather than prompt submission.

Repo-truth consequence:

1. Slice `55` should freeze this split rather than broaden it,
2. broader non-REPL targeting remains a later deliberate product decision.

### 5. World-backed start wording still needs one contract-cleanup pass

There is wording drift today between:

1. docs that describe `substrate agent start --scope world` as a host-rooted world-backed start,
2. gap-matrix lines that still say there is no `public world-root start`,
3. older historical wording about `born_unattached` and standalone world-root continuity.

Repo-truth consequence:

1. Slice `55` must define the canonical wording,
2. the intended meaning should be:
   - host-rooted world-backed start exists,
   - standalone member-root public start/continuity does not.

## Frozen Decision Ledger

These are the decisions Slice `55` should freeze as v1 contract truth.

| Contract area | Allowed in v1 | Not allowed in v1 | Deferred follow-on |
|---|---|---|---|
| Default backend vs default-agent routing | config/runtime default backend resolution for runtime, policy, inventory, and adapter realization | implicit prompt routing from plain REPL input, `substrate -c`, `--command`, piped stdin, missing `--backend`, or latest-session heuristics | any future default-agent UX |
| Prompt-taking surfaces | exact REPL `::<backend_id> <prompt>` and explicit `substrate agent start|turn` | ambient shell aliases or fuzzy “prompt from anywhere” semantics | broader non-REPL targeting beyond the current namespace |
| Shell-wrap surfaces | `substrate -c`, `--command`, and piped stdin as shell execution only | reinterpreting shell-wrap surfaces as agent prompt entrypoints | any future one-shot explicit agent caller, if introduced deliberately |
| Public follow-up selectors | exact `--session <orchestration_session_id> --backend <backend_id>` | `participant_id`, `session_handle_id`, `active_session_handle_id`, `internal.uaa_session_id`, recency selectors, or hidden backend inference | richer write-side discovery such as a bounded `--current` |
| World-start wording | host-rooted world-backed `start` under a host orchestration session | implying standalone member-root public start/continuity already ships | direct member-root public lifecycle |
| Broader ergonomics | narrow explicit caller surfaces with fail-closed routing | hidden routing convenience or broader member selector semantics | later caller-surface ergonomics/productization work |

### Decision 1: config/runtime default backend is allowed; implicit default-agent routing is not

Allowed:

1. global/workspace/effective config may resolve a default backend for runtime, policy, inventory, and adapter realization purposes.

Disallowed:

1. plain REPL input as “message the default agent,”
2. `substrate -c` / `--command` / pipe mode as “prompt the default agent,”
3. public `turn` without `--backend` falling back to an active/default member,
4. latest-session or latest-backend write-side routing heuristics.

### Decision 2: sanctioned non-REPL prompt-taking remains the explicit `substrate agent` namespace only

Allowed:

1. `substrate agent start`
2. `substrate agent turn`
3. `substrate agent reattach` as non-prompt-taking lifecycle recovery
4. the already-landed exact REPL targeted-turn grammar

Deferred:

1. broader non-REPL targeting beyond the existing namespace,
2. ambient shell aliases for prompting,
3. fuzzy “target anything from anywhere” caller semantics.

### Decision 3: `substrate -c`, `--command`, and piped stdin remain shell-wrap-only

The v1 contract must keep:

1. shell command mode as shell command mode,
2. agent prompting as an explicit agent surface,
3. prompt-taking and shell execution as separate product concepts.

### Decision 4: public follow-up remains exact-session plus exact-backend only

The canonical public follow-up contract remains:

```text
substrate agent turn --session <orchestration_session_id> --backend <backend_id> ...
```

Rejected as public write-side selectors:

1. `participant_id`
2. `session_handle_id`
3. `active_session_handle_id`
4. `internal.uaa_session_id`
5. fuzzy recency/latest selection
6. hidden backend inference

### Decision 5: host-rooted world-backed start exists; standalone member-root public start/continuity does not

The public product meaning should be:

1. a host orchestration session may launch/use world members underneath it,
2. a world-backed `start` is still host-rooted in public lifecycle terms,
3. world members are not yet first-class public root sessions,
4. direct member-root start/resume/recovery remains deferred.

### Decision 6: broader ergonomics/productization remain separate follow-on work

Deferred follow-ons include:

1. default-agent UX,
2. broader non-REPL targeting,
3. public member-level selectors,
4. richer current-session discovery or `--current` write-side affordances,
5. standalone member-root public lifecycle,
6. any future ambient shell prompt aliases.

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

Targeted contract checks:

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

Targeted repo-truth greps:

```bash
rg -n "default-agent routing|shell wrap|world-root start|host-rooted world|standalone member-root|exact \\(orchestration_session_id, backend_id\\)" \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile
```

## Project Structure

Primary files for the contract freeze:

```text
AGENT_ORCHESTRATION_GAP_MATRIX.md
  Canonical repo-wide product truth and remaining-gap wording.

docs/USAGE.md
  User-facing CLI contract wording.

llm-last-mile/20-public-non-interactive-agent-caller-surface.md
  Historical caller-surface authority for explicit prompt-taking and exact selectors.

llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md
llm-last-mile/PLAN-30.md
  Authority for host-rooted world-backed start vs standalone world-root continuity.

llm-last-mile/README.md
  Slice-family summary wording that must stay aligned to current contract truth.

crates/shell/tests/agent_public_control_surface_v1.rs
  Public caller-surface regression floor.

crates/shell/tests/repl_world_first_routing_v1.rs
  REPL targeted-turn exactness regression floor.
```

Potential bounded implementation files only if current help text contradicts the frozen contract:

```text
crates/shell/src/execution/cli.rs
crates/shell/src/execution/agents_cmd.rs
```

## Code Style

For this slice, “style” means precise contract wording and exact examples. Prefer boring, explicit rules like:

```md
- `substrate -c` remains shell-wrap-only.
- `substrate agent turn` requires exact `--session` plus exact `--backend`.
- No implicit default-agent routing is introduced.
- Host-rooted world-backed start exists; standalone member-root public start/continuity does not.
```

Conventions:

1. use exact command examples, not paraphrases,
2. prefer “allowed / disallowed / deferred” wording for product rules,
3. distinguish read-side ergonomics from write-side selector authority,
4. avoid overloaded terms like “default” unless the subject is explicit (`default backend` vs `default-agent routing`).

## Testing Strategy

Primary validation wall:

1. public caller-surface integration coverage in `agent_public_control_surface_v1.rs`,
2. exact REPL targeting coverage in `repl_world_first_routing_v1.rs`,
3. repo-truth grep checks across gap matrix, usage docs, and `llm-last-mile/` docs.

Testing levels:

1. **Docs/contract validation**
   - verify wording does not contradict the frozen decisions,
   - verify no remaining doc overclaims ambient prompting, default-agent routing, or public member-root lifecycle.
2. **Regression tests**
   - preserve `substrate -c` shell-wrap behavior,
   - preserve exact public selector behavior,
   - preserve explicit REPL targeted-turn grammar.
3. **Bounded runtime/help-text checks**
   - only if implementation touches CLI/help text because it currently contradicts the frozen contract.

## Boundaries

- Always:
  - preserve exact-selector, exact-command semantics,
  - keep docs/tests/help text aligned to the same contract,
  - describe runtime truth as it exists rather than as future product ambition.
- Ask first:
  - adding new public prompt-taking surfaces,
  - changing `substrate -c` semantics,
  - adding public member-root lifecycle or selector UX,
  - introducing implicit default-agent routing.
- Never:
  - add fuzzy write-side routing,
  - accept noncanonical public selectors by implication,
  - describe standalone member-root lifecycle as live if the runtime does not ship it,
  - overload shell command mode into agent prompting without an explicit new contract.

## Success Criteria

1. Slice `55` records one explicit caller-surface decision ledger that matches the user-approved posture from `2026-06-11`.
2. Repo docs can clearly answer:
   - what is the default backend concept,
   - what is not default-agent routing,
   - what surfaces are prompt-taking,
   - what surfaces remain shell-only,
   - what selectors are canonical.
3. Wording about world-backed start is reconciled so the repo no longer implies both “host-rooted world-backed start exists” and “public world-root start exists” as if those were the same thing.
4. The slice stays contract-first and does not widen into runtime redesign or broader productization.
5. The spec is sufficient to drive a narrow `PLAN-55` and `TASKS-55` implementation pass.

## Open Questions

No blocking product questions remain for the contract freeze itself after the user decisions recorded on `2026-06-11`.

Non-blocking follow-on questions that remain intentionally deferred:

1. whether v2 should introduce explicit default-agent UX,
2. whether v2 should add broader non-REPL targeting,
3. whether v2 should add public member-root lifecycle or richer write-side selector ergonomics.
