#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INSTALLER="${REPO_ROOT}/scripts/substrate/dev-install-substrate.sh"
WARM="${REPO_ROOT}/scripts/mac/lima-warm.sh"
LIFECYCLE="${REPO_ROOT}/scripts/mac/lima-lifecycle.sh"
COMMON="${REPO_ROOT}/crates/common/src/managed_artifact.rs"
EXECUTOR="${REPO_ROOT}/src/bin/substrate-lifecycle-macos.rs"
CONTRACTS="${REPO_ROOT}/llm-last-mile/runtime-refactor/04-contracts-and-gates.md"

WORK_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/substrate-prestage-artifact-route.XXXXXX")"
cleanup() {
    rm -rf -- "${WORK_ROOT}"
}
trap cleanup EXIT

FAKE_ZIG="${WORK_ROOT}/zig"
CAPTURE="${WORK_ROOT}/zig-args"
GENERATOR="${WORK_ROOT}/generate-linker-wrapper.sh"
LINKER_WRAPPER="${WORK_ROOT}/aarch64-linux-gnu-zig-cc"

cat > "${FAKE_ZIG}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\0' "$@" > "${CAPTURE:?}"
EOF
chmod 0700 "${FAKE_ZIG}"

cat > "${GENERATOR}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
zig="$1"
linker_wrapper="$2"
EOF
python3 - "${INSTALLER}" >> "${GENERATOR}" <<'PY'
from pathlib import Path
import sys

source = Path(sys.argv[1]).read_text(encoding="utf-8")
start = source.index('  cat > "${linker_wrapper}" <<EOF\n')
end = source.index('\nEOF\n', start) + len('\nEOF\n')
sys.stdout.write(source[start:end])
PY
chmod 0700 "${GENERATOR}"
"${GENERATOR}" "${FAKE_ZIG}" "${LINKER_WRAPPER}"
chmod 0700 "${LINKER_WRAPPER}"

CAPTURE="${CAPTURE}" "${LINKER_WRAPPER}" \
    --target=aarch64-unknown-linux-gnu \
    "source file.c" \
    --target=aarch64-unknown-linux-gnu-extra \
    --target aarch64-unknown-linux-gnu \
    -L \
    "/tmp/target/release/build/openssl-sys/out/openssl-build/install/lib" \
    "-Wl,-rpath,/tmp/path with spaces" \
    ""

python3 - "${CAPTURE}" <<'PY'
from pathlib import Path
import sys

captured = Path(sys.argv[1]).read_bytes().split(b"\0")
if captured[-1:] == [b""]:
    captured.pop()
expected = [
    b"cc",
    b"-target",
    b"aarch64-linux-gnu",
    b"source file.c",
    b"--target=aarch64-unknown-linux-gnu-extra",
    b"--target",
    b"aarch64-unknown-linux-gnu",
    b"-L",
    b"/tmp/target/release/build/openssl-sys/out/openssl-build/install/lib",
    b"-Wl,-rpath,/tmp/path with spaces",
    b"",
    b"/tmp/target/release/build/openssl-sys/out/openssl-build/install/lib/libssl.a",
    b"/tmp/target/release/build/openssl-sys/out/openssl-build/install/lib/libcrypto.a",
]
if captured != expected:
    raise SystemExit(f"generated Zig wrapper forwarded unexpected arguments: {captured!r}")
PY

if [[ ! -x "${LIFECYCLE}" ]]; then
    echo "lima lifecycle dispatcher must remain executable for lima-warm and lima-stop" >&2
    exit 1
fi

python3 - "${INSTALLER}" "${WARM}" "${LIFECYCLE}" "${COMMON}" "${EXECUTOR}" "${CONTRACTS}" <<'PY'
from pathlib import Path
import re
import sys

installer, warm, lifecycle, common, executor, contracts = [Path(path).read_text() for path in sys.argv[1:]]

def require(label, text, needle):
    if needle not in text:
        raise SystemExit(f"{label} is missing required closed prestage route fragment: {needle}")

def forbid(label, text, needle):
    if needle in text:
        raise SystemExit(f"{label} retains unsupported world artifact role or path: {needle}")

for text in (
    "build_and_stage_mac_aarch64_lima_artifacts_v1()",
    "aarch64-unknown-linux-gnu",
    "--locked",
    "--offline",
    "substrate-lifecycle-linux",
    "world-service",
    "substrate-gateway",
    "--bin substrate",
    "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER",
    "aarch64-linux-gnu-zig-cc",
    'mktemp -d "/private/tmp/substrate-mac-aarch64-build.XXXXXX"',
    "retained Linux bundle contains a mismatched prior",
    "mv \"${artifact_stage}\" \"${bundle_dir}\"",
    "adopt_retained_linux_artifacts_transactionally()",
    "os.O_NOFOLLOW",
    "os.fchown(fd, 0, 0)",
    "os.fchmod(fd, 0o755)",
    "rollback_error",
    "rollback verification mismatch",
    "leaf.st_dev != after.st_dev or leaf.st_ino != after.st_ino",
    "retained_linux_artifacts",
    "publisher-bootstrap",
    "--lima-stage-one-authorization-v1 \"${stage_one_authorization}\"",
    "direct publisher-bootstrap returned an invalid Stage-1 result",
):
    require("installer", installer, text)
