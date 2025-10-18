#!/usr/bin/env bash
set -euo pipefail

if limactl list substrate >/dev/null 2>&1; then
    status=$(limactl list substrate --json | jq -r '.status // ""')
    if [ "$status" = "Running" ]; then
        echo "Stopping Lima VM 'substrate'..."
        limactl stop substrate
        echo "Lima VM 'substrate' stopped."
    else
        echo "Lima VM 'substrate' is not running (status: $status)."
    fi
else
    echo "Lima VM 'substrate' not defined."
fi