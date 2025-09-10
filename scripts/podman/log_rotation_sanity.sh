#!/usr/bin/env bash
set -euo pipefail

# This script sanity-checks writer-only rotation inside a privileged Podman
# dev container with the repo bind-mounted at /src.
#
# Usage:
#   ./scripts/podman/log_rotation_sanity.sh substrate-dev-ctl
#
# Where `substrate-dev-ctl` is the running container name.

CTR="${1:-substrate-dev-ctl}"

exec_in_ctr() {
  podman exec -it "$CTR" bash -lc "$*"
}

echo "== Podman rotation sanity for $CTR =="

exec_in_ctr 'export HOME=/root PATH=/usr/local/cargo/bin:$PATH; cd /src && cargo build -q'

CMD='export HOME=/root TRACE_LOG_MAX_MB=1 TRACE_LOG_KEEP=2; \
     rm -f /root/.substrate/trace.jsonl /root/.substrate/trace.jsonl.* || true; \
     for i in $(seq 1 4000); do /src/target/debug/substrate -c true >/dev/null 2>&1 || true; done; \
     ls -l /root/.substrate | grep trace.jsonl || true; \
     for i in $(seq 1 4000); do /src/target/debug/substrate -c true >/dev/null 2>&1 || true; done; \
     ls -l /root/.substrate | grep trace.jsonl || true; \
     for i in $(seq 1 4000); do /src/target/debug/substrate -c true >/dev/null 2>&1 || true; done; \
     ls -l /root/.substrate | grep trace.jsonl || true; \
     if command -v stat >/dev/null 2>&1; then \
       stat -c "%n %s" /root/.substrate/trace.jsonl /root/.substrate/trace.jsonl.1 /root/.substrate/trace.jsonl.2 2>/dev/null || true; \
     fi; \
     if [ -f /root/.substrate/trace.jsonl.3 ]; then echo "UNEXPECTED: .3 exists"; else echo ".3 absent (ok)"; fi'

exec_in_ctr "$CMD"

echo "== Done =="
