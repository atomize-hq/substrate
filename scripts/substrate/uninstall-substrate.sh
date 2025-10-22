#!/usr/bin/env bash
set -euo pipefail

log() { printf '[substrate-uninstall] %s\n' "$1"; }

maybe_sudo() {
  if [[ ${EUID} -eq 0 ]]; then
    "$@"
    return
  fi

  if command -v sudo >/dev/null 2>&1; then
    sudo -n "$@"
    local status=$?
    if [[ ${status} -eq 0 ]]; then
      return
    fi
    if [[ ${status} -eq 1 ]]; then
      log "sudo password required for '$*'; rerun uninstall with sudo to complete this step."
    else
      log "sudo failed running '$*' (exit ${status})."
    fi
    return ${status}
  fi

  log "sudo not available; attempting '$*' without elevation"
  "$@"
}

run_python() {
  local clean_path
  clean_path="${SHIM_ORIGINAL_PATH:-/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin}"
  env -i PATH="${clean_path}" HOME="${HOME}" python3 "$@"
}

log "Stopping substrate processes (if any)..."
pgrep -fl substrate || true
pkill -x substrate || true
pkill -f '/substrate/bin/substrate-shim' || true
pkill -f '/substrate-forwarder' || true
pkill -f '/substrate-world-agent' || true

log "Removing PATH/BASH_ENV snippets..."
run_python - <<'PY'
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
run_python - <<'PY'
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

if command -v systemctl >/dev/null 2>&1; then
    log "Stopping substrate-world-agent service..."
    maybe_sudo systemctl stop substrate-world-agent.service 2>/dev/null || true
    maybe_sudo systemctl disable substrate-world-agent.service 2>/dev/null || true

    log "Removing systemd unit + runtime directories..."
    maybe_sudo rm -f /etc/systemd/system/substrate-world-agent.service || true
    maybe_sudo rm -rf /var/lib/substrate || true
    maybe_sudo rm -rf /run/substrate || true
    maybe_sudo systemctl daemon-reload 2>/dev/null || true
fi

log "Removing world-agent binary from /usr/local/bin (if present)..."
maybe_sudo rm -f /usr/local/bin/substrate-world-agent || true

if command -v limactl >/dev/null 2>&1; then
  # Only relevant on macOS hosts where Lima is installed.
  if [[ "$(uname -s)" == "Darwin" ]]; then
    log "Removing Lima VM..."
    if limactl list 2>/dev/null | grep -q substrate; then
      limactl stop substrate || true
      limactl delete substrate || true
    fi
  fi
fi

log "Checking for host symlinks..."
ls -l /usr/local/bin 2>/dev/null | grep substrate || true
ls -l "$HOME/bin" 2>/dev/null | grep substrate || true

log "Clearing shell command cache..."
hash -r || true

log "Done. Open a new shell to pick up changes."
