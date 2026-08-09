#!/bin/bash
set -euo pipefail

SCRIPT_NAME="dev-install-code-requirement-r3"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INSTALLER_PATH="${REPO_ROOT}/scripts/substrate/dev-install-substrate.sh"
BASE_COMMIT="3bd9979485e30d2da2e558d95d7cdc0737ec24f4"

fail() {
  printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2
  exit 1
}

if [[ "$(uname -s)" != "Darwin" ]]; then
  printf '[%s] SKIP: macOS code-requirement fixture\n' "${SCRIPT_NAME}"
  exit 0
fi

MODE="green"
if [[ "${1-}" == "--expect-base-failure" ]]; then
  MODE="base-red"
  shift
fi
[[ $# -eq 0 ]] || fail "usage: ${0##*/} [--expect-base-failure]"

for tool in /usr/bin/codesign /usr/bin/lipo /usr/bin/python3; do
  [[ -x "${tool}" ]] || fail "required macOS fixture tool is absent: ${tool}"
done
[[ -f "${INSTALLER_PATH}" && ! -L "${INSTALLER_PATH}" ]] \
  || fail "canonical installer is absent or linked"

WORK_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/substrate-code-requirement-r3.XXXXXX")"
cleanup() {
  rm -rf -- "${WORK_ROOT}"
}
trap cleanup EXIT

SOURCE_PATH="${INSTALLER_PATH}"
if [[ "${MODE}" == "base-red" ]]; then
  SOURCE_PATH="${WORK_ROOT}/base-dev-install-substrate.sh"
  git -C "${REPO_ROOT}" show "${BASE_COMMIT}:scripts/substrate/dev-install-substrate.sh" >"${SOURCE_PATH}" \
    || fail "cannot read exact base installer ${BASE_COMMIT}"
  git -C "${REPO_ROOT}" diff --quiet "${BASE_COMMIT}" -- crates/common/src/managed_artifact.rs \
    || fail "bootstrap schema validator drifted from the exact base"
fi

make_adhoc_fixture() {
  local source_path="$1"
  local output_path="$2"
  local identifier="$3"
  local host_arch
  local fixture_arch
  local archs

  host_arch="$(uname -m)"
  archs="$(/usr/bin/lipo -archs "${source_path}")"
  case " ${archs} " in
    *" ${host_arch} "*) fixture_arch="${host_arch}" ;;
    *) fixture_arch="${archs%% *}" ;;
  esac
  /usr/bin/lipo "${source_path}" -thin "${fixture_arch}" -output "${output_path}" \
    || fail "cannot create thin fixture image ${identifier}"
  chmod 0755 "${output_path}"
  /usr/bin/codesign --force --sign - --identifier "${identifier}" -- "${output_path}" \
    >/dev/null 2>"${output_path}.sign.err" \
    || fail "cannot ad-hoc sign fixture image ${identifier}"
  [[ -z "$(/usr/bin/codesign -d -r- -- "${output_path}" 2>&1 | sed -n 's/^designated => //p' | head -n 1)" ]] \
    || fail "ad-hoc fixture unexpectedly emitted a designated requirement: ${identifier}"
}

CONTROL_IMAGE="${WORK_ROOT}/control-image"
EXECUTOR_IMAGE="${WORK_ROOT}/executor-image"
LIMA_IMAGE="${WORK_ROOT}/limactl-image"
MISMATCH_IMAGE="${WORK_ROOT}/mismatch-image"
make_adhoc_fixture /bin/echo "${CONTROL_IMAGE}" "fixture.substrate.control"
make_adhoc_fixture /bin/cat "${EXECUTOR_IMAGE}" "fixture.substrate.executor"
make_adhoc_fixture /usr/bin/true "${LIMA_IMAGE}" "fixture.substrate.limactl"
make_adhoc_fixture /usr/bin/false "${MISMATCH_IMAGE}" "fixture.substrate.mismatch"

/usr/bin/python3 - "${SOURCE_PATH}" "${WORK_ROOT}/helpers.sh" "${WORK_ROOT}/producer.sh" <<'PY'
from pathlib import Path
import sys

