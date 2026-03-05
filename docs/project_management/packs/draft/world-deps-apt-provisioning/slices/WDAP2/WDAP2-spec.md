# WDAP2-spec — Provisioning wiring: scripts/installer integration + ordering

## Behavior delta (single)
- Existing: `substrate world enable` delegates to `scripts/substrate/world-enable.sh`, which provisions the world backend and then runs `substrate world deps current sync` as part of the helper flow. Scripted flows (`install-substrate.sh`, `world-enable.sh`) treat `world deps current sync` failures generically and do not encode an explicit provisioning-time APT step.
- New: Scripted entrypoints are wired so `substrate world enable --provision-deps` provisions APT requirements **before** any `substrate world deps current sync` is invoked, and installer/helper messaging deterministically points operators at `substrate world enable --provision-deps` when runtime `sync` fails due to missing APT provisioning. No script in scope introduces host OS APT/dpkg mutation; all APT execution remains in-world (guest-only) or is rejected per contract.
- Why: Preserve hardened-runtime invariants and “no host OS mutation” guard rails across all provisioning entrypoints, while keeping installer flows coherent under the new provisioning-time APT posture.

## Scope
- Wire `substrate world enable --provision-deps` so helper-script-driven `world deps current sync` does not run before provisioning-time APT completes.
- Update provisioning helper/installer scripts to surface deterministic remediation for missing provisioning and to avoid host OS APT/dpkg mutation.
- Keep platform warmers (`scripts/linux/world-provision.sh`, `scripts/mac/lima-warm.sh`, `scripts/windows/wsl-warm.ps1`) scoped to world-agent/world-backend readiness only; they do not perform world-deps APT provisioning.

## Behavior (authoritative)
### Ordering invariant for `substrate world enable --provision-deps`
When invoked as `substrate world enable --provision-deps` (without `--dry-run`):
1) World-backend enable runs (baseline `substrate world enable` behavior, via `scripts/substrate/world-enable.sh`).
2) APT provisioning runs (guest-only; `WDAP0`).
3) `substrate world deps current sync` runs exactly once after APT provisioning completes successfully (exit `0`) or is a no-op by contract.

Critical rule:
- `substrate world deps current sync` MUST NOT run before step (2) when `--provision-deps` is present.

Implementation wiring requirement (host-side):
- When `--provision-deps` is present, the `scripts/substrate/world-enable.sh` helper MUST be invoked with `--no-sync-deps` so the helper does not run `world deps current sync` internally.

### `--dry-run` ordering
When invoked as `substrate world enable --provision-deps --dry-run`:
- No helper script is executed.
- No `world deps current sync` is executed.
- Only the provisioning preview behavior in `WDAP0` occurs (derive + print requirements; no mutation).

### Helper script behavior (`scripts/substrate/world-enable.sh`)
Rules:
- When `--no-sync-deps` is present, the helper MUST skip calling `substrate world deps current sync` and MUST emit the existing deterministic log line:
  - `Skipping world deps sync (--no-sync-deps)`
- The helper MUST NOT run `apt-get` or `dpkg` on the host OS as part of world-deps provisioning.

### Installer behavior (`scripts/substrate/install-substrate.sh`)
When the installer is invoked with `--sync-deps`, it runs `substrate world deps current sync` after world backend provisioning.

New remediation rule:
- If `substrate world deps current sync` exits `4`, the installer MUST emit a stderr note that includes the exact remediation command:

  ```text
  substrate world enable --provision-deps
  ```

- The installer MUST still exit successfully (exit `0`) after emitting this note.
- The installer MUST NOT attempt to run APT/dpkg on the host OS to “fix” the situation.

### Platform entrypoint constraints (no host OS mutation)
- Linux host-native:
  - No script in scope introduces host APT/dpkg mutation.
  - Any remediation that references provisioning MUST include the contract language `Substrate will not mutate the host OS`.
- macOS Lima:
  - Scripts ensure the Lima VM + world-agent are running; APT provisioning is performed in-world via `substrate world enable --provision-deps`.
- Windows:
  - Scripts may provision the WSL backend and world-agent readiness; APT provisioning remains unsupported for this feature and any remediation MUST include `unsupported on Windows` (per contract).

## Acceptance criteria
- AC-WDAP2-01: With `substrate world enable --provision-deps` (non-dry-run), `scripts/substrate/world-enable.sh` is invoked with `--no-sync-deps`, and `substrate world deps current sync` is invoked only after provisioning-time APT completes.
- AC-WDAP2-02: With `substrate world enable --provision-deps --dry-run`, neither the helper script nor `substrate world deps current sync` executes.
- AC-WDAP2-03: `scripts/substrate/world-enable.sh --no-sync-deps` emits the exact line `Skipping world deps sync (--no-sync-deps)` and does not invoke `substrate world deps current sync`.
- AC-WDAP2-04: `scripts/substrate/install-substrate.sh --sync-deps` prints a remediation note containing `substrate world enable --provision-deps` when `substrate world deps current sync` exits `4`, and the installer still exits `0`.
- AC-WDAP2-05: No script in this slice adds host OS APT/dpkg execution as part of world-deps provisioning; all APT execution remains in-world (guest-only) or is rejected per the contract.

## Out of scope
- APT requirement derivation, probe, and in-world execution details (owned by `WDAP0`).
- Runtime fail-early behavior for `substrate world deps current sync|install` (owned by `WDAP1`).
- Operator-doc and upstream contract reconciliation work (owned by `WDAP3`).
