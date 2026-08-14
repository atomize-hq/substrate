#!/bin/zsh
set -euo pipefail

SCRIPT_DIR="${0:A:h}"
REPOSITORY="${SCRIPT_DIR:h:h}"
FIXTURE="${SCRIPT_DIR}/fixtures/r3_finalizer_cleanup_record_actual_v2.json"
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
    /usr/bin/python3 - \
    "${REPOSITORY}/scripts/mac/r3-macos-finalizer-root-install.py" \
    "${FIXTURE}" \
    "${WORK_ROOT}" <<'PY'
import copy
import hashlib
import importlib.util
import json
import os
import pathlib
import sys


source = pathlib.Path(sys.argv[1])
fixture_path = pathlib.Path(sys.argv[2])
root = pathlib.Path(sys.argv[3])
spec = importlib.util.spec_from_file_location("r3_rcv_02_root_installer", source)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def reject(call, label):
    try:
        call()
    except module.Stop:
        return
    raise SystemExit(f"cleanup-record regression accepted {label}")


# This immutable projection contains the exact 14 installed_artifacts objects serialized by the
# preserved native completion.  It deliberately excludes the preclaim, process snapshots, and the
# rest of the 565-KiB candidate manifest.  The source-document hashes and the projection hash make
# the provenance and every retained byte independently reviewable.
fixture_bytes = fixture_path.read_bytes()
if hashlib.sha256(fixture_bytes).hexdigest() != (
    "8e6aeaf410d5038da2d48288b1445da08a0a21c4d98442744fb6d8a5f925c6f3"
):
    raise SystemExit("actual cleanup-record fixture bytes changed")
fixture = json.loads(fixture_bytes)
if set(fixture) != {
    "schema_owner",
    "schema_version",
    "source_completion_sha256",
    "source_manifest_sha256",
    "source_completion_installed_artifact_identity_set_sha256",
    "projection_sha256",
    "completion_outer_fields",
    "installed_artifacts",
    "manifest_artifact_authorities",
}:
    raise SystemExit("actual cleanup-record fixture changed its closed shape")
if (
    fixture["schema_owner"]
    != "substrate.r3-macos-finalizer-cleanup-record-regression-fixture"
    or fixture["schema_version"] != 1
    or fixture["source_completion_sha256"]
    != "429a1e1ba7b5a7ad077d0771a2de7f4290ba826c54717502d496517cf476025c"
    or fixture["source_manifest_sha256"]
    != "331756c8c4265b30c0b5647841aa1330844e3544d7603a557428be69c1cbc95c"
):
    raise SystemExit("actual cleanup-record fixture changed source provenance")
projection = {
    "completion_outer_fields": fixture["completion_outer_fields"],
    "installed_artifacts": fixture["installed_artifacts"],
    "manifest_artifact_authorities": fixture["manifest_artifact_authorities"],
}
if module.document_sha256(projection) != fixture["projection_sha256"]:
    raise SystemExit("actual cleanup-record fixture changed its projection digest")
if module.document_sha256(fixture["installed_artifacts"]) != fixture[
    "source_completion_installed_artifact_identity_set_sha256"
]:
    raise SystemExit("actual installed-artifact set differs from the source completion")

expected_completion_fields = {
    "schema_owner",
    "schema_version",
    "experiment_id",
    "sequence",
    "preclaim_sha256",
    "candidate_freeze_manifest_sha256",
    "reviewed_admin_block_sha256",
    "candidate_artifact_set_sha256",
    "install_parent_directory_set_sha256",
    "preinstall_absence_observation_set_sha256",
    "rollback_plan_sha256",
    "supporting_manifest_set_sha256",
    "installed_directories",
    "installed_directory_set_sha256",
    "installed_artifacts",
    "installed_artifact_identity_set_sha256",
    "preclaim_leaf_identity",
    "completion_leaf_lstat_return",
    "completion_leaf_raw_errno",
    "launchd_label_absence",
    "endpoint_absence",
    "install_complete",
    "launchd_bootstrap_authorized",
}
if fixture["completion_outer_fields"] != sorted(expected_completion_fields):
    raise SystemExit("actual completion outer shape differs from the canonical validator")

artifacts = fixture["installed_artifacts"]
authorities = fixture["manifest_artifact_authorities"]
roles = [specification[0] for specification in module.ARTIFACT_SPECS]
if (
    len(artifacts) != 14
    or len(authorities) != 14
    or [artifact["role"] for artifact in artifacts] != roles
    or [authority["role"] for authority in authorities] != roles
):
    raise SystemExit("actual completion changed its 14-role canonical order")
