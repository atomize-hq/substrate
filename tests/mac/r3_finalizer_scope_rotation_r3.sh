#!/bin/zsh
set -euo pipefail

readonly REPOSITORY="$(/usr/bin/git rev-parse --show-toplevel)"
readonly PYTHON="/usr/bin/python3"

env -i PATH=/usr/bin:/bin:/usr/sbin:/sbin LANG=C LC_ALL=C TZ=UTC \
    "${PYTHON}" - "${REPOSITORY}" <<'PY'
import ast
import hashlib
import pathlib
import re
import sys

repository = pathlib.Path(sys.argv[1])
new_experiment = "01a007ef-e124-7f0e-b711-0d023dfd7d52"
new_scopes = [
    "01a007ef-e126-757e-b1c9-b9990aef00e9",
    "01a007ef-e128-7247-93d0-a3a05621da2f",
]
old_experiment = "019ffec6-95f6-7d30-80bc-8003ce27d5ba"
old_scopes = [
    "019ffeb5-b252-79ae-8f41-e161419fbbcd",
    "019ffeb5-b255-75a5-870f-49323ebb2c19",
]
terminal_experiments = [
    old_experiment,
    "01a0033f-9fa7-700c-b0b3-4ad0a8ed372b",
    "01a003da-91f6-7710-b86b-dc28c771a773",
    "01a006ed-1dbc-7d16-b88d-a2b1818ce801",
    "01a00732-3426-7a57-b0e7-9d673bd19b59",
]
terminal_scopes = [
    *old_scopes,
    "01a0033f-9faa-75e4-afb2-f96731adb7de",
    "01a0033f-9fac-79f4-a137-391728ab2f76",
    "01a003da-91f9-712b-b261-fe3be9b5550d",
    "01a003da-91fc-7aad-83e2-75cd7b026fe7",
    "01a006ed-1dbd-78a2-8d18-1f9c039634da",
    "01a006ed-1dbe-7ea2-a3e9-81225eba65f7",
    "01a00732-3429-76c8-83f9-b9fb165252c1",
    "01a00732-342b-7918-9c9c-ed777d5c9e22",
]
expected_routes = {
    "deea3cb3aaf05dd641936bdb98bcc2d3098504d64b455794af689cabe68cd76a",
    "bd98dc77cfde7f6956f43845294edeb9da5879525def1bcef5d6c8a3133d839a",
    "7eb9e26e6fb61b10297b85d1b925b980de77c89cc3fe53bc022c91ca04d723c9",
    "f9959128723ed4b18ad9ed7960dbb63788f51f386532265321940cf8e29b141e",
}
expected_sources = {
    "4ae8266e919e0d202e785009695f61820467d73b3836adf9ac92741ac5624698",
    "5416f9df95e34e480f3f45460c500d8f72ed287b8b21e915b8968f09091c69fd",
    "d07da7afca829c710fd4f5416250df661a6b9699ea9a2938c93f8443181e1ae3",
    "e808fc84170cd43b7b62872f7b0479157e727b3fce31f25e140d2c3364ce868c",
    "f962f6777dc5bc7a327f1c30b406851aaca7151b492e9ba38802ffd565566dc9",
}


def text(relative):
    return (repository / relative).read_text()


def python_assignment(relative, name):
    tree = ast.parse(text(relative))
    values = [
        node.value
        for node in tree.body
        if isinstance(node, ast.Assign)
        and len(node.targets) == 1
        and isinstance(node.targets[0], ast.Name)
        and node.targets[0].id == name
    ]
    if len(values) != 1:
        raise SystemExit(f"{relative} does not define exactly one {name}")
    return ast.literal_eval(values[0])


uuid_v7 = re.compile(
    r"^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$"
)
for value in [new_experiment, *new_scopes]:
    if not uuid_v7.fullmatch(value):
        raise SystemExit(f"rotated identity is not canonical UUIDv7: {value}")
if len({new_experiment, *new_scopes, *terminal_experiments, *terminal_scopes}) != 18:
    raise SystemExit("rotated and terminal identities are not pairwise distinct")

root_source = text("scripts/mac/r3-macos-finalizer-root-install.py")
if python_assignment("scripts/mac/r3-macos-finalizer-root-install.py", "EXPERIMENT_ID") != new_experiment:
    raise SystemExit("root installer retained the terminal experiment identity")
