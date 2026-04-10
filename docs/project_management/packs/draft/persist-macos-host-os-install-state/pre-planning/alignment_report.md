## Misalignment / follow-ups (wrapper-detected)
- None detected

## Consolidated FSE pre-planning follow-ups (wrapper-compiled)
### Gates / hard decisions
- None

### Decision Register required
- None detected

### Checkpoint intent follow-ups
- None

### Risks + unknowns
- None

### Other follow-ups
- docs/project_management/packs/draft/persist-macos-host-os-install-state/decision_register.md — lock DR-0001 to hosted installer plus dev installer, DR-0002 to `host_state.os.family` plus available leaves, and DR-0003 to `tests/mac/installer_parity_fixture.sh` primary plus `tests/installers/install_state_smoke.sh` secondary. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L245`)
- docs/project_management/adrs/draft/ADR-0039-capturing-koala.md — reconcile the hosted-only validation wording with the selected shared producer scope before promotion. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L246`)
- docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md — define the no-new-CLI contract, canonical path wording, warning-only diagnostics, and future-consumer read precedence. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L249`)
- docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md — define the `host_state.os.*` field set, field-level absence semantics, and merge preservation of Linux and cleanup subtrees. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L250`)
- docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md — define the same-directory temp-file path, atomic replace sequence, parse-failure recovery, and failed-write cleanup posture. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L251`)
- docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md — define the macOS producer matrix, Linux and Windows no-change guarantees, the uninstaller no-change boundary, and the exact automated evidence map. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L252`)
- docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md — define additive-only compatibility, unknown-key preservation, and reader tolerance of `host_state.os.*`. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L253`)
- docs/project_management/packs/draft/persist-macos-host-os-install-state/manual_testing_playbook.md — cover hosted macOS install, hosted macOS `--no-world`, dev-install macOS producer coverage, malformed-file recovery, and Linux cleanup no-change verification. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L254`)
- Ratify the producer-scope baseline in `decision_register.md` and align `contract.md`, `filesystem-semantics-spec.md`, and `platform-parity-spec.md` to the same installer-entrypoint set. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/minimal_spec_draft.md#L105`)
- Ratify the partial-capture baseline in `decision_register.md` and encode the exact leaf-presence contract in `install-state-schema-spec.md`, including the object-presence rule for `host_state.os`. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/minimal_spec_draft.md#L106`)
- Ratify the validation split in `decision_register.md` and map each assertion to `tests/mac/installer_parity_fixture.sh`, `tests/installers/install_state_smoke.sh`, and `manual_testing_playbook.md`. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/minimal_spec_draft.md#L107`)
- Reconcile ADR-0039 validation wording with the selected producer scope so the ADR and downstream specs point at the same installer matrix. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/minimal_spec_draft.md#L108`)
- Reconcile `docs/INSTALLATION.md` so operator docs state that macOS writes diagnostic-only `host_state.os.*` metadata while Windows remains on the no-write side for this feature. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/minimal_spec_draft.md#L109`)
- Restate the existing upstream `effective_prefix` resolution rule in `contract.md` so downstream docs do not drift on CLI/config/env precedence even though this feature adds no new precedence inputs. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/minimal_spec_draft.md#L110`)
- Producer-scope wording remains unresolved in the ADR body (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md#L321`)
- Partial-capture semantics remain unresolved (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md#L325`)
- Automated validation ownership remains unresolved (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md#L329`)
- Operator docs contain live drift (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md#L333`)
- Promote this staged candidate after overlap-safe validation. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/workstream_triage.md#L314`)
- Carry the draft candidate order `PMHOS-01`, `PMHOS-02`, `PMHOS-03` forward as the baseline candidate skeleton during downstream planning. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/workstream_triage.md#L315`)
- Resolve execution wiring later in the subsystem that owns checkpoint execution and implementation sequencing. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/workstream_triage.md#L316`)

