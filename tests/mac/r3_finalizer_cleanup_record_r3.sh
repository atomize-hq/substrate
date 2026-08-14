#!/bin/zsh
set -euo pipefail

SCRIPT_DIR="${0:A:h}"
REPOSITORY="${SCRIPT_DIR:h:h}"
PYTHON="/usr/bin/python3"
WORK_ROOT="$(/usr/bin/mktemp -d "/private/tmp/substrate-r3-rcv-02.XXXXXX")"

cleanup() {
    /bin/rm -rf -- "${WORK_ROOT}"
}
trap cleanup EXIT

env -i \
    PATH=/usr/bin:/bin:/usr/sbin:/sbin \
    LANG=C \
    LC_ALL=C \
    TZ=UTC \
    PYTHONDONTWRITEBYTECODE=1 \
    "${PYTHON}" - \
    "${REPOSITORY}/scripts/mac/r3-macos-finalizer-root-install.py" \
    "${WORK_ROOT}" <<'PY'
import copy
import errno
import importlib.util
import os
import pathlib
import stat
import sys


source = pathlib.Path(sys.argv[1])
root = pathlib.Path(sys.argv[2])
spec = importlib.util.spec_from_file_location("r3_rcv_02_root_installer", source)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def reject(call, label):
    try:
        call()
    except module.Stop:
        return
    raise SystemExit(f"cleanup-record regression accepted {label}")


def bound_observation(probe, raw):
    predicate_sha256 = module.document_sha256(probe)
    raw_sha256 = module.document_sha256(raw)
    return {
        "predicate_sha256": predicate_sha256,
        "raw_observation": raw,
        "raw_observation_sha256": raw_sha256,
        "observation_sha256": module.document_sha256(
            [predicate_sha256, raw_sha256, probe["classification"]]
        ),
    }


artifact_path = root / "canonical-installed-artifact"
artifact_bytes = b"r3-rcv-02-canonical-installed-artifact\n"
artifact_path.write_bytes(artifact_bytes)
artifact_path.chmod(0o400)
artifact_observed = os.lstat(artifact_path)
artifact_identity = module.physical_identity(artifact_path, artifact_observed)
artifact_manifest = {
    "role": "capability_manifest",
    "intended_path": str(artifact_path),
    "uid": artifact_observed.st_uid,
    "gid": artifact_observed.st_gid,
    "mode": 0o400,
    "size": len(artifact_bytes),
    "sha256": module.sha_bytes(artifact_bytes),
}
artifact_record = {
    "role": artifact_manifest["role"],
    "path": artifact_manifest["intended_path"],
    "physical_identity": artifact_identity,
    "sha256": artifact_manifest["sha256"],
    "executable_identity": None,
    "signing_posture": None,
}

# Exercise the exact serialized producer shape: identity facts live only in the nested
# physical_identity object.  The outer object retains the role/path/hash/code bindings.
artifact_record = module.parse_canonical(
    module.canonical(artifact_record), "canonical installed-artifact fixture"
)
if any(field in artifact_record for field in ("uid", "gid", "mode", "size")):
    raise SystemExit("canonical installed-artifact fixture relocated physical identity")
module.validate_root_install_artifact_identity(artifact_record, artifact_manifest)

