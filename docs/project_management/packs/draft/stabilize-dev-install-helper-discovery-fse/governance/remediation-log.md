# Remediation Log - stabilize-dev-install-helper-discovery

## Open remediations

```yaml
remediation_id: REM-002
origin_phase: pre_exec
source_gate: revalidation
related_seam: SEAM-3
related_slice: null
related_thread: THR-02
related_contract: C-02
related_artifact: manual_testing_playbook.md
severity: material
status: open
owner_seam: SEAM-3
blocked_targets: []
summary: macOS validation surfaces can overclaim provisioning parity because helper discovery correctness does not guarantee all release-root assets are staged.
required_fix: Keep `threaded-seams/seam-3-cross-platform-proof-drift-guards/slice-1-freeze-platform-evidence-boundaries.md`, `threaded-seams/seam-3-cross-platform-proof-drift-guards/slice-2-refresh-cross-platform-proof-surfaces.md`, and `threaded-seams/seam-3-cross-platform-proof-drift-guards/slice-3-seam-exit-gate.md` explicit that macOS scope is limited to helper discovery, validation, and managed cleanup unless additional release-root assets are intentionally added, and keep Windows wording compile-parity only through seam-exit accounting.
resolution_evidence: []
```

Rules:

- Use canonical YAML blocks for remediation entries.
- Use seam ownership only. Do not emit `WS-*` owners.
- For `severity: blocking`, `blocked_targets` must not be empty.
- For `severity: material` or `follow_up`, use `blocked_targets: []` unless a concrete blocked transition also applies.

## Resolved remediations

```yaml
remediation_id: REM-001
origin_phase: pre_exec
source_gate: review
related_seam: SEAM-1
related_slice: null
related_thread: THR-02
related_contract: C-01
related_artifact: crates/shell/src/builtins/world_enable/runner/paths.rs
severity: material
status: resolved
owner_seam: SEAM-1
blocked_targets: []
summary: Helper-missing remediation text now matches staged-prefix dev-install reality when all helper candidates are absent.
required_fix: Resolved by the landed helper-guidance and prefix-bundle fallback fixes plus the passing helper-resolution and world_enable test runs.
resolution_evidence:
  - `b5130df2` updates `crates/shell/src/builtins/world_enable/runner/paths.rs` so the fail-closed helper-missing branch points operators at rerunning `dev-install-substrate.sh --prefix <home>` and the staged prefix helper path under `$SUBSTRATE_HOME/scripts/substrate/world-enable.sh`.
  - `b5130df2` adds `locate_helper_script_reports_staged_prefix_guidance_when_missing` in `crates/shell/src/builtins/world_enable/runner/paths.rs` and `world_enable_reports_staged_prefix_guidance_when_helper_missing` in `crates/shell/tests/world_enable.rs` to lock the wording through both the unit and CLI paths.
  - `f5b9e050` updates `crates/shell/src/builtins/world_enable/runner.rs` and `crates/shell/src/builtins/world_enable/runner/paths.rs` so the prefix helper remains usable even when the inferred version directory cannot be resolved from `$SUBSTRATE_HOME/bin/substrate`.
  - `f5b9e050` adds `locate_helper_script_uses_prefix_bundle_without_version_dir` and `world_enable_uses_prefix_runtime_bundle_when_version_binary_is_missing` to lock the post-`cargo clean` helper path through both the unit and CLI surfaces.
  - `cargo test -p shell locate_helper_script -- --nocapture` passed.
  - `cargo test -p shell world_enable -- --nocapture` passed.
```

```yaml
remediation_id: REM-003
origin_phase: pre_exec
source_gate: revalidation
related_seam: SEAM-1
related_slice: null
related_thread: THR-01
related_contract: C-02
related_artifact: scripts/substrate/dev-install-substrate.sh
severity: follow_up
status: resolved
owner_seam: SEAM-1
blocked_targets: []
summary: ADR-0035 shares install-script and helper-script surfaces that can stale the extracted basis before SEAM-1 promotes.
required_fix: Revalidate SEAM-1 and downstream seam bases against any ADR-0035 changes touching shared script surfaces before promotion.
resolution_evidence:
  - 2026-03-30 pre-exec revalidation confirmed the current `scripts/substrate/dev-install-substrate.sh` runtime-bundle surface still matches the SEAM-1 owned contract.
  - 2026-03-30 pre-exec revalidation confirmed `crates/shell/src/builtins/world_enable/runner/paths.rs` still preserves the planned helper-order contract alongside the current `crates/shell/tests/world_enable.rs` coverage.
  - ADR-0035 remains a future stale trigger only if shared install-script or helper-script surfaces change again.
```
