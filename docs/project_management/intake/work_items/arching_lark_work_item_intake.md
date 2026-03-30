---
codename: arching_lark
created: "2026-03-30T21:46:34Z"
status: draft
depends_on: []
---

# Work Item Intake Sheet

## 1. Codename + date + status

- Codename: `arching_lark`
- Created: 2026-03-30T21:46:34Z
- Status: draft

## 2. Title (imperative)

Add alternate macOS Lima VM/profile support to installer world provisioning.

## 3. Why not ADR

- This is installer/runtime parity work, not a new architectural choice.
- The repo already has an alternate Arch-family Lima profile and a VM-name override model in the backend and doctor surfaces; the gap is that installer flows still lag that model.
- If the team later wants to choose between materially different installer contracts for alternate world images, that contract decision can be raised separately. This WI is about implementing the already-implied capability consistently.

## 4. Task definition (bounded)

- Make the macOS installer provisioning path support a non-default Lima VM name and a non-default Lima profile end-to-end, instead of assuming the VM is always literally `substrate` and the profile is always Ubuntu-based.
- Primary target scenario:
  - `LIMA_VM_NAME=substrate-arch`
  - `LIMA_PROFILE_PATH=scripts/mac/lima/substrate-arch.yaml`
  - `./scripts/substrate/dev-install-substrate.sh --profile release --world-netfilter`
- The end state must allow an operator to provision/build/install against `substrate-arch` without renaming or disturbing the default Ubuntu `substrate` fixture.
- Remove or replace APT-only guest bootstrap assumptions inside the macOS installer path so an Arch-family guest can complete the “build Linux substrate + world-agent inside Lima” fallback path.
- Cover the staged runtime-bundle path too: the helper/profile artifacts linked into the effective prefix must be sufficient for later reuse of the selected profile/VM strategy, or the implementation must intentionally constrain that path and document the reason.
- Review the production installer’s macOS path at the same time. It currently shares the same hardcoded `substrate` assumption during post-provision verification, so future planning should treat dev/prod parity as part of the same touch set unless there is a deliberate split.

## 5. Done means (<= 8 outcomes)

- `scripts/substrate/dev-install-substrate.sh` can provision a non-default macOS Lima VM/profile by env override, including the Arch-family profile at `scripts/mac/lima/substrate-arch.yaml`.
- The dev installer no longer contains macOS world-provisioning calls that hardcode `limactl shell substrate ...` or `limactl copy ... substrate:...`.
- The macOS installer path no longer assumes guest package bootstrap must use `apt-get`; the Arch-family path can obtain Rust/cargo or otherwise build the needed Linux binaries successfully.
- An Arch-family run can build/install `/usr/local/bin/substrate` and `/usr/local/bin/substrate-world-agent` inside the alternate VM and cache Linux ELFs back into the managed prefix.
- The default Ubuntu `substrate` installer path still works unchanged when no override vars are set.
- The production installer’s macOS verification path is either updated to the same VM-name/profile contract or explicitly split into a follow-up with rationale captured in the implementation notes.
- macOS installer tests cover both default and alternate VM/profile behavior.
- If the installer begins treating any env vars as supported operator inputs, the contract/docs inventory is updated accordingly.

## 6. Current gap / reproduction

- The operator expectation that triggered this WI was:

  ```bash
  SUBSTRATE_LIMA_VM_NAME=substrate-arch \
    ./scripts/substrate/dev-install-substrate.sh --profile release --world-netfilter
  ```

- That does **not** work today for multiple reasons:
  - `scripts/mac/lima-warm.sh` reads `LIMA_VM_NAME` and `LIMA_PROFILE_PATH`, not `SUBSTRATE_LIMA_VM_NAME`.
  - After calling `lima-warm.sh`, `scripts/substrate/dev-install-substrate.sh` hardcodes `substrate` in every later `limactl shell` and `limactl copy` call.
  - The in-guest fallback build bootstrap inside the installer still does `apt-get update && apt-get install -y rustc cargo`, which is Ubuntu-specific and unsuitable for an Arch-family guest.
- Even if the caller uses `LIMA_VM_NAME=substrate-arch` and `LIMA_PROFILE_PATH=scripts/mac/lima/substrate-arch.yaml`, the installer will still drift back to the default VM because of the hardcoded `substrate` calls after the warm step.
- The production installer has a narrower but related gap: its macOS post-provision verification still checks `limactl shell substrate ...`, so it does not yet verify alternate VM names correctly either.

## 7. Likely touch paths

