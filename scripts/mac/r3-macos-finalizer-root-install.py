#!/usr/bin/python3
"""Sealed root install/resume/remove membrane for the R3 disposable proof.

This source is never executed from the repository.  The candidate-freeze script embeds these
exact bytes in the one reviewed administrator block, whose SHA-256 is in turn bound by the
candidate manifest.  It accepts no arguments, targets only compiled literals, installs with
no-clobber descriptor-relative publication, and leaves the launchd/native protocol to the exact
root runner.
"""

from __future__ import annotations

import base64
import ctypes
import errno
import hashlib
import json
import os
import pathlib
import pwd
import re
import signal
import stat
import subprocess
import sys
from typing import Any, NoReturn


EXPERIMENT_ID = "019ffec6-95f6-7d30-80bc-8003ce27d5ba"
EXPERIMENT_ROOT = (
    pathlib.Path(
        "/Users/spensermcconnell/Library/Application Support/Atomize/"
        "R3MacEvidenceFinalizer/experiments"
    )
    / EXPERIMENT_ID
)
FREEZE_ROOT = EXPERIMENT_ROOT / "candidate-freeze"
ARTIFACT_ROOT = FREEZE_ROOT / "artifacts"
MANIFEST_PATH = FREEZE_ROOT / "candidate-artifact-manifest.v2.json"
ADMIN_BLOCK_PATH = FREEZE_ROOT / "reviewed-admin-command-block.v2.txt"
CLAIM_ROOT = pathlib.Path(
    "/private/tmp/com.atomize.substrate.r3-macos-finalizer-freeze.v2"
)
PRECLAIM_PATH = CLAIM_ROOT / "root-install-preclaim.v2.json"
COMPLETION_PATH = CLAIM_ROOT / "root-install-completion.v2.json"
FINALIZER_LABEL = "com.atomize.substrate.r3-macos-evidence-finalizer.v2"
ENDPOINT_PATH = pathlib.Path(
    "/private/var/run/com.atomize.substrate.r3-macos-evidence-finalizer.v2.sock"
)
PLATFORM_MANAGED_SOCKET_PARENT = pathlib.Path("/private/var/run")
JOURNAL_ROOT = pathlib.Path(
    "/private/var/db/com.atomize.substrate.r3-macos-evidence-finalizer.v2"
)
LATCH_ROOT = JOURNAL_ROOT / "latches"
PUBLISHER_ROOT = pathlib.Path(
    "/private/var/db/com.atomize.substrate.r3-macos-disposable-publisher.v2"
)
RUNNER_ROOT = pathlib.Path(
    "/private/var/db/com.atomize.substrate.r3-macos-disposable-experiment-runner.v2"
)
CREATOR_ROOT = pathlib.Path(
    "/private/var/db/com.atomize.substrate.r3-macos-signer-acl-experiment.v2"
)
GLOBAL_EXCHANGE = EXPERIMENT_ROOT / "global-publisher-exchange"
GLOBAL_PRE_EFFECT_PATH = GLOBAL_EXCHANGE / "global-pre-effect-packet.v2.json"
NATIVE_EVIDENCE_EXPORT_PATH = GLOBAL_EXCHANGE / "native-evidence-export.v2.json"
NATIVE_EVIDENCE_ACKNOWLEDGEMENT_PATH = (
    GLOBAL_EXCHANGE / "native-evidence-export-acknowledgement.v2.json"
)
NATIVE_EVIDENCE_CLEANUP_PATH = (
    GLOBAL_EXCHANGE / "native-evidence-cleanup-receipt.v2.json"
)
TERMINAL_ADMIN_CLEANUP_CLAIM_PATH = (
    GLOBAL_EXCHANGE / "terminal-admin-cleanup-claim.v2.json"
)
TERMINAL_ADMIN_RESTORATION_RECEIPT_PATH = (
    GLOBAL_EXCHANGE / "terminal-admin-restoration-receipt.v2.json"
)
TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH = (
    RUNNER_ROOT / "terminal-admin-cleanup-authorization.v2.json"
)
INSTALLED_CANDIDATE_MANIFEST_PATH = pathlib.Path(
    "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/"
    "candidate-artifact-manifest.v2.json"
)
ADMIN_RESTORATION_RECEIPT_PATH = (
    GLOBAL_EXCHANGE / "admin-install-restoration-receipt.v2.json"
)
INBOX_ROOT = pathlib.Path(
    "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/inbox"
)
REQUEST_PATH = INBOX_ROOT / "finalization-request.v2.json"
TERMINAL_PATH = INBOX_ROOT / "terminal-binding-request.v1.json"
RUNNER_PATH = pathlib.Path(
    "/Library/PrivilegedHelperTools/"
    "com.atomize.substrate.r3-macos-disposable-experiment-runner.v2"
)
STARTING_HEAD = "40015a6cfa508c112e6444086341d6df8473e875"
REPOSITORY = "/Users/spensermcconnell/.codex/worktrees/r3-macos-finalizer-proof-candidate/substrate"
BRANCH = "feat/r3-macos-finalizer-proof-candidate"
PROCESS_SNAPSHOT_IDENTITY = "candidate_preinstall_process_snapshot_v2"
FIXED_ENV = {
    "PATH": "/usr/bin:/bin:/usr/sbin:/sbin",
    "LANG": "C",
    "LC_ALL": "C",
    "TZ": "UTC",
}
ROOT_INSTALL_AUTHORITY_ENV = "R3_ROOT_INSTALL_AUTHORITY_SHA256"
EMPTY_SHA256 = hashlib.sha256(b"").hexdigest()
CODE_FLAGS = 0x12002
MAX_DOCUMENT = 4 * 1024 * 1024
MAX_PROCESS_SNAPSHOT = 256 * 1024
MAX_RUNNER_CHILD_DOCUMENT = 1024 * 1024
MAX_INSTALLED_ARTIFACT_BYTES = 16 * 1024 * 1024
MAX_TERMINAL_RUNNER_ARCHIVE_BYTES = 64 * 1024 * 1024
# A 64 MiB raw child archive expands to at most ~85.4 MiB in unpadded base64url.  The
# remaining 42+ MiB is a closed envelope budget for the two <=4 MiB root-install claims,
# the <=1 MiB terminal authorization/inventory, duplicated per-entry bindings, and schema data.
MAX_TERMINAL_CLEANUP_CLAIM = 128 * 1024 * 1024
SUCCESS_SENTINEL = "R3 sealed root install/rollback PASS"


# role, external filename, intended path, uid, gid, mode, signing identifier
ARTIFACT_SPECS = [
    (
        "finalizer_executable",
        "substrate-r3-macos-evidence-finalizer",
        "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-evidence-finalizer.v2",
        0,
        0,
        0o555,
        "com.atomize.substrate.r3-macos-evidence-finalizer.v2",
    ),
    (
        "coordinator_executable",
        "substrate-r3-macos-evidence-coordinator",
        "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-evidence-coordinator",
        0,
        0,
        0o555,
        "com.atomize.substrate.r3-macos-evidence-coordinator.v2",
    ),
    (
        "disposable_harness_executable",
        "substrate-r3-macos-disposable-harness",
        "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-disposable-harness",
        0,
        0,
        0o555,
        "com.atomize.substrate.r3-macos-disposable-harness.v2",
    ),
    (
        "peer_code_probe_executable",
        "substrate-r3-macos-peer-code-probe",
        "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-peer-code-probe",
        0,
        0,
        0o555,
        "com.atomize.substrate.r3-macos-peer-code-probe.v2",
    ),
    (
        "alternate_coordinator_executable",
        "substrate-r3-macos-evidence-coordinator-alternate-path",
        "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-evidence-coordinator-alternate-path",
        0,
        0,
        0o555,
        "com.atomize.substrate.r3-macos-evidence-coordinator.v2",
    ),
    (
        "creator_executable",
        "substrate-r3-macos-signer-acl-creator",
        "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-signer-acl-creator.v1",
        0,
        0,
        0o555,
        "com.atomize.substrate.r3-macos-signer-acl-creator.v1",
    ),
    (
        "wrong_identity_executable",
        "substrate-r3-macos-signer-acl-wrong-identity",
        "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-signer-acl-wrong-identity.v1",
        0,
        0,
        0o555,
        "com.atomize.substrate.r3-macos-signer-acl-wrong-identity.v1",
    ),
    (
        "disposable_publisher_executable",
        "substrate-r3-macos-disposable-publisher",
        "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-disposable-publisher.v2",
        0,
        0,
        0o555,
        "com.atomize.substrate.r3-macos-disposable-publisher.v2",
    ),
    (
        "disposable_experiment_runner_executable",
        "substrate-r3-macos-disposable-experiment-runner",
        str(RUNNER_PATH),
        0,
        0,
        0o555,
        "com.atomize.substrate.r3-macos-disposable-experiment-runner.v2",
    ),
    (
        "nobody_owner_probe_executable",
        "substrate-r3-macos-nobody-owner-probe",
        "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-nobody-owner-probe",
        0,
        0,
        0o555,
        "com.atomize.substrate.r3-macos-nobody-owner-probe.v2",
    ),
    (
        "security_agent_observer_executable",
        "substrate-r3-macos-securityagent-observer",
        "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-securityagent-observer",
        0,
        0,
        0o555,
        "com.atomize.substrate.r3-macos-securityagent-observer.v2",
    ),
    (
        "benign_injection_library",
        "substrate-r3-macos-benign-injection-probe.dylib",
        "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-benign-injection-probe.dylib",
        0,
        0,
        0o555,
        "com.atomize.substrate.r3-macos-benign-injection-probe.v2",
    ),
    (
        "launchd_plist",
        "com.atomize.substrate.r3-macos-evidence-finalizer.v2.plist",
        "/Library/LaunchDaemons/com.atomize.substrate.r3-macos-evidence-finalizer.v2.plist",
        0,
        0,
        0o644,
        None,
    ),
    (
        "capability_manifest",
        "capability-v2.json",
        str(FREEZE_ROOT / "capability-v2.json"),
        501,
        20,
        0o400,
        None,
    ),
]

SUPPORT_DIRECTORIES = [
    ("/Library/Application Support/Atomize", 0, 0, 0o755),
    ("/Library/Application Support/Atomize/R3MacEvidenceFinalizer", 0, 0, 0o755),
    ("/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2", 0, 0, 0o755),
]
INSTALL_DIRECTORIES = SUPPORT_DIRECTORIES + [
    (str(INBOX_ROOT), 0, 20, 0o750),
    (str(JOURNAL_ROOT), 0, 0, 0o700),
    (str(GLOBAL_EXCHANGE), 501, 0, 0o700),
]
SCOPES = [
    "019ffeb5-b252-79ae-8f41-e161419fbbcd",
    "019ffeb5-b255-75a5-870f-49323ebb2c19",
]


class Stop(RuntimeError):
    pass


def fail(message: str) -> None:
    raise Stop(message)


