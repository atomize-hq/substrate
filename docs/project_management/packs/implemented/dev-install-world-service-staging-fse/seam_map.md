# Seam Map - dev-install-world-service-staging

This map converts the deep-planned source pack into governance-ready seam briefs without carrying forward slice-level execution units.

## Extraction axis and horizon

- **Primary seam axis**: workflow-first capability seams with one conformance seam.
- **Active seam**: `SEAM-1` because the runtime preflight, accepted staged path rule, dry-run semantics, and deterministic failure posture are the first publishable contracts and are upstream of both staging and validation.
- **Next seam**: `SEAM-2` because Linux dev-install staging must align to the exact path rule, ordering invariants, and override carve-outs that `SEAM-1` publishes.
- **Future seam**: `SEAM-3` because parity evidence and checkpoint closeout should consume landed truth from the first two seams rather than invent that truth early.

## Seam inventory

| Seam | Name | Type | Horizon | Source-pack absorption | Value boundary | Primary verification path |
| --- | --- | --- | --- | --- | --- | --- |
| `SEAM-1` | Standard version-dir preflight + deterministic remediation | capability | active | Absorbs source slice `DIWAS0`, the runtime-facing portions of `contract.md`, `decision_register.md`, `minimal_spec_draft.md`, and the `world enable` impact-map decisions | `substrate world enable` either finds an accepted staged artifact or fails early with one deterministic, operator-visible remediation before helper launch or privileged work | runner tests, dry-run / non-dry-run ordering proof, config-write ordering proof, override carve-out proof |
| `SEAM-2` | Linux dev-install world-service staging | capability | next | Absorbs source slice `DIWAS1`, the selected-profile / overwrite decisions, Linux staging behavior in `contract.md`, and installer-smoke alignment for dev-install | Linux `dev-install-substrate.sh --no-world` leaves the enable-later workflow ready by staging `world-service` into both accepted paths while keeping the world disabled | `readlink` staging proof for debug and release, rerun refresh proof, `world.enabled: false` proof, installer-smoke regression proof |
| `SEAM-3` | Cross-platform validation + drift guards | conformance | future | Absorbs `platform-parity-spec.md`, `manual_testing_playbook.md`, `smoke/linux-smoke.sh`, `plan.md`, `quality_gate_report.md`, `tasks.json`, `session_log.md`, and the checkpoint plan intent | The full feature stays provable, checkpointable, and non-drifting across Linux behavior evidence plus macOS/Windows parity surfaces | Linux feature smoke, installer smoke, manual cases 1-5, compile parity on `linux`, `macos`, `windows`, checkpoint evidence capture |

## Why these seams were kept

- `SEAM-1` is a real user-facing capability seam, not an org-chart “runner seam”: it changes what an operator sees when enable-later is missing its staged artifact and it can be verified independently.
- `SEAM-2` is a separate capability seam because Linux dev-install staging has a distinct owned surface, a distinct acceptance path, and a different failure / drift profile than runtime preflight.
- `SEAM-3` is a justified conformance seam because the remaining meaningful work is cross-seam hardening: smoke alignment, checkpoint proof, platform-claim boundaries, and drift guards after the two producer seams land.

## Candidates that were pruned

- **Standalone contract-definition seam**: pruned because the path rule, missing-artifact remediation, and staging profile / refresh rules belong to the seams that produce and consume them; a separate contracts seam would mostly mirror the source workstream triage.
- **Standalone tasks / kickoff / checkpoint orchestration seam**: pruned because `tasks.json`, kickoff prompts, and session-log wiring are governance scaffolds and evidence surfaces, not a separate product or runtime capability boundary.
- **Standalone dry-run seam**: pruned because dry-run ordering is inseparable from the runtime preflight contract in `SEAM-1`.
- **Standalone macOS or Windows seam**: pruned because the source pack explicitly keeps behavior delta on Linux only and uses macOS / Windows as parity surfaces rather than new capability boundaries.
- **Standalone production-installer seam**: pruned because the source contract keeps `scripts/substrate/install-substrate.sh` outside the owned behavior change except for the shared accepted path rule and regression evidence.

## Source-pack mapping notes

- Source slice `DIWAS0` maps cleanly to `SEAM-1`.
- Source slice `DIWAS1` maps cleanly to `SEAM-2`.
- Source checkpoint `CP1-ci-checkpoint`, `platform-parity-spec.md`, `manual_testing_playbook.md`, `smoke/linux-smoke.sh`, `tasks.json`, and `quality_gate_report.md` map to `SEAM-3`.
- Source pack authorities such as `contract.md`, `decision_register.md`, `spec_manifest.md`, `impact_map.md`, and `plan.md` feed all seams and remain pack-wide basis documents rather than standalone seams.

## Decomposition posture

- `SEAM-1` is ready for authoritative downstream seam-local planning because it owns the first publishable contracts and the source pack already narrows the touch set and verification path.
- `SEAM-2` should stay only provisionally deeper-planned until `SEAM-1` publishes the accepted path rule, no-write ordering, and override carve-out as closeout-backed truth.
- `SEAM-3` remains seam-brief depth only; it must consume closeout-backed reality from `SEAM-1` and `SEAM-2` rather than guessing the landed evidence matrix ahead of time.
