#!/usr/bin/env bash
set -euo pipefail

# Exit codes:
# - 0: success or intentional skip
# - 2: invalid inputs
# - 3: missing local prerequisites

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "SKIP: LAITDP macOS smoke is supported only on macOS"
  exit 0
fi

slice_id="${SUBSTRATE_SMOKE_SLICE_ID:-LAITDP1}"
case "$slice_id" in
  LAITDP1 | LAITDP2) ;;
  *)
    echo "FAIL: unsupported SUBSTRATE_SMOKE_SLICE_ID=$slice_id (expected LAITDP1 or LAITDP2)" >&2
    exit 2
    ;;
esac

need_cmd() {
  local name="$1"
  if ! command -v "$name" >/dev/null 2>&1; then
    echo "FAIL: required command not found: $name" >&2
    exit 3
  fi
}

need_cmd cargo

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -n "${SUBSTRATE_SMOKE_REPO_ROOT:-}" ]]; then
  repo_root="${SUBSTRATE_SMOKE_REPO_ROOT}"
elif [[ -n "${GITHUB_WORKSPACE:-}" && -d "${GITHUB_WORKSPACE}/candidate" ]]; then
  repo_root="${GITHUB_WORKSPACE}/candidate"
elif [[ -n "${GITHUB_WORKSPACE:-}" ]]; then
  repo_root="${GITHUB_WORKSPACE}"
else
  repo_root="$(cd "$script_dir/../../../.." && pwd)"
fi

run_test() {
  local test_name="$1"
  echo "INFO: running $test_name"
  (
    cd "$repo_root"
    cargo test -p shell --test world_gateway "$test_name" -- --exact --nocapture
  )
}

run_test "world_gateway_status_json_publishes_tuple_and_posture_as_top_level_siblings"
run_test "world_gateway_status_json_preserves_unavailable_shape_from_runtime"
run_test "world_gateway_status_json_keeps_tuple_metadata_when_runtime_is_unavailable"
run_test "world_gateway_status_human_output_uses_contract_label_order"
run_test "world_gateway_status_human_output_omits_missing_optional_fields_without_placeholders"

if [[ "$slice_id" == "LAITDP2" ]]; then
  echo "INFO: running transport-api-types LAITDP2 parity tests"
  (
    cd "$repo_root"
    cargo test -p transport-api-types laitdp2_ -- --nocapture
  )
  echo "INFO: running gateway integrated auth validation tests"
  (
    cd "$repo_root"
    cargo test -p transport-api-types gateway_integrated_auth_validation -- --nocapture
  )
fi

echo "OK: $slice_id macOS smoke"
