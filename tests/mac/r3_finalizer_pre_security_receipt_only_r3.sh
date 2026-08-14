#!/bin/zsh
set -euo pipefail

readonly REPOSITORY="${0:A:h:h:h}"
readonly ROOT_SOURCE="${REPOSITORY}/scripts/mac/r3-macos-finalizer-root-install.py"
readonly WORK_ROOT="$(/usr/bin/mktemp -d /private/tmp/substrate-r3-rcv-05.XXXXXX)"

cleanup() {
    /bin/rm -rf -- "${WORK_ROOT}"
}
trap cleanup EXIT

env -i PATH=/usr/bin:/bin:/usr/sbin:/sbin LANG=C LC_ALL=C TZ=UTC \
    PYTHONDONTWRITEBYTECODE=1 \
    /usr/bin/python3 - "${ROOT_SOURCE}" "${WORK_ROOT}" <<'PY'
import ast
import copy
import errno
import importlib.util
import os
import pathlib
import stat
import sys
from types import SimpleNamespace


source = pathlib.Path(sys.argv[1])
work_root = pathlib.Path(sys.argv[2])
spec = importlib.util.spec_from_file_location("r3_rcv_05_root_installer", source)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def reject(call, label):
    try:
        call()
    except module.Stop:
        return
    raise SystemExit(f"receipt-only regression accepted {label}")


# Admission is one exact environment selector and dispatches before the ordinary install lane.
saved_geteuid = module.os.geteuid
saved_getegid = module.os.getegid
saved_getpwuid = module.pwd.getpwuid
saved_cwd = module.pathlib.Path.cwd
saved_argv = list(module.sys.argv)
module.os.geteuid = lambda: 0
module.os.getegid = lambda: 0
module.pwd.getpwuid = lambda uid: SimpleNamespace(pw_name="root")
module.pathlib.Path.cwd = classmethod(lambda cls: pathlib.Path("/"))
module.sys.argv = [str(source)]


def surface(environment):
    os.environ.clear()
    os.environ.update(environment)
    return module.verify_process_surface()


fixed = dict(module.FIXED_ENV)
receipt_environment = dict(fixed)
receipt_environment[module.PRE_SECURITY_RECOVERY_AUTHORITY_ENV] = "a" * 64
if surface(receipt_environment) != ("pre_security_receipt_only", "a" * 64):
    raise SystemExit("exact pre-Security receipt selector was not admitted")
root_environment = dict(fixed)
root_environment[module.ROOT_INSTALL_AUTHORITY_ENV] = "b" * 64
if surface(root_environment) != ("root_install", "b" * 64):
    raise SystemExit("ordinary root-install selector changed")
both = dict(fixed)
both[module.ROOT_INSTALL_AUTHORITY_ENV] = "b" * 64
both[module.PRE_SECURITY_RECOVERY_AUTHORITY_ENV] = "a" * 64
reject(lambda: surface(both), "two R3 authorities")
alternate = dict(fixed)
alternate["R3_PRE_SECURITY_RECOVERY_AUTHORITY_SHA512"] = "a" * 64
reject(lambda: surface(alternate), "an alternate recovery selector")
module.os.geteuid = saved_geteuid
module.os.getegid = saved_getegid
module.pwd.getpwuid = saved_getpwuid
module.pathlib.Path.cwd = saved_cwd
module.sys.argv = saved_argv


# Static branch-specific reachability: only the receipt publication primitive may unlink its own
# deterministic pending link. No installer, cleanup mutation, runner, codesign, or Security route
# is reachable from the receipt-only continuation.
tree = ast.parse(source.read_text())
functions = {
    node.name: node
    for node in tree.body
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
}


def direct_calls(name):
    return {
        node.func.id
        for node in ast.walk(functions[name])
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
    }


def closure(starts):
    reached = set(starts)
    pending = list(starts)
    while pending:
        current = pending.pop()
        for called in direct_calls(current):
            if called in functions and called not in reached:
                reached.add(called)
                pending.append(called)
    return reached


