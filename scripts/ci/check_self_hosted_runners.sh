#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/ci/check_self_hosted_runners.sh

Checks that the repo has self-hosted runners matching the expected label contract:
  - Native Linux: [self-hosted, Linux, linux-host]
  - Linux-in-WSL: [self-hosted, Linux, wsl]
  - macOS: [self-hosted, macOS]
  - Windows: [self-hosted, Windows]

Requirements:
  - gh (authenticated)
  - jq
USAGE
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    usage
    exit 0
fi

if ! command -v gh >/dev/null 2>&1; then
    echo "ERROR: gh not found on PATH" >&2
    exit 2
fi
if ! command -v jq >/dev/null 2>&1; then
    echo "ERROR: jq not found on PATH" >&2
    exit 2
fi

gh auth status >/dev/null

repo="$(gh repo view --json nameWithOwner -q '.nameWithOwner')"
echo "Repo: ${repo}"

json="$(gh api "repos/${repo}/actions/runners" --paginate)"

has_runner() {
    local expr="$1"
    jq -e "${expr}" >/dev/null <<<"${json}"
}

describe_match() {
    local expr="$1"
    jq -r "${expr}" <<<"${json}"
}

missing=0

check() {
    local label_desc="$1"
    local has_expr="$2"
    local list_expr="$3"

    if has_runner "${has_expr}"; then
        echo "OK: ${label_desc}"
        describe_match "${list_expr}" | sed 's/^/  - /'
    else
        echo "MISSING: ${label_desc}" >&2
        missing=1
    fi
}

check \
    "native Linux runner ([Linux, linux-host])" \
    '.runners | any(.labels | map(.name) | (index("Linux") != null) and (index("linux-host") != null))' \
    '.runners[] | select(.labels | map(.name) | (index("Linux") != null) and (index("linux-host") != null)) | [.name, (.labels|map(.name)|join(","))] | @tsv'

check \
    "Linux-in-WSL runner ([Linux, wsl])" \
    '.runners | any(.labels | map(.name) | (index("Linux") != null) and (index("wsl") != null))' \
    '.runners[] | select(.labels | map(.name) | (index("Linux") != null) and (index("wsl") != null)) | [.name, (.labels|map(.name)|join(","))] | @tsv'

check \
    "macOS runner ([macOS])" \
    '.runners | any(.labels | map(.name) | index("macOS") != null)' \
    '.runners[] | select(.labels | map(.name) | index("macOS") != null) | [.name, (.labels|map(.name)|join(","))] | @tsv'

check \
    "Windows runner ([Windows])" \
    '.runners | any(.labels | map(.name) | index("Windows") != null)' \
    '.runners[] | select(.labels | map(.name) | index("Windows") != null) | [.name, (.labels|map(.name)|join(","))] | @tsv'

if [[ "${missing}" -ne 0 ]]; then
    echo "" >&2
    echo "FAIL: missing one or more required self-hosted runners/labels" >&2
    exit 1
fi

echo "OK: all required self-hosted runners present"

