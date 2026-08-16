#!/bin/zsh
set -euo pipefail

# Freeze the one prospective R3 macOS evidence-finalizer candidate.  This command is deliberately
# no-argument and user-owned.  It builds and measures artifacts, records read-only pre-install
# absence evidence, and writes (but never executes) the one reviewed root command block.

readonly REPOSITORY="/Users/spensermcconnell/.codex/worktrees/r3-macos-finalizer-rcv-stack/substrate"
readonly BRANCH="feat/r3-macos-finalizer-rcv-stack"
readonly EXPERIMENT_ID="01a00b11-d9d4-75d2-ad3f-8ea379698723"
readonly EXPERIMENT_ROOT="/Users/spensermcconnell/Library/Application Support/Atomize/R3MacEvidenceFinalizer/experiments/${EXPERIMENT_ID}"
readonly FREEZE_ROOT="${EXPERIMENT_ROOT}/candidate-freeze"
readonly ARTIFACT_ROOT="${FREEZE_ROOT}/artifacts"
readonly WORK_ROOT="${FREEZE_ROOT}/.freeze-work.v2"
readonly DRIVER="${WORK_ROOT}/freeze_driver.py"
readonly FINALIZER_MANIFEST="tools/r3-macos-finalizer/Cargo.toml"
readonly SIGNER_MANIFEST="tools/r3-macos-signer-acl/Cargo.toml"
readonly PLIST_SOURCE="scripts/mac/com.atomize.substrate.r3-macos-evidence-finalizer.v2.plist"
readonly CAPABILITY_SOURCE="tools/r3-macos-finalizer/capability-v2.json"
readonly COORDINATOR_INPUT="${FREEZE_ROOT}/coordinator-provenance-input.v2.json"
readonly GLOBAL_INPUT="${FREEZE_ROOT}/global-provenance-input.v2.json"
readonly MANIFEST_INPUT="${FREEZE_ROOT}/candidate-manifest-input.v2.json"
readonly SOURCE_HASHES="${FREEZE_ROOT}/source-hashes.v2.json"
readonly COORDINATOR_BUILD_INPUTS="${FREEZE_ROOT}/coordinator-build-inputs.v2.json"
readonly GLOBAL_BUILD_INPUTS="${FREEZE_ROOT}/global-build-inputs.v2.json"
readonly FINAL_MANIFEST="${FREEZE_ROOT}/candidate-artifact-manifest.v2.json"
readonly ADMIN_BLOCK="${FREEZE_ROOT}/reviewed-admin-command-block.v2.txt"
readonly CAPABILITY_COPY="${FREEZE_ROOT}/capability-v2.json"
readonly RAW_ABSENCE_INPUT="${WORK_ROOT}/preinstall-raw-absence.v2.json"
readonly IDENTITY_ROOT="${WORK_ROOT}/identities"
readonly PYTHON="/usr/bin/python3"
readonly GIT="/usr/bin/git"
readonly XCRUN="/usr/bin/xcrun"
readonly CARGO="/Users/spensermcconnell/.cargo/bin/cargo"
readonly RUSTC="/Users/spensermcconnell/.cargo/bin/rustc"
readonly CLANG="/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang"
readonly LD="/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/ld"
readonly SWIFTC="/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/swiftc"
readonly FIXED_PATH="/Users/spensermcconnell/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin"
readonly ZERO40="0000000000000000000000000000000000000000"
readonly ZERO64="0000000000000000000000000000000000000000000000000000000000000000"
readonly ACCEPTED_PARENT_HEAD="52f188c248ba2e4f5c4c97628fbccfdfbd3201ef"

export PATH="${FIXED_PATH}"
export LANG=C
export LC_ALL=C
export TZ=UTC

fail() {
    print -u2 -- "R3 candidate freeze failed: $*"
    exit 1
}

