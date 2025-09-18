#!/usr/bin/env bash
set -euo pipefail

scripts/mac/lima-warm.sh
substrate -c 'echo smoke-nonpty'
substrate --pty -c 'printf smoke-pty\n'
span=$(substrate -c 'bash -lc "mkdir -p /tmp/world-mac && echo data > /tmp/world-mac/file.txt"' --trace-id)
substrate --replay-verbose --replay "$span" | tee /tmp/world-mac-replay.json
jq '.fs_diff | map(.path)' /tmp/world-mac-replay.json | grep '/tmp/world-mac/file.txt'
