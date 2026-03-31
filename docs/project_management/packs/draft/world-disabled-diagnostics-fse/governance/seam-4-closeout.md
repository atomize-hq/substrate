---
seam_id: SEAM-4
status: landed
closeout_version: v0
seam_exit_gate:
  source_ref: threaded-seams/seam-4-cross-platform-conformance/slice-3-seam-exit-gate.md
  status: passed
  promotion_readiness: blocked
basis:
  currentness: current
  upstream_closeouts:
    - seam-1-closeout.md
    - seam-2-closeout.md
    - seam-3-closeout.md
  required_threads:
    - THR-04
    - THR-05
  stale_triggers:
    - any landed delta in SEAM-1 through SEAM-3 that changes disabled-mode status, omission, or exact-copy contracts
    - platform-specific socket, pipe, or path assumptions change on Linux/macOS/Windows before conformance evidence is captured
    - scripts/mac/smoke.sh, scripts/windows/wsl-smoke.ps1, or the Linux world/health doctor workflow drift without synchronized revalidation
    - future packs touch health.rs, shim_doctor/report.rs, shim_doctor/output.rs, or docs/USAGE.md before closeout
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations:
  - REM-001
---

# Closeout - SEAM-4 Cross-platform conformance

This is the final closeout record for SEAM-4. It records landed evidence and the remaining blocker for promotion readiness.

## Cross-platform proof matrix

This matrix is the operator-playbook view that `S2` and `S3` must consume. It is intentionally evidence-first and does not claim that the runs below have already been completed.

### Contract mapping

- `C-02`: disabled shim doctor status and omission rules.
- `C-03`: disabled shim doctor exact-copy / disabled-text contract.
- `C-04`: disabled shim health summary posture and guidance suppression.
- `C-05`: disabled health machine-readable and text alignment, including docs alignment.

### Required proof paths

| Platform | Disabled mode proof | Enabled-but-broken proof | Invalid-config fail-fast proof | Contract focus | Repo-native surface |
| --- | --- | --- | --- | --- | --- |
| Linux | `substrate shim doctor` and `substrate health` evidence showing disabled status, disabled omission rules, and no world probe dependence | `substrate shim doctor` / `substrate health` evidence showing failures remain visible when world is enabled but broken | `substrate shim doctor` / `substrate health` evidence showing config resolution fails fast with exit 2 and no report | `C-02`, `C-03`, `C-04`, `C-05` | `crates/shell/tests/shim_doctor.rs`, `crates/shell/tests/shim_health.rs`, Linux doctor/health commands |
| macOS | `scripts/mac/smoke.sh` evidence showing disabled posture and exact disabled copy without backend probing | `scripts/mac/smoke.sh` evidence showing enabled-but-broken failure visibility remains intact | `scripts/mac/smoke.sh` evidence showing invalid-config exits fail fast instead of producing a report | `C-02`, `C-03`, `C-04`, `C-05` | `scripts/mac/smoke.sh` |
| Windows | `scripts/windows/wsl-smoke.ps1` evidence showing disabled posture and exact disabled copy without backend probing | `scripts/windows/wsl-smoke.ps1` evidence showing enabled-but-broken failure visibility remains intact | `scripts/windows/wsl-smoke.ps1` evidence showing invalid-config exits fail fast instead of producing a report | `C-02`, `C-03`, `C-04`, `C-05` | `scripts/windows/wsl-smoke.ps1` |

### Operator-playbook assertions

- The matrix is read as a contract-to-proof map, not as a suggested test wish list.
- Linux remains the regression anchor for the detailed `shim_doctor.rs` and `shim_health.rs` assertions.
- macOS and Windows smoke runs must prove the same disabled, enabled-but-broken, and invalid-config behaviors using repo-native scripts only.
- Disabled-mode evidence must distinguish between exact disabled copy, omission rules, and fail-fast config behavior rather than collapsing them into generic success or failure.
- S2 must revalidate shared-file overlap before closeout by checking `health.rs`, `shim_doctor/report.rs`, `shim_doctor/output.rs`, and `docs/USAGE.md`.
- S3 must consume the evidence bundle as downstream closeout truth only after S2 has recorded the platform results and the shared-file revalidation statement.

## Seam-exit gate record

