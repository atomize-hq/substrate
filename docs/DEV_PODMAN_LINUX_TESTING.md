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

Netfilter sanity (inside container)
- nft list tables || true
- dmesg -T | grep 'substrate-dropped-' | tail -n 5 || true

Codex “free roam” usage (inside container)
- codex -C /src --yolo
- Use the repo workspace at /src; approvals and sandbox are bypassed inside the container.

Common issues
- Cannot find -lseccomp during build: fixed by installing libseccomp-dev in the image (Dockerfile updated).
- Overlayfs/userns failures on macOS Docker Desktop: move to Podman rootful VM (this doc) or a Linux host/VM.
- Compose using Docker: our scripts avoid docker-compose on macOS to prevent auth/socket issues; use podman build/run.