receipt_surface = closure(
    {
        "load_manifest_documents_only",
        "publish_or_validate_pre_security_admin_restoration_receipt",
    }
)
forbidden = {
    "verify_code_identity",
    "ensure_directory",
    "install_candidate",
    "install_bytes_prefix_recoverable",
    "create_or_load_preclaim",
    "create_or_load_completion",
    "cleanup_installed_candidate",
    "remove_root_install_claims",
    "unlink_exact_file",
    "unlink_terminal_runner_file",
    "remove_terminal_runner_root",
    "complete_terminal_admin_cleanup",
    "complete_successful_admin_cleanup",
    "reset_unclaimed_root",
}
unexpected = sorted(receipt_surface & forbidden)
if unexpected:
    raise SystemExit(f"receipt-only branch reaches a forbidden effect function: {unexpected}")
reachable_text = "\n".join(ast.unparse(functions[name]) for name in sorted(receipt_surface))
for token in (
    "RUNNER_PATH",
    "Security.framework",
    "SecItem",
    "Keychain",
    "/usr/bin/codesign",
    "launchctl', 'bootout",
    "launchctl', 'bootstrap",
    "os.kill",
    "os.rmdir",
):
    if token in reachable_text:
        raise SystemExit(f"receipt-only branch acquired forbidden call surface: {token}")
if "publish_bytes_no_clobber" not in receipt_surface:
    raise SystemExit("receipt-only branch bypasses the one permitted publication primitive")
if "observe_platform_managed_socket_absence" not in receipt_surface:
    raise SystemExit("receipt-only branch bypasses the RCV-04 socket observation")
if not {
    "validate_archived_root_install_claims",
    "validate_installed_candidate_manifest_binding",
    "validate_pre_security_cleanup_observation",
}.issubset(direct_calls("validate_pre_security_admin_authorization")):
    raise SystemExit("durable authorization bypasses an embedded authority validator")
if not {
    "validate_root_install_claims_binding_shape",
    "validate_archived_preclaim_document",
    "validate_completion",
}.issubset(direct_calls("validate_archived_root_install_claims")):
    raise SystemExit("archived claims bypass the predecessor document validators")


# Closed authorization/receipt fixtures exercise their real outer bindings while the already
# predecessor-tested root-install document validators are replaced by exact fixture comparators.
manifest_bytes = b'{"synthetic":"manifest"}'
manifest = {"source_tree": "1" * 40}
expected_global = {
    "path": str(module.GLOBAL_EXCHANGE),
    "device": 10,
    "inode": 20,
    "uid": 501,
    "gid": 0,
    "mode": stat.S_IFDIR | 0o700,
    "link_count": 2,
    "size": 64,
    "modified_seconds": 100,
    "modified_nanoseconds": 200,
}
claims = {
    "preclaim": {"fixture": "preclaim"},
    "completion": {
        "installed_directories": [expected_global],
        "rollback_plan_sha256": "2" * 64,
    },
}
installed_manifest = {"fixture": "installed-manifest"}
pre_cleanup = {"fixture": "pre-cleanup-observation"}
operator_bytes = b'{"synthetic":"operator"}'
operator = {
    "operator_authority_sha256": "3" * 64,
    "failed_attempt_state_sha256": "4" * 64,
}

saved_archived_claims = module.validate_archived_root_install_claims
saved_installed_manifest = module.validate_installed_candidate_manifest_binding
saved_cleanup_observation = module.validate_pre_security_cleanup_observation
module.validate_archived_root_install_claims = (
    lambda value, _manifest, _bytes: value
    if value == claims
    else module.fail("synthetic archived claims changed")
)
module.validate_installed_candidate_manifest_binding = (
    lambda value, _manifest, _bytes: value
    if value == installed_manifest
    else module.fail("synthetic installed manifest changed")
)
module.validate_pre_security_cleanup_observation = (
    lambda value, _manifest, _claims, _installed: value
    if value == pre_cleanup
    else module.fail("synthetic cleanup observation changed")
)