source = Path(sys.argv[1]).read_text(encoding="utf-8")
helper_start = source.find("cdhash() {")
assignment_start = source.find('control_cdhash="$(cdhash "$control_src")"', helper_start)
assignment_end = source.find('\n', source.find('lima_requirement=', assignment_start))
if min(helper_start, assignment_start, assignment_end) < 0:
    raise SystemExit("canonical privileged requirement producer is absent")
Path(sys.argv[2]).write_text(source[helper_start:assignment_start], encoding="utf-8")
Path(sys.argv[3]).write_text(source[helper_start:assignment_end + 1], encoding="utf-8")
PY

cat >"${WORK_ROOT}/run-producer.sh" <<'SH'
#!/bin/sh
set -eu
control_src="$1"
executor_path="$2"
limactl_path="$3"
producer="$4"
. "$producer"
printf 'control_cdhash=%s\n' "$control_cdhash"
printf 'executor_cdhash=%s\n' "$executor_cdhash"
printf 'lima_cdhash=%s\n' "$lima_cdhash"
printf 'control_requirement=%s\n' "$control_requirement"
printf 'executor_requirement=%s\n' "$executor_requirement"
printf 'lima_requirement=%s\n' "$lima_requirement"
SH
chmod 0755 "${WORK_ROOT}/run-producer.sh"

PRODUCER_OUTPUT="$(PATH=/usr/bin:/bin:/usr/sbin:/sbin /bin/sh "${WORK_ROOT}/run-producer.sh" \
  "${CONTROL_IMAGE}" "${EXECUTOR_IMAGE}" "${LIMA_IMAGE}" "${WORK_ROOT}/producer.sh")" \
  || fail "actual privileged provenance requirement producer failed"
printf '%s\n' "${PRODUCER_OUTPUT}" >"${WORK_ROOT}/producer.out"

field() {
  sed -n "s/^$1=//p" "${WORK_ROOT}/producer.out" | head -n 1
}

CONTROL_CDHASH="$(field control_cdhash)"
EXECUTOR_CDHASH="$(field executor_cdhash)"
LIMA_CDHASH="$(field lima_cdhash)"
CONTROL_REQUIREMENT="$(field control_requirement)"
EXECUTOR_REQUIREMENT="$(field executor_requirement)"
LIMA_REQUIREMENT="$(field lima_requirement)"

for value in "${CONTROL_CDHASH}" "${EXECUTOR_CDHASH}" "${LIMA_CDHASH}"; do
  [[ "${value}" =~ ^[0-9a-f]{40}$ ]] || fail "producer emitted a non-canonical measured CDHash"
done

cat >"${WORK_ROOT}/Cargo.toml" <<EOF
[package]
name = "mac-code-requirement-schema-fixture"
version = "0.0.0"
edition = "2021"

[dependencies]
serde_json = "1.0"
substrate-common = { path = "${REPO_ROOT}/crates/common" }
EOF
mkdir -p "${WORK_ROOT}/src"
cat >"${WORK_ROOT}/src/main.rs" <<'RS'
use std::{env, fs};

use substrate_common::managed_artifact::{
    canonical_mac_publisher_install_provenance_v1,
    validate_mac_publisher_install_provenance_v1, MacPublisherInstallProvenanceV1,
};

fn main() {
    let path = env::args().nth(1).expect("provenance path");
    let input = fs::read(path).expect("read provenance fixture");
    let provenance: MacPublisherInstallProvenanceV1 =
        serde_json::from_slice(&input).expect("decode provenance fixture");
    if let Err(error) = validate_mac_publisher_install_provenance_v1(&provenance)
        .and_then(|_| canonical_mac_publisher_install_provenance_v1(&provenance).map(|_| ()))
    {
        eprintln!("{error:#}");
        std::process::exit(2);
    }
}
RS

/usr/bin/python3 - "${WORK_ROOT}/provenance.json" \
  "${CONTROL_IMAGE}" "${EXECUTOR_IMAGE}" "${LIMA_IMAGE}" \
  "${CONTROL_CDHASH}" "${EXECUTOR_CDHASH}" "${LIMA_CDHASH}" \
  "${CONTROL_REQUIREMENT}" "${EXECUTOR_REQUIREMENT}" "${LIMA_REQUIREMENT}" <<'PY'
