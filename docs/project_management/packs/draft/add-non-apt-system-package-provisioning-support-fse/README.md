# Add non-APT system-package provisioning support - seam extraction

Source: `add-non-apt-system-package-provisioning-support.zip`

This pack captures seam briefs, authoritative threading, pack-level review surfaces, seam-exit intent, and governance scaffolds. It is intentionally one level above seam-local decomposition.

- Start here: `scope_brief.md`
- Seam overview: `seam_map.md`
- Threading: `threading.md`
- Pack review surfaces: `review_surfaces.md`
- Governance: `governance/remediation-log.md`

## Restated scope

Transform the deep-researched planning pack for manager-aware world-deps system-package provisioning into a governance-ready seam pack without carrying forward the old pack's slice files as authoritative execution units.

The extracted scope remains the same as the source pack:

- make `substrate world enable --provision-deps` manager-aware for system-package items whose `install.method` is `apt` or `pacman`
- keep world-manager selection in-world by probing `/etc/os-release` plus in-world package-manager availability instead of host PATH or host installer state
- add additive `install.method=pacman` plus `install.pacman` schema support without inventing a distro-translation layer
- keep runtime `substrate world deps current sync|install` read-only for system-package managers, with deterministic fail-early remediation that points back to provisioning
- preserve Linux/macOS/Windows validation scope while keeping Linux host-native and Windows provisioning unsupported and fail-closed
- reconcile shared contract authority with overlapping ADR and planning-pack docs so this pack becomes the single manager-aware truth

## Extraction assumptions

- The source pack's accepted slice order `NASP0 -> NASP1 -> NASP2 -> NASP3 -> NASP4` is the best code-grounded signal for the delivery path inside the feature.
- The source pack's `NASP-PWS-contract` workstream is elevated into its own seam instead of being hidden inside `NASP0`, because every other source workstream depends on it and it owns the multi-pack authority handoff plus the accepted decision register.
- `NASP-PWS-tasks_checkpoints` remains governance scaffolding, not a seam. Its outputs stay represented through pack-level threading, review surfaces, and closeout scaffolds rather than as a feature seam.
- `plan.md`, `tasks.json`, `session_log.md`, kickoff prompts, and checkpoint docs are preserved as lineage and governance inputs only.
- Closeout documents in `governance/` start as scaffolds and become authoritative as seams land.

Execution horizon:

- Active seam: `SEAM-2`
- Next seam: `SEAM-3`

Policy:

- only the active seam is eligible for authoritative downstream sub-slices by default
- the next seam may later receive seam-local review + slices, and only provisional candidate-subslice hints
- active and next seams must eventually terminate in a dedicated final `seam-exit-gate` slice once seam-local planning begins
- future seams remain seam briefs

## Source-plan lineage

The deep-researched source pack is preserved as lineage, not as the output structure:

- `NASP-PWS-contract`, `contract.md`, and `decision_register.md` roll up into `SEAM-1`
- `NASP-PWS-os_probe` and `NASP0` roll up into `SEAM-2`
- `NASP-PWS-schema_inventory`, `world-deps-pacman-schema-spec.md`, and `NASP1` roll up into `SEAM-3`
- `NASP-PWS-provisioning_wiring` and `NASP2` roll up into `SEAM-4`
- `NASP-PWS-runtime_fail_early` and `NASP3` roll up into `SEAM-5`
- `NASP-PWS-docs_validation`, `NASP4`, `platform-parity-spec.md`, `manual_testing_playbook.md`, and the three smoke scripts roll up into `SEAM-6`
- `NASP-PWS-tasks_checkpoints` remains a governance-only lineage input

## Why the seam count is six

The seam count is intentionally **not** derived mechanically from the source pack's slice count.

The source pack already split its draft three-slice model into five accepted slices because probe routing, schema support, provisioning execution, runtime fail-early behavior, and cross-platform validation each carry distinct churn and verification risk. On top of that, the source pack also kept a separate prerequisite contract/decision workstream with no dependencies. This extractor preserves that shape by:

- promoting the prerequisite contract/decision workstream into `SEAM-1`
- keeping the five accepted slice seams visible as `SEAM-2` through `SEAM-6`
- leaving checkpoint/task automation as governance instead of inventing a seventh seam

The result stays compliant with the seam extractor skill by removing authoritative slice files while retaining the source pack's depth, critical-path reasoning, cross-pack authority boundaries, and validation posture.
