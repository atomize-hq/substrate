#!/usr/bin/python3
"""One-attempt reboot-aware cleanup membrane for the preserved R3 E03 attempt.

The normal root-install rollback remains physical-identity strict.  This separate membrane accepts
only a freshly sealed authority for the one demonstrated reboot boundary, rebinds every surviving
managed object by logical/content/code identity, and then treats the rebound physical identities as
same-boot cleanup authority.  It never runs the experiment or queries Security.framework/Keychain.
"""

from __future__ import annotations

import datetime
import errno
import hashlib
import importlib.util
import json
import os
import pathlib
import plistlib
import pwd
import re
import stat
import subprocess
import sys
from typing import Any, NoReturn


EXPERIMENT_ID = "01a00c2b-4966-7750-b906-ef68495ff1dc"
SCOPES = [
    "01a00c2b-4967-7f62-810d-54a4456ce781",
    "01a00c2b-4968-7cac-9c7c-d8943f94daaa",
]
REPOSITORY = pathlib.Path(
    "/Users/spensermcconnell/.codex/worktrees/r3-macos-finalizer-rcv-stack/substrate"
)
SOURCE_PATH = REPOSITORY / "scripts/mac/r3-macos-finalizer-reboot-cleanup.py"
INSTALLER_SOURCE_PATH = REPOSITORY / "scripts/mac/r3-macos-finalizer-root-install.py"
PRESERVATION_ROOT = pathlib.Path(
    "/Users/spensermcconnell/.codex/preservations/"
    "substrate-r3-e03-denied-sign-20260816T200238Z"
)
PRESERVED_INVENTORY_PATH = PRESERVATION_ROOT / "post-recovery-stop-inventory.txt"
PRESERVED_INVENTORY_SHA256 = (
    "204e68bf567b4275a4749834a15a908aadb43b3a9c27baea277101ff51433717"
)
PRESERVATION_FINISHED_PATH = PRESERVATION_ROOT / "recovery.finished-utc"
PRESERVATION_FINISHED_SHA256 = (
    "34b0c8a85b4c60e062e9f48296dc6adf2ec0ec966f3e1241c05fed50a74fb7b6"
)
PRESERVATION_FINISHED_UTC = "2026-08-16T20:23:57Z"
OBSERVED_REBOOT_UTC = "2026-08-16T21:47:52Z"
AUTHORITY_PATH = PRESERVATION_ROOT / "reboot-aware-cleanup-authority.v1.json"
RESTORATION_PATH = PRESERVATION_ROOT / "reboot-aware-cleanup-restoration.v1.json"
AUTHORITY_ENV = "R3_REBOOT_CLEANUP_AUTHORITY_SHA256"
FIXED_ENV = {
    "PATH": "/usr/bin:/bin:/usr/sbin:/sbin",
    "LANG": "C",
    "LC_ALL": "C",
    "TZ": "UTC",
}
MAX_AUTHORITY_BYTES = 4 * 1024 * 1024
MAX_MANAGED_FILE_BYTES = 16 * 1024 * 1024
EMPTY_SHA256 = hashlib.sha256(b"").hexdigest()
ROOT_INSTALL_CLAIMS_SHA256 = (
    "d26fcb5bbe87a0bb17d5fe8169202db0ee66b4fcc35572fd7bd1d8170e6a8b68"
)
ROOT_INSTALL_PRECLAIM_SHA256 = (
    "5125a93448e8e11eff51ecc069abea5f3476b41f9c1d0772abfe4ad64e7e0bfe"
)
ROOT_INSTALL_COMPLETION_SHA256 = (
    "c16b491e3dd924de72ed106644b4395dab5330eaff5673cf835c4792f5fb4c19"
)
LAUNCHD_PLIST_SHA256 = (
    "ba462aef89cbffb4054506e9e6d21e925f298338fe5519d45024c97714e6a0ae"
)

RUNNER_CHILDREN = (
    "activation-journal-root-lock-observation.v2.json",
    "creator-arm-01.native-receipt.v2.json",
    "creator-arm-01.observed.cursor.v2.json",
    "creator-arm-01.prepared.cursor.v2.json",
    "creator-arm-01.process-attestation.v2.json",
    "creator-arm-01.securityagent-observation.v2.json",
    "creator-arm-01.shared-receipt.v2.json",
    "creator-arm-01.v2.json",
    "creator-arm-02.native-receipt.v2.json",
    "creator-arm-02.observed.cursor.v2.json",
    "creator-arm-02.prepared.cursor.v2.json",
    "creator-arm-02.process-attestation.v2.json",
    "creator-arm-02.securityagent-observation.v2.json",
    "creator-arm-02.shared-receipt.v2.json",
    "creator-arm-02.v2.json",
    "creator-arm-03.prepared.cursor.v2.json",
    "creator-arm-03.process-attestation.v2.json",
    "creator-arm-03.v2.json",
    "creator-emergency-rollback.process-attestation."
    "37ee4c12ac2e5954e769436ac22d19502afbaa74efa4b52772bcc308242fa3ec.v2.json",
    "creator-emergency-rollback.receipt.v2.json",
    "creator-terminal-cleanup.cursor.v2.json",
    "global-nonce-absence-baseline.ui.observation.v2.json",
    "global-pre-effect-disposable-baseline.v2.json",
    "global-pre-effect-packet.v2.json",
    "legacy-access-nonmatch-owner-probe.v3.json",
    "legacy-access-nonmatch-owner-readback.v2.json",
    "runner-first-security-interaction-denial-01.ui.observation.v2.json",
    "runner-self-observation.v2.json",
    "terminal-finalizer-failure-cleanup.ui.observation.v2.json",
    "terminal-finalizer-failure-cleanup.v2.json",
)
CREATOR_CHILDREN = (
    "creator-emergency-rollback.v2.json",
    "creator-route.v2",
)
PROTOCOL_FILES = (
    (
        "prepared_input_sha256",
        "disposable-publisher-prepared-input.v2.json",
    ),
    ("candidate_identity_packet_sha256", "candidate-identity-packet.v2.json"),
    ("peer_control_identity_packet_sha256", "peer-control-identity-packet.v2.json"),
)


class Stop(RuntimeError):
    pass


def fail(message: str) -> NoReturn:
    raise Stop(message)


