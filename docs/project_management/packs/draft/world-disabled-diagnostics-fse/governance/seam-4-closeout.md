---
seam_id: SEAM-4
status: landed
closeout_version: v0
seam_exit_gate:
  source_ref: threaded-seams/seam-4-cross-platform-conformance/slice-<final>-seam-exit-gate.md
  status: pending
  promotion_readiness: blocked
basis:
  currentness: stale
  upstream_closeouts: []
  required_threads:
    - THR-04
    - THR-05
  stale_triggers: []
gates:
  post_exec:
    landing: pending
    closeout: pending
open_remediations: []
---

# Closeout - SEAM-4 Cross-platform conformance

This is a post-exec scaffold. Do not treat it as landed evidence until the seam-local exit slice exists and the fields below are populated from real landed behavior.

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
- **Cross-platform evidence matrix**: pending execution in `S2`; this scaffold defines the required proof paths and operator assertions.
- **Landed evidence**:
- **Contracts published or changed**: none or minimal
- **Threads published / advanced**: `THR-04`, `THR-05`
- **Review-surface delta**:
- **Planned-vs-landed delta**:
- **Downstream stale triggers raised**:
- **Remediation disposition**:
- **Promotion blockers**:
- **Promotion readiness**: blocked

## Post-exec gate disposition

- **Landing gate**: pending
- **Closeout gate**: pending
- **Unresolved remediations**:
- **Carried-forward remediations**:
