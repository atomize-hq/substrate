---
pack_id: execution-surface-parity-hardening-fse
pack_version: v1
pack_status: extracted
source_ref: docs/project_management/intake/work_items/taming_tapir_work_item_intake.md; docs/project_management/intake/work_items/taming_tapir_fact_finding.md; docs/project_management/intake/work_items/aligning_otter_work_item_intake.md; docs/project_management/intake/work_items/untangling_lemur_work_item_intake.md
execution_horizon:
  active_seam: SEAM-1
  next_seam: SEAM-2
---

# Scope Brief - Execution Surface Parity Hardening

- **Goal**:
  Make replay routing, tracing/validation semantics, and interactive REPL disconnect handling behave as one coherent execution contract instead of three drifting operator surfaces.
- **Why now**:
  The current state already shows three different classes of drift:
  security-sensitive replay routing does not consume the canonical world-network contract, the active tracing parity pack has an ambiguous Case B validation target, and macOS PTY revoke can strand a CPU-spinning async REPL that exits as if it succeeded.
- **Primary user(s) + JTBD**:
  Substrate maintainers and operators need to trust that `substrate replay`, `substrate --command`, and `substrate --async-repl` surface abnormal runtime states, routing decisions, and trace expectations deterministically enough to debug, validate, and document them without guesswork.
- **In-scope**:
  - clarify the canonical replay routing contract and remove replay-specific policy drift
  - decide and document what `SUBSTRATE_ENABLE_PREEXEC` and `builtin_command` assertions mean across execution modes and platforms
  - harden async REPL behavior on controlling-TTY revoke or disconnect, including exit code and diagnostic posture
  - reserve downstream conformance work that locks docs, tests, smoke scripts, and playbooks to the landed behavior
- **Out-of-scope**:
  - world-agent, shim, or backend transport redesign
  - a broad Reedline replacement or general REPL architecture rewrite
  - new public CLI flags or config keys unless a later ADR or follow-on work item explicitly justifies them
  - unrelated trace-schema expansion beyond what is required to make the chosen contracts explicit
- **Success criteria**:
  - replay and normal execution share the same four-case world-network routing semantics
  - the tracing parity pack has one unambiguous Case B assertion and a published behavior matrix for mode and platform differences
  - abnormal async REPL terminal loss exits promptly with a non-zero status and no orphaned busy-spin process
  - docs, tests, and smoke guidance can later be locked to one consistent post-landing story
- **Constraints**:
  - preserve safe-by-default canonical trace posture, especially no raw builtin or preexec command bodies in canonical trace
  - keep the replay fix minimal and shared-helper oriented rather than duplicating routing logic
  - keep the REPL regression proof host-only, macOS-targeted for revoke behavior, and specific to the Reedline path rather than the stdio fallback
  - treat the extractor as seam-brief planning only; no slice or subslice files are emitted here
- **External systems / dependencies**:
  - existing world backend contract expressed through effective `world.net.filter` and canonical `policy_snapshot.net_allowed`
  - Reedline and crossterm prompt behavior inside `crates/shell/src/repl/async_repl.rs`
  - the active `docs/project_management/packs/active/world_process_exec_tracing_parity/` planning pack and its manual playbook / smoke assets
  - shared exit-code taxonomy and environment-contract documentation
- **Known unknowns / risks**:
  - the cleanest boundary between docs-only clarification and code change for preexec semantics is still unresolved
  - replay routing helper extraction may expose shared ownership questions between shell and replay crates
  - macOS revoke handling may require more than error classification if the prompt worker never unwinds out of `read_line()`
  - cross-surface docs can drift again if conformance is not explicitly owned after runtime seams land
- **Assumptions**:
  - the pack uses an integration-first seam axis because the highest-risk failures come from contract drift between execution surfaces
  - `SEAM-1` is active because it publishes the ambiguous routing and validation contracts that later documentation and conformance work must trust
  - `SEAM-2` is next because it is a bounded runtime hardening seam whose docs and operator wording should consume the same normalized execution contract language
  - `SEAM-3` remains future because meaningful cross-surface lock-in only starts after the runtime contracts from `SEAM-1` and `SEAM-2` are published