def canonical(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode()


def sha_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def document_sha256(value: Any) -> str:
    return sha_bytes(canonical(value))


def is_sha256(value: Any) -> bool:
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None


def is_git_id(value: Any) -> bool:
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{40}", value) is not None


def parse_strict_json(data: bytes, label: str) -> Any:
    def reject_duplicate(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        value: dict[str, Any] = {}
        for key, item in pairs:
            if key in value:
                fail(f"{label} duplicated JSON field {key}")
            value[key] = item
        return value

    def reject_noninteger(_: str) -> NoReturn:
        fail(f"{label} acquired a non-integer JSON number")

    try:
        return json.loads(
            data,
            object_pairs_hook=reject_duplicate,
            parse_float=reject_noninteger,
            parse_constant=reject_noninteger,
        )
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise Stop(f"{label} is not strict JSON") from error


def parse_utc(value: str) -> datetime.datetime:
    try:
        parsed = datetime.datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ")
    except ValueError as error:
        raise Stop(f"noncanonical UTC timestamp: {value}") from error
    return parsed.replace(tzinfo=datetime.timezone.utc)


def validate_reboot_timeline(preservation_utc: str, boot_utc: str) -> None:
    if preservation_utc != PRESERVATION_FINISHED_UTC or boot_utc != OBSERVED_REBOOT_UTC:
        fail("reboot recovery changed its exact demonstrated timeline")
    if parse_utc(preservation_utc) >= parse_utc(boot_utc):
        fail("preserved inventory does not predate the observed reboot")


def load_installer(expected_sha256: str):
    if not is_sha256(expected_sha256):
        fail("installer source digest is noncanonical")
    before = os.lstat(INSTALLER_SOURCE_PATH)
    if (
        not stat.S_ISREG(before.st_mode)
        or before.st_uid != 501
        or before.st_gid != 20
        or stat.S_IMODE(before.st_mode) != 0o644
        or before.st_nlink != 1
        or before.st_size < 1
        or before.st_size > MAX_AUTHORITY_BYTES
    ):
        fail("installer source changed its exact repository-file posture")
    source = INSTALLER_SOURCE_PATH.read_bytes()
    after = os.lstat(INSTALLER_SOURCE_PATH)
    if sha_bytes(source) != expected_sha256 or stat_tuple(before) != stat_tuple(after):
        fail("installer source changed across its sealed read")
    spec = importlib.util.spec_from_file_location(
        "r3_root_install_membrane", INSTALLER_SOURCE_PATH
    )
    if spec is None or spec.loader is None:
        fail("could not load the exact root-install membrane")
    module = importlib.util.module_from_spec(spec)
    prior_bytecode_setting = sys.dont_write_bytecode
    sys.dont_write_bytecode = True
    try:
        spec.loader.exec_module(module)
    finally:
        sys.dont_write_bytecode = prior_bytecode_setting
    final = os.lstat(INSTALLER_SOURCE_PATH)
    if stat_tuple(after) != stat_tuple(final):
        fail("installer source changed while loading its cleanup primitives")
    return module


def stat_tuple(value: os.stat_result) -> tuple[int, ...]:
    return (
        value.st_dev,
        value.st_ino,
        value.st_uid,
        value.st_gid,
        value.st_mode,
        value.st_nlink,
        value.st_size,
        value.st_mtime_ns,
        value.st_ctime_ns,
    )


def root_owned_chain(path: pathlib.Path, *, permit_writable_leaf: bool = False) -> None:
    if not path.is_absolute():
        fail(f"managed parent is not absolute: {path}")
    descriptor = os.open("/", os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    current = pathlib.Path("/")
    try:
        for index, component in enumerate(path.parts[1:]):
            next_descriptor = os.open(
                component,
                os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW,
                dir_fd=descriptor,
            )
            observed = os.fstat(next_descriptor)
            current /= component
            if not stat.S_ISDIR(observed.st_mode) or observed.st_uid != 0:
                os.close(next_descriptor)
                fail(f"managed parent is not a root-owned directory: {current}")
            terminal = index == len(path.parts[1:]) - 1
            if observed.st_mode & 0o022 and not (terminal and permit_writable_leaf):
                os.close(next_descriptor)
                fail(f"managed parent acquired group/world write access: {current}")
            os.close(descriptor)
            descriptor = next_descriptor
    finally:
        os.close(descriptor)


def physical(path: pathlib.Path, observed: os.stat_result) -> dict[str, Any]:
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


def stable_directory_identity(value: dict[str, Any]) -> dict[str, Any]:
    return {
        key: value[key] for key in ("path", "device", "inode", "uid", "gid", "mode")
    }


def read_managed_file(
    membrane: Any,
    path: pathlib.Path,
    uid: int,
    gid: int,
    mode: int,
    expected_size: int,
    expected_sha256: str,
) -> tuple[bytes, dict[str, Any]]:
    root_owned_chain(path.parent)
    data, observed = membrane.read_exact_sized_file(
        path, uid, gid, mode, expected_size, MAX_MANAGED_FILE_BYTES
    )
    if sha_bytes(data) != expected_sha256:
        fail(f"managed file changed its exact content digest: {path}")
    return data, physical(path, observed)


def code_identity(entry: dict[str, Any]) -> dict[str, Any] | None:
    if entry["signing_identifier"] is None:
        return None
    return {
        "signing_identifier": entry["signing_identifier"],
        "designated_requirement": entry["designated_requirement"],
        "cdhash": entry["cdhash"],
        "code_flags": entry["code_flags"],
        "team_id": entry["team_id"],
        "entitlements_size": entry["entitlements_size"],
        "entitlements_sha256": entry["entitlements_sha256"],
    }


def artifact_record(membrane: Any, entry: dict[str, Any]) -> dict[str, Any]:
    path = pathlib.Path(entry["intended_path"])
    data, identity = read_managed_file(
        membrane,
        path,
        entry["uid"],
        entry["gid"],
        entry["mode"],
        entry["size"],
        entry["sha256"],
    )
    del data
    if entry["signing_identifier"] is not None:
        membrane.verify_code_identity(path, entry)
    return {
        "role": entry["role"],
        "path": str(path),
        "file_type": "regular",
        "uid": entry["uid"],
        "gid": entry["gid"],
        "mode": entry["mode"],
        "link_count": 1,
        "size": entry["size"],
        "sha256": entry["sha256"],
        "code_identity": code_identity(entry),
        "post_reboot_physical_identity": identity,
    }


def directory_record(
    membrane: Any,
    path: pathlib.Path,
    uid: int,
    gid: int,
    mode: int,
    *,
    root_owned_parent: bool = True,
) -> dict[str, Any]:
    if root_owned_parent:
        root_owned_chain(path.parent)
    else:
        descriptor = membrane.open_directory_chain(path.parent)
        os.close(descriptor)
    identity = membrane.directory_identity(path, uid, gid, mode)
    observed = os.lstat(path)
    return {
        "path": str(path),
        "file_type": "directory",
        "uid": uid,
        "gid": gid,
        "mode": mode,
        "link_count": observed.st_nlink,
        "post_reboot_physical_identity": identity,
    }


def absent(path: pathlib.Path) -> dict[str, Any]:
    try:
        os.lstat(path)
    except FileNotFoundError:
        return {"path": str(path), "lstat_return": -1, "raw_errno": errno.ENOENT}
    fail(f"reboot recovery required an exact managed absence: {path}")


def raw_stream(value: bytes) -> dict[str, Any]:
    import base64

    return {
        "raw_base64url": base64.urlsafe_b64encode(value).rstrip(b"=").decode(),
        "raw_byte_length": len(value),
        "raw_sha256": sha_bytes(value),
    }


def run(arguments: list[str]) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        arguments,
        cwd="/",
        env=FIXED_ENV,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def exact_git_identity(cleanup_commit: str) -> dict[str, str]:
    if not is_git_id(cleanup_commit):
        fail("cleanup commit is noncanonical")
    prefix = [
        "/usr/bin/git",
        "-c",
        f"safe.directory={REPOSITORY}",
        "-C",
        str(REPOSITORY),
    ]
    head = run([*prefix, "rev-parse", "HEAD"])
    tree = run([*prefix, "rev-parse", "HEAD^{tree}"])
    status = run([*prefix, "status", "--porcelain=v2", "-z"])
    if (
        head.returncode != 0
        or tree.returncode != 0
        or status.returncode != 0
        or head.stderr
        or tree.stderr
        or head.stdout != f"{cleanup_commit}\n".encode()
        or status.stdout
        or status.stderr
    ):
        fail("cleanup source repository is not the exact clean committed authority")
    tree_id = tree.stdout.decode().strip()
    if not is_git_id(tree_id):
        fail("cleanup tree is noncanonical")
    return {"cleanup_commit": cleanup_commit, "cleanup_tree": tree_id}


def exact_source_record(
    cleanup_commit: str, source_sha256: str, installer_sha256: str
) -> dict[str, Any]:
    git = exact_git_identity(cleanup_commit)
    values = []
    for path, expected in (
        (SOURCE_PATH, source_sha256),
        (INSTALLER_SOURCE_PATH, installer_sha256),
    ):
        observed = os.lstat(path)
        data = path.read_bytes()
        after = os.lstat(path)
        if (
            not is_sha256(expected)
            or not stat.S_ISREG(observed.st_mode)
            or observed.st_uid != 501
            or observed.st_gid != 20
            or stat.S_IMODE(observed.st_mode) != 0o644
            or observed.st_nlink != 1
            or sha_bytes(data) != expected
            or stat_tuple(observed) != stat_tuple(after)
        ):
            fail(f"cleanup implementation source changed: {path}")
        values.append(
            {
                "path": str(path),
                "sha256": expected,
                "byte_length": len(data),
                "physical_identity": physical(path, observed),
            }
        )
    return {**git, "implementation_sources": values}


def observed_boot() -> dict[str, Any]:
    output = run(["/usr/sbin/sysctl", "-n", "kern.boottime"])
    if output.returncode != 0 or output.stderr or len(output.stdout) > 1024:
        fail("could not establish the exact reboot boundary")
    match = re.fullmatch(rb"\{ sec = ([0-9]+), usec = ([0-9]+) \} .+\n", output.stdout)
    if match is None:
        fail("kern.boottime changed its exact raw shape")
    seconds = int(match.group(1))
    microseconds = int(match.group(2))
    if not 0 <= microseconds < 1_000_000:
        fail("kern.boottime microseconds are invalid")
    boot_utc = datetime.datetime.fromtimestamp(
        seconds, tz=datetime.timezone.utc
    ).strftime("%Y-%m-%dT%H:%M:%SZ")
    validate_reboot_timeline(PRESERVATION_FINISHED_UTC, boot_utc)
    return {
        "boot_seconds": seconds,
        "boot_microseconds": microseconds,
        "boot_utc": boot_utc,
        "raw_sysctl": raw_stream(output.stdout),
    }


def preservation_record(membrane: Any) -> dict[str, Any]:
    inventory, _ = membrane.read_bounded_exact_file(
        PRESERVED_INVENTORY_PATH, 501, 20, 0o644, MAX_AUTHORITY_BYTES
    )
    finished, _ = membrane.read_bounded_exact_file(
        PRESERVATION_FINISHED_PATH, 501, 20, 0o644, 128
    )
    if (
        sha_bytes(inventory) != PRESERVED_INVENTORY_SHA256
        or sha_bytes(finished) != PRESERVATION_FINISHED_SHA256
        or finished != f"{PRESERVATION_FINISHED_UTC}\n".encode()
    ):
        fail("preserved pre-reboot evidence changed its exact durable binding")
    return {
        "inventory_path": str(PRESERVED_INVENTORY_PATH),
        "inventory_sha256": PRESERVED_INVENTORY_SHA256,
        "preservation_finished_path": str(PRESERVATION_FINISHED_PATH),
        "preservation_finished_sha256": PRESERVATION_FINISHED_SHA256,
        "preservation_finished_utc": PRESERVATION_FINISHED_UTC,
    }


def inventory_root(
    membrane: Any,
    root: pathlib.Path,
    expected_names: tuple[str, ...],
    file_mode: int,
) -> dict[str, Any]:
    root_owned_chain(root.parent)
    root_identity = membrane.directory_identity(root, 0, 0, 0o700)
    descriptor = membrane.open_directory_chain(root)
    try:
        before = os.fstat(descriptor)
        names = tuple(sorted(os.listdir(descriptor)))
        if names != expected_names:
            fail(
                f"managed root has unexplained live-state differences: {root}: {names}"
            )
        children = []
        for name in names:
            path = root / name
            observed = os.stat(name, dir_fd=descriptor, follow_symlinks=False)
            if observed.st_size < 1 or observed.st_size > MAX_AUTHORITY_BYTES:
                fail(f"managed child exceeds its exact recovery bound: {path}")
            data, held = membrane.read_exact_sized_file(
                path, 0, 0, file_mode, observed.st_size, MAX_AUTHORITY_BYTES
            )
            children.append(
                {
                    "name": name,
                    "path": str(path),
                    "file_type": "regular",
                    "uid": 0,
                    "gid": 0,
                    "mode": file_mode,
                    "link_count": 1,
                    "size": len(data),
                    "sha256": sha_bytes(data),
                    "physical_identity": physical(path, held),
                }
            )
        after = os.fstat(descriptor)
        pathname = os.lstat(root)
        if (
            membrane.same_stat(before, after) is False
            or membrane.same_stat(after, pathname) is False
        ):
            fail(f"managed root changed during exact inventory: {root}")
    finally:
        os.close(descriptor)
    return {
        "path": str(root),
        "file_type": "directory",
        "uid": 0,
        "gid": 0,
        "mode": 0o700,
        "link_count": pathname.st_nlink,
        "physical_identity": root_identity,
        "children": children,
        "child_set_sha256": document_sha256(children),
    }


def validate_emergency_receipts(
    membrane: Any, runner: dict[str, Any], creator: dict[str, Any]
) -> dict[str, Any]:
    runner_by_name = {entry["name"]: entry for entry in runner["children"]}
    creator_by_name = {entry["name"]: entry for entry in creator["children"]}
    runner_path = pathlib.Path(
        runner_by_name["creator-emergency-rollback.receipt.v2.json"]["path"]
    )
    creator_path = pathlib.Path(
        creator_by_name["creator-emergency-rollback.v2.json"]["path"]
    )
    runner_value = membrane.parse_canonical(
        runner_path.read_bytes(), "emergency signer-deletion receipt"
    )
    creator_value = parse_strict_json(
        creator_path.read_bytes(), "creator emergency rollback state"
    )
    cursor_path = pathlib.Path(
        runner_by_name["creator-terminal-cleanup.cursor.v2.json"]["path"]
    )
    cursor = membrane.parse_canonical(
        cursor_path.read_bytes(), "creator cleanup cursor"
    )
    finalizer_path = pathlib.Path(
        runner_by_name["terminal-finalizer-failure-cleanup.v2.json"]["path"]
    )
    finalizer = membrane.parse_canonical(
        finalizer_path.read_bytes(), "terminal finalizer cleanup receipt"
    )
    if (
        runner_by_name["creator-emergency-rollback.receipt.v2.json"]["sha256"]
        != "ac6037f43205602cd8ccf465283ef680ef3b3fee8c8abf28bd96413adb47a49a"
        or runner_value
        != {
            "schema_owner": "substrate.r3-macos-signer-acl.creator-emergency-rollback-receipt",
            "schema_version": 2,
            "repetition": "first",
            "failure_observation_sha256": "b4560a05658b1c7e287485f0f6f06a4765b48865d285cbfdb600480155b71ae0",
            "exact_delete": {
                "classification": "deleted_and_absent",
                "present_after": False,
                "raw_os_status": 0,
            },
            "exact_identity_absent": True,
        }
        or creator_value.get("receipt_sha256")
        != runner_by_name["creator-emergency-rollback.receipt.v2.json"]["sha256"]
        or creator_value.get("phase") != "observed"
        or cursor.get("creator_rollback_receipt_sha256")
        != runner_by_name["creator-emergency-rollback.receipt.v2.json"]["sha256"]
        or cursor.get("cleanup_may_begin") is not True
        or cursor.get("terminal_no_normal_resume") is not True
        or finalizer.get("journal_root_absent") is not True
        or finalizer.get("endpoint_absent") is not True
    ):
        fail("emergency signer deletion or terminal cleanup authority changed")
    return {
        "signer_deletion_receipt_sha256": runner_by_name[
            "creator-emergency-rollback.receipt.v2.json"
        ]["sha256"],
        "creator_emergency_state_sha256": creator_by_name[
            "creator-emergency-rollback.v2.json"
        ]["sha256"],
        "creator_cleanup_cursor_sha256": runner_by_name[
            "creator-terminal-cleanup.cursor.v2.json"
        ]["sha256"],
        "terminal_finalizer_cleanup_sha256": runner_by_name[
            "terminal-finalizer-failure-cleanup.v2.json"
        ]["sha256"],
        "exact_identity_absent": True,
        "journal_root_was_exactly_removed": True,
    }


def launchd_rehydration(membrane: Any, manifest: dict[str, Any]) -> dict[str, Any]:
    entries = [
        entry for entry in manifest["artifacts"] if entry["role"] == "launchd_plist"
    ]
    if len(entries) != 1:
        fail("manifest changed its exact launchd-plist role")
    entry = entries[0]
    path = pathlib.Path(entry["intended_path"])
    plist_bytes, identity = read_managed_file(
        membrane, path, 0, 0, 0o644, entry["size"], entry["sha256"]
    )
    try:
        plist = plistlib.loads(plist_bytes)
    except plistlib.InvalidFileException as error:
        raise Stop("installed launchd plist is invalid") from error
    expected_plist = {
        "Label": membrane.FINALIZER_LABEL,
        "Program": str(pathlib.Path(membrane.ARTIFACT_SPECS[0][2])),
        "UserName": "root",
        "StandardInPath": "/dev/null",
        "WorkingDirectory": "/",
        "Sockets": {
            "Listener": {
                "SockPathName": str(membrane.ENDPOINT_PATH),
                "SockPathOwner": 0,
                "SockPathGroup": 20,
                "SockPathMode": 0o660,
            }
        },
    }
    if plist != expected_plist:
        fail("installed launchd plist changed its exact service/socket definition")
    output = run(["/bin/launchctl", "print", f"system/{membrane.FINALIZER_LABEL}"])
    text = output.stdout.decode(errors="strict")
    required_fragments = (
        f"system/{membrane.FINALIZER_LABEL} = {{",
        "active count = 0",
        f"path = {path}",
        "type = LaunchDaemon",
        "state = not running",
        f"program = {expected_plist['Program']}",
        "runs = 0",
        "last exit code = (never exited)",
        f"path = {membrane.ENDPOINT_PATH}",
        "mode = 660",
        "active = 0",
        "passive = 1",
    )
    if (
        output.returncode != 0
        or output.stderr
        or any(value not in text for value in required_fragments)
    ):
        fail("launchd state is not exact reboot rehydration from the retained plist")
    socket = os.lstat(membrane.ENDPOINT_PATH)
    if (
        not stat.S_ISSOCK(socket.st_mode)
        or socket.st_uid != 0
        or socket.st_gid != 20
        or stat.S_IMODE(socket.st_mode) != 0o660
        or socket.st_nlink != 1
        or socket.st_size != 0
    ):
        fail("rehydrated finalizer socket changed its exact launchd posture")
    return {
        "classification": "launchd_rehydrated_socket_from_exact_installed_plist_after_reboot",
        "label": membrane.FINALIZER_LABEL,
        "plist_path": str(path),
        "plist_sha256": entry["sha256"],
        "plist_physical_identity": identity,
        "launchctl_print_exit_status": output.returncode,
        "launchctl_print_stdout": raw_stream(output.stdout),
        "launchctl_print_stderr": raw_stream(output.stderr),
        "socket_path": str(membrane.ENDPOINT_PATH),
        "socket_physical_identity": physical(membrane.ENDPOINT_PATH, socket),
    }


def process_absence(membrane: Any, manifest: dict[str, Any]) -> dict[str, Any]:
    targets = sorted(
        entry["intended_path"]
        for entry in manifest["artifacts"]
        if entry["signing_identifier"] is not None
    )
    records = membrane.process_snapshot()
    matching = [record for record in records if record.get("path") in targets]
    if matching:
        fail(
            f"a conflicting E03/finalizer/coordinator/runner process exists: {matching}"
        )
    return {"target_paths": targets, "matching_processes": []}


def validate_fixture_projection(value: Any) -> dict[str, Any]:
    expected_fields = {
        "experiment_id",
        "repetition_scopes",
        "ephemeral_claims_absent",
        "journal_root_absent",
        "cleanup_authority_source",
        "durable_claim_bindings",
        "preservation_finished_utc",
        "observed_reboot_utc",
        "installed_artifacts",
        "frozen_artifact_bindings",
        "launchd_rehydration",
        "conflicting_processes",
        "emergency_signer_deletion_receipt_sha256",
        "unexplained_live_state_differences",
    }
    if not isinstance(value, dict) or set(value) != expected_fields:
        fail("reboot recovery projection changed its closed shape")
    validate_reboot_timeline(
        value.get("preservation_finished_utc"), value.get("observed_reboot_utc")
    )
    claims = value.get("durable_claim_bindings")
    launchd = value.get("launchd_rehydration")
    artifacts = value.get("installed_artifacts")
    if (
        value.get("experiment_id") != EXPERIMENT_ID
        or value.get("repetition_scopes") != SCOPES
        or value.get("ephemeral_claims_absent") is not True
        or value.get("journal_root_absent") is not True
        or value.get("cleanup_authority_source")
        != "durable_canonical_global_pre_effect_claim_copies"
        or not isinstance(claims, dict)
        or set(claims)
        != {
            "root_install_claims_binding_sha256",
            "root_install_preclaim_sha256",
            "root_install_completion_sha256",
        }
        or not all(is_sha256(item) for item in claims.values())
        or claims.get("root_install_claims_binding_sha256")
        != ROOT_INSTALL_CLAIMS_SHA256
        or claims.get("root_install_preclaim_sha256") != ROOT_INSTALL_PRECLAIM_SHA256
        or claims.get("root_install_completion_sha256")
        != ROOT_INSTALL_COMPLETION_SHA256
        or not isinstance(artifacts, list)
        or not artifacts
        or artifacts != value.get("frozen_artifact_bindings")
        or value.get("conflicting_processes") != []
        or value.get("emergency_signer_deletion_receipt_sha256")
        != "ac6037f43205602cd8ccf465283ef680ef3b3fee8c8abf28bd96413adb47a49a"
        or value.get("unexplained_live_state_differences") != []
        or not isinstance(launchd, dict)
        or launchd.get("classification")
        != "launchd_rehydrated_socket_from_exact_installed_plist_after_reboot"
        or launchd.get("label")
        != "com.atomize.substrate.r3-macos-evidence-finalizer.v2"
        or not is_sha256(launchd.get("plist_sha256"))
        or launchd.get("plist_sha256") != LAUNCHD_PLIST_SHA256
        or launchd.get("plist_path") != "/Library/LaunchDaemons/"
        "com.atomize.substrate.r3-macos-evidence-finalizer.v2.plist"
        or launchd.get("socket_path") != "/private/var/run/"
        "com.atomize.substrate.r3-macos-evidence-finalizer.v2.sock"
        or launchd.get("socket_type") != "socket"
        or launchd.get("socket_uid") != 0
        or launchd.get("socket_gid") != 20
        or launchd.get("socket_mode") != 0o660
        or launchd.get("socket_link_count") != 1
    ):
        fail("reboot recovery projection changed an exact authority predicate")
    roles: set[str] = set()
    for artifact in artifacts:
        if (
            not isinstance(artifact, dict)
            or set(artifact)
            != {
                "role",
                "path",
                "file_type",
                "uid",
                "gid",
                "mode",
                "link_count",
                "size",
                "sha256",
                "code_identity",
            }
            or artifact.get("role") in roles
            or artifact.get("file_type") != "regular"
            or artifact.get("uid") != 0
            or artifact.get("gid") != 0
            or artifact.get("mode") not in (0o555, 0o644)
            or artifact.get("link_count") != 1
            or not isinstance(artifact.get("size"), int)
            or not 0 < artifact["size"] <= MAX_MANAGED_FILE_BYTES
            or not is_sha256(artifact.get("sha256"))
        ):
            fail("post-reboot installed-artifact logical rebind changed")
        identity = artifact.get("code_identity")
        if artifact["mode"] == 0o555:
            if (
                not isinstance(identity, dict)
                or not isinstance(identity.get("signing_identifier"), str)
                or not is_sha256(identity.get("identity_sha256"))
            ):
                fail("post-reboot executable code identity changed")
        elif identity is not None:
            fail("unsigned launchd artifact acquired code identity")
        roles.add(artifact["role"])
    return value


def fixture_projection(
    claims: dict[str, Any],
    artifacts: list[dict[str, Any]],
    launchd: dict[str, Any],
    manifest: dict[str, Any],
) -> dict[str, Any]:
    projected_artifacts = []
    for artifact in artifacts:
        identity = artifact["code_identity"]
        projected_artifacts.append(
            {
                key: artifact[key]
                for key in (
                    "role",
                    "path",
                    "file_type",
                    "uid",
                    "gid",
                    "mode",
                    "link_count",
                    "size",
                    "sha256",
                )
            }
            | {
                "code_identity": (
                    None
                    if identity is None
                    else {
                        "signing_identifier": identity["signing_identifier"],
                        "identity_sha256": document_sha256(identity),
                    }
                )
            }
        )
    frozen_artifacts = []
    by_role = {
        entry["role"]: entry
        for entry in manifest["artifacts"]
        if entry["role"] != "capability_manifest"
    }
    for observed in projected_artifacts:
        entry = by_role[observed["role"]]
        identity = code_identity(entry)
        frozen_artifacts.append(
            {
                "role": entry["role"],
                "path": entry["intended_path"],
                "file_type": "regular",
                "uid": entry["uid"],
                "gid": entry["gid"],
                "mode": entry["mode"],
                "link_count": 1,
                "size": entry["size"],
                "sha256": entry["sha256"],
                "code_identity": (
                    None
                    if identity is None
                    else {
                        "signing_identifier": identity["signing_identifier"],
                        "identity_sha256": document_sha256(identity),
                    }
                ),
            }
        )
    socket = launchd["socket_physical_identity"]
    return validate_fixture_projection(
        {
            "experiment_id": EXPERIMENT_ID,
            "repetition_scopes": SCOPES,
            "ephemeral_claims_absent": True,
            "journal_root_absent": True,
            "cleanup_authority_source": (
                "durable_canonical_global_pre_effect_claim_copies"
            ),
            "durable_claim_bindings": {
                "root_install_claims_binding_sha256": document_sha256(claims),
                "root_install_preclaim_sha256": document_sha256(claims["preclaim"]),
                "root_install_completion_sha256": document_sha256(claims["completion"]),
            },
            "preservation_finished_utc": PRESERVATION_FINISHED_UTC,
            "observed_reboot_utc": OBSERVED_REBOOT_UTC,
            "installed_artifacts": projected_artifacts,
            "frozen_artifact_bindings": frozen_artifacts,
            "launchd_rehydration": {
                "classification": launchd["classification"],
                "label": launchd["label"],
                "plist_sha256": launchd["plist_sha256"],
                "plist_path": launchd["plist_path"],
                "socket_path": launchd["socket_path"],
                "socket_type": "socket",
                "socket_uid": socket["uid"],
                "socket_gid": socket["gid"],
                "socket_mode": stat.S_IMODE(socket["mode"]),
                "socket_link_count": socket["link_count"],
            },
            "conflicting_processes": [],
            "emergency_signer_deletion_receipt_sha256": (
                "ac6037f43205602cd8ccf465283ef680ef3b3fee8c8abf28bd96413adb47a49a"
            ),
            "unexplained_live_state_differences": [],
        }
    )


def observe_state(
    cleanup_commit: str, source_sha256: str, installer_sha256: str
) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any], Any]:
    membrane = load_installer(installer_sha256)
    source = exact_source_record(cleanup_commit, source_sha256, installer_sha256)
    preservation = preservation_record(membrane)
    reboot = observed_boot()
    manifest, manifest_bytes = membrane.load_manifest()
    (
        external_global_bytes,
        global_packet,
        claims,
        _,
        _,
        installed_manifest_binding,
    ) = membrane.load_global_cleanup_authority(manifest, manifest_bytes)
    claims = membrane.validate_archived_root_install_claims(
        claims, manifest, manifest_bytes
    )
    runner_global_path = membrane.RUNNER_ROOT / "global-pre-effect-packet.v2.json"
    runner_global_bytes, _ = membrane.read_bounded_exact_file(
        runner_global_path, 0, 0, 0o600, MAX_AUTHORITY_BYTES
    )
    if runner_global_bytes != external_global_bytes:
        fail("durable runner/global pre-effect packets changed their exact equality")

    claim_absences = [
        absent(membrane.CLAIM_ROOT),
        absent(membrane.PRECLAIM_PATH),
        absent(membrane.COMPLETION_PATH),
    ]
    protocol_absences = [
        absent(membrane.JOURNAL_ROOT),
        absent(membrane.PUBLISHER_ROOT),
        absent(membrane.TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_PATH),
        absent(membrane.TERMINAL_ADMIN_CLEANUP_CLAIM_PATH),
        absent(membrane.TERMINAL_ADMIN_RESTORATION_RECEIPT_PATH),
    ]
    artifacts = [
        artifact_record(membrane, entry)
        for entry in manifest["artifacts"]
        if entry["role"] != "capability_manifest"
    ]
    installed_manifest_data, installed_manifest_physical = read_managed_file(
        membrane,
        membrane.INSTALLED_CANDIDATE_MANIFEST_PATH,
        0,
        0,
        0o400,
        len(manifest_bytes),
        sha_bytes(manifest_bytes),
    )
    if installed_manifest_data != manifest_bytes:
        fail("installed manifest differs from the exact frozen manifest bytes")
    installed_manifest = {
        "path": str(membrane.INSTALLED_CANDIDATE_MANIFEST_PATH),
        "file_type": "regular",
        "uid": 0,
        "gid": 0,
        "mode": 0o400,
        "link_count": 1,
        "size": len(manifest_bytes),
        "sha256": sha_bytes(manifest_bytes),
        "post_reboot_physical_identity": installed_manifest_physical,
    }
    directories = []
    for path, uid, gid, mode in membrane.INSTALL_DIRECTORIES[:-1]:
        candidate = pathlib.Path(path)
        if candidate == membrane.JOURNAL_ROOT:
            continue
        directories.append(directory_record(membrane, candidate, uid, gid, mode))
    global_exchange = directory_record(
        membrane,
        membrane.GLOBAL_EXCHANGE,
        501,
        0,
        0o700,
        root_owned_parent=False,
    )
    global_names = sorted(os.listdir(membrane.GLOBAL_EXCHANGE))
    if global_names != [membrane.GLOBAL_PRE_EFFECT_PATH.name]:
        fail(f"global exchange has unexplained live state: {global_names}")

    runner = inventory_root(membrane, membrane.RUNNER_ROOT, RUNNER_CHILDREN, 0o600)
    creator = inventory_root(membrane, membrane.CREATOR_ROOT, CREATOR_CHILDREN, 0o600)
    emergency = validate_emergency_receipts(membrane, runner, creator)
    launchd = launchd_rehydration(membrane, manifest)
    processes = process_absence(membrane, manifest)

    protocol_files = []
    for digest_field, name in PROTOCOL_FILES:
        path = membrane.INBOX_ROOT.parent / name
        observed = os.lstat(path)
        data, identity = read_managed_file(
            membrane,
            path,
            0,
            0,
            0o444,
            observed.st_size,
            global_packet[digest_field],
        )
        membrane.parse_canonical(data, f"reboot cleanup protocol file {name}")
        protocol_files.append(
            {
                "name": name,
                "path": str(path),
                "file_type": "regular",
                "uid": 0,
                "gid": 0,
                "mode": 0o444,
                "link_count": 1,
                "size": len(data),
                "sha256": sha_bytes(data),
                "physical_identity": identity,
            }
        )
    if list(os.scandir(membrane.INBOX_ROOT)):
        fail("coordinator inbox acquired unexplained live state")
    expected_v2_names = sorted(
        [membrane.INSTALLED_CANDIDATE_MANIFEST_PATH.name, membrane.INBOX_ROOT.name]
        + [entry["name"] for entry in protocol_files]
        + [
            pathlib.Path(entry["path"]).name
            for entry in artifacts
            if pathlib.Path(entry["path"]).parent == membrane.INBOX_ROOT.parent
        ]
    )
    actual_v2_names = sorted(os.listdir(membrane.INBOX_ROOT.parent))
    if actual_v2_names != expected_v2_names:
        fail(
            f"installed support directory has unexplained live state: {actual_v2_names}"
        )

    external_global_data, external_global_observed = membrane.read_bounded_exact_file(
        membrane.GLOBAL_PRE_EFFECT_PATH, 0, 0, 0o444, MAX_AUTHORITY_BYTES
    )
    if external_global_data != external_global_bytes:
        fail("external global pre-effect packet changed during logical rebind")
    external_global = {
        "path": str(membrane.GLOBAL_PRE_EFFECT_PATH),
        "file_type": "regular",
        "uid": 0,
        "gid": 0,
        "mode": 0o444,
        "link_count": 1,
        "size": len(external_global_data),
        "sha256": sha_bytes(external_global_data),
        "physical_identity": physical(
            membrane.GLOBAL_PRE_EFFECT_PATH, external_global_observed
        ),
    }
    fixture = fixture_projection(claims, artifacts, launchd, manifest)
    state = {
        "source_identity": source,
        "preserved_pre_reboot_evidence": preservation,
        "observed_reboot": reboot,
        "ephemeral_claim_absences": claim_absences,
        "protocol_absences": protocol_absences,
        "external_global_pre_effect": external_global,
        "runner_global_pre_effect_sha256": sha_bytes(runner_global_bytes),
        "installed_artifacts": artifacts,
        "installed_manifest": installed_manifest,
        "installed_directories": directories,
        "global_exchange": global_exchange,
        "runner_root": runner,
        "creator_root": creator,
        "protocol_files": protocol_files,
        "launchd_rehydration": launchd,
        "process_absence": processes,
        "emergency_signer_deletion": emergency,
        "fixture_projection": fixture,
        "unexplained_live_state_differences": [],
    }
    return state, claims, installed_manifest_binding, membrane


def authority_binding(value: dict[str, Any]) -> str:
    body = {key: item for key, item in value.items() if key != "authority_sha256"}
    return document_sha256(
        {
            "domain": "substrate.r3-macos-finalizer-reboot-aware-cleanup-authority.v1",
            "authority": body,
        }
    )


def build_authority(
    cleanup_commit: str, source_sha256: str, installer_sha256: str
) -> dict[str, Any]:
    state, claims, installed_manifest_binding, _ = observe_state(
        cleanup_commit, source_sha256, installer_sha256
    )
    value = {
        "schema_owner": "substrate.r3-macos-finalizer-reboot-aware-cleanup-authority",
        "schema_version": 1,
        "experiment_id": EXPERIMENT_ID,
        "repetition_scopes": SCOPES,
        "authority_path": str(AUTHORITY_PATH),
        "cleanup_commit": cleanup_commit,
        "cleanup_source_sha256": source_sha256,
        "installer_source_sha256": installer_sha256,
        "candidate_freeze_manifest_sha256": state["installed_manifest"]["sha256"],
        "root_install_claims": claims,
        "root_install_claims_binding_sha256": document_sha256(claims),
        "root_install_preclaim_sha256": document_sha256(claims["preclaim"]),
        "root_install_completion_sha256": document_sha256(claims["completion"]),
        "installed_candidate_manifest_binding": installed_manifest_binding,
        "installed_candidate_manifest_binding_sha256": document_sha256(
            installed_manifest_binding
        ),
        "post_reboot_rebind": state,
        "post_reboot_rebind_sha256": document_sha256(state),
        "cleanup_only": True,
        "native_experiment_execution_authorized": False,
        "security_framework_or_keychain_queries_authorized": False,
        "authority_sha256": "",
    }
    value["authority_sha256"] = authority_binding(value)
    return value


def validate_authority(
    value: Any, expected_file_sha256: str
) -> tuple[dict[str, Any], Any]:
    expected_fields = {
        "schema_owner",
        "schema_version",
        "experiment_id",
        "repetition_scopes",
        "authority_path",
        "cleanup_commit",
        "cleanup_source_sha256",
        "installer_source_sha256",
        "candidate_freeze_manifest_sha256",
        "root_install_claims",
        "root_install_claims_binding_sha256",
        "root_install_preclaim_sha256",
        "root_install_completion_sha256",
        "installed_candidate_manifest_binding",
        "installed_candidate_manifest_binding_sha256",
        "post_reboot_rebind",
        "post_reboot_rebind_sha256",
        "cleanup_only",
        "native_experiment_execution_authorized",
        "security_framework_or_keychain_queries_authorized",
        "authority_sha256",
    }
    if not isinstance(value, dict) or set(value) != expected_fields:
        fail("reboot cleanup authority changed its closed shape")
    claims = value.get("root_install_claims")
    if not isinstance(claims, dict):
        fail("reboot cleanup authority lacks exact durable canonical claim copies")
    if (
        value.get("schema_owner")
        != "substrate.r3-macos-finalizer-reboot-aware-cleanup-authority"
        or value.get("schema_version") != 1
        or value.get("experiment_id") != EXPERIMENT_ID
        or value.get("repetition_scopes") != SCOPES
        or value.get("authority_path") != str(AUTHORITY_PATH)
        or not is_git_id(value.get("cleanup_commit"))
        or not is_sha256(value.get("cleanup_source_sha256"))
        or not is_sha256(value.get("installer_source_sha256"))
        or not is_sha256(value.get("candidate_freeze_manifest_sha256"))
        or value.get("root_install_claims_binding_sha256") != document_sha256(claims)
        or value.get("root_install_preclaim_sha256")
        != document_sha256(claims.get("preclaim"))
        or value.get("root_install_completion_sha256")
        != document_sha256(claims.get("completion"))
        or value.get("installed_candidate_manifest_binding_sha256")
        != document_sha256(value.get("installed_candidate_manifest_binding"))
        or value.get("post_reboot_rebind_sha256")
        != document_sha256(value.get("post_reboot_rebind"))
        or value.get("candidate_freeze_manifest_sha256")
        != value.get("post_reboot_rebind", {})
        .get("installed_manifest", {})
        .get("sha256")
        or value.get("cleanup_only") is not True
        or value.get("native_experiment_execution_authorized") is not False
        or value.get("security_framework_or_keychain_queries_authorized") is not False
        or value.get("authority_sha256") != authority_binding(value)
        or not is_sha256(expected_file_sha256)
    ):
        fail("reboot cleanup authority changed its exact binding")
    fresh, fresh_claims, fresh_installed, membrane = observe_state(
        value["cleanup_commit"],
        value["cleanup_source_sha256"],
        value["installer_source_sha256"],
    )
    if (
        fresh != value["post_reboot_rebind"]
        or fresh_claims != value["root_install_claims"]
        or fresh_installed != value["installed_candidate_manifest_binding"]
    ):
        fail("live post-reboot state differs from the exact sealed logical rebind")
    return value, membrane


def unlink_bound(membrane: Any, value: dict[str, Any], physical_field: str) -> None:
    path = pathlib.Path(value["path"])
    data, observed = membrane.read_exact_sized_file(
        path,
        value["uid"],
        value["gid"],
        value["mode"],
        value["size"],
        MAX_MANAGED_FILE_BYTES,
    )
    if (
        sha_bytes(data) != value["sha256"]
        or physical(path, observed) != value[physical_field]
    ):
        fail(f"rebound cleanup file changed before exact unlink: {path}")
    parent = membrane.open_directory_chain(path.parent)
    try:
        before = os.stat(path.name, dir_fd=parent, follow_symlinks=False)
        if not membrane.same_stat(before, observed):
            fail(f"rebound cleanup file changed before unlinkat: {path}")
        os.unlink(path.name, dir_fd=parent)
        os.fsync(parent)
    finally:
        os.close(parent)
    absent(path)


def remove_inventory_root(membrane: Any, value: dict[str, Any]) -> None:
    root = pathlib.Path(value["path"])
    live = membrane.directory_identity(root, 0, 0, value["mode"])
    if live != value["physical_identity"]:
        fail(f"rebound managed root changed before cleanup: {root}")
    for child in reversed(value["children"]):
        unlink_bound(membrane, child, "physical_identity")
    descriptor = membrane.open_directory_chain(root)
    try:
        held = os.fstat(descriptor)
        after_identity = membrane.directory_identity(root, 0, 0, value["mode"])
        if os.listdir(descriptor) or stable_directory_identity(
            after_identity
        ) != stable_directory_identity(live):
            fail(f"rebound managed root is not exact empty: {root}")
        parent = membrane.open_directory_chain(root.parent)
        try:
            before = os.stat(root.name, dir_fd=parent, follow_symlinks=False)
            if not membrane.same_stat(before, held):
                fail(f"rebound managed root changed before rmdir: {root}")
            os.rmdir(root.name, dir_fd=parent)
            os.fsync(parent)
        finally:
            os.close(parent)
    finally:
        os.close(descriptor)
    absent(root)


def remove_empty_directory(membrane: Any, value: dict[str, Any]) -> None:
    path = pathlib.Path(value["path"])
    live = membrane.directory_identity(path, value["uid"], value["gid"], value["mode"])
    if stable_directory_identity(live) != stable_directory_identity(
        value["post_reboot_physical_identity"]
    ) or list(os.scandir(path)):
        fail(f"rebound directory changed or is not empty before rmdir: {path}")
    parent = membrane.open_directory_chain(path.parent)
    try:
        os.rmdir(path.name, dir_fd=parent)
        os.fsync(parent)
    finally:
        os.close(parent)
    absent(path)


def restoration_binding(value: dict[str, Any]) -> str:
    body = {key: item for key, item in value.items() if key != "restoration_sha256"}
    return document_sha256(
        {
            "domain": "substrate.r3-macos-finalizer-reboot-aware-cleanup-restoration.v1",
            "restoration": body,
        }
    )


def execute(authority: dict[str, Any], membrane: Any) -> dict[str, Any]:
    if os.path.lexists(RESTORATION_PATH):
        fail("reboot cleanup restoration receipt already exists; route is terminal")
    state = authority["post_reboot_rebind"]
    launchd = state["launchd_rehydration"]
    bootout = run(
        [
            "/bin/launchctl",
            "bootout",
            "system",
            launchd["plist_path"],
        ]
    )
    if bootout.returncode != 0:
        fail(f"exact reboot-rehydrated launchd bootout failed: {bootout.returncode}")
    launchd_absent = run(
        ["/bin/launchctl", "print", f"system/{membrane.FINALIZER_LABEL}"]
    )
    membrane.validate_launchctl_not_found_streams(
        membrane.FINALIZER_LABEL,
        launchd_absent.returncode,
        membrane.raw_stream(launchd_absent.stdout),
        membrane.raw_stream(launchd_absent.stderr),
    )
    socket_absent = membrane.observe_platform_managed_socket_absence(
        membrane.ENDPOINT_PATH
    )

    # Exact manifest rollback order after the service/socket boundary.
    inbox = next(
        value
        for value in state["installed_directories"]
        if value["path"] == str(membrane.INBOX_ROOT)
    )
    remove_empty_directory(membrane, inbox)
    remove_inventory_root(membrane, state["runner_root"])
    for value in state["protocol_files"]:
        unlink_bound(membrane, value, "physical_identity")
    remove_inventory_root(membrane, state["creator_root"])
    unlink_bound(membrane, state["installed_manifest"], "post_reboot_physical_identity")
    by_role = {value["role"]: value for value in state["installed_artifacts"]}
    cleanup_roles = [
        spec[0]
        for spec in reversed(membrane.ARTIFACT_SPECS)
        if spec[0] != "capability_manifest"
    ]
    for role in cleanup_roles:
        unlink_bound(membrane, by_role[role], "post_reboot_physical_identity")
    support = [
        value
        for value in state["installed_directories"]
        if value["path"] != str(membrane.INBOX_ROOT)
    ]
    for value in reversed(support):
        remove_empty_directory(membrane, value)
    external_global = state["external_global_pre_effect"]
    unlink_bound(membrane, external_global, "physical_identity")
    if os.listdir(membrane.GLOBAL_EXCHANGE):
        fail("global exchange retained unexplained state after cleanup")

    manifest, manifest_bytes = membrane.load_manifest()
    expected_global_values = [
        value
        for value in authority["root_install_claims"]["completion"][
            "installed_directories"
        ]
        if value["path"] == str(membrane.GLOBAL_EXCHANGE)
    ]
    if len(expected_global_values) != 1:
        fail("durable claim copy lost the exact global exchange binding")
    final_observation = membrane.observe_admin_restoration_absence(
        manifest, expected_global_values[0]
    )
    if os.listdir(membrane.GLOBAL_EXCHANGE):
        fail("global exchange changed after complete baseline observation")
    receipt = {
        "schema_owner": "substrate.r3-macos-finalizer-reboot-aware-cleanup-restoration",
        "schema_version": 1,
        "experiment_id": EXPERIMENT_ID,
        "repetition_scopes": SCOPES,
        "receipt_path": str(RESTORATION_PATH),
        "cleanup_commit": authority["cleanup_commit"],
        "cleanup_authority_file_sha256": sha_bytes(canonical(authority)),
        "cleanup_authority_sha256": authority["authority_sha256"],
        "candidate_freeze_manifest_sha256": sha_bytes(manifest_bytes),
        "root_install_claims_binding_sha256": authority[
            "root_install_claims_binding_sha256"
        ],
        "preserved_inventory_sha256": PRESERVED_INVENTORY_SHA256,
        "preservation_finished_utc": PRESERVATION_FINISHED_UTC,
        "observed_reboot_utc": OBSERVED_REBOOT_UTC,
        "post_reboot_rebind_sha256": authority["post_reboot_rebind_sha256"],
        "launchctl_bootout_exit_status": bootout.returncode,
        "launchctl_bootout_stdout": raw_stream(bootout.stdout),
        "launchctl_bootout_stderr": raw_stream(bootout.stderr),
        "launchctl_print_absent_exit_status": launchd_absent.returncode,
        "platform_managed_socket_absence": socket_absent,
        "final_restoration_observation": final_observation,
        "global_exchange_children": [],
        "security_framework_or_keychain_queries_performed": False,
        "native_experiment_executed": False,
        "complete_disposable_baseline_restored": True,
        "restoration_sha256": "",
    }
    receipt["restoration_sha256"] = restoration_binding(receipt)
    membrane.publish_bytes_no_clobber(canonical(receipt), RESTORATION_PATH, 0, 0, 0o444)
    receipt_bytes, _ = membrane.read_bounded_exact_file(
        RESTORATION_PATH, 0, 0, 0o444, MAX_AUTHORITY_BYTES
    )
    if receipt_bytes != canonical(receipt):
        fail("reboot cleanup restoration receipt changed after publication")
    return receipt


def load_authority() -> tuple[dict[str, Any], Any]:
    expected = os.environ.get(AUTHORITY_ENV)
    if not is_sha256(expected):
        fail("reboot cleanup route lacks one canonical authority digest")
    data = AUTHORITY_PATH.read_bytes()
    observed = os.lstat(AUTHORITY_PATH)
    after = os.lstat(AUTHORITY_PATH)
    if (
        not stat.S_ISREG(observed.st_mode)
        or observed.st_uid != 501
        or observed.st_gid != 20
        or stat.S_IMODE(observed.st_mode) != 0o400
        or observed.st_nlink != 1
        or not 0 < len(data) <= MAX_AUTHORITY_BYTES
        or len(data) != observed.st_size
        or stat_tuple(observed) != stat_tuple(after)
        or sha_bytes(data) != expected
    ):
        fail("reboot cleanup authority file changed its sealed identity")
    try:
        value = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise Stop("reboot cleanup authority is not strict JSON") from error
    if canonical(value) != data:
        fail("reboot cleanup authority is not canonical JSON")
    return validate_authority(value, expected)


def main() -> int:
    if (
        os.geteuid() != 0
        or os.getuid() != 0
        or os.getegid() != 0
        or pwd.getpwuid(0).pw_name != "root"
        or pathlib.Path.cwd() != pathlib.Path("/")
    ):
        fail("reboot cleanup membrane requires exact root execution")
    if any(key.startswith("SUBSTRATE_") for key in os.environ):
        fail("reboot cleanup membrane rejects SUBSTRATE_* environment input")
    for key, expected in FIXED_ENV.items():
        if os.environ.get(key) != expected:
            fail(f"reboot cleanup fixed environment changed: {key}")
    if len(sys.argv) == 5 and sys.argv[1] == "observe":
        if any(key.startswith("R3_") for key in os.environ):
            fail("reboot cleanup observation received authority input")
        os.environ.clear()
        os.environ.update(FIXED_ENV)
        authority = build_authority(sys.argv[2], sys.argv[3], sys.argv[4])
        sys.stdout.buffer.write(canonical(authority))
        return 0
    if len(sys.argv) != 1:
        fail("reboot cleanup execution accepts no arguments")
    authority_keys = sorted(key for key in os.environ if key.startswith("R3_"))
    if authority_keys != [AUTHORITY_ENV] or not is_sha256(
        os.environ.get(AUTHORITY_ENV)
    ):
        fail("reboot cleanup execution lacks its one exact authority input")
    authority_digest = os.environ[AUTHORITY_ENV]
    os.environ.clear()
    os.environ.update({**FIXED_ENV, AUTHORITY_ENV: authority_digest})
    authority, membrane = load_authority()
    receipt = execute(authority, membrane)
    print(
        "R3 reboot-aware cleanup PASS "
        f"restoration_sha256={receipt['restoration_sha256']}",
        flush=True,
    )
    return 0


def run_entrypoint() -> NoReturn:
    try:
        raise SystemExit(main())
    except Stop as error:
        print(f"R3 reboot-aware cleanup stopped: {error}", file=sys.stderr)
        raise SystemExit(78) from error
    except Exception as error:
        # Imported root-install primitives carry their own fail-closed Stop type.
        print(f"R3 reboot-aware cleanup stopped: {error}", file=sys.stderr)
        raise SystemExit(78) from error


if __name__ == "__main__":
    run_entrypoint()
