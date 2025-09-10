Title: Podman-based Linux Dev/Testing Environment (macOS host)

Overview
- Runs a privileged Debian-based Rust image with Codex preinstalled and substrate tooling.
- Uses Podman VM (Fedora CoreOS) with rootful mode to enable overlayfs, nftables, cgroups v2, and dmesg access.
- Reuses your host Codex credentials by mounting `${HOME}/.codex` into the container.

Prerequisites (macOS)
- Podman installed (brew install podman)
- Rust project in this repo at /src inside the container (we mount the repo folder)

One-time VM setup (host)
1) Set rootful and restart VM
   - podman machine set --rootful
   - podman machine stop; podman machine start
2) Select root connection
   - podman system connection default podman-machine-default-root
   - podman info (verify serviceIsRemote: true, rootless: false)
3) Configure kernel prerequisites inside VM
   - bash scripts/podman/setup-machine.sh
   - This enables: overlay module, nftables modules, cgroup v2 (default), dmesg read, and attempts userns clone.

Build image (host)
- Default (native arch):
  - bash scripts/podman/build.sh
- Force x86_64 (optional):
  - BUILD_ARCH=amd64 bash scripts/podman/build.sh

Run dev shell (host)
- bash scripts/podman/run.sh
- Inside container:
  - bash scripts/check-container-prereqs.sh
  - codex whoami
  - codex -C /src --dangerously-bypass-approvals-and-sandbox

Browser login (only if needed)
- We mount ${HOME}/.codex, so this is usually not required.
- If you must run codex login inside the container:
  1) Terminal A (host): bash scripts/podman/login-host-browser.sh 1455
  2) Terminal B (host): bash scripts/podman/run-hostnet.sh
  3) Inside container: codex login
  4) Open printed URL in macOS browser; tokens persist in ~/.codex

Validate features (inside container)
- cargo build
- RUST_LOG=info cargo test -p world -- --nocapture test_nftables_rules
- Replay with world isolation (write span example):
  - mkdir -p /tmp/pretest && cd /tmp/pretest
  - target/debug/substrate -c "bash -lc 'mkdir demo && echo data > demo/file.txt'"
  - span_id=$(tail -n 50 /root/.substrate/trace.jsonl | jq -r 'select(.event_type=="command_complete") | .span_id' | tail -n1)
  - export SUBSTRATE_REPLAY_USE_WORLD=1
  - target/debug/substrate --replay "$span_id"
- Expect fs_diff to include demo/ and demo/file.txt if overlayfs-in-userns is active

Per-world cgroups and nftables (Phase B additions)
- cgroups attach (privileged): during replay, a per-world cgroup appears at `/sys/fs/cgroup/substrate/<span_id>`.
  - Quick check in a second shell while replay is running:
    - `grep -H . /sys/fs/cgroup/substrate/*/cgroup.procs || true`
    - Expect the world’s cgroup `cgroup.procs` to contain at least one PID while the command runs.
- nftables LOG: a per-world inet table is installed with a default LOG+drop policy for egress (loopback and established traffic allowed).
  - Installed inside a per-replay network namespace when available (`ip netns add substrate-<span>`), so host rules remain untouched.
  - Ensure `kernel.dmesg_restrict=0` to see LOG lines: `sysctl -w kernel.dmesg_restrict=0` (already configured by setup script).
  - Run a replayed curl to an external host to generate a LOG:
    - `target/debug/substrate -c "bash -lc 'curl -m2 http://example.com || true'"`
    - Capture its span id and replay with `SUBSTRATE_REPLAY_USE_WORLD=1`.
    - Check: `dmesg -T | grep substrate-dropped- | tail -n 5` and expect entries with the per-world prefix.
  - On constrained kernels or missing nft, replay prints: `[replay] warn: nft not available; netfilter scoping/logging disabled` and continues.
  - On missing netns privileges, replay prints: `[replay] warn: netns unavailable or insufficient privileges; applying host-wide rules or skipping network scoping`.

Expected warnings (non-root/limited hosts)
- `[replay] warn: cgroup v2 unavailable or insufficient privileges; skipping cgroup attach`
- `[replay] warn: nft not available; netfilter scoping/logging disabled`
- `[replay] warn: kernel.dmesg_restrict=1; LOG lines may not be visible`

Netfilter sanity (inside container)
- nft list tables || true  # host should remain empty for substrate tables
- For a running replay: `ip netns list | grep substrate-<span>` then `ip netns exec substrate-<span> nft list tables` (should show the per-world table)
- dmesg -T | grep 'substrate-dropped-' | tail -n 5 || true
  - Note: In the default netns setup only loopback is up, so egress often fails before hitting the nft output hook. To guarantee LOGs, bring up a veth pair inside the netns and route traffic through it.

Codex “free roam” usage (inside container)
- codex -C /src --yolo
- Use the repo workspace at /src; approvals and sandbox are bypassed inside the container.

Common issues
- Cannot find -lseccomp during build: fixed by installing libseccomp-dev in the image (Dockerfile updated).
- Overlayfs/userns failures on macOS Docker Desktop: move to Podman rootful VM (this doc) or a Linux host/VM.
- Compose using Docker: our scripts avoid docker-compose on macOS to prevent auth/socket issues; use podman build/run.
