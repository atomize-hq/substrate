#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
wrapper_source="${repo_root}/scripts/substrate/install.sh"

log() {
  printf '[install-wrapper-smoke] %s\n' "$*" >&2
}

assert_eq() {
  local actual="$1"
  local expected="$2"
  local label="$3"

  if [[ "${actual}" != "${expected}" ]]; then
    printf '[install-wrapper-smoke] expected %s=%q, got %q\n' "${label}" "${expected}" "${actual}" >&2
    exit 1
  fi
}

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"

  if [[ "${haystack}" != *"${needle}"* ]]; then
    printf '[install-wrapper-smoke] expected %s to contain %q\n' "${label}" "${needle}" >&2
    exit 1
  fi
}

tmpdir="$(mktemp -d -t substrate-install-wrapper.XXXXXX)"
trap 'rm -rf "${tmpdir}"' EXIT

fixture_root="${tmpdir}/fixture"
prefix_root="${tmpdir}/prefix"
home_root="${tmpdir}/home"
mkdir -p "${fixture_root}/loader" "${prefix_root}" "${home_root}"

cp "${wrapper_source}" "${fixture_root}/install.sh"
chmod +x "${fixture_root}/install.sh"

cat >"${fixture_root}/install-substrate.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
status="${SUBSTRATE_TEST_UPSTREAM_STATUS:-0}"
printf 'upstream-status=%s\n' "${status}" >&2
exit "${status}"
EOF
chmod +x "${fixture_root}/install-substrate.sh"

cat >"${fixture_root}/loader/bash_loading_animations.sh" <<'EOF'
#!/usr/bin/env bash
BLA_braille_fill_bar=()
BLA::start_loading_animation() { :; }
BLA::stop_loading_animation() { :; }
EOF

run_case() {
  local expected_status="$1"
  local output=""
  local actual_status=0

  set +e
  output="$(
    HOME="${home_root}" \
    SHELL="/bin/bash" \
    SUBSTRATE_TEST_UPSTREAM_STATUS="${expected_status}" \
    "${fixture_root}/install.sh" --prefix "${prefix_root}" 2>&1
  )"
  actual_status=$?
  set -e

  assert_eq "${actual_status}" "${expected_status}" "wrapper exit status ${expected_status}"
  if [[ "${expected_status}" -eq 0 ]]; then
    assert_contains "${output}" "Substrate install successful!" "success output"
  else
    assert_contains "${output}" "[substrate-install] Failed." "failure prefix ${expected_status}"
    assert_contains "${output}" "upstream-status=${expected_status}" "upstream stderr ${expected_status}"
  fi
}

for status in 0 2 3 4; do
  run_case "${status}"
done

log "verified wrapper exit pass-through for statuses 0, 2, 3, and 4"
