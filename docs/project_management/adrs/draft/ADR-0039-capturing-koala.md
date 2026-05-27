# ADR-0039 — Persist macOS host OS details in `install_state.json`

## Status

- Status: Draft
- Date (UTC): 2026-03-30
- Owner(s): TBD (ASSUMPTION: installer/host-provisioning maintainers)

## Current Curated Draft ADR

- Current curated draft ADR: `docs/adr/draft/ADR-0039-persist-macos-host-os-details-in-install-state.md`
- This project-management file remains the planning-rich historical source retained for
  compatibility while `docs/project_management/**` is being retired.

## Scope

- Feature directory: `docs/project_management/packs/draft/persist-macos-host-os-install-state/` (ASSUMPTION: created during planning)
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/EXECUTIVE_SUMMARY_STANDARD.md`

## Related Docs (links only)

- Intake: `docs/project_management/intake/adrs/capturing_koala_adr_intake.md`
- Prior art (Linux platform persistence):
  - `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager-fse/`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager-fse/`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`

## Executive Summary (Operator)

ADR_BODY_SHA256: de7636d619e3dfdb95500c54a78c1c5700edcc46f8994e5318a3d86d4de03680

### Changes (operator-facing)

- macOS installer writes a stable record of host OS details to `<effective_prefix>/install_state.json` (default: `~/.substrate/install_state.json`), in an additive `schema_version = 1` extension aligned with other platforms.
  - Existing: `install_state.json` is primarily a Linux host-state metadata file (group/linger + Linux platform detection) and is often absent on successful macOS installs.
  - New: After a successful macOS install, `install_state.json` exists and includes a macOS OS-details block (`host_state.os.*`) suitable for later diagnostics and support workflows.
  - Why: reduces support back-and-forth and enables deterministic “what OS/version/build/arch was this installed on?” attribution for macOS, similar to the Windows host install flow.

## Problem / Context

- Substrate’s install flows increasingly rely on best-effort platform detection plus persisted installer metadata for later diagnostics and guidance (see ADR-0032 for Linux).
- On macOS, diagnosing Lima/Virtualization.framework prerequisites and behavioral differences often starts with “what macOS version/build/arch are you on?”, but there is no canonical, installer-owned record to consult later.
- This gap is especially painful when a user installs Substrate, then later reports world issues (or a support artifact is collected without complete host details).

## Goals

- After a successful macOS install, `<effective_prefix>/install_state.json` exists and includes stable macOS OS details captured at install time.
- Keep the file additive and backwards compatible:
  - `schema_version` remains `1`.
  - existing keys (`host_state.group`, `host_state.linger`, Linux `host_state.platform.*`) are preserved when present.
  - unknown keys are preserved.
- Best-effort, warning-only metadata: failures to read/parse/write metadata must not convert an otherwise successful install into a failure.

## Non-Goals

- Changing Linux distro/pkg-manager persistence (`host_state.platform.os_release.*`, `host_state.platform.pkg_manager.*`).
- Introducing new macOS provisioning behavior or altering the Lima VM configuration.
- Persisting sensitive host information (no hostnames, no serial numbers, no broad `system_profiler` capture).

## User Contract (Authoritative)

### Installer metadata file

- Canonical file: `install_state.json`
- Canonical location: `<effective_prefix>/install_state.json`
  - (Default effective prefix on macOS via `scripts/substrate/install-substrate.sh`: `~/.substrate`)
- Schema-version policy: `schema_version` MUST remain integer `1`.
- Compatibility policy: additive-only; consumers MUST ignore unknown keys.

### New fields (macOS)

Add a macOS-only host OS block under `host_state.os`:

- `host_state.os`
  - Type: object
  - Required: yes after successful macOS installs (unless explicitly disabled by a future knob)
- `host_state.os.family`
  - Type: string
  - Value: `"macos"`
- `host_state.os.product_version`
  - Type: string
  - Source: `sw_vers -productVersion` (best-effort)
- `host_state.os.build_version`
  - Type: string
  - Source: `sw_vers -buildVersion` (best-effort)
- `host_state.os.arch`
  - Type: string
  - Source: `uname -m` (best-effort)

Write semantics:
- macOS successful installs MUST write (create or update) the canonical file even when no Linux-only host-state “events” exist.
- Writes MUST use a same-directory temp-file + atomic replace flow (no in-place truncation), aligned with the Linux persistence posture.
- On read/parse failures, the installer MUST emit warning-only diagnostics and seed from a fresh `schema_version = 1` document.

Read semantics (future consumers):
- Prefer persisted `host_state.os.*` when present for user-facing guidance and support bundles, but always tolerate missing/partial values and fall back to runtime detection.

## Options

### Option A — Extend `install_state.json` with macOS `host_state.os.*` (recommended)

- Pros: one canonical installer metadata file; consistent with Linux persistence and with cross-platform “support bundle” needs.
- Cons: expands `install_state.json` from “Linux cleanup metadata” to “cross-platform install metadata”.

### Option B — Write macOS OS details to `config.yaml` under `install.*` (avoid)

- Pros: file exists on macOS already; CLI can read it easily.
- Cons: conflates operator intent config with detected host facts; less appropriate for “installer evidence”.

### Option C — Create `host_os.json` beside `install_state.json` (avoid)

- Pros: keeps `install_state.json` narrowly scoped.
- Cons: state fragmentation; more documentation + tooling complexity.

## Decision Summary

- Chosen: **Option A**
- Rationale: `install_state.json` is already the installer’s durable metadata store and is explicitly designed to be additive and warning-only. Extending it to record macOS OS details creates a single, canonical place for later diagnostics without changing provisioning behavior.

## Validation Plan (Authoritative)

- Smoke coverage:
  - Add/extend macOS installer parity/smoke harness to assert:
    - `install_state.json` exists after a successful macOS install.
    - `host_state.os.family`, `product_version`, `build_version`, and `arch` are present when the underlying commands succeed.
    - Failures to collect OS details do not fail the installer (warnings only).
- Manual validation:
  - Run `scripts/substrate/install-substrate.sh` on macOS (hosted + `--no-world`) and verify the presence and content of `<effective_prefix>/install_state.json`.
