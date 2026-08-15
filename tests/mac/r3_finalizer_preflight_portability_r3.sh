#!/bin/zsh
set -euo pipefail

readonly REPOSITORY="${0:A:h:h:h}"
readonly FREEZE_SOURCE="${REPOSITORY}/scripts/mac/freeze-r3-macos-finalizer-candidate.sh"
readonly PYTHON="/usr/bin/python3"

[[ -f "${FREEZE_SOURCE}" && ! -L "${FREEZE_SOURCE}" ]] \
    || { print -u2 -- "R3 preflight portability source is unavailable"; exit 1; }

readonly WORK_ROOT="$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/r3-preflight-portability.XXXXXX")"
cleanup() {
    /bin/chmod -R u+rwX "${WORK_ROOT}" 2>/dev/null || true
    /bin/rm -rf -- "${WORK_ROOT}"
}
trap cleanup EXIT

env -i PATH=/usr/bin:/bin:/usr/sbin:/sbin LANG=C LC_ALL=C TZ=UTC \
    PYTHONDONTWRITEBYTECODE=1 \
    "${PYTHON}" - "${FREEZE_SOURCE}" "${WORK_ROOT}" <<'PY'
import ast
import hashlib
import os
import pathlib
import stat
import subprocess
import sys


freeze_source = pathlib.Path(sys.argv[1])
work_root = pathlib.Path(sys.argv[2])
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


def string_assignment(name):
    matches = []
    for node in generator_tree.body:
        if not isinstance(node, ast.Assign) or len(node.targets) != 1:
            continue
        target = node.targets[0]
        if isinstance(target, ast.Name) and target.id == name:
            matches.append(ast.literal_eval(node.value))
    if len(matches) != 1 or not isinstance(matches[0], str):
        raise SystemExit(f"generator does not define one literal {name}")
    return matches[0]


loader_source = string_assignment("SEALED_RECOVERY_LOADER")
loader_tree = ast.parse(loader_source)
required_assignments = [
    node
    for node in loader_tree.body
    if isinstance(node, ast.Assign)
    and len(node.targets) == 1
    and isinstance(node.targets[0], ast.Name)
    and node.targets[0].id == "REQUIRED_ROOT_COMMANDS"
]
if len(required_assignments) != 1:
    raise SystemExit("sealed loader lacks one closed required-command preflight")
required_commands = ast.literal_eval(required_assignments[0].value)
if required_commands != ("/usr/bin/env", "/bin/zsh", "/usr/bin/python3"):
    raise SystemExit("sealed loader required-command plan changed")
preflight_functions = [
    node
    for node in loader_tree.body
    if isinstance(node, ast.FunctionDef) and node.name == "require_sealed_root_command"
]
if len(preflight_functions) != 1:
    raise SystemExit("sealed loader lacks one required-command preflight function")
preflight_calls = [
    f"{node.func.value.id}.{node.func.attr}"
    for node in ast.walk(preflight_functions[0])
    if isinstance(node, ast.Call)
    and isinstance(node.func, ast.Attribute)
    and isinstance(node.func.value, ast.Name)
]
if preflight_calls.count("os.lstat") != 2 or any(
    call in preflight_calls for call in ("os.open", "subprocess.run", "shutil.which")
):
    raise SystemExit("required-command preflight is not two stable lstat observations")
for forbidden in ("/usr/bin/test", "/bin/test", "command -v", "shutil.which"):
    if forbidden in generator_source or forbidden in loader_source:
        raise SystemExit(f"generated preflight retains forbidden shell lookup: {forbidden}")

# Generate, but never execute, the exact reviewed route so its outer propagation surface is tested.
synthetic_source = work_root / "synthetic-root-source.py"
synthetic_source.write_text("pass\n")
synthetic_source.chmod(0o644)
synthetic_admin_block = work_root / "synthetic-admin-block.txt"
generated = subprocess.run(
    [
        sys.executable,
        "-c",
        generator_source,
        str(synthetic_source),
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
route_text = synthetic_admin_block.read_text()
if "/usr/bin/test" in route_text or "/bin/test" in route_text:
    raise SystemExit("generated recovery/preflight surface retains test(1)")
sanitized_root = (
    "exec /usr/bin/sudo /usr/bin/env -i "
    "PATH=/usr/bin:/bin:/usr/sbin:/sbin LANG=C LC_ALL=C TZ=UTC"
)
if route_text.count(sanitized_root) != 1:
    raise SystemExit("generated recovery route changed its exact sanitized root environment")
if route_text.count("exec /usr/bin/sudo ") != 1 or route_text.count("/usr/bin/sudo ") != 1:
    raise SystemExit("generated recovery route is not one tail exec of sudo")
if "PASS" in route_text or "SUCCESS_SENTINEL" in route_text:
    raise SystemExit("generated recovery/preflight surface can mask a failure with PASS")


def expected(path):
    value = os.stat(path, follow_symlinks=False)
    return (
        hashlib.sha256(path.read_bytes()).hexdigest(),
        value.st_size,
        (
            value.st_dev,
            value.st_ino,
            value.st_uid,
            value.st_gid,
            stat.S_IMODE(value.st_mode),
            value.st_nlink,
            value.st_mtime_ns,
            value.st_ctime_ns,
        ),
    )


def invoke_loader(source_bytes, loader=loader_source):
    source = work_root / "loader-probe.py"
    if source.exists():
        source.chmod(0o600)
    source.write_bytes(source_bytes)
    source.chmod(0o400)
    digest, size, identity = expected(source)
    return subprocess.run(
        [
            sys.executable,
            "-c",
            loader,
            str(source),
            digest,
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


# Command-path absence and invalid executable posture must fail before the synthetic route source.
missing_command = work_root / "missing-required-command"
missing_loader = loader_source.replace("/usr/bin/env", str(missing_command), 1)
missing = invoke_loader(b"print('SOURCE-RAN')\n", missing_loader)
if (
    missing.returncode == 0
    or b"SOURCE-RAN" in missing.stdout
    or b"PASS" in missing.stdout
    or b"PASS" in missing.stderr
):
    raise SystemExit("missing required root command did not stop preflight nonzero")

non_executable = work_root / "non-executable-command"
non_executable.write_bytes(b"not executable\n")
non_executable.chmod(0o400)
invalid_loader = loader_source.replace("/usr/bin/env", str(non_executable), 1)
invalid = invoke_loader(b"print('SOURCE-RAN')\n", invalid_loader)
if (
    invalid.returncode == 0
    or b"SOURCE-RAN" in invalid.stdout
    or b"PASS" in invalid.stdout
    or b"PASS" in invalid.stderr
):
    raise SystemExit("required-command probe failure did not stop preflight nonzero")

failure_78 = invoke_loader(b"raise SystemExit(78)\n")
if failure_78.returncode != 78 or b"PASS" in failure_78.stdout or b"PASS" in failure_78.stderr:
    raise SystemExit(
        f"route exit 78 was changed or masked by PASS: "
        f"status={failure_78.returncode} stdout={failure_78.stdout!r} stderr={failure_78.stderr!r}"
    )

success = invoke_loader(b"print('EXACT-SUCCESS')\n")
if success.returncode != 0 or success.stdout.splitlines() != [b"EXACT-SUCCESS"] or success.stderr:
    raise SystemExit("successful preflight changed exact route success")

print("R3-RCV-06 macOS preflight portability regression: PASS")
PY