from hashlib import sha256
import json
from pathlib import Path
import sys

(
    output, control_path, executor_path, lima_path,
    control_cdhash, executor_cdhash, lima_cdhash,
    control_requirement, executor_requirement, lima_requirement,
) = sys.argv[1:]

def digest(path):
    return sha256(Path(path).read_bytes()).hexdigest()

def identity(path):
    stat = Path(path).stat()
    return f"dev:{stat.st_dev}:ino:{stat.st_ino}"

def image(path, cdhash, requirement):
    return {
        "target_triple": "aarch64-apple-darwin",
        "artifact_sha256": digest(path),
        "physical_identity": identity(path),
        "code_identity": "cdhash:" + cdhash,
        "code_requirement": requirement,
    }

build_command = (
    "cargo build --locked --offline --target aarch64-unknown-linux-gnu --release "
    "-p substrate --bin substrate-lifecycle-linux -p world-service --bin world-service "
    "-p substrate-gateway --bin substrate-gateway -p substrate --bin substrate"
)
artifact_specs = (
    ("mac.lima.publisher-executor", "bin/linux/substrate-lifecycle-linux", "substrate", "substrate-lifecycle-linux"),
    ("mac.lima.guest-binary(substrate-world-service)", "bin/linux/world-service", "world-service", "world-service"),
    ("mac.lima.guest-binary(substrate-gateway)", "bin/linux/substrate-gateway", "substrate-gateway", "substrate-gateway"),
    ("mac.lima.guest-binary(substrate)", "bin/linux/substrate", "substrate", "substrate"),
)
retained = []
for index, (role, relative_path, package, binary) in enumerate(artifact_specs, start=1):
    retained.append({
        "logical_role": role,
        "retained_relative_path": relative_path,
        "cargo_package": package,
        "cargo_binary": binary,
        "target_triple": "aarch64-unknown-linux-gnu",
        "cargo_lock_sha256": "1" * 64,
        "toolchain": "rustc 1.89.0 (fixture)",
        "build_command": build_command,
        "artifact_sha256": format(index, "x") * 64,
        "physical_identity": f"dev:9:ino:{index}",
        "mode": "0755",
    })

record = {
    "schema_owner": "substrate.mac-publisher-install-provenance",
    "schema_version": 2,
    "source_commit": "a" * 40,
    "source_tree": "b" * 40,
    "source_ref": "refs/heads/fixture",
    "review_record_sha256": "c" * 64,
    "host_context_commitment": "d" * 64,
    "selected_host_prefix": "/tmp/substrate-fixture",
    "control_authority": {
        "schema_owner": "substrate.mac-publisher-control-authority",
        "schema_version": 1,
        "control_binary": "substrate-lifecycle-control",
        "source_commit": "a" * 40,
        "source_tree": "b" * 40,
        "source_ref": "refs/heads/fixture",
        "target_triple": "aarch64-apple-darwin",
        "artifact_sha256": digest(control_path),
        "designated_requirement": control_requirement,
    },
    "control_image": image(control_path, control_cdhash, control_requirement),
    "executor_image": image(executor_path, executor_cdhash, executor_requirement),
    "launch_daemon_plist_sha256": "e" * 64,
    "lima_tool": {
        "absolute_path": lima_path,
        "image": image(lima_path, lima_cdhash, lima_requirement),
        "version": "limactl version fixture",
    },
    "retained_linux_artifacts": retained,
    "profile_template_algorithm": "substrate.mac-lima-stage-one-profile-template",
    "profile_template_version": 1,
    "profile_template_sha256": "f" * 64,
}
Path(output).write_text(json.dumps(record, sort_keys=True, separators=(",", ":")), encoding="utf-8")
PY

