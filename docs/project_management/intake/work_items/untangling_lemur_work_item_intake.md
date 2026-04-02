---
codename: untangling_lemur
created: "2026-04-02T16:06:42Z"
status: draft
depends_on: []
---

# Work Item Intake Sheet

## 1. Codename + date + status

- Codename: `untangling_lemur`
- Created: 2026-04-02T16:06:42Z
- Status: draft

## 2. Title (imperative)

Clarify and harden preexec vs builtin tracing semantics (and make pack validation unambiguous).

## 3. Why not ADR

- This is discovery and contract clarification work, not a single shipped behavior delta yet.
- Multiple viable paths exist (runtime semantics, config surface, smoke/playbook semantics). We should research and select intentionally before proposing an ADR or implementation WI.

## 4. Task definition (bounded)

Build an evidence-backed decision record (and follow-up plan) for how preexec tracing should work across modes and platforms, and how the `world_process_exec_tracing_parity` pack should validate it.

### 4.1 Problem statement (current confusion)

We have two separate “tracing” concerns that currently look similar in the planning pack, but behave differently in the codebase:

1) World process telemetry (`world_process_start` / `world_process_exit`)
   - Generated inside the world backend (Linux-backed ptrace capture) and returned to the host via world-agent.
   - Persisted into canonical host trace (`~/.substrate/trace.jsonl` or `SHIM_TRACE_LOG`) by the host shell.
   - Joinability via `parent_span == shell command_complete.span_id`.
   - Does not require `SUBSTRATE_ENABLE_PREEXEC`.

2) Preexec/builtin privacy posture (`builtin_command` / optional `builtin_command_raw`)
   - A safe-by-default contract: canonical trace MUST NOT contain raw command bodies for preexec/builtin telemetry.
   - There are (at least) two sources of `builtin_command` in practice:
     - Bash preexec hook (via `BASH_ENV` script + `trap ... DEBUG`) when enabled.
     - Substrate host “lightweight builtin” handling (cd/pwd/export/unset) when executed by Substrate itself.
   - Today, bash preexec hook enablement is not consistent across execution modes (notably `--command` wrap mode), which makes validation ambiguous.

This ambiguity caused real failures in pack Case B: a command expected to trigger preexec (`SUBSTRATE_ENABLE_PREEXEC=1 substrate --command 'echo …'`) did not emit any `builtin_command` records, even though other parts of WPEP3 (world process telemetry) were correct end-to-end.

### 4.2 Research plan (evidence to gather)

- Establish the current intended contract for `SUBSTRATE_ENABLE_PREEXEC`:
  - “Override-only / internal” env var inventory entry semantics (what modes should honor it?).
  - Which execution paths forward or scrub it, and why.
- Determine where `builtin_command` is expected to come from:
  - Preexec trap vs Rust builtin handling vs future sources.
  - Which sources should satisfy the planning pack’s “preexec safety” requirement.
- Confirm shipped user-facing behavior:
  - In interactive REPL, in `--command` wrap mode, and in `--file` script mode.
  - For Linux/macOS (Lima) where world telemetry is expected to be available.
- Bound risks and costs:
  - Volume/perf implications of enabling preexec broadly (DEBUG trap can be chatty).
  - Security posture implications of accidental raw command persistence.
  - Test determinism and platform parity implications.

Deliverable (research artifact inside this WI):
- A short “behavior matrix” table: (mode × shell × platform) → {preexec enabled?, builtin_command emitted?, world_process_* expected?}

### 4.3 Paths to choose from (options set)

Option A — “Pack validates safety posture only; source of builtin_command is irrelevant”
- Define Case B as: “when builtin/preexec telemetry is emitted, it must omit bodies in canonical trace.”
- Adjust smoke/playbook to generate a deterministic builtin command via Substrate’s builtin handling (e.g., `export FOO=1`) and assert:
  - `builtin_command.command_omitted == true`
  - no `builtin_command_raw` in canonical trace
- Keep `SUBSTRATE_ENABLE_PREEXEC` as best-effort / limited to chosen contexts.

Option B — “Pack validates the bash preexec hook specifically”
- Treat `SUBSTRATE_ENABLE_PREEXEC` as a supported diagnostic switch that should be honored in `--command` wrap mode (and/or other modes).
- Implementation would ensure:
  - If `SUBSTRATE_ENABLE_PREEXEC=1`, the child bash is configured with the preexec script, and `builtin_command` records with `preexec:true` appear.
- Case B would assert on `preexec:true` specifically (not just any builtin).

