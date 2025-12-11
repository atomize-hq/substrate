# M1-spec – Lima Migration & Socket Parity

## Scope
- Detect and migrate existing Lima `substrate` VMs to the socket-activated world-agent layout used in P0:
  - Ensure `/etc/systemd/system/substrate-world-agent.service` and `.socket` are present/enabled, with `/run/substrate.sock` created at boot.
  - Install the Linux `substrate-world-agent` binary inside the VM (copied or built) when missing or outdated.
  - Align socket permissions/ownership with Linux/WSL expectations (root + substrate group or documented equivalent) and ensure the default SSH user can access the socket via group membership or forwarding.
  - Ensure the Lima-side provisioning path (including any reuse of `scripts/linux/world-provision.sh` or equivalents) sets `SocketGroup=substrate`, adds the user to the `substrate` group, and carries linger guidance so socket activation works post-login.
- Make `scripts/mac/lima-warm.sh` (and related helpers/profiles) idempotent:
  - Start or create the VM, detect legacy/non-socket setups, and trigger reprovision/restart when required.
  - Surface actionable guidance when migration cannot proceed (e.g., missing agent, stale profile, damaged units).
- Provide clear doctor/log output when migration is required or after a successful migration.

## Acceptance Criteria
- Running the updated warm/provision flow on a pre-P0 Lima VM results in active `substrate-world-agent.service` + `.socket`, an installed agent binary, and a reachable `/run/substrate.sock` via the configured forwarding path.
- Socket ownership/permissions are consistent with the documented mac model (explicitly documented if diverging from Linux), and the SSH user can reach the socket without manual chmods.
- Re-running warm/provision is idempotent and does not regress a healthy VM; migration failures emit actionable messages (what to fix/try next).
- World doctor (host or in-VM, as defined later) reports the migrated socket/agent state and fails loudly when migration is pending.

## Out of Scope
- Linux/WSL provisioning changes beyond mac parity.
- Replay or policy semantics (handled in later triads).
- Installer/uninstaller changes (M2 handles installer parity).
