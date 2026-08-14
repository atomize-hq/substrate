#!/bin/zsh
set -euo pipefail

readonly REPOSITORY="${0:A:h:h:h}"
readonly ROOT_SOURCE="${REPOSITORY}/scripts/mac/r3-macos-finalizer-root-install.py"

[[ -f "${ROOT_SOURCE}" && ! -L "${ROOT_SOURCE}" ]] \
    || { print -u2 -- "platform socket fixture source is unavailable"; exit 1; }

env -i PATH=/usr/bin:/bin:/usr/sbin:/sbin LANG=C LC_ALL=C TZ=UTC \
    PYTHONDONTWRITEBYTECODE=1 \
    /usr/bin/python3 - "${ROOT_SOURCE}" <<'PY'
import ast
import copy
import errno
import importlib.util
import inspect
import os
import pathlib
import stat
import sys
from types import SimpleNamespace


source = pathlib.Path(sys.argv[1])
spec = importlib.util.spec_from_file_location("r3_rcv_04_root_installer", source)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def reject(call, label):
    try:
        call()
    except module.Stop:
        return
    raise SystemExit(f"platform socket regression accepted {label}")


def resign(value):
    value["platform_managed_socket_absence_sha256"] = (
        module.platform_managed_socket_absence_binding(value)
    )
    return value


def mutate(value, change):
    candidate = copy.deepcopy(value)
    change(candidate)
    return resign(candidate)


# The generic immutable-ancestor validator remains closed: the live root:daemon 0775 runtime
# directory is intentionally rejected.  RCV-04 does not add a path or endpoint exception there.
generic_source = inspect.getsource(module.open_directory_chain)
if any(
    token in generic_source
    for token in ("ENDPOINT_PATH", "PLATFORM_MANAGED_SOCKET_PARENT", "/private/var/run")
):
    raise SystemExit("generic directory-chain validator contains a platform socket exception")
reject(
    lambda: module.open_directory_chain(pathlib.Path("/private/var/run")),
    "the group-writable platform runtime as an immutable ancestor",
)

# This is the only live host observation in the regression.  It opens descriptors read-only and
# observes the currently absent compiled child; it does not create, unlink, or fsync the runtime.
if os.path.islink("/private/var/run") or not os.path.islink("/var"):
    raise SystemExit("host runtime aliases changed before the read-only observation")
require_no_effect_calls = inspect.getsource(
    module.observe_platform_managed_socket_absence
)
if any(
    token in require_no_effect_calls
    for token in (
        "os.fsync",
        "os.unlink",
        "os.remove",
        "os.rmdir",
        "os.mkdir",
        "os.O_CREAT",
        "os.O_WRONLY",
        "os.O_RDWR",
        "require_durable_absence",
    )
):
    raise SystemExit("platform socket observer gained a native mutation or durability claim")
canonical_socket = module.observe_platform_managed_socket_absence(
    module.ENDPOINT_PATH
)
module.validate_platform_managed_socket_absence(canonical_socket)
reject(
    lambda: module.observe_platform_managed_socket_absence(
        pathlib.Path(
            "/private/var/run/com.atomize.substrate.r3-macos-evidence-finalizer.other.sock"
        )
    ),
    "an alternate platform socket",
)
reject(
    lambda: module.observe_platform_managed_socket_absence(
        pathlib.Path(
            "/var/run/com.atomize.substrate.r3-macos-evidence-finalizer.v2.sock"
        )
    ),
    "the /var/run alias",
)


def validate_mutation(label, change):
    reject(
        lambda: module.validate_platform_managed_socket_absence(
            mutate(canonical_socket, change)
        ),
        label,
    )