for artifact, authority, specification in zip(
    artifacts, authorities, module.ARTIFACT_SPECS
):
    if set(authority) != {
        "role",
        "intended_path",
        "uid",
        "gid",
        "mode",
        "size",
        "sha256",
    }:
        raise SystemExit("manifest artifact projection changed its closed authority shape")
    role, _, path, uid, gid, mode, _ = specification
    if (
        authority["role"] != role
        or authority["intended_path"] != path
        or authority["uid"] != uid
        or authority["gid"] != gid
        or authority["mode"] != mode
    ):
        raise SystemExit("manifest artifact projection differs from the compiled plan")
    module.validate_root_install_artifact_identity(artifact, authority)


def mutated_artifact(mutator):
    artifact = copy.deepcopy(artifacts[0])
    mutator(artifact)
    return artifact


reject(
    lambda: module.validate_root_install_artifact_identity(
        mutated_artifact(lambda value: value.__setitem__("unknown", True)),
        authorities[0],
    ),
    "an outer installed-artifact field",
)
reject(
    lambda: module.validate_root_install_artifact_identity(
        mutated_artifact(
            lambda value: value["physical_identity"].__setitem__("unknown", True)
        ),
        authorities[0],
    ),
    "a nested physical-identity field",
)


def relocate_uid(value):
    value["uid"] = value["physical_identity"].pop("uid")


reject(
    lambda: module.validate_root_install_artifact_identity(
        mutated_artifact(relocate_uid), authorities[0]
    ),
    "a relocated physical-identity field",
)
for field, replacement in (
    ("sha256", "f" * 64),
    ("path", artifacts[1]["path"]),
    ("role", artifacts[1]["role"]),
):
    reject(
        lambda field=field, replacement=replacement: (
            module.validate_root_install_artifact_identity(
                mutated_artifact(
                    lambda value: value.__setitem__(field, replacement)
                ),
                authorities[0],
            )
        ),
        f"an installed-artifact {field} substitution",
    )
for field, replacement in (
    ("size", artifacts[0]["physical_identity"]["size"] + 1),
    ("mode", artifacts[0]["physical_identity"]["mode"] ^ 0o020),
    ("path", artifacts[1]["physical_identity"]["path"]),
):
    reject(
        lambda field=field, replacement=replacement: (
            module.validate_root_install_artifact_identity(
                mutated_artifact(
                    lambda value: value["physical_identity"].__setitem__(
                        field, replacement
                    )
                ),
                authorities[0],
            )
        ),
        f"an installed-artifact physical {field} substitution",
    )

# The live-object checks remain separate from the immutable 14-record gate above.  Every mutation
# below is confined to this fresh /private/tmp root; no compiled native target is inspected.
artifact_path = root / "temporary-installed-artifact"
artifact_bytes = b"r3-rcv-02-temporary-installed-artifact\n"
artifact_path.write_bytes(artifact_bytes)
artifact_path.chmod(0o400)
artifact_observed = os.lstat(artifact_path)
artifact_authority = {
    "role": "capability_manifest",
    "intended_path": str(artifact_path),
    "uid": artifact_observed.st_uid,
    "gid": artifact_observed.st_gid,
    "mode": 0o400,
    "size": len(artifact_bytes),
    "sha256": module.sha_bytes(artifact_bytes),
}
artifact_record = {
    "role": artifact_authority["role"],
    "path": artifact_authority["intended_path"],
    "physical_identity": module.physical_identity(artifact_path, artifact_observed),
    "sha256": artifact_authority["sha256"],
    "executable_identity": None,
    "signing_posture": None,
}
substituted = copy.deepcopy(artifact_record)
substituted["physical_identity"]["inode"] += 1
reject(
    lambda: module.unlink_exact_file(artifact_path, substituted),
    "an installed-artifact live physical identity substitution",
)
if not artifact_path.exists():
    raise SystemExit("physical-identity substitution removed the temporary artifact")
module.unlink_exact_file(artifact_path, artifact_record)

manifest = {
    "reviewed_admin_block_sha256": "2" * 64,
    "artifacts": [artifact_authority],
}
manifest_bytes = module.canonical(manifest)
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
module.ARTIFACT_SPECS = [
    (
        artifact_authority["role"],
        "temporary-installed-artifact",
        artifact_authority["intended_path"],
        artifact_authority["uid"],
        artifact_authority["gid"],
        artifact_authority["mode"],
        None,
    )
]
module.INSTALL_DIRECTORIES = [(str(global_exchange), 501, 0, 0o700)]
cleanup_completion = {
    "installed_artifacts": [artifact_record],
    "installed_directories": [global_identity],
}
original_read_exact_file = module.read_exact_file
original_physical_identity = module.physical_identity


def read_installed_manifest(path, expected_uid, expected_gid, expected_mode):
    if path != installed_manifest or expected_uid != 0 or expected_gid != 0:
        raise SystemExit("temporary cleanup escaped its installed-manifest fence")
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
    raise SystemExit("permission-only installed-manifest record was not accepted")

global_exchange.rmdir()
print("r3_rcv_02_actual_cleanup_record_regression=PASS")
PY
