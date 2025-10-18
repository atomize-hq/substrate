#!/usr/bin/env bash
set -euo pipefail

PROJECT_PATH=${1:-$(pwd)}
PROFILE=scripts/mac/lima/substrate.yaml
TMP_PROFILE=$(mktemp)

# Clean up temp file on exit
trap 'rm -f "$TMP_PROFILE"' EXIT

# Substitute PROJECT path in profile
PROJECT="$PROJECT_PATH" envsubst < "$PROFILE" > "$TMP_PROFILE"

# Check if VM already exists
if limactl list substrate >/dev/null 2>&1; then
    status=$(limactl list substrate --json | jq -r '.status // ""')
    if [ "$status" = "Running" ]; then
        echo "Lima VM 'substrate' is already running."
        exit 0
    else
        echo "Starting existing Lima VM 'substrate'..."
        limactl start substrate
    fi
else
    echo "Creating new Lima VM 'substrate' from profile..."
    limactl start --tty=false --name substrate "$TMP_PROFILE"
fi

# Wait until running
timeout=120
echo "Waiting for VM to reach Running state..."
while [ $timeout -gt 0 ]; do
    status=$(limactl list substrate --json | jq -r '.status // ""')
    if [ "$status" = "Running" ]; then
        echo "Lima VM 'substrate' is running."
        exit 0
    fi
    sleep 2
    timeout=$((timeout-2))
done

echo "ERROR: Lima VM did not reach Running state within 120 seconds" >&2
exit 1