authorization = {
    "schema_owner": "substrate.r3-macos-finalizer-pre-security-admin-cleanup-authorization",
    "schema_version": 2,
    "experiment_id": module.EXPERIMENT_ID,
    "authority_kind": "failed_runner_before_first_security",
    "authorization_path": str(module.PRE_SECURITY_ADMIN_AUTHORIZATION_PATH),
    "operator_authority_file_sha256": module.sha_bytes(operator_bytes),
    "operator_authority_sha256": operator["operator_authority_sha256"],
    "candidate_freeze_manifest_sha256": module.sha_bytes(manifest_bytes),
    "root_install_claims": claims,
    "root_install_claims_binding_sha256": module.document_sha256(claims),
    "installed_candidate_manifest": installed_manifest,
    "installed_candidate_manifest_identity_sha256": module.document_sha256(
        installed_manifest
    ),
    "pre_cleanup_observation": pre_cleanup,
    "pre_cleanup_observation_sha256": module.document_sha256(pre_cleanup),
    "rollback_plan_sha256": claims["completion"]["rollback_plan_sha256"],
    "security_framework_or_keychain_queries_authorized": False,
    "native_experiment_effects_authorized": False,
    "admin_cleanup_authorized": True,
    "authorization_sha256": "",
}
authorization["authorization_sha256"] = module.pre_security_recovery_binding(
    authorization, "authorization_sha256"
)
module.validate_pre_security_admin_authorization(
    authorization, manifest, manifest_bytes, operator, operator_bytes
)


def resign_authorization(value):
    value["root_install_claims_binding_sha256"] = module.document_sha256(
        value["root_install_claims"]
    )
    value["installed_candidate_manifest_identity_sha256"] = module.document_sha256(
        value["installed_candidate_manifest"]
    )
    value["pre_cleanup_observation_sha256"] = module.document_sha256(
        value["pre_cleanup_observation"]
    )
    value["authorization_sha256"] = module.pre_security_recovery_binding(
        value, "authorization_sha256"
    )
    return value


for label, change in (
    (
        "changed embedded claims",
        lambda value: value.__setitem__("root_install_claims", {"changed": True}),
    ),
    (
        "changed embedded installed manifest",
        lambda value: value.__setitem__(
            "installed_candidate_manifest", {"changed": True}
        ),
    ),
    (
        "changed embedded cleanup observation",
        lambda value: value.__setitem__("pre_cleanup_observation", {"changed": True}),
    ),
):
    candidate = copy.deepcopy(authorization)
    change(candidate)
    resign_authorization(candidate)
    reject(
        lambda candidate=candidate: module.validate_pre_security_admin_authorization(
            candidate, manifest, manifest_bytes, operator, operator_bytes
        ),
        label,
    )
unknown = copy.deepcopy(authorization)
unknown["unknown"] = True
reject(
    lambda: module.validate_pre_security_admin_authorization(
        unknown, manifest, manifest_bytes, operator, operator_bytes
    ),
    "an unknown authorization field",
)
changed_operator = copy.deepcopy(operator)
changed_operator["operator_authority_sha256"] = "5" * 64
reject(
    lambda: module.validate_pre_security_admin_authorization(
        authorization, manifest, manifest_bytes, changed_operator, operator_bytes
    ),
    "changed operator authority",
)

module.validate_archived_root_install_claims = saved_archived_claims
module.validate_installed_candidate_manifest_binding = saved_installed_manifest
module.validate_pre_security_cleanup_observation = saved_cleanup_observation


