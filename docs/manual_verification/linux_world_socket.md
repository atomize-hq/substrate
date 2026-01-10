# Linux world socket verification harness

Use this harness when you need to *prove* that a provisioned Linux host emits the `world_socket`
block in `substrate host doctor --json` (and in the `host` block of `substrate world doctor --json`) and the socket-activation summary in
`substrate --shim-status-json`. It automates the manual instructions already listed in `docs/WORLD.md`
(step-by-step systemctl checks, doctor, shim status) and captures the results under
`artifacts/linux/` for audit trails.

## Requirements

- Linux host with sudo access and systemd (the script installs/starts
  `substrate-world-agent.service` + `.socket`).
- Rust toolchain installed so `cargo build -p substrate --bin substrate` succeeds.
- `jq` available on PATH to slice the doctor JSON.
- Ability to tolerate `/usr/local/bin/substrate-world-agent` and systemd unit changes during the test.

## Running the harness

From the repository root:

```bash
scripts/linux/world-socket-verify.sh \
  --profile release \
  # optionally: --skip-cleanup if you want the units left enabled afterward
```

The script performs the following:

1. Builds the local `substrate` CLI (debug profile) if needed.
2. Calls `scripts/linux/world-provision.sh --profile <profile>` to install the
   world-agent, write both systemd units, and enable the socket.
3. Records `systemctl status` output for the `.socket` and `.service` into
   `artifacts/linux/world-socket-verify-<timestamp>/`.
4. Captures `/run/substrate.sock` ownership/perms (`root:substrate 0660`),
   the invoking user's group memberships, and the current `loginctl` lingering
   state so the archive proves group/linger guidance was surfaced.
5. Runs `substrate world doctor --json` with the newly provisioned socket and writes
   the JSON and `host.world_socket` extract to the same artifact directory.
6. Runs `substrate --shim-status-json` and stores the output.
7. By default, executes `scripts/substrate/uninstall-substrate.sh` so the host returns to its
   pre-test state (use `--skip-cleanup` if you want to keep the units running).

Artifacts include:

- `systemctl-socket.txt` / `systemctl-service.txt`
- `world-doctor.json`
- `world-doctor-world_socket.json` (extracts `host.world_socket`; falls back to legacy keys when present)
- `shim-status.json`
- `run-substrate-socket.txt` (stat output showing owner/group/mode)
- `invoking-user-groups.txt`
- `loginctl-linger.txt`

Attach these files or summaries to the session log / PR when demonstrating socket-activation support on real hardware.

## Tips

- Provide `sudo` upfront (`sudo -v`) before running the script to avoid mid-run prompts.
- To inspect the raw doctor block quickly:
  ```bash
  jq '.' artifacts/linux/world-socket-verify-*/world-doctor-world_socket.json
  ```
- If you want the release binary for doctor/shim status, run `cargo build -p substrate --bin substrate --release`
and update the script invocation to point at `target/release/substrate` (or temporarily edit `SUBSTRATE_BIN`).
