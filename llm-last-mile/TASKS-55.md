# TASKS-55: Broader Caller-Surface Contract Freeze

Source spec: [SPEC-55-broader-caller-surface-contract-freeze.md](./SPEC-55-broader-caller-surface-contract-freeze.md)  
Source plan: [PLAN-55-broader-caller-surface-contract-freeze.md](./PLAN-55-broader-caller-surface-contract-freeze.md)  
Execution model: four sequential implementation packets  
Phase: `TASKS`  
Status: proposed on `2026-06-11`

## Current tree note

1. Slice `54` landed selected-host runtime-family host-tool parity and is no longer the active next seam.
2. The remaining immediate caller-surface gap is contract/product wording, not missing prompt-taking runtime foundations.
3. Slice `55` should freeze caller-surface semantics without widening into default-agent UX, fuzzy selector ergonomics, or public member-root lifecycle.

## Task List

- [ ] Task 55.1: Freeze the caller-surface decision ledger in repo-facing docs
  - Acceptance:
    - the repo explicitly distinguishes config/runtime default backend selection from implicit default-agent routing
    - the repo explicitly lists the allowed prompt-taking surfaces, shell-only surfaces, exact selector rules, world-start wording, and deferred follow-ons
    - no updated doc implies fuzzy or implicit prompt routing
  - Verify:
    - `rg -n "default-agent routing|default backend|shell wrap|world-root start|standalone member-root" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile`
  - Files:
    - `llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md`
    - `AGENT_ORCHESTRATION_GAP_MATRIX.md`
    - `llm-last-mile/README.md` (only if a bounded summary update is needed)

- [ ] Task 55.2: Reconcile operator-facing wording for prompt-taking vs shell-wrap surfaces
  - Acceptance:
    - `docs/USAGE.md` clearly says `substrate -c`, `--command`, and piped stdin remain shell-wrap-only
    - `docs/USAGE.md` clearly says sanctioned non-REPL prompt-taking remains the existing `substrate agent` namespace
    - no touched doc implies that shell surfaces are ambient agent-prompt aliases
  - Verify:
    - `rg -n "substrate -c|--command|piped stdin|shell-wrap|shell execution surfaces rather than agent-prompt aliases" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md`
  - Files:
    - `docs/USAGE.md`
    - `llm-last-mile/20-public-non-interactive-agent-caller-surface.md` (only if bounded wording cleanup is needed)

- [ ] Task 55.3: Reconcile host-rooted world-backed start wording against deferred member-root lifecycle
  - Acceptance:
    - touched docs consistently say host-rooted world-backed start exists
    - touched docs consistently say standalone member-root public start/continuity does not
    - touched docs do not conflate world-backed host-rooted start with public member-root lifecycle
    - the slice does not reopen older `born_unattached` debates as a new happy-path contract
  - Verify:
    - `rg -n "host-rooted world|world-backed start|public world-root start|standalone member-root|born_unattached" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile`
  - Files:
    - `AGENT_ORCHESTRATION_GAP_MATRIX.md`
    - `docs/USAGE.md`
    - `llm-last-mile/README.md`
    - `llm-last-mile/20-public-non-interactive-agent-caller-surface.md` (only if needed)

- [ ] Task 55.4: Preserve the exact public follow-up selector contract and shell-wrap regression floor
  - Acceptance:
    - public follow-up remains exact `(--session <orchestration_session_id>, --backend <backend_id>)`
    - noncanonical public selectors remain rejected
    - shell-wrap regression coverage still proves `substrate -c` is not an agent prompt surface
    - no touched test/help text implies implicit backend inference or public member-root selectors
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Files:
    - `crates/shell/tests/agent_public_control_surface_v1.rs`
    - `crates/shell/tests/repl_world_first_routing_v1.rs`
    - `crates/shell/src/execution/cli.rs` (only if help text contradicts the frozen contract)
    - `crates/shell/src/execution/agents_cmd.rs` (only if help text/status wording contradicts the frozen contract)

- [ ] Task 55.5: Close the Slice 55 validation wall and document what remains deferred
  - Acceptance:
    - fmt/clippy and targeted caller-surface regression tests are green for any touched Rust files
    - the final spec/plan/tasks/docs wording is internally consistent
    - Slice `55` states clearly that default-agent UX, broader non-REPL targeting, public member-root lifecycle, and richer write-side selector ergonomics remain deferred follow-ons
    - the slice remains a contract-freeze/productization slice rather than a runtime redesign
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Files:
    - docs/tests/help-text files touched by Tasks `55.1` through `55.4`
    - `llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md`
    - `llm-last-mile/PLAN-55-broader-caller-surface-contract-freeze.md`
    - `llm-last-mile/TASKS-55.md`