authorization_bytes = module.canonical(authorization)
authorization_observed = SimpleNamespace(
    st_dev=1,
    st_ino=2,
    st_uid=0,
    st_gid=0,
    st_mode=stat.S_IFREG | 0o444,
    st_nlink=1,
    st_size=len(authorization_bytes),
    st_mtime_ns=7_000_000_008,
)
admin_observation = {"fixture": "current-clean-baseline"}
restoration_observation = {
    "admin_restoration_observation": admin_observation,
    "retained_global_exchange": {
        "identity": expected_global,
        "children": [],
        "children_sha256": module.document_sha256([]),
    },
}
saved_restoration_validator = module.validate_pre_security_restoration_observation
module.validate_pre_security_restoration_observation = (
    lambda value, _manifest, _claims, _global: value
    if value == restoration_observation
    else module.fail("synthetic current clean baseline changed")
)
receipt = module.build_pre_security_admin_restoration_receipt(
    manifest,
    manifest_bytes,
    authorization,
    authorization_bytes,
    authorization_observed,
    operator,
    operator_bytes,
    restoration_observation,
)
module.validate_pre_security_admin_restoration_receipt(
    receipt,
    manifest,
    manifest_bytes,
    authorization,
    authorization_bytes,
    authorization_observed,
    operator,
    operator_bytes,
)
bad_receipt = copy.deepcopy(receipt)
bad_receipt["unknown"] = True
reject(
    lambda: module.validate_pre_security_admin_restoration_receipt(
        bad_receipt,
        manifest,
        manifest_bytes,
        authorization,
        authorization_bytes,
        authorization_observed,
        operator,
        operator_bytes,
    ),
    "an unknown receipt field",
)
wrong_identity = copy.deepcopy(receipt)
wrong_identity["admin_cleanup_authorization_physical_identity_sha256"] = "6" * 64
wrong_identity["restoration_sha256"] = module.pre_security_recovery_binding(
    wrong_identity, "restoration_sha256"
)
reject(
    lambda: module.validate_pre_security_admin_restoration_receipt(
        wrong_identity,
        manifest,
        manifest_bytes,
        authorization,
        authorization_bytes,
        authorization_observed,
        operator,
        operator_bytes,
    ),
    "a wrong authorization physical identity",
)


# The actual state-machine continuation publishes exactly once for receipt absence, replays an
# exact existing receipt without write, and preserves/rejects invalid existing bytes.
saved_route = {
    name: getattr(module, name)
    for name in (
        "load_pre_security_admin_authorization",
        "observe_pre_security_restoration_baseline",
        "path_present",
        "publish_bytes_no_clobber",
        "read_bounded_exact_file",
        "read_complete_pre_security_receipt_residue",
        "revalidate_pre_security_authorization_unchanged",
        "validate_pre_security_receipt_root_state",
    )
}
module.load_pre_security_admin_authorization = lambda *_args: (
    authorization,
    authorization_bytes,
    authorization_observed,
    operator,
    operator_bytes,
)
module.observe_pre_security_restoration_baseline = lambda *_args: restoration_observation
module.read_complete_pre_security_receipt_residue = lambda *_args: None
module.revalidate_pre_security_authorization_unchanged = lambda *_args: None
state = {"final": False, "temporary": False, "bytes": None}
publication_calls = []


def synthetic_present(path):
    if path == module.PRE_SECURITY_ADMIN_RESTORATION_RECEIPT_PATH:
        return state["final"]
    if path == module.pending_path(module.PRE_SECURITY_ADMIN_RESTORATION_RECEIPT_PATH):
        return state["temporary"]
    return False


def synthetic_publish(data, path, uid, gid, mode):
    publication_calls.append((data, path, uid, gid, mode))
    state.update({"final": True, "temporary": False, "bytes": data})
    return authorization_observed


module.path_present = synthetic_present
module.publish_bytes_no_clobber = synthetic_publish
module.read_bounded_exact_file = lambda path, *_args: (
    state["bytes"], authorization_observed
)
module.validate_pre_security_receipt_root_state = lambda: sorted(
    [
        module.PRE_SECURITY_ADMIN_AUTHORIZATION_PATH.name,
        module.PRE_SECURITY_ADMIN_RESTORATION_RECEIPT_PATH.name,
    ]
)
published = module.publish_or_validate_pre_security_admin_restoration_receipt(
    manifest, manifest_bytes, module.sha_bytes(authorization_bytes)
)
if published != receipt or len(publication_calls) != 1:
    raise SystemExit("canonical all-clean continuation did not publish exactly once")

publication_calls.clear()
existing = module.publish_or_validate_pre_security_admin_restoration_receipt(
    manifest, manifest_bytes, module.sha_bytes(authorization_bytes)
)
if existing != receipt or publication_calls:
    raise SystemExit("exact existing restoration receipt was not replayed without write")

state["bytes"] = b"{}"
reject(
    lambda: module.publish_or_validate_pre_security_admin_restoration_receipt(
        manifest, manifest_bytes, module.sha_bytes(authorization_bytes)
    ),
    "an invalid existing restoration receipt",
)
if publication_calls or state["bytes"] != b"{}":
    raise SystemExit("invalid existing receipt was modified instead of preserved")