[[ $# -eq 0 ]] || fail "this sealed freeze command accepts no arguments"
[[ "${PWD}" == "${REPOSITORY}" ]] || fail "run only from ${REPOSITORY}"
[[ "$(/usr/bin/id -u)" == "501" && "$(/usr/bin/id -g)" == "20" ]] || fail "requires exact UID501:staff"
[[ "$(/usr/bin/id -un)" == "spensermcconnell" ]] || fail "requires canonical account spensermcconnell"
[[ -x "${PYTHON}" ]] || fail "fixed input encoder is unavailable"
[[ -x "${GIT}" && -x "${XCRUN}" && -x "${CARGO}" && -x "${RUSTC}" \
   && -x "${CLANG}" && -x "${LD}" && -x "${SWIFTC}" ]] || fail "one frozen build executable is unavailable"
[[ "$(${XCRUN} --find clang)" == "${CLANG}" && "$(${XCRUN} --find ld)" == "${LD}" \
   && "$(${XCRUN} --find swiftc)" == "${SWIFTC}" ]] \
    || fail "Xcode tool resolution differs from the frozen compiler paths"
[[ "$(${GIT} -C "${REPOSITORY}" branch --show-current)" == "${BRANCH}" ]] || fail "wrong branch"
readonly SOURCE_COMMIT="$(${GIT} -C "${REPOSITORY}" rev-parse HEAD)"
readonly SOURCE_PARENT="$(${GIT} -C "${REPOSITORY}" rev-parse HEAD^)"
readonly REMOTE_COMMIT="$(${GIT} -C "${REPOSITORY}" ls-remote --heads origin "${BRANCH}" | /usr/bin/awk '{print $1}')"
if ${GIT} -C "${REPOSITORY}" rev-parse --verify '@{upstream}' >/dev/null 2>&1; then
    fail "local-only proof-candidate branch unexpectedly has an upstream"
fi
[[ "${SOURCE_PARENT}" == "${ACCEPTED_PARENT_HEAD}" && -z "${REMOTE_COMMIT}" ]] \
    || fail "committed candidate parent or remote-absence binding diverged from the accepted stack"

# E02 freezes only the exact committed candidate. Refuse every staged, unstaged, or untracked path
# so the manifest's commit/tree pair cannot silently absorb a working-tree overlay.
env -i PATH="${FIXED_PATH}" LANG=C LC_ALL=C TZ=UTC \
    "${PYTHON}" - "${REPOSITORY}" "${GIT}" <<'PY'
import pathlib
import subprocess
import sys

repository = pathlib.Path(sys.argv[1])
git = sys.argv[2]
status = subprocess.check_output(
    [git, "-C", str(repository), "status", "--porcelain=v2", "-z"]
)
if status:
    raise SystemExit("committed candidate worktree or index is not exactly clean")
PY
[[ ! -e "${EXPERIMENT_ROOT}" && ! -L "${EXPERIMENT_ROOT}" ]] \
    || fail "fixed external experiment root already exists"
[[ ! -e /private/tmp/com.atomize.substrate.r3-macos-finalizer-freeze.v2 \
   && ! -L /private/tmp/com.atomize.substrate.r3-macos-finalizer-freeze.v2 ]] \
    || fail "fixed temporary-root absence predicate is not clean"

umask 077
/bin/mkdir -m 0700 -p "${EXPERIMENT_ROOT}"
/bin/chmod 0700 "${EXPERIMENT_ROOT}"
/bin/mkdir -m 0700 "${FREEZE_ROOT}" "${ARTIFACT_ROOT}" "${WORK_ROOT}" "${IDENTITY_ROOT}"

# Effect-free regression for the root membrane's crash-repair primitive.  It operates only inside
# the fresh freeze work root and proves an already-absent child is accepted only after its exact
# parent fsync/reopen/ENOENT sequence.
env -i PATH="${FIXED_PATH}" LANG=C LC_ALL=C TZ=UTC PYTHONDONTWRITEBYTECODE=1 \
    "${PYTHON}" - \
    "${REPOSITORY}/scripts/mac/r3-macos-finalizer-root-install.py" "${WORK_ROOT}" <<'PY'
import importlib.util
import os
import pathlib
import sys

source = pathlib.Path(sys.argv[1])
work = pathlib.Path(sys.argv[2])
spec = importlib.util.spec_from_file_location("r3_root_install_membrane", source)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
root = work / "durable-absence-pure-test.v2"
root.mkdir(mode=0o700)

if module.MAX_INSTALLED_ARTIFACT_BYTES <= module.MAX_RUNNER_CHILD_DOCUMENT:
    raise SystemExit("installed-artifact ceiling reused the child-document bound")
artifact = root / "multi-megabyte-installed-artifact"
# Match the largest observed frozen executable exactly so the Python installer
# regression exercises the same multi-megabyte size as the Rust reattestation.
artifact_bytes = b"A" * 7_212_192
artifact.write_bytes(artifact_bytes)
artifact.chmod(0o400)
data, _ = module.read_exact_sized_file(
    artifact,
    os.getuid(),
    os.getgid(),
    0o400,
    len(artifact_bytes),
    module.MAX_INSTALLED_ARTIFACT_BYTES,
)
if data != artifact_bytes:
    raise SystemExit("multi-megabyte installed-artifact exact read changed bytes")
for invalid_size in (len(artifact_bytes) - 1, len(artifact_bytes) + 1):
    try:
        module.read_exact_sized_file(
            artifact,
            os.getuid(),
            os.getgid(),
            0o400,
            invalid_size,
            module.MAX_INSTALLED_ARTIFACT_BYTES,
        )
    except module.Stop:
        pass
    else:
        raise SystemExit("installed-artifact read accepted a size mismatch")
artifact.chmod(0o600)
artifact.write_bytes(artifact_bytes[:-1])
artifact.chmod(0o400)
try:
    module.read_exact_sized_file(
        artifact,
        os.getuid(),
        os.getgid(),
        0o400,
        len(artifact_bytes),
        module.MAX_INSTALLED_ARTIFACT_BYTES,
    )
except module.Stop:
    pass
else:
    raise SystemExit("installed-artifact read accepted truncation")
artifact.chmod(0o600)
artifact.write_bytes(artifact_bytes + b"B")
artifact.chmod(0o400)
try:
    module.read_exact_sized_file(
        artifact,
        os.getuid(),
        os.getgid(),
        0o400,
        len(artifact_bytes),
        module.MAX_INSTALLED_ARTIFACT_BYTES,
    )
except module.Stop:
    pass
else:
    raise SystemExit("installed-artifact read accepted growth")
artifact.unlink()
ceiling = root / "installed-artifact-ceiling-overflow"
with ceiling.open("wb") as output:
    output.truncate(module.MAX_INSTALLED_ARTIFACT_BYTES + 1)
ceiling.chmod(0o400)
try:
    module.read_bounded_exact_file(
        ceiling,
        os.getuid(),
        os.getgid(),
        0o400,
        module.MAX_INSTALLED_ARTIFACT_BYTES,
    )
except module.Stop:
    pass
else:
    raise SystemExit("installed-artifact read accepted true ceiling overflow")
ceiling.unlink()

leaf = root / "closed-leaf"
first = module.require_durable_absence(leaf)
if not first or first["parent_fsync_return"] != 0:
    raise SystemExit("initial durable-absence proof failed")
descriptor = os.open(
    leaf,
    os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
    0o600,
)
os.write(descriptor, b"x")
os.fsync(descriptor)
os.close(descriptor)
os.unlink(leaf)
second = module.require_durable_absence(leaf)
if not second or not second["parent_reopened_same_stable_identity"]:
    raise SystemExit("post-unlink durable-absence proof failed")
parent = os.open(root.parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
os.rmdir(root)
os.fsync(parent)
os.close(parent)
if root.exists() or root.is_symlink():
    raise SystemExit("durable-absence pure-test root remains")
PY

# Record the exact committed tree that supplies every frozen source and build input.
readonly CANDIDATE_PATHS="${WORK_ROOT}/candidate-changed-paths.v2.json"
readonly SOURCE_TREE="$(env -i PATH="${FIXED_PATH}" LANG=C LC_ALL=C TZ=UTC \
    "${PYTHON}" - "${REPOSITORY}" "${GIT}" "${SOURCE_COMMIT}" "${CANDIDATE_PATHS}" <<'PY'
import hashlib
import json
import os
import pathlib
import subprocess
import sys

repository = pathlib.Path(sys.argv[1])
git = sys.argv[2]
commit = sys.argv[3]
output = pathlib.Path(sys.argv[4])
observed_commit = subprocess.check_output(
    [git, "-C", str(repository), "rev-parse", "HEAD"], text=True
).strip()
tree = subprocess.check_output(
    [git, "-C", str(repository), "rev-parse", "HEAD^{tree}"], text=True
).strip()
status = subprocess.check_output(
    [git, "-C", str(repository), "status", "--porcelain=v2", "-z"]
)
if observed_commit != commit or status:
    raise SystemExit("committed candidate identity changed before tree capture")
if len(tree) != 40 or any(character not in "0123456789abcdef" for character in tree):
    raise SystemExit("committed candidate did not produce one Git tree identity")
value = {
    "schema_owner": "substrate.r3-macos-candidate-committed-tree-inventory",
    "schema_version": 1,
    "source_base_commit": commit,
    "candidate_source_tree": tree,
    "paths": [],
}
value["path_set_sha256"] = hashlib.sha256(
    json.dumps(value["paths"], sort_keys=True, separators=(",", ":")).encode()
).hexdigest()
encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
fd = os.open(output, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o400)
try:
    remaining = memoryview(encoded)
    while remaining:
        written = os.write(fd, remaining)
        if written <= 0:
            raise OSError("short candidate inventory write")
        remaining = remaining[written:]
    os.fsync(fd)
finally:
    os.close(fd)
print(tree)
PY
)"

cleanup_work_on_error() {
    local exit_code=$?
    if [[ ${exit_code} -ne 0 ]]; then
        print -u2 -- "Candidate evidence root retained after failure: ${FREEZE_ROOT}"
    fi
    return ${exit_code}
}
trap cleanup_work_on_error EXIT

cat > "${DRIVER}" <<'PY'
import base64
import ctypes
import errno
import hashlib
import json
import os
import pathlib
import re
import stat
import subprocess
import sys

REPO = pathlib.Path("/Users/spensermcconnell/.codex/worktrees/r3-macos-finalizer-rcv-stack/substrate")
BRANCH = "feat/r3-macos-finalizer-rcv-stack"
EXPERIMENT_ID = "01a00b11-d9d4-75d2-ad3f-8ea379698723"
EXPERIMENT_ROOT = pathlib.Path("/Users/spensermcconnell/Library/Application Support/Atomize/R3MacEvidenceFinalizer/experiments") / EXPERIMENT_ID
FREEZE_ROOT = EXPERIMENT_ROOT / "candidate-freeze"
ARTIFACT_ROOT = FREEZE_ROOT / "artifacts"
WORK_ROOT = FREEZE_ROOT / ".freeze-work.v2"
IDENTITY_ROOT = WORK_ROOT / "identities"
SOURCE_HASHES_PATH = FREEZE_ROOT / "source-hashes.v2.json"
COORDINATOR_BUILD_PATH = FREEZE_ROOT / "coordinator-build-inputs.v2.json"
GLOBAL_BUILD_PATH = FREEZE_ROOT / "global-build-inputs.v2.json"
COORDINATOR_INPUT_PATH = FREEZE_ROOT / "coordinator-provenance-input.v2.json"
GLOBAL_INPUT_PATH = FREEZE_ROOT / "global-provenance-input.v2.json"
MANIFEST_INPUT_PATH = FREEZE_ROOT / "candidate-manifest-input.v2.json"
ADMIN_BLOCK_PATH = FREEZE_ROOT / "reviewed-admin-command-block.v2.txt"
RAW_ABSENCE_PATH = WORK_ROOT / "preinstall-raw-absence.v2.json"
OWNER = "substrate.r3-macos-candidate-freeze-manifest"
VERSION = 2
ZERO40 = "0" * 40
ZERO64 = "0" * 64
EMPTY_SHA256 = hashlib.sha256(b"").hexdigest()
MAX_PROCESS_SNAPSHOT = 256 * 1024
FLAGS = 0x12002
RUSTC = "/Users/spensermcconnell/.cargo/bin/rustc"
CARGO = "/Users/spensermcconnell/.cargo/bin/cargo"
GIT = "/usr/bin/git"
XCRUN = "/usr/bin/xcrun"
CODESIGN = "/usr/bin/codesign"
CLANG = "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang"
LD = "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/ld"
SWIFTC = "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/swiftc"
RUSTFLAGS = [f"-C link-arg=-fuse-ld={LD}", f"-C linker={CLANG}"]
LINKER_FLAGS = [f"-fuse-ld={LD}"]
CANDIDATE_PATH_INVENTORY = pathlib.Path(os.environ["R3_CANDIDATE_PATH_INVENTORY"])
CANDIDATE_SOURCE_TREE = os.environ["R3_CANDIDATE_SOURCE_TREE"]

ROLES = [
    ("finalizer_executable", "substrate-r3-macos-evidence-finalizer", "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-evidence-finalizer.v2", 0, 0, 0o555, "com.atomize.substrate.r3-macos-evidence-finalizer.v2"),
    ("coordinator_executable", "substrate-r3-macos-evidence-coordinator", "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-evidence-coordinator", 0, 0, 0o555, "com.atomize.substrate.r3-macos-evidence-coordinator.v2"),
    ("disposable_harness_executable", "substrate-r3-macos-disposable-harness", "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-disposable-harness", 0, 0, 0o555, "com.atomize.substrate.r3-macos-disposable-harness.v2"),
    ("peer_code_probe_executable", "substrate-r3-macos-peer-code-probe", "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-peer-code-probe", 0, 0, 0o555, "com.atomize.substrate.r3-macos-peer-code-probe.v2"),
    ("alternate_coordinator_executable", "substrate-r3-macos-evidence-coordinator-alternate-path", "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-evidence-coordinator-alternate-path", 0, 0, 0o555, "com.atomize.substrate.r3-macos-evidence-coordinator.v2"),
    ("creator_executable", "substrate-r3-macos-signer-acl-creator", "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-signer-acl-creator.v1", 0, 0, 0o555, "com.atomize.substrate.r3-macos-signer-acl-creator.v1"),
    ("wrong_identity_executable", "substrate-r3-macos-signer-acl-wrong-identity", "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-signer-acl-wrong-identity.v1", 0, 0, 0o555, "com.atomize.substrate.r3-macos-signer-acl-wrong-identity.v1"),
    ("disposable_publisher_executable", "substrate-r3-macos-disposable-publisher", "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-disposable-publisher.v2", 0, 0, 0o555, "com.atomize.substrate.r3-macos-disposable-publisher.v2"),
    ("disposable_experiment_runner_executable", "substrate-r3-macos-disposable-experiment-runner", "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-disposable-experiment-runner.v2", 0, 0, 0o555, "com.atomize.substrate.r3-macos-disposable-experiment-runner.v2"),
    ("nobody_owner_probe_executable", "substrate-r3-macos-nobody-owner-probe", "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-nobody-owner-probe", 0, 0, 0o555, "com.atomize.substrate.r3-macos-nobody-owner-probe.v2"),
    ("security_agent_observer_executable", "substrate-r3-macos-securityagent-observer", "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-securityagent-observer", 0, 0, 0o555, "com.atomize.substrate.r3-macos-securityagent-observer.v2"),
    ("benign_injection_library", "substrate-r3-macos-benign-injection-probe.dylib", "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-benign-injection-probe.dylib", 0, 0, 0o555, "com.atomize.substrate.r3-macos-benign-injection-probe.v2"),
    ("launchd_plist", "com.atomize.substrate.r3-macos-evidence-finalizer.v2.plist", "/Library/LaunchDaemons/com.atomize.substrate.r3-macos-evidence-finalizer.v2.plist", 0, 0, 0o644, None),
    ("capability_manifest", "capability-v2.json", str(FREEZE_ROOT / "capability-v2.json"), 501, 20, 0o400, None),
]

def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()

def sha_bytes(data):
    return hashlib.sha256(data).hexdigest()

def sha_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()

def write_immutable(path, value):
    data = canonical(value)
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o400)
    try:
        remaining = memoryview(data)
        while remaining:
            written = os.write(fd, remaining)
            if written <= 0:
                raise OSError("short immutable candidate write")
            remaining = remaining[written:]
        os.fsync(fd)
    finally:
        os.close(fd)
    os.chmod(path, 0o400, follow_symlinks=False)
    parent = os.open(pathlib.Path(path).parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        os.fsync(parent)
    finally:
        os.close(parent)

def source_paths():
    text = (REPO / "tools/r3-macos-finalizer/src/experiment/freeze_provenance.rs").read_text()
    match = re.search(r"pub const CANDIDATE_FREEZE_SOURCE_PATHS_V2: &\[&str\] = &\[(.*?)\n\];", text, re.S)
    if not match:
        raise SystemExit("cannot extract closed candidate source sequence")
    values = re.findall(r'\s*"([^"]+)",', match.group(1))
    if not values or len(values) != len(set(values)):
        raise SystemExit("candidate source sequence is empty or duplicated")
    return values

def source_input_and_manifest(commit, tree):
    entries = []
    for relative in source_paths():
        path = REPO / relative
        if not path.is_file() or path.is_symlink():
            raise SystemExit(f"frozen source is not one nofollow regular file: {relative}")
        data = path.read_bytes()
        if not data:
            raise SystemExit(f"empty frozen source: {relative}")
        entries.append({"repository_relative_path": relative, "byte_length": len(data), "sha256": sha_bytes(data)})
    source_input = {"entries": entries}
    entry_set = sha_bytes(canonical({"domain": "substrate.r3-macos-candidate-source-hash-set.v2", "entries": entries}))
    manifest = {
        "schema_owner": "substrate.r3-macos-candidate-source-hashes",
        "schema_version": 2,
        "repository_path": str(REPO),
        "source_commit": commit,
        "source_tree": tree,
        "entries": entries,
        "entry_set_sha256": entry_set,
    }
    return source_input, manifest, sha_bytes(canonical(manifest))

def raw_tool(tool, path, argv, expected_status=0):
    proc = subprocess.run([path] + argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if proc.returncode != expected_status:
        raise SystemExit(
            f"unexpected version-probe status for {tool}: {proc.returncode}, "
            f"expected {expected_status}"
        )
    version = proc.stdout + proc.stderr
    if not version:
        raise SystemExit(f"empty version output for {tool}")
    return {
        "tool": tool,
        "executable_path": path,
        "executable_sha256": sha_file(path),
        "version_output_base64url": base64.urlsafe_b64encode(version).rstrip(b"=").decode(),
        "version_output_sha256": sha_bytes(version),
        "version_output_byte_length": len(version),
    }

def tool_paths(lane):
    if (
        subprocess.check_output([XCRUN, "--find", "clang"], text=True).strip() != CLANG
        or subprocess.check_output([XCRUN, "--find", "ld"], text=True).strip() != LD
    ):
        raise SystemExit("Xcode linker resolution differs from the frozen paths")
    values = [
        raw_tool("rustc", RUSTC, ["--version", "--verbose"]),
        raw_tool("cargo", CARGO, ["--version", "--verbose"]),
        raw_tool("clang", CLANG, ["--version"]),
        raw_tool("ld", LD, ["-v"]),
    ]
    if lane == "global":
        if subprocess.check_output([XCRUN, "--find", "swiftc"], text=True).strip() != SWIFTC:
            raise SystemExit("Xcode compiler resolution differs from the frozen paths")
        values.append(raw_tool("swiftc", SWIFTC, ["--version"]))
    # The installed codesign(1) has no version flag. Its bounded help probe exits 2 by contract;
    # the manifest independently binds the exact executable bytes as the authoritative identity.
    values.extend([raw_tool("codesign", CODESIGN, ["-h"], 2), raw_tool("xcrun", XCRUN, ["--version"])])
    return values

def recipes(lane):
    if lane == "coordinator":
        return [{"sequence_ordinal": 1, "tool": "cargo", "package_manifest": "tools/r3-macos-finalizer/Cargo.toml", "cargo_features": [], "binary_products": ["substrate-r3-macos-evidence-coordinator"], "artifact_roles": ["coordinator_executable", "alternate_coordinator_executable"]}]
    return [
        {"sequence_ordinal": 1, "tool": "cargo", "package_manifest": "tools/r3-macos-finalizer/Cargo.toml", "cargo_features": [], "binary_products": ["substrate-r3-macos-evidence-finalizer", "substrate-r3-macos-peer-code-probe"], "artifact_roles": ["finalizer_executable", "peer_code_probe_executable"]},
        {"sequence_ordinal": 2, "tool": "cargo", "package_manifest": "tools/r3-macos-finalizer/Cargo.toml", "cargo_features": ["experiment-harness"], "binary_products": ["substrate-r3-macos-disposable-harness"], "artifact_roles": ["disposable_harness_executable"]},
        {"sequence_ordinal": 3, "tool": "cargo", "package_manifest": "tools/r3-macos-finalizer/Cargo.toml", "cargo_features": ["candidate-freeze-builder"], "binary_products": ["substrate-r3-macos-candidate-freeze-provenance-builder", "substrate-r3-macos-candidate-freeze-global-provenance-builder", "substrate-r3-macos-candidate-freeze-manifest-builder"], "artifact_roles": []},
        {"sequence_ordinal": 4, "tool": "cargo", "package_manifest": "tools/r3-macos-signer-acl/Cargo.toml", "cargo_features": [], "binary_products": ["substrate-r3-macos-signer-acl-creator", "substrate-r3-macos-signer-acl-wrong-identity", "substrate-r3-macos-disposable-publisher", "substrate-r3-macos-disposable-experiment-runner", "substrate-r3-macos-nobody-owner-probe"], "artifact_roles": ["creator_executable", "wrong_identity_executable", "disposable_publisher_executable", "disposable_experiment_runner_executable", "nobody_owner_probe_executable"]},
        {"sequence_ordinal": 5, "tool": "swiftc", "package_manifest": None, "cargo_features": [], "binary_products": ["substrate-r3-macos-securityagent-observer"], "artifact_roles": ["security_agent_observer_executable"]},
        {"sequence_ordinal": 6, "tool": "clang", "package_manifest": None, "cargo_features": [], "binary_products": ["substrate-r3-macos-benign-injection-probe.dylib"], "artifact_roles": ["benign_injection_library"]},
    ]

def environment(commit, tree, source_digest, capability, plist, lane, coordinator=None):
    commit_epoch = subprocess.check_output([GIT, "-C", str(REPO), "log", "-1", "--format=%ct"], text=True).strip()
    sdk = subprocess.check_output([XCRUN, "--sdk", "macosx", "--show-sdk-path"], text=True).strip()
    values = {
        "CARGO_HOME": "/Users/spensermcconnell/.cargo",
        "CARGO_INCREMENTAL": "0",
        "CARGO_TARGET_DIR": str(WORK_ROOT / ("coordinator-target" if lane == "coordinator" else "global-target")),
        "LANG": "C",
        "LC_ALL": "C",
        "MACOSX_DEPLOYMENT_TARGET": "15.0",
        "PATH": "/Users/spensermcconnell/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin",
        "R3_CAPABILITY_DIGEST": capability,
        "R3_COORDINATOR_ACCOUNT": "spensermcconnell",
        "R3_COORDINATOR_BUILD_INPUTS_SHA256": ZERO64,
        "R3_COORDINATOR_CDHASH": ZERO40,
        "R3_COORDINATOR_GID": "20",
        "R3_COORDINATOR_REQUIREMENT": f'identifier com.atomize.substrate.r3-macos-evidence-coordinator.v2 and cdhash H"{ZERO40}"',
        "R3_COORDINATOR_SHA256": ZERO64,
        "R3_COORDINATOR_SOURCE_COMMIT": ZERO40,
        "R3_COORDINATOR_SOURCE_HASHES_SHA256": ZERO64,
        "R3_COORDINATOR_SOURCE_TREE": ZERO40,
        "R3_COORDINATOR_UID": "501",
        "R3_FINALIZER_LAUNCHD_PLIST_SHA256": plist,
        "R3_FREEZE_INPUT_ENCODER": "/usr/bin/python3",
        "R3_FREEZE_INPUT_ENCODER_SHA256": sha_file("/usr/bin/python3"),
        "R3_FREEZE_SCRIPT_SHA256": sha_file(REPO / "scripts/mac/freeze-r3-macos-finalizer-candidate.sh"),
        "R3_SOURCE_COMMIT": commit,
        "R3_SOURCE_IDENTITY_SHA256": source_digest,
        "R3_SOURCE_TREE": tree,
        "RUSTUP_HOME": "/Users/spensermcconnell/.rustup",
        "RUSTFLAGS": " ".join(RUSTFLAGS),
        "SDKROOT": sdk,
        "SOURCE_DATE_EPOCH": commit_epoch,
        "TZ": "UTC",
        "ZERO_AR_DATE": "1",
    }
    if lane == "global":
        if coordinator is None:
            raise SystemExit("global environment lacks coordinator")
        values.update({
            "R3_COORDINATOR_BUILD_INPUTS_SHA256": coordinator["build_inputs_sha256"],
            "R3_COORDINATOR_CDHASH": coordinator["cdhash"],
            "R3_COORDINATOR_REQUIREMENT": coordinator["designated_requirement"],
            "R3_COORDINATOR_SHA256": coordinator["executable_sha256"],
            "R3_COORDINATOR_SOURCE_COMMIT": commit,
            "R3_COORDINATOR_SOURCE_HASHES_SHA256": source_digest,
            "R3_COORDINATOR_SOURCE_TREE": tree,
        })
    return [{"name": name, "value": value} for name, value in sorted(values.items())]

def build_input(lane, commit, tree, source_digest, capability, plist, coordinator=None):
    sdk = subprocess.check_output([XCRUN, "--sdk", "macosx", "--show-sdk-path"], text=True).strip()
    sdk_version = subprocess.check_output([XCRUN, "--sdk", "macosx", "--show-sdk-version"], text=True).strip()
    host = next(line.split(":", 1)[1].strip() for line in subprocess.check_output([RUSTC, "-vV"], text=True).splitlines() if line.startswith("host:"))
    prerequisite = None
    if coordinator is not None:
        prerequisite = coordinator
    return {
        "lane": lane,
        "coordinator_prerequisite": prerequisite,
        "tools": tool_paths(lane),
        "macos_sdk_path": sdk,
        "macos_sdk_version": sdk_version,
        "macos_sdk_settings_sha256": sha_file(pathlib.Path(sdk) / "SDKSettings.json"),
        "target_triple": host,
        "macos_deployment_target": "15.0",
        "cargo_profile": "release",
        "cargo_locked": True,
        "recipes": recipes(lane),
        "rustflags": RUSTFLAGS,
        "linker_flags": LINKER_FLAGS,
        "frozen_environment": environment(commit, tree, source_digest, capability, plist, lane, coordinator),
    }

def stage1():
    commit = subprocess.check_output([GIT, "-C", str(REPO), "rev-parse", "HEAD"], text=True).strip()
    inventory = json.loads(CANDIDATE_PATH_INVENTORY.read_bytes())
    if (
        inventory.get("source_base_commit") != commit
        or inventory.get("candidate_source_tree") != CANDIDATE_SOURCE_TREE
        or len(CANDIDATE_SOURCE_TREE) != 40
        or any(character not in "0123456789abcdef" for character in CANDIDATE_SOURCE_TREE)
    ):
        raise SystemExit("private candidate tree differs from the frozen path inventory")
    tree = CANDIDATE_SOURCE_TREE
    source_input, _, source_digest = source_input_and_manifest(commit, tree)
    capability = sha_file(REPO / "tools/r3-macos-finalizer/capability-v2.json")
    plist = sha_file(REPO / "scripts/mac/com.atomize.substrate.r3-macos-evidence-finalizer.v2.plist")
    value = {
        "schema_owner": OWNER, "schema_version": VERSION, "experiment_id": EXPERIMENT_ID,
        "repository_path": str(REPO), "repository_branch": BRANCH,
        "source_commit": commit, "source_tree": tree, "source_hashes": source_input,
        "coordinator_build_inputs": build_input("coordinator", commit, tree, source_digest, capability, plist),
        "capability_manifest_sha256": capability, "launchd_plist_sha256": plist,
    }
    write_immutable(COORDINATOR_INPUT_PATH, value)

def global_stage():
    coordinator_input = json.loads(COORDINATOR_INPUT_PATH.read_bytes())
    source_digest = sha_file(SOURCE_HASHES_PATH)
    coordinator_build = json.loads(COORDINATOR_BUILD_PATH.read_bytes())
    identity = json.loads((IDENTITY_ROOT / "coordinator_executable.json").read_bytes())
    prerequisite = {
        "source_commit": coordinator_input["source_commit"],
        "source_tree": coordinator_input["source_tree"],
        "source_hashes_sha256": source_digest,
        "build_inputs_sha256": coordinator_build["input_set_sha256"],
        "executable_sha256": identity["sha256"], "executable_size": identity["size"],
        "external_path": identity["external_path"], "intended_path": identity["intended_path"],
        "signing_identifier": identity["signing_identifier"],
        "designated_requirement": identity["designated_requirement"], "cdhash": identity["cdhash"],
        "code_flags": identity["code_flags"], "team_id": identity["team_id"],
        "entitlements_size": identity["entitlements_size"],
        "entitlements_sha256": identity["entitlements_sha256"],
    }
    build = build_input("global", coordinator_input["source_commit"], coordinator_input["source_tree"], source_digest, coordinator_input["capability_manifest_sha256"], coordinator_input["launchd_plist_sha256"], prerequisite)
    value = {
        "schema_owner": OWNER, "schema_version": VERSION, "experiment_id": EXPERIMENT_ID,
        "coordinator_provenance_input_sha256": sha_bytes(canonical(coordinator_input)),
        "global_build_inputs": build,
    }
    write_immutable(GLOBAL_INPUT_PATH, value)

def measure(role):
    item = next(value for value in ROLES if value[0] == role)
    role, filename, intended, uid, gid, mode, identifier = item
    path = ARTIFACT_ROOT / filename
    data = path.read_bytes()
    value = {"role": role, "external_path": str(path), "intended_path": intended, "uid": uid, "gid": gid, "mode": mode, "size": len(data), "sha256": sha_bytes(data), "signing_identifier": identifier, "designated_requirement": None, "cdhash": None, "code_flags": None, "team_id": None, "entitlements_size": None, "entitlements_sha256": None}
    if identifier is not None:
        detail = subprocess.run(["/usr/bin/codesign", "-d", "--verbose=4", str(path)], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
        text = (detail.stdout + detail.stderr).decode(errors="strict")
        found_identifier = re.search(r"^Identifier=(.+)$", text, re.M)
        found_cdhash = re.search(r"^CDHash=([0-9a-f]{40})$", text, re.M)
        found_flags = re.search(r"flags=0x([0-9a-fA-F]+)", text)
        found_team = re.search(r"^TeamIdentifier=(.+)$", text, re.M)
        if not found_identifier or found_identifier.group(1) != identifier or not found_cdhash or not found_flags or int(found_flags.group(1), 16) != FLAGS:
            raise SystemExit(f"code identity mismatch for {role}: {text}")
        requirement_proc = subprocess.run(["/usr/bin/codesign", "-d", "-r-", str(path)], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
        requirement_text = (requirement_proc.stdout + requirement_proc.stderr).decode(errors="strict")
        requirement = next((line.split("designated =>", 1)[1].strip() for line in requirement_text.splitlines() if "designated =>" in line), None)
        if not requirement:
            raise SystemExit(f"missing designated requirement for {role}")
        entitlement_proc = subprocess.run(["/usr/bin/codesign", "-d", "--entitlements", "-", str(path)], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
        entitlements = entitlement_proc.stdout
        if entitlements:
            raise SystemExit(f"candidate {role} unexpectedly carries entitlements")
        team = found_team.group(1) if found_team else None
        if team == "not set":
            team = None
        if team is not None:
            raise SystemExit(f"candidate {role} unexpectedly carries TeamIdentifier {team}")
        value.update({"designated_requirement": requirement, "cdhash": found_cdhash.group(1), "code_flags": int(found_flags.group(1), 16), "team_id": None, "entitlements_size": 0, "entitlements_sha256": EMPTY_SHA256})
    write_immutable(IDENTITY_ROOT / f"{role}.json", value)

def raw_stream(data):
    return {"base64url": base64.urlsafe_b64encode(data).rstrip(b"=").decode(), "sha256": sha_bytes(data), "byte_length": len(data)}

def process_snapshot():
    libproc = ctypes.CDLL("/usr/lib/libproc.dylib", use_errno=True)
    libproc.proc_listallpids.argtypes = [ctypes.c_void_p, ctypes.c_int]
    libproc.proc_listallpids.restype = ctypes.c_int
    count = libproc.proc_listallpids(None, 0)
    if count <= 0:
        raise OSError(ctypes.get_errno(), "proc_listallpids count")
    capacity = count + 64
    values = (ctypes.c_int * capacity)()
    actual = libproc.proc_listallpids(values, ctypes.sizeof(values))
    if actual <= 0:
        raise OSError(ctypes.get_errno(), "proc_listallpids population")
    if actual >= capacity:
        raise RuntimeError("proc_listallpids saturated the candidate process buffer")
    libproc.proc_pidpath.argtypes = [ctypes.c_int, ctypes.c_void_p, ctypes.c_uint32]
    libproc.proc_pidpath.restype = ctypes.c_int
    records = []
    for pid in sorted(set(values[:actual])):
        if pid <= 0:
            continue
        buffer = ctypes.create_string_buffer(4096)
        ctypes.set_errno(0)
        length = libproc.proc_pidpath(pid, buffer, len(buffer))
        raw_errno = ctypes.get_errno()
        if length > 0:
            records.append({"pid": pid, "path": buffer.value.decode(errors="strict"), "classification": "path_observed", "raw_errno": 0})
        elif raw_errno in (errno.ESRCH, errno.ENOENT):
            records.append({"pid": pid, "path": None, "classification": "disappeared_during_snapshot", "raw_errno": raw_errno})
        else:
            raise OSError(raw_errno, f"proc_pidpath could not classify candidate pid {pid}")
    return records

def absence_plan():
    values = [{"kind": "filesystem_path", "identity": item[2]} for item in ROLES if item[0] != "capability_manifest"]
    paths = [
        "/Library/Application Support/Atomize", "/Library/Application Support/Atomize/R3MacEvidenceFinalizer", "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2", "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/candidate-artifact-manifest.v2.json",
        "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/disposable-publisher-prepared-input.v2.json", "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/candidate-identity-packet.v2.json", "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/peer-control-identity-packet.v2.json", "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/inbox", "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/inbox/finalization-request.v2.json", "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/inbox/terminal-binding-request.v1.json",
        "/private/var/db/com.atomize.substrate.r3-macos-evidence-finalizer.v2", "/private/var/db/com.atomize.substrate.r3-macos-evidence-finalizer.v2/latches", "/private/var/db/com.atomize.substrate.r3-macos-disposable-publisher.v2", "/private/var/db/com.atomize.substrate.r3-macos-disposable-experiment-runner.v2", "/private/var/db/com.atomize.substrate.r3-macos-signer-acl-experiment.v2", "/private/tmp/com.atomize.substrate.r3-macos-finalizer-freeze.v2", str(EXPERIMENT_ROOT / "global-publisher-exchange"),
    ]
    values.extend({"kind": "filesystem_path", "identity": path} for path in paths)
    for scope in ["01a00b11-d9d5-7af6-b9b7-e17d1cbb746b", "01a00b11-d9d6-7872-9b8c-2fb75c1f047b"]:
        values.extend([
            {"kind": "filesystem_path", "identity": f"/private/var/db/com.atomize.substrate.r3-macos-evidence-finalizer.v2/{scope}"},
            {"kind": "filesystem_path", "identity": f"/private/var/db/com.atomize.substrate.r3-macos-evidence-finalizer.v2/capability/{scope}"},
            {"kind": "filesystem_path", "identity": f"/private/var/db/com.atomize.substrate.r3-macos-evidence-finalizer.v2/latches/{scope}.retirement-terminal.v2.latch"},
        ])
    values.append({"kind": "launchd_label", "identity": "com.atomize.substrate.r3-macos-evidence-finalizer.v2"})
    values.append({"kind": "unix_endpoint", "identity": "/private/var/run/com.atomize.substrate.r3-macos-evidence-finalizer.v2.sock"})
    values.append({"kind": "process_snapshot", "identity": "candidate_preinstall_process_snapshot_v2"})
    values.extend({"kind": "process_executable_path", "identity": item[2]} for item in ROLES if item[6] is not None)
    return values

def observe_absence():
    processes = process_snapshot()
    snapshot_bytes = canonical(processes)
    if len(snapshot_bytes) > MAX_PROCESS_SNAPSHOT:
        raise SystemExit("candidate process snapshot exceeds its fixed canonical byte bound")
    snapshot_stream = raw_stream(snapshot_bytes)
    snapshot_digest = snapshot_stream["sha256"]
    output = []
    for probe in absence_plan():
        kind, identity = probe["kind"], probe["identity"]
        if kind in ("filesystem_path", "unix_endpoint"):
            try:
                os.lstat(identity)
            except FileNotFoundError as error:
                output.append({"kind": kind, "identity": identity, "lstat_return": -1, "raw_errno": error.errno, "stat_result": None})
            else:
                raise SystemExit(f"preinstall path is not absent: {identity}")
        elif kind == "launchd_label":
            proc = subprocess.run(["/bin/launchctl", "print", f"system/{identity}"], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            if proc.returncode != 113:
                raise SystemExit(f"launchd label absence returned {proc.returncode}, expected 113")
            expected_stderr = f'Bad request.\nCould not find service "{identity}" in domain for system\n'.encode()
            if proc.stdout or proc.stderr != expected_stderr:
                raise SystemExit("launchctl output is not the frozen service-not-found classification")
            output.append({"kind": kind, "identity": identity, "raw_exit_status": proc.returncode, "stdout": raw_stream(proc.stdout), "stderr": raw_stream(proc.stderr)})
        elif kind == "process_snapshot":
            output.append({"kind": kind, "identity": identity, "snapshot": {"total_process_count": len(processes), "process_records": processes, "process_snapshot": snapshot_stream, "process_snapshot_sha256": snapshot_digest}})
        elif kind == "process_executable_path":
            matching = [record["pid"] for record in processes if record["path"] == identity]
            if matching:
                raise SystemExit(f"exact-path process already exists: {identity} {matching}")
            output.append({"kind": kind, "identity": identity, "enumeration": {"matching_pids": matching, "process_snapshot_sha256": snapshot_digest}})
        else:
            raise SystemExit(f"unknown absence kind {kind}")
    write_immutable(RAW_ABSENCE_PATH, output)

def rollback_plan():
    targets = [
        ("launchd_service", "bootout_then_verify_absent"), ("finalizer_endpoint", "observe_exact_absence_only"), ("finalizer_journal_root", "observe_exact_absence_only"), ("coordinator_inbox_root", "remove_exact_then_verify_absent"), ("publisher_root", "remove_exact_then_verify_absent"), ("runner_root", "remove_exact_then_verify_absent"), ("prepared_input_packet", "remove_exact_then_verify_absent"), ("candidate_identity_packet", "remove_exact_then_verify_absent"), ("peer_control_identity_packet", "remove_exact_then_verify_absent"), ("creator_marker_root", "remove_exact_then_verify_absent"), ("freeze_temporary_root", "remove_exact_then_verify_absent"), ("installed_candidate_manifest", "remove_exact_then_verify_absent"),
    ]
    for item in reversed([item for item in ROLES if item[0] != "capability_manifest"]):
        targets.append(({"installed_artifact": item[0]}, "remove_exact_then_verify_absent"))
    targets.extend([("installed_support_v2_directory", "remove_if_same_empty_then_verify_absent"), ("installed_support_product_directory", "remove_if_same_empty_then_verify_absent"), ("installed_support_atomize_directory", "remove_if_same_empty_then_verify_absent"), ("external_evidence_root", "retain_durable_external_evidence")])
    return [{"sequence_ordinal": index + 1, "target": target, "disposition": disposition} for index, (target, disposition) in enumerate(targets)]

def root_install_authority_sha256():
    coordinator_input = json.loads(COORDINATOR_INPUT_PATH.read_bytes())
    global_input = json.loads(GLOBAL_INPUT_PATH.read_bytes())
    coordinator_build = json.loads(COORDINATOR_BUILD_PATH.read_bytes())
    global_build = json.loads(GLOBAL_BUILD_PATH.read_bytes())
    artifacts = [json.loads((IDENTITY_ROOT / f"{item[0]}.json").read_bytes()) for item in ROLES]
    raw_absence = json.loads(RAW_ABSENCE_PATH.read_bytes())
    directories = [{"path": path, "uid": 0, "gid": 0, "mode": 0o755} for path in ["/Library/Application Support/Atomize", "/Library/Application Support/Atomize/R3MacEvidenceFinalizer", "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2"]]
    projection = {
        "schema_owner": "substrate.r3-macos-finalizer-root-install-authority",
        "schema_version": VERSION,
        "experiment_id": EXPERIMENT_ID,
        "repository_path": str(REPO),
        "repository_branch": BRANCH,
        "source_commit": coordinator_input["source_commit"],
        "source_tree": coordinator_input["source_tree"],
        "source_hashes_manifest_sha256": sha_file(SOURCE_HASHES_PATH),
        "coordinator_build_input_manifest_sha256": sha_file(COORDINATOR_BUILD_PATH),
        "coordinator_build_digest": coordinator_build["input_set_sha256"],
        "global_build_input_manifest_sha256": sha_file(GLOBAL_BUILD_PATH),
        "global_build_digest": global_build["input_set_sha256"],
        "coordinator_provenance_input_sha256": sha_file(COORDINATOR_INPUT_PATH),
        "global_provenance_input_sha256": sha_file(GLOBAL_INPUT_PATH),
        "capability_manifest_sha256": coordinator_input["capability_manifest_sha256"],
        "launchd_plist_sha256": coordinator_input["launchd_plist_sha256"],
        "install_parent_directory_set_sha256": sha_bytes(canonical(directories)),
        "artifact_set_sha256": sha_bytes(canonical({"domain": "substrate.r3-macos-candidate-freeze-artifact-set.v2", "artifacts": artifacts})),
        "preinstall_raw_absence_sha256": sha_bytes(canonical(raw_absence)),
        "rollback_plan_sha256": sha_bytes(canonical(rollback_plan())),
    }
    print(sha_bytes(canonical(projection)))

def final_stage():
    coordinator_input = json.loads(COORDINATOR_INPUT_PATH.read_bytes())
    global_input = json.loads(GLOBAL_INPUT_PATH.read_bytes())
    artifacts = [json.loads((IDENTITY_ROOT / f"{item[0]}.json").read_bytes()) for item in ROLES]
    value = {
        "schema_owner": OWNER, "schema_version": VERSION, "experiment_id": EXPERIMENT_ID,
        "artifact_root": str(FREEZE_ROOT), "repository_path": str(REPO), "repository_branch": BRANCH,
        "source_commit": coordinator_input["source_commit"], "source_tree": coordinator_input["source_tree"],
        "source_hashes": coordinator_input["source_hashes"], "coordinator_build_inputs": coordinator_input["coordinator_build_inputs"], "global_build_inputs": global_input["global_build_inputs"],
        "capability_manifest_sha256": coordinator_input["capability_manifest_sha256"], "launchd_plist_sha256": coordinator_input["launchd_plist_sha256"],
        "install_parent_directories": [{"path": path, "uid": 0, "gid": 0, "mode": 0o755} for path in ["/Library/Application Support/Atomize", "/Library/Application Support/Atomize/R3MacEvidenceFinalizer", "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2"]],
        "artifacts": artifacts, "preinstall_raw_absence_observations": json.loads(RAW_ABSENCE_PATH.read_bytes()), "rollback_plan": rollback_plan(),
        "reviewed_admin_block_path": str(ADMIN_BLOCK_PATH), "reviewed_admin_block_sha256": sha_file(ADMIN_BLOCK_PATH),
    }
    write_immutable(MANIFEST_INPUT_PATH, value)

command = sys.argv[1] if len(sys.argv) == 2 else None
if command == "stage1": stage1()
elif command == "global": global_stage()
elif command == "measure-coordinator": measure("coordinator_executable")
elif command == "measure-all":
    for item in ROLES: measure(item[0])
elif command == "observe": observe_absence()
elif command == "root-authority": root_install_authority_sha256()
elif command == "final": final_stage()
else: raise SystemExit("freeze driver accepts one internal closed stage")
PY
/bin/chmod 0500 "${DRIVER}"

readonly BOOTSTRAP_TARGET="${WORK_ROOT}/bootstrap-target"
readonly COORDINATOR_TARGET="${WORK_ROOT}/coordinator-target"
readonly GLOBAL_TARGET="${WORK_ROOT}/global-target"
freeze_driver_env=(
    HOME=/Users/spensermcconnell PATH="${FIXED_PATH}" LANG=C LC_ALL=C TZ=UTC
    R3_CANDIDATE_SOURCE_TREE="${SOURCE_TREE}" R3_CANDIDATE_PATH_INVENTORY="${CANDIDATE_PATHS}"
)
readonly CAPABILITY_SHA256="$(/usr/bin/shasum -a 256 "${CAPABILITY_SOURCE}" | /usr/bin/awk '{print $1}')"
readonly PLIST_SHA256="$(/usr/bin/shasum -a 256 "${PLIST_SOURCE}" | /usr/bin/awk '{print $1}')"
readonly FREEZE_SCRIPT_SHA256="$(/usr/bin/shasum -a 256 scripts/mac/freeze-r3-macos-finalizer-candidate.sh | /usr/bin/awk '{print $1}')"
readonly FREEZE_ENCODER_SHA256="$(/usr/bin/shasum -a 256 "${PYTHON}" | /usr/bin/awk '{print $1}')"

/usr/bin/plutil -lint "${PLIST_SOURCE}" >/dev/null
/bin/cp -p "${CAPABILITY_SOURCE}" "${CAPABILITY_COPY}"
/bin/chmod 0400 "${CAPABILITY_COPY}"
/bin/cp -p "${CAPABILITY_SOURCE}" "${ARTIFACT_ROOT}/capability-v2.json"
/bin/chmod 0400 "${ARTIFACT_ROOT}/capability-v2.json"
/bin/cp -p "${PLIST_SOURCE}" "${ARTIFACT_ROOT}/com.atomize.substrate.r3-macos-evidence-finalizer.v2.plist"
/bin/chmod 0400 "${ARTIFACT_ROOT}/com.atomize.substrate.r3-macos-evidence-finalizer.v2.plist"

# Bootstrap only the three deterministic validators.  Their outputs are later rechecked by the
# globally frozen copies before the final manifest is accepted.
env -i "${freeze_driver_env[@]}" CARGO_HOME=/Users/spensermcconnell/.cargo \
    RUSTUP_HOME=/Users/spensermcconnell/.rustup CARGO_TARGET_DIR="${BOOTSTRAP_TARGET}" \
    "${CARGO}" build --release --locked \
    --manifest-path "${FINALIZER_MANIFEST}" --features candidate-freeze-builder \
    --bin substrate-r3-macos-candidate-freeze-provenance-builder \
    --bin substrate-r3-macos-candidate-freeze-global-provenance-builder \
    --bin substrate-r3-macos-candidate-freeze-manifest-builder

env -i "${freeze_driver_env[@]}" "${PYTHON}" "${DRIVER}" stage1
env -i "${freeze_driver_env[@]}" \
    "${BOOTSTRAP_TARGET}/release/substrate-r3-macos-candidate-freeze-provenance-builder"
readonly SOURCE_IDENTITY_SHA256="$(/usr/bin/shasum -a 256 "${SOURCE_HASHES}" | /usr/bin/awk '{print $1}')"
readonly COORDINATOR_BUILD_SHA256="$(env -i "${freeze_driver_env[@]}" "${PYTHON}" -c 'import json,sys; print(json.load(open(sys.argv[1]))["input_set_sha256"])' "${COORDINATOR_BUILD_INPUTS}")"

build_env=(
    CARGO_HOME=/Users/spensermcconnell/.cargo CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="${COORDINATOR_TARGET}" LANG=C LC_ALL=C
    MACOSX_DEPLOYMENT_TARGET=15.0 PATH=/Users/spensermcconnell/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin
    R3_CAPABILITY_DIGEST="${CAPABILITY_SHA256}" R3_COORDINATOR_ACCOUNT=spensermcconnell
    R3_COORDINATOR_BUILD_INPUTS_SHA256="${ZERO64}" R3_COORDINATOR_CDHASH="${ZERO40}"
    R3_COORDINATOR_GID=20
    R3_COORDINATOR_REQUIREMENT="identifier com.atomize.substrate.r3-macos-evidence-coordinator.v2 and cdhash H\"${ZERO40}\""
    R3_COORDINATOR_SHA256="${ZERO64}" R3_COORDINATOR_SOURCE_COMMIT="${ZERO40}"
    R3_COORDINATOR_SOURCE_HASHES_SHA256="${ZERO64}" R3_COORDINATOR_SOURCE_TREE="${ZERO40}"
    R3_COORDINATOR_UID=501 R3_FINALIZER_LAUNCHD_PLIST_SHA256="${PLIST_SHA256}"
    R3_FREEZE_INPUT_ENCODER="${PYTHON}" R3_FREEZE_INPUT_ENCODER_SHA256="${FREEZE_ENCODER_SHA256}"
    R3_FREEZE_SCRIPT_SHA256="${FREEZE_SCRIPT_SHA256}"
    R3_SOURCE_COMMIT="${SOURCE_COMMIT}" R3_SOURCE_IDENTITY_SHA256="${SOURCE_IDENTITY_SHA256}"
    R3_SOURCE_TREE="${SOURCE_TREE}" R3_BUILD_INPUTS_SHA256="${COORDINATOR_BUILD_SHA256}"
    RUSTFLAGS="-C link-arg=-fuse-ld=${LD} -C linker=${CLANG}"
    RUSTUP_HOME=/Users/spensermcconnell/.rustup SDKROOT="$(${XCRUN} --sdk macosx --show-sdk-path)"
    SOURCE_DATE_EPOCH="$(${GIT} -C "${REPOSITORY}" log -1 --format=%ct)" TZ=UTC ZERO_AR_DATE=1
)
env -i "${build_env[@]}" \
    "${CARGO}" build --release --locked \
    --manifest-path "${FINALIZER_MANIFEST}" --bin substrate-r3-macos-evidence-coordinator

sign_artifact() {
    local source="$1" destination="$2" identifier="$3" lane="$4"
    local -a signing_env
    if [[ "${lane}" == coordinator ]]; then
        signing_env=("${build_env[@]}")
    elif [[ "${lane}" == global ]]; then
        signing_env=("${global_env[@]}")
    else
        fail "unknown signing lane ${lane}"
    fi
    /bin/cp -p "${source}" "${destination}"
    /bin/chmod 0755 "${destination}"
    env -i "${signing_env[@]}" /usr/bin/codesign --force --sign - --timestamp=none --options runtime,library \
        --identifier "${identifier}" "${destination}"
    env -i "${signing_env[@]}" /usr/bin/codesign --verify --strict --verbose=4 "${destination}"
    /bin/chmod 0400 "${destination}"
}

sign_artifact "${COORDINATOR_TARGET}/release/substrate-r3-macos-evidence-coordinator" \
    "${ARTIFACT_ROOT}/substrate-r3-macos-evidence-coordinator" \
    com.atomize.substrate.r3-macos-evidence-coordinator.v2 coordinator
/bin/cp -p "${ARTIFACT_ROOT}/substrate-r3-macos-evidence-coordinator" \
    "${ARTIFACT_ROOT}/substrate-r3-macos-evidence-coordinator-alternate-path"
/bin/chmod 0400 "${ARTIFACT_ROOT}/substrate-r3-macos-evidence-coordinator-alternate-path"
env -i "${freeze_driver_env[@]}" "${PYTHON}" "${DRIVER}" measure-coordinator
[[ -f "${IDENTITY_ROOT}/coordinator_executable.json" ]] || fail "coordinator measurement failed"

env -i "${freeze_driver_env[@]}" "${PYTHON}" "${DRIVER}" global
env -i "${freeze_driver_env[@]}" \
    "${BOOTSTRAP_TARGET}/release/substrate-r3-macos-candidate-freeze-global-provenance-builder"
readonly GLOBAL_BUILD_SHA256="$(env -i "${freeze_driver_env[@]}" "${PYTHON}" -c 'import json,sys; print(json.load(open(sys.argv[1]))["input_set_sha256"])' "${GLOBAL_BUILD_INPUTS}")"
readonly COORDINATOR_SHA256="$(/usr/bin/shasum -a 256 "${ARTIFACT_ROOT}/substrate-r3-macos-evidence-coordinator" | /usr/bin/awk '{print $1}')"
readonly COORDINATOR_CDHASH="$(env -i "${freeze_driver_env[@]}" "${PYTHON}" -c 'import json,sys; print(json.load(open(sys.argv[1]))["cdhash"])' "${IDENTITY_ROOT}/coordinator_executable.json")"
readonly COORDINATOR_REQUIREMENT="$(env -i "${freeze_driver_env[@]}" "${PYTHON}" -c 'import json,sys; print(json.load(open(sys.argv[1]))["designated_requirement"])' "${IDENTITY_ROOT}/coordinator_executable.json")"

global_env=(
    CARGO_HOME=/Users/spensermcconnell/.cargo CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="${GLOBAL_TARGET}" LANG=C LC_ALL=C
    MACOSX_DEPLOYMENT_TARGET=15.0 PATH=/Users/spensermcconnell/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin
    R3_CAPABILITY_DIGEST="${CAPABILITY_SHA256}" R3_COORDINATOR_ACCOUNT=spensermcconnell
    R3_COORDINATOR_BUILD_INPUTS_SHA256="${COORDINATOR_BUILD_SHA256}" R3_COORDINATOR_CDHASH="${COORDINATOR_CDHASH}"
    R3_COORDINATOR_GID=20
    R3_COORDINATOR_REQUIREMENT="${COORDINATOR_REQUIREMENT}" R3_COORDINATOR_SHA256="${COORDINATOR_SHA256}"
    R3_COORDINATOR_SOURCE_COMMIT="${SOURCE_COMMIT}" R3_COORDINATOR_SOURCE_HASHES_SHA256="${SOURCE_IDENTITY_SHA256}"
    R3_COORDINATOR_SOURCE_TREE="${SOURCE_TREE}" R3_COORDINATOR_UID=501
    R3_FINALIZER_LAUNCHD_PLIST_SHA256="${PLIST_SHA256}" R3_SOURCE_COMMIT="${SOURCE_COMMIT}"
    R3_FREEZE_INPUT_ENCODER="${PYTHON}" R3_FREEZE_INPUT_ENCODER_SHA256="${FREEZE_ENCODER_SHA256}"
    R3_FREEZE_SCRIPT_SHA256="${FREEZE_SCRIPT_SHA256}"
    R3_SOURCE_IDENTITY_SHA256="${SOURCE_IDENTITY_SHA256}" R3_SOURCE_TREE="${SOURCE_TREE}"
    R3_BUILD_INPUTS_SHA256="${GLOBAL_BUILD_SHA256}"
    RUSTFLAGS="-C link-arg=-fuse-ld=${LD} -C linker=${CLANG}"
    RUSTUP_HOME=/Users/spensermcconnell/.rustup
    SDKROOT="$(${XCRUN} --sdk macosx --show-sdk-path)" SOURCE_DATE_EPOCH="$(${GIT} -C "${REPOSITORY}" log -1 --format=%ct)"
    TZ=UTC ZERO_AR_DATE=1
)

env -i "${global_env[@]}" \
    "${CARGO}" build --release --locked \
    --manifest-path "${FINALIZER_MANIFEST}" \
    --bin substrate-r3-macos-evidence-finalizer --bin substrate-r3-macos-peer-code-probe
sign_artifact "${GLOBAL_TARGET}/release/substrate-r3-macos-evidence-finalizer" \
    "${ARTIFACT_ROOT}/substrate-r3-macos-evidence-finalizer" \
    com.atomize.substrate.r3-macos-evidence-finalizer.v2 global
sign_artifact "${GLOBAL_TARGET}/release/substrate-r3-macos-peer-code-probe" \
    "${ARTIFACT_ROOT}/substrate-r3-macos-peer-code-probe" \
    com.atomize.substrate.r3-macos-peer-code-probe.v2 global

env -i "${global_env[@]}" \
    "${CARGO}" build --release --locked \
    --manifest-path "${FINALIZER_MANIFEST}" --features experiment-harness \
    --bin substrate-r3-macos-disposable-harness
sign_artifact "${GLOBAL_TARGET}/release/substrate-r3-macos-disposable-harness" \
    "${ARTIFACT_ROOT}/substrate-r3-macos-disposable-harness" \
    com.atomize.substrate.r3-macos-disposable-harness.v2 global

env -i "${global_env[@]}" \
    "${CARGO}" build --release --locked \
    --manifest-path "${FINALIZER_MANIFEST}" --features candidate-freeze-builder \
    --bin substrate-r3-macos-candidate-freeze-provenance-builder \
    --bin substrate-r3-macos-candidate-freeze-global-provenance-builder \
    --bin substrate-r3-macos-candidate-freeze-manifest-builder

env -i "${global_env[@]}" \
    "${CARGO}" build --release --locked \
    --manifest-path "${SIGNER_MANIFEST}" \
    --bin substrate-r3-macos-signer-acl-creator \
    --bin substrate-r3-macos-signer-acl-wrong-identity \
    --bin substrate-r3-macos-disposable-publisher \
    --bin substrate-r3-macos-disposable-experiment-runner \
    --bin substrate-r3-macos-nobody-owner-probe

# Cargo's final-product dep-info is the compiler-observed local Rust source closure.  Require
# every repo-local dependency of every feature-isolated Rust product to be present in the closed
# source-hash sequence; a newly compiled module cannot silently escape per-source hashing.
env -i PATH="${FIXED_PATH}" LANG=C LC_ALL=C TZ=UTC "${PYTHON}" - \
    "${REPOSITORY}" "${COORDINATOR_TARGET}" "${GLOBAL_TARGET}" \
    "${COORDINATOR_BUILD_INPUTS}" "${GLOBAL_BUILD_INPUTS}" <<'PY'
import json
import pathlib
import re
import shlex
import sys

repository = pathlib.Path(sys.argv[1]).resolve()
targets = [pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3])]
manifests = [pathlib.Path(sys.argv[4]), pathlib.Path(sys.argv[5])]
source = (
    repository
    / "tools/r3-macos-finalizer/src/experiment/freeze_provenance.rs"
).read_text()
match = re.search(
    r'pub const CANDIDATE_FREEZE_SOURCE_PATHS_V2: &\[&str\] = &\[(.*?)\n\];',
    source,
    re.S,
)
if not match:
    raise SystemExit("cannot extract frozen source paths for dep-info closure")
frozen = set(re.findall(r'\s*"([^"]+)",', match.group(1)))
compiled = set()
products = []
for target, manifest_path in zip(targets, manifests):
    manifest = json.loads(manifest_path.read_bytes())
    for recipe in manifest["recipes"]:
        if recipe["tool"] != "cargo":
            continue
        for product in recipe["binary_products"]:
            dep_info = target / "release" / f"{product}.d"
            if not dep_info.is_file() or dep_info.is_symlink():
                raise SystemExit(f"missing nofollow final-product dep-info: {dep_info}")
            products.append(product)
            for token in shlex.split(dep_info.read_text().replace("\\\n", " "), posix=True):
                candidate = pathlib.Path(token.rstrip(":"))
                if not candidate.is_absolute():
                    candidate = repository / candidate
                try:
                    relative = candidate.resolve().relative_to(repository)
                except ValueError:
                    continue
                value = relative.as_posix()
                if value.startswith("target/"):
                    continue
                compiled.add(value)
                if not candidate.is_file() or candidate.is_symlink():
                    raise SystemExit(
                        f"dep-info local input is not one nofollow regular file: {value}"
                    )
missing = sorted(compiled - frozen)
if missing:
    raise SystemExit(f"compiled local source escaped the frozen hash closure: {missing}")
if len(products) != 12 or len(products) != len(set(products)) or not compiled:
    raise SystemExit(
        f"dep-info product/source cardinality changed: products={products}, sources={len(compiled)}"
    )
PY
sign_artifact "${GLOBAL_TARGET}/release/substrate-r3-macos-signer-acl-creator" \
    "${ARTIFACT_ROOT}/substrate-r3-macos-signer-acl-creator" \
    com.atomize.substrate.r3-macos-signer-acl-creator.v1 global
sign_artifact "${GLOBAL_TARGET}/release/substrate-r3-macos-signer-acl-wrong-identity" \
    "${ARTIFACT_ROOT}/substrate-r3-macos-signer-acl-wrong-identity" \
    com.atomize.substrate.r3-macos-signer-acl-wrong-identity.v1 global
sign_artifact "${GLOBAL_TARGET}/release/substrate-r3-macos-disposable-publisher" \
    "${ARTIFACT_ROOT}/substrate-r3-macos-disposable-publisher" \
    com.atomize.substrate.r3-macos-disposable-publisher.v2 global
sign_artifact "${GLOBAL_TARGET}/release/substrate-r3-macos-disposable-experiment-runner" \
    "${ARTIFACT_ROOT}/substrate-r3-macos-disposable-experiment-runner" \
    com.atomize.substrate.r3-macos-disposable-experiment-runner.v2 global
sign_artifact "${GLOBAL_TARGET}/release/substrate-r3-macos-nobody-owner-probe" \
    "${ARTIFACT_ROOT}/substrate-r3-macos-nobody-owner-probe" \
    com.atomize.substrate.r3-macos-nobody-owner-probe.v2 global

env -i "${global_env[@]}" "${SWIFTC}" -parse-as-library -O -whole-module-optimization \
    -target arm64-apple-macos15.0 \
    -o "${WORK_ROOT}/substrate-r3-macos-securityagent-observer" \
    tools/r3-macos-finalizer/native/securityagent_observer.swift
sign_artifact "${WORK_ROOT}/substrate-r3-macos-securityagent-observer" \
    "${ARTIFACT_ROOT}/substrate-r3-macos-securityagent-observer" \
    com.atomize.substrate.r3-macos-securityagent-observer.v2 global

env -i "${global_env[@]}" "${CLANG}" -dynamiclib -Os -target arm64-apple-macos15.0 \
    "-fuse-ld=${LD}" \
    -o "${WORK_ROOT}/substrate-r3-macos-benign-injection-probe.dylib" \
    tools/r3-macos-signer-acl/native/benign_injection_probe.c
sign_artifact "${WORK_ROOT}/substrate-r3-macos-benign-injection-probe.dylib" \
    "${ARTIFACT_ROOT}/substrate-r3-macos-benign-injection-probe.dylib" \
    com.atomize.substrate.r3-macos-benign-injection-probe.v2 global

# Measure the complete artifact set only after all signed bytes are immutable.
/bin/rm -f "${IDENTITY_ROOT}"/*.json
env -i "${freeze_driver_env[@]}" "${PYTHON}" "${DRIVER}" measure-all

# The production finalizer build must not contain the experiment-only private signing path.
for binary in substrate-r3-macos-evidence-finalizer substrate-r3-macos-evidence-coordinator substrate-r3-macos-peer-code-probe; do
    strings_scan="${WORK_ROOT}/${binary}.strings.v2.txt"
    symbols_scan="${WORK_ROOT}/${binary}.symbols.v2.txt"
    /usr/bin/strings "${ARTIFACT_ROOT}/${binary}" > "${strings_scan}"
    /usr/bin/nm -m "${ARTIFACT_ROOT}/${binary}" > "${symbols_scan}"
    [[ -s "${strings_scan}" && -s "${symbols_scan}" ]] \
        || fail "private-signing hygiene scan produced no evidence for ${binary}"
    if /usr/bin/grep -E \
        'EphemeralHarnessSigner|ed25519_dalek.*SigningKey|SigningKey::from_bytes|harness-ed25519-seed|generate_seed_in_memory' \
        "${strings_scan}" "${symbols_scan}"; then
        fail "private signing implementation is reachable from ${binary}"
    fi
done

env -i "${freeze_driver_env[@]}" "${PYTHON}" "${DRIVER}" observe

# This acyclic projection is the root installation authority. It binds the candidate tree,
# source/build/provenance manifests, exact signed artifact set, capability/plist, raw root-time
# absence evidence, directory set, and rollback plan before the reviewed block exists. The final
# manifest may then bind the reviewed block SHA without a manifest/block hash cycle.
readonly ROOT_INSTALL_AUTHORITY_SHA256="$(env -i "${freeze_driver_env[@]}" \
    "${PYTHON}" "${DRIVER}" root-authority)"
[[ "${ROOT_INSTALL_AUTHORITY_SHA256}" =~ ^[0-9a-f]{64}$ ]] \
    || fail "root-install authority projection did not produce one SHA-256"

# This is the only administrator command. It seals the exact root program's physical identity,
# length, and digest before publishing a route. At execution the root loader opens that source
# exactly once with O_NOFOLLOW, hashes and compiles the one descriptor read, and executes those
# same in-memory bytes. The route is the closed resume/rollback route after an administrator crash.
readonly ROOT_INSTALL_SOURCE="scripts/mac/r3-macos-finalizer-root-install.py"
[[ -f "${ROOT_INSTALL_SOURCE}" && ! -L "${ROOT_INSTALL_SOURCE}" ]] \
    || fail "sealed root-install source is not one regular repository file"
# R3_SEALED_RECOVERY_ROUTE_GENERATOR_V1_BEGIN
env -i PATH="${FIXED_PATH}" LANG=C LC_ALL=C TZ=UTC PYTHONDONTWRITEBYTECODE=1 \
    "${PYTHON}" - \
    "${REPOSITORY}/${ROOT_INSTALL_SOURCE}" \
    "${ADMIN_BLOCK}" \
    "${ROOT_INSTALL_AUTHORITY_SHA256}" \
    "$(/usr/bin/id -u)" \
    "$(/usr/bin/id -g)" <<'PY'
import hashlib
import os
import shlex
import stat
import sys

TERMINAL_EXECUTED_RECOVERY_ROUTE_SHA256S = frozenset(
    {
        "deea3cb3aaf05dd641936bdb98bcc2d3098504d64b455794af689cabe68cd76a",
        "bd98dc77cfde7f6956f43845294edeb9da5879525def1bcef5d6c8a3133d839a",
        "7eb9e26e6fb61b10297b85d1b925b980de77c89cc3fe53bc022c91ca04d723c9",
        "f9959128723ed4b18ad9ed7960dbb63788f51f386532265321940cf8e29b141e",
    }
)
TERMINAL_EXECUTED_RECOVERY_SOURCE_SHA256S = frozenset(
    {
        "4ae8266e919e0d202e785009695f61820467d73b3836adf9ac92741ac5624698",
        "5416f9df95e34e480f3f45460c500d8f72ed287b8b21e915b8968f09091c69fd",
        "d07da7afca829c710fd4f5416250df661a6b9699ea9a2938c93f8443181e1ae3",
        "e808fc84170cd43b7b62872f7b0479157e727b3fce31f25e140d2c3364ce868c",
        "f962f6777dc5bc7a327f1c30b406851aaca7151b492e9ba38802ffd565566dc9",
    }
)


def is_terminal_executed_recovery_route(route_sha256):
    return route_sha256 in TERMINAL_EXECUTED_RECOVERY_ROUTE_SHA256S


SEALED_RECOVERY_LOADER = r'''import hashlib
import os
import stat
import sys

REQUIRED_ROOT_COMMANDS = ("/usr/bin/env", "/bin/zsh", "/usr/bin/python3")


def command_identity(value):
    return (
        value.st_dev,
        value.st_ino,
        value.st_uid,
        value.st_gid,
        stat.S_IMODE(value.st_mode),
        value.st_nlink,
        value.st_size,
        value.st_mtime_ns,
        value.st_ctime_ns,
    )


def require_sealed_root_command(path):
    try:
        before = os.lstat(path)
    except OSError as error:
        raise SystemExit(
            f"sealed recovery required root command is unavailable: {path}: errno={error.errno}"
        ) from error
    if (
        not stat.S_ISREG(before.st_mode)
        or before.st_uid != 0
        or stat.S_IMODE(before.st_mode) & 0o111 == 0
        or before.st_nlink < 1
        or before.st_size < 1
    ):
        raise SystemExit(
            f"sealed recovery required root command has invalid posture: {path}"
        )
    try:
        after = os.lstat(path)
    except OSError as error:
        raise SystemExit(
            f"sealed recovery required root command changed after inspection: {path}: errno={error.errno}"
        ) from error
    if command_identity(after) != command_identity(before):
        raise SystemExit(
            f"sealed recovery required root command changed during inspection: {path}"
        )


# Validate literal command paths inside the same sanitized root process that loads the route.
# State-specific absence probes stay in the sealed Python source, where descriptor-relative
# lstat checks cannot be replaced by a shell utility whose location or failure may be masked.
for required_root_command in REQUIRED_ROOT_COMMANDS:
    require_sealed_root_command(required_root_command)

if len(sys.argv) != 12:
    raise SystemExit("sealed recovery loader received the wrong argument count")

source_path = sys.argv[1]
expected_sha256 = sys.argv[2]
expected_size = int(sys.argv[3])
expected_identity = tuple(int(value) for value in sys.argv[4:12])
if expected_size < 1 or expected_size > 4 * 1024 * 1024:
    raise SystemExit("sealed recovery source size is outside the closed bound")
if len(expected_sha256) != 64 or any(
    character not in "0123456789abcdef" for character in expected_sha256
):
    raise SystemExit("sealed recovery source SHA-256 is noncanonical")


def identity(value):
    return (
        value.st_dev,
        value.st_ino,
        value.st_uid,
        value.st_gid,
        stat.S_IMODE(value.st_mode),
        value.st_nlink,
        value.st_mtime_ns,
        value.st_ctime_ns,
    )


flags = os.O_RDONLY | os.O_NOFOLLOW
if hasattr(os, "O_CLOEXEC"):
    flags |= os.O_CLOEXEC
fd = os.open(source_path, flags)
try:
    before = os.fstat(fd)
    if not stat.S_ISREG(before.st_mode) or before.st_size != expected_size:
        raise SystemExit("sealed recovery source is not the exact expected regular file")
    if identity(before) != expected_identity:
        raise SystemExit("sealed recovery source physical identity changed")
    chunks = []
    remaining = expected_size + 1
    while remaining:
        chunk = os.read(fd, min(131072, remaining))
        if not chunk:
            break
        chunks.append(chunk)
        remaining -= len(chunk)
    after = os.fstat(fd)
finally:
    os.close(fd)

if identity(after) != identity(before) or after.st_size != before.st_size:
    raise SystemExit("sealed recovery source changed during its descriptor read")
path_after = os.stat(source_path, follow_symlinks=False)
if identity(path_after) != identity(before) or path_after.st_size != before.st_size:
    raise SystemExit("sealed recovery source pathname identity changed")
source_bytes = b"".join(chunks)
if len(source_bytes) != expected_size:
    raise SystemExit("sealed recovery source length changed")
if hashlib.sha256(source_bytes).hexdigest() != expected_sha256:
    raise SystemExit("sealed recovery source SHA-256 changed")

sys.argv = [source_path]
namespace = {
    "__builtins__": __builtins__,
    "__file__": source_path,
    "__name__": "__main__",
    "__package__": None,
}
compiled = compile(source_bytes, source_path, "exec", dont_inherit=True)
exec(compiled, namespace, namespace)
'''


def physical_identity(value):
    return (
        value.st_dev,
        value.st_ino,
        value.st_uid,
        value.st_gid,
        stat.S_IMODE(value.st_mode),
        value.st_nlink,
        value.st_mtime_ns,
        value.st_ctime_ns,
    )


source_path = sys.argv[1]
admin_block = sys.argv[2]
authority_sha256 = sys.argv[3]
expected_uid = int(sys.argv[4])
expected_gid = int(sys.argv[5])
flags = os.O_RDONLY | os.O_NOFOLLOW
if hasattr(os, "O_CLOEXEC"):
    flags |= os.O_CLOEXEC
source_fd = os.open(source_path, flags)
try:
    source_before = os.fstat(source_fd)
    if (
        not stat.S_ISREG(source_before.st_mode)
        or source_before.st_uid != expected_uid
        or source_before.st_gid != expected_gid
        or stat.S_IMODE(source_before.st_mode) != 0o644
        or source_before.st_nlink != 1
        or source_before.st_size < 1
        or source_before.st_size > 4 * 1024 * 1024
    ):
        raise SystemExit("root-install source physical identity is outside the closed plan")
    source_chunks = []
    remaining = source_before.st_size + 1
    while remaining:
        chunk = os.read(source_fd, min(131072, remaining))
        if not chunk:
            break
        source_chunks.append(chunk)
        remaining -= len(chunk)
    source_after = os.fstat(source_fd)
finally:
    os.close(source_fd)
if (
    physical_identity(source_after) != physical_identity(source_before)
    or source_after.st_size != source_before.st_size
):
    raise SystemExit("root-install source changed while sealing the recovery route")
source_bytes = b"".join(source_chunks)
if len(source_bytes) != source_before.st_size:
    raise SystemExit("root-install source length changed while sealing the recovery route")
source_sha256 = hashlib.sha256(source_bytes).hexdigest()
if source_sha256 in TERMINAL_EXECUTED_RECOVERY_SOURCE_SHA256S:
    raise SystemExit("refusing to reuse a terminal executed recovery source")
identity_arguments = " ".join(str(value) for value in physical_identity(source_before))
route = f'''set -u
readonly R3_SEALED_RECOVERY_LOADER={shlex.quote(SEALED_RECOVERY_LOADER)}
exec /usr/bin/sudo /usr/bin/env -i PATH=/usr/bin:/bin:/usr/sbin:/sbin LANG=C LC_ALL=C TZ=UTC \\
    R3_ROOT_INSTALL_AUTHORITY_SHA256={authority_sha256} \\
    /bin/zsh -c 'cd / && exec /usr/bin/python3 -c "$1" "${{@:2}}"' -- \\
    "${{R3_SEALED_RECOVERY_LOADER}}" \\
    {shlex.quote(source_path)} {source_sha256} {len(source_bytes)} {identity_arguments}
'''.encode("utf-8")
route_sha256 = hashlib.sha256(route).hexdigest()
if is_terminal_executed_recovery_route(route_sha256):
    raise SystemExit("refusing to regenerate a terminal executed recovery route")

admin_flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW
admin_fd = os.open(admin_block, admin_flags, 0o400)
try:
    offset = 0
    while offset < len(route):
        written = os.write(admin_fd, route[offset:])
        if written <= 0:
            raise SystemExit("reviewed recovery route write made no progress")
        offset += written
    os.fsync(admin_fd)
finally:
    os.close(admin_fd)
parent_fd = os.open(os.path.dirname(admin_block), os.O_RDONLY | os.O_DIRECTORY)
try:
    os.fsync(parent_fd)
finally:
    os.close(parent_fd)
PY
# R3_SEALED_RECOVERY_ROUTE_GENERATOR_V1_END

/bin/chmod 0400 "${ADMIN_BLOCK}"
env -i "${freeze_driver_env[@]}" "${PYTHON}" "${DRIVER}" final

# Re-run the globally built validators against every stage and publish the immutable final manifest.
env -i "${freeze_driver_env[@]}" \
    "${GLOBAL_TARGET}/release/substrate-r3-macos-candidate-freeze-provenance-builder"
env -i "${freeze_driver_env[@]}" \
    "${GLOBAL_TARGET}/release/substrate-r3-macos-candidate-freeze-global-provenance-builder"
env -i "${freeze_driver_env[@]}" \
    "${GLOBAL_TARGET}/release/substrate-r3-macos-candidate-freeze-manifest-builder"

[[ -s "${FINAL_MANIFEST}" ]] || fail "final candidate manifest was not created"
readonly MANIFEST_SHA256="$(/usr/bin/shasum -a 256 "${FINAL_MANIFEST}" | /usr/bin/awk '{print $1}')"
readonly ARTIFACT_SET_SHA256="$(env -i "${freeze_driver_env[@]}" "${PYTHON}" -c 'import json,sys; print(json.load(open(sys.argv[1]))["artifact_set_sha256"])' "${FINAL_MANIFEST}")"
/bin/chmod 0400 "${FREEZE_ROOT}"/*.json "${ADMIN_BLOCK}" "${ARTIFACT_ROOT}"/*

# Rebind the committed identity after all builds and require the same clean HEAD/tree pair.
readonly REOBSERVED_SOURCE_TREE="$(env -i PATH="${FIXED_PATH}" LANG=C LC_ALL=C TZ=UTC \
    "${PYTHON}" - "${REPOSITORY}" "${GIT}" "${SOURCE_COMMIT}" <<'PY'
import pathlib
import subprocess
import sys

repository = pathlib.Path(sys.argv[1])
git = sys.argv[2]
commit = sys.argv[3]
observed_commit = subprocess.check_output(
    [git, "-C", str(repository), "rev-parse", "HEAD"], text=True
).strip()
status = subprocess.check_output(
    [git, "-C", str(repository), "status", "--porcelain=v2", "-z"]
)
if observed_commit != commit or status:
    raise SystemExit("committed candidate identity changed during freeze")
print(subprocess.check_output(
    [git, "-C", str(repository), "rev-parse", "HEAD^{tree}"], text=True
).strip())
PY
)"
[[ "${REOBSERVED_SOURCE_TREE}" == "${SOURCE_TREE}" ]] \
    || fail "candidate source tree changed while the exact artifacts were built"

# Work products are not evidence and are removed before the packet is offered for administrator
# review.  Removal is exact and bounded to the freshly created candidate work directory.
[[ "${WORK_ROOT}" == "${FREEZE_ROOT}/.freeze-work.v2" \
   && ! -L "${WORK_ROOT}" && -d "${WORK_ROOT}" \
   && "$(/usr/bin/stat -f '%u:%g:%Lp' "${WORK_ROOT}")" == '501:20:700' ]] \
    || fail "candidate work root lost its exact bounded identity"
/bin/rm -rf -- "${WORK_ROOT}"
[[ ! -e "${WORK_ROOT}" && ! -L "${WORK_ROOT}" ]] || fail "candidate work root remains"

trap - EXIT
print -- "R3 candidate freeze complete"
print -- "source_commit=${SOURCE_COMMIT}"
print -- "candidate_source_tree=${SOURCE_TREE}"
print -- "source_hashes_sha256=${SOURCE_IDENTITY_SHA256}"
print -- "coordinator_build_inputs_sha256=${COORDINATOR_BUILD_SHA256}"
print -- "global_build_inputs_sha256=${GLOBAL_BUILD_SHA256}"
print -- "candidate_manifest=${FINAL_MANIFEST}"
print -- "candidate_manifest_sha256=${MANIFEST_SHA256}"
print -- "artifact_set_sha256=${ARTIFACT_SET_SHA256}"
print -- "reviewed_admin_block=${ADMIN_BLOCK}"