unknown = copy.deepcopy(canonical_socket)
unknown["unknown"] = True
reject(
    lambda: module.validate_platform_managed_socket_absence(unknown),
    "an unknown platform socket field",
)
missing = copy.deepcopy(canonical_socket)
del missing["classification"]
reject(
    lambda: module.validate_platform_managed_socket_absence(missing),
    "a missing platform socket field",
)
validate_mutation(
    "an alternate compiled socket path",
    lambda value: value.__setitem__(
        "socket_path", "/private/var/run/alternate.sock"
    ),
)
validate_mutation(
    "the /var/run platform-parent alias",
    lambda value: value.__setitem__("platform_parent_path", "/var/run"),
)
validate_mutation(
    "the /var/run nested platform-parent alias",
    lambda value: [
        identity.__setitem__("path", "/var/run")
        for identity in value["parent_identity_observations"]
    ],
)
for label, field, replacement in (
    ("an alternate platform-parent owner", "uid", 501),
    ("an alternate platform-parent group", "gid", 0),
    ("an alternate platform-parent permission mode", "mode", stat.S_IFDIR | 0o755),
    ("a symlink platform parent", "mode", stat.S_IFLNK | 0o775),
):
    validate_mutation(
        label,
        lambda value, field=field, replacement=replacement: [
            identity.__setitem__(field, replacement)
            for identity in value["parent_identity_observations"]
        ],
    )
validate_mutation(
    "a changed platform-parent identity",
    lambda value: value["parent_identity_observations"][2].__setitem__(
        "inode", value["parent_identity_observations"][2]["inode"] + 1
    ),
)
validate_mutation(
    "a nested platform-parent unknown field",
    lambda value: value["parent_identity_observations"][0].__setitem__(
        "unknown", True
    ),
)
validate_mutation(
    "a missing nested platform-parent field",
    lambda value: value["parent_identity_observations"][0].pop("device"),
)
validate_mutation(
    "reordered platform-parent observations",
    lambda value: value["parent_identity_observations"].reverse(),
)
validate_mutation(
    "reordered socket-child observations",
    lambda value: value["child_absence_observations"].reverse(),
)
for index, label in (
    (0, "a socket present before descriptor inspection"),
    (-1, "a socket reappearing after descriptor inspection"),
):
    validate_mutation(
        label,
        lambda value, index=index: value["child_absence_observations"][index].update(
            {"lstat_return": 0, "raw_errno": 0}
        ),
    )

# One minimal manifest is sufficient here because this gate is about restoration classification,
# not the separately fixture-bound 14-artifact cleanup schema covered by RCV-02.
installed_executable = (
    "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/"
    "synthetic-finalizer"
)
manifest = {
    "artifacts": [
        {
            "role": "finalizer_service",
            "intended_path": installed_executable,
            "signing_identifier": "com.atomize.substrate.synthetic-finalizer",
        }
    ]
}
expected_paths = module.admin_restoration_absence_paths(manifest)
if str(module.ENDPOINT_PATH) in expected_paths or any(
    pathlib.Path(value) == module.PLATFORM_MANAGED_SOCKET_PARENT
    or module.PLATFORM_MANAGED_SOCKET_PARENT in pathlib.Path(value).parents
    for value in expected_paths
):
    raise SystemExit("platform-managed socket escaped into ordinary absence paths")


def durable_record(path):
    return {
        "path": path,
        "parent_path": str(pathlib.Path(path).parent),
        "parent_device": 1,
        "parent_inode": len(path) + 1,
        "parent_uid": 0,
        "parent_gid": 0,
        "parent_mode": stat.S_IFDIR | 0o755,
        "parent_fsync_return": 0,
        "child_fstatat_return": -1,
        "child_raw_errno": errno.ENOENT,
        "parent_reopened_same_stable_identity": True,
    }


process_records = [
    {
        "pid": 1,
        "path": "/sbin/launchd",
        "classification": "path_observed",
        "raw_errno": 0,
    }
]
expected_global = {
    "path": str(module.GLOBAL_EXCHANGE),
    "device": 9,
    "inode": 10,
    "uid": 501,
    "gid": 0,
    "mode": stat.S_IFDIR | 0o700,
}
claims = {"completion": {"installed_directories": [expected_global]}}
launchctl_stderr = (
    f'Bad request.\nCould not find service "{module.FINALIZER_LABEL}" '
    "in domain for system\n"
).encode()

# Exercise the actual observation producer without native effects by substituting only its external
# launchctl/process/durability probes.  Every ordinary path must retain the generic durable-absence
# call, while the socket has exactly one separately typed read-only observation.
durable_calls = []
absent_calls = []
module.observe_platform_managed_socket_absence = lambda path: canonical_socket
module.require_absent = lambda path: absent_calls.append(str(path))


def synthetic_durable(path):
    durable_calls.append(str(path))
    return durable_record(str(path))


