#!/usr/bin/env bash
set -euo pipefail
echo "== E2E smoke: cargo test -p triad_e2e_smoke_demo =="
cargo test -p triad_e2e_smoke_demo
