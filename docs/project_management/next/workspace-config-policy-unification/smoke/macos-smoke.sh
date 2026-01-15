#!/usr/bin/env bash
set -euo pipefail

# For this feature, macOS behavior is host-local and must match Linux semantics.
# This smoke script intentionally mirrors linux-smoke.sh.

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
bash "${script_dir}/linux-smoke.sh"
