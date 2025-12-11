# LP1-spec – Linux World Provision Parity Fix

## Scope
- Align `scripts/linux/world-provision.sh` with installer behavior:
  - Ensure the socket unit uses `SocketGroup=substrate` and results in `/run/substrate.sock` owned `root:substrate 0660` after the script runs.
  - Create the `substrate` group when missing, add the invoking user (with warnings when unknown), and emit logout/newgrp guidance.
  - Include linger guidance so socket activation survives logout.
  - Keep `--skip-build`/profile handling intact; no root-only invocation (script should self-escalate via sudo as today).
- Update supporting docs/scripts so Linux standalone provisioning instructions match the corrected behavior (e.g., WORLD.md/INSTALLATION.md/world-socket-verify references).

## Acceptance Criteria
- Running `scripts/linux/world-provision.sh` as a non-root user installs units with `SocketGroup=substrate`, recreates `/run/substrate.sock` as `root:substrate 0660`, and prints linger guidance.
- The script ensures the `substrate` group exists and adds the invoking user when possible (or clearly warns when it cannot).
- Documentation and helper scripts referencing the provisioner (world-socket-verify, manuals) reflect the updated group/linger expectations.
- No regressions to existing installer flows; script remains idempotent and respects `--skip-build`.

## Out of Scope
- Changes to mac/WSL provisioning logic beyond updating references to the corrected Linux script.
- Policy/agent behavior or world backend changes.
