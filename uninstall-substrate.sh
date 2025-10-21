#!/usr/bin/env bash
set -euo pipefail

log() { printf '[substrate-uninstall] %s\n' "$1"; }

log "Stopping substrate processes (if any)..."
pgrep -fl substrate || true
pkill -f substrate || true

log "Removing PATH/BASH_ENV snippets..."
python3 - <<'PY'
import pathlib
home = pathlib.Path.home()
marker = 'substrate installer'
for rc_name in ['.zshrc', '.bashrc', '.bash_profile']:
    path = home / rc_name
    if not path.exists():
        continue
    lines = path.read_text().splitlines()
    new_lines = []
    skip = False
    for line in lines:
        if marker in line:
            skip = True
            continue
        if skip:
            if line.strip() == 'fi':
                skip = False
                continue
            if 'substrate_bashenv' in line or 'BASH_ENV' in line:
                continue
            continue
        if 'substrate_bashenv' in line:
            continue
        new_lines.append(line)
    path.write_text('\n'.join(new_lines) + ('\n' if new_lines else ''))
PY

log "Removing substrate directories..."
python3 - <<'PY'
import pathlib, shutil
home = pathlib.Path.home()
for target in [
    '.substrate',
    '.substrate_bashenv',
    '.substrate_bashenv_trampoline',
    '.substrate_preexec',
    '.substrate_history',
    '.substrate.lock',
]:
    path = home / target
    if path.is_dir():
        shutil.rmtree(path, ignore_errors=True)
    elif path.exists():
        path.unlink()
PY

log "Removing Lima VM..."
if command -v limactl >/dev/null 2>&1 && limactl list 2>/dev/null | grep -q substrate; then
  limactl stop substrate || true
  limactl delete substrate || true
fi

log "Checking for host symlinks..."
ls -l /usr/local/bin 2>/dev/null | grep substrate || true
ls -l "$HOME/bin" 2>/dev/null | grep substrate || true

log "Clearing shell command cache..."
hash -r || true

log "Done. Open a new shell to pick up changes."