Option C — “Move preexec enablement to an explicit config/policy lever; stop treating env as the contract”
- Introduce a config key (and/or policy snapshot attribute) that controls:
  - whether preexec hook is enabled,
  - in which modes it applies (interactive only vs wrap vs script),
  - whether raw log file capture is permitted at all.
- Keep `SUBSTRATE_ENABLE_PREEXEC` as an internal override that maps onto the same decision path.
- Update docs + pack to validate the chosen lever semantics.

Option D — “Remove preexec as a ‘core contract’; demote to debug-only tooling”
- Keep canonical trace safe-by-default invariants, but do not require `builtin_command` records to exist in any mode.
- Replace Case B with a direct test of “no raw bodies” for the sources we control, without requiring preexec emission to occur.

### 4.4 Recommendation rubric (what decides between options)

At the end of this WI, we should recommend one option using explicit criteria:

- Correctness: does the validation actually test what we care about?
- Explicitness: will operators understand how to turn it on/off?
- Safety: does it preserve “safe-by-default” for canonical trace?
- Determinism: can we validate it reliably across Linux/macOS/Windows in CI?
- Cost: is the runtime/perf overhead acceptable for default-on paths?

### 4.5 “Choose X when…” examples (required)

Include multiple concrete examples in the final recommendation section, e.g.:

- Choose Option A when…
  - the product goal is “canonical trace must not leak secrets,” not “preexec is always on,” and we want deterministic CI validation.
- Choose Option B when…
  - we want `SUBSTRATE_ENABLE_PREEXEC=1` to be a reliable operator switch in wrap mode for incident debugging, and we can accept the trace volume overhead when enabled.
- Choose Option C when…
  - we need policy-governed control over preexec emission (e.g., org policy forbids raw logs; certain workspaces allow preexec for debugging).
- Choose Option D when…
  - we want to strictly minimize runtime hooks and only assert on safety invariants, not emission existence.

## 5. Done means (<= 8 outcomes)

- A single, unambiguous statement of the current behavior for:
  - world process telemetry vs preexec/builtin telemetry,
  - and whether `SUBSTRATE_ENABLE_PREEXEC` is expected to be honored in wrap mode.
- A behavior matrix for (mode × platform) covering:
  - `--command` wrap mode, `--file` script mode, and interactive REPL.
- A reviewed options set (A–D) with crisp pros/cons/risks.
- A recommendation with “Choose X when…” examples.
- A follow-up execution plan:
  - either an ADR intake (if we are changing a contract), or
  - an implementation WI (if it’s straightforward), or
  - a docs-only update if the pack was simply testing the wrong thing.
- Pack validation guidance is updated in this WI to specify what Case B should assert in the chosen direction (even if the actual edits land in a follow-up record).

## 6. Likely touch paths

- Pack validation:
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/_core.sh`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/manual_testing_playbook.md`
- Preexec implementation + env wiring:
  - `crates/shell/src/scripts/bash_preexec.rs`
  - `crates/shell/src/execution/manager.rs` (child shell env forwarding)
  - `crates/shell/src/execution/routing/dispatch/exec.rs` (mode selection)
- Trace schema documentation (if contract changes):
  - `docs/TRACE.md`
  - `docs/internals/env/inventory.md`

## 7. Dependencies (ADR/WI)

- depends_on_adrs: []
- depends_on_work_items: []
- blocks: []

## 8. Lift Summary

### Lift Vector v1

<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "model_version": 1,
  "touch": {
    "create_files": 1,
    "edit_files": 0,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": 0,
    "boundary_crossings": 1
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 0,
    "behavior_deltas": null
  },
  "qa": { "new_test_files": 0, "new_test_cases": 0 },
  "docs": { "new_docs_files": 1 },
  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },
  "risk": {
    "cross_platform": true,
    "security_sensitive": true,
    "concurrency_or_ordering": false,
    "migration_or_backfill": false,
    "unknowns_high": 2
  },
  "notes": "Discovery-only WI intake. Follow-up ADR/WI will carry the actual implementation lift."
}
```
<!-- PM_LIFT_VECTOR:END -->

### Computed outputs

```text
Lift Score (v1): 16
Estimated slices: 2
Confidence: low
Triggers:
- missing_inputs:contract.behavior_deltas
Missing inputs:
- contract.behavior_deltas
```

## 9. Open questions

- Should `SUBSTRATE_ENABLE_PREEXEC=1` be honored in `--command` wrap mode? If not, should the env var inventory/docs say so explicitly?
- Is Case B truly “preexec hook must run” or “canonical trace must be safe when builtin/preexec telemetry exists”?
- Should there be a first-class config/policy lever for preexec, rather than relying on env overrides?
- Do we want any guarantee that `builtin_command` records are emitted at all (volume vs determinism tradeoff)?
