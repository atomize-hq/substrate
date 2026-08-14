#!/bin/zsh
set -euo pipefail

readonly REPOSITORY="${0:A:h:h:h}"
readonly FREEZE_SOURCE="${REPOSITORY}/scripts/mac/freeze-r3-macos-finalizer-candidate.sh"
readonly ROOT_SOURCE="${REPOSITORY}/scripts/mac/r3-macos-finalizer-root-install.py"
readonly PYTHON="/usr/bin/python3"

[[ -f "${FREEZE_SOURCE}" && ! -L "${FREEZE_SOURCE}" ]] \
    || { print -u2 -- "sealed recovery fixture source is unavailable"; exit 1; }
[[ -f "${ROOT_SOURCE}" && ! -L "${ROOT_SOURCE}" ]] \
    || { print -u2 -- "root membrane fixture source is unavailable"; exit 1; }

readonly WORK_ROOT="$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/r3-sealed-recovery.XXXXXX")"
cleanup() {
    /bin/chmod -R u+rwX "${WORK_ROOT}" 2>/dev/null || true
    /bin/rm -rf -- "${WORK_ROOT}"
}
trap cleanup EXIT

env -i PATH=/usr/bin:/bin:/usr/sbin:/sbin LANG=C LC_ALL=C TZ=UTC \
    PYTHONDONTWRITEBYTECODE=1 \
    "${PYTHON}" - "${FREEZE_SOURCE}" "${ROOT_SOURCE}" "${WORK_ROOT}" <<'PY'
import ast
import hashlib
import importlib.util
import os
import pathlib
import signal
import stat
import subprocess
import sys

freeze_source = pathlib.Path(sys.argv[1])
root_source = pathlib.Path(sys.argv[2])
work_root = pathlib.Path(sys.argv[3])
freeze_text = freeze_source.read_text()
begin = "# R3_SEALED_RECOVERY_ROUTE_GENERATOR_V1_BEGIN"
end = "# R3_SEALED_RECOVERY_ROUTE_GENERATOR_V1_END"
if freeze_text.count(begin) != 1 or freeze_text.count(end) != 1:
    raise SystemExit("sealed recovery generator markers are not unique")
generator_shell = freeze_text.split(begin, 1)[1].split(end, 1)[0]
heredoc_start = "<<'PY'\n"
if generator_shell.count(heredoc_start) != 1 or not generator_shell.rstrip().endswith("PY"):
    raise SystemExit("sealed recovery generator heredoc is not closed")
generator_source = generator_shell.split(heredoc_start, 1)[1].rsplit("\nPY", 1)[0]
generator_tree = ast.parse(generator_source)

# Execute only the effect-free generator against a synthetic source. This proves its shell route
# renders cleanly without executing sudo, root code, or any recovery/admin route.
synthetic_generator_source = work_root / "synthetic-root-source.py"
synthetic_generator_source.write_text("raise SystemExit(78)\n")
synthetic_generator_source.chmod(0o644)
synthetic_admin_block = work_root / "synthetic-admin-block.txt"
generated = subprocess.run(
    [
        sys.executable,
        "-c",
        generator_source,
        str(synthetic_generator_source),
        str(synthetic_admin_block),
        "a" * 64,
        str(os.getuid()),
        str(os.getgid()),
    ],
    cwd="/",
    env={"PATH": "/usr/bin:/bin:/usr/sbin:/sbin", "LANG": "C", "LC_ALL": "C", "TZ": "UTC"},
    stdin=subprocess.DEVNULL,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    check=False,
)
if generated.returncode != 0 or not synthetic_admin_block.is_file():
    raise SystemExit(f"effect-free recovery route generation failed: {generated.stderr!r}")
syntax = subprocess.run(
    ["/bin/zsh", "-n", str(synthetic_admin_block)],
    cwd="/",
    env={"PATH": "/usr/bin:/bin:/usr/sbin:/sbin", "LANG": "C", "LC_ALL": "C", "TZ": "UTC"},
    stdin=subprocess.DEVNULL,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    check=False,
)
if syntax.returncode != 0:
    raise SystemExit(f"generated recovery route is not valid zsh: {syntax.stderr!r}")


def literal_assignment(name):
    values = [
        node.value
        for node in generator_tree.body
        if isinstance(node, ast.Assign)
        and len(node.targets) == 1
        and isinstance(node.targets[0], ast.Name)
        and node.targets[0].id == name
    ]
    if len(values) != 1:
        raise SystemExit(f"generator does not define exactly one {name}")
    value = values[0]
    if isinstance(value, ast.Call) and isinstance(value.func, ast.Name) and value.func.id == "frozenset":
        if len(value.args) != 1:
            raise SystemExit(f"{name} frozenset is noncanonical")
        value = value.args[0]
    return ast.literal_eval(value)