if python_assignment("scripts/mac/r3-macos-finalizer-root-install.py", "SCOPES") != new_scopes:
    raise SystemExit("root installer scope order differs from the rotated repetitions")
if any(value in root_source for value in [*terminal_experiments, *terminal_scopes]):
    raise SystemExit("root installer can still authorize a terminal candidate identity")
if "not is_git_id(manifest.get(\"source_commit\"))" not in root_source:
    raise SystemExit("root installer does not bind the committed manifest identity")

experiment_source = text("tools/r3-macos-finalizer/src/experiment/mod.rs")
if f'pub const EXPERIMENT_ID_V2: &str = "{new_experiment}";' not in experiment_source:
    raise SystemExit("finalizer crate experiment identity did not rotate")
for scope in new_scopes:
    if experiment_source.count(f'"{scope}"') != 1:
        raise SystemExit(f"finalizer crate does not compile exactly one scope join: {scope}")
for scope in terminal_scopes:
    refusal = f'"{scope}"'
    if experiment_source.count(refusal) != 1:
        raise SystemExit(f"terminal scope is not confined to one refusal regression: {scope}")

signer_source = text("tools/r3-macos-signer-acl/src/lib.rs")
for scope in new_scopes:
    if signer_source.count(f'"{scope}"') != 1:
        raise SystemExit(f"signer crate does not compile exactly one scope constant: {scope}")
for scope in terminal_scopes:
    if scope in signer_source:
        raise SystemExit(f"signer crate retains a terminal scope: {scope}")
if "repetition.finalizer_scope()" not in signer_source:
    raise SystemExit("signer tags do not derive from the rotated finalizer scope")

freeze_source = text("scripts/mac/freeze-r3-macos-finalizer-candidate.sh")
for required in (
    f'readonly EXPERIMENT_ID="{new_experiment}"',
    'readonly BRANCH="feat/r3-macos-finalizer-rcv-stack"',
    'readonly ACCEPTED_PARENT_HEAD="c0ddc8e57cbbc61b7b9a7d60eb941db486c18fbb"',
    '"schema_owner": "substrate.r3-macos-candidate-committed-tree-inventory"',
    '"status", "--porcelain=v2", "-z"',
    '"rev-parse", "HEAD^{tree}"',
):
    if required not in freeze_source:
        raise SystemExit(f"committed-tree freeze binding is incomplete: {required}")
if any(value in freeze_source for value in [*terminal_experiments, *terminal_scopes]):
    raise SystemExit("freeze generator retains a terminal candidate identity")

start = freeze_source.index("# R3_SEALED_RECOVERY_ROUTE_GENERATOR_V1_BEGIN")
end = freeze_source.index("# R3_SEALED_RECOVERY_ROUTE_GENERATOR_V1_END")
generator = freeze_source[start:end].split("<<'PY'\n", 1)[1].rsplit("\nPY", 1)[0]
generator_tree = ast.parse(generator)


def generator_set(name):
    values = [
        node.value
        for node in generator_tree.body
        if isinstance(node, ast.Assign)
        and len(node.targets) == 1
        and isinstance(node.targets[0], ast.Name)
        and node.targets[0].id == name
    ]
    if len(values) != 1 or not isinstance(values[0], ast.Call):
        raise SystemExit(f"sealed route generator lacks one {name}")
    return ast.literal_eval(values[0].args[0])


routes = generator_set("TERMINAL_EXECUTED_RECOVERY_ROUTE_SHA256S")
sources = generator_set("TERMINAL_EXECUTED_RECOVERY_SOURCE_SHA256S")
if routes != expected_routes or sources != expected_sources or routes & sources:
    raise SystemExit("terminal route/source refusal sets are incomplete or conflated")

fixture = repository / "tests/mac/fixtures/r3_finalizer_cleanup_record_actual_v2.json"
if hashlib.sha256(fixture.read_bytes()).hexdigest() != "8e6aeaf410d5038da2d48288b1445da08a0a21c4d98442744fb6d8a5f925c6f3":
    raise SystemExit("historical actual cleanup fixture changed during scope rotation")
if fixture.read_text().count(old_experiment) != 3 or new_experiment in fixture.read_text():
    raise SystemExit("historical actual fixture was rewritten into the corrected identity")

print("R3-RCV-E02 macOS candidate scope rotation regression: PASS")
PY
