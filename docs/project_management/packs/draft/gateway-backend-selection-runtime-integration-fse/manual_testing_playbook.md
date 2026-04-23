# gateway-backend-selection-runtime-integration - manual testing playbook

This playbook is authoritative for slice 2 manual validation.
It consumes the canonical runtime parity contract and the automated parity matrix from slice 1 as upstream inputs.

## Scope

In scope:

- platform evidence for Linux, macOS, and Windows
- manual visibility for unsupported-backend failures
- smoke wrappers that mirror the same evidence shape on each platform

Out of scope:

- compatibility publication
- rollout messaging
- new backend-selection rules
- new operator commands or schema fields

## Prerequisites

- A working `substrate` binary on `PATH`, or `SUBSTRATE_BIN=/path/to/substrate`.
- Ability to create temporary directories and write temp config files.
- The same canonical runtime parity contract used by slice 1 and the upstream automated parity matrix.

## Smoke scripts

- Linux: `smoke/linux-smoke.sh`
- macOS: `smoke/macos-smoke.sh`
- Windows: `smoke/windows-smoke.ps1`

## Case 1 - Smoke validation on the current platform

Run the smoke wrapper for the host platform.

Examples:

```bash
bash docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/smoke/linux-smoke.sh
bash docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/smoke/macos-smoke.sh
pwsh -File docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/smoke/windows-smoke.ps1
```

Expected:

- Exit code `0`.
- Output contains a `PASS:` line for the platform.
- The smoke run exercises `status`, `sync`, and `restart` against an unsupported backend and confirms the failure is explicit.

## Case 2 - Manual unsupported-backend visibility

Use a temporary home and substrate home, then point the selected backend at an inventory entry that does not exist.
The failure should happen before any gateway dispatch.

Setup:

1. Create a temporary workspace directory.
2. Set `HOME`, `USERPROFILE`, and `SUBSTRATE_HOME` to temp locations.
3. Write a minimal config that selects an unsupported backend id, for example `api:anthropic`.
4. Write a permissive policy file that allows the same backend id so the failure stays focused on missing inventory.
5. Do not create the matching inventory file under `SUBSTRATE_HOME/agents/`.

Run:

```bash
substrate world gateway status
substrate world gateway sync
substrate world gateway restart
```

Expected:

- Each command exits `2`.
- stderr contains `invalid integration`.
- stderr does not mention a Codex fallback path.
- The failure is the same class on Linux, macOS, and Windows.

## Case 3 - Evidence check

Confirm that the manual evidence still traces to the upstream contract and slice-1 matrix.

Check:

- The manual cases do not introduce compatibility claims.
- The manual cases do not redefine the canonical runtime parity contract.
- The manual cases keep platform transport differences as hidden implementation detail only.
- The unsupported-backend case remains a negative visibility check, not a rollout or support promise.

Pass condition:

- The playbook reads as an evidence checklist, not as a second contract.