adoption = installer[
    installer.index("adopt_retained_linux_artifacts_transactionally()"):
    installer.index("adopt_retained_linux_artifacts_transactionally\n", installer.index("adopt_retained_linux_artifacts_transactionally()"))
]
if adoption.index("mutated.append((path, fd, before))") > adoption.index("os.fchown(fd, 0, 0)"):
    raise SystemExit("transactional adoption records rollback state after its first mutation")

forbidden_selector = re.compile(r'--(?:artifact|role|path)(?:=|\b)')
if forbidden_selector.search(installer):
    raise SystemExit("installer exposes a caller-selected artifact, role, or path selector")

for text in (
    "validate_stage_one_completion_response_v1()",
    "post_pm_requests_v1 remains a catalogue",
    "fixed installation sequence",
    "The privileged executor owns",
    "emit_r6_pairing_continuation_v1()",
    '"auto_run": False',
):
    require("lima-warm", warm, text)
for selector in ("--publisher-request-v1)", "--platform-bootstrap-mapping-v1)", "--executor-build-evidence-v1)"):
    if selector in warm:
        raise SystemExit("lima-warm still accepts a caller-selected post-PM input")
ensure = re.search(r'(?ms)^ensure_vm_ready\(\) \{.*?(?=^[A-Za-z_][A-Za-z0-9_]*\(\) \{|\Z)', warm)
if not ensure or "post_pm_action" in ensure.group(0):
    raise SystemExit("lima-warm can still execute an ordinary post-PM request")

for text in (
    "stage_one_create",
    "post_pm_action",
    "ordinary closed post-PM",
    'stage_value["successor_template"]["executor_build_evidence"]',
):
    require("lima-lifecycle", lifecycle, text)
if "stage_one_create|post_pm_action)" not in lifecycle:
    raise SystemExit("lima-lifecycle no longer admits the existing ordinary post-PM branch")
if "mac.lima.guest-binary(world)" in lifecycle or "post_pm_requests_v1" in lifecycle:
    raise SystemExit("lima-lifecycle admits the removed role or catalogue traversal")
if "${TMPDIR:-/tmp}/substrate-mac-aarch64-build" in installer:
    raise SystemExit("installer still accepts a caller-selected AArch64 build root")

for text in (
    "pub struct MacLimaRetainedArtifactProvenanceV1",
    "retained_linux_artifacts",
    "validate_mac_lima_retained_artifact_provenance_v1",
    '"substrate-world-service",',
    '"substrate-gateway",',
    '"substrate",',
):
    require("managed artifact model", common, text)

for text in (
    "validate_mac_lima_retained_artifacts_against_provenance_v1",
    "mac_fixed_install_steps_v1",
    "mac_execute_fixed_install_sequence_v1",
    "execute_mac_post_pm_action_with_policy_v1(executor, &control, true)",
    "fixed install receipt index is not the exact plan prefix",
    "mac.lima.guest-binary(substrate-world-service)",
    "mac.lima.guest-binary(substrate-gateway)",
    "mac.lima.guest-binary(substrate)",
):
    require("macOS lifecycle executor", executor, text)

def function_body(text, name):
    match = re.search(rf'(?ms)^fn {re.escape(name)}\(.*?(?=^#\[cfg|^fn |\Z)', text)
    if not match:
        raise SystemExit(f"macOS lifecycle executor is missing {name}")
    return match.group(0)

fixed_steps = function_body(executor, "mac_fixed_install_steps_v1")
for text in (
    'ManagedActionV1::Create',
    'mac.lima.publisher-executor',
    'mac.lima.guest-binary(substrate-world-service)',
    'mac.lima.guest-binary(substrate-gateway)',
    'mac.lima.guest-binary(substrate)',
    'mac.lima.guest-service-state(socket)',
    'ManagedActionV1::Enable',
    'ManagedActionV1::Start',
):
    require("fixed macOS install steps", fixed_steps, text)
for forbidden in ('ManagedActionV1::Stop', 'ManagedActionV1::Remove',
                  'ManagedActionV1::Restore', 'ManagedActionV1::Replace',
                  'post_pm_requests_v1'):
    forbid("fixed macOS install steps", fixed_steps, forbidden)

fixed_execute = function_body(executor, "mac_execute_fixed_install_sequence_v1")
for text in ('index.entries[0]', 'mac_validate_fixed_install_completed_receipt_v1',
             'fixed install retry has an ambiguous prepared step',
             'index.authority_domain != "mac_host_shared"',
             'index.scope_id != capsule.scope_id',
             'sha256_hex_bootstrap_v1(&canonical_index)',
             'head.v1.json',
             'index.previous_index_sha256.as_deref()',
             'head.previous_head_sha256.as_deref()',
             'fixed install retry tail is not the exact next planned receipt',
             'current_anchor_counter: state.current_anchor.action_receipt_index_revision'):
    require("fixed macOS install executor", fixed_execute, text)