for name, value in saved_route.items():
    setattr(module, name, value)


# Alternate receipt-root residue is rejected without removal.
saved_children = module.pre_security_receipt_root_children
module.pre_security_receipt_root_children = lambda: [
    module.PRE_SECURITY_ADMIN_AUTHORIZATION_PATH.name,
    ".alternate-restoration.pending",
]
reject(module.validate_pre_security_receipt_root_state, "alternate temporary residue")
module.pre_security_receipt_root_children = saved_children


# The shared current-baseline producer rejects every named reappearance class before publication.
synthetic_artifact = "/Library/PrivilegedHelperTools/synthetic-r3-finalizer"
state_manifest = {
    "artifacts": [
        {
            "role": "synthetic_finalizer",
            "intended_path": synthetic_artifact,
            "signing_identifier": "com.atomize.synthetic",
        }
    ]
}
absence_paths = module.admin_restoration_absence_paths(state_manifest)
required = {
    synthetic_artifact,
    str(module.INSTALLED_CANDIDATE_MANIFEST_PATH),
    str(module.JOURNAL_ROOT),
    str(module.RUNNER_ROOT),
    str(module.PUBLISHER_ROOT),
    str(module.CREATOR_ROOT),
    str(module.PRECLAIM_PATH),
    str(module.COMPLETION_PATH),
    str(module.CLAIM_ROOT),
}
if not required.issubset(absence_paths):
    raise SystemExit("all-clean classifier omits a required installed-state class")

saved_observers = {
    name: getattr(module, name)
    for name in (
        "observe_platform_managed_socket_absence",
        "require_absent",
        "require_durable_absence",
        "run_command",
        "process_snapshot",
        "directory_identity",
    )
}
socket_observation = {
    "schema_owner": "synthetic",
}
module.observe_platform_managed_socket_absence = lambda _path: socket_observation
module.require_absent = lambda _path: None


def durable(path):
    return {
        "path": str(path),
        "parent_path": str(path.parent),
        "parent_device": 1,
        "parent_inode": len(str(path)) + 1,
        "parent_uid": 0,
        "parent_gid": 0,
        "parent_mode": stat.S_IFDIR | 0o755,
        "parent_fsync_return": 0,
        "child_fstatat_return": -1,
        "child_raw_errno": errno.ENOENT,
        "parent_reopened_same_stable_identity": True,
    }


module.require_durable_absence = durable
launchd_stderr = (
    f'Bad request.\nCould not find service "{module.FINALIZER_LABEL}" '
    "in domain for system\n"
).encode()
module.run_command = lambda _argv: SimpleNamespace(
    returncode=113, stdout=b"", stderr=launchd_stderr
)
module.process_snapshot = lambda: [
    {
        "pid": 1,
        "path": "/sbin/launchd",
        "classification": "path_observed",
        "raw_errno": 0,
    }
]
module.directory_identity = lambda *_args: expected_global
clean_observation = module.observe_admin_restoration_absence(
    state_manifest, expected_global
)
if clean_observation["platform_managed_socket_absence"] != socket_observation:
    raise SystemExit("all-clean classifier dropped the RCV-04 socket observation")
churned_observation = copy.deepcopy(clean_observation)
churned_observation["process_records"] = [
    {
        "pid": 999,
        "path": "/usr/bin/unrelated",
        "classification": "path_observed",
        "raw_errno": 0,
    }
]
churned_observation["process_snapshot_sha256"] = "9" * 64
if module.pre_security_admin_restoration_projection(
    clean_observation, state_manifest
) != module.pre_security_admin_restoration_projection(
    churned_observation, state_manifest
):
    raise SystemExit("unrelated process churn changed resumable restoration receipt bytes")
for path in (
    pathlib.Path(synthetic_artifact),
    module.INSTALLED_CANDIDATE_MANIFEST_PATH,
    module.INBOX_ROOT,
    module.JOURNAL_ROOT,
    module.RUNNER_ROOT,
    module.PUBLISHER_ROOT,
    module.CREATOR_ROOT,
    module.PRECLAIM_PATH,
    module.CLAIM_ROOT,
):
    module.require_absent = (
        lambda candidate, path=path: module.fail("synthetic reappearance")
        if candidate == path
        else None
    )
    reject(
        lambda: module.observe_admin_restoration_absence(
            state_manifest, expected_global
        ),
        f"reappearing {path}",
    )