module.ARTIFACT_SPECS = [
    (
        artifact_manifest["role"],
        "canonical-installed-artifact",
        artifact_manifest["intended_path"],
        artifact_manifest["uid"],
        artifact_manifest["gid"],
        artifact_manifest["mode"],
        None,
    )
]
module.INSTALL_DIRECTORIES = []
digest = "1" * 64
manifest = {
    "reviewed_admin_block_sha256": "2" * 64,
    "artifact_set_sha256": "3" * 64,
    "install_parent_directory_set_sha256": "4" * 64,
    "preinstall_absence_observation_set_sha256": "5" * 64,
    "rollback_plan_sha256": "6" * 64,
    "source_hashes_manifest_sha256": digest,
    "coordinator_build_input_manifest_sha256": digest,
    "global_build_input_manifest_sha256": digest,
    "coordinator_provenance_input_sha256": digest,
    "global_provenance_input_sha256": digest,
    "manifest_input_sha256": digest,
    "artifacts": [artifact_manifest],
}
manifest_bytes = module.canonical(manifest)
preclaim = {"schema_owner": "r3-rcv-02-canonical-preclaim-fixture"}
launchd_probe = next(
    probe for probe in module.absence_plan() if probe["kind"] == "launchd_label"
)
endpoint_probe = next(
    probe for probe in module.absence_plan() if probe["kind"] == "unix_endpoint"
)
launchd_raw = {
    "kind": "launchd_label",
    "identity": launchd_probe["identity"],
    "raw_exit_status": 113,
    "stdout": module.raw_stream(b""),
    "stderr": module.raw_stream(
        (
            "Bad request.\nCould not find service "
            f'"{launchd_probe["identity"]}" in domain for system\n'
        ).encode()
    ),
}
endpoint_raw = {
    "kind": "unix_endpoint",
    "identity": endpoint_probe["identity"],
    "lstat_return": -1,
    "raw_errno": errno.ENOENT,
    "stat_result": None,
}
completion = {
    "schema_owner": "substrate.r3-macos-finalizer-root-install-completion",
    "schema_version": 2,
    "experiment_id": module.EXPERIMENT_ID,
    "sequence": 2,
    "preclaim_sha256": module.document_sha256(preclaim),
    "candidate_freeze_manifest_sha256": module.sha_bytes(manifest_bytes),
    "reviewed_admin_block_sha256": manifest["reviewed_admin_block_sha256"],
    "candidate_artifact_set_sha256": manifest["artifact_set_sha256"],
    "install_parent_directory_set_sha256": manifest[
        "install_parent_directory_set_sha256"
    ],
    "preinstall_absence_observation_set_sha256": manifest[
        "preinstall_absence_observation_set_sha256"
    ],
    "rollback_plan_sha256": manifest["rollback_plan_sha256"],
    "supporting_manifest_set_sha256": module.supporting_manifest_set_sha256(manifest),
    "installed_directories": [],
    "installed_directory_set_sha256": module.document_sha256([]),
    "installed_artifacts": [artifact_record],
    "installed_artifact_identity_set_sha256": module.document_sha256(
        [artifact_record]
    ),
    "preclaim_leaf_identity": artifact_identity,
    "completion_leaf_lstat_return": -1,
    "completion_leaf_raw_errno": errno.ENOENT,
    "launchd_label_absence": bound_observation(launchd_probe, launchd_raw),
    "endpoint_absence": bound_observation(endpoint_probe, endpoint_raw),
    "install_complete": True,
    "launchd_bootstrap_authorized": False,
}
completion = module.parse_canonical(
    module.canonical(completion), "canonical root-install completion fixture"
)
module.validate_completion(completion, manifest, manifest_bytes, preclaim)


def changed_artifact(mutator):
    changed = copy.deepcopy(completion)
    mutator(changed["installed_artifacts"][0])
    changed["installed_artifact_identity_set_sha256"] = module.document_sha256(
        changed["installed_artifacts"]
    )
    return changed


changed = copy.deepcopy(completion)
changed["unknown"] = True
reject(
    lambda: module.validate_completion(changed, manifest, manifest_bytes, preclaim),
    "an outer completion field",
)
reject(
    lambda: module.validate_completion(
        changed_artifact(lambda artifact: artifact.__setitem__("unknown", True)),
        manifest,
        manifest_bytes,
        preclaim,
    ),
    "an outer installed-artifact field",
)
reject(
    lambda: module.validate_completion(
        changed_artifact(
            lambda artifact: artifact["physical_identity"].__setitem__("unknown", True)
        ),
        manifest,
        manifest_bytes,
        preclaim,
    ),
    "a nested physical-identity field",
)


def relocate_uid(artifact):
    artifact["uid"] = artifact["physical_identity"].pop("uid")


reject(
    lambda: module.validate_completion(
        changed_artifact(relocate_uid), manifest, manifest_bytes, preclaim
    ),
    "a relocated physical-identity field",
)
reject(
    lambda: module.validate_completion(
        changed_artifact(lambda artifact: artifact.__setitem__("sha256", "f" * 64)),
        manifest,
        manifest_bytes,
        preclaim,
    ),
    "an installed-artifact hash substitution",
)
reject(
    lambda: module.validate_completion(
        changed_artifact(
            lambda artifact: artifact["physical_identity"].__setitem__(
                "size", artifact["physical_identity"]["size"] + 1
            )
        ),
        manifest,
        manifest_bytes,
        preclaim,
    ),
    "an installed-artifact size substitution",
)
reject(
    lambda: module.validate_completion(
        changed_artifact(
            lambda artifact: artifact["physical_identity"].__setitem__(
                "mode", stat.S_IFREG | 0o600
            )
        ),
        manifest,
        manifest_bytes,
        preclaim,
    ),
    "an installed-artifact mode substitution",
)