if 'current_anchor_counter: state.counter' in fixed_execute:
    raise SystemExit("fixed macOS install recovery derives its source counter from mutable state")
for text in (
    "MacPreparedFirstEffectDecisionV1::ConvergeExactAfter",
    "fixed install non-Start step is unexpectedly already After",
    '"recovered_exact_after":true',
):
    require("macOS lifecycle executor", executor, text)
for forbidden in ('post_pm_requests_v1', 'mac_issue_closed_post_pm_requests_after_stage_one_v1',
                  'ManagedActionV1::Stop', 'ManagedActionV1::Remove',
                  'ManagedActionV1::Restore', 'ManagedActionV1::Replace'):
    forbid("fixed macOS install executor", fixed_execute, forbidden)

for label, text in (
    ("managed artifact model", common),
    ("macOS lifecycle executor", executor),
    ("runtime contract", contracts),
):
    forbid(label, text, "mac.lima.guest-binary(world)")
    forbid(label, text, '"/usr/local/bin/world"')

print("AUX-R3-MAC prestage artifact route regression: PASS")
PY

# Execute the exact production adoption helper under an unprivileged UID/GID substitution. This
# proves descriptor verification, all-role convergence, and rollback after an injected mid-set
# failure rather than merely checking source tokens.
python3 - "${INSTALLER}" "${WORK_ROOT}/adoption" <<'PY'
from pathlib import Path
import hashlib
import os
import re
import stat
import subprocess
import sys

source = Path(sys.argv[1]).read_text()
root = Path(sys.argv[2])
root.mkdir()
match = re.search(
    r"(?ms)^adopt_retained_linux_artifacts_transactionally\(\) \{.*?<<'PY'\n(.*?)\nPY\n\}",
    source,
)
if not match:
    raise SystemExit("cannot extract production transactional adoption helper")
uid, gid = os.getuid(), os.getgid()
program = match.group(1)
program = program.replace("os.fchown(fd, 0, 0)", f"os.fchown(fd, {uid}, {gid})")
program = program.replace("after.st_uid != 0", f"after.st_uid != {uid}")
program = program.replace("after.st_gid != 0", f"after.st_gid != {gid}")

def make_roles(prefix):
    args = []
    observations = []
    for index in range(4):
        path = root / f"{prefix}-role-{index}"
        payload = f"role-{index}-payload\n".encode()
        path.write_bytes(payload)
        path.chmod(0o755)
        before = path.stat()
        args.extend([
            str(path),
            hashlib.sha256(payload).hexdigest(),
            f"dev:{before.st_dev}:ino:{before.st_ino}",
        ])
        observations.append((path, before.st_dev, before.st_ino, before.st_uid, before.st_gid,
                             stat.S_IMODE(before.st_mode), payload))
    return args, observations

args, observations = make_roles("success")
subprocess.run([sys.executable, "-", *args], input=program, text=True, check=True)
for path, device, inode, owner, group, mode, payload in observations:
    after = path.stat()
    if (after.st_dev, after.st_ino, after.st_uid, after.st_gid,
            stat.S_IMODE(after.st_mode), path.read_bytes()) != (
            device, inode, uid, gid, 0o755, payload):
        raise SystemExit("production adoption helper did not converge an exact descriptor set")

args, observations = make_roles("rollback")
injected = program.replace(
    "        os.fsync(fd)\n        after = os.fstat(fd)",
    "        os.fsync(fd)\n        if path.endswith('role-2'):\n"
    "            raise RuntimeError('injected adoption failure')\n"
    "        after = os.fstat(fd)",
    1,
)
failed = subprocess.run([sys.executable, "-", *args], input=injected, text=True,
                        stdout=subprocess.PIPE, stderr=subprocess.PIPE)
if failed.returncode == 0:
    raise SystemExit("injected transactional adoption failure unexpectedly succeeded")
for path, device, inode, owner, group, mode, payload in observations:
    after = path.stat()
    if (after.st_dev, after.st_ino, after.st_uid, after.st_gid,
            stat.S_IMODE(after.st_mode), path.read_bytes()) != (
            device, inode, owner, group, mode, payload):
        raise SystemExit("production adoption helper did not restore exact rollback state")
PY

printf 'AUX-R3-MAC transactional adoption execution: PASS\n'

netfilter_prefix="${WORK_ROOT}/netfilter-must-not-install"
if "${INSTALLER}" --prefix "${netfilter_prefix}" --world-netfilter \
    >"${WORK_ROOT}/netfilter.stdout" 2>"${WORK_ROOT}/netfilter.stderr"; then
    echo 'macOS fixed installer accepted forbidden --world-netfilter' >&2
    exit 1
fi
grep -Fq 'macOS --world-netfilter is not authorized for the fixed R3 install path' \
    "${WORK_ROOT}/netfilter.stderr"
if [[ -e "${netfilter_prefix}" ]]; then
    echo 'macOS netfilter rejection occurred after install-prefix mutation' >&2
    exit 1
fi
printf 'AUX-R3-MAC early netfilter rejection: PASS\n'