module.require_absent = lambda _path: None
module.observe_platform_managed_socket_absence = lambda _path: module.fail(
    "synthetic socket reappearance"
)
reject(
    lambda: module.observe_admin_restoration_absence(state_manifest, expected_global),
    "reappearing socket",
)
module.observe_platform_managed_socket_absence = lambda _path: socket_observation
module.run_command = lambda _argv: SimpleNamespace(
    returncode=0, stdout=b"", stderr=b""
)
reject(
    lambda: module.observe_admin_restoration_absence(state_manifest, expected_global),
    "reappearing launchd service",
)
module.run_command = lambda _argv: SimpleNamespace(
    returncode=113, stdout=b"", stderr=launchd_stderr
)
module.process_snapshot = lambda: [
    {
        "pid": 55,
        "path": synthetic_artifact,
        "classification": "path_observed",
        "raw_errno": 0,
    }
]
reject(
    lambda: module.observe_admin_restoration_absence(state_manifest, expected_global),
    "matching experiment process",
)
for name, value in saved_observers.items():
    setattr(module, name, value)


# Retained exchange identity and emptiness use a stable descriptor and reject both drift classes.
saved_global = module.GLOBAL_EXCHANGE
synthetic_global = work_root / "global-exchange"
synthetic_global.mkdir(mode=0o700)
module.GLOBAL_EXCHANGE = synthetic_global
observed = os.stat(synthetic_global, follow_symlinks=False)
synthetic_expected = module.physical_identity(synthetic_global, observed)
empty = module.observe_retained_global_exchange_empty(synthetic_expected)
if empty["children"] != []:
    raise SystemExit("empty retained exchange was not accepted")
(synthetic_global / "reappeared").write_text("state")
reject(
    lambda: module.observe_retained_global_exchange_empty(synthetic_expected),
    "nonempty retained global exchange",
)
(synthetic_global / "reappeared").unlink()
changed_global = dict(synthetic_expected)
changed_global["inode"] += 1
reject(
    lambda: module.observe_retained_global_exchange_empty(changed_global),
    "changed retained global exchange identity",
)
module.GLOBAL_EXCHANGE = saved_global


# The sealed loader's generic stable reader rejects owner, mode, link, size, and link substitution;
# the receipt-only authorization loader additionally compares the exact sealed byte digest before
# parsing or state observation.
identity_root = work_root / "authorization-identity"
identity_root.mkdir(mode=0o700)
identity_leaf = identity_root / "authorization.json"
identity_leaf.write_bytes(b'{"authority":true}')
identity_leaf.chmod(0o400)
identity_stat = os.stat(identity_leaf, follow_symlinks=False)
identity_data, _ = module.read_bounded_exact_file(
    identity_leaf,
    identity_stat.st_uid,
    identity_stat.st_gid,
    0o400,
    len(b'{"authority":true}'),
)
if identity_data != b'{"authority":true}':
    raise SystemExit("canonical synthetic authority did not pass stable read")
