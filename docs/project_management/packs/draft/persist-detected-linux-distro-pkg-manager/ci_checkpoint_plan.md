# persist-detected-linux-distro-pkg-manager — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/impact_map.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/spec_manifest.md`
- Slice specs (planned; see `spec_manifest.md`):
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/C0/C0-spec.md`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/C1/C1-spec.md`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/C2/C2-spec.md`
- Required platforms (authoritative; from `tasks.json`):
  - Behavior smoke platforms: `linux`, `macos`, `windows`
  - CI parity platforms: `linux`, `macos`, `windows`

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (new slice added, new platform scope, new contract surface), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: update `tasks.json` `meta.checkpoint_boundaries` to list the **last slice** in each checkpoint group (this is linted once slices exist in `tasks.json`).

## Machine-readable plan (linted)

```json
{
  "version": 1,
  "defaults": {
    "min_triads_per_checkpoint": 4,
    "max_triads_per_checkpoint": 8
  },
  "checkpoints": [
    {
      "id": "CP1",
      "task_id": "CP1-ci-checkpoint",
      "slices": ["C0", "C1", "C2"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "quick"
      },
      "rationale": "Single checkpoint covering all planned slices (C0–C2). This feature is currently planned as < min_triads_per_checkpoint, so CP1 is the contract-completion seam where Linux installer persistence + the post-success presence guarantee + installer smoke assertions are validated together across platforms."
    }
  ]
}
```

## Human-readable rationale (required)

CP1 is the only checkpoint for this feature because the currently planned slice count is `<4` (C0–C2).

Boundary rationale (code-grounded):
- CP1 is a **contract completion seam**: it comes after the persistence behavior is implemented (C0/C1) and after the installer smoke assertions are extended to cover the new keys + negative cases (C2).

Stabilized surfaces (from `spec_manifest.md` intent):
- Installer contract: best-effort persistence into `$SUBSTRATE_HOME/install_state.json` (Linux-only behavior delta; macOS/Windows must not gain `host_state.platform.*` fields).
- Install-state schema extension: additive `host_state.platform.os_release.*` and `host_state.platform.pkg_manager.*` keys (schema_version remains `1`).
- Failure posture + redaction: unreadable `/etc/os-release` and/or inability to write the file must not make installs fail solely due to persistence; logs must not dump `/etc/os-release`.

Risk reduced (from `impact_map.md`):
- Changes touch installer scripts (`scripts/substrate/install-substrate.sh`, `scripts/substrate/dev-install-substrate.sh`) and installer smoke validation (`tests/installers/install_state_smoke.sh`), so a single cross-platform checkpoint after smoke coverage is in place reduces the risk of shipping a Linux-only behavior delta that accidentally regresses macOS/Windows installer semantics.

## Follow-ups

1) Populate `tasks.json` with slice triads for `C0`/`C1`/`C2` and add `CP1-ci-checkpoint` wiring (deps + kickoff prompt).
2) Add `tasks.json` `meta.checkpoint_boundaries=["C2"]` once slices/integ tasks exist (schema v4 boundary-only platform-fix).
3) Add feature smoke scripts under `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/smoke/` (linux/macos/windows), or explicitly revise this plan to set `feature_smoke=false` if smoke is out-of-scope for this pack.

