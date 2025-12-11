# M2-spec – Installer Parity (macOS)

## Scope
- Align mac dev/prod installers with P0 behaviors delivered for Linux/WSL:
  - When a Linux `world-agent` binary is absent or non-ELF, build it inside Lima (with resilient toolchain bootstrap) and install it in the guest; prefer copying if a valid ELF is bundled.
  - (Optional) Install a CLI shim inside Lima when required for diagnostics, keeping the behavior consistent across dev/prod.
  - Ensure installer metadata/cleanup-state flows mirror Linux (group membership, linger guidance, cleanup-state flag handling).
  - Prod path should copy the bundled Linux agent into Lima by default; only fall back to in-guest builds when the bundle is missing or invalid. Dev path may build in-guest for iteration.
- Update mac uninstall flows to mirror Linux cleanup:
  - Remove agent binary, CLI shim (if installed), units, runtime/state dirs, and host-forwarded sockets; respect cleanup-state behavior.
- Keep installer logging and failure modes consistent with P0 (actionable guidance, no silent skips).

## Acceptance Criteria
- Dev and prod mac installers can provision a usable world even when no Linux agent is bundled, by building inside Lima or failing fast with clear guidance.
- Prod installer copies the bundled Linux agent into Lima when present/valid; in-guest build only occurs as a logged fallback.
- Host and guest cleanup paths remove agent binaries/units/shims and sockets in parity with Linux `--cleanup-state`.
- Installer/uninstaller metadata handling matches Linux semantics (no unexpected lingering group/linger changes on mac).
- Logging clearly distinguishes copy vs build-in-VM flows and surfaces next steps when prerequisites (Lima, toolchain, network) are missing.

## Out of Scope
- Lima VM state verification/provisioning (handled in M1).
- Backend runtime behavior (fs_mode, forwarding) and doctor UX (handled in M3).