# Device/inode observations have no manifest-side value.  They are instead joined to the live
# no-follow object by the cleanup consumer; a substituted observation must preserve the file.
substituted = copy.deepcopy(artifact_record)
substituted["physical_identity"]["inode"] += 1
reject(
    lambda: module.unlink_exact_file(artifact_path, substituted),
    "an installed-artifact physical identity substitution",
)
if not artifact_path.exists():
    raise SystemExit("physical-identity substitution removed the installed artifact")
module.unlink_exact_file(artifact_path, artifact_record)
if artifact_path.exists() or artifact_path.is_symlink():
    raise SystemExit("canonical nested installed-artifact record did not remove its exact file")

# Build the complete installed-manifest binding shape.  Its mode is intentionally permission-only
# (0400), while physical_identity records the regular-file type plus 0400.
installed_manifest = root / "candidate-artifact-manifest.v2.json"
installed_manifest.write_bytes(manifest_bytes)
installed_manifest.chmod(0o400)
installed_observed = os.lstat(installed_manifest)
module.INSTALLED_CANDIDATE_MANIFEST_PATH = installed_manifest
installed_physical = {
    "path": str(installed_manifest),
    "sha256": module.sha_bytes(manifest_bytes),
    "device": installed_observed.st_dev,
    "inode": installed_observed.st_ino,
    "owner_uid": 0,
    "owner_gid": 0,
    "mode": 0o400,
    "link_count": installed_observed.st_nlink,
    "size": installed_observed.st_size,
}
installed_physical_sha256 = module.document_sha256(installed_physical)
installed_manifest_record = {
    "external_manifest_path": str(module.MANIFEST_PATH),
    "installed_manifest_path": str(installed_manifest),
    "external_manifest_sha256": module.sha_bytes(manifest_bytes),
    "installed_manifest_sha256": module.sha_bytes(manifest_bytes),
    "reviewed_admin_block_sha256": manifest["reviewed_admin_block_sha256"],
    "ancestor_chain_before_sha256": "7" * 64,
    "ancestor_chain_after_sha256": "7" * 64,
    "pathname_before_identity_sha256": installed_physical_sha256,
    "descriptor_identity_sha256": installed_physical_sha256,
    "pathname_after_identity_sha256": installed_physical_sha256,
    "device": installed_observed.st_dev,
    "inode": installed_observed.st_ino,
    "owner_uid": 0,
    "owner_gid": 0,
    "mode": 0o400,
    "link_count": installed_observed.st_nlink,
    "size": installed_observed.st_size,
}
module.validate_installed_candidate_manifest_binding(
    installed_manifest_record, manifest, manifest_bytes
)

global_exchange = root / "global-publisher-exchange"
global_exchange.mkdir(mode=0o700)
global_identity = module.physical_identity(global_exchange, os.lstat(global_exchange))
module.GLOBAL_EXCHANGE = global_exchange
module.INSTALL_DIRECTORIES = [(str(global_exchange), 501, 0, 0o700)]
cleanup_completion = copy.deepcopy(completion)
cleanup_completion["installed_directories"] = [global_identity]
original_read_exact_file = module.read_exact_file
original_physical_identity = module.physical_identity


def read_installed_manifest(path, expected_uid, expected_gid, expected_mode):
    if path != installed_manifest or expected_uid != 0 or expected_gid != 0:
        raise SystemExit("cleanup regression escaped its installed-manifest path/owner fence")
    return original_read_exact_file(
        path, installed_observed.st_uid, installed_observed.st_gid, expected_mode
    )


def root_owned_installed_identity(path, observed):
    identity = original_physical_identity(path, observed)
    if path == installed_manifest:
        identity["uid"] = 0
        identity["gid"] = 0
    return identity


module.read_exact_file = read_installed_manifest
module.physical_identity = root_owned_installed_identity
module.directory_identity = lambda path, uid, gid, mode: global_identity
module.cleanup_installed_candidate(
    manifest, manifest_bytes, cleanup_completion, installed_manifest_record
)
if installed_manifest.exists() or installed_manifest.is_symlink():
    raise SystemExit("permission-only installed-manifest record did not remove its exact file")

global_exchange.rmdir()
print("r3_rcv_02_cleanup_record_regression=PASS")
PY