if [[ "${MODE}" == "base-red" ]]; then
  [[ -n "${CONTROL_REQUIREMENT}" ]] || fail "exact base lost the control requirement"
  [[ -z "${EXECUTOR_REQUIREMENT}" && -z "${LIMA_REQUIREMENT}" ]] \
    || fail "exact base unexpectedly produced executor/limactl requirements"
  if CARGO_TARGET_DIR="${WORK_ROOT}/target" cargo run --offline --quiet \
    --manifest-path "${WORK_ROOT}/Cargo.toml" -- "${WORK_ROOT}/provenance.json" \
    >"${WORK_ROOT}/schema.out" 2>"${WORK_ROOT}/schema.err"; then
    fail "exact-base empty requirements unexpectedly passed bootstrap schema validation"
  fi
  grep -Fq -- 'macOS bootstrap image code_requirement must be non-empty' "${WORK_ROOT}/schema.err" \
    || fail "exact-base schema rejection did not report the non-empty requirement invariant"
  printf '[%s] PASS: exact base dynamically reproduced empty executor/limactl requirements and bootstrap rejection\n' "${SCRIPT_NAME}"
  exit 0
fi

[[ "${EXECUTOR_REQUIREMENT}" =~ ^cdhash\ H\"[0-9a-f]{40}\"$ ]] \
  || fail "executor did not receive the canonical CDHash fallback requirement"
[[ "${LIMA_REQUIREMENT}" =~ ^cdhash\ H\"[0-9a-f]{40}\"$ ]] \
  || fail "limactl did not receive the canonical CDHash fallback requirement"
/usr/bin/codesign --verify --strict "-R=${EXECUTOR_REQUIREMENT}" -- "${EXECUTOR_IMAGE}" \
  || fail "executor fallback requirement does not match the exact fixture image"
/usr/bin/codesign --verify --strict "-R=${LIMA_REQUIREMENT}" -- "${LIMA_IMAGE}" \
  || fail "limactl fallback requirement does not match the exact fixture image"

cat >"${WORK_ROOT}/exercise-helper.sh" <<'SH'
#!/bin/sh
set -eu
helpers="$1"
operation="$2"
image="$3"
measured_cdhash="$4"
. "$helpers"
case "$operation" in
  canonical) canonical_code_requirement "$image" "$measured_cdhash" ;;
  *) exit 64 ;;
esac
SH
chmod 0755 "${WORK_ROOT}/exercise-helper.sh"

if PATH=/usr/bin:/bin:/usr/sbin:/sbin /bin/sh "${WORK_ROOT}/exercise-helper.sh" \
  "${WORK_ROOT}/helpers.sh" canonical "${MISMATCH_IMAGE}" "${EXECUTOR_CDHASH}" \
  >"${WORK_ROOT}/mismatch.out" 2>"${WORK_ROOT}/mismatch.err"; then
  fail "mismatched CDHash/file pair unexpectedly produced a requirement"
fi

EXPLICIT_REQUIREMENT="$(/usr/bin/codesign -d -r- -- /bin/echo 2>&1 \
  | sed -n 's/^designated => //p' | head -n 1)"
[[ -n "${EXPLICIT_REQUIREMENT}" ]] || fail "explicit designated-requirement fixture is absent"
EXPLICIT_CDHASH="$(/usr/bin/codesign -d -vvv -- /bin/echo 2>&1 \
  | sed -n 's/^CDHash=//p' | head -n 1)"
ACTUAL_EXPLICIT="$(PATH=/usr/bin:/bin:/usr/sbin:/sbin /bin/sh "${WORK_ROOT}/exercise-helper.sh" \
  "${WORK_ROOT}/helpers.sh" canonical /bin/echo "${EXPLICIT_CDHASH}")" \
  || fail "explicit designated requirement did not survive producer validation"
[[ "${ACTUAL_EXPLICIT}" == "${EXPLICIT_REQUIREMENT}" ]] \
  || fail "producer did not preserve the explicit designated requirement exactly"
/usr/bin/codesign --verify --strict "-R=${ACTUAL_EXPLICIT}" -- /bin/echo \
  || fail "preserved designated requirement does not match its exact image"

CARGO_TARGET_DIR="${WORK_ROOT}/target" cargo run --offline --quiet \
  --manifest-path "${WORK_ROOT}/Cargo.toml" -- "${WORK_ROOT}/provenance.json" \
  || fail "generated non-empty requirements failed bootstrap schema acceptance"

printf '[%s] PASS: fallback requirements are image-bound, mismatches fail closed, explicit requirements are preserved, and bootstrap schema accepts the record\n' "${SCRIPT_NAME}"