module.require_durable_absence = synthetic_durable
module.run_command = lambda argv: SimpleNamespace(
    returncode=113, stdout=b"", stderr=launchctl_stderr
)
module.process_snapshot = lambda: process_records
module.directory_identity = lambda path, uid, gid, mode: expected_global
observation = module.observe_admin_restoration_absence(manifest, expected_global)
if absent_calls != expected_paths or durable_calls != expected_paths:
    raise SystemExit("ordinary restoration paths lost exact absence or durability coverage")
if observation["platform_managed_socket_absence"] != canonical_socket:
    raise SystemExit("restoration omitted the typed platform-managed socket observation")
if str(module.ENDPOINT_PATH) in {
    item["path"] for item in observation["durable_absences"]
}:
    raise SystemExit("platform-managed socket was mislabeled as a durable absence")

# Successful, terminal, and pre-Security terminal receipts converge on this one closed validator.
# Its behavioral gate accepts the canonical producer value, while AST proves both outer receipt
# validators invoke it rather than maintaining divergent socket schemas.
module.validate_admin_restoration_receipt_observation(
    observation, manifest, claims, expected_global
)
outer_unknown = copy.deepcopy(observation)
outer_unknown["unknown"] = True
reject(
    lambda: module.validate_admin_restoration_receipt_observation(
        outer_unknown, manifest, claims, expected_global
    ),
    "an unknown restoration observation field",
)
outer_missing = copy.deepcopy(observation)
del outer_missing["platform_managed_socket_absence"]
reject(
    lambda: module.validate_admin_restoration_receipt_observation(
        outer_missing, manifest, claims, expected_global
    ),
    "a missing restoration observation field",
)
tree = ast.parse(source.read_text())
functions = {
    node.name: node
    for node in tree.body
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
}


def called_names(function_name):
    return {
        node.func.id
        for node in ast.walk(functions[function_name])
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
    }


shared_validator = "validate_admin_restoration_receipt_observation"
if shared_validator not in called_names("validate_admin_restoration_receipt"):
    raise SystemExit("successful restoration receipt bypasses the shared socket schema")
if shared_validator not in called_names("validate_terminal_admin_restoration_receipt"):
    raise SystemExit("terminal/pre-Security receipt bypasses the shared socket schema")
if "observe_platform_managed_socket_absence" not in called_names(
    "observe_admin_restoration_absence"
):
    raise SystemExit("restoration producer bypasses the typed platform socket observer")
if "observe_platform_managed_socket_absence" not in called_names(
    "exact_service_absent"
):
    raise SystemExit("service-absence gate bypasses the typed platform socket observer")

outer_substitution = copy.deepcopy(observation)
outer_substitution["platform_managed_socket_absence"] = mutate(
    canonical_socket,
    lambda value: value.__setitem__("socket_path", "/private/var/run/alternate.sock"),
)
reject(
    lambda: module.validate_admin_restoration_receipt_observation(
        outer_substitution, manifest, claims, expected_global
    ),
    "a receipt-level socket substitution",
)

bad_launchd_status = copy.deepcopy(observation)
bad_launchd_status["launchctl_print_not_found_exit_status"] = 112
reject(
    lambda: module.validate_admin_restoration_receipt_observation(
        bad_launchd_status, manifest, claims, expected_global
    ),
    "a non-113 launchd status",
)
bad_launchd_raw = copy.deepcopy(observation)
bad_launchd_raw["launchctl_print_stderr"] = module.raw_stream(b"not found\n")
reject(
    lambda: module.validate_admin_restoration_receipt_observation(
        bad_launchd_raw, manifest, claims, expected_global
    ),
    "a noncanonical launchd not-found stream",
)
matching_records = [
    {
        "pid": 1,
        "path": installed_executable,
        "classification": "path_observed",
        "raw_errno": 0,
    }
]
matching_process = copy.deepcopy(observation)
matching_process["process_records"] = matching_records
matching_process["process_snapshot"] = module.raw_stream(
    module.canonical(matching_records)
)
matching_process["process_snapshot_sha256"] = matching_process[
    "process_snapshot"
]["sha256"]
matching_process["matching_experiment_processes"] = matching_records
reject(
    lambda: module.validate_admin_restoration_receipt_observation(
        matching_process, manifest, claims, expected_global
    ),
    "a matching installed experiment process",
)

print("R3-RCV-04 platform-managed socket absence regression: PASS")
PY
