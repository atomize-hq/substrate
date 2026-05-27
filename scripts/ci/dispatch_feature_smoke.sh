#!/usr/bin/env bash
set -euo pipefail

emit_kv() {
    local key="$1"
    local val="$2"
    printf '%s=%s\n' "$key" "$val"
}

emit_kv "DISPATCH_OK" "0"
emit_kv "HEAD" ""
emit_kv "TEMP_BRANCH" ""
emit_kv "RUN_ID" ""
emit_kv "RUN_URL" ""
emit_kv "CONCLUSION" "retired"
emit_kv "SMOKE_SLICE_ID" ""
emit_kv "SMOKE_PASSED_PLATFORMS" ""
emit_kv "SMOKE_FAILED_PLATFORMS" ""
emit_kv "RUNNER_MISPROVISIONED" "0"
emit_kv "RUNNER_MISPROVISIONED_REASON" ""
emit_kv "ERROR_KIND" "retired"
emit_kv "ERROR_MESSAGE" "feature-smoke automation was retired with project-management pack automation"

echo "ERROR: feature-smoke automation was retired with project-management pack automation." >&2
echo "See docs/PROJECT_MANAGEMENT_RETIREMENT.md for the replacement workflow." >&2
exit 2
