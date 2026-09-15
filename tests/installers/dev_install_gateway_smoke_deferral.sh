#!/usr/bin/env bash
set -euo pipefail

# Only isolated parsing and the actual Linux provision_args dispatch region run.
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
python3 - "${REPO_ROOT}" <<'PY'
import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

repo = Path(sys.argv[1])
source = (repo / 'scripts/substrate/dev-install-substrate.sh').read_text()
# Source copies belong on persistent storage, never /tmp or a retained target.
parent = Path(os.environ.get('SUBSTRATE_TEST_TMPDIR', repo.parent)).resolve()
assert subprocess.check_output(['stat', '-f', '-c', '%T', str(parent)], text=True).strip() != 'tmpfs'
with tempfile.TemporaryDirectory(prefix='dev-gateway-deferral-', dir=parent) as temporary:
    root = Path(temporary)
    stub = root / 'bin'
    stub.mkdir()
    events = root / 'events.jsonl'
    script = root / 'dev-install.sh'
    script.write_text(source)
    python = shutil.which('python3')
    bash = shutil.which('bash')
    assert python and bash
    for tool in ('cat',):
        (stub / tool).symlink_to(shutil.which(tool))
    dispatcher = stub / 'dispatch'
    dispatcher.write_text('''#!'''+python+'''
import json, os, sys
from pathlib import Path
name = Path(sys.argv[0]).name
args = sys.argv[1:]
if name == 'uname':
    print(os.environ['TEST_HOST'])
elif name == 'grep' and args == ['-qi', 'microsoft', '/proc/version']:
    sys.exit(0 if os.environ['TEST_WSL'] == '1' else 1)
else:
    with open(os.environ['TEST_EVENTS'], 'a') as stream:
        stream.write(json.dumps({'command': name, 'args': args, 'context': {k: v for k, v in os.environ.items() if k.startswith('SUBSTRATE_')}})+'\\n')
    if name == 'python3' and os.environ.get('TEST_RESOLVE') == '1':
        sys.stdin.read()
        for value in json.loads(os.environ['TEST_CONTEXT_VALUES']):
            sys.stdout.buffer.write(value.encode()+b'\\0')
    elif name != 'provision':
        sys.exit(99)
''')
    dispatcher.chmod(0o755)
    # Any context resolution, build, privilege, account, network or install
    # command during rejection is a recorded failure, with no real fallback.
    for tool in ('uname', 'grep', 'python3', 'cargo', 'rustup', 'sudo', 'install',
                 'systemctl', 'curl', 'wget', 'getent', 'id', 'mkdir', 'cp',
                 'mv', 'chmod', 'apt-get', 'dnf', 'yum', 'pacman', 'zypper', 'provision'):
        (stub / tool).symlink_to(dispatcher)

    def run(args, host='Linux', wsl='0', selected=script, extra=None):
        events.write_text('')
        env = {'PATH': str(stub), 'HOME': str(root / 'ambient-home'),
               'TEST_EVENTS': str(events), 'TEST_HOST': host, 'TEST_WSL': wsl,
               'LANG': 'C', 'LC_ALL': 'C'}
        env.update(extra or {})
        result = subprocess.run([bash, str(selected), *args], env=env,
                                text=True, capture_output=True, timeout=10)
        calls = [json.loads(line) for line in events.read_text().splitlines()]
        print('CASE ' + json.dumps({'args': args, 'host': host, 'wsl': wsl,
                                   'exit_code': result.returncode, 'calls': calls,
                                   'stdout': result.stdout, 'stderr': result.stderr}, sort_keys=True))
        return result, calls

    def reject(args, diagnostic, host='Linux', wsl='0'):
        result, calls = run(args, host, wsl)
        assert result.returncode != 0 and diagnostic in result.stderr, (args, result)
        assert not calls, (args, 'side effect before rejection', calls)
        print('PASS reject before context/build/install:', host, wsl, *args)

    for option in ('--help', '-h'):
        result, calls = run([option])
        assert result.returncode == 0 and '--skip-gateway-smoke' in result.stdout, result
        assert 'maintainer' in result.stdout.lower() and not calls
    print('PASS help conventions')
    reject(['--unknown-deferral-option'], 'Unknown argument: --unknown-deferral-option')
    reject(['--skip-gateway-smoke=yes'], 'Unknown argument: --skip-gateway-smoke=yes')
    for args in (['--skip-gateway-smoke', '--no-world'], ['--no-world', '--skip-gateway-smoke']):
        reject(args, '--no-world')
    for host in ('Darwin', 'MINGW64_NT-10.0', 'MSYS_NT-10.0', 'CYGWIN_NT-10.0', 'Windows_NT', 'FreeBSD'):
        reject(['--skip-gateway-smoke'], 'native Linux', host)
    reject(['--skip-gateway-smoke'], 'native Linux', 'Linux', '1')

    # Evaluate the source's complete pre-build path, then its unchanged Linux
    # argument construction and environment dispatch. No installer body executes.
    boundary = 'if ! command -v cargo >/dev/null 2>&1; then\n'
    assert source.count(boundary) == 1
    prefix = source.split(boundary)[0]
    begin = '\t    provision_status=0\n'
    end = '        if [[ "${provision_status}" -ne 0 ]]; then\n'
    assert source.count(begin) == source.count(end) == 1
    forwarding = source.split(begin)[1].split(end)[0]
    forward_script = root / 'forward.sh'
    forward_script.write_text(prefix + '\nPROVISION_SCRIPT="${TEST_PROVISION}"\n'
                              'use_noninteractive_world_provision="${TEST_NONINTERACTIVE}"\n'
                              'provision_status=0\n' + forwarding + '\nexit "${provision_status}"\n')
    selected_prefix = str(root / 'selected prefix')
    account = 'deferral-fixture'
    uid = '12345'
    def b64(value):
        return base64.urlsafe_b64encode(value.encode()).rstrip(b'=').decode()
    frame = ('domain=substrate.install_bootstrap_context\nversion=1\n'
             f'selected_host_prefix={b64(selected_prefix)}\nhost_substrate_home={b64(selected_prefix)}\n'
             f'host_substrate_root={b64(selected_prefix)}\nprincipal_kind=unix\n'
             f'principal_account={b64(account)}\nprincipal_uid={uid}\n')
    commitment = hashlib.sha256(frame.encode()).hexdigest()
    carrier = b64(frame + f'host_context_commitment={commitment}\n')
    expected_context = {'SUBSTRATE_HOME': selected_prefix, 'SUBSTRATE_ROOT': selected_prefix,
                        'SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT': commitment,
                        'SUBSTRATE_INSTALL_PRIMARY_USER': account,
                        'SUBSTRATE_INSTALL_PRIMARY_UID': uid,
                        'SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1': carrier}
    for skip in (False, True):
        for netfilter, noninteractive in ((False, '0'), (True, '1')):
            args = ['--prefix', selected_prefix, '--profile', 'release',
                    '--install-bootstrap-context-v1', carrier]
            if skip:
                args += ['--skip-gateway-smoke']
            if netfilter:
                args += ['--world-netfilter']
            result, calls = run(args, selected=forward_script, extra={
                'TEST_RESOLVE': '1', 'TEST_CONTEXT_VALUES': json.dumps([
                    selected_prefix, carrier, commitment, account, uid, str(root / 'account-home')]),
                'TEST_PROVISION': str(stub / 'provision'), 'TEST_NONINTERACTIVE': noninteractive})
            assert result.returncode == 0, result
            assert [call['command'] for call in calls] == ['python3', 'provision'], calls
            assert calls[0]['args'] == ['-', '1', selected_prefix, carrier], calls[0]
            expected_args = ['--home', selected_prefix, '--install-bootstrap-context-v1', carrier,
                             '--profile', 'release', '--skip-build']
            if skip:
                expected_args += ['--skip-gateway-smoke']
            if netfilter:
                expected_args += ['--world-netfilter', '--sudo-noninteractive']
            assert calls[1]['args'] == expected_args, calls[1]
            assert calls[1]['context'] == expected_context, calls[1]
            print('PASS exact args/bootstrap forwarding:', 'deferred' if skip else 'default',
                  'netfilter/noninteractive' if netfilter else 'ordinary')
print('PASS dev installer gateway smoke deferral focused tests')
PY