- **Source artifact**: `threaded-seams/seam-4-cross-platform-conformance/slice-<final>-seam-exit-gate.md`
- **Cross-platform evidence matrix**: retained as the required proof map; `S2` has Linux anchor evidence and the repo-native smoke wrappers now carry the disabled-diagnostics conformance mode, but native Windows execution is still unproven in this environment.
- **Landed evidence**:
  - [`cargo test -p shell --test shim_doctor -- --nocapture`](/home/spenser/__Active_code/substrate/crates/shell/tests/shim_doctor.rs) passed locally on Linux (`14/14` tests green).
  - [`cargo test -p shell --test shim_health -- --nocapture`](/home/spenser/__Active_code/substrate/crates/shell/tests/shim_health.rs) passed locally on Linux (`6/6` tests green).
  - [`scripts/mac/smoke.sh`](/home/spenser/__Active_code/substrate/scripts/mac/smoke.sh) now exposes `--world-disabled-diagnostics`; `scripts/mac/smoke.sh --world-disabled-diagnostics` passed locally on Linux as a host-neutral contract check.
  - [`scripts/windows/wsl-smoke.ps1`](/home/spenser/__Active_code/substrate/scripts/windows/wsl-smoke.ps1) now exposes `-WorldDisabledDiagnostics`, but native Windows execution remains blocked in this environment because neither `pwsh` nor `powershell` is installed.
  - Manual Linux proof commands captured the current contract shape:
    - `SUBSTRATE_OVERRIDE_WORLD=disabled target/debug/substrate shim doctor` prints `World backend: disabled` and `World deps: skipped (world disabled)`.
    - `SUBSTRATE_WORLD_SOCKET=<missing> target/debug/substrate --world shim doctor` prints `World backend: needs attention` plus `Details:` / `Applied:` visibility; JSON carries `world.details` and `world_deps.report`.
    - `SUBSTRATE_WORLD_SOCKET=<missing> target/debug/substrate --world health` prints `World backend: needs attention`, `World deps: unavailable`, `Overall status: attention required`, and the world-backend-failure bullet; JSON carries `summary.world_deps_error` and omits a disabled-path `summary.world_error`.
  - Shared-file revalidation statement: repo evidence shows [`crates/shell/src/builtins/health.rs`](/home/spenser/__Active_code/substrate/crates/shell/src/builtins/health.rs), [`crates/shell/src/builtins/shim_doctor/report.rs`](/home/spenser/__Active_code/substrate/crates/shell/src/builtins/shim_doctor/report.rs), [`crates/shell/src/builtins/shim_doctor/output.rs`](/home/spenser/__Active_code/substrate/crates/shell/src/builtins/shim_doctor/output.rs), and [`docs/USAGE.md`](/home/spenser/__Active_code/substrate/docs/USAGE.md) still expose the published disabled/skipped contract surfaces, and S2 did not need to edit those files because the proof-surface alignment work was confined to the smoke wrappers and this closeout.
- **Contracts published or changed**: none or minimal
- **Threads published / advanced**: `THR-04`, `THR-05`
- **Review-surface delta**:
  - the macOS smoke wrapper now has a dedicated disabled-diagnostics conformance mode keyed by `SUBSTRATE_SMOKE_SLICE_ID=WDD0|WDD1|WDD2`
  - the Windows smoke wrapper now has a dedicated disabled-diagnostics conformance mode behind `-WorldDisabledDiagnostics`
  - both wrappers now track the actual current broken-path JSON shape: `shim doctor` exposes `details` / `report`, and `health` carries `summary.world_deps_error` without inventing a `summary.world_error`
- **Planned-vs-landed delta**:
  - the planned cross-platform proof shape held, but Windows-native execution is still blocked here by missing PowerShell tooling
  - the disabled-text and omission proof remained intact; the enabled-broken branch required adjusting the smoke wrappers to the live `details`/`report` contract
- **Downstream stale triggers raised**:
  - any future drift in `crates/shell/tests/shim_doctor.rs`, `crates/shell/tests/shim_health.rs`, or the root smoke wrappers that stops proving the current disabled and enabled-broken contract shape
  - any future change to the human broken-path summary wording in `shim_doctor/output.rs` or `health.rs` that invalidates the updated smoke assertions
- **Remediation disposition**:
  - `REM-001` is open and records the native Windows proof gap that still blocks promotion readiness.
- **Promotion blockers**:
  - native Windows proof has not been executed in this environment; see `REM-001`.
- **Promotion readiness**: blocked

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**:
  - `REM-001`
- **Carried-forward remediations**:
  - `REM-001`
