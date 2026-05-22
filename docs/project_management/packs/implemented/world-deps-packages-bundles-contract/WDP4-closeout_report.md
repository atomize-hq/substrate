# Slice Closeout Gate Report — world-deps-packages-bundles-contract / WDP4

Date (UTC): 2026-02-14T19:56:55Z

Standards:

- `docs/project_management/system/standards/execution/SLICE_CLOSEOUT_GATE_STANDARD.md`
- `docs/project_management/system/standards/adr/EXECUTIVE_SUMMARY_STANDARD.md` (behavior delta format)

Feature directory:

- `docs/project_management/packs/active/world-deps-packages-bundles-contract`

Slice spec:

- `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP4-spec.md`

## Behavior Delta (Existing → New → Why)

- Existing behavior:
  - `substrate world deps current install` / `... sync` were `--dry-run` only (WDP3), with no in-world script execution and no wrapper generation.
- New behavior:
  - `substrate world deps current install <item...>` now applies **script** packages in-world (no apt in WDP4), running installer scripts via `bash -lc` when available (fallback `sh -c`).
  - `substrate world deps current sync` applies the current effective enabled set (script-only), generating deterministic wrapper entrypoints for `wrappers[]`.
  - Wrapper entrypoints are created in the world-deps bin dir and include actionable stderr on failure (wrapper kind, key fields, and a next step).
  - Script install paths remain contract-default (`/var/lib/substrate/world-deps/bin`) and honor the internal override `SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR` (used by host-execute test stubs).
- Why:
  - Make `install.method=script` produce runnable entrypoints under the world shell contract (non-interactive `/bin/sh -c` and REPL evaluator semantics).
- Links:
  - `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP4-spec.md`
  - `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md` (Wrapper generation + script installs; `current install` / `current sync`)

## Spec Parity (No Drift)

- [x] Acceptance criteria satisfied
- [x] Any spec changes during the slice are recorded (with rationale) (none)

## Checks Run (Evidence)

- `cargo fmt`: pass
- `cargo clippy --workspace --all-targets -- -D warnings`: pass
- Relevant tests:
  - `cargo test -p shell --test world_deps_script_install_wdp4 -- --nocapture`: pass
- `make integ-checks`: pass

## Cross-Platform Smoke (if applicable)

Record run ids/URLs for required platforms:

- Linux: not run (not required for WDP4)
- macOS: not run (not required for WDP4)
- WSL: not run (not required for WDP4)

## Smoke ↔ Manual Parity

- [ ] Smoke scripts run the same commands/workflows as the manual testing playbook (minimal viable subset) (not evaluated for WDP4)
- [ ] Smoke scripts validate exit codes and key output (not just “command ran”) (not evaluated for WDP4)

Notes:

- WDP4 validates script install + wrapper generation via a host-execute world-service stub and ensures wrappers are deterministic and fail with actionable stderr when wrapper sources are invalid.