loader = literal_assignment("SEALED_RECOVERY_LOADER")
terminal_routes = literal_assignment("TERMINAL_EXECUTED_RECOVERY_ROUTE_SHA256S")
terminal_sources = literal_assignment("TERMINAL_EXECUTED_RECOVERY_SOURCE_SHA256S")
expected_terminal_routes = {
    "deea3cb3aaf05dd641936bdb98bcc2d3098504d64b455794af689cabe68cd76a",
    "bd98dc77cfde7f6956f43845294edeb9da5879525def1bcef5d6c8a3133d839a",
}
if terminal_routes != expected_terminal_routes:
    raise SystemExit("executed recovery-route terminal set changed")
if terminal_sources != {
    "4ae8266e919e0d202e785009695f61820467d73b3836adf9ac92741ac5624698"
}:
    raise SystemExit("executed recovery-source terminal set changed")
unknown = "0" * 64
if unknown in terminal_routes or unknown in terminal_sources:
    raise SystemExit("unknown unexecuted digest was falsely classified terminal")
route_check = 'if is_terminal_executed_recovery_route(route_sha256):'
source_check = 'if source_sha256 in TERMINAL_EXECUTED_RECOVERY_SOURCE_SHA256S:'
publish = 'admin_fd = os.open(admin_block, admin_flags, 0o400)'
if not (
    generator_source.index(source_check)
    < generator_source.index(route_check)
    < generator_source.index(publish)
):
    raise SystemExit("terminal digest classification does not precede route publication")

loader_tree = ast.parse(loader)
calls = [node for node in ast.walk(loader_tree) if isinstance(node, ast.Call)]


def dotted_name(value):
    if isinstance(value, ast.Name):
        return value.id
    if isinstance(value, ast.Attribute):
        prefix = dotted_name(value.value)
        return f"{prefix}.{value.attr}" if prefix else value.attr
    return ""


open_calls = [call for call in calls if dotted_name(call.func) == "os.open"]
if len(open_calls) != 1:
    raise SystemExit("sealed recovery loader does not perform exactly one descriptor open")
if "flags = os.O_RDONLY | os.O_NOFOLLOW" not in loader:
    raise SystemExit("sealed recovery loader open lacks O_RDONLY|O_NOFOLLOW")
if any(dotted_name(call.func) in {"open", "pathlib.Path.read_bytes", "pathlib.Path.read_text"} for call in calls):
    raise SystemExit("sealed recovery loader contains an alternate pathname read")
compile_calls = [call for call in calls if dotted_name(call.func) == "compile"]
if len(compile_calls) != 1 or ast.unparse(compile_calls[0].args[0]) != "source_bytes":
    raise SystemExit("sealed recovery loader does not compile the descriptor-read bytes")
exec_calls = [call for call in calls if dotted_name(call.func) == "exec"]
if len(exec_calls) != 1 or ast.unparse(exec_calls[0].args[0]) != "compiled":
    raise SystemExit("sealed recovery loader does not execute the compiled descriptor bytes")
hash_calls = [call for call in calls if dotted_name(call.func) == "hashlib.sha256"]
if len(hash_calls) != 1 or ast.unparse(hash_calls[0].args[0]) != "source_bytes":
    raise SystemExit("sealed recovery loader hashes bytes other than its descriptor read")


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


def expected(path):
    observed = os.stat(path, follow_symlinks=False)
    data = path.read_bytes()
    return hashlib.sha256(data).hexdigest(), len(data), physical_identity(observed)