reject(
    lambda: module.read_bounded_exact_file(
        identity_leaf,
        identity_stat.st_uid + 1,
        identity_stat.st_gid,
        0o400,
        len(identity_data),
    ),
    "wrong authorization owner",
)
reject(
    lambda: module.read_bounded_exact_file(
        identity_leaf,
        identity_stat.st_uid,
        identity_stat.st_gid,
        0o444,
        len(identity_data),
    ),
    "wrong authorization mode",
)
reject(
    lambda: module.read_bounded_exact_file(
        identity_leaf,
        identity_stat.st_uid,
        identity_stat.st_gid,
        0o400,
        len(identity_data) - 1,
    ),
    "oversize authorization",
)
identity_link = identity_root / "authorization-hardlink.json"
os.link(identity_leaf, identity_link)
reject(
    lambda: module.read_bounded_exact_file(
        identity_leaf,
        identity_stat.st_uid,
        identity_stat.st_gid,
        0o400,
        len(identity_data),
    ),
    "multiply linked authorization",
)
identity_link.unlink()
identity_symlink = identity_root / "authorization-symlink.json"
identity_symlink.symlink_to(identity_leaf)
reject(
    lambda: module.read_bounded_exact_file(
        identity_symlink,
        identity_stat.st_uid,
        identity_stat.st_gid,
        0o400,
        len(identity_data),
    ),
    "symlink authorization",
)
authorization_loader_source = ast.unparse(
    functions["load_pre_security_admin_authorization"]
)
digest_check = "sha_bytes(authorization_bytes) != expected_file_sha256"
parse_call = "parse_canonical(authorization_bytes"
if (
    digest_check not in authorization_loader_source
    or parse_call not in authorization_loader_source
    or authorization_loader_source.index(digest_check)
    > authorization_loader_source.index(parse_call)
):
    raise SystemExit("authorization loader does not reject the sealed digest before parsing")


# No-clobber publication rejoins exact prefixes at create/write/fsync/link/reopen states without
# producing a second final leaf. Invalid prefix bytes remain present and are rejected.
real_os_stat = module.os.stat


def root_owned_pending_stat(path, *args, **kwargs):
    observed = real_os_stat(path, *args, **kwargs)
    name = os.fspath(path)
    if name.startswith(".pre-security-admin-restoration-receipt"):
        values = {
            field: getattr(observed, field)
            for field in dir(observed)
            if field.startswith("st_")
        }
        values["st_uid"] = 0
        values["st_gid"] = 0
        return SimpleNamespace(**values)
    return observed


publication_bytes = module.canonical({"receipt": "fixed", "sequence": 1})
for index, prefix_length in enumerate(
    (0, 1, len(publication_bytes) // 2, len(publication_bytes))
):
    root = work_root / f"publication-{index}"
    root.mkdir(mode=0o700)
    final = root / "pre-security-admin-restoration-receipt.v2.json"
    temporary = module.pending_path(final)
    temporary.write_bytes(publication_bytes[:prefix_length])
    temporary.chmod(0o600)
    module.os.stat = root_owned_pending_stat
    module.publish_bytes_no_clobber(
        publication_bytes, final, os.getuid(), 0, 0o444
    )
    module.os.stat = real_os_stat
    if final.read_bytes() != publication_bytes or temporary.exists():
        raise SystemExit(f"publication prefix did not rejoin: {prefix_length}")
    if [entry.name for entry in root.iterdir()] != [final.name]:
        raise SystemExit("publication created a duplicate receipt leaf")

link_root = work_root / "publication-link"
link_root.mkdir(mode=0o700)
link_final = link_root / "pre-security-admin-restoration-receipt.v2.json"
link_temp = module.pending_path(link_final)
link_temp.write_bytes(publication_bytes)
link_temp.chmod(0o444)
os.link(link_temp, link_final)
module.os.stat = root_owned_pending_stat
module.publish_bytes_no_clobber(
    publication_bytes, link_final, os.getuid(), 0, 0o444
)
module.os.stat = real_os_stat
if link_temp.exists() or link_final.read_bytes() != publication_bytes:
    raise SystemExit("linked publication residue did not rejoin exactly")

invalid_root = work_root / "publication-invalid"
invalid_root.mkdir(mode=0o700)
invalid_final = invalid_root / "pre-security-admin-restoration-receipt.v2.json"
invalid_temp = module.pending_path(invalid_final)
invalid_temp.write_bytes(b"alternate")
invalid_temp.chmod(0o600)
module.os.stat = root_owned_pending_stat
reject(
    lambda: module.publish_bytes_no_clobber(
        publication_bytes, invalid_final, os.getuid(), 0, 0o444
    ),
    "alternate publication prefix",
)
module.os.stat = real_os_stat
if invalid_temp.read_bytes() != b"alternate" or invalid_final.exists():
    raise SystemExit("alternate publication residue was not preserved")

module.validate_pre_security_restoration_observation = saved_restoration_validator
print("R3-RCV-05 pre-Security receipt-only continuation regression: PASS")
PY