- `scripts/substrate/dev-install-substrate.sh`
  - `cache_linux_binary_from_lima()` currently copies from `substrate:${vm_path}`.
  - `stage_dev_world_runtime_bundle()` currently links only `scripts/mac/lima/substrate.yaml` and `scripts/mac/lima/substrate-dev.yaml`; it does not stage the Arch profile today.
  - The macOS world-provisioning block around the `LIMA_WARM` call, the in-guest build shell, the Linux binary install steps, and the systemd enable/restart calls all hardcode `substrate`.
  - The in-guest bootstrap currently assumes `apt-get` for Rust/cargo installation.
- `scripts/substrate/install-substrate.sh`
  - `provision_macos_world()` already delegates to `scripts/mac/lima-warm.sh`, but its post-provision verification still hardcodes `limactl shell substrate ...`.
  - Future planning should decide whether prod installer parity belongs in this WI or a follow-up, but this file is part of the relevant touch surface.
- `scripts/mac/lima-warm.sh`
  - This is already the source of truth for `LIMA_VM_NAME` and `LIMA_PROFILE_PATH`.
  - Any installer changes should align to this helper’s contract instead of inventing a parallel one.
- `scripts/mac/lima/substrate.yaml`
- `scripts/mac/lima/substrate-dev.yaml`
- `scripts/mac/lima/substrate-arch.yaml`
  - The future agent should decide whether the staged runtime bundle should ship/link this profile by default once installer support lands.
- `tests/mac/installer_parity_fixture.sh`
  - This fixture copies/stubs `scripts/mac/lima-warm.sh` and currently stages only the default Ubuntu profile.
  - This is the most likely place to add alternate-profile installer parity coverage without depending on a real Lima VM.
- `tests/installers/install_state_smoke.sh`
- `tests/installers/install_smoke.sh`
  - These may need a focused macOS env-override case if the installer contract becomes operator-visible rather than purely internal.
- `docs/internals/env/inventory.md`
  - Update if the implementation formalizes `LIMA_VM_NAME`, `LIMA_PROFILE_PATH`, or `SUBSTRATE_LIMA_VM_NAME` as installer-consumed variables rather than only helper-script inputs.
- `docs/INSTALLATION.md`
  - Update only if the user-facing installer contract changes and the team wants operators to rely on alternate VM/profile provisioning through installer entrypoints.

## 8. Dependencies (ADR/WI)

- depends_on_adrs: []
- depends_on_work_items: []
- blocks: []

## 9. Lift Summary

### Lift Vector v1

<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "model_version": 1,
  "touch": {
    "create_files": 1,
    "edit_files": 8,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": 0,
    "boundary_crossings": 4
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 0,
    "behavior_deltas": 1
  },
  "qa": {
    "new_test_files": 0,
    "new_test_cases": 5
  },
  "docs": {
    "new_docs_files": 1
  },
  "ops": {
    "new_smoke_steps": 1,
    "ci_changes": 0
  },
  "risk": {
    "cross_platform": false,
    "security_sensitive": false,
    "concurrency_or_ordering": false,
    "migration_or_backfill": false,
    "unknowns_high": 2
  },
  "notes": "Estimate assumes macOS-only installer work across dev installer, prod installer verification parity, Lima profile staging, and installer fixture coverage."
}
```
<!-- PM_LIFT_VECTOR:END -->

### Computed outputs (estimated pending `make pm-lift-intake`)

```text
Lift Score (v1): ~54
Estimated slices: 4
Confidence: medium
Likely split triggers:
- installer shell script touch set spans dev + prod + staged helper bundle + fixture tests
- guest bootstrap behavior changes across Ubuntu and Arch-family paths
```

## 10. Open questions

- Should the installer itself accept `SUBSTRATE_LIMA_VM_NAME` as a first-class alias, or should the supported operator contract stay limited to `LIMA_VM_NAME` + `LIMA_PROFILE_PATH` because that is what `lima-warm.sh` already consumes?
  - Recommended default: support both VM-name env vars with the same precedence used by `world-mac-lima` and doctor (`SUBSTRATE_LIMA_VM_NAME`, then `LIMA_VM_NAME`), while continuing to use `LIMA_PROFILE_PATH` for profile selection.
- Is production installer parity in scope for the same implementation?
  - Recommended default: yes, at least for VM-name/profile verification parity, because the production installer already delegates to the same `lima-warm.sh` helper and will otherwise keep drifting.
- Should the staged runtime bundle always link `scripts/mac/lima/substrate-arch.yaml`, or only whatever profile was selected at install time?
  - Recommended default: link the Arch profile alongside the existing default/dev profiles so the staged helper remains self-contained and deterministic.
- Should the in-guest Rust bootstrap become package-manager-aware, or should it move to a manager-neutral `rustup`-first flow with package-manager install only as an optimization/fallback?
  - Recommended default: prefer a manager-neutral `rustup` path or reuse the `lima-warm.sh` bootstrap logic, to avoid re-encoding distro-specific branching in the installer.