def canonical(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode()


def sha_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def document_sha256(value: Any) -> str:
    return sha_bytes(canonical(value))


def duplicate_rejecting_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    output: dict[str, Any] = {}
    for key, value in pairs:
        if key in output:
            fail(f"duplicate canonical JSON field {key}")
        output[key] = value
    return output


def reject_float(_: str) -> None:
    fail("floating-point JSON is not canonical for the root install membrane")


def parse_canonical_bounded(data: bytes, label: str, maximum: int) -> Any:
    if not data or len(data) > maximum:
        fail(f"{label} has an invalid byte length")
    try:
        value = json.loads(
            data,
            object_pairs_hook=duplicate_rejecting_object,
            parse_float=reject_float,
            parse_constant=reject_float,
        )
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise Stop(f"{label} is not strict JSON: {error}") from error
    if canonical(value) != data:
        fail(f"{label} is not canonical JSON")
    return value


def parse_canonical(data: bytes, label: str) -> Any:
    return parse_canonical_bounded(data, label, MAX_DOCUMENT)


def parse_terminal_cleanup_claim(data: bytes) -> Any:
    return parse_canonical_bounded(
        data, "terminal admin cleanup claim", MAX_TERMINAL_CLEANUP_CLAIM
    )


def sha_file(
    path: pathlib.Path, expected_uid: int, expected_gid: int, expected_mode: int
) -> tuple[bytes, os.stat_result]:
    data, observed = read_exact_file(path, expected_uid, expected_gid, expected_mode)
    return data, observed


def same_stat(left: os.stat_result, right: os.stat_result) -> bool:
    return (
        left.st_dev,
        left.st_ino,
        left.st_uid,
        left.st_gid,
        left.st_mode,
        left.st_nlink,
        left.st_size,
        left.st_mtime_ns,
    ) == (
        right.st_dev,
        right.st_ino,
        right.st_uid,
        right.st_gid,
        right.st_mode,
        right.st_nlink,
        right.st_size,
        right.st_mtime_ns,
    )


def open_directory_chain(path: pathlib.Path) -> int:
    if not path.is_absolute():
        fail(f"non-absolute directory path {path}")
    descriptor = os.open("/", os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    current = pathlib.Path("/")
    try:
        for component in path.parts[1:]:
            next_descriptor = os.open(
                component,
                os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW,
                dir_fd=descriptor,
            )
            observed = os.fstat(next_descriptor)
            current /= component
            if not stat.S_ISDIR(observed.st_mode):
                os.close(next_descriptor)
                fail(f"directory ancestor changed type: {current}")
            sticky_tmp = str(current) == "/private/tmp"
            if observed.st_uid not in (0, 501):
                os.close(next_descriptor)
                fail(f"directory ancestor changed owner: {current}")
            if not sticky_tmp and observed.st_mode & 0o022:
                os.close(next_descriptor)
                fail(f"directory ancestor is group/world writable: {current}")
            if sticky_tmp and (
                observed.st_uid != 0
                or observed.st_gid != 0
                or observed.st_mode & 0o7777 != 0o1777
            ):
                os.close(next_descriptor)
                fail("/private/tmp changed its root:wheel sticky identity")
            os.close(descriptor)
            descriptor = next_descriptor
        return descriptor
    except BaseException:
        os.close(descriptor)
        raise


def read_exact_file(
    path: pathlib.Path, expected_uid: int, expected_gid: int, expected_mode: int
) -> tuple[bytes, os.stat_result]:
    parent = open_directory_chain(path.parent)
    try:
        before = os.stat(path.name, dir_fd=parent, follow_symlinks=False)
        descriptor = os.open(path.name, os.O_RDONLY | os.O_NOFOLLOW, dir_fd=parent)
        try:
            held = os.fstat(descriptor)
            if not same_stat(before, held):
                fail(f"file changed between pathname and descriptor: {path}")
            if (
                not stat.S_ISREG(held.st_mode)
                or held.st_uid != expected_uid
                or held.st_gid != expected_gid
                or held.st_mode & 0o7777 != expected_mode
                or held.st_nlink != 1
                or held.st_size < 0
                or held.st_size > MAX_DOCUMENT * 128
            ):
                fail(f"file physical identity changed: {path}")
            chunks = []
            while True:
                block = os.read(descriptor, 1024 * 1024)
                if not block:
                    break
                chunks.append(block)
            data = b"".join(chunks)
            if len(data) != held.st_size:
                fail(f"file read length changed: {path}")
            after = os.stat(path.name, dir_fd=parent, follow_symlinks=False)
            if not same_stat(held, after):
                fail(f"file changed after descriptor read: {path}")
            return data, held
        finally:
            os.close(descriptor)
    finally:
        os.close(parent)


def read_exact_sized_file(
    path: pathlib.Path,
    expected_uid: int,
    expected_gid: int,
    expected_mode: int,
    expected_size: int,
    maximum_size: int,
) -> tuple[bytes, os.stat_result]:
    if (
        not isinstance(expected_size, int)
        or isinstance(expected_size, bool)
        or expected_size < 0
        or expected_size > maximum_size
    ):
        fail(f"file expected size exceeds its compiled read bound: {path}")
    parent = open_directory_chain(path.parent)
    try:
        before = os.stat(path.name, dir_fd=parent, follow_symlinks=False)
        descriptor = os.open(path.name, os.O_RDONLY | os.O_NOFOLLOW, dir_fd=parent)
        try:
            held = os.fstat(descriptor)
            if not same_stat(before, held):
                fail(f"file changed between pathname and descriptor: {path}")
            if (
                not stat.S_ISREG(held.st_mode)
                or held.st_uid != expected_uid
                or held.st_gid != expected_gid
                or held.st_mode & 0o7777 != expected_mode
                or held.st_nlink != 1
                or held.st_size != expected_size
            ):
                fail(f"file physical identity or exact size changed: {path}")
            data = bytearray()
            while len(data) <= expected_size:
                block = os.read(
                    descriptor, min(1024 * 1024, expected_size + 1 - len(data))
                )
                if not block:
                    break
                data.extend(block)
            descriptor_after = os.fstat(descriptor)
            after = os.stat(path.name, dir_fd=parent, follow_symlinks=False)
            if (
                len(data) != expected_size
                or not same_stat(held, descriptor_after)
                or not same_stat(descriptor_after, after)
            ):
                fail(f"file changed or exceeded its exact expected size: {path}")
            return bytes(data), held
        finally:
            os.close(descriptor)
    finally:
        os.close(parent)


def read_bounded_exact_file(
    path: pathlib.Path,
    expected_uid: int,
    expected_gid: int,
    expected_mode: int,
    maximum_size: int,
) -> tuple[bytes, os.stat_result]:
    observed = os.lstat(path)
    if observed.st_size < 0 or observed.st_size > maximum_size:
        fail(f"bounded file exceeds its compiled read ceiling: {path}")
    return read_exact_sized_file(
        path,
        expected_uid,
        expected_gid,
        expected_mode,
        observed.st_size,
        maximum_size,
    )


def physical_identity(path: pathlib.Path, observed: os.stat_result) -> dict[str, Any]:
    return {
        "path": str(path),
        "device": observed.st_dev,
        "inode": observed.st_ino,
        "uid": observed.st_uid,
        "gid": observed.st_gid,
        "mode": observed.st_mode,
        "link_count": observed.st_nlink,
        "size": observed.st_size,
        "modified_seconds": observed.st_mtime_ns // 1_000_000_000,
        "modified_nanoseconds": observed.st_mtime_ns % 1_000_000_000,
    }


ROOT_INSTALL_PHYSICAL_IDENTITY_FIELDS = {
    "path",
    "device",
    "inode",
    "uid",
    "gid",
    "mode",
    "link_count",
    "size",
    "modified_seconds",
    "modified_nanoseconds",
}

ROOT_INSTALL_ARTIFACT_IDENTITY_FIELDS = {
    "role",
    "path",
    "physical_identity",
    "sha256",
    "executable_identity",
    "signing_posture",
}


def validate_root_install_artifact_identity_shape(value: Any) -> dict[str, Any]:
    if not isinstance(value, dict) or set(value) != ROOT_INSTALL_ARTIFACT_IDENTITY_FIELDS:
        fail("root-install artifact identity changed its closed outer shape")
    physical = value.get("physical_identity")
    if not isinstance(physical, dict) or set(physical) != ROOT_INSTALL_PHYSICAL_IDENTITY_FIELDS:
        fail("root-install artifact identity changed its closed physical shape")
    if (
        not isinstance(value.get("role"), str)
        or not isinstance(value.get("path"), str)
        or not is_sha256(value.get("sha256"))
        or not isinstance(physical.get("path"), str)
        or any(
            type(physical.get(field)) is not int
            for field in ROOT_INSTALL_PHYSICAL_IDENTITY_FIELDS - {"path"}
        )
        or any(
            physical[field] < 0
            for field in ("device", "inode", "uid", "gid", "mode", "link_count", "size")
        )
        or not 0 <= physical["modified_nanoseconds"] < 1_000_000_000
    ):
        fail("root-install artifact identity changed its typed physical shape")
    return value


def validate_root_install_artifact_identity(
    value: Any, expected: dict[str, Any]
) -> dict[str, Any]:
    artifact = validate_root_install_artifact_identity_shape(value)
    physical = artifact["physical_identity"]
    if (
        artifact["role"] != expected["role"]
        or artifact["path"] != expected["intended_path"]
        or artifact["sha256"] != expected["sha256"]
        or physical["path"] != expected["intended_path"]
        or physical["uid"] != expected["uid"]
        or physical["gid"] != expected["gid"]
        or physical["mode"] != stat.S_IFREG | expected["mode"]
        or physical["link_count"] != 1
        or physical["size"] != expected["size"]
    ):
        fail("root-install artifact identity differs from the candidate manifest")
    return artifact


def runner_file_physical_identity(observed: os.stat_result) -> dict[str, Any]:
    return {
        "device": observed.st_dev,
        "inode": observed.st_ino,
        "owner_uid": observed.st_uid,
        "owner_gid": observed.st_gid,
        "mode": observed.st_mode,
        "link_count": observed.st_nlink,
        "size": observed.st_size,
        "modified_seconds": observed.st_mtime_ns // 1_000_000_000,
        "modified_nanoseconds": observed.st_mtime_ns % 1_000_000_000,
    }


def runner_root_stable_identity(observed: os.stat_result) -> dict[str, Any]:
    return {
        "path": str(RUNNER_ROOT),
        "device": observed.st_dev,
        "inode": observed.st_ino,
        "owner_uid": observed.st_uid,
        "owner_gid": observed.st_gid,
        "mode": observed.st_mode,
    }


def directory_identity(
    path: pathlib.Path, uid: int, gid: int, mode: int
) -> dict[str, Any]:
    descriptor = open_directory_chain(path)
    try:
        observed = os.fstat(descriptor)
        if (
            observed.st_uid != uid
            or observed.st_gid != gid
            or observed.st_mode & 0o7777 != mode
            or not stat.S_ISDIR(observed.st_mode)
            or observed.st_nlink < 2
        ):
            fail(f"directory identity changed: {path}")
        path_observed = os.lstat(path)
        if not same_stat(observed, path_observed):
            fail(f"directory pathname changed around descriptor: {path}")
        return physical_identity(path, observed)
    finally:
        os.close(descriptor)


def require_absent(path: pathlib.Path) -> None:
    try:
        os.lstat(path)
    except FileNotFoundError as error:
        if error.errno != errno.ENOENT:
            raise
        return
    fail(f"expected exact path absence: {path}")


def require_durable_absence(path: pathlib.Path) -> dict[str, Any] | None:
    """Fsync and reopen the exact existing parent before accepting one absent child.

    When a containing directory is already absent, its own ordered cleanup boundary supplies the
    durability proof.  Callers perform that outer-boundary check before publishing restoration.
    """
    require_absent(path)
    if not path_present(path.parent):
        return None
    parent = open_directory_chain(path.parent)
    try:
        before = os.fstat(parent)
        try:
            os.stat(path.name, dir_fd=parent, follow_symlinks=False)
        except FileNotFoundError as error:
            if error.errno != errno.ENOENT:
                raise
        else:
            fail(f"absent child reappeared before parent fsync: {path}")
        os.fsync(parent)
    finally:
        os.close(parent)
    reopened = open_directory_chain(path.parent)
    try:
        after = os.fstat(reopened)
        if any(
            getattr(before, field) != getattr(after, field)
            for field in ("st_dev", "st_ino", "st_uid", "st_gid", "st_mode")
        ):
            fail(f"parent changed around durable absence proof: {path.parent}")
        try:
            os.stat(path.name, dir_fd=reopened, follow_symlinks=False)
        except FileNotFoundError as error:
            if error.errno != errno.ENOENT:
                raise
        else:
            fail(f"absent child reappeared after parent fsync: {path}")
    finally:
        os.close(reopened)
    require_absent(path)
    return {
        "path": str(path),
        "parent_path": str(path.parent),
        "parent_device": before.st_dev,
        "parent_inode": before.st_ino,
        "parent_uid": before.st_uid,
        "parent_gid": before.st_gid,
        "parent_mode": before.st_mode,
        "parent_fsync_return": 0,
        "child_fstatat_return": -1,
        "child_raw_errno": errno.ENOENT,
        "parent_reopened_same_stable_identity": True,
    }


PLATFORM_MANAGED_SOCKET_PARENT_IDENTITY_PHASES = (
    "pathname_before",
    "held_descriptor",
    "reopened_descriptor",
    "pathname_after",
)
PLATFORM_MANAGED_SOCKET_CHILD_ABSENCE_PHASES = (
    "pathname_before",
    "held_parent_descriptor",
    "reopened_parent_descriptor",
    "pathname_after",
)


def platform_managed_socket_absence_binding(value: dict[str, Any]) -> str:
    body = {
        key: item
        for key, item in value.items()
        if key != "platform_managed_socket_absence_sha256"
    }
    return document_sha256(
        {
            "domain": (
                "substrate.r3-macos-finalizer-platform-managed-socket-absence.v1"
            ),
            "observation": body,
        }
    )


def platform_managed_socket_parent_identity(
    phase: str, observed: os.stat_result
) -> dict[str, Any]:
    return {
        "phase": phase,
        "path": str(PLATFORM_MANAGED_SOCKET_PARENT),
        "device": observed.st_dev,
        "inode": observed.st_ino,
        "uid": observed.st_uid,
        "gid": observed.st_gid,
        "mode": observed.st_mode,
    }


def platform_managed_socket_child_absence(phase: str) -> dict[str, Any]:
    return {
        "phase": phase,
        "path": str(ENDPOINT_PATH),
        "lstat_return": -1,
        "raw_errno": errno.ENOENT,
    }


def validate_platform_managed_socket_absence(value: Any) -> dict[str, Any]:
    if (
        not isinstance(value, dict)
        or set(value)
        != {
            "schema_owner",
            "schema_version",
            "classification",
            "socket_path",
            "platform_parent_path",
            "parent_identity_observations",
            "child_absence_observations",
            "platform_managed_socket_absence_sha256",
        }
        or value.get("schema_owner")
        != "substrate.r3-macos-finalizer-platform-managed-socket-absence"
        or value.get("schema_version") != 1
        or value.get("classification")
        != "platform_managed_live_absence_observation"
        or value.get("socket_path") != str(ENDPOINT_PATH)
        or value.get("platform_parent_path")
        != str(PLATFORM_MANAGED_SOCKET_PARENT)
        or value.get("platform_managed_socket_absence_sha256")
        != platform_managed_socket_absence_binding(value)
    ):
        fail("platform-managed socket absence changed its closed authority")
    identities = value.get("parent_identity_observations")
    if (
        not isinstance(identities, list)
        or [item.get("phase") if isinstance(item, dict) else None for item in identities]
        != list(PLATFORM_MANAGED_SOCKET_PARENT_IDENTITY_PHASES)
    ):
        fail("platform-managed socket parent observations changed order")
    stable_identity: tuple[int, int, int, int, int] | None = None
    for item in identities:
        if (
            not isinstance(item, dict)
            or set(item) != {"phase", "path", "device", "inode", "uid", "gid", "mode"}
            or item.get("path") != str(PLATFORM_MANAGED_SOCKET_PARENT)
            or any(
                type(item.get(field)) is not int
                for field in ("device", "inode", "uid", "gid", "mode")
            )
            or item["device"] < 0
            or item["inode"] <= 0
            or item["uid"] != 0
            or item["gid"] != 1
            or item["mode"] != stat.S_IFDIR | 0o775
        ):
            fail("platform-managed socket parent changed its exact directory identity")
        observed_identity = (
            item["device"],
            item["inode"],
            item["uid"],
            item["gid"],
            item["mode"],
        )
        if stable_identity is None:
            stable_identity = observed_identity
        elif stable_identity != observed_identity:
            fail("platform-managed socket parent changed during observation")
    absences = value.get("child_absence_observations")
    if (
        not isinstance(absences, list)
        or [item.get("phase") if isinstance(item, dict) else None for item in absences]
        != list(PLATFORM_MANAGED_SOCKET_CHILD_ABSENCE_PHASES)
    ):
        fail("platform-managed socket child observations changed order")
    for item in absences:
        if (
            not isinstance(item, dict)
            or set(item) != {"phase", "path", "lstat_return", "raw_errno"}
            or item.get("path") != str(ENDPOINT_PATH)
            or item.get("lstat_return") != -1
            or item.get("raw_errno") != errno.ENOENT
        ):
            fail("platform-managed socket child absence changed")
    return value


def observe_platform_managed_socket_absence(path: pathlib.Path) -> dict[str, Any]:
    if path != ENDPOINT_PATH:
        fail("platform-managed socket observer received a noncompiled path")
    require_absent(path)
    child_absences = [platform_managed_socket_child_absence("pathname_before")]
    immutable_parent = open_directory_chain(PLATFORM_MANAGED_SOCKET_PARENT.parent)
    try:
        pathname_before = os.stat(
            PLATFORM_MANAGED_SOCKET_PARENT.name,
            dir_fd=immutable_parent,
            follow_symlinks=False,
        )
        identities = [
            platform_managed_socket_parent_identity("pathname_before", pathname_before)
        ]
        held_parent = os.open(
            PLATFORM_MANAGED_SOCKET_PARENT.name,
            os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW,
            dir_fd=immutable_parent,
        )
        try:
            identities.append(
                platform_managed_socket_parent_identity(
                    "held_descriptor", os.fstat(held_parent)
                )
            )
            try:
                os.stat(path.name, dir_fd=held_parent, follow_symlinks=False)
            except FileNotFoundError as error:
                if error.errno != errno.ENOENT:
                    raise
            else:
                fail("platform-managed socket appeared during held-parent inspection")
            child_absences.append(
                platform_managed_socket_child_absence("held_parent_descriptor")
            )
        finally:
            os.close(held_parent)
        reopened_parent = os.open(
            PLATFORM_MANAGED_SOCKET_PARENT.name,
            os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW,
            dir_fd=immutable_parent,
        )
        try:
            identities.append(
                platform_managed_socket_parent_identity(
                    "reopened_descriptor", os.fstat(reopened_parent)
                )
            )
            try:
                os.stat(path.name, dir_fd=reopened_parent, follow_symlinks=False)
            except FileNotFoundError as error:
                if error.errno != errno.ENOENT:
                    raise
            else:
                fail("platform-managed socket reappeared after parent reopen")
            child_absences.append(
                platform_managed_socket_child_absence("reopened_parent_descriptor")
            )
        finally:
            os.close(reopened_parent)
        require_absent(path)
        child_absences.append(
            platform_managed_socket_child_absence("pathname_after")
        )
        pathname_after = os.stat(
            PLATFORM_MANAGED_SOCKET_PARENT.name,
            dir_fd=immutable_parent,
            follow_symlinks=False,
        )
        identities.append(
            platform_managed_socket_parent_identity("pathname_after", pathname_after)
        )
    finally:
        os.close(immutable_parent)
    value = {
        "schema_owner": "substrate.r3-macos-finalizer-platform-managed-socket-absence",
        "schema_version": 1,
        "classification": "platform_managed_live_absence_observation",
        "socket_path": str(ENDPOINT_PATH),
        "platform_parent_path": str(PLATFORM_MANAGED_SOCKET_PARENT),
        "parent_identity_observations": identities,
        "child_absence_observations": child_absences,
    }
    value["platform_managed_socket_absence_sha256"] = (
        platform_managed_socket_absence_binding(value)
    )
    return validate_platform_managed_socket_absence(value)


def path_present(path: pathlib.Path) -> bool:
    try:
        os.lstat(path)
        return True
    except FileNotFoundError as error:
        if error.errno != errno.ENOENT:
            raise
        return False


def raw_stream(data: bytes) -> dict[str, Any]:
    return {
        "base64url": base64.urlsafe_b64encode(data).rstrip(b"=").decode(),
        "sha256": sha_bytes(data),
        "byte_length": len(data),
    }


def raw_stream_bytes(value: Any, label: str) -> bytes:
    if not isinstance(value, dict) or set(value) != {
        "base64url",
        "sha256",
        "byte_length",
    }:
        fail(f"{label} changed its closed raw-stream shape")
    encoded = value.get("base64url")
    if not isinstance(encoded, str):
        fail(f"{label} is not base64url text")
    try:
        padding = "=" * ((4 - len(encoded) % 4) % 4)
        decoded = base64.urlsafe_b64decode(encoded + padding)
    except (ValueError, TypeError) as error:
        raise Stop(f"{label} is not base64url") from error
    if (
        base64.urlsafe_b64encode(decoded).rstrip(b"=").decode() != encoded
        or len(decoded) > MAX_DOCUMENT
        or value.get("sha256") != sha_bytes(decoded)
        or value.get("byte_length") != len(decoded)
    ):
        fail(f"{label} changed its raw bytes, digest, or length")
    return decoded


def validate_raw_stream(value: Any, label: str) -> None:
    raw_stream_bytes(value, label)


def bounded_control_stream_bytes(value: Any, label: str) -> bytes:
    if not isinstance(value, dict) or set(value) != {
        "raw_base64url",
        "raw_sha256",
        "raw_byte_length",
    }:
        fail(f"{label} changed its closed bounded-stream shape")
    encoded = value.get("raw_base64url")
    if not isinstance(encoded, str):
        fail(f"{label} is not base64url text")
    try:
        padding = "=" * ((4 - len(encoded) % 4) % 4)
        decoded = base64.urlsafe_b64decode(encoded + padding)
    except (ValueError, TypeError) as error:
        raise Stop(f"{label} is not base64url") from error
    if (
        base64.urlsafe_b64encode(decoded).rstrip(b"=").decode() != encoded
        or len(decoded) > 256 * 1024
        or value.get("raw_sha256") != sha_bytes(decoded)
        or value.get("raw_byte_length") != len(decoded)
    ):
        fail(f"{label} changed its raw bytes, digest, or length")
    return decoded


def validate_launchctl_not_found_streams(
    label: str, exit_status: int, stdout: Any, stderr: Any
) -> None:
    stdout_bytes = raw_stream_bytes(stdout, "launchctl not-found stdout")
    stderr_bytes = raw_stream_bytes(stderr, "launchctl not-found stderr")
    expected_stderr = (
        f'Bad request.\nCould not find service "{label}" in domain for system\n'.encode(
            "utf-8"
        )
    )
    if (
        exit_status != 113
        or stdout_bytes
        or stderr_bytes != expected_stderr
    ):
        fail("launchctl output is not the frozen service-not-found classification")


def validate_bounded_launchctl_not_found_streams(
    label: str, exit_status: int, stdout: Any, stderr: Any
) -> None:
    stdout_bytes = bounded_control_stream_bytes(
        stdout, "bounded launchctl not-found stdout"
    )
    stderr_bytes = bounded_control_stream_bytes(
        stderr, "bounded launchctl not-found stderr"
    )
    expected_stderr = (
        f'Bad request.\nCould not find service "{label}" in domain for system\n'.encode(
            "utf-8"
        )
    )
    if (
        exit_status != 113
        or stdout_bytes
        or stderr_bytes != expected_stderr
    ):
        fail("bounded launchctl output is not the frozen service-not-found classification")


def validate_bounded_launchctl_bootout_success_streams(
    exit_status: int, stdout: Any, stderr: Any
) -> None:
    stdout_bytes = bounded_control_stream_bytes(
        stdout, "bounded launchctl bootout stdout"
    )
    stderr_bytes = bounded_control_stream_bytes(
        stderr, "bounded launchctl bootout stderr"
    )
    if exit_status != 0 or stdout_bytes or stderr_bytes:
        fail("bounded launchctl bootout result is not the frozen empty-stream success tuple")


def validate_runner_private_archive(value: Any, label: str) -> tuple[list[dict[str, Any]], int]:
    if not isinstance(value, dict) or set(value) != {
        "entries",
        "entry_set_sha256",
        "total_bytes",
        "maximum_bytes",
    }:
        fail(f"{label} changed its closed archive shape")
    entries = value.get("entries")
    if (
        not isinstance(entries, list)
        or not entries
        or value.get("maximum_bytes") != MAX_TERMINAL_RUNNER_ARCHIVE_BYTES
        or value.get("entry_set_sha256") != document_sha256(entries)
    ):
        fail(f"{label} changed its exact set authority")
    total = 0
    prior: str | None = None
    for entry in entries:
        if (
            not isinstance(entry, dict)
            or set(entry)
            != {
                "name",
                "canonical_byte_length",
                "canonical_sha256",
                "physical_identity_sha256",
                "canonical_base64url",
            }
        ):
            fail(f"{label} contains an entry with an alternate shape")
        name = entry.get("name")
        length = entry.get("canonical_byte_length")
        encoded = entry.get("canonical_base64url")
        if (
            not isinstance(name, str)
            or not name
            or name.startswith(".")
            or any(character in name for character in "/\0\n\r")
            or prior is not None
            and prior >= name
            or not isinstance(length, int)
            or length < 1
            or length > MAX_RUNNER_CHILD_DOCUMENT
            or not is_sha256(entry.get("canonical_sha256"))
            or not is_sha256(entry.get("physical_identity_sha256"))
            or not isinstance(encoded, str)
        ):
            fail(f"{label} contains an invalid sorted bounded entry")
        padding = "=" * ((4 - len(encoded) % 4) % 4)
        try:
            raw = base64.urlsafe_b64decode(encoded + padding)
        except (ValueError, TypeError) as error:
            raise Stop(f"{label} entry is not base64url") from error
        if (
            base64.urlsafe_b64encode(raw).rstrip(b"=").decode() != encoded
            or len(raw) != length
            or sha_bytes(raw) != entry["canonical_sha256"]
        ):
            fail(f"{label} entry changed its exact raw bytes")
        total += len(raw)
        prior = name
    if total != value.get("total_bytes") or total > MAX_TERMINAL_RUNNER_ARCHIVE_BYTES:
        fail(f"{label} changed or exceeded its exact aggregate byte total")
    return entries, total


def run_command(arguments: list[str]) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        arguments,
        cwd="/",
        env=FIXED_ENV,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def process_snapshot() -> list[dict[str, Any]]:
    libproc = ctypes.CDLL("/usr/lib/libproc.dylib", use_errno=True)
    libproc.proc_listallpids.argtypes = [ctypes.c_void_p, ctypes.c_int]
    libproc.proc_listallpids.restype = ctypes.c_int
    count = libproc.proc_listallpids(None, 0)
    if count <= 0:
        fail("proc_listallpids could not establish the root-time process baseline")
    capacity = count + 64
    values = (ctypes.c_int * capacity)()
    actual = libproc.proc_listallpids(values, ctypes.sizeof(values))
    if actual <= 0:
        fail("proc_listallpids could not populate the root-time process baseline")
    if actual >= capacity:
        fail("proc_listallpids saturated the root-time process baseline buffer")
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
            records.append(
                {
                    "pid": pid,
                    "path": buffer.value.decode(errors="strict"),
                    "classification": "path_observed",
                    "raw_errno": 0,
                }
            )
        elif raw_errno in (errno.ESRCH, errno.ENOENT):
            records.append(
                {
                    "pid": pid,
                    "path": None,
                    "classification": "disappeared_during_snapshot",
                    "raw_errno": raw_errno,
                }
            )
        else:
            fail(
                "proc_pidpath could not classify one root-time process "
                f"(pid={pid}, errno={raw_errno})"
            )
    return records


def validate_process_snapshot_records(
    records: Any, snapshot: Any, snapshot_sha256: Any
) -> list[dict[str, Any]]:
    if not isinstance(records, list) or not records or not isinstance(snapshot, dict):
        fail("process snapshot omitted its bounded typed records")
    validate_raw_stream(snapshot, "process snapshot")
    if snapshot.get("byte_length", MAX_PROCESS_SNAPSHOT + 1) > MAX_PROCESS_SNAPSHOT:
        fail("process snapshot exceeds its fixed canonical byte bound")
    encoded = snapshot.get("base64url")
    if not isinstance(encoded, str):
        fail("process snapshot lacks canonical base64url bytes")
    padding = "=" * ((4 - len(encoded) % 4) % 4)
    try:
        raw = base64.urlsafe_b64decode(encoded + padding)
    except (ValueError, TypeError) as error:
        raise Stop("process snapshot is not base64url") from error
    parsed = parse_canonical(raw, "process snapshot")
    if (
        parsed != records
        or canonical(records) != raw
        or snapshot_sha256 != snapshot.get("sha256")
    ):
        fail("process snapshot changed its typed canonical records")
    prior_pid: int | None = None
    for record in records:
        if (
            not isinstance(record, dict)
            or set(record) != {"pid", "path", "classification", "raw_errno"}
            or not isinstance(record.get("pid"), int)
            or record["pid"] <= 0
            or prior_pid is not None
            and prior_pid >= record["pid"]
        ):
            fail("process snapshot is not a strictly PID-ordered typed record set")
        prior_pid = record["pid"]
        classification = record.get("classification")
        if classification == "path_observed":
            path = record.get("path")
            if (
                not isinstance(path, str)
                or not path.startswith("/")
                or any(character in path for character in "\0\n\r")
                or record.get("raw_errno") != 0
            ):
                fail("observed process record changed its exact path classification")
        elif classification == "disappeared_during_snapshot":
            if record.get("path") is not None or record.get("raw_errno") not in (
                errno.ESRCH,
                errno.ENOENT,
            ):
                fail("disappeared process record changed its exact errno classification")
        else:
            fail("process snapshot contains an unknown record classification")
    return records


def validate_process_snapshot_evidence(
    snapshot: Any,
) -> tuple[list[dict[str, Any]], str]:
    expected_fields = {
        "total_process_count",
        "process_records",
        "process_snapshot",
        "process_snapshot_sha256",
    }
    if not isinstance(snapshot, dict) or set(snapshot) != expected_fields:
        fail("shared process snapshot changed its closed raw-evidence shape")
    records = validate_process_snapshot_records(
        snapshot.get("process_records"),
        snapshot.get("process_snapshot"),
        snapshot.get("process_snapshot_sha256"),
    )
    if snapshot.get("total_process_count") != len(records):
        fail("shared process snapshot changed its exact record cardinality")
    return records, snapshot["process_snapshot_sha256"]


def validate_process_enumeration(
    enumeration: Any,
    expected_path: str,
    shared_records: list[dict[str, Any]],
    shared_snapshot_sha256: str,
) -> None:
    expected_fields = {"matching_pids", "process_snapshot_sha256"}
    if not isinstance(enumeration, dict) or set(enumeration) != expected_fields:
        fail("process enumeration changed its closed shared-snapshot reference shape")
    if enumeration.get("process_snapshot_sha256") != shared_snapshot_sha256:
        fail("process enumeration changed its shared snapshot binding")
    matching = [
        record["pid"]
        for record in shared_records
        if record.get("path") == expected_path
    ]
    if enumeration.get("matching_pids") != matching or matching:
        fail("process enumeration found an exact-path process")


def classification_for(kind: str) -> dict[str, Any]:
    if kind == "filesystem_path":
        return {"classification": "path_absent"}
    if kind == "unix_endpoint":
        return {"classification": "endpoint_absent"}
    if kind == "launchd_label":
        return {"classification": "launchd_label_absent", "raw_exit_status": 113}
    if kind == "process_snapshot":
        return {"classification": "process_snapshot_captured"}
    if kind == "process_executable_path":
        return {"classification": "process_absent"}
    fail(f"unknown absence kind {kind}")


def absence_plan() -> list[dict[str, Any]]:
    values = [
        {
            "kind": "filesystem_path",
            "identity": intended,
            "classification": classification_for("filesystem_path"),
        }
        for role, _, intended, _, _, _, _ in ARTIFACT_SPECS
        if role != "capability_manifest"
    ]
    paths = [
        *[entry[0] for entry in SUPPORT_DIRECTORIES],
        "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/candidate-artifact-manifest.v2.json",
        "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/disposable-publisher-prepared-input.v2.json",
        "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/candidate-identity-packet.v2.json",
        "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/peer-control-identity-packet.v2.json",
        str(INBOX_ROOT),
        str(REQUEST_PATH),
        str(TERMINAL_PATH),
        str(JOURNAL_ROOT),
        str(LATCH_ROOT),
        str(PUBLISHER_ROOT),
        str(RUNNER_ROOT),
        str(CREATOR_ROOT),
        str(CLAIM_ROOT),
        str(GLOBAL_EXCHANGE),
    ]
    values.extend(
        {
            "kind": "filesystem_path",
            "identity": path,
            "classification": classification_for("filesystem_path"),
        }
        for path in paths
    )
    for scope in SCOPES:
        for path in (
            JOURNAL_ROOT / scope,
            JOURNAL_ROOT / "capability" / scope,
            LATCH_ROOT / f"{scope}.retirement-terminal.v2.latch",
        ):
            values.append(
                {
                    "kind": "filesystem_path",
                    "identity": str(path),
                    "classification": classification_for("filesystem_path"),
                }
            )
    values.extend(
        [
            {
                "kind": "launchd_label",
                "identity": FINALIZER_LABEL,
                "classification": classification_for("launchd_label"),
            },
            {
                "kind": "unix_endpoint",
                "identity": str(ENDPOINT_PATH),
                "classification": classification_for("unix_endpoint"),
            },
        ]
    )
    values.append(
        {
            "kind": "process_snapshot",
            "identity": PROCESS_SNAPSHOT_IDENTITY,
            "classification": classification_for("process_snapshot"),
        }
    )
    values.extend(
        {
            "kind": "process_executable_path",
            "identity": intended,
            "classification": classification_for("process_executable_path"),
        }
        for _, _, intended, _, _, _, identifier in ARTIFACT_SPECS
        if identifier is not None
    )
    return values


def observe_probe(
    probe: dict[str, Any], processes: list[dict[str, Any]], process_digest: str
) -> dict[str, Any]:
    kind = probe["kind"]
    identity = probe["identity"]
    if kind in ("filesystem_path", "unix_endpoint"):
        try:
            os.lstat(identity)
        except FileNotFoundError as error:
            raw = {
                "kind": kind,
                "identity": identity,
                "lstat_return": -1,
                "raw_errno": error.errno,
                "stat_result": None,
            }
        else:
            fail(f"root-time path absence changed: {identity}")
    elif kind == "launchd_label":
        result = run_command(["/bin/launchctl", "print", f"system/{identity}"])
        if result.returncode != 113:
            fail(f"root-time launchd absence returned {result.returncode}: {identity}")
        stdout = raw_stream(result.stdout)
        stderr = raw_stream(result.stderr)
        validate_launchctl_not_found_streams(identity, result.returncode, stdout, stderr)
        raw = {
            "kind": kind,
            "identity": identity,
            "raw_exit_status": result.returncode,
            "stdout": stdout,
            "stderr": stderr,
        }
    elif kind == "process_snapshot":
        raw = {
            "kind": kind,
            "identity": identity,
            "snapshot": {
                "total_process_count": len(processes),
                "process_records": processes,
                "process_snapshot": raw_stream(canonical(processes)),
                "process_snapshot_sha256": process_digest,
            },
        }
    elif kind == "process_executable_path":
        matching = [record["pid"] for record in processes if record["path"] == identity]
        if matching:
            fail(f"root-time exact process path is already live: {identity}:{matching}")
        raw = {
            "kind": kind,
            "identity": identity,
            "enumeration": {
                "matching_pids": matching,
                "process_snapshot_sha256": process_digest,
            },
        }
    else:
        fail(f"unknown root-time absence kind {kind}")
    predicate_sha256 = document_sha256(probe)
    raw_sha256 = document_sha256(raw)
    return {
        "predicate_sha256": predicate_sha256,
        "raw_observation": raw,
        "raw_observation_sha256": raw_sha256,
        "observation_sha256": document_sha256(
            [predicate_sha256, raw_sha256, probe["classification"]]
        ),
    }


def collect_live_absence_observations() -> list[dict[str, Any]]:
    processes = process_snapshot()
    process_digest = document_sha256(processes)
    return [
        observe_probe(probe, processes, process_digest)
        for probe in absence_plan()
        if probe["identity"] != str(CLAIM_ROOT)
    ]


def validate_live_observation(
    value: dict[str, Any],
    probe: dict[str, Any],
    shared_process_snapshot: tuple[list[dict[str, Any]], str] | None = None,
) -> tuple[list[dict[str, Any]], str] | None:
    raw = value.get("raw_observation")
    if (
        value.get("predicate_sha256") != document_sha256(probe)
        or value.get("raw_observation_sha256") != document_sha256(raw)
        or value.get("observation_sha256")
        != document_sha256(
            [
                value.get("predicate_sha256"),
                value.get("raw_observation_sha256"),
                probe["classification"],
            ]
        )
        or not isinstance(raw, dict)
        or raw.get("kind") != probe["kind"]
        or raw.get("identity") != probe["identity"]
    ):
        fail("root-install live absence observation changed its canonical binding")
    kind = probe["kind"]
    if kind in ("filesystem_path", "unix_endpoint"):
        if (
            raw.get("lstat_return") != -1
            or raw.get("raw_errno") != errno.ENOENT
            or raw.get("stat_result") is not None
        ):
            fail("root-install path absence is not exact ENOENT")
    elif kind == "launchd_label":
        if raw.get("raw_exit_status") != 113:
            fail("root-install launchd absence is not exact exit 113")
        for field in ("stdout", "stderr"):
            stream = raw.get(field)
            if not isinstance(stream, dict):
                fail("root-install launchd raw stream is missing")
            try:
                encoded = stream["base64url"]
                padding = "=" * ((4 - len(encoded) % 4) % 4)
                decoded = base64.urlsafe_b64decode(encoded + padding)
            except (KeyError, ValueError) as error:
                raise Stop("root-install launchd raw stream is malformed") from error
            if stream.get("sha256") != sha_bytes(decoded) or stream.get(
                "byte_length"
            ) != len(decoded):
                fail("root-install launchd raw stream digest changed")
        validate_launchctl_not_found_streams(
            probe["identity"], raw["raw_exit_status"], raw["stdout"], raw["stderr"]
        )
    elif kind == "process_snapshot":
        if shared_process_snapshot is not None or set(raw) != {
            "kind",
            "identity",
            "snapshot",
        }:
            fail("root-install shared process snapshot changed its unique shape")
        shared_process_snapshot = validate_process_snapshot_evidence(
            raw.get("snapshot")
        )
    elif kind == "process_executable_path":
        if shared_process_snapshot is None:
            fail("root-install process absence precedes its shared snapshot")
        enumeration = raw.get("enumeration")
        if (
            set(raw) != {"kind", "identity", "enumeration"}
            or not isinstance(enumeration, dict)
            or enumeration.get("matching_pids") != []
            or not is_sha256(enumeration.get("process_snapshot_sha256"))
        ):
            fail("root-install process absence is incomplete")
        validate_process_enumeration(
            enumeration,
            probe["identity"],
            shared_process_snapshot[0],
            shared_process_snapshot[1],
        )
    return shared_process_snapshot


def validate_live_observations(
    observations: list[dict[str, Any]], probes: list[dict[str, Any]]
) -> None:
    shared_process_snapshot = None
    for observation, probe in zip(observations, probes):
        shared_process_snapshot = validate_live_observation(
            observation, probe, shared_process_snapshot
        )
    if shared_process_snapshot is None:
        fail("root-install live evidence omitted its shared process snapshot")


def is_sha256(value: Any) -> bool:
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None


def is_git_id(value: Any) -> bool:
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{40}", value) is not None


def expected_rollback_plan() -> list[dict[str, Any]]:
    targets: list[tuple[Any, str]] = [
        ("launchd_service", "bootout_then_verify_absent"),
        ("finalizer_endpoint", "observe_exact_absence_only"),
        ("finalizer_journal_root", "observe_exact_absence_only"),
        ("coordinator_inbox_root", "remove_exact_then_verify_absent"),
        ("publisher_root", "remove_exact_then_verify_absent"),
        ("runner_root", "remove_exact_then_verify_absent"),
        ("prepared_input_packet", "remove_exact_then_verify_absent"),
        ("candidate_identity_packet", "remove_exact_then_verify_absent"),
        ("peer_control_identity_packet", "remove_exact_then_verify_absent"),
        ("creator_marker_root", "remove_exact_then_verify_absent"),
        ("freeze_temporary_root", "remove_exact_then_verify_absent"),
        ("installed_candidate_manifest", "remove_exact_then_verify_absent"),
    ]
    for role, *_ in reversed(
        [item for item in ARTIFACT_SPECS if item[0] != "capability_manifest"]
    ):
        targets.append(
            ({"installed_artifact": role}, "remove_exact_then_verify_absent")
        )
    targets.extend(
        [
            (
                "installed_support_v2_directory",
                "remove_if_same_empty_then_verify_absent",
            ),
            (
                "installed_support_product_directory",
                "remove_if_same_empty_then_verify_absent",
            ),
            (
                "installed_support_atomize_directory",
                "remove_if_same_empty_then_verify_absent",
            ),
            ("external_evidence_root", "retain_durable_external_evidence"),
        ]
    )
    return [
        {"sequence_ordinal": index + 1, "target": target, "disposition": disposition}
        for index, (target, disposition) in enumerate(targets)
    ]


def decode_candidate_raw_observation(
    observation: dict[str, Any],
    probe: dict[str, Any],
    shared_process_snapshot: tuple[list[dict[str, Any]], str] | None = None,
) -> tuple[dict[str, Any], tuple[list[dict[str, Any]], str] | None]:
    encoded = observation.get("raw_observation_base64url")
    if not isinstance(encoded, str):
        fail("candidate preinstall raw observation encoding changed")
    try:
        padding = "=" * ((4 - len(encoded) % 4) % 4)
        raw_bytes = base64.urlsafe_b64decode(encoded + padding)
    except (ValueError, TypeError) as error:
        raise Stop("candidate preinstall raw observation is not base64url") from error
    if base64.urlsafe_b64encode(raw_bytes).rstrip(b"=").decode() != encoded:
        fail("candidate preinstall raw observation is not canonical base64url")
    if len(raw_bytes) != observation.get("raw_observation_byte_length") or sha_bytes(
        raw_bytes
    ) != observation.get("raw_observation_sha256"):
        fail("candidate preinstall raw observation bytes changed")
    raw = parse_canonical(raw_bytes, "candidate preinstall raw observation")
    if (
        not isinstance(raw, dict)
        or raw.get("kind") != probe["kind"]
        or raw.get("identity") != probe["identity"]
    ):
        fail("candidate preinstall raw observation changed its exact predicate")
    if probe["kind"] in ("filesystem_path", "unix_endpoint"):
        if (
            set(raw) != {"kind", "identity", "lstat_return", "raw_errno", "stat_result"}
            or raw["lstat_return"] != -1
            or raw["raw_errno"] != errno.ENOENT
            or raw["stat_result"] is not None
        ):
            fail("candidate preinstall path absence changed from exact ENOENT")
    elif probe["kind"] == "launchd_label":
        if (
            set(raw)
            != {
                "kind",
                "identity",
                "raw_exit_status",
                "stdout",
                "stderr",
            }
            or raw["raw_exit_status"] != 113
        ):
            fail("candidate preinstall launchd absence changed from exit 113")
        for field in ("stdout", "stderr"):
            stream = raw[field]
            if not isinstance(stream, dict) or set(stream) != {
                "base64url",
                "sha256",
                "byte_length",
            }:
                fail("candidate preinstall launchd stream changed shape")
            try:
                stream_padding = "=" * ((4 - len(stream["base64url"]) % 4) % 4)
                stream_bytes = base64.urlsafe_b64decode(
                    stream["base64url"] + stream_padding
                )
            except (KeyError, ValueError, TypeError) as error:
                raise Stop(
                    "candidate preinstall launchd stream is not base64url"
                ) from error
            if (
                base64.urlsafe_b64encode(stream_bytes).rstrip(b"=").decode()
                != stream["base64url"]
                or sha_bytes(stream_bytes) != stream["sha256"]
                or len(stream_bytes) != stream["byte_length"]
            ):
                fail("candidate preinstall launchd stream bytes changed")
        validate_launchctl_not_found_streams(
            probe["identity"], raw["raw_exit_status"], raw["stdout"], raw["stderr"]
        )
    elif probe["kind"] == "process_snapshot":
        if (
            set(raw) != {"kind", "identity", "snapshot"}
            or shared_process_snapshot is not None
        ):
            fail("candidate preinstall shared process snapshot changed")
        shared_process_snapshot = validate_process_snapshot_evidence(
            raw.get("snapshot")
        )
    elif probe["kind"] == "process_executable_path":
        if shared_process_snapshot is None:
            fail("candidate preinstall process absence precedes its shared snapshot")
        enumeration = raw.get("enumeration")
        if (
            set(raw) != {"kind", "identity", "enumeration"}
            or not isinstance(enumeration, dict)
            or set(enumeration) != {"matching_pids", "process_snapshot_sha256"}
            or enumeration["matching_pids"] != []
            or not is_sha256(enumeration["process_snapshot_sha256"])
        ):
            fail("candidate preinstall process absence changed")
        validate_process_enumeration(
            enumeration,
            probe["identity"],
            shared_process_snapshot[0],
            shared_process_snapshot[1],
        )
    else:
        fail("candidate preinstall raw observation has an unknown kind")
    return raw, shared_process_snapshot


def decode_candidate_raw_observations(
    observations: list[dict[str, Any]], probes: list[dict[str, Any]]
) -> list[dict[str, Any]]:
    values = []
    shared_process_snapshot = None
    for observation, probe in zip(observations, probes):
        raw, shared_process_snapshot = decode_candidate_raw_observation(
            observation, probe, shared_process_snapshot
        )
        values.append(raw)
    if shared_process_snapshot is None:
        fail("candidate preinstall evidence omitted its shared process snapshot")
    return values


def verify_manifest_shape(manifest: dict[str, Any], manifest_bytes: bytes) -> None:
    manifest_fields = {
        "schema_owner",
        "schema_version",
        "experiment_id",
        "artifact_root",
        "repository_path",
        "repository_branch",
        "source_commit",
        "source_tree",
        "source_hashes_manifest_path",
        "source_hashes_manifest_sha256",
        "coordinator_build_input_manifest_path",
        "coordinator_build_input_manifest_sha256",
        "coordinator_build_digest",
        "global_build_input_manifest_path",
        "global_build_input_manifest_sha256",
        "global_build_digest",
        "coordinator_provenance_input_path",
        "coordinator_provenance_input_sha256",
        "global_provenance_input_path",
        "global_provenance_input_sha256",
        "installed_manifest_path",
        "capability_manifest_sha256",
        "launchd_plist_sha256",
        "install_parent_directories",
        "install_parent_directory_set_sha256",
        "artifacts",
        "artifact_set_sha256",
        "preinstall_absence_observations",
        "preinstall_absence_observation_set_sha256",
        "rollback_plan",
        "rollback_plan_sha256",
        "reviewed_admin_block_path",
        "reviewed_admin_block_sha256",
        "manifest_input_sha256",
    }
    if (
        set(manifest) != manifest_fields
        or manifest.get("schema_owner")
        != "substrate.r3-macos-candidate-freeze-manifest"
        or manifest.get("schema_version") != 2
        or manifest.get("experiment_id") != EXPERIMENT_ID
        or manifest.get("artifact_root") != str(FREEZE_ROOT)
        or manifest.get("repository_path") != REPOSITORY
        or manifest.get("repository_branch") != BRANCH
        or manifest.get("source_commit") != STARTING_HEAD
        or not is_git_id(manifest.get("source_tree"))
        or manifest.get("source_hashes_manifest_path")
        != str(FREEZE_ROOT / "source-hashes.v2.json")
        or manifest.get("coordinator_build_input_manifest_path")
        != str(FREEZE_ROOT / "coordinator-build-inputs.v2.json")
        or manifest.get("global_build_input_manifest_path")
        != str(FREEZE_ROOT / "global-build-inputs.v2.json")
        or manifest.get("coordinator_provenance_input_path")
        != str(FREEZE_ROOT / "coordinator-provenance-input.v2.json")
        or manifest.get("global_provenance_input_path")
        != str(FREEZE_ROOT / "global-provenance-input.v2.json")
        or manifest.get("installed_manifest_path")
        != "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/"
        "candidate-artifact-manifest.v2.json"
        or manifest.get("reviewed_admin_block_path") != str(ADMIN_BLOCK_PATH)
        or not is_sha256(manifest.get("reviewed_admin_block_sha256"))
        or not is_sha256(manifest.get("artifact_set_sha256"))
    ):
        fail("candidate manifest changed its fixed root-install authority")
    for field in (
        "source_hashes_manifest_sha256",
        "coordinator_build_input_manifest_sha256",
        "coordinator_build_digest",
        "global_build_input_manifest_sha256",
        "global_build_digest",
        "coordinator_provenance_input_sha256",
        "global_provenance_input_sha256",
        "capability_manifest_sha256",
        "launchd_plist_sha256",
        "install_parent_directory_set_sha256",
        "preinstall_absence_observation_set_sha256",
        "rollback_plan_sha256",
        "manifest_input_sha256",
    ):
        if not is_sha256(manifest.get(field)):
            fail(f"candidate manifest lacks exact digest field {field}")
    expected_directories = [
        {"path": path, "uid": uid, "gid": gid, "mode": mode}
        for path, uid, gid, mode in SUPPORT_DIRECTORIES
    ]
    if (
        manifest.get("install_parent_directories") != expected_directories
        or manifest.get("install_parent_directory_set_sha256")
        != document_sha256(expected_directories)
        or manifest.get("rollback_plan") != expected_rollback_plan()
        or manifest.get("rollback_plan_sha256")
        != document_sha256(expected_rollback_plan())
    ):
        fail("candidate manifest changed its directory or rollback surface")
    artifacts = manifest.get("artifacts")
    if not isinstance(artifacts, list) or len(artifacts) != len(ARTIFACT_SPECS):
        fail("candidate manifest changed its artifact cardinality")
    for entry, spec in zip(artifacts, ARTIFACT_SPECS):
        role, filename, intended, uid, gid, mode, signing_identifier = spec
        if (
            not isinstance(entry, dict)
            or set(entry)
            != {
                "role",
                "external_path",
                "intended_path",
                "uid",
                "gid",
                "mode",
                "size",
                "sha256",
                "signing_identifier",
                "designated_requirement",
                "cdhash",
                "code_flags",
                "team_id",
                "entitlements_size",
                "entitlements_sha256",
            }
            or entry.get("role") != role
            or entry.get("external_path") != str(ARTIFACT_ROOT / filename)
            or entry.get("intended_path") != intended
            or entry.get("uid") != uid
            or entry.get("gid") != gid
            or entry.get("mode") != mode
            or not isinstance(entry.get("size"), int)
            or entry["size"] <= 0
            or entry["size"] > MAX_INSTALLED_ARTIFACT_BYTES
            or not is_sha256(entry.get("sha256"))
            or entry.get("signing_identifier") != signing_identifier
        ):
            fail(f"candidate artifact manifest changed fixed role {role}")
        if signing_identifier is None:
            for key in (
                "designated_requirement",
                "cdhash",
                "code_flags",
                "team_id",
                "entitlements_size",
                "entitlements_sha256",
            ):
                if entry.get(key) is not None:
                    fail(f"unsigned candidate artifact acquired code identity: {role}")
        elif (
            not entry.get("designated_requirement")
            or not is_git_id(entry.get("cdhash"))
            or entry.get("code_flags") != CODE_FLAGS
            or entry.get("team_id") is not None
            or entry.get("entitlements_size") != 0
            or entry.get("entitlements_sha256") != EMPTY_SHA256
        ):
            fail(f"signed candidate artifact changed hardened posture: {role}")
    if manifest.get("artifact_set_sha256") != document_sha256(
        {
            "domain": "substrate.r3-macos-candidate-freeze-artifact-set.v2",
            "artifacts": artifacts,
        }
    ):
        fail("candidate artifact-set digest changed")
    probes = absence_plan()
    observations = manifest.get("preinstall_absence_observations")
    if not isinstance(observations, list) or len(observations) != len(probes):
        fail("candidate preinstall absence cardinality changed")
    for observation, probe in zip(observations, probes):
        if (
            not isinstance(observation, dict)
            or set(observation)
            != {
                "kind",
                "identity",
                "exact_predicate_sha256",
                "classification",
                "observation_sha256",
                "raw_observation_base64url",
                "raw_observation_sha256",
                "raw_observation_byte_length",
            }
            or observation.get("kind") != probe["kind"]
            or observation.get("identity") != probe["identity"]
            or observation.get("classification") != probe["classification"]
            or observation.get("exact_predicate_sha256") != document_sha256(probe)
            or not is_sha256(observation.get("observation_sha256"))
            or not is_sha256(observation.get("raw_observation_sha256"))
            or not isinstance(observation.get("raw_observation_byte_length"), int)
        ):
            fail("candidate preinstall absence sequence changed")
        if observation["observation_sha256"] != document_sha256(
            [
                observation["exact_predicate_sha256"],
                observation["raw_observation_sha256"],
                probe["classification"],
            ]
        ):
            fail("candidate preinstall observation digest changed its raw binding")
    decode_candidate_raw_observations(observations, probes)
    if manifest.get("preinstall_absence_observation_set_sha256") != document_sha256(
        observations
    ):
        fail("candidate preinstall absence aggregate changed")
    if document_sha256(manifest) != sha_bytes(manifest_bytes):
        fail("candidate manifest document digest is not its exact byte digest")


def supporting_manifest_set_sha256(manifest: dict[str, Any]) -> str:
    return document_sha256(
        [
            manifest["source_hashes_manifest_sha256"],
            manifest["coordinator_build_input_manifest_sha256"],
            manifest["global_build_input_manifest_sha256"],
            manifest["coordinator_provenance_input_sha256"],
            manifest["global_provenance_input_sha256"],
            manifest["manifest_input_sha256"],
        ]
    )


def load_manifest() -> tuple[dict[str, Any], bytes]:
    manifest_bytes, _ = read_exact_file(MANIFEST_PATH, 501, 20, 0o400)
    manifest = parse_canonical(manifest_bytes, "candidate freeze manifest")
    if not isinstance(manifest, dict):
        fail("candidate freeze manifest is not an object")
    verify_manifest_shape(manifest, manifest_bytes)
    admin_bytes, _ = read_exact_file(ADMIN_BLOCK_PATH, 501, 20, 0o400)
    if sha_bytes(admin_bytes) != manifest["reviewed_admin_block_sha256"]:
        fail("reviewed administrator block differs from candidate manifest")
    supporting = [
        ("source_hashes_manifest_path", "source_hashes_manifest_sha256"),
        (
            "coordinator_build_input_manifest_path",
            "coordinator_build_input_manifest_sha256",
        ),
        ("global_build_input_manifest_path", "global_build_input_manifest_sha256"),
        ("coordinator_provenance_input_path", "coordinator_provenance_input_sha256"),
        ("global_provenance_input_path", "global_provenance_input_sha256"),
    ]
    for path_field, digest_field in supporting:
        value = manifest.get(path_field)
        if not isinstance(value, str) or not value.startswith(str(FREEZE_ROOT) + "/"):
            fail(f"supporting manifest path changed: {path_field}")
        data, _ = read_exact_file(pathlib.Path(value), 501, 20, 0o400)
        if sha_bytes(data) != manifest.get(digest_field):
            fail(f"supporting manifest bytes changed: {path_field}")
    # candidate-manifest-input is fixed even though the final manifest carries only its digest.
    input_path = FREEZE_ROOT / "candidate-manifest-input.v2.json"
    input_bytes, _ = read_exact_file(input_path, 501, 20, 0o400)
    if sha_bytes(input_bytes) != manifest.get("manifest_input_sha256"):
        fail("candidate manifest input bytes changed")
    for entry in manifest["artifacts"]:
        data, _ = read_exact_sized_file(
            pathlib.Path(entry["external_path"]),
            501,
            20,
            0o400,
            entry["size"],
            MAX_INSTALLED_ARTIFACT_BYTES,
        )
        if len(data) != entry["size"] or sha_bytes(data) != entry["sha256"]:
            fail(f"external candidate artifact changed: {entry['role']}")
        if entry["signing_identifier"] is not None:
            verify_code_identity(pathlib.Path(entry["external_path"]), entry)
    return manifest, manifest_bytes


def root_install_authority_projection(manifest: dict[str, Any]) -> dict[str, Any]:
    raw_absence = decode_candidate_raw_observations(
        manifest["preinstall_absence_observations"], absence_plan()
    )
    return {
        "schema_owner": "substrate.r3-macos-finalizer-root-install-authority",
        "schema_version": 2,
        "experiment_id": EXPERIMENT_ID,
        "repository_path": REPOSITORY,
        "repository_branch": BRANCH,
        "source_commit": manifest["source_commit"],
        "source_tree": manifest["source_tree"],
        "source_hashes_manifest_sha256": manifest["source_hashes_manifest_sha256"],
        "coordinator_build_input_manifest_sha256": manifest[
            "coordinator_build_input_manifest_sha256"
        ],
        "coordinator_build_digest": manifest["coordinator_build_digest"],
        "global_build_input_manifest_sha256": manifest[
            "global_build_input_manifest_sha256"
        ],
        "global_build_digest": manifest["global_build_digest"],
        "coordinator_provenance_input_sha256": manifest[
            "coordinator_provenance_input_sha256"
        ],
        "global_provenance_input_sha256": manifest["global_provenance_input_sha256"],
        "capability_manifest_sha256": manifest["capability_manifest_sha256"],
        "launchd_plist_sha256": manifest["launchd_plist_sha256"],
        "install_parent_directory_set_sha256": manifest[
            "install_parent_directory_set_sha256"
        ],
        "artifact_set_sha256": manifest["artifact_set_sha256"],
        "preinstall_raw_absence_sha256": document_sha256(raw_absence),
        "rollback_plan_sha256": manifest["rollback_plan_sha256"],
    }


def validate_root_install_authority(
    manifest: dict[str, Any], expected_sha256: str
) -> None:
    observed = document_sha256(root_install_authority_projection(manifest))
    if observed != expected_sha256:
        fail(
            "candidate manifest differs from the exact authority embedded in the reviewed "
            "administrator block"
        )


def decode_canonical_document_base64url(value: Any, label: str) -> tuple[Any, bytes]:
    if not isinstance(value, str):
        fail(f"{label} is not base64url text")
    try:
        padding = "=" * ((4 - len(value) % 4) % 4)
        data = base64.urlsafe_b64decode(value + padding)
    except (ValueError, TypeError) as error:
        raise Stop(f"{label} is not base64url") from error
    if base64.urlsafe_b64encode(data).rstrip(b"=").decode() != value:
        fail(f"{label} is not canonical base64url")
    return parse_canonical(data, label), data


def candidate_manifest_ancestor_identity(
    path: pathlib.Path, observed: os.stat_result
) -> dict[str, Any]:
    return {
        "path": str(path),
        "device": observed.st_dev,
        "inode": observed.st_ino,
        "owner_uid": observed.st_uid,
        "owner_gid": observed.st_gid,
        "mode": observed.st_mode,
        "link_count": observed.st_nlink,
    }


def read_installed_candidate_manifest_nofollow() -> tuple[bytes, os.stat_result, str]:
    path = INSTALLED_CANDIDATE_MANIFEST_PATH
    components = path.parts[1:]
    if not components:
        fail("installed candidate manifest path lacks a physical component")
    descriptors = [os.open("/", os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)]
    identities: list[dict[str, Any]] = []
    current = pathlib.Path("/")

    def validate_ancestor(candidate: pathlib.Path, observed: os.stat_result) -> None:
        if (
            not stat.S_ISDIR(observed.st_mode)
            or observed.st_uid != 0
            or observed.st_mode & 0o022
        ):
            fail(
                "installed candidate manifest ancestry is not root-owned immutable "
                f"directory state: {candidate}"
            )
        expected = [
            item for item in SUPPORT_DIRECTORIES if pathlib.Path(item[0]) == candidate
        ]
        if expected:
            _, uid, gid, mode = expected[0]
            if (
                observed.st_uid != uid
                or observed.st_gid != gid
                or observed.st_mode & 0o7777 != mode
            ):
                fail(
                    "installed candidate manifest support ancestry differs from the "
                    "root-install completion"
                )

    try:
        root_stat = os.fstat(descriptors[0])
        validate_ancestor(current, root_stat)
        identities.append(candidate_manifest_ancestor_identity(current, root_stat))
        for component in components[:-1]:
            next_descriptor = os.open(
                component,
                os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW,
                dir_fd=descriptors[-1],
            )
            descriptors.append(next_descriptor)
            current /= component
            observed = os.fstat(next_descriptor)
            validate_ancestor(current, observed)
            identities.append(candidate_manifest_ancestor_identity(current, observed))
        leaf = components[-1]
        before = os.stat(leaf, dir_fd=descriptors[-1], follow_symlinks=False)
        descriptor = os.open(
            leaf, os.O_RDONLY | os.O_NOFOLLOW, dir_fd=descriptors[-1]
        )
        try:
            held = os.fstat(descriptor)
            if not same_stat(before, held):
                fail("installed candidate manifest changed before descriptor read")
            if (
                not stat.S_ISREG(held.st_mode)
                or held.st_uid != 0
                or held.st_gid != 0
                or held.st_mode & 0o7777 != 0o400
                or held.st_nlink != 1
                or held.st_size < 1
                or held.st_size > MAX_DOCUMENT
            ):
                fail("installed candidate manifest changed its root:wheel 0400 identity")
            chunks = []
            while True:
                block = os.read(descriptor, 1024 * 1024)
                if not block:
                    break
                chunks.append(block)
            data = b"".join(chunks)
            after = os.stat(leaf, dir_fd=descriptors[-1], follow_symlinks=False)
            absolute_after = os.lstat(path)
            if (
                len(data) != held.st_size
                or not same_stat(held, after)
                or not same_stat(held, absolute_after)
            ):
                fail("installed candidate manifest changed during no-follow stable read")
            return data, held, document_sha256(identities)
        finally:
            os.close(descriptor)
    finally:
        for descriptor in reversed(descriptors):
            os.close(descriptor)


def validate_installed_candidate_manifest_binding(
    value: Any, manifest: dict[str, Any], manifest_bytes: bytes
) -> dict[str, Any]:
    expected_fields = {
        "external_manifest_path",
        "installed_manifest_path",
        "external_manifest_sha256",
        "installed_manifest_sha256",
        "reviewed_admin_block_sha256",
        "ancestor_chain_before_sha256",
        "ancestor_chain_after_sha256",
        "pathname_before_identity_sha256",
        "descriptor_identity_sha256",
        "pathname_after_identity_sha256",
        "device",
        "inode",
        "owner_uid",
        "owner_gid",
        "mode",
        "link_count",
        "size",
    }
    if not isinstance(value, dict) or set(value) != expected_fields:
        fail("installed candidate manifest binding changed its closed shape")
    physical = {
        "path": str(INSTALLED_CANDIDATE_MANIFEST_PATH),
        "sha256": sha_bytes(manifest_bytes),
        "device": value.get("device"),
        "inode": value.get("inode"),
        "owner_uid": value.get("owner_uid"),
        "owner_gid": value.get("owner_gid"),
        "mode": value.get("mode"),
        "link_count": value.get("link_count"),
        "size": value.get("size"),
    }
    physical_sha256 = document_sha256(physical)
    if (
        value.get("external_manifest_path") != str(MANIFEST_PATH)
        or value.get("installed_manifest_path")
        != str(INSTALLED_CANDIDATE_MANIFEST_PATH)
        or value.get("external_manifest_sha256") != sha_bytes(manifest_bytes)
        or value.get("installed_manifest_sha256") != sha_bytes(manifest_bytes)
        or value.get("reviewed_admin_block_sha256")
        != manifest["reviewed_admin_block_sha256"]
        or value.get("ancestor_chain_before_sha256")
        != value.get("ancestor_chain_after_sha256")
        or not is_sha256(value.get("ancestor_chain_before_sha256"))
        or value.get("pathname_before_identity_sha256") != physical_sha256
        or value.get("descriptor_identity_sha256") != physical_sha256
        or value.get("pathname_after_identity_sha256") != physical_sha256
        or value.get("owner_uid") != 0
        or value.get("owner_gid") != 0
        or value.get("mode") != 0o400
        or value.get("link_count") != 1
        or value.get("size") != len(manifest_bytes)
        or not all(
            isinstance(value.get(field), int) and value[field] >= 0
            for field in ("device", "inode")
        )
    ):
        fail("installed candidate manifest binding changed its exact physical identity")
    return value


def load_installed_candidate_manifest_binding(
    manifest: dict[str, Any], manifest_bytes: bytes
) -> dict[str, Any]:
    before_bytes, before_stat, before_ancestors = (
        read_installed_candidate_manifest_nofollow()
    )
    after_bytes, after_stat, after_ancestors = (
        read_installed_candidate_manifest_nofollow()
    )
    if (
        before_bytes != manifest_bytes
        or after_bytes != manifest_bytes
        or before_ancestors != after_ancestors
        or not same_stat(before_stat, after_stat)
    ):
        fail("installed and external candidate manifests changed across stable reopen")
    mode = before_stat.st_mode & 0o7777
    physical = {
        "path": str(INSTALLED_CANDIDATE_MANIFEST_PATH),
        "sha256": sha_bytes(manifest_bytes),
        "device": before_stat.st_dev,
        "inode": before_stat.st_ino,
        "owner_uid": before_stat.st_uid,
        "owner_gid": before_stat.st_gid,
        "mode": mode,
        "link_count": before_stat.st_nlink,
        "size": before_stat.st_size,
    }
    physical_sha256 = document_sha256(physical)
    value = {
        "external_manifest_path": str(MANIFEST_PATH),
        "installed_manifest_path": str(INSTALLED_CANDIDATE_MANIFEST_PATH),
        "external_manifest_sha256": sha_bytes(manifest_bytes),
        "installed_manifest_sha256": sha_bytes(manifest_bytes),
        "reviewed_admin_block_sha256": manifest["reviewed_admin_block_sha256"],
        "ancestor_chain_before_sha256": before_ancestors,
        "ancestor_chain_after_sha256": after_ancestors,
        "pathname_before_identity_sha256": physical_sha256,
        "descriptor_identity_sha256": physical_sha256,
        "pathname_after_identity_sha256": physical_sha256,
        "device": before_stat.st_dev,
        "inode": before_stat.st_ino,
        "owner_uid": before_stat.st_uid,
        "owner_gid": before_stat.st_gid,
        "mode": mode,
        "link_count": before_stat.st_nlink,
        "size": before_stat.st_size,
    }
    return validate_installed_candidate_manifest_binding(value, manifest, manifest_bytes)


def validate_root_install_claims_binding_shape(
    value: Any, manifest: dict[str, Any], manifest_bytes: bytes
) -> dict[str, Any]:
    expected_fields = {
        "preclaim",
        "completion",
        "preclaim_sha256",
        "completion_sha256",
        "claim_root_current_identity",
        "claim_root_pathname_before_sha256",
        "claim_root_descriptor_sha256",
        "claim_root_pathname_after_sha256",
        "completion_leaf_identity",
    }
    if not isinstance(value, dict) or set(value) != expected_fields:
        fail("root-install claims binding changed its closed shape")
    preclaim = value.get("preclaim")
    completion = value.get("completion")
    current_root = value.get("claim_root_current_identity")
    completion_leaf = value.get("completion_leaf_identity")
    if not all(
        isinstance(item, dict)
        for item in (preclaim, completion, current_root, completion_leaf)
    ):
        fail("root-install claims binding lacks its canonical documents or identities")
    current_sha256 = document_sha256(current_root)
    if (
        value.get("preclaim_sha256") != document_sha256(preclaim)
        or value.get("completion_sha256") != document_sha256(completion)
        or value.get("claim_root_pathname_before_sha256") != current_sha256
        or value.get("claim_root_descriptor_sha256") != current_sha256
        or value.get("claim_root_pathname_after_sha256") != current_sha256
        or preclaim.get("candidate_freeze_manifest_sha256")
        != sha_bytes(manifest_bytes)
        or completion.get("candidate_freeze_manifest_sha256")
        != sha_bytes(manifest_bytes)
        or completion.get("preclaim_sha256") != document_sha256(preclaim)
    ):
        fail("root-install claims binding changed its exact document digests")
    expected_root = preclaim.get("claim_root_identity")
    if (
        not isinstance(expected_root, dict)
        or any(
            current_root.get(key) != expected_root.get(key)
            for key in ("path", "device", "inode", "uid", "gid", "mode")
        )
        or current_root.get("path") != str(CLAIM_ROOT)
        or current_root.get("uid") != 0
        or current_root.get("gid") != 0
        or current_root.get("mode") != stat.S_IFDIR | 0o700
    ):
        fail("root-install claim directory changed its stable physical identity")
    expected_leaf_fields = {
        "path",
        "device",
        "inode",
        "uid",
        "gid",
        "mode",
        "link_count",
        "size",
        "modified_seconds",
        "modified_nanoseconds",
    }
    if (
        set(completion_leaf) != expected_leaf_fields
        or completion_leaf.get("path") != str(COMPLETION_PATH)
        or completion_leaf.get("uid") != 0
        or completion_leaf.get("gid") != 0
        or completion_leaf.get("mode") != stat.S_IFREG | 0o400
        or completion_leaf.get("link_count") != 1
        or completion_leaf.get("size") != len(canonical(completion))
    ):
        fail("root-install completion leaf changed its exact physical identity")
    return value


def load_live_root_install_claims_binding(
    manifest: dict[str, Any], manifest_bytes: bytes
) -> dict[str, Any]:
    descriptor = open_directory_chain(CLAIM_ROOT)
    try:
        before = os.lstat(CLAIM_ROOT)
        held = os.fstat(descriptor)
        if not same_stat(before, held):
            fail("root-install claim directory changed before claim reconstruction")
        preclaim_bytes, preclaim_stat = read_exact_file(PRECLAIM_PATH, 0, 0, 0o400)
        preclaim = parse_canonical(preclaim_bytes, "root-install preclaim")
        if not isinstance(preclaim, dict):
            fail("root-install preclaim is not an object")
        validate_preclaim(preclaim, manifest, manifest_bytes)
        completion_bytes, completion_stat = read_exact_file(
            COMPLETION_PATH, 0, 0, 0o400
        )
        completion = parse_canonical(completion_bytes, "root-install completion")
        if not isinstance(completion, dict):
            fail("root-install completion is not an object")
        validate_completion(completion, manifest, manifest_bytes, preclaim)
        if physical_identity(PRECLAIM_PATH, preclaim_stat) != completion.get(
            "preclaim_leaf_identity"
        ):
            fail("root-install preclaim identity differs from its completion")
        final_root = os.lstat(CLAIM_ROOT)
        if not same_stat(held, final_root):
            fail("root-install claim directory changed during claim reconstruction")
        current_root = physical_identity(CLAIM_ROOT, held)
        current_sha256 = document_sha256(current_root)
        value = {
            "preclaim": preclaim,
            "completion": completion,
            "preclaim_sha256": sha_bytes(preclaim_bytes),
            "completion_sha256": sha_bytes(completion_bytes),
            "claim_root_current_identity": current_root,
            "claim_root_pathname_before_sha256": current_sha256,
            "claim_root_descriptor_sha256": current_sha256,
            "claim_root_pathname_after_sha256": current_sha256,
            "completion_leaf_identity": physical_identity(
                COMPLETION_PATH, completion_stat
            ),
        }
        return validate_root_install_claims_binding_shape(
            value, manifest, manifest_bytes
        )
    finally:
        os.close(descriptor)


def load_global_cleanup_authority(
    manifest: dict[str, Any], manifest_bytes: bytes
) -> tuple[
    bytes,
    dict[str, Any],
    dict[str, Any],
    dict[str, Any],
    dict[str, Any],
    dict[str, Any],
]:
    global_bytes, _ = read_exact_file(GLOBAL_PRE_EFFECT_PATH, 0, 0, 0o444)
    global_packet = parse_canonical(global_bytes, "global pre-effect packet")
    if not isinstance(global_packet, dict):
        fail("global pre-effect packet is not a canonical object")
    claims = global_packet.get("root_install_claims")
    installed_manifest = global_packet.get("installed_candidate_manifest")
    if not isinstance(claims, dict):
        fail("global pre-effect packet lacks root-install claims")
    if not isinstance(installed_manifest, dict):
        fail("global pre-effect packet lacks installed-manifest identity")
    preclaim = claims.get("preclaim")
    completion = claims.get("completion")
    if not isinstance(preclaim, dict) or not isinstance(completion, dict):
        fail("global pre-effect packet changed its canonical root-install claims")
    claims_sha256 = document_sha256(claims)
    preclaim_sha256 = document_sha256(preclaim)
    completion_sha256 = document_sha256(completion)
    if (
        global_packet.get("schema_owner")
        != "substrate.r3-macos-disposable-global-pre-effect-packet"
        or global_packet.get("schema_version") != 2
        or global_packet.get("experiment_id") != EXPERIMENT_ID
        or global_packet.get("candidate_freeze_manifest") != manifest
        or global_packet.get("candidate_freeze_manifest_sha256")
        != sha_bytes(manifest_bytes)
        or global_packet.get("root_install_claims_binding_sha256") != claims_sha256
        or global_packet.get("root_install_preclaim_sha256") != preclaim_sha256
        or global_packet.get("root_install_completion_sha256") != completion_sha256
        or installed_manifest.get("external_manifest_path") != str(MANIFEST_PATH)
        or installed_manifest.get("installed_manifest_path")
        != "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/"
        "candidate-artifact-manifest.v2.json"
        or installed_manifest.get("external_manifest_sha256")
        != sha_bytes(manifest_bytes)
        or installed_manifest.get("installed_manifest_sha256")
        != sha_bytes(manifest_bytes)
        or installed_manifest.get("reviewed_admin_block_sha256")
        != manifest["reviewed_admin_block_sha256"]
        or installed_manifest.get("owner_uid") != 0
        or installed_manifest.get("owner_gid") != 0
        or installed_manifest.get("mode") != 0o400
        or installed_manifest.get("link_count") != 1
        or installed_manifest.get("size") != len(manifest_bytes)
        or not all(
            isinstance(installed_manifest.get(field), int)
            and installed_manifest[field] >= 0
            for field in ("device", "inode")
        )
    ):
        fail("global pre-effect packet did not bind this exact root installation")
    validate_root_install_claims_binding_shape(claims, manifest, manifest_bytes)
    validate_installed_candidate_manifest_binding(
        installed_manifest, manifest, manifest_bytes
    )
    return (
        global_bytes,
        global_packet,
        claims,
        preclaim,
        completion,
        installed_manifest,
    )


def terminal_cleanup_global_absence_paths() -> list[pathlib.Path]:
    return [
        GLOBAL_PRE_EFFECT_PATH,
        pending_path(GLOBAL_PRE_EFFECT_PATH),
        NATIVE_EVIDENCE_EXPORT_PATH,
        pending_path(NATIVE_EVIDENCE_EXPORT_PATH),
        NATIVE_EVIDENCE_ACKNOWLEDGEMENT_PATH,
        pending_path(NATIVE_EVIDENCE_ACKNOWLEDGEMENT_PATH),
        NATIVE_EVIDENCE_CLEANUP_PATH,
        pending_path(NATIVE_EVIDENCE_CLEANUP_PATH),
    ]


def observe_terminal_cleanup_global_absence() -> list[dict[str, Any]]:
    observations = []
    for path in terminal_cleanup_global_absence_paths():
        observation = require_durable_absence(path)
        if observation is None:
            fail("global evidence exchange disappeared during pre-effect absence proof")
        observations.append(observation)
    return observations


def terminal_cleanup_authority_binding(value: dict[str, Any]) -> str:
    body = {key: item for key, item in value.items() if key != "authority_sha256"}
    return document_sha256(
        {
            "domain": "substrate.r3-macos-finalizer-terminal-cleanup-authority.v2",
            "authority": body,
        }
    )


def validate_terminal_cleanup_authority(
    value: Any, manifest: dict[str, Any], manifest_bytes: bytes
) -> dict[str, Any]:
    expected_fields = {
        "schema_owner",
        "schema_version",
        "experiment_id",
        "authority_kind",
        "global_pre_effect_path",
        "global_pre_effect_sha256",
        "global_pre_effect_absence_observations",
        "global_pre_effect_absence_observation_set_sha256",
        "root_install_claims",
        "root_install_claims_binding_sha256",
        "installed_candidate_manifest",
        "installed_candidate_manifest_identity_sha256",
        "authority_sha256",
    }
    if not isinstance(value, dict) or set(value) != expected_fields:
        fail("terminal cleanup authority changed its closed shape")
    claims = validate_root_install_claims_binding_shape(
        value.get("root_install_claims"), manifest, manifest_bytes
    )
    installed_manifest = validate_installed_candidate_manifest_binding(
        value.get("installed_candidate_manifest"), manifest, manifest_bytes
    )
    absences = value.get("global_pre_effect_absence_observations")
    if (
        value.get("schema_owner")
        != "substrate.r3-macos-finalizer-terminal-cleanup-authority"
        or value.get("schema_version") != 2
        or value.get("experiment_id") != EXPERIMENT_ID
        or value.get("global_pre_effect_path") != str(GLOBAL_PRE_EFFECT_PATH)
        or value.get("root_install_claims_binding_sha256")
        != document_sha256(claims)
        or value.get("installed_candidate_manifest_identity_sha256")
        != document_sha256(installed_manifest)
        or not isinstance(absences, list)
        or value.get("global_pre_effect_absence_observation_set_sha256")
        != document_sha256(absences)
        or value.get("authority_sha256")
        != terminal_cleanup_authority_binding(value)
    ):
        fail("terminal cleanup authority changed its exact root-install binding")
    kind = value.get("authority_kind")
    if kind == "global_pre_effect_packet":
        if not is_sha256(value.get("global_pre_effect_sha256")) or absences:
            fail("present global pre-effect authority acquired absence evidence")
        (
            global_bytes,
            _,
            global_claims,
            _,
            _,
            global_installed_manifest,
        ) = load_global_cleanup_authority(manifest, manifest_bytes)
        if (
            sha_bytes(global_bytes) != value["global_pre_effect_sha256"]
            or global_claims != claims
            or global_installed_manifest != installed_manifest
        ):
            fail("terminal cleanup authority differs from the global pre-effect packet")
    elif kind == "root_install_claims_before_global_pre_effect":
        if value.get("global_pre_effect_sha256") is not None:
            fail("pre-global terminal authority acquired a packet digest")
        observed_absences = observe_terminal_cleanup_global_absence()
        if absences != observed_absences:
            fail("pre-global terminal authority changed its durable absence proof")
    else:
        fail("terminal cleanup authority has an unknown authority kind")
    return value


def load_terminal_cleanup_authority(
    manifest: dict[str, Any], manifest_bytes: bytes
) -> dict[str, Any]:
    if path_present(GLOBAL_PRE_EFFECT_PATH):
        (
            global_bytes,
            _,
            claims,
            _,
            _,
            installed_manifest,
        ) = load_global_cleanup_authority(manifest, manifest_bytes)
        kind = "global_pre_effect_packet"
        global_sha256: str | None = sha_bytes(global_bytes)
        absences: list[dict[str, Any]] = []
    else:
        claims = load_live_root_install_claims_binding(manifest, manifest_bytes)
        installed_manifest = load_installed_candidate_manifest_binding(
            manifest, manifest_bytes
        )
        kind = "root_install_claims_before_global_pre_effect"
        global_sha256 = None
        absences = observe_terminal_cleanup_global_absence()
    value = {
        "schema_owner": "substrate.r3-macos-finalizer-terminal-cleanup-authority",
        "schema_version": 2,
        "experiment_id": EXPERIMENT_ID,
        "authority_kind": kind,
        "global_pre_effect_path": str(GLOBAL_PRE_EFFECT_PATH),
        "global_pre_effect_sha256": global_sha256,
        "global_pre_effect_absence_observations": absences,
        "global_pre_effect_absence_observation_set_sha256": document_sha256(absences),
        "root_install_claims": claims,
        "root_install_claims_binding_sha256": document_sha256(claims),
        "installed_candidate_manifest": installed_manifest,
        "installed_candidate_manifest_identity_sha256": document_sha256(
            installed_manifest
        ),
        "authority_sha256": "",
    }
    value["authority_sha256"] = terminal_cleanup_authority_binding(value)
    return validate_terminal_cleanup_authority(value, manifest, manifest_bytes)


def load_success_cleanup_authority(
    manifest: dict[str, Any], manifest_bytes: bytes
) -> tuple[
    dict[str, Any],
    dict[str, Any],
    dict[str, Any],
    dict[str, Any],
    dict[str, Any],
    bytes,
]:
    (
        global_bytes,
        _,
        claims,
        preclaim,
        completion,
        installed_manifest,
    ) = load_global_cleanup_authority(manifest, manifest_bytes)
    export_bytes, _ = read_exact_file(NATIVE_EVIDENCE_EXPORT_PATH, 0, 0, 0o444)
    # The acknowledgement is created by the exact UID501:staff harness.  The
    # retained exchange directory is UID501:wheel, but it is not setgid and the
    # immutable publisher-input primitive deliberately does not chown the leaf.
    acknowledgement_bytes, _ = read_exact_file(
        NATIVE_EVIDENCE_ACKNOWLEDGEMENT_PATH, 501, 20, 0o400
    )
    cleanup_bytes, _ = read_exact_file(NATIVE_EVIDENCE_CLEANUP_PATH, 0, 0, 0o444)
    export = parse_canonical(export_bytes, "native evidence export")
    acknowledgement = parse_canonical(
        acknowledgement_bytes, "native evidence export acknowledgement"
    )
    cleanup = parse_canonical(cleanup_bytes, "native evidence cleanup receipt")
    if not all(isinstance(value, dict) for value in (export, acknowledgement, cleanup)):
        fail("success cleanup authority is not three canonical terminal objects")
    validate_bounded_launchctl_bootout_success_streams(
        cleanup.get("launchctl_bootout_exit_status"),
        cleanup.get("launchctl_bootout_stdout"),
        cleanup.get("launchctl_bootout_stderr"),
    )
    validate_bounded_launchctl_not_found_streams(
        FINALIZER_LABEL,
        cleanup.get("launchctl_print_not_found_exit_status"),
        cleanup.get("launchctl_print_stdout"),
        cleanup.get("launchctl_print_stderr"),
    )
    if set(acknowledgement) != {
        "schema_owner",
        "schema_version",
        "experiment_id",
        "native_evidence_export_sha256",
        "artifact_set_sha256",
        "raw_securityagent_archive_sha256",
        "harness_process_attestation_sha256",
        "canonical_validation_complete",
    }:
        fail("native evidence acknowledgement changed its closed shape")
    export_archive_entries, export_archive_total = validate_runner_private_archive(
        export.get("runner_private_archive"), "native evidence export runner archive"
    )
    cleanup_archive_entries, cleanup_archive_total = validate_runner_private_archive(
        cleanup.get("cleanup_private_archive"), "native cleanup runner archive"
    )
    union = sorted(
        [*export_archive_entries, *cleanup_archive_entries], key=lambda entry: entry["name"]
    )
    if any(left["name"] >= right["name"] for left, right in zip(union, union[1:])):
        fail("native evidence runner archives overlap or are not a disjoint union")
    exported_preclaim, exported_preclaim_bytes = decode_canonical_document_base64url(
        export.get("root_install_preclaim_canonical_base64url"),
        "exported root-install preclaim",
    )
    (
        exported_completion,
        exported_completion_bytes,
    ) = decode_canonical_document_base64url(
        export.get("root_install_completion_canonical_base64url"),
        "exported root-install completion",
    )
    claims_sha256 = document_sha256(claims)
    preclaim_sha256 = document_sha256(preclaim)
    completion_sha256 = document_sha256(completion)
    if (
        export.get("schema_owner")
        != "substrate.r3-macos-disposable-native-evidence-export"
        or export.get("schema_version") != 2
        or export.get("experiment_id") != EXPERIMENT_ID
        or export.get("global_pre_effect_sha256") != sha_bytes(global_bytes)
        or export.get("root_install_claims_binding_sha256") != claims_sha256
        or export.get("root_install_preclaim_sha256") != preclaim_sha256
        or export.get("root_install_completion_sha256") != completion_sha256
        or export.get("runner_private_archive_sha256")
        != document_sha256(export.get("runner_private_archive"))
        or exported_preclaim != preclaim
        or exported_completion != completion
        or sha_bytes(exported_preclaim_bytes) != preclaim_sha256
        or sha_bytes(exported_completion_bytes) != completion_sha256
        or acknowledgement.get("schema_owner")
        != "substrate.r3-macos-disposable-native-evidence-export"
        or acknowledgement.get("schema_version") != 2
        or acknowledgement.get("experiment_id") != EXPERIMENT_ID
        or acknowledgement.get("native_evidence_export_sha256")
        != sha_bytes(export_bytes)
        or acknowledgement.get("artifact_set_sha256")
        != export.get("artifact_set_sha256")
        or acknowledgement.get("raw_securityagent_archive_sha256")
        != export.get("raw_securityagent_archive_sha256")
        or not is_sha256(acknowledgement.get("harness_process_attestation_sha256"))
        or acknowledgement.get("canonical_validation_complete") is not True
        or cleanup.get("schema_owner")
        != "substrate.r3-macos-disposable-native-evidence-export"
        or cleanup.get("schema_version") != 2
        or cleanup.get("experiment_id") != EXPERIMENT_ID
        or cleanup.get("native_evidence_export_sha256") != sha_bytes(export_bytes)
        or cleanup.get("acknowledgement_sha256") != sha_bytes(acknowledgement_bytes)
        or cleanup.get("launchd_label") != FINALIZER_LABEL
        or cleanup.get("launchctl_bootout_exit_status") != 0
        or cleanup.get("launchctl_print_not_found_exit_status") != 113
        or cleanup.get("endpoint_path") != str(ENDPOINT_PATH)
        or cleanup.get("endpoint_absent_without_cleanup_unlink") is not True
        or cleanup.get("journal_root_path") != str(JOURNAL_ROOT)
        or cleanup.get("journal_root_absent") is not True
        or cleanup.get("root_install_claim_root_path") != str(CLAIM_ROOT)
        or cleanup.get("root_install_preclaim_path") != str(PRECLAIM_PATH)
        or cleanup.get("root_install_completion_path") != str(COMPLETION_PATH)
        or cleanup.get("root_install_claims_binding_sha256") != claims_sha256
        or cleanup.get("root_install_preclaim_sha256") != preclaim_sha256
        or cleanup.get("root_install_completion_sha256") != completion_sha256
        or cleanup.get("root_install_claims_retained") is not True
        or cleanup.get("admin_cleanup_authorized") is not True
        or cleanup.get("cleanup_private_archive_sha256")
        != document_sha256(cleanup.get("cleanup_private_archive"))
        or cleanup.get("runner_private_archive_union_sha256") != document_sha256(union)
        or cleanup.get("runner_private_archive_union_cardinality") != len(union)
        or cleanup.get("runner_private_archive_total_bytes")
        != export_archive_total + cleanup_archive_total
        or cleanup.get("runner_private_archive_max_bytes")
        != MAX_TERMINAL_RUNNER_ARCHIVE_BYTES
        or not isinstance(cleanup.get("native_cleanup_step_sha256"), list)
        or not cleanup.get("native_cleanup_step_sha256")
        or not all(is_sha256(value) for value in cleanup["native_cleanup_step_sha256"])
        or not is_sha256(cleanup.get("native_cleanup_plan_sha256"))
        or cleanup.get("runner_root_path") != str(RUNNER_ROOT)
        or not is_sha256(cleanup.get("runner_root_stable_identity_sha256"))
        or cleanup.get("runner_root_empty_before_removal") is not True
        or cleanup.get("runner_root_removed_via_held_parent_dirfd") is not True
        or cleanup.get("runner_root_parent_fsynced") is not True
        or cleanup.get("runner_root_absent") is not True
        or not is_sha256(cleanup.get("runner_root_absence_observation_sha256"))
        or not is_sha256(cleanup.get("cleanup_artifact_set_sha256"))
        or not is_sha256(
            cleanup.get("root_install_claims_retention_observation_sha256")
        )
        or cleanup.get("root_install_claim_root_identity_sha256")
        != document_sha256(claims.get("claim_root_current_identity"))
    ):
        fail("native evidence cleanup did not authorize this exact root installation")
    return (
        claims,
        preclaim,
        completion,
        cleanup,
        installed_manifest,
        acknowledgement_bytes,
    )


def validate_live_root_install_claims(
    claims: dict[str, Any], preclaim: dict[str, Any], completion: dict[str, Any]
) -> None:
    expected_root = claims.get("claim_root_current_identity")
    expected_completion = claims.get("completion_leaf_identity")
    expected_preclaim = completion.get("preclaim_leaf_identity")
    if not all(
        isinstance(value, dict)
        for value in (expected_root, expected_completion, expected_preclaim)
    ):
        fail("root-install claim binding lacks original physical identities")
    live_root = directory_identity(CLAIM_ROOT, 0, 0, 0o700)
    if any(
        live_root[key] != expected_root[key]
        for key in ("path", "device", "inode", "uid", "gid", "mode")
    ):
        fail("retained root-install claim root changed before admin cleanup")
    preclaim_present = path_present(PRECLAIM_PATH)
    completion_present = path_present(COMPLETION_PATH)
    # Claims are removed only after installed-state cleanup, in the fixed completion -> preclaim
    # -> root order.  These are the only exact crash prefixes; a surviving completion after the
    # preclaim disappeared cannot have been produced by this membrane.
    if completion_present and not preclaim_present:
        fail("root-install claim removal is not an exact cleanup prefix")
    for path, value, expected in (
        (PRECLAIM_PATH, preclaim, expected_preclaim),
        (COMPLETION_PATH, completion, expected_completion),
    ):
        if not path_present(path):
            if require_durable_absence(path) is None:
                fail(
                    "root-install claim root disappeared before its leaf durability proof"
                )
            continue
        data, observed = read_exact_file(path, 0, 0, 0o400)
        if data != canonical(value) or physical_identity(path, observed) != expected:
            fail(f"retained root-install claim leaf changed before cleanup: {path}")


def terminal_receipt_name(terminal_outcome_kind: str) -> str:
    names = {
        "root_operation_securityagent_alert": (
            "root-operation-ui-terminal-stop.receipt.v2.json"
        ),
        "publisher_securityagent_alert": (
            "publisher-ui-terminal-stop.receipt.v2.json"
        ),
        "general_closed_failure": "general-failure-restoration.receipt.v2.json",
        "creator_closed_failure": "creator-terminal-cleanup.receipt.v2.json",
    }
    try:
        return names[terminal_outcome_kind]
    except KeyError as error:
        raise Stop("terminal admin cleanup authorization has an unknown outcome") from error


def validate_terminal_admin_cleanup_authorization(
    value: Any, claims: dict[str, Any]
) -> dict[str, Any]:
    expected_fields = {
        "schema_owner",
        "schema_version",
        "experiment_id",
        "terminal_outcome_kind",
        "authorization_path",
        "runner_root_path",
        "runner_root_stable_identity_sha256",
        "terminal_receipt_path",
        "terminal_receipt_sha256",
        "terminal_receipt_physical_identity_sha256",
        "runner_child_inventory",
        "runner_child_inventory_sha256",
        "runner_child_inventory_cardinality",
        "runner_child_archive_total_bytes",
        "runner_child_archive_max_bytes",
        "surrogate_restoration_sha256",
        "finalizer_cleanup_sha256",
        "creator_marker_restoration_sha256",
        "root_install_claims_binding_sha256",
        "root_install_claims_retention_observation_sha256",
        "claims_retained",
        "admin_cleanup_authorized",
        "terminal_no_normal_resume",
    }
    if not isinstance(value, dict) or set(value) != expected_fields:
        fail("terminal admin cleanup authorization changed its closed shape")
    outcome = value.get("terminal_outcome_kind")
    receipt_name = terminal_receipt_name(outcome)
    inventory = value.get("runner_child_inventory")
    if (
        value.get("schema_owner")
        != "substrate.r3-macos-terminal-admin-cleanup-authorization"
        or value.get("schema_version") != 2
        or value.get("experiment_id") != EXPERIMENT_ID
        or value.get("authorization_path")
        != str(TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH)
        or value.get("runner_root_path") != str(RUNNER_ROOT)
        or value.get("terminal_receipt_path") != str(RUNNER_ROOT / receipt_name)
        or value.get("root_install_claims_binding_sha256")
        != document_sha256(claims)
        or not isinstance(inventory, list)
        or not inventory
        or value.get("runner_child_inventory_cardinality") != len(inventory)
        or value.get("runner_child_inventory_sha256")
        != document_sha256(inventory)
        or value.get("runner_child_archive_max_bytes")
        != MAX_TERMINAL_RUNNER_ARCHIVE_BYTES
        or value.get("runner_child_archive_total_bytes")
        != sum(
            entry.get("canonical_byte_length", 0)
            for entry in inventory
            if isinstance(entry, dict)
        )
        or value.get("runner_child_archive_total_bytes", -1)
        > MAX_TERMINAL_RUNNER_ARCHIVE_BYTES
        or value.get("claims_retained") is not True
        or value.get("admin_cleanup_authorized") is not True
        or value.get("terminal_no_normal_resume") is not True
    ):
        fail("terminal admin cleanup authorization changed its closed authority")
    for field in (
        "runner_root_stable_identity_sha256",
        "terminal_receipt_sha256",
        "terminal_receipt_physical_identity_sha256",
        "runner_child_inventory_sha256",
        "surrogate_restoration_sha256",
        "finalizer_cleanup_sha256",
        "creator_marker_restoration_sha256",
        "root_install_claims_binding_sha256",
        "root_install_claims_retention_observation_sha256",
    ):
        if not is_sha256(value.get(field)):
            fail("terminal admin cleanup authorization contains a non-digest binding")
    prior: str | None = None
    for entry in inventory:
        if (
            not isinstance(entry, dict)
            or set(entry)
            != {
                "name",
                "canonical_byte_length",
                "canonical_sha256",
                "physical_identity_sha256",
            }
            or not isinstance(entry.get("name"), str)
            or not entry["name"]
            or entry["name"] == TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH.name
            or entry["name"].startswith(".")
            or any(character in entry["name"] for character in "/\0\n\r")
            or not isinstance(entry.get("canonical_byte_length"), int)
            or entry["canonical_byte_length"] < 1
            or entry["canonical_byte_length"] > MAX_RUNNER_CHILD_DOCUMENT
            or prior is not None
            and prior >= entry["name"]
            or not is_sha256(entry.get("canonical_sha256"))
            or not is_sha256(entry.get("physical_identity_sha256"))
        ):
            fail("terminal runner child inventory is not exact sorted immutable state")
        prior = entry["name"]
    matches = [entry for entry in inventory if entry["name"] == receipt_name]
    if (
        len(matches) != 1
        or matches[0]["canonical_sha256"] != value["terminal_receipt_sha256"]
        or matches[0]["physical_identity_sha256"]
        != value["terminal_receipt_physical_identity_sha256"]
    ):
        fail("terminal runner inventory differs from its terminal receipt anchor")
    return value


def read_terminal_runner_child(
    entry: dict[str, Any]
) -> tuple[bytes, os.stat_result]:
    path = RUNNER_ROOT / entry["name"]
    data, observed = read_exact_file(path, 0, 0, 0o600)
    parse_canonical(data, f"terminal runner child {entry['name']}")
    if (
        len(data) != entry["canonical_byte_length"]
        or len(data) > MAX_RUNNER_CHILD_DOCUMENT
        or sha_bytes(data) != entry["canonical_sha256"]
        or document_sha256(runner_file_physical_identity(observed))
        != entry["physical_identity_sha256"]
    ):
        fail(f"terminal runner child changed before admin cleanup: {path}")
    return data, observed


def validate_full_terminal_runner_authority(
    authorization: dict[str, Any], authorization_bytes: bytes, observed: os.stat_result
) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    root = directory_identity(RUNNER_ROOT, 0, 0, 0o700)
    stable_root = {
        "path": str(RUNNER_ROOT),
        "device": root["device"],
        "inode": root["inode"],
        "owner_uid": root["uid"],
        "owner_gid": root["gid"],
        "mode": root["mode"],
    }
    if document_sha256(stable_root) != authorization["runner_root_stable_identity_sha256"]:
        fail("terminal runner root changed before external cleanup claim")
    if canonical(authorization) != authorization_bytes:
        fail("terminal admin cleanup authorization is not its canonical bytes")
    expected_names = [
        entry["name"] for entry in authorization["runner_child_inventory"]
    ] + [TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH.name]
    descriptor = open_directory_chain(RUNNER_ROOT)
    try:
        live_names = sorted(os.listdir(descriptor))
    finally:
        os.close(descriptor)
    if live_names != sorted(expected_names):
        fail("terminal runner root differs from its authorized complete inventory")
    documents = []
    archive_total_bytes = 0
    for entry in authorization["runner_child_inventory"]:
        data, _ = read_terminal_runner_child(entry)
        archive_total_bytes += len(data)
        if archive_total_bytes > MAX_TERMINAL_RUNNER_ARCHIVE_BYTES:
            fail("terminal runner archive exceeds its fixed aggregate raw-byte bound")
        documents.append(
            {
                "name": entry["name"],
                "canonical_base64url": base64.urlsafe_b64encode(data)
                .rstrip(b"=")
                .decode(),
                "canonical_sha256": entry["canonical_sha256"],
                "physical_identity_sha256": entry["physical_identity_sha256"],
            }
        )
    return stable_root, documents


def terminal_cleanup_claim_binding(value: dict[str, Any]) -> str:
    body = {key: item for key, item in value.items() if key != "claim_sha256"}
    return document_sha256(
        {
            "domain": "substrate.r3-macos-finalizer-terminal-admin-cleanup-claim.v2",
            "claim": body,
        }
    )


def validate_terminal_cleanup_claim(
    value: Any,
    manifest: dict[str, Any],
    manifest_bytes: bytes,
    root_authority_sha256: str,
) -> dict[str, Any]:
    expected_fields = {
        "schema_owner",
        "schema_version",
        "experiment_id",
        "claim_path",
        "root_install_authority_sha256",
        "candidate_freeze_manifest_sha256",
        "terminal_cleanup_authority",
        "terminal_cleanup_authority_sha256",
        "root_install_claims_binding_sha256",
        "root_install_preclaim_sha256",
        "root_install_completion_sha256",
        "installed_candidate_manifest_identity_sha256",
        "terminal_admin_cleanup_authorization",
        "terminal_admin_cleanup_authorization_sha256",
        "terminal_admin_cleanup_authorization_physical_identity",
        "terminal_admin_cleanup_authorization_physical_identity_sha256",
        "runner_root_stable_identity",
        "runner_root_stable_identity_sha256",
        "runner_child_canonical_documents",
        "runner_child_canonical_documents_sha256",
        "runner_child_removal_order",
        "runner_child_removal_order_sha256",
        "rollback_plan_sha256",
        "claim_sha256",
    }
    if not isinstance(value, dict) or set(value) != expected_fields:
        fail("terminal admin cleanup claim changed its closed shape")
    cleanup_authority = validate_terminal_cleanup_authority(
        value.get("terminal_cleanup_authority"), manifest, manifest_bytes
    )
    claims = cleanup_authority["root_install_claims"]
    installed_manifest_identity = cleanup_authority["installed_candidate_manifest"]
    authorization = validate_terminal_admin_cleanup_authorization(
        value.get("terminal_admin_cleanup_authorization"), claims
    )
    authorization_identity = value.get(
        "terminal_admin_cleanup_authorization_physical_identity"
    )
    root_identity = value.get("runner_root_stable_identity")
    documents = value.get("runner_child_canonical_documents")
    removal_order = value.get("runner_child_removal_order")
    expected_removal_order = [
        entry["name"] for entry in reversed(authorization["runner_child_inventory"])
    ]
    if (
        value.get("schema_owner")
        != "substrate.r3-macos-finalizer-terminal-admin-cleanup-claim"
        or value.get("schema_version") != 2
        or value.get("experiment_id") != EXPERIMENT_ID
        or value.get("claim_path") != str(TERMINAL_ADMIN_CLEANUP_CLAIM_PATH)
        or value.get("root_install_authority_sha256") != root_authority_sha256
        or value.get("candidate_freeze_manifest_sha256")
        != sha_bytes(manifest_bytes)
        or value.get("terminal_cleanup_authority_sha256")
        != document_sha256(cleanup_authority)
        or value.get("root_install_claims_binding_sha256")
        != document_sha256(claims)
        or value.get("root_install_preclaim_sha256")
        != document_sha256(claims["preclaim"])
        or value.get("root_install_completion_sha256")
        != document_sha256(claims["completion"])
        or value.get("installed_candidate_manifest_identity_sha256")
        != document_sha256(installed_manifest_identity)
        or value.get("terminal_admin_cleanup_authorization_sha256")
        != document_sha256(authorization)
        or not isinstance(authorization_identity, dict)
        or value.get("terminal_admin_cleanup_authorization_physical_identity_sha256")
        != document_sha256(authorization_identity)
        or not isinstance(root_identity, dict)
        or value.get("runner_root_stable_identity_sha256")
        != document_sha256(root_identity)
        or value.get("runner_root_stable_identity_sha256")
        != authorization["runner_root_stable_identity_sha256"]
        or not isinstance(documents, list)
        or value.get("runner_child_canonical_documents_sha256")
        != document_sha256(documents)
        or removal_order != expected_removal_order
        or value.get("runner_child_removal_order_sha256")
        != document_sha256(expected_removal_order)
        or value.get("rollback_plan_sha256")
        != document_sha256(expected_rollback_plan())
        or value.get("claim_sha256") != terminal_cleanup_claim_binding(value)
    ):
        fail("terminal admin cleanup claim changed its exact authority")
    if (
        set(root_identity)
        != {"path", "device", "inode", "owner_uid", "owner_gid", "mode"}
        or root_identity.get("path") != str(RUNNER_ROOT)
        or root_identity.get("owner_uid") != 0
        or root_identity.get("owner_gid") != 0
        or root_identity.get("mode") != stat.S_IFDIR | 0o700
        or not all(
            isinstance(root_identity.get(field), int)
            for field in ("device", "inode")
        )
    ):
        fail("terminal cleanup claim changed the stable runner-root identity")
    expected_authorization_identity_fields = {
        "path",
        "device",
        "inode",
        "uid",
        "gid",
        "mode",
        "link_count",
        "size",
        "modified_seconds",
        "modified_nanoseconds",
    }
    if (
        set(authorization_identity) != expected_authorization_identity_fields
        or authorization_identity.get("path")
        != str(TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH)
        or authorization_identity.get("uid") != 0
        or authorization_identity.get("gid") != 0
        or authorization_identity.get("mode") != stat.S_IFREG | 0o600
        or authorization_identity.get("link_count") != 1
        or authorization_identity.get("size") != len(canonical(authorization))
    ):
        fail("terminal cleanup claim changed the authorization-file identity")
    inventory = authorization["runner_child_inventory"]
    if len(documents) != len(inventory):
        fail("terminal cleanup claim omitted one canonical runner child")
    archive_total_bytes = 0
    for document, entry in zip(documents, inventory):
        if (
            not isinstance(document, dict)
            or set(document)
            != {
                "name",
                "canonical_base64url",
                "canonical_sha256",
                "physical_identity_sha256",
            }
            or document.get("name") != entry["name"]
            or document.get("canonical_sha256") != entry["canonical_sha256"]
            or document.get("physical_identity_sha256")
            != entry["physical_identity_sha256"]
        ):
            fail("terminal cleanup claim changed one runner child binding")
        parsed, raw = decode_canonical_document_base64url(
            document.get("canonical_base64url"),
            f"terminal runner child archive {entry['name']}",
        )
        archive_total_bytes += len(raw)
        if (
            len(raw) != entry["canonical_byte_length"]
            or sha_bytes(raw) != entry["canonical_sha256"]
            or not isinstance(parsed, dict)
        ):
            fail("terminal cleanup claim changed one canonical runner child")
    if archive_total_bytes > MAX_TERMINAL_RUNNER_ARCHIVE_BYTES:
        fail("terminal cleanup claim exceeded its fixed aggregate raw-byte bound")
    return value


def load_or_create_terminal_cleanup_claim(
    manifest: dict[str, Any],
    manifest_bytes: bytes,
    root_authority_sha256: str,
) -> dict[str, Any]:
    if path_present(TERMINAL_ADMIN_CLEANUP_CLAIM_PATH):
        finish_no_clobber_link_residue(
            TERMINAL_ADMIN_CLEANUP_CLAIM_PATH, 0, 0, 0o444
        )
        data, _ = read_exact_file(TERMINAL_ADMIN_CLEANUP_CLAIM_PATH, 0, 0, 0o444)
        return validate_terminal_cleanup_claim(
            parse_terminal_cleanup_claim(data),
            manifest,
            manifest_bytes,
            root_authority_sha256,
        )
    cleanup_authority = load_terminal_cleanup_authority(manifest, manifest_bytes)
    claims = cleanup_authority["root_install_claims"]
    installed_manifest_identity = cleanup_authority["installed_candidate_manifest"]
    authorization_bytes, authorization_stat = read_exact_file(
        TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH, 0, 0, 0o600
    )
    authorization = validate_terminal_admin_cleanup_authorization(
        parse_canonical(
            authorization_bytes, "terminal admin cleanup authorization"
        ),
        claims,
    )
    stable_root, child_documents = validate_full_terminal_runner_authority(
        authorization, authorization_bytes, authorization_stat
    )
    removal_order = [
        entry["name"] for entry in reversed(authorization["runner_child_inventory"])
    ]
    value = {
        "schema_owner": "substrate.r3-macos-finalizer-terminal-admin-cleanup-claim",
        "schema_version": 2,
        "experiment_id": EXPERIMENT_ID,
        "claim_path": str(TERMINAL_ADMIN_CLEANUP_CLAIM_PATH),
        "root_install_authority_sha256": root_authority_sha256,
        "candidate_freeze_manifest_sha256": sha_bytes(manifest_bytes),
        "terminal_cleanup_authority": cleanup_authority,
        "terminal_cleanup_authority_sha256": document_sha256(cleanup_authority),
        "root_install_claims_binding_sha256": document_sha256(claims),
        "root_install_preclaim_sha256": document_sha256(claims["preclaim"]),
        "root_install_completion_sha256": document_sha256(claims["completion"]),
        "installed_candidate_manifest_identity_sha256": document_sha256(
            installed_manifest_identity
        ),
        "terminal_admin_cleanup_authorization": authorization,
        "terminal_admin_cleanup_authorization_sha256": document_sha256(
            authorization
        ),
        "terminal_admin_cleanup_authorization_physical_identity": physical_identity(
            TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH, authorization_stat
        ),
        "terminal_admin_cleanup_authorization_physical_identity_sha256": document_sha256(
            physical_identity(
                TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH, authorization_stat
            )
        ),
        "runner_root_stable_identity": stable_root,
        "runner_root_stable_identity_sha256": document_sha256(stable_root),
        "runner_child_canonical_documents": child_documents,
        "runner_child_canonical_documents_sha256": document_sha256(child_documents),
        "runner_child_removal_order": removal_order,
        "runner_child_removal_order_sha256": document_sha256(removal_order),
        "rollback_plan_sha256": document_sha256(expected_rollback_plan()),
        "claim_sha256": "",
    }
    value["claim_sha256"] = terminal_cleanup_claim_binding(value)
    validate_terminal_cleanup_claim(
        value,
        manifest,
        manifest_bytes,
        root_authority_sha256,
    )
    encoded = canonical(value)
    if len(encoded) > MAX_TERMINAL_CLEANUP_CLAIM:
        fail("terminal admin cleanup claim exceeds the fixed canonical document bound")
    publish_bytes_no_clobber(
        encoded, TERMINAL_ADMIN_CLEANUP_CLAIM_PATH, 0, 0, 0o444
    )
    data, _ = read_exact_file(TERMINAL_ADMIN_CLEANUP_CLAIM_PATH, 0, 0, 0o444)
    if data != encoded:
        fail("terminal admin cleanup claim changed after durable publication")
    return value


def verify_code_identity(path: pathlib.Path, entry: dict[str, Any]) -> None:
    verified = run_command(
        ["/usr/bin/codesign", "--verify", "--strict", "--verbose=4", str(path)]
    )
    if verified.returncode != 0:
        fail(f"codesign verification failed for {entry['role']}: {verified.stderr!r}")
    detail = run_command(["/usr/bin/codesign", "-d", "--verbose=4", str(path)])
    if detail.returncode != 0:
        fail(f"codesign detail failed for {entry['role']}")
    text = (detail.stdout + detail.stderr).decode(errors="strict")
    identifier = re.search(r"^Identifier=(.+)$", text, re.M)
    cdhash = re.search(r"^CDHash=([0-9a-f]{40})$", text, re.M)
    flags = re.search(r"flags=0x([0-9a-fA-F]+)", text)
    team = re.search(r"^TeamIdentifier=(.+)$", text, re.M)
    if (
        not identifier
        or identifier.group(1) != entry["signing_identifier"]
        or not cdhash
        or cdhash.group(1) != entry["cdhash"]
        or not flags
        or int(flags.group(1), 16) != CODE_FLAGS
        or (team and team.group(1) != "not set")
    ):
        fail(f"installed code detail changed for {entry['role']}")
    requirement = run_command(["/usr/bin/codesign", "-d", "-r-", str(path)])
    if requirement.returncode != 0:
        fail(f"codesign requirement failed for {entry['role']}")
    requirement_text = (requirement.stdout + requirement.stderr).decode(errors="strict")
    observed_requirement = next(
        (
            line.split("designated =>", 1)[1].strip()
            for line in requirement_text.splitlines()
            if "designated =>" in line
        ),
        None,
    )
    if observed_requirement != entry["designated_requirement"]:
        fail(f"installed designated requirement changed for {entry['role']}")
    entitlements = run_command(
        ["/usr/bin/codesign", "-d", "--entitlements", "-", str(path)]
    )
    if entitlements.returncode != 0 or entitlements.stdout:
        fail(f"installed code acquired entitlements for {entry['role']}")


def ensure_directory(
    path: pathlib.Path, uid: int, gid: int, mode: int
) -> dict[str, Any]:
    try:
        os.lstat(path)
    except FileNotFoundError:
        parent = open_directory_chain(path.parent)
        try:
            try:
                os.mkdir(path.name, mode, dir_fd=parent)
            except FileExistsError as error:
                raise Stop(
                    f"directory appeared during no-clobber create: {path}"
                ) from error
            descriptor = os.open(
                path.name,
                os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW,
                dir_fd=parent,
            )
            try:
                os.fchown(descriptor, uid, gid)
                os.fchmod(descriptor, mode)
                os.fsync(descriptor)
            finally:
                os.close(descriptor)
            os.fsync(parent)
        finally:
            os.close(parent)
    return directory_identity(path, uid, gid, mode)


def pending_path(path: pathlib.Path) -> pathlib.Path:
    return path.with_name(f".{path.name}.r3-install.pending")


def finish_no_clobber_link_residue(
    path: pathlib.Path, uid: int, gid: int, mode: int
) -> None:
    temporary = pending_path(path)
    if not path_present(temporary):
        return
    if not path_present(path):
        return
    parent = open_directory_chain(path.parent)
    try:
        final = os.stat(path.name, dir_fd=parent, follow_symlinks=False)
        residue = os.stat(temporary.name, dir_fd=parent, follow_symlinks=False)
        if (
            not stat.S_ISREG(final.st_mode)
            or final.st_uid != uid
            or final.st_gid != gid
            or final.st_mode & 0o7777 != mode
            or final.st_nlink != 2
            or final.st_dev != residue.st_dev
            or final.st_ino != residue.st_ino
        ):
            fail(f"no-clobber link residue changed same-inode state: {path}")
        os.unlink(temporary.name, dir_fd=parent)
        os.fsync(parent)
    finally:
        os.close(parent)


def publish_bytes_no_clobber(
    source_bytes: bytes, path: pathlib.Path, uid: int, gid: int, mode: int
) -> os.stat_result:
    parent = open_directory_chain(path.parent)
    temporary = pending_path(path)
    try:
        try:
            existing = os.stat(path.name, dir_fd=parent, follow_symlinks=False)
        except FileNotFoundError:
            existing = None
        try:
            temp_stat = os.stat(temporary.name, dir_fd=parent, follow_symlinks=False)
        except FileNotFoundError:
            temp_stat = None
        if existing is not None:
            if temp_stat is not None:
                if (
                    temp_stat.st_dev != existing.st_dev
                    or temp_stat.st_ino != existing.st_ino
                ):
                    fail(f"no-clobber residue is not the final file inode: {temporary}")
                descriptor = os.open(
                    path.name, os.O_RDONLY | os.O_NOFOLLOW, dir_fd=parent
                )
                try:
                    held = os.fstat(descriptor)
                    data = b""
                    while len(data) < held.st_size:
                        block = os.read(descriptor, held.st_size - len(data))
                        if not block:
                            break
                        data += block
                    if data != source_bytes or held.st_nlink != 2:
                        fail(
                            f"linked no-clobber residue changed bytes or links: {path}"
                        )
                finally:
                    os.close(descriptor)
                os.unlink(temporary.name, dir_fd=parent)
                os.fsync(parent)
            data, observed = read_exact_file(path, uid, gid, mode)
            if data != source_bytes:
                fail(f"existing no-clobber file has alternate bytes: {path}")
            return observed
        if temp_stat is None:
            descriptor = os.open(
                temporary.name,
                os.O_RDWR | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                mode,
                dir_fd=parent,
            )
        else:
            if (
                not stat.S_ISREG(temp_stat.st_mode)
                or temp_stat.st_uid != 0
                or temp_stat.st_nlink != 1
            ):
                fail(f"no-clobber temporary changed identity: {temporary}")
            descriptor = os.open(
                temporary.name, os.O_RDWR | os.O_NOFOLLOW, dir_fd=parent
            )
        try:
            held = os.fstat(descriptor)
            prefix = os.pread(descriptor, held.st_size, 0)
            if (
                held.st_size > len(source_bytes)
                or source_bytes[: held.st_size] != prefix
            ):
                fail(f"no-clobber temporary is not an exact byte prefix: {temporary}")
            offset = held.st_size
            while offset < len(source_bytes):
                written = os.pwrite(descriptor, source_bytes[offset:], offset)
                if written <= 0:
                    fail(f"short no-clobber write: {temporary}")
                offset += written
            os.fchown(descriptor, uid, gid)
            os.fchmod(descriptor, mode)
            os.fsync(descriptor)
        finally:
            os.close(descriptor)
        os.link(
            temporary.name,
            path.name,
            src_dir_fd=parent,
            dst_dir_fd=parent,
            follow_symlinks=False,
        )
        os.fsync(parent)
        observed = os.stat(path.name, dir_fd=parent, follow_symlinks=False)
        linked = os.stat(temporary.name, dir_fd=parent, follow_symlinks=False)
        if (
            linked.st_dev != observed.st_dev
            or linked.st_ino != observed.st_ino
            or observed.st_nlink != 2
        ):
            fail(f"published no-clobber temporary is not the final inode: {temporary}")
        descriptor = os.open(path.name, os.O_RDONLY | os.O_NOFOLLOW, dir_fd=parent)
        try:
            held = os.fstat(descriptor)
            data = b""
            while len(data) < held.st_size:
                block = os.read(descriptor, held.st_size - len(data))
                if not block:
                    break
                data += block
            if data != source_bytes:
                fail(
                    f"published no-clobber file differs before residue cleanup: {path}"
                )
        finally:
            os.close(descriptor)
        os.unlink(temporary.name, dir_fd=parent)
        os.fsync(parent)
        _, final = read_exact_file(path, uid, gid, mode)
        return final
    finally:
        os.close(parent)


def install_bytes_prefix_recoverable(
    source_bytes: bytes,
    path: pathlib.Path,
    uid: int,
    gid: int,
    mode: int,
    maximum_size: int,
) -> os.stat_result:
    """O_EXCL final-path install whose only recoverable incomplete state is an exact prefix.

    The preclaim proves the final path absent before installation. A crash may expose a partial
    file, but launchd remains absent and no child is invoked before completion. Resume accepts only
    the same root-owned inode, final owner/mode, and a byte-for-byte prefix of the frozen artifact;
    alternate state is preserved and rejected.
    """
    parent = open_directory_chain(path.parent)
    try:
        try:
            descriptor = os.open(
                path.name,
                os.O_RDWR | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                mode,
                dir_fd=parent,
            )
            created = True
        except FileExistsError:
            descriptor = os.open(path.name, os.O_RDWR | os.O_NOFOLLOW, dir_fd=parent)
            created = False
        try:
            if created:
                os.fchown(descriptor, uid, gid)
                os.fchmod(descriptor, mode)
                os.fsync(descriptor)
                os.fsync(parent)
            held = os.fstat(descriptor)
            if (
                not stat.S_ISREG(held.st_mode)
                or held.st_uid != uid
                or held.st_gid != gid
                or held.st_mode & 0o7777 != mode
                or held.st_nlink != 1
                or held.st_size < 0
                or held.st_size > len(source_bytes)
            ):
                fail(f"prefix-recoverable installed file changed identity: {path}")
            prefix = os.pread(descriptor, held.st_size, 0)
            if source_bytes[: held.st_size] != prefix:
                fail(f"prefix-recoverable installed file has alternate bytes: {path}")
            offset = held.st_size
            while offset < len(source_bytes):
                written = os.pwrite(descriptor, source_bytes[offset:], offset)
                if written <= 0:
                    fail(f"short prefix-recoverable install write: {path}")
                offset += written
            os.fsync(descriptor)
        finally:
            os.close(descriptor)
        os.fsync(parent)
    finally:
        os.close(parent)
    data, observed = read_exact_sized_file(
        path, uid, gid, mode, len(source_bytes), maximum_size
    )
    if data != source_bytes:
        fail(f"installed file changed after prefix completion: {path}")
    return observed


def reset_unclaimed_root() -> None:
    try:
        observed = os.lstat(CLAIM_ROOT)
    except FileNotFoundError:
        return
    if (
        not stat.S_ISDIR(observed.st_mode)
        or observed.st_uid != 0
        or observed.st_gid != 0
        or observed.st_mode & 0o7777 != 0o700
    ):
        fail("unclaimed root-install directory has alternate identity")
    entries = list(os.scandir(CLAIM_ROOT))
    allowed = {pending_path(PRECLAIM_PATH).name}
    if any(
        entry.name not in allowed
        or entry.is_symlink()
        or not entry.is_file(follow_symlinks=False)
        for entry in entries
    ):
        fail("unclaimed root-install directory contains alternate state")
    for entry in entries:
        os.unlink(entry.path)
    root_fd = open_directory_chain(CLAIM_ROOT)
    try:
        os.fsync(root_fd)
    finally:
        os.close(root_fd)
    os.rmdir(CLAIM_ROOT)
    private_tmp = open_directory_chain(pathlib.Path("/private/tmp"))
    try:
        os.fsync(private_tmp)
    finally:
        os.close(private_tmp)


def recover_claim_link_residue(path: pathlib.Path) -> None:
    temporary = pending_path(path)
    if not path_present(path) or not path_present(temporary):
        return
    root = open_directory_chain(CLAIM_ROOT)
    try:
        final = os.stat(path.name, dir_fd=root, follow_symlinks=False)
        residue = os.stat(temporary.name, dir_fd=root, follow_symlinks=False)
        if (
            not stat.S_ISREG(final.st_mode)
            or final.st_uid != 0
            or final.st_gid != 0
            or final.st_mode & 0o7777 != 0o400
            or final.st_nlink != 2
            or final.st_dev != residue.st_dev
            or final.st_ino != residue.st_ino
        ):
            fail("root-install claim link residue changed its exact same-inode state")
        os.unlink(temporary.name, dir_fd=root)
        os.fsync(root)
    finally:
        os.close(root)


def build_preclaim(manifest: dict[str, Any], manifest_bytes: bytes) -> dict[str, Any]:
    require_absent(PRECLAIM_PATH)
    require_absent(COMPLETION_PATH)
    observations = collect_live_absence_observations()
    value = {
        "schema_owner": "substrate.r3-macos-finalizer-root-install-preclaim",
        "schema_version": 2,
        "experiment_id": EXPERIMENT_ID,
        "sequence": 1,
        "claim_root_path": str(CLAIM_ROOT),
        "preclaim_path": str(PRECLAIM_PATH),
        "completion_path": str(COMPLETION_PATH),
        "private_ancestor_identity": directory_identity(
            pathlib.Path("/private"), 0, 0, 0o755
        ),
        "private_tmp_ancestor_identity": directory_identity(
            pathlib.Path("/private/tmp"), 0, 0, 0o1777
        ),
        "claim_root_identity": directory_identity(CLAIM_ROOT, 0, 0, 0o700),
        "candidate_freeze_manifest_sha256": sha_bytes(manifest_bytes),
        "reviewed_admin_block_sha256": manifest["reviewed_admin_block_sha256"],
        "candidate_artifact_set_sha256": manifest["artifact_set_sha256"],
        "install_parent_directory_set_sha256": manifest[
            "install_parent_directory_set_sha256"
        ],
        "preinstall_absence_observation_set_sha256": manifest[
            "preinstall_absence_observation_set_sha256"
        ],
        "rollback_plan_sha256": manifest["rollback_plan_sha256"],
        "supporting_manifest_set_sha256": supporting_manifest_set_sha256(manifest),
        "live_absence_observations": observations,
        "live_absence_observation_set_sha256": document_sha256(observations),
        "preclaim_leaf_lstat_return": -1,
        "preclaim_leaf_raw_errno": errno.ENOENT,
        "completion_leaf_lstat_return": -1,
        "completion_leaf_raw_errno": errno.ENOENT,
        "pre_effects_authorized": False,
    }
    validate_preclaim(value, manifest, manifest_bytes)
    return value


def validate_preclaim(
    value: dict[str, Any], manifest: dict[str, Any], manifest_bytes: bytes
) -> None:
    expected = [
        probe for probe in absence_plan() if probe["identity"] != str(CLAIM_ROOT)
    ]
    if (
        value.get("schema_owner")
        != "substrate.r3-macos-finalizer-root-install-preclaim"
        or value.get("schema_version") != 2
        or value.get("experiment_id") != EXPERIMENT_ID
        or value.get("sequence") != 1
        or value.get("claim_root_path") != str(CLAIM_ROOT)
        or value.get("preclaim_path") != str(PRECLAIM_PATH)
        or value.get("completion_path") != str(COMPLETION_PATH)
        or value.get("candidate_freeze_manifest_sha256") != sha_bytes(manifest_bytes)
        or value.get("reviewed_admin_block_sha256")
        != manifest["reviewed_admin_block_sha256"]
        or value.get("candidate_artifact_set_sha256") != manifest["artifact_set_sha256"]
        or value.get("install_parent_directory_set_sha256")
        != manifest["install_parent_directory_set_sha256"]
        or value.get("preinstall_absence_observation_set_sha256")
        != manifest["preinstall_absence_observation_set_sha256"]
        or value.get("rollback_plan_sha256") != manifest["rollback_plan_sha256"]
        or value.get("supporting_manifest_set_sha256")
        != supporting_manifest_set_sha256(manifest)
        or value.get("live_absence_observation_set_sha256")
        != document_sha256(value.get("live_absence_observations"))
        or value.get("preclaim_leaf_lstat_return") != -1
        or value.get("preclaim_leaf_raw_errno") != errno.ENOENT
        or value.get("completion_leaf_lstat_return") != -1
        or value.get("completion_leaf_raw_errno") != errno.ENOENT
        or value.get("pre_effects_authorized") is not False
    ):
        fail("root-install preclaim changed its exact authority")
    observations = value.get("live_absence_observations")
    if not isinstance(observations, list) or len(observations) != len(expected):
        fail("root-install preclaim changed live absence cardinality")
    validate_live_observations(observations, expected)
    for field, path, uid, gid, mode in (
        ("private_ancestor_identity", pathlib.Path("/private"), 0, 0, 0o755),
        ("private_tmp_ancestor_identity", pathlib.Path("/private/tmp"), 0, 0, 0o1777),
        ("claim_root_identity", CLAIM_ROOT, 0, 0, 0o700),
    ):
        identity = value.get(field)
        if not isinstance(identity, dict):
            fail(f"root-install preclaim lacks {field}")
        live = directory_identity(path, uid, gid, mode)
        if any(
            identity.get(key) != live[key]
            for key in ("path", "device", "inode", "uid", "gid", "mode")
        ):
            fail(f"root-install preclaim stable directory identity changed: {field}")


def create_or_load_preclaim(
    manifest: dict[str, Any], manifest_bytes: bytes
) -> dict[str, Any]:
    try:
        os.lstat(CLAIM_ROOT)
    except FileNotFoundError:
        parent = open_directory_chain(pathlib.Path("/private/tmp"))
        try:
            os.mkdir(CLAIM_ROOT.name, 0o700, dir_fd=parent)
            descriptor = os.open(
                CLAIM_ROOT.name,
                os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW,
                dir_fd=parent,
            )
            try:
                os.fchown(descriptor, 0, 0)
                os.fchmod(descriptor, 0o700)
                os.fsync(descriptor)
            finally:
                os.close(descriptor)
            os.fsync(parent)
        finally:
            os.close(parent)
    else:
        if not path_present(PRECLAIM_PATH):
            reset_unclaimed_root()
            return create_or_load_preclaim(manifest, manifest_bytes)
    if path_present(PRECLAIM_PATH):
        recover_claim_link_residue(PRECLAIM_PATH)
        data, _ = read_exact_file(PRECLAIM_PATH, 0, 0, 0o400)
        value = parse_canonical(data, "root-install preclaim")
        validate_preclaim(value, manifest, manifest_bytes)
        return value
    value = build_preclaim(manifest, manifest_bytes)
    publish_bytes_no_clobber(canonical(value), PRECLAIM_PATH, 0, 0, 0o400)
    data, _ = read_exact_file(PRECLAIM_PATH, 0, 0, 0o400)
    parsed = parse_canonical(data, "published root-install preclaim")
    validate_preclaim(parsed, manifest, manifest_bytes)
    return parsed


def install_candidate(
    manifest: dict[str, Any], manifest_bytes: bytes
) -> list[dict[str, Any]]:
    for path, uid, gid, mode in INSTALL_DIRECTORIES:
        ensure_directory(pathlib.Path(path), uid, gid, mode)
    for entry in manifest["artifacts"]:
        if entry["role"] == "capability_manifest":
            continue
        source, _ = read_exact_sized_file(
            pathlib.Path(entry["external_path"]),
            501,
            20,
            0o400,
            entry["size"],
            MAX_INSTALLED_ARTIFACT_BYTES,
        )
        install_bytes_prefix_recoverable(
            source,
            pathlib.Path(entry["intended_path"]),
            entry["uid"],
            entry["gid"],
            entry["mode"],
            MAX_INSTALLED_ARTIFACT_BYTES,
        )
        if entry["signing_identifier"] is not None:
            verify_code_identity(pathlib.Path(entry["intended_path"]), entry)
    install_bytes_prefix_recoverable(
        manifest_bytes,
        pathlib.Path(
            "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/"
            "candidate-artifact-manifest.v2.json"
        ),
        0,
        0,
        0o400,
        MAX_DOCUMENT,
    )
    # Re-measure after all child creation so the completion records the directory state presented
    # to the runner before its first Security call.
    return [
        directory_identity(pathlib.Path(path), uid, gid, mode)
        for path, uid, gid, mode in INSTALL_DIRECTORIES
    ]


def executable_physical_digest(identity: dict[str, Any]) -> str:
    return document_sha256(
        {
            "device": identity["device"],
            "inode": identity["inode"],
            "owner_uid": identity["uid"],
            "owner_gid": identity["gid"],
            "mode": identity["mode"],
            "link_count": identity["link_count"],
            "size": identity["size"],
            "modified_seconds": identity["modified_seconds"],
            "modified_nanoseconds": identity["modified_nanoseconds"],
        }
    )


def installed_artifact_identities(manifest: dict[str, Any]) -> list[dict[str, Any]]:
    values = []
    for entry in manifest["artifacts"]:
        path = pathlib.Path(entry["intended_path"])
        data, observed = read_exact_sized_file(
            path,
            entry["uid"],
            entry["gid"],
            entry["mode"],
            entry["size"],
            MAX_INSTALLED_ARTIFACT_BYTES,
        )
        if len(data) != entry["size"] or sha_bytes(data) != entry["sha256"]:
            fail(f"installed artifact bytes changed: {entry['role']}")
        identity = physical_identity(path, observed)
        executable = None
        posture = None
        if entry["signing_identifier"] is not None:
            verify_code_identity(path, entry)
            build_digest = (
                manifest["coordinator_build_digest"]
                if entry["role"]
                in ("coordinator_executable", "alternate_coordinator_executable")
                else manifest["global_build_digest"]
            )
            executable = {
                "source_commit": manifest["source_commit"],
                "source_tree": manifest["source_tree"],
                "source_hashes_sha256": manifest["source_hashes_manifest_sha256"],
                "build_inputs_sha256": build_digest,
                "executable_sha256": entry["sha256"],
                "executable_size": entry["size"],
                "intended_path": entry["intended_path"],
                "physical_identity_sha256": executable_physical_digest(identity),
                "signing_identifier": entry["signing_identifier"],
                "designated_requirement": entry["designated_requirement"],
                "cdhash": entry["cdhash"],
            }
            executable_digest = document_sha256(executable)
            observation = {
                "executable_identity_sha256": executable_digest,
                "cdhash": entry["cdhash"],
                "code_directory_flags": entry["code_flags"],
                "team_identifier": entry["team_id"],
                "entitlements_blob_size": entry["entitlements_size"],
                "entitlements_blob_sha256": entry["entitlements_sha256"],
                "entitlement_keys": [],
            }
            posture = {
                "schema_owner": "substrate.r3-macos-disposable-finalizer-experiment",
                "schema_version": 2,
                "executable_identity_sha256": executable_digest,
                "signature_kind": "ad_hoc",
                "code_directory_flags": entry["code_flags"],
                "hardened_runtime": True,
                "library_validation": True,
                "team_identifier": entry["team_id"],
                "entitlements_blob_size": entry["entitlements_size"],
                "entitlements_blob_sha256": entry["entitlements_sha256"],
                "entitlement_keys": [],
                "verification_observation_sha256": document_sha256(observation),
            }
        values.append(
            {
                "role": entry["role"],
                "path": entry["intended_path"],
                "physical_identity": identity,
                "sha256": entry["sha256"],
                "executable_identity": executable,
                "signing_posture": posture,
            }
        )
    return values


def build_completion(
    manifest: dict[str, Any],
    manifest_bytes: bytes,
    preclaim: dict[str, Any],
    directories: list[dict[str, Any]],
) -> dict[str, Any]:
    require_absent(COMPLETION_PATH)
    artifacts = installed_artifact_identities(manifest)
    preclaim_bytes, preclaim_stat = read_exact_file(PRECLAIM_PATH, 0, 0, 0o400)
    if preclaim_bytes != canonical(preclaim):
        fail("preclaim bytes changed before completion")
    processes = process_snapshot()
    process_digest = document_sha256(processes)
    launchd_probe = next(
        probe for probe in absence_plan() if probe["kind"] == "launchd_label"
    )
    endpoint_probe = next(
        probe for probe in absence_plan() if probe["kind"] == "unix_endpoint"
    )
    value = {
        "schema_owner": "substrate.r3-macos-finalizer-root-install-completion",
        "schema_version": 2,
        "experiment_id": EXPERIMENT_ID,
        "sequence": 2,
        "preclaim_sha256": sha_bytes(preclaim_bytes),
        "candidate_freeze_manifest_sha256": sha_bytes(manifest_bytes),
        "reviewed_admin_block_sha256": manifest["reviewed_admin_block_sha256"],
        "candidate_artifact_set_sha256": manifest["artifact_set_sha256"],
        "install_parent_directory_set_sha256": manifest[
            "install_parent_directory_set_sha256"
        ],
        "preinstall_absence_observation_set_sha256": manifest[
            "preinstall_absence_observation_set_sha256"
        ],
        "rollback_plan_sha256": manifest["rollback_plan_sha256"],
        "supporting_manifest_set_sha256": supporting_manifest_set_sha256(manifest),
        "installed_directories": directories,
        "installed_directory_set_sha256": document_sha256(directories),
        "installed_artifacts": artifacts,
        "installed_artifact_identity_set_sha256": document_sha256(artifacts),
        "preclaim_leaf_identity": physical_identity(PRECLAIM_PATH, preclaim_stat),
        "completion_leaf_lstat_return": -1,
        "completion_leaf_raw_errno": errno.ENOENT,
        "launchd_label_absence": observe_probe(
            launchd_probe, processes, process_digest
        ),
        "endpoint_absence": observe_probe(endpoint_probe, processes, process_digest),
        "install_complete": True,
        "launchd_bootstrap_authorized": False,
    }
    validate_completion(value, manifest, manifest_bytes, preclaim)
    return value


def validate_completion(
    value: dict[str, Any],
    manifest: dict[str, Any],
    manifest_bytes: bytes,
    preclaim: dict[str, Any],
) -> None:
    expected_fields = {
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
    if not isinstance(value, dict) or set(value) != expected_fields:
        fail("root-install completion changed its closed shape")
    if (
        value.get("schema_owner")
        != "substrate.r3-macos-finalizer-root-install-completion"
        or value.get("schema_version") != 2
        or value.get("experiment_id") != EXPERIMENT_ID
        or value.get("sequence") != 2
        or value.get("preclaim_sha256") != document_sha256(preclaim)
        or value.get("candidate_freeze_manifest_sha256") != sha_bytes(manifest_bytes)
        or value.get("reviewed_admin_block_sha256")
        != manifest["reviewed_admin_block_sha256"]
        or value.get("candidate_artifact_set_sha256") != manifest["artifact_set_sha256"]
        or value.get("install_parent_directory_set_sha256")
        != manifest["install_parent_directory_set_sha256"]
        or value.get("preinstall_absence_observation_set_sha256")
        != manifest["preinstall_absence_observation_set_sha256"]
        or value.get("rollback_plan_sha256") != manifest["rollback_plan_sha256"]
        or value.get("supporting_manifest_set_sha256")
        != supporting_manifest_set_sha256(manifest)
        or value.get("installed_directory_set_sha256")
        != document_sha256(value.get("installed_directories"))
        or value.get("installed_artifact_identity_set_sha256")
        != document_sha256(value.get("installed_artifacts"))
        or value.get("completion_leaf_lstat_return") != -1
        or value.get("completion_leaf_raw_errno") != errno.ENOENT
        or value.get("install_complete") is not True
        or value.get("launchd_bootstrap_authorized") is not False
    ):
        fail("root-install completion changed its exact authority")
    if len(value.get("installed_directories", [])) != len(INSTALL_DIRECTORIES):
        fail("root-install completion changed directory cardinality")
    if len(value.get("installed_artifacts", [])) != len(ARTIFACT_SPECS):
        fail("root-install completion changed artifact cardinality")
    for artifact, expected in zip(value["installed_artifacts"], manifest["artifacts"]):
        validate_root_install_artifact_identity(artifact, expected)
    launchd_probe = next(
        probe for probe in absence_plan() if probe["kind"] == "launchd_label"
    )
    endpoint_probe = next(
        probe for probe in absence_plan() if probe["kind"] == "unix_endpoint"
    )
    validate_live_observation(value["launchd_label_absence"], launchd_probe)
    validate_live_observation(value["endpoint_absence"], endpoint_probe)


def create_or_load_completion(
    manifest: dict[str, Any], manifest_bytes: bytes, preclaim: dict[str, Any]
) -> dict[str, Any]:
    if path_present(COMPLETION_PATH):
        recover_claim_link_residue(COMPLETION_PATH)
        data, _ = read_exact_file(COMPLETION_PATH, 0, 0, 0o400)
        value = parse_canonical(data, "root-install completion")
        validate_completion(value, manifest, manifest_bytes, preclaim)
        # Files must remain byte- and inode-exact. Directories are rejoin membranes: mutable
        # size/mtime/link-count are checked by the runner's closed cursor/state validators.
        installed_artifact_identities(manifest)
        for identity, (path, uid, gid, mode) in zip(
            value["installed_directories"], INSTALL_DIRECTORIES
        ):
            live = directory_identity(pathlib.Path(path), uid, gid, mode)
            if any(
                identity[key] != live[key]
                for key in ("path", "device", "inode", "uid", "gid", "mode")
            ):
                fail(
                    f"root-install directory stable identity changed on resume: {path}"
                )
        return value
    completion_temporary = pending_path(COMPLETION_PATH)
    if path_present(completion_temporary):
        claim = open_directory_chain(CLAIM_ROOT)
        try:
            observed = os.stat(
                completion_temporary.name, dir_fd=claim, follow_symlinks=False
            )
            if (
                not stat.S_ISREG(observed.st_mode)
                or observed.st_uid != 0
                or observed.st_gid != 0
                or observed.st_nlink != 1
            ):
                fail("incomplete completion temporary changed physical identity")
            # No runner or native effect can begin before the final completion exists. The raw
            # launchd observation is not reconstructible byte-for-byte, so remove only this exact
            # unpublished leaf, fsync the held claim root, and reobserve it.
            os.unlink(completion_temporary.name, dir_fd=claim)
            os.fsync(claim)
        finally:
            os.close(claim)
    directories = install_candidate(manifest, manifest_bytes)
    value = build_completion(manifest, manifest_bytes, preclaim, directories)
    publish_bytes_no_clobber(canonical(value), COMPLETION_PATH, 0, 0, 0o400)
    data, _ = read_exact_file(COMPLETION_PATH, 0, 0, 0o400)
    parsed = parse_canonical(data, "published root-install completion")
    validate_completion(parsed, manifest, manifest_bytes, preclaim)
    return parsed


def exact_service_absent(
    *, claims_may_remain: bool = False, runner_may_remain: bool = False
) -> None:
    output = run_command(["/bin/launchctl", "print", f"system/{FINALIZER_LABEL}"])
    if output.returncode != 113:
        fail(
            f"finalizer launchd service is not exact absent status 113: {output.returncode}"
        )
    validate_launchctl_not_found_streams(
        FINALIZER_LABEL,
        output.returncode,
        raw_stream(output.stdout),
        raw_stream(output.stderr),
    )
    observe_platform_managed_socket_absence(ENDPOINT_PATH)
    roots = [JOURNAL_ROOT, PUBLISHER_ROOT, CREATOR_ROOT]
    if not runner_may_remain:
        roots.append(RUNNER_ROOT)
    if not claims_may_remain:
        roots.append(CLAIM_ROOT)
    for path in roots:
        if require_durable_absence(path) is None:
            fail(
                f"experiment root parent disappeared before durable absence proof: {path}"
            )
    for path in (REQUEST_PATH, TERMINAL_PATH):
        require_durable_absence(path)
    if path_present(INBOX_ROOT) and any(os.scandir(INBOX_ROOT)):
        fail("coordinator inbox is not exact empty after runner cleanup")
    processes = process_snapshot()
    expected_paths = {
        intended
        for _, _, intended, _, _, _, signing_identifier in ARTIFACT_SPECS
        if signing_identifier is not None
    }
    live = [record for record in processes if record["path"] in expected_paths]
    if live:
        fail(f"one exact experiment process remains after runner cleanup: {live}")


def unlink_exact_file(path: pathlib.Path, expected: dict[str, Any]) -> None:
    artifact = validate_root_install_artifact_identity_shape(expected)
    physical = artifact["physical_identity"]
    if artifact["path"] != str(path) or physical["path"] != str(path):
        fail(f"installed artifact path differs from its cleanup authority: {path}")
    try:
        os.lstat(path)
    except FileNotFoundError:
        require_durable_absence(path)
        return
    data, observed = read_exact_sized_file(
        path,
        physical["uid"],
        physical["gid"],
        physical["mode"] & 0o7777,
        physical["size"],
        MAX_INSTALLED_ARTIFACT_BYTES,
    )
    if (
        sha_bytes(data) != artifact["sha256"]
        or len(data) != physical["size"]
        or physical_identity(path, observed) != physical
    ):
        fail(f"installed artifact changed before exact removal: {path}")
    parent = open_directory_chain(path.parent)
    try:
        before = os.stat(path.name, dir_fd=parent, follow_symlinks=False)
        if not same_stat(before, observed):
            fail(f"installed artifact changed before unlinkat: {path}")
        os.unlink(path.name, dir_fd=parent)
        os.fsync(parent)
    finally:
        os.close(parent)
    if require_durable_absence(path) is None:
        fail(
            f"installed artifact parent disappeared during exact unlink: {path.parent}"
        )


def validate_terminal_runner_cleanup_prefix(claim: dict[str, Any]) -> None:
    authorization = claim["terminal_admin_cleanup_authorization"]
    inventory = authorization["runner_child_inventory"]
    if not path_present(RUNNER_ROOT):
        require_durable_absence(RUNNER_ROOT)
        if path_present(TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH) or any(
            path_present(RUNNER_ROOT / entry["name"]) for entry in inventory
        ):
            fail("terminal runner root absence disagrees with one child path")
        return
    root = directory_identity(RUNNER_ROOT, 0, 0, 0o700)
    expected_root = claim["runner_root_stable_identity"]
    if any(
        root[source] != expected_root[target]
        for source, target in (
            ("path", "path"),
            ("device", "device"),
            ("inode", "inode"),
            ("uid", "owner_uid"),
            ("gid", "owner_gid"),
            ("mode", "mode"),
        )
    ):
        fail("terminal runner root changed during admin cleanup")
    present_names = []
    missing_seen = False
    for entry in inventory:
        present = path_present(RUNNER_ROOT / entry["name"])
        if not present:
            missing_seen = True
            require_durable_absence(RUNNER_ROOT / entry["name"])
        elif missing_seen:
            fail("terminal runner child absence is not the exact reverse-order prefix")
        else:
            read_terminal_runner_child(entry)
            present_names.append(entry["name"])
    authorization_present = path_present(TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH)
    if present_names and not authorization_present:
        fail("terminal cleanup authorization disappeared before runner children")
    expected_live_names = list(present_names)
    if authorization_present:
        authorization_bytes, observed = read_exact_file(
            TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH, 0, 0, 0o600
        )
        if (
            authorization_bytes != canonical(authorization)
            or physical_identity(
                TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH, observed
            )
            != claim["terminal_admin_cleanup_authorization_physical_identity"]
        ):
            fail("terminal cleanup authorization changed before private removal")
        expected_live_names.append(TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH.name)
    else:
        require_durable_absence(TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH)
    descriptor = open_directory_chain(RUNNER_ROOT)
    try:
        live_names = sorted(os.listdir(descriptor))
    finally:
        os.close(descriptor)
    if live_names != sorted(expected_live_names):
        fail("terminal runner root contains an artifact outside its cleanup prefix")


def unlink_terminal_runner_file(
    path: pathlib.Path, expected_bytes: bytes, expected_identity_sha256: str
) -> None:
    if not path_present(path):
        require_durable_absence(path)
        return
    data, observed = read_exact_file(path, 0, 0, 0o600)
    if (
        data != expected_bytes
        or document_sha256(runner_file_physical_identity(observed))
        != expected_identity_sha256
    ):
        fail(f"terminal runner private artifact changed before unlink: {path}")
    parent = open_directory_chain(RUNNER_ROOT)
    try:
        before = os.stat(path.name, dir_fd=parent, follow_symlinks=False)
        if not same_stat(before, observed):
            fail(f"terminal runner private artifact changed before unlinkat: {path}")
        os.unlink(path.name, dir_fd=parent)
        os.fsync(parent)
    finally:
        os.close(parent)
    if require_durable_absence(path) is None:
        fail("terminal runner root disappeared during private-leaf cleanup")


def remove_terminal_runner_root(claim: dict[str, Any]) -> None:
    validate_terminal_runner_cleanup_prefix(claim)
    authorization = claim["terminal_admin_cleanup_authorization"]
    documents = {
        item["name"]: item for item in claim["runner_child_canonical_documents"]
    }
    inventory = {item["name"]: item for item in authorization["runner_child_inventory"]}
    for name in claim["runner_child_removal_order"]:
        entry = inventory[name]
        _, raw = decode_canonical_document_base64url(
            documents[name]["canonical_base64url"],
            f"terminal runner cleanup archive {name}",
        )
        unlink_terminal_runner_file(
            RUNNER_ROOT / name, raw, entry["physical_identity_sha256"]
        )
    if path_present(TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH):
        data, observed = read_exact_file(
            TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH, 0, 0, 0o600
        )
        if (
            data != canonical(authorization)
            or physical_identity(
                TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH, observed
            )
            != claim["terminal_admin_cleanup_authorization_physical_identity"]
        ):
            fail("terminal admin cleanup authorization changed before final unlink")
        parent = open_directory_chain(RUNNER_ROOT)
        try:
            before = os.stat(
                TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH.name,
                dir_fd=parent,
                follow_symlinks=False,
            )
            if not same_stat(before, observed):
                fail("terminal admin cleanup authorization changed before unlinkat")
            os.unlink(
                TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH.name, dir_fd=parent
            )
            os.fsync(parent)
        finally:
            os.close(parent)
        if require_durable_absence(TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH) is None:
            fail("terminal runner root disappeared during authorization cleanup")
    else:
        require_durable_absence(TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH)
    if not path_present(RUNNER_ROOT):
        require_durable_absence(RUNNER_ROOT)
        return
    validate_terminal_runner_cleanup_prefix(claim)
    root = open_directory_chain(RUNNER_ROOT)
    try:
        held = os.fstat(root)
        expected = claim["runner_root_stable_identity"]
        if (
            held.st_dev != expected["device"]
            or held.st_ino != expected["inode"]
            or held.st_uid != expected["owner_uid"]
            or held.st_gid != expected["owner_gid"]
            or held.st_mode != expected["mode"]
            or os.listdir(root)
        ):
            fail("terminal runner root is not its exact empty authorized inode")
        parent = open_directory_chain(RUNNER_ROOT.parent)
        try:
            before = os.stat(RUNNER_ROOT.name, dir_fd=parent, follow_symlinks=False)
            if not same_stat(before, held):
                fail("terminal runner root changed before rmdir")
            os.rmdir(RUNNER_ROOT.name, dir_fd=parent)
            os.fsync(parent)
        finally:
            os.close(parent)
    finally:
        os.close(root)
    if require_durable_absence(RUNNER_ROOT) is None:
        fail("terminal runner parent disappeared during exact rmdir")


def cleanup_installed_candidate(
    manifest: dict[str, Any],
    manifest_bytes: bytes,
    completion: dict[str, Any],
    installed_manifest_identity: dict[str, Any],
) -> None:
    if len(completion.get("installed_artifacts", [])) != len(manifest["artifacts"]):
        fail("installed cleanup authority changed artifact cardinality")
    for artifact, expected in zip(completion["installed_artifacts"], manifest["artifacts"]):
        validate_root_install_artifact_identity(artifact, expected)
    by_role = {entry["role"]: entry for entry in completion["installed_artifacts"]}
    # The runner owns the alternate-path negative-control removal. All other installed artifacts
    # are removed here in the exact manifest rollback order, with absence accepted only as a
    # prefix of this already-started cleanup sequence.
    cleanup_roles = [
        spec[0] for spec in reversed(ARTIFACT_SPECS) if spec[0] != "capability_manifest"
    ]
    ordered_prefix_roles = [
        role for role in cleanup_roles if role != "alternate_coordinator_executable"
    ]
    seen_present = False
    for role in ordered_prefix_roles:
        present = path_present(pathlib.Path(by_role[role]["path"]))
        if present:
            seen_present = True
        elif seen_present:
            fail("installed artifact absence is not the exact cleanup prefix")
    installed_manifest = INSTALLED_CANDIDATE_MANIFEST_PATH
    if not path_present(installed_manifest) and any(
        path_present(pathlib.Path(by_role[role]["path"]))
        for role in ordered_prefix_roles
    ):
        fail("installed manifest disappeared before artifact cleanup completed")
    for role in cleanup_roles:
        path = pathlib.Path(by_role[role]["path"])
        if role == "alternate_coordinator_executable" and not path_present(path):
            continue
        unlink_exact_file(path, by_role[role])
    if path_present(installed_manifest):
        data, observed = read_exact_file(installed_manifest, 0, 0, 0o400)
        identity = physical_identity(installed_manifest, observed)
        if data != manifest_bytes or (
            identity["device"] != installed_manifest_identity["device"]
            or identity["inode"] != installed_manifest_identity["inode"]
            or identity["uid"] != installed_manifest_identity["owner_uid"]
            or identity["gid"] != installed_manifest_identity["owner_gid"]
            or identity["mode"] & 0o7777 != installed_manifest_identity["mode"]
            or identity["link_count"] != installed_manifest_identity["link_count"]
            or identity["size"] != installed_manifest_identity["size"]
        ):
            fail("installed candidate manifest changed before cleanup")
        parent = open_directory_chain(installed_manifest.parent)
        try:
            before = os.stat(
                installed_manifest.name, dir_fd=parent, follow_symlinks=False
            )
            if not same_stat(before, observed):
                fail("installed candidate manifest changed before unlinkat")
            os.unlink(installed_manifest.name, dir_fd=parent)
            os.fsync(parent)
        finally:
            os.close(parent)
        if require_durable_absence(installed_manifest) is None:
            fail("installed manifest parent disappeared during exact unlink")
    else:
        require_durable_absence(installed_manifest)
    directory_cleanup_order = [
        value
        for value in reversed(INSTALL_DIRECTORIES[:-1])
        if pathlib.Path(value[0]) != JOURNAL_ROOT
    ]
    seen_present = False
    for path, _, _, _ in directory_cleanup_order:
        present = path_present(pathlib.Path(path))
        if present:
            seen_present = True
        elif seen_present:
            fail("installed directory absence is not the exact cleanup prefix")
    for path, uid, gid, mode in reversed(INSTALL_DIRECTORIES[:-1]):
        candidate = pathlib.Path(path)
        if candidate == JOURNAL_ROOT:
            require_absent(candidate)
            continue
        try:
            identity = directory_identity(candidate, uid, gid, mode)
        except FileNotFoundError:
            require_durable_absence(candidate)
            continue
        matching = [
            expected
            for expected in completion["installed_directories"]
            if expected["path"] == path
        ]
        if len(matching) != 1 or any(
            identity[key] != matching[0][key]
            for key in ("device", "inode", "uid", "gid", "mode")
        ):
            fail(f"installed directory changed before exact rmdir: {path}")
        if any(os.scandir(candidate)):
            fail(f"installed directory is not exact empty before rmdir: {path}")
        parent = open_directory_chain(candidate.parent)
        try:
            os.rmdir(candidate.name, dir_fd=parent)
            os.fsync(parent)
        finally:
            os.close(parent)
        if require_durable_absence(candidate) is None:
            fail(f"installed directory parent disappeared during exact rmdir: {path}")
    # The UID-501 global exchange is retained external evidence by contract.
    matching_global = [
        value
        for value in completion["installed_directories"]
        if value["path"] == str(GLOBAL_EXCHANGE)
    ]
    live_global = directory_identity(GLOBAL_EXCHANGE, 501, 0, 0o700)
    if len(matching_global) != 1 or any(
        live_global[key] != matching_global[0][key]
        for key in ("path", "device", "inode", "uid", "gid", "mode")
    ):
        fail("retained global evidence exchange changed stable identity")


def remove_root_install_claims(
    claims: dict[str, Any], preclaim: dict[str, Any], completion: dict[str, Any]
) -> None:
    expected_completion = claims["completion_leaf_identity"]
    expected_preclaim = completion["preclaim_leaf_identity"]
    if not path_present(CLAIM_ROOT):
        if require_durable_absence(CLAIM_ROOT) is None:
            fail("root-install claim parent disappeared before absence proof")
        return
    validate_live_root_install_claims(claims, preclaim, completion)
    for path, value, expected in (
        (COMPLETION_PATH, completion, expected_completion),
        (PRECLAIM_PATH, preclaim, expected_preclaim),
    ):
        if not path_present(path):
            continue
        data, observed = read_exact_file(path, 0, 0, 0o400)
        if data != canonical(value) or physical_identity(path, observed) != expected:
            fail(f"root-install claim leaf changed before exact removal: {path}")
        parent = open_directory_chain(CLAIM_ROOT)
        try:
            before = os.stat(path.name, dir_fd=parent, follow_symlinks=False)
            if not same_stat(before, observed):
                fail(f"root-install claim leaf changed before unlinkat: {path}")
            os.unlink(path.name, dir_fd=parent)
            os.fsync(parent)
        finally:
            os.close(parent)
        if require_durable_absence(path) is None:
            fail(f"root-install claim parent disappeared during leaf removal: {path}")
    root = open_directory_chain(CLAIM_ROOT)
    try:
        held = os.fstat(root)
        live = os.lstat(CLAIM_ROOT)
        if not same_stat(held, live) or list(os.scandir(CLAIM_ROOT)):
            fail("root-install claim root is not the exact empty held directory")
        parent = open_directory_chain(CLAIM_ROOT.parent)
        try:
            before = os.stat(CLAIM_ROOT.name, dir_fd=parent, follow_symlinks=False)
            if before.st_dev != held.st_dev or before.st_ino != held.st_ino:
                fail("root-install claim root changed before rmdir")
            os.rmdir(CLAIM_ROOT.name, dir_fd=parent)
            os.fsync(parent)
        finally:
            os.close(parent)
    finally:
        os.close(root)
    if require_durable_absence(CLAIM_ROOT) is None:
        fail("root-install claim parent disappeared during final rmdir")


def admin_restoration_absence_paths(manifest: dict[str, Any]) -> list[str]:
    values = [
        entry["intended_path"]
        for entry in manifest["artifacts"]
        if entry["role"] != "capability_manifest"
    ]
    values.extend(
        [
            "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/"
            "candidate-artifact-manifest.v2.json",
            *[path for path, _, _, _ in INSTALL_DIRECTORIES[:-1]],
            str(PUBLISHER_ROOT),
            str(RUNNER_ROOT),
            str(CREATOR_ROOT),
            str(REQUEST_PATH),
            str(TERMINAL_PATH),
            str(PRECLAIM_PATH),
            str(COMPLETION_PATH),
            str(CLAIM_ROOT),
        ]
    )
    ordered = list(dict.fromkeys(values))
    for value in ordered:
        candidate = pathlib.Path(value)
        if (
            candidate == PLATFORM_MANAGED_SOCKET_PARENT
            or PLATFORM_MANAGED_SOCKET_PARENT in candidate.parents
        ):
            fail(
                "ordinary admin restoration target entered the platform-managed "
                "socket parent"
            )
    return ordered


def observe_admin_restoration_absence(
    manifest: dict[str, Any], expected_global: dict[str, Any]
) -> dict[str, Any]:
    platform_socket = observe_platform_managed_socket_absence(ENDPOINT_PATH)
    paths = []
    ordered_absences = admin_restoration_absence_paths(manifest)
    for value in ordered_absences:
        require_absent(pathlib.Path(value))
        paths.append({"path": value, "lstat_return": -1, "raw_errno": errno.ENOENT})
    durability = []
    for value in ordered_absences:
        observation = require_durable_absence(pathlib.Path(value))
        if observation is not None:
            durability.append(observation)
    required_boundaries = {
        "/Library/Application Support/Atomize",
        str(JOURNAL_ROOT),
        str(PUBLISHER_ROOT),
        str(RUNNER_ROOT),
        str(CREATOR_ROOT),
        str(CLAIM_ROOT),
    }
    observed_boundaries = {value["path"] for value in durability}
    if not required_boundaries.issubset(observed_boundaries):
        fail("admin restoration omitted one outer durable-absence boundary")
    launchd = run_command(["/bin/launchctl", "print", f"system/{FINALIZER_LABEL}"])
    if launchd.returncode != 113:
        fail("finalizer launchd service reappeared before admin restoration receipt")
    launchd_stdout = raw_stream(launchd.stdout)
    launchd_stderr = raw_stream(launchd.stderr)
    validate_launchctl_not_found_streams(
        FINALIZER_LABEL, launchd.returncode, launchd_stdout, launchd_stderr
    )
    processes = process_snapshot()
    executable_paths = {
        entry["intended_path"]
        for entry in manifest["artifacts"]
        if entry["signing_identifier"] is not None
    }
    matching = [value for value in processes if value["path"] in executable_paths]
    if matching:
        fail(
            f"one installed experiment process remains after admin cleanup: {matching}"
        )
    process_snapshot_stream = raw_stream(canonical(processes))
    validate_process_snapshot_records(
        processes,
        process_snapshot_stream,
        process_snapshot_stream["sha256"],
    )
    live_global = directory_identity(GLOBAL_EXCHANGE, 501, 0, 0o700)
    if any(
        live_global[key] != expected_global[key]
        for key in ("path", "device", "inode", "uid", "gid", "mode")
    ):
        fail("global evidence exchange changed before admin restoration receipt")
    return {
        "path_absences": paths,
        "path_absence_set_sha256": document_sha256(paths),
        "durable_absences": durability,
        "durable_absence_set_sha256": document_sha256(durability),
        "platform_managed_socket_absence": platform_socket,
        "launchd_label": FINALIZER_LABEL,
        "launchctl_print_not_found_exit_status": launchd.returncode,
        "launchctl_print_stdout": launchd_stdout,
        "launchctl_print_stderr": launchd_stderr,
        "matching_experiment_processes": matching,
        "process_records": processes,
        "process_snapshot": process_snapshot_stream,
        "process_snapshot_sha256": process_snapshot_stream["sha256"],
        "global_exchange_stable_identity": {
            key: live_global[key]
            for key in ("path", "device", "inode", "uid", "gid", "mode")
        },
    }


def admin_restoration_binding(value: dict[str, Any]) -> str:
    body = {key: item for key, item in value.items() if key != "restoration_sha256"}
    return document_sha256(
        {
            "domain": "substrate.r3-macos-finalizer-admin-install-restoration.v2",
            "receipt": body,
        }
    )


def validate_admin_restoration_receipt(
    value: dict[str, Any],
    manifest_bytes: bytes,
    root_authority_sha256: str,
    global_bytes: bytes,
    export_bytes: bytes,
    acknowledgement_bytes: bytes,
    cleanup_bytes: bytes,
    claims: dict[str, Any],
) -> None:
    if (
        set(value)
        != {
            "schema_owner",
            "schema_version",
            "experiment_id",
            "root_install_authority_sha256",
            "candidate_freeze_manifest_sha256",
            "global_pre_effect_sha256",
            "native_evidence_export_sha256",
            "native_evidence_acknowledgement_sha256",
            "native_evidence_cleanup_sha256",
            "root_install_claims_binding_sha256",
            "root_install_preclaim_sha256",
            "root_install_completion_sha256",
            "observation",
            "restoration_sha256",
        }
        or value.get("schema_owner")
        != "substrate.r3-macos-finalizer-admin-install-restoration"
        or value.get("schema_version") != 2
        or value.get("experiment_id") != EXPERIMENT_ID
        or value.get("root_install_authority_sha256") != root_authority_sha256
        or value.get("candidate_freeze_manifest_sha256") != sha_bytes(manifest_bytes)
        or value.get("global_pre_effect_sha256") != sha_bytes(global_bytes)
        or value.get("native_evidence_export_sha256") != sha_bytes(export_bytes)
        or value.get("native_evidence_acknowledgement_sha256")
        != sha_bytes(acknowledgement_bytes)
        or value.get("native_evidence_cleanup_sha256") != sha_bytes(cleanup_bytes)
        or value.get("root_install_claims_binding_sha256") != document_sha256(claims)
        or value.get("root_install_preclaim_sha256")
        != document_sha256(claims["preclaim"])
        or value.get("root_install_completion_sha256")
        != document_sha256(claims["completion"])
        or not isinstance(value.get("observation"), dict)
        or value.get("restoration_sha256") != admin_restoration_binding(value)
    ):
        fail("admin installation restoration receipt changed its exact authority")
    manifest = parse_canonical(manifest_bytes, "admin restoration candidate manifest")
    if not isinstance(manifest, dict):
        fail("admin restoration candidate manifest is not an object")
    expected_globals = [
        item
        for item in claims["completion"]["installed_directories"]
        if item["path"] == str(GLOBAL_EXCHANGE)
    ]
    if len(expected_globals) != 1:
        fail("admin restoration claims lack one retained evidence-root identity")
    validate_admin_restoration_receipt_observation(
        value["observation"], manifest, claims, expected_globals[0]
    )


def publish_or_validate_admin_restoration_receipt(
    manifest: dict[str, Any],
    manifest_bytes: bytes,
    root_authority_sha256: str,
    global_bytes: bytes,
    export_bytes: bytes,
    acknowledgement_bytes: bytes,
    cleanup_bytes: bytes,
    claims: dict[str, Any],
    expected_global: dict[str, Any],
) -> dict[str, Any]:
    exact_service_absent()
    for path in admin_restoration_absence_paths(manifest):
        require_absent(pathlib.Path(path))
    if path_present(ADMIN_RESTORATION_RECEIPT_PATH):
        temporary = pending_path(ADMIN_RESTORATION_RECEIPT_PATH)
        if path_present(temporary):
            parent = open_directory_chain(GLOBAL_EXCHANGE)
            try:
                final = os.stat(
                    ADMIN_RESTORATION_RECEIPT_PATH.name,
                    dir_fd=parent,
                    follow_symlinks=False,
                )
                residue = os.stat(temporary.name, dir_fd=parent, follow_symlinks=False)
                if (
                    not stat.S_ISREG(final.st_mode)
                    or final.st_uid != 0
                    or final.st_gid != 0
                    or final.st_mode & 0o7777 != 0o444
                    or final.st_nlink != 2
                    or final.st_dev != residue.st_dev
                    or final.st_ino != residue.st_ino
                ):
                    fail("admin restoration link residue changed same-inode state")
                os.unlink(temporary.name, dir_fd=parent)
                os.fsync(parent)
            finally:
                os.close(parent)
        data, _ = read_exact_file(ADMIN_RESTORATION_RECEIPT_PATH, 0, 0, 0o444)
        value = parse_canonical(data, "admin installation restoration receipt")
        if not isinstance(value, dict):
            fail("admin installation restoration receipt is not an object")
        validate_admin_restoration_receipt(
            value,
            manifest_bytes,
            root_authority_sha256,
            global_bytes,
            export_bytes,
            acknowledgement_bytes,
            cleanup_bytes,
            claims,
        )
        observe_admin_restoration_absence(manifest, expected_global)
        return value
    temporary = pending_path(ADMIN_RESTORATION_RECEIPT_PATH)
    if path_present(temporary):
        parent = open_directory_chain(GLOBAL_EXCHANGE)
        try:
            observed = os.stat(temporary.name, dir_fd=parent, follow_symlinks=False)
            if (
                not stat.S_ISREG(observed.st_mode)
                or observed.st_uid != 0
                or observed.st_gid != 0
                or observed.st_nlink != 1
            ):
                fail("admin restoration temporary changed physical identity")
            os.unlink(temporary.name, dir_fd=parent)
            os.fsync(parent)
        finally:
            os.close(parent)
    value = {
        "schema_owner": "substrate.r3-macos-finalizer-admin-install-restoration",
        "schema_version": 2,
        "experiment_id": EXPERIMENT_ID,
        "root_install_authority_sha256": root_authority_sha256,
        "candidate_freeze_manifest_sha256": sha_bytes(manifest_bytes),
        "global_pre_effect_sha256": sha_bytes(global_bytes),
        "native_evidence_export_sha256": sha_bytes(export_bytes),
        "native_evidence_acknowledgement_sha256": sha_bytes(
            acknowledgement_bytes
        ),
        "native_evidence_cleanup_sha256": sha_bytes(cleanup_bytes),
        "root_install_claims_binding_sha256": document_sha256(claims),
        "root_install_preclaim_sha256": document_sha256(claims["preclaim"]),
        "root_install_completion_sha256": document_sha256(claims["completion"]),
        "observation": observe_admin_restoration_absence(manifest, expected_global),
        "restoration_sha256": "",
    }
    value["restoration_sha256"] = admin_restoration_binding(value)
    validate_admin_restoration_receipt(
        value,
        manifest_bytes,
        root_authority_sha256,
        global_bytes,
        export_bytes,
        acknowledgement_bytes,
        cleanup_bytes,
        claims,
    )
    publish_bytes_no_clobber(
        canonical(value), ADMIN_RESTORATION_RECEIPT_PATH, 0, 0, 0o444
    )
    data, _ = read_exact_file(ADMIN_RESTORATION_RECEIPT_PATH, 0, 0, 0o444)
    parsed = parse_canonical(data, "published admin installation restoration receipt")
    if parsed != value:
        fail("admin installation restoration receipt changed after publication")
    return value


def terminal_admin_restoration_binding(value: dict[str, Any]) -> str:
    body = {key: item for key, item in value.items() if key != "restoration_sha256"}
    return document_sha256(
        {
            "domain": (
                "substrate.r3-macos-finalizer-terminal-admin-restoration.v2"
            ),
            "receipt": body,
        }
    )


def validate_terminal_admin_restoration_receipt(
    value: Any,
    manifest: dict[str, Any],
    manifest_bytes: bytes,
    root_authority_sha256: str,
    cleanup_authority: dict[str, Any],
    claims: dict[str, Any],
    installed_manifest_identity: dict[str, Any],
    claim: dict[str, Any],
    claim_bytes: bytes,
    claim_observed: os.stat_result,
    expected_global: dict[str, Any],
) -> dict[str, Any]:
    expected_fields = {
        "schema_owner",
        "schema_version",
        "experiment_id",
        "receipt_path",
        "terminal_outcome_kind",
        "native_evidence_pass",
        "terminal_failure_restored",
        "root_install_authority_sha256",
        "candidate_freeze_manifest_sha256",
        "terminal_cleanup_authority_kind",
        "terminal_cleanup_authority_sha256",
        "global_pre_effect_sha256",
        "terminal_admin_cleanup_claim_sha256",
        "terminal_admin_cleanup_claim_physical_identity_sha256",
        "terminal_admin_cleanup_authorization_sha256",
        "terminal_admin_cleanup_authorization_physical_identity_sha256",
        "runner_root_stable_identity_sha256",
        "runner_child_inventory_sha256",
        "runner_child_canonical_documents_sha256",
        "runner_child_removal_order_sha256",
        "root_install_claims_binding_sha256",
        "root_install_preclaim_sha256",
        "root_install_completion_sha256",
        "installed_candidate_manifest_identity_sha256",
        "observation",
        "restoration_sha256",
    }
    if not isinstance(value, dict) or set(value) != expected_fields:
        fail("terminal admin restoration receipt changed its closed shape")
    authorization = claim["terminal_admin_cleanup_authorization"]
    if (
        value.get("schema_owner")
        != "substrate.r3-macos-finalizer-terminal-admin-restoration"
        or value.get("schema_version") != 2
        or value.get("experiment_id") != EXPERIMENT_ID
        or value.get("receipt_path")
        != str(TERMINAL_ADMIN_RESTORATION_RECEIPT_PATH)
        or value.get("terminal_outcome_kind")
        != authorization["terminal_outcome_kind"]
        or value.get("native_evidence_pass") is not False
        or value.get("terminal_failure_restored") is not True
        or value.get("root_install_authority_sha256") != root_authority_sha256
        or value.get("candidate_freeze_manifest_sha256")
        != sha_bytes(manifest_bytes)
        or value.get("terminal_cleanup_authority_kind")
        != cleanup_authority["authority_kind"]
        or value.get("terminal_cleanup_authority_sha256")
        != document_sha256(cleanup_authority)
        or value.get("global_pre_effect_sha256")
        != cleanup_authority["global_pre_effect_sha256"]
        or value.get("terminal_admin_cleanup_claim_sha256")
        != sha_bytes(claim_bytes)
        or value.get("terminal_admin_cleanup_claim_physical_identity_sha256")
        != document_sha256(
            physical_identity(TERMINAL_ADMIN_CLEANUP_CLAIM_PATH, claim_observed)
        )
        or value.get("terminal_admin_cleanup_authorization_sha256")
        != claim["terminal_admin_cleanup_authorization_sha256"]
        or value.get(
            "terminal_admin_cleanup_authorization_physical_identity_sha256"
        )
        != claim["terminal_admin_cleanup_authorization_physical_identity_sha256"]
        or value.get("runner_root_stable_identity_sha256")
        != claim["runner_root_stable_identity_sha256"]
        or value.get("runner_child_inventory_sha256")
        != authorization["runner_child_inventory_sha256"]
        or value.get("runner_child_canonical_documents_sha256")
        != claim["runner_child_canonical_documents_sha256"]
        or value.get("runner_child_removal_order_sha256")
        != claim["runner_child_removal_order_sha256"]
        or value.get("root_install_claims_binding_sha256")
        != document_sha256(claims)
        or value.get("root_install_preclaim_sha256")
        != document_sha256(claims["preclaim"])
        or value.get("root_install_completion_sha256")
        != document_sha256(claims["completion"])
        or value.get("installed_candidate_manifest_identity_sha256")
        != document_sha256(installed_manifest_identity)
        or not isinstance(value.get("observation"), dict)
        or value.get("restoration_sha256")
        != terminal_admin_restoration_binding(value)
    ):
        fail("terminal admin restoration receipt changed its exact authority")
    validate_admin_restoration_receipt_observation(
        value["observation"], manifest, claims, expected_global
    )
    return value


def validate_admin_restoration_receipt_observation(
    observation: dict[str, Any],
    manifest: dict[str, Any],
    claims: dict[str, Any],
    expected_global: dict[str, Any],
) -> None:
    expected_globals = [
        item
        for item in claims["completion"]["installed_directories"]
        if item["path"] == str(GLOBAL_EXCHANGE)
    ]
    if len(expected_globals) != 1 or expected_globals[0] != expected_global:
        fail("terminal restoration global identity left the root-install claims")
    if set(observation) != {
        "path_absences",
        "path_absence_set_sha256",
        "durable_absences",
        "durable_absence_set_sha256",
        "platform_managed_socket_absence",
        "launchd_label",
        "launchctl_print_not_found_exit_status",
        "launchctl_print_stdout",
        "launchctl_print_stderr",
        "matching_experiment_processes",
        "process_records",
        "process_snapshot",
        "process_snapshot_sha256",
        "global_exchange_stable_identity",
    }:
        fail("terminal admin restoration observation changed its closed shape")
    expected_paths = admin_restoration_absence_paths(manifest)
    expected_absences = [
        {"path": path, "lstat_return": -1, "raw_errno": errno.ENOENT}
        for path in expected_paths
    ]
    durability = observation.get("durable_absences")
    platform_socket = observation.get("platform_managed_socket_absence")
    if (
        observation.get("path_absences") != expected_absences
        or observation.get("path_absence_set_sha256")
        != document_sha256(expected_absences)
        or not isinstance(durability, list)
        or observation.get("durable_absence_set_sha256")
        != document_sha256(durability)
        or observation.get("launchd_label") != FINALIZER_LABEL
        or observation.get("launchctl_print_not_found_exit_status") != 113
        or not is_sha256(observation.get("process_snapshot_sha256"))
    ):
        fail("terminal admin restoration observation changed its exact absence evidence")
    validate_platform_managed_socket_absence(platform_socket)
    records = validate_process_snapshot_records(
        observation.get("process_records"),
        observation.get("process_snapshot"),
        observation.get("process_snapshot_sha256"),
    )
    executable_paths = {
        entry["intended_path"]
        for entry in manifest["artifacts"]
        if entry["signing_identifier"] is not None
    }
    matching = [record for record in records if record.get("path") in executable_paths]
    if observation.get("matching_experiment_processes") != matching or matching:
        fail("terminal admin restoration snapshot contains an experiment process")
    validate_raw_stream(
        observation.get("launchctl_print_stdout"),
        "terminal admin restoration launchctl stdout",
    )
    validate_raw_stream(
        observation.get("launchctl_print_stderr"),
        "terminal admin restoration launchctl stderr",
    )
    validate_launchctl_not_found_streams(
        FINALIZER_LABEL,
        observation["launchctl_print_not_found_exit_status"],
        observation["launchctl_print_stdout"],
        observation["launchctl_print_stderr"],
    )
    durable_paths = []
    for item in durability:
        if (
            not isinstance(item, dict)
            or set(item)
            != {
                "path",
                "parent_path",
                "parent_device",
                "parent_inode",
                "parent_uid",
                "parent_gid",
                "parent_mode",
                "parent_fsync_return",
                "child_fstatat_return",
                "child_raw_errno",
                "parent_reopened_same_stable_identity",
            }
            or item.get("path") not in expected_paths
            or item.get("parent_path") != str(pathlib.Path(item["path"]).parent)
            or item.get("parent_fsync_return") != 0
            or item.get("child_fstatat_return") != -1
            or item.get("child_raw_errno") != errno.ENOENT
            or item.get("parent_reopened_same_stable_identity") is not True
            or not all(
                isinstance(item.get(field), int)
                for field in (
                    "parent_device",
                    "parent_inode",
                    "parent_uid",
                    "parent_gid",
                    "parent_mode",
                )
            )
            or not stat.S_ISDIR(item["parent_mode"])
        ):
            fail("terminal admin restoration durable absence changed")
        durable_paths.append(item["path"])
    if len(durable_paths) != len(set(durable_paths)):
        fail("terminal admin restoration duplicated a durable absence")
    required_boundaries = {
        "/Library/Application Support/Atomize",
        str(JOURNAL_ROOT),
        str(PUBLISHER_ROOT),
        str(RUNNER_ROOT),
        str(CREATOR_ROOT),
        str(CLAIM_ROOT),
    }
    if not required_boundaries.issubset(set(durable_paths)):
        fail("terminal admin restoration lacks a durable outer absence")
    stable = observation.get("global_exchange_stable_identity")
    if (
        not isinstance(stable, dict)
        or set(stable) != {"path", "device", "inode", "uid", "gid", "mode"}
        or any(stable[key] != expected_global[key] for key in stable)
    ):
        fail("terminal admin restoration changed the retained exchange identity")


def publish_or_validate_terminal_admin_restoration_receipt(
    manifest: dict[str, Any],
    manifest_bytes: bytes,
    root_authority_sha256: str,
    cleanup_authority: dict[str, Any],
    claims: dict[str, Any],
    installed_manifest_identity: dict[str, Any],
    claim: dict[str, Any],
    expected_global: dict[str, Any],
) -> dict[str, Any]:
    claim_bytes, claim_observed = read_exact_file(
        TERMINAL_ADMIN_CLEANUP_CLAIM_PATH, 0, 0, 0o444
    )
    if claim_bytes != canonical(claim):
        fail("terminal admin cleanup claim changed before restoration publication")
    if path_present(TERMINAL_ADMIN_RESTORATION_RECEIPT_PATH):
        finish_no_clobber_link_residue(
            TERMINAL_ADMIN_RESTORATION_RECEIPT_PATH, 0, 0, 0o444
        )
        data, _ = read_exact_file(
            TERMINAL_ADMIN_RESTORATION_RECEIPT_PATH, 0, 0, 0o444
        )
        value = parse_canonical(data, "terminal admin restoration receipt")
        validate_terminal_admin_restoration_receipt(
            value,
            manifest,
            manifest_bytes,
            root_authority_sha256,
            cleanup_authority,
            claims,
            installed_manifest_identity,
            claim,
            claim_bytes,
            claim_observed,
            expected_global,
        )
        observe_admin_restoration_absence(manifest, expected_global)
        return value
    temporary = pending_path(TERMINAL_ADMIN_RESTORATION_RECEIPT_PATH)
    if path_present(temporary):
        parent = open_directory_chain(GLOBAL_EXCHANGE)
        try:
            observed = os.stat(temporary.name, dir_fd=parent, follow_symlinks=False)
            if (
                not stat.S_ISREG(observed.st_mode)
                or observed.st_uid != 0
                or observed.st_gid != 0
                or observed.st_nlink != 1
            ):
                fail("terminal admin restoration temporary changed identity")
            os.unlink(temporary.name, dir_fd=parent)
            os.fsync(parent)
        finally:
            os.close(parent)
    authorization = claim["terminal_admin_cleanup_authorization"]
    value = {
        "schema_owner": "substrate.r3-macos-finalizer-terminal-admin-restoration",
        "schema_version": 2,
        "experiment_id": EXPERIMENT_ID,
        "receipt_path": str(TERMINAL_ADMIN_RESTORATION_RECEIPT_PATH),
        "terminal_outcome_kind": authorization["terminal_outcome_kind"],
        "native_evidence_pass": False,
        "terminal_failure_restored": True,
        "root_install_authority_sha256": root_authority_sha256,
        "candidate_freeze_manifest_sha256": sha_bytes(manifest_bytes),
        "terminal_cleanup_authority_kind": cleanup_authority["authority_kind"],
        "terminal_cleanup_authority_sha256": document_sha256(cleanup_authority),
        "global_pre_effect_sha256": cleanup_authority["global_pre_effect_sha256"],
        "terminal_admin_cleanup_claim_sha256": sha_bytes(claim_bytes),
        "terminal_admin_cleanup_claim_physical_identity_sha256": document_sha256(
            physical_identity(TERMINAL_ADMIN_CLEANUP_CLAIM_PATH, claim_observed)
        ),
        "terminal_admin_cleanup_authorization_sha256": claim[
            "terminal_admin_cleanup_authorization_sha256"
        ],
        "terminal_admin_cleanup_authorization_physical_identity_sha256": claim[
            "terminal_admin_cleanup_authorization_physical_identity_sha256"
        ],
        "runner_root_stable_identity_sha256": claim[
            "runner_root_stable_identity_sha256"
        ],
        "runner_child_inventory_sha256": authorization[
            "runner_child_inventory_sha256"
        ],
        "runner_child_canonical_documents_sha256": claim[
            "runner_child_canonical_documents_sha256"
        ],
        "runner_child_removal_order_sha256": claim[
            "runner_child_removal_order_sha256"
        ],
        "root_install_claims_binding_sha256": document_sha256(claims),
        "root_install_preclaim_sha256": document_sha256(claims["preclaim"]),
        "root_install_completion_sha256": document_sha256(claims["completion"]),
        "installed_candidate_manifest_identity_sha256": document_sha256(
            installed_manifest_identity
        ),
        "observation": observe_admin_restoration_absence(manifest, expected_global),
        "restoration_sha256": "",
    }
    value["restoration_sha256"] = terminal_admin_restoration_binding(value)
    validate_terminal_admin_restoration_receipt(
        value,
        manifest,
        manifest_bytes,
        root_authority_sha256,
        cleanup_authority,
        claims,
        installed_manifest_identity,
        claim,
        claim_bytes,
        claim_observed,
        expected_global,
    )
    publish_bytes_no_clobber(
        canonical(value), TERMINAL_ADMIN_RESTORATION_RECEIPT_PATH, 0, 0, 0o444
    )
    data, _ = read_exact_file(
        TERMINAL_ADMIN_RESTORATION_RECEIPT_PATH, 0, 0, 0o444
    )
    if data != canonical(value):
        fail("terminal admin restoration receipt changed after publication")
    return value


def complete_terminal_admin_cleanup(
    manifest: dict[str, Any],
    manifest_bytes: bytes,
    root_authority_sha256: str,
) -> dict[str, Any]:
    claim = load_or_create_terminal_cleanup_claim(
        manifest,
        manifest_bytes,
        root_authority_sha256,
    )
    cleanup_authority = claim["terminal_cleanup_authority"]
    claims = cleanup_authority["root_install_claims"]
    preclaim = claims["preclaim"]
    completion = claims["completion"]
    installed_manifest_identity = cleanup_authority[
        "installed_candidate_manifest"
    ]
    expected_global_values = [
        value
        for value in completion["installed_directories"]
        if value["path"] == str(GLOBAL_EXCHANGE)
    ]
    if len(expected_global_values) != 1:
        fail("root-install completion lacks one global exchange identity")
    expected_global = expected_global_values[0]
    if path_present(CLAIM_ROOT):
        validate_live_root_install_claims(claims, preclaim, completion)
    # Do not destroy the only private terminal archive until service/journal absence has been
    # independently proved.  The immutable external claim remains the crash-recovery authority.
    exact_service_absent(claims_may_remain=True, runner_may_remain=True)
    remove_terminal_runner_root(claim)
    exact_service_absent(claims_may_remain=True)
    cleanup_installed_candidate(
        manifest, manifest_bytes, completion, installed_manifest_identity
    )
    remove_root_install_claims(claims, preclaim, completion)
    return publish_or_validate_terminal_admin_restoration_receipt(
        manifest,
        manifest_bytes,
        root_authority_sha256,
        cleanup_authority,
        claims,
        installed_manifest_identity,
        claim,
        expected_global,
    )


def complete_successful_admin_cleanup(
    manifest: dict[str, Any],
    manifest_bytes: bytes,
    root_authority_sha256: str,
) -> dict[str, Any]:
    (
        claims,
        preclaim,
        completion,
        _,
        installed_manifest_identity,
        acknowledgement_bytes,
    ) = load_success_cleanup_authority(manifest, manifest_bytes)
    expected_global_values = [
        value
        for value in completion["installed_directories"]
        if value["path"] == str(GLOBAL_EXCHANGE)
    ]
    if len(expected_global_values) != 1:
        fail("root-install completion lacks one global exchange identity")
    expected_global = expected_global_values[0]
    if path_present(CLAIM_ROOT):
        validate_live_root_install_claims(claims, preclaim, completion)
    exact_service_absent(claims_may_remain=True)
    cleanup_installed_candidate(
        manifest, manifest_bytes, completion, installed_manifest_identity
    )
    remove_root_install_claims(claims, preclaim, completion)
    global_bytes, _ = read_exact_file(GLOBAL_PRE_EFFECT_PATH, 0, 0, 0o444)
    export_bytes, _ = read_exact_file(NATIVE_EVIDENCE_EXPORT_PATH, 0, 0, 0o444)
    live_acknowledgement_bytes, _ = read_exact_file(
        NATIVE_EVIDENCE_ACKNOWLEDGEMENT_PATH, 501, 20, 0o400
    )
    if live_acknowledgement_bytes != acknowledgement_bytes:
        fail("native evidence acknowledgement changed during admin cleanup")
    cleanup_bytes, _ = read_exact_file(NATIVE_EVIDENCE_CLEANUP_PATH, 0, 0, 0o444)
    return publish_or_validate_admin_restoration_receipt(
        manifest,
        manifest_bytes,
        root_authority_sha256,
        global_bytes,
        export_bytes,
        acknowledgement_bytes,
        cleanup_bytes,
        claims,
        expected_global,
    )


def installed_state_present(manifest: dict[str, Any]) -> bool:
    installed_manifest = pathlib.Path(
        "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/"
        "candidate-artifact-manifest.v2.json"
    )
    paths = [
        pathlib.Path(entry["intended_path"])
        for entry in manifest["artifacts"]
        if entry["role"] != "capability_manifest"
    ]
    nonretained_directories = [
        pathlib.Path(path) for path, _, _, _ in INSTALL_DIRECTORIES[:-1]
    ]
    return (
        path_present(installed_manifest)
        or any(path_present(path) for path in paths)
        or any(path_present(path) for path in nonretained_directories)
    )


def verify_process_surface() -> str:
    if len(sys.argv) != 1:
        fail("sealed root installer accepts no arguments")
    if os.geteuid() != 0 or os.getegid() != 0 or pwd.getpwuid(0).pw_name != "root":
        fail("sealed root installer requires exact root:wheel identity")
    if pathlib.Path.cwd() != pathlib.Path("/"):
        fail("sealed root installer requires cwd=/")
    if any(key.startswith("SUBSTRATE_") for key in os.environ):
        fail("sealed root installer rejects SUBSTRATE_* environment input")
    authority_keys = sorted(key for key in os.environ if key.startswith("R3_"))
    if authority_keys != [ROOT_INSTALL_AUTHORITY_ENV]:
        fail("sealed root installer received an alternate R3 authority input")
    authority_sha256 = os.environ.get(ROOT_INSTALL_AUTHORITY_ENV)
    if not is_sha256(authority_sha256):
        fail("sealed root installer lacks one exact authority SHA-256")
    for key, value in FIXED_ENV.items():
        if os.environ.get(key) != value:
            fail(f"sealed root installer fixed environment changed: {key}")
    # Apple's /usr/bin/python3 launcher may add deterministic SDK search variables. None are
    # authority and no value is read above. Clear them before any manifest or target inspection.
    os.environ.clear()
    os.environ.update(FIXED_ENV)
    if dict(os.environ) != FIXED_ENV:
        fail("sealed root installer could not normalize its environment")
    os.umask(0o077)
    return authority_sha256


def main() -> tuple[int, bool]:
    root_install_authority_sha256 = verify_process_surface()
    manifest, manifest_bytes = load_manifest()
    validate_root_install_authority(manifest, root_install_authority_sha256)
    terminal_cleanup_present = any(
        path_present(path)
        for path in (
            TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH,
            TERMINAL_ADMIN_CLEANUP_CLAIM_PATH,
            TERMINAL_ADMIN_RESTORATION_RECEIPT_PATH,
        )
    )
    success_cleanup_present = path_present(NATIVE_EVIDENCE_CLEANUP_PATH)
    if terminal_cleanup_present and success_cleanup_present:
        fail("terminal-failure and native-PASS cleanup authorities both exist")
    if terminal_cleanup_present:
        receipt = complete_terminal_admin_cleanup(
            manifest, manifest_bytes, root_install_authority_sha256
        )
        return (
            (
                86
                if receipt["terminal_outcome_kind"]
                in {
                    "root_operation_securityagent_alert",
                    "publisher_securityagent_alert",
                }
                else 78
            ),
            False,
        )
    if success_cleanup_present:
        complete_successful_admin_cleanup(
            manifest, manifest_bytes, root_install_authority_sha256
        )
        return 0, True
    if not path_present(CLAIM_ROOT) and installed_state_present(manifest):
        fail(
            "installed candidate state exists without the retained root-install claims and "
            "typed native cleanup authorization"
        )
    preclaim = create_or_load_preclaim(manifest, manifest_bytes)
    create_or_load_completion(manifest, manifest_bytes, preclaim)
    result = subprocess.run(
        [str(RUNNER_PATH)],
        cwd="/",
        env={},
        stdin=subprocess.DEVNULL,
        stdout=sys.stdout,
        stderr=sys.stderr,
        check=False,
    )
    terminal_cleanup_present = any(
        path_present(path)
        for path in (
            TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH,
            TERMINAL_ADMIN_CLEANUP_CLAIM_PATH,
            TERMINAL_ADMIN_RESTORATION_RECEIPT_PATH,
        )
    )
    if terminal_cleanup_present and path_present(NATIVE_EVIDENCE_CLEANUP_PATH):
        fail("runner published conflicting terminal and native-PASS cleanup authority")
    if terminal_cleanup_present:
        receipt = complete_terminal_admin_cleanup(
            manifest, manifest_bytes, root_install_authority_sha256
        )
        return (
            (
                86
                if receipt["terminal_outcome_kind"]
                in {
                    "root_operation_securityagent_alert",
                    "publisher_securityagent_alert",
                }
                else 78
            ),
            False,
        )
    if path_present(NATIVE_EVIDENCE_CLEANUP_PATH):
        complete_successful_admin_cleanup(
            manifest, manifest_bytes, root_install_authority_sha256
        )
        return result.returncode, result.returncode == 0
    # The same exact reviewed block is the only permitted resume route. No installed byte or
    # root-install claim is removed until the runner has exported acknowledged evidence and
    # published the typed native cleanup authorization.
    if result.returncode == 0:
        fail("runner returned success without typed native cleanup authorization")
    return result.returncode, False


def propagate_route_exit(status: int, success_receipt_validated: bool) -> NoReturn:
    """Exit exactly as the executed route did, emitting PASS only for validated success."""
    if isinstance(status, bool) or not isinstance(status, int):
        fail("executed route returned a non-integer status")
    if status == 0:
        if not success_receipt_validated:
            fail("executed route returned success without a validated success receipt")
        print(SUCCESS_SENTINEL, flush=True)
        raise SystemExit(0)
    if success_receipt_validated:
        fail("validated success receipt accompanied a nonzero route status")
    if status < 0:
        signum = -status
        if signum <= 0 or signum >= signal.NSIG:
            fail("executed route returned an invalid terminating signal")
        if signum not in {signal.SIGKILL, signal.SIGSTOP}:
            signal.signal(signum, signal.SIG_DFL)
        if hasattr(signal, "pthread_sigmask"):
            signal.pthread_sigmask(signal.SIG_UNBLOCK, {signum})
        os.kill(os.getpid(), signum)
        os._exit(128 + signum)
    if status > 255:
        fail("executed route returned an out-of-range exit status")
    raise SystemExit(status)


def run_entrypoint() -> NoReturn:
    try:
        status, success_receipt_validated = main()
        propagate_route_exit(status, success_receipt_validated)
    except Stop as error:
        print(f"R3 sealed root install/rollback stopped: {error}", file=sys.stderr)
        raise SystemExit(78) from error


if __name__ == "__main__":
    run_entrypoint()
