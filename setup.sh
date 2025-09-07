#!/bin/bash
  set -euo pipefail
  apt-get update
  apt-get install -y --no-install-recommends \
    build-essential pkg-config libseccomp-dev nftables conntrack jq curl ca-certificates
  update-ca-certificates

  # Sanity: show toolchain
  rustc --version
  cargo --version

  # Clean, build, and test (world tests best-effort)
  cargo clean
