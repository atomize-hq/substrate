#!/usr/bin/env bash
set -euo pipefail

# Run the complete uninstaller only against disposable prefixes. The selected
# shim-removal executable is a fixture; no service-removal option is supplied.
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
python3 - "${REPO_ROOT}" <<'PY'
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

repo = Path(sys.argv[1])
script = repo / 'scripts/substrate/dev-uninstall-substrate.sh'
with tempfile.TemporaryDirectory(prefix='dev-uninstall-gateway-') as temporary:
    root = Path(temporary)
    shim = root / 'shim-removal-fixture'
    shim_marker = root / 'shim-invoked'
    shim.write_text('#!/bin/sh\nprintf invoked > "' + str(shim_marker) + '"\n')
    shim.chmod(0o700)
    old_repo = root / 'old-checkout'
    for relative in ('Cargo.toml', 'Cargo.lock', 'src/main.rs',
                     'scripts/substrate/dev-install-substrate.sh',
                     'scripts/substrate/dev-uninstall-substrate.sh'):
        destination = old_repo / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(repo / relative, destination)
    (old_repo / '.git').mkdir()
    env = {key: value for key, value in os.environ.items()
           if not key.startswith('SUBSTRATE_')}

    def run(prefix, options=()):
        return subprocess.run(['bash', str(script), '--prefix', str(prefix),
                               '--bin', str(shim), *options], env=env, text=True,
                              capture_output=True, timeout=15)

    for selected, name in ((selected, name) for selected in (False, True)
                           for name in ('substrate-gateway', 'substrate-gateway.exe',
                                        'linux/substrate-gateway')):
        owner = old_repo if selected else repo
        for kind in ('managed', 'foreign', 'foreign-dangling', 'prefix-collision', 'file', 'directory'):
            label = name.replace('/', '-')
            prefix = root / f'{selected}-{label}-{kind}'
            binary = prefix / 'bin' / name
            binary.parent.mkdir(parents=True)
            if selected:
                # An authentic fixed-name link proves this prefix selected the
                # old checkout even when that checkout's build targets are gone.
                (prefix / 'bin/substrate').symlink_to(owner / 'target/debug/substrate')
            if kind == 'managed':
                # Managed links remain removable when their build target is gone.
                binary.symlink_to(owner / 'target/debug' / Path(name).name)
            elif kind == 'foreign':
                target = root / f'foreign-{label}'
                target.write_text('foreign target sentinel\n')
                binary.symlink_to(target)
            elif kind == 'foreign-dangling':
                binary.symlink_to(root / 'absent-foreign-target')
            elif kind == 'prefix-collision':
                binary.symlink_to(Path(str(owner) + '-foreign') / 'target/debug' / Path(name).name)
            elif kind == 'file':
                binary.write_text('unmanaged binary sentinel\n')
            else:
                binary.mkdir()
                (binary / 'sentinel').write_text('unmanaged directory sentinel\n')
            before = binary.lstat()
            link = os.readlink(binary) if binary.is_symlink() else None
            options = ['--managed-source-root', str(owner)] if selected else []
            result = run(prefix, options)
            if kind == 'managed':
                assert result.returncode == 0, (name, kind, result)
                assert not binary.exists() and not binary.is_symlink(), (name, kind)
                assert f'Removing managed symlink {binary}' in result.stdout, result
            else:
                assert result.returncode == 5, (name, kind, result)
                assert f'Preserving protected path {binary}' in result.stderr, result
                after = binary.lstat()
                assert (after.st_dev, after.st_ino, after.st_mode) == (before.st_dev, before.st_ino, before.st_mode)
                if link is not None:
                    assert os.readlink(binary) == link
                elif kind == 'file':
                    assert binary.read_text() == 'unmanaged binary sentinel\n'
                else:
                    assert (binary / 'sentinel').read_text() == 'unmanaged directory sentinel\n'
                if kind == 'foreign':
                    assert target.read_text() == 'foreign target sentinel\n'
            print(f'PASS {"selected" if selected else "default"} {name}: {kind}')

    # Do not infer authority from a lookalike repository or normalize an alias.
    alias = root / 'old-alias'
    alias.symlink_to(old_repo, target_is_directory=True)
    plain_file = root / 'not-directory'
    plain_file.write_text('not a checkout\n')
    no_layout = root / 'no-layout'
    no_layout.mkdir()
    for label, selection, evidence in (
            ('empty', '', True), ('root', '/', True),
            ('missing', str(root / 'absent'), True), ('file', str(plain_file), True),
            ('relative', 'relative-checkout', True), ('alias', str(alias), True),
            ('traversal', str(old_repo) + '/../old-checkout', True),
            ('trailing-slash', str(old_repo) + '/', True),
            ('non-substrate', str(no_layout), True),
            ('no-owned-link', str(old_repo), False)):
        prefix = root / ('invalid-' + label)
        (prefix / 'bin').mkdir(parents=True)
        if evidence:
            (prefix / 'bin/substrate').symlink_to(old_repo / 'target/debug/substrate')
        else:
            (prefix / 'bin/substrate').symlink_to(root / 'foreign-substrate')
        config = prefix / 'config.yaml'
        config.write_text('configuration sentinel\n')
        before = config.stat()
        shim_marker.unlink(missing_ok=True)
        result = run(prefix, ['--managed-source-root', selection])
        assert result.returncode != 0, (label, result)
        assert 'Unable to select managed source root' in result.stderr, (label, result)
        assert not shim_marker.exists(), label
        assert config.read_text() == 'configuration sentinel\n'
        assert config.stat().st_ino == before.st_ino
        assert (prefix / 'bin/substrate').is_symlink()
        print('PASS invalid selector rejected before shim/removal:', label)

    # Selection must not execute the selected checkout's binary, and links to
    # the invoking checkout remain foreign under an explicit different owner.
    prefix = root / 'cross-owner'
    (prefix / 'bin').mkdir(parents=True)
    (prefix / 'bin/substrate').symlink_to(old_repo / 'target/debug/substrate')
    (prefix / 'bin/substrate-gateway').symlink_to(repo / 'target/debug/substrate-gateway')
    result = run(prefix, ['--managed-source-root', str(old_repo)])
    assert result.returncode == 5 and shim_marker.exists(), result
    assert not (prefix / 'bin/substrate').is_symlink()
    assert (prefix / 'bin/substrate-gateway').is_symlink()
    assert f'Preserving protected path {prefix / "bin/substrate-gateway"}' in result.stderr
    print('PASS selected ownership and explicit execution stay separate')

    invoking_repo = root / 'invoking-checkout'
    invoking_script = invoking_repo / 'scripts/substrate/dev-uninstall-substrate.sh'
    invoking_script.parent.mkdir(parents=True)
    shutil.copyfile(script, invoking_script)
    invoking_bin = invoking_repo / 'target/debug/substrate'
    invoking_bin.parent.mkdir(parents=True)
    shutil.copyfile(shim, invoking_bin)
    invoking_bin.chmod(0o700)
    forbidden = root / 'old-binary-executed'
    old_bin = old_repo / 'target/debug/substrate'
    old_bin.parent.mkdir(parents=True)
    old_bin.write_text('#!/bin/sh\nprintf forbidden > "' + str(forbidden) + '"\n')
    old_bin.chmod(0o700)
    prefix = root / 'default-execution'
    (prefix / 'bin').mkdir(parents=True)
    (prefix / 'bin/substrate').symlink_to(old_bin)
    shim_marker.unlink()
    result = subprocess.run(['bash', str(invoking_script), '--prefix', str(prefix),
                             '--managed-source-root', str(old_repo)], env=env,
                            text=True, capture_output=True, timeout=15)
    assert result.returncode == 0 and shim_marker.exists(), result
    assert not forbidden.exists()
    print('PASS default execution uses invoking checkout, never selected owner')

    # Exercise the actual helper-discovery function with fixture ps/kill/sleep;
    # no real process is signalled and the selected root alone matches by source.
    source = script.read_text()
    helper = source.split('kill_live_dev_owner_helpers() {\n', 1)[1].split('\nPREFIX=""', 1)[0]
    driver = '''set -euo pipefail
log() { :; }
warn() { :; }
KILL_LIVE_PROCESSES=1
REPO_ROOT="$1"
MANAGED_SOURCE_ROOT="$2"
PREFIX="$3"
VERSION_DIR="$PREFIX/versions/dev"
ps() {
  printf '111 %s/target/debug/substrate agent __owner-helper --fixture\n' "$MANAGED_SOURCE_ROOT"
  printf '222 %s/target/debug/substrate agent __owner-helper --fixture\n' "$REPO_ROOT"
  printf '333 %s-foreign/target/debug/substrate agent __owner-helper --fixture\n' "$MANAGED_SOURCE_ROOT"
}
kill() {
  [[ "$1" != -0 ]] || return 1
  printf 'signal %s\n' "$*"
}
sleep() { :; }
kill_live_dev_owner_helpers() {
''' + helper + '\nkill_live_dev_owner_helpers\n'
    result = subprocess.run(['bash', '-c', driver, 'fixture', str(invoking_repo),
                             str(old_repo), str(root / 'helpers-prefix')], env=env,
                            text=True, capture_output=True, timeout=15)
    assert result.returncode == 0 and result.stdout == 'signal 111\n', result
    print('PASS helper discovery uses selected exact owner, not invoking/colliding roots')
print('PASS 36 gateway ownership + 10 invalid selector + 3 execution/ownership cases')
PY