def invoke(path, sha256, size, identity):
    return subprocess.run(
        [
            sys.executable,
            "-c",
            loader,
            str(path),
            sha256,
            str(size),
            *(str(value) for value in identity),
        ],
        cwd="/",
        env={"PATH": "/usr/bin:/bin:/usr/sbin:/sbin", "LANG": "C", "LC_ALL": "C", "TZ": "UTC"},
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def create(name, source):
    path = work_root / name
    path.write_bytes(source)
    path.chmod(0o400)
    return path


for status_value in (0, 1, 78):
    path = create(f"exit-{status_value}.py", f"raise SystemExit({status_value})\n".encode())
    digest, size, identity = expected(path)
    result = invoke(path, digest, size, identity)
    if result.returncode != status_value:
        raise SystemExit(
            f"sealed loader changed exit {status_value} to {result.returncode}: {result.stderr!r}"
        )

signal_path = create(
    "signal.py",
    b"import os, signal\nos.kill(os.getpid(), signal.SIGTERM)\n",
)
digest, size, identity = expected(signal_path)
result = invoke(signal_path, digest, size, identity)
if result.returncode != -signal.SIGTERM:
    raise SystemExit(f"sealed loader did not preserve signal termination: {result.returncode}")

# The source can replace its pathname only after execution begins. Returning 7 proves the bytes
# already compiled from the one descriptor read remain the bytes that execute.
same_bytes = create(
    "same-bytes.py",
    (
        "import os\n"
        "os.unlink(__file__)\n"
        "with open(__file__, 'w') as replacement:\n"
        "    replacement.write('raise SystemExit(99)\\n')\n"
        "raise SystemExit(7)\n"
    ).encode(),
)
digest, size, identity = expected(same_bytes)
result = invoke(same_bytes, digest, size, identity)
if result.returncode != 7:
    raise SystemExit("sealed loader executed bytes from a second pathname read")

marker = work_root / "must-not-execute"
guard_source = (
    f"import pathlib\npathlib.Path({str(marker)!r}).write_text('executed')\n"
    "raise SystemExit(0)\n# padding padding padding\n"
).encode()


def require_pre_execution_rejection(label, path, digest, size, identity):
    marker.unlink(missing_ok=True)
    result = invoke(path, digest, size, identity)
    if result.returncode == 0 or marker.exists():
        raise SystemExit(f"{label} was not rejected before recovery-source execution")


original = create("original.py", guard_source)
original_digest, original_size, original_identity = expected(original)
substitute = create("substitute.py", guard_source + b"# alternate\n")
require_pre_execution_rejection(
    "path/source substitution", substitute, original_digest, original_size, original_identity
)

link = work_root / "source-link.py"
link.symlink_to(original)
require_pre_execution_rejection(
    "symlink source", link, original_digest, original_size, original_identity
)

require_pre_execution_rejection(
    "hash mismatch", original, "f" * 64, original_size, original_identity
)
require_pre_execution_rejection(
    "smaller declared size", original, original_digest, original_size - 1, original_identity
)
require_pre_execution_rejection(
    "larger declared size", original, original_digest, original_size + 1, original_identity
)

truncated = create("truncated.py", guard_source)
digest, size, identity = expected(truncated)
truncated.chmod(0o600)
truncated.write_bytes(guard_source[:-1])
truncated.chmod(0o400)
require_pre_execution_rejection("truncation", truncated, digest, size, identity)

grown = create("grown.py", guard_source)
digest, size, identity = expected(grown)
grown.chmod(0o600)
grown.write_bytes(guard_source + b"# growth\n")
grown.chmod(0o400)
require_pre_execution_rejection("growth", grown, digest, size, identity)

identity_changed = create("identity-change.py", guard_source)
digest, size, identity = expected(identity_changed)
replacement = create("identity-replacement.py", guard_source)
os.replace(replacement, identity_changed)
require_pre_execution_rejection("physical identity change", identity_changed, digest, size, identity)

spec = importlib.util.spec_from_file_location("r3_root_exit_integrity", root_source)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
entrypoint_driver = (
    "import importlib.util,sys\n"
    "spec=importlib.util.spec_from_file_location('root_membrane',sys.argv[1])\n"
    "module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)\n"
    "status=int(sys.argv[2]);validated=sys.argv[3]=='true'\n"
    "module.main=lambda:(status,validated)\n"
    "module.run_entrypoint()\n"
)


def invoke_entrypoint(status_value, validated):
    return subprocess.run(
        [sys.executable, "-c", entrypoint_driver, str(root_source), str(status_value), str(validated).lower()],
        cwd="/",
        env={"PATH": "/usr/bin:/bin:/usr/sbin:/sbin", "LANG": "C", "LC_ALL": "C", "TZ": "UTC"},
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


for status_value in (1, 78):
    result = invoke_entrypoint(status_value, False)
    if result.returncode != status_value or module.SUCCESS_SENTINEL.encode() in result.stdout:
        raise SystemExit(f"root entrypoint changed exit {status_value} or falsely emitted PASS")
result = invoke_entrypoint(-signal.SIGTERM, False)
if result.returncode != -signal.SIGTERM or module.SUCCESS_SENTINEL.encode() in result.stdout:
    raise SystemExit("root entrypoint did not preserve exact signal termination")
result = invoke_entrypoint(0, True)
if result.returncode != 0 or result.stdout.splitlines() != [module.SUCCESS_SENTINEL.encode()]:
    raise SystemExit("root entrypoint omitted PASS after validated exact success")
result = invoke_entrypoint(0, False)
if result.returncode != 78 or module.SUCCESS_SENTINEL.encode() in result.stdout:
    raise SystemExit("root entrypoint accepted success without a validated success receipt")
result = invoke_entrypoint(1, True)
if result.returncode != 78 or module.SUCCESS_SENTINEL.encode() in result.stdout:
    raise SystemExit("root entrypoint accepted a success receipt with nonzero route status")

root_text = root_source.read_text()
if root_text.count("SUCCESS_SENTINEL =") != 1 or root_text.count("print(SUCCESS_SENTINEL") != 1:
    raise SystemExit("PASS sentinel emission is not one closed entrypoint operation")
if "complete_successful_admin_cleanup(\n            manifest, manifest_bytes, root_install_authority_sha256\n        )\n        return 0, True" not in root_text:
    raise SystemExit("existing-success route does not bind PASS eligibility to receipt validation")
if "return result.returncode, result.returncode == 0" not in root_text:
    raise SystemExit("executed route status is not paired with its validated success receipt")

print("R3 sealed recovery loader and exit-integrity regressions passed")
PY
